"""The graphics device the sketchbook renderer draws its page on.

This module holds every call to the graphics library. It opens a context,
builds a program from one fragment shader, holds the textures the shader
reads, draws into a frame buffer of its own, and reads the result back into
the pixels the caller owns.[^1]

**The renderer never draws on the screen.** It draws into a frame buffer this
module owns and reads the result back, so one path serves a window, a picture
on a disk and a test with no display. A path that drew on the screen would
leave the picture flag and the tests on a second path, and a second path
decays.[^2]

**The context is opened once, on the first frame, and never in the
constructor.** The demonstration builds the renderer before it opens its
window, so a context opened in the constructor would be a second context.

Why a whole frame in one pass
-----------------------------

The page composites about one and a quarter million pixels through a dozen
stages. An array library builds one whole array for each stage, so the frame
costs hundreds of megabytes of memory traffic. A fragment shader runs the
whole chain for one pixel in registers, and it runs many pixels at once. That
is the difference this module buys.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0067, the viewer reads the world and never writes to it, decision D3.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

import ctypes
import os
from typing import TYPE_CHECKING, Any

import numpy as np

if TYPE_CHECKING:
    import numpy.typing as npt

# The name of the setting that makes this module open a context with no
# display. A machine with a display still has none inside a test runner that
# a build server started, so the setting exists to drive the path a server
# takes from a machine that has a screen.
HEADLESS_SETTING = "CACHETTE_HEADLESS_GL"

# The version of the shading language the shaders declare. The page needs
# integer textures and texel fetches, which arrived in this version.
GLSL_VERSION = "#version 330 core"

# The two triangles that cover the whole target, in clip space. The page is
# one pass over every pixel, so the geometry is a rectangle and nothing else.
SCREEN = np.array(
    [-1.0, -1.0, 3.0, -1.0, -1.0, 3.0],
    dtype=np.float32,
)

# The vertex shader every pass shares. It spreads one triangle over the whole
# target and carries nothing to the fragment shader, because each fragment
# reads its own place from the built-in coordinate.
VERTEX_SOURCE = f"""{GLSL_VERSION}
in vec2 place;
void main() {{
    gl_Position = vec4(place, 0.0, 1.0);
}}
"""


# The texture unit this module builds and fills a texture on.
#
# **A call that makes or fills a texture must bind that texture somewhere.** It
# binds on the unit that is current. A call that used the unit a shader reads
# would take that texture away from the shader, and nothing would report it:
# the shader would read the wrong texture and draw a picture that looks nearly
# right. This unit is therefore kept for that work, and no shader reads it.
# The graphics library promises sixteen units, so this is the last one.
SCRATCH_UNIT = 15


class DeviceGap(Exception):
    """The machine cannot give this module a graphics context."""


def _wanted_headless() -> bool:
    """Say whether the caller asked for a context with no display."""
    return os.environ.get(HEADLESS_SETTING, "").strip() not in ("", "0", "false")


class Device:
    """One graphics context, and the frame buffer the page draws into.

    The context belongs to this object when it opened one, and to the
    application when the application had already opened a window. Either way
    the frame buffer, the textures and the programs are this object's.
    """

    __slots__ = (
        "_borrowed",
        "_buffers",
        "_colour",
        "_frame",
        "_programs",
        "_size",
        "_textures",
        "_vertices",
        "_window",
        "gl",
        "renderer",
    )

    def __init__(self) -> None:
        """Open a context, or take the one the application already opened.

        Raises ``DeviceGap`` when the machine gives no context at all. The
        caller then has no graphics path, and it must say so rather than fall
        back quietly to a slower one.
        """
        import pyglet
        from pyglet import gl

        self.gl = gl
        self._borrowed = False
        self._window: Any = None
        existing = [held for held in pyglet.app.windows]
        if existing and not _wanted_headless():
            # The application opened a window. Its context is the one to draw
            # in, because a second context would hold a second copy of every
            # texture and would have to be made current against the first.
            self._window = existing[0]
            self._borrowed = True
        else:
            self._window = self._open(pyglet)
        self._window.switch_to()
        self.renderer = self._string(gl.GL_RENDERER)
        self._size = (0, 0)
        self._frame = gl.GLuint(0)
        self._colour = gl.GLuint(0)
        self._textures: dict[str, tuple[Any, int, int, str]] = {}
        self._programs: dict[str, Any] = {}
        self._buffers: dict[str, Any] = {}
        self._vertices = None

    @staticmethod
    def _open(pyglet: Any) -> Any:
        """Open a hidden window of this module's own, on a display or without.

        A machine with a display gives a hidden window. A machine without one
        gives a context through the display-free path the graphics library
        offers. The second attempt runs only when the first fails, so a run
        with a screen never turns the display-free path on for the process.
        """
        failures = []
        if not _wanted_headless():
            try:
                return pyglet.window.Window(width=16, height=16, visible=False)
            except Exception as refusal:
                failures.append(f"a hidden window: {refusal}")
        try:
            pyglet.options["headless"] = True
            return pyglet.window.Window(width=16, height=16, visible=False)
        except Exception as refusal:
            failures.append(f"a display-free context: {refusal}")
        message = "the machine gives no graphics context (" + "; ".join(failures) + ")"
        raise DeviceGap(message)

    def _string(self, name: int) -> str:
        """Give back one of the strings the graphics library reports."""
        found = self.gl.glGetString(name)
        if not found:
            return "unknown"
        return str(ctypes.cast(found, ctypes.c_char_p).value or b"", "ascii")

    def make_current(self) -> None:
        """Make this context the one the calls below act on."""
        self._window.switch_to()

    # ------------------------------------------------------------------
    # Programs

    def program(self, name: str, fragment: str) -> Any:
        """Build the program for this fragment shader, once for each name."""
        held = self._programs.get(name)
        if held is not None:
            return held
        from pyglet.graphics.shader import Shader, ShaderProgram

        built = ShaderProgram(
            Shader(VERTEX_SOURCE, "vertex"),
            Shader(fragment, "fragment"),
        )
        self._programs[name] = built
        return built

    def _screen(self, program: Any) -> Any:
        """Give back the rectangle that covers the target, built once."""
        held = self._buffers.get("screen")
        if held is None:
            held = program.vertex_list(3, self.gl.GL_TRIANGLES, place=("f", SCREEN))
            self._buffers["screen"] = held
        return held

    # ------------------------------------------------------------------
    # Textures

    def _named(self, name: str, width: int, height: int, layout: str) -> Any:
        """Give back the texture of this name, rebuilding it on a new size."""
        gl = self.gl
        held = self._textures.get(name)
        if held is not None and held[1:] == (width, height, layout):
            return held[0]
        if held is not None:
            gl.glDeleteTextures(1, ctypes.byref(held[0]))
        handle = gl.GLuint(0)
        gl.glGenTextures(1, ctypes.byref(handle))
        gl.glActiveTexture(gl.GL_TEXTURE0 + SCRATCH_UNIT)
        gl.glBindTexture(gl.GL_TEXTURE_2D, handle)
        inner, _, _ = LAYOUTS[layout]
        gl.glTexStorage2D(gl.GL_TEXTURE_2D, 1, inner, width, height)
        # Every read is a texel fetch at a whole coordinate, so no filter and
        # no wrap ever runs. They are set anyway, because a driver refuses a
        # texture with no mip levels and the default filter.
        gl.glTexParameteri(gl.GL_TEXTURE_2D, gl.GL_TEXTURE_MIN_FILTER, gl.GL_NEAREST)
        gl.glTexParameteri(gl.GL_TEXTURE_2D, gl.GL_TEXTURE_MAG_FILTER, gl.GL_NEAREST)
        gl.glTexParameteri(gl.GL_TEXTURE_2D, gl.GL_TEXTURE_WRAP_S, gl.GL_CLAMP_TO_EDGE)
        gl.glTexParameteri(gl.GL_TEXTURE_2D, gl.GL_TEXTURE_WRAP_T, gl.GL_CLAMP_TO_EDGE)
        self._textures[name] = (handle, width, height, layout)
        return handle

    def upload(self, name: str, field: npt.NDArray[Any], layout: str) -> None:
        """Put a two-dimensional field into the texture of this name.

        The field arrives with its first row first, and it stays that way.
        Every read is a texel fetch at a whole coordinate, so the row order of
        the page and the row order of the texture are the same order.
        """
        gl = self.gl
        _, form, kind = LAYOUTS[layout]
        bands = LAYOUT_BANDS[layout]
        rows, columns = field.shape[0], field.shape[1]
        handle = self._named(name, columns, rows, layout)
        block = np.ascontiguousarray(field, dtype=LAYOUT_TYPES[layout])
        gl.glActiveTexture(gl.GL_TEXTURE0 + SCRATCH_UNIT)
        gl.glBindTexture(gl.GL_TEXTURE_2D, handle)
        gl.glPixelStorei(gl.GL_UNPACK_ALIGNMENT, 1)
        gl.glTexSubImage2D(
            gl.GL_TEXTURE_2D,
            0,
            0,
            0,
            columns,
            rows,
            form,
            kind,
            block.ctypes.data_as(ctypes.c_void_p),
        )
        del bands

    def bind(self, program: Any, name: str, unit: int, texture: str = "") -> None:
        """Put a texture on this unit, and point the named sampler at it.

        The sampler and the texture share a name unless the caller gives one,
        because a pass that blurs its own last result reads a texture whose
        name is not the name of the sampler that reads it.
        """
        gl = self.gl
        if unit >= SCRATCH_UNIT:
            message = (
                f"unit {unit} is the unit this module builds a texture on, and "
                f"a shader that read it would read whatever was built last"
            )
            raise DeviceGap(message)
        held = self._textures.get(texture or name)
        if held is None:
            message = f"no texture named {texture or name!r} was uploaded"
            raise DeviceGap(message)
        gl.glActiveTexture(gl.GL_TEXTURE0 + unit)
        gl.glBindTexture(gl.GL_TEXTURE_2D, held[0])
        self.put(program, name, unit)

    @staticmethod
    def put(program: Any, name: str, value: Any) -> None:
        """Give a value to a uniform, and pass over one the driver removed.

        **Every pass here is built from the same shared blocks.** A block
        declares what it needs, and a pass that includes it may read none of
        that. The compiler then removes the uniform, and the program reports
        that it holds no such name. That is the normal outcome of sharing the
        blocks, and it is not a mistake to report.

        A misspelt name passes through here without a word. The test that
        compares this renderer against the array renderer is what catches
        that, because a uniform that never arrives changes the picture.

        The graphics library raises its own error outside the ordinary tree of
        errors, so this names that error rather than the ordinary one.
        """
        from pyglet.graphics.shader import ShaderException

        try:
            program[name] = value
        except (ShaderException, KeyError):
            return

    def has(self, name: str) -> bool:
        """Say whether a texture of this name was uploaded."""
        return name in self._textures

    def ensure(self, name: str, layout: str, bands: int = 1) -> None:
        """Make a texture of one point, so a sampler that reads none is bound.

        A sampler that names no texture is not defined, even where no branch
        of the shader reads it. This gives every such sampler something real
        and small to name.
        """
        if self.has(name):
            return
        shape = (1, 1) if bands == 1 else (1, 1, bands)
        self.upload(name, np.zeros(shape, dtype=LAYOUT_TYPES[layout]), layout)

    # ------------------------------------------------------------------
    # Targets

    def _target(self, width: int, height: int, layout: str) -> None:
        """Point the frame buffer at a colour texture of this size."""
        gl = self.gl
        if not self._frame.value:
            gl.glGenFramebuffers(1, ctypes.byref(self._frame))
        handle = self._named(f"target:{layout}", width, height, layout)
        gl.glBindFramebuffer(gl.GL_FRAMEBUFFER, self._frame)
        gl.glFramebufferTexture2D(
            gl.GL_FRAMEBUFFER,
            gl.GL_COLOR_ATTACHMENT0,
            gl.GL_TEXTURE_2D,
            handle,
            0,
        )
        state = gl.glCheckFramebufferStatus(gl.GL_FRAMEBUFFER)
        if state != gl.GL_FRAMEBUFFER_COMPLETE:
            message = f"the frame buffer is not usable: state {state}"
            raise DeviceGap(message)
        gl.glViewport(0, 0, width, height)

    def run(
        self,
        program: Any,
        width: int,
        height: int,
        layout: str = "rgba8",
    ) -> None:
        """Run one pass over every pixel of a target of this size."""
        gl = self.gl
        self._target(width, height, layout)
        gl.glDisable(gl.GL_DEPTH_TEST)
        gl.glDisable(gl.GL_BLEND)
        program.use()
        self._screen(program).draw(gl.GL_TRIANGLES)
        program.stop()

    def keep(self, name: str, layout: str) -> None:
        """Keep the last target under this name, so a later pass reads it.

        The pass wrote into the texture the frame buffer points at. This gives
        that texture a name of its own, and takes the name the frame buffer
        holds off it, so the next pass on a target of the same shape does not
        write over what this one produced.
        """
        held = self._textures.pop(f"target:{layout}")
        was = self._textures.get(name)
        if was is not None:
            self.gl.glDeleteTextures(1, ctypes.byref(was[0]))
        self._textures[name] = held

    def read(self, width: int, height: int) -> npt.NDArray[np.uint8]:
        """Read the last target back, as one row of red, green, blue, opacity.

        The graphics library numbers the rows of a target from the bottom, and
        it reads them back in that order. The shader therefore writes the
        first row of the frame at the bottom, so the bytes arrive in the order
        the caller's array holds them.
        """
        gl = self.gl
        into = np.empty((height, width, 4), dtype=np.uint8)
        gl.glPixelStorei(gl.GL_PACK_ALIGNMENT, 1)
        gl.glReadPixels(
            0,
            0,
            width,
            height,
            gl.GL_RGBA,
            gl.GL_UNSIGNED_BYTE,
            into.ctypes.data_as(ctypes.c_void_p),
        )
        return into

    def close(self) -> None:
        """Give the context back, and close the one this module opened."""
        if self._window is not None and not self._borrowed:
            self._window.close()
        self._window = None


# How each kind of field is stored, given as the storage form, the band order
# and the number type. One name here is one kind of field the page uploads.
def _layouts() -> tuple[
    dict[str, tuple[int, int, int]], dict[str, Any], dict[str, int]
]:
    """Give back the three tables that describe every texture layout.

    The tables name constants of the graphics library, so they are built
    inside a function and the module imports nothing at the top level that a
    machine with no graphics library would refuse.
    """
    from pyglet import gl

    forms = {
        "r32f": (gl.GL_R32F, gl.GL_RED, gl.GL_FLOAT),
        "rg32f": (gl.GL_RG32F, gl.GL_RG, gl.GL_FLOAT),
        "rgb32f": (gl.GL_RGB32F, gl.GL_RGB, gl.GL_FLOAT),
        "r32i": (gl.GL_R32I, gl.GL_RED_INTEGER, gl.GL_INT),
        "r8ui": (gl.GL_R8UI, gl.GL_RED_INTEGER, gl.GL_UNSIGNED_BYTE),
        "rgba8": (gl.GL_RGBA8, gl.GL_RGBA, gl.GL_UNSIGNED_BYTE),
    }
    types = {
        "r32f": np.float32,
        "rg32f": np.float32,
        "rgb32f": np.float32,
        "r32i": np.int32,
        "r8ui": np.uint8,
        "rgba8": np.uint8,
    }
    bands = {"r32f": 1, "rg32f": 2, "rgb32f": 3, "r32i": 1, "r8ui": 1, "rgba8": 4}
    return forms, types, bands


class _Late:
    """A table that builds itself the first time anything reads it.

    The tables name constants of the graphics library. A machine that has no
    graphics library must still import this module, because the renderer
    reports the gap rather than failing at the import.
    """

    __slots__ = ("_at", "_held")

    def __init__(self, at: int) -> None:
        self._at = at
        self._held: Any = None

    def __getitem__(self, name: str) -> Any:
        if self._held is None:
            self._held = _layouts()[self._at]
        return self._held[name]


LAYOUTS = _Late(0)
LAYOUT_TYPES = _Late(1)
LAYOUT_BANDS = _Late(2)
