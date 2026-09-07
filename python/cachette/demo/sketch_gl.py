"""The sketchbook renderer, composited on the graphics device.

This renderer draws the same page as the array renderer. It takes the same
call, from the same place, and it gives back the same reading.[^1] The
difference is where the pixels are made.

**The array renderer builds one whole array for each stage of the page.** The
page is over a million points, it composites in three bands of a real number,
and the array library keeps no stage in a register, so a dozen chained stages
cost hundreds of megabytes of memory traffic for one frame. The device runs
the whole chain for one pixel at a time, many pixels at once, and writes the
result once.

How the terrain reaches the page
--------------------------------

**The terrain is a mesh, and the hardware draws it.** The height of every tile
crosses to the device once, as vertex data, because the ground never moves. A
turn of the view, a lean, a zoom or a move of the camera then costs a handful
of uniform values rather than a new page.

**The depth test resolves the occlusion.** Each tile carries a top face at its
own height and a face below it that reaches down to where the ground stood.
The depth of a vertex is the row of the flat page, which grows towards the
watcher, so the nearest ground wins and what stands behind a ridge stays
hidden. Nothing sorts, nothing scans a column, and no array holds a page.

**The marks are one pass over the pixels.** The tone of the light, every set
of hatching, the silhouette, the grain of the paper, the wash of a faction,
the cloud, the wash of an overlay and the fit of the page into the frame are
all functions of one point. They are the whole of the rest of the cost, and
they are what a fragment shader is for.

What this renderer promises
---------------------------

**It draws what the array renderer draws.** The arithmetic is the same
arithmetic and the constants are the same constants: this module takes every
number from the module that declares it and puts it into the shader, so the
two cannot drift apart on a value. A test draws one frame each way and holds
the difference under a stated bound.

**It writes nothing to the world.** It reads the fields the engine publishes
and it fills the pixels the caller owns. It touches no simulated value, no
state hash and no step.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0067, the viewer reads the world and never writes to it, decision D3.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING, Any

import numpy as np

from cachette.demo import sketch as ink
from cachette.demo import sketch_shader as source
from cachette.demo.glpage import Device, DeviceGap
from cachette.demo.sketch import Projection, Sketch

if TYPE_CHECKING:
    from collections.abc import Sequence

    import numpy.typing as npt

    from cachette import World
    from cachette._core import Camera, FrameReading
    from cachette.demo.view import View

__all__ = ["DeviceGap", "GlSketch"]

# How many faction colours the shader holds room for.
#
# **The engine publishes the colours and this only bounds them.** A run with
# more factions than this is refused rather than drawn with the wrong colours,
# because a picture that names the wrong holder is worse than no picture.
PALETTE_ROOM = 64

# The two reaches of the blur that finds the rim of a wash, in page points.
# They are the reaches the array renderer uses, and one test holds them equal.
NEAR_REACH = 4
FAR_REACH = 14

# The bits of the flag byte the page carries for each of its points.
DRAWN_BIT = 1
CLIFF_BIT = 2
WATER_BIT = 4


def _defines() -> str:
    """Give back every constant of the page, as a shader definition.

    **Each number is taken from the module that declares it.** A number typed
    again in the shader would be a second declaration site, and nothing would
    fail when the two disagreed.
    """
    numbers = {
        "CONTOUR_STEP": ink.CONTOUR_STEP,
        "CONTOUR_WEIGHT": ink.CONTOUR_WEIGHT,
        "HATCH_SPACING": ink.HATCH_SPACING,
        "CLIFF_SPACING": ink.CLIFF_SPACING,
        "WATER_SPACING": ink.WATER_SPACING,
        "CLOUD_SPACING": ink.CLOUD_SPACING,
        "SHADOW_START": ink.SHADOW_START,
        "CROSS_START": ink.CROSS_START,
        "CLOUD_FLOOR": ink.CLOUD_FLOOR,
        "CLOUD_SHADOW_DEPTH": ink.CLOUD_SHADOW_DEPTH,
        "HOLDER_WASH": ink.HOLDER_WASH,
        "WASH_GAIN": ink.WASH_GAIN,
        "GRANULATION": ink.GRANULATION,
        "RIM_GAIN": ink.RIM_GAIN,
        "GLAZE_LIFT": ink.GLAZE_LIFT,
    }
    colours = {"PAPER": ink.PAPER, "INK": ink.INK, "SKY_INK": ink.SKY_INK}
    whole = {
        "PALETTE_ROOM": PALETTE_ROOM,
        "NEAR_REACH": NEAR_REACH,
        "FAR_REACH": FAR_REACH,
    }
    lines = [f"#define {name} {value!r}" for name, value in whole.items()]
    lines += [
        f"const float {name} = {float(value)!r};" for name, value in numbers.items()
    ]
    lines += [
        f"const vec3 {name} = vec3({float(band[0])!r}, {float(band[1])!r}, "
        f"{float(band[2])!r});"
        for name, band in colours.items()
    ]
    return "\n".join(lines) + "\n"


# The four corners of the square of one tile, in tiles.
#
# **The square is what the array renderer draws.** That renderer names the
# tile nearest to each point of the page, so one tile covers the square around
# its own place. The mesh draws that square, and the two therefore cover the
# same points.
SQUARE = np.array(
    [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]], dtype=np.float32
)

# How many blocks of four corners one tile carries, and how many vertices
# that makes.
#
# The first block is the top face, at the height of the ground. The second is
# the square where the ground stood, which is where the face below the tile
# reaches down to.
BLOCK_COUNT = 2
VERTEX_COUNT = BLOCK_COUNT * 4

# Whether each vertex of a tile stands where the ground stood.
#
# **This answer also says whether a triangle is a face below the top.** The
# shading language takes a flat value from the last vertex of a triangle, and
# every triangle below the top face ends on a vertex of the lower square. One
# attribute therefore carries both answers, and the mesh holds eight vertices
# for a tile rather than twelve.
SKIRT = np.array([0.0] * 4 + [1.0] * 4, dtype=np.float32)


def _triangles() -> np.ndarray:
    """Give back the triangles of one tile, as places in its own vertices.

    The tile draws its top face, the square where the ground stood, and the
    four faces between the two. Together they cover every point of the paper
    that the array renderer fills from this tile.

    **Every triangle below the top face ends on the lower square.** The
    shading language takes a flat value from the last vertex, so that ending
    is what tells the fragment stage that the point is a face and not a top.
    """
    made = [[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]]
    for edge in range(4):
        near = edge
        far = (edge + 1) % 4
        made.append([near, far, 4 + far])
        made.append([near, 4 + far, 4 + near])
    return np.array(made, dtype=np.int32).ravel()


TRIANGLES = _triangles()

# The name the device holds the terrain under.
TERRAIN = "terrain"


class Page:
    """The paper one camera and one pair of angles ask for.

    **This holds no picture.** The terrain is on the device, and the mesh pass
    draws it against the projection below. This says where the window of tiles
    falls on the paper, how large the paper is, and which part of it holds the
    ground.
    """

    __slots__ = ("box", "sample", "shape", "stand", "stood")

    stood: Projection
    shape: tuple[int, int]
    stand: tuple[float, float]
    box: tuple[int, int, int, int]
    sample: tuple[np.ndarray, np.ndarray] | None

    def __init__(
        self,
        stood: Projection,
        shape: tuple[int, int],
        stand: tuple[float, float],
        box: tuple[int, int, int, int],
    ) -> None:
        """Record the projection, the frame, the angles and the part in use."""
        self.stood = stood
        self.shape = shape
        self.stand = stand
        self.box = box
        # Where the engine drew each tile on its own flat map. It is built
        # only when a wash needs it.
        self.sample = None

    @property
    def window(self) -> tuple[int, int, int, int]:
        """Give back the window of tiles this paper covers."""
        return self.stood.window


class GlSketch(Sketch):
    """The sketchbook renderer, composited on the graphics device.

    A caller builds one of these exactly as it builds the array renderer, and
    calls it exactly as it calls the drawing method of the world.

    **The device opens on the first frame, not here.** The demonstration builds
    the renderer before it opens its window, and a context opened here would be
    a second context that the window never draws in.

    Raises ``DeviceGap`` on the first frame when the machine gives no graphics
    context. It never falls back to the array renderer on its own: a renderer
    that quietly draws on the processor would report the speed of the processor
    while everyone believed the device was drawing.
    """

    # **The build of the array renderer never runs here.** This renderer draws
    # the terrain as a mesh and it draws every mark itself, so it needs
    # neither the turn nor the lift that the array renderer works out.
    _marks = False

    __slots__ = ("_device", "_held", "_pages", "_palette", "_stamp")

    def __init__(
        self,
        world: World,
        *,
        view: View | None = None,
        relief: float = ink.RELIEF,
        sky: bool = True,
    ) -> None:
        """Build the renderer for one world.

        **The view says where the watcher stands, and this holds no angle of
        its own.** The turn and the lean are read from it on every frame, so a
        mouse that moves the view moves the page.
        """
        super().__init__(world, view=view, relief=relief, sky=sky)
        self._device: Device | None = None
        # The paper the last frame drew on, held so that a frame which asks
        # for the same one gets it back rather than a second copy.
        self._held: Page | None = None
        # The paper the device last drew the terrain onto. Holding it lets a
        # frame that reuses one paper draw the mesh again for nothing.
        self._stamp: Page | None = None
        self._pages: dict[str, Any] = {}
        self._palette: np.ndarray | None = None

    # ------------------------------------------------------------------
    # The device

    @property
    def device(self) -> Device:
        """Give back the device, opening it on the first ask."""
        if self._device is None:
            self._device = Device()
        return self._device

    def close(self) -> None:
        """Give the graphics context back."""
        if self._device is not None:
            self._device.close()
            self._device = None
            self._stamp = None
            self._held = None

    def _program(self, name: str, body: str) -> Any:
        """Build one program from the shared head and this body."""
        return self.device.program(
            name, source.HEAD + _defines() + source.PAGE + source.TILES + body
        )

    def _geometry(self) -> Any:
        """Build the program that draws the terrain, once."""
        return self.device.program(
            "geometry", source.GEOMETRY_FRAGMENT, vertex=source.GEOMETRY_VERTEX
        )

    # ------------------------------------------------------------------
    # The page

    def _page_for(self, camera: Camera, width: int, height: int) -> Page:
        """Give back the paper the camera and the two angles ask for.

        **Nothing is built here.** The paper is a handful of numbers: where
        the window of tiles falls on it, how large it is, and which part of it
        holds the ground. The terrain is already on the device, and the mesh
        pass draws it against these numbers.
        """
        window = self._window(camera, width, height)
        stand = (self.view.turn, self.view.lean)
        held = self._held
        if (
            held is not None
            and held.window == window
            and held.shape == (height, width)
            and held.stand == stand
        ):
            return held
        stood = self.projection(window, width, height)
        built = Page(stood, (height, width), stand, self.box_of(stood))
        self._held = built
        return built

    def window(self) -> tuple[int, int, int, int] | None:
        """Give back the window of tiles the last frame drew, or nothing."""
        return None if self._held is None else self._held.window

    # ------------------------------------------------------------------
    # What crosses to the device

    def _send_mesh(self) -> Any:
        """Put the terrain on the device, as a mesh, on the first frame.

        **The tile heights never change, so they cross once.** One tile gives
        twelve vertices: the square of the tile at the height of the ground,
        the same square again for the faces below it, and the square where the
        ground stood. A turn of the view then costs a uniform value.
        """
        # **The device holds the mesh, and it is the only thing that holds
        # it.** A second cache here would be a second declaration of one fact,
        # and a frame would take the older of the two without anything
        # failing.
        program = self._geometry()
        if self.device.has_mesh(TERRAIN):
            return self.device.mesh(TERRAIN, program, 0, ())
        rows, columns = self._world.height, self._world.width
        grid_q, grid_r = np.meshgrid(
            np.arange(columns, dtype=np.float32), np.arange(rows, dtype=np.float32)
        )
        tiles = rows * columns
        spread = np.repeat(
            np.stack([grid_q.ravel(), grid_r.ravel()], axis=-1), VERTEX_COUNT, axis=0
        )
        corners = np.tile(SQUARE, (tiles * BLOCK_COUNT, 1))
        step = np.arange(tiles, dtype=np.int32)[:, None] * VERTEX_COUNT
        indices = (TRIANGLES[None, :] + step).ravel()
        return self.device.mesh(
            TERRAIN,
            program,
            tiles * VERTEX_COUNT,
            indices.tolist(),
            tile=("f", spread.ravel().tolist()),
            corner=("f", corners.ravel().tolist()),
            raised=("f", np.repeat(self._raised.ravel(), VERTEX_COUNT).tolist()),
            deep=("f", np.repeat(self._deep.ravel(), VERTEX_COUNT).tolist()),
            wet=(
                "f",
                np.repeat(
                    self._water.ravel().astype(np.float32), VERTEX_COUNT
                ).tolist(),
            ),
            skirt=("f", np.tile(SKIRT, tiles).tolist()),
        )

    def _draw_page(self, page: Page) -> None:
        """Draw the terrain onto the paper, and let the depth test resolve it.

        **This is the whole of the geometry.** The vertex stage turns the
        world, leans it, and lifts every tile by the height of the ground on
        it. The depth test keeps the nearest surface at each point of the
        paper. The result is one texture that holds the height, the depth of
        the water, the flags and the tile of the ground a watcher sees.
        """
        # **The test is the paper itself, not a description of it.** A new
        # paper comes back whenever the window, the frame or either angle
        # moves, so holding the last one and comparing it by identity cannot
        # miss a change that a list of properties would forget to name.
        if self._stamp is page:
            return
        device = self.device
        mesh = self._send_mesh()
        program = self._geometry()
        stood = page.stood
        first_q, first_r, last_q, last_r = stood.window
        program.use()
        device.put(program, "first_tile", (float(first_q), float(first_r)))
        device.put(program, "plan_half", (stood.plan_wide / 2.0, stood.plan_tall / 2.0))
        device.put(program, "turn_by", (math.cos(stood.turn), math.sin(stood.turn)))
        device.put(program, "scale", float(stood.scale))
        device.put(program, "lean", float(stood.lean))
        device.put(program, "page_middle", (stood.cols / 2.0, stood.flat_rows / 2.0))
        device.put(program, "page_span", (float(stood.cols), float(stood.rows)))
        device.put(program, "rise", float(stood.rise))
        device.put(program, "lift_margin", float(ink.LIFT_MARGIN))
        device.put(program, "row_pitch", float(ink.ROW_PITCH))
        device.put(
            program,
            "window",
            (float(first_q), float(first_r), float(last_q), float(last_r)),
        )
        device.put(program, "world_wide", int(self._world.width))
        program.stop()
        # A point that no triangle covers holds no ground. The tile it shows
        # is below nought, and the shader reads that as nothing.
        device.draw_mesh(
            program, mesh, stood.cols, stood.rows, "rgba32f", (0.0, 0.0, 0.0, -1.0)
        )
        device.keep("page", "rgba32f")
        device.upload("page_grain", self._paper_grain(stood.rows, stood.cols), "r32f")
        self._stamp = page

    def _send_tiles(self) -> None:
        """Put the fields that change with the world on the device.

        **These cross at the size of the world, not of the paper.** A tile
        covers many points of the paper, so the paper reads a tile many times
        and the tile crosses once for each frame.
        """
        from cachette import faction_colours

        device = self.device
        rows, columns = self._world.height, self._world.width

        if self._palette is None:
            self._palette = np.array(
                [
                    [(c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF]
                    for c in faction_colours()
                ],
                dtype=np.float32,
            )
            if not len(self._palette):
                message = "the engine publishes no faction colour"
                raise DeviceGap(message)
            # The colours go across one row, so the shader reads one by its
            # number and this module holds no bound of its own on how many
            # colours the engine publishes.
            device.upload("palette", self._palette[None, :, :], "rgb32f")

        holders = self._world.tile_holders().reshape(rows, columns)
        device.upload("tile_holder", holders.astype(np.int32), "r32i")

        if self._sky:
            cloud = self._world.cloud_shares().reshape(rows, columns).astype(
                np.float32
            ) / max(self._whole_sky, 1.0)
            winds = self._world.tile_winds()
            wind_q = winds["q"].reshape(rows, columns).astype(np.float32)
            wind_r = winds["r"].reshape(rows, columns).astype(np.float32)
            # The wind stands on the axes of the hex grid, and the page turns
            # those axes, so the wind turns with them. The angles come from
            # the view, which is the one place that holds them.
            plan_x = wind_q + wind_r * 0.5
            plan_y = wind_r * ink.ROW_PITCH
            along_turn = math.cos(self.view.turn)
            across_turn = math.sin(self.view.turn)
            page_dx = plan_x * along_turn - plan_y * across_turn
            page_dy = (plan_x * across_turn + plan_y * along_turn) * self.view.lean
            length = np.hypot(page_dx, page_dy)
            moving = length > 0.0
            page_dx = np.where(moving, page_dx / np.where(moving, length, 1.0), 1.0)
            page_dy = np.where(moving, page_dy / np.where(moving, length, 1.0), 0.0)
            device.upload("tile_cloud", cloud, "r32f")
            device.upload("tile_across_x", page_dy.astype(np.float32), "r32f")
            device.upload("tile_across_y", (-page_dx).astype(np.float32), "r32f")
        else:
            # The shader binds these whether it reads them or not.
            for name in ("tile_cloud", "tile_across_x", "tile_across_y"):
                device.ensure(name, "r32f")

    def _send_wash(
        self,
        page: Page,
        overlay: str | None,
        camera: Camera,
        width: int,
        height: int,
    ) -> bool:
        """Put the pigment a named overlay laid down on the device.

        **The colours are the engine's own.** This asks the engine for the
        frame with the overlay and for the frame without it, and the
        difference between the two is the pigment. It then reads that
        difference at the place the engine drew each tile, so the wash follows
        the ground the paper shows rather than the flat map the engine drew.

        Gives back whether there is any pigment at all.
        """
        device = self.device
        device.ensure("tile_hue", "rgb32f", bands=3)
        device.ensure("tile_flow", "r32f")
        if not overlay:
            return False
        bare = self._buffer("bare", width, height)
        tinted = self._buffer("tinted", width, height)
        self._world.draw(camera, width, height, bare)
        self._world.draw(camera, width, height, tinted, overlay=overlay)
        plain = ink._channels(bare, width, height)
        painted = ink._channels(tinted, width, height)
        flow = self._sampled(
            page, np.abs(painted - plain).max(axis=-1) / 255.0, camera, width, height
        )
        if not flow.any():
            return False
        hue = np.stack(
            [
                self._sampled(page, painted[..., band], camera, width, height)
                for band in range(3)
            ],
            axis=-1,
        )
        device.upload("tile_flow", flow.astype(np.float32), "r32f")
        device.upload("tile_hue", hue.astype(np.float32), "rgb32f")
        return True

    def _sampled(
        self, page: Page, frame: np.ndarray, camera: Camera, width: int, height: int
    ) -> np.ndarray:
        """Read a field the engine painted on the flat map, tile by tile.

        The engine paints an overlay on its own map. The paper shows the same
        tiles in another place, so the reading finds where the engine drew
        each tile and then gives that value for that tile.

        **The engine answers where a tile is.** The pass asks it over a coarse
        grid of pixels and takes the middle of the pixels that named each
        tile, so this module holds no layout of its own for the flat map.
        """
        rows, columns = self._world.height, self._world.width
        if page.sample is None:
            found_x = np.zeros((rows, columns), dtype=np.int64)
            found_y = np.zeros((rows, columns), dtype=np.int64)
            counted = np.zeros((rows, columns), dtype=np.int64)
            for y in range(0, height, ink.SAMPLE_STEP):
                for x in range(0, width, ink.SAMPLE_STEP):
                    tile_q, tile_r = camera.tile_at(float(x), float(y))
                    if 0 <= tile_q < columns and 0 <= tile_r < rows:
                        found_x[tile_r, tile_q] += x
                        found_y[tile_r, tile_q] += y
                        counted[tile_r, tile_q] += 1
            safe = np.clip(counted, 1, None)
            page.sample = (
                np.clip(found_x // safe, 0, width - 1).astype(np.int32),
                np.clip(found_y // safe, 0, height - 1).astype(np.int32),
            )
        at_x, at_y = page.sample
        whole: np.ndarray = frame[at_y, at_x]
        return whole

    # ------------------------------------------------------------------
    # The frame

    def __call__(
        self,
        camera: Camera,
        width: int,
        height: int,
        pixels: npt.NDArray[np.uint32],
        reference: bool = False,
        panel: bool = False,
        panels: Sequence[str] | None = None,
        pointer: tuple[int, int] | None = None,
        overlay: str | None = None,
        phase: float = 0.0,
        speed_milli: int = 0,
    ) -> FrameReading:
        """Fill one frame with the sketch, and report what the pass read.

        The arguments are the arguments of the drawing method of the world, so
        a caller swaps one renderer for the other and changes nothing else.
        """
        reading = self._world.draw(
            camera,
            width,
            height,
            pixels,
            reference=reference,
            panel=panel,
            panels=panels,
            pointer=pointer,
            overlay=overlay,
            phase=phase,
            speed_milli=speed_milli,
        )
        page = self._page_for(camera, width, height)
        kept = self._panels(pixels, camera, width, height, overlay, phase)
        drawn = self._composite(page, overlay, camera, width, height)
        frame = pixels.reshape(height, width)
        frame[...] = np.where(kept, frame, drawn)
        return reading

    def _composite(
        self,
        page: Page,
        overlay: str | None,
        camera: Camera,
        width: int,
        height: int,
    ) -> npt.NDArray[np.uint32]:
        """Run the paper through the device, and give back the packed frame."""
        device = self.device
        device.make_current()
        self._draw_page(page)
        self._send_tiles()
        washes = self._send_wash(page, overlay, camera, width, height)
        stood = page.stood
        page_cols, page_rows = stood.cols, stood.rows
        if washes:
            self._blur_wash(page_cols, page_rows)
        device.ensure("wash_settled", "r32f")
        device.ensure("wash_blurred", "rg32f", bands=2)

        fit_w, fit_h = self._send_fit(page, width, height)
        program = self._program(
            "composite", source.TONE + source.HATCH + source.COMPOSITE
        )
        program.use()
        units = [
            ("page", 0),
            ("page_grain", 1),
            ("tile_holder", 2),
            ("tile_cloud", 3),
            ("tile_across_x", 4),
            ("tile_across_y", 5),
            ("tile_hue", 6),
            ("wash_settled", 7),
            ("wash_blurred", 8),
            ("palette", 9),
            ("fit_x", 10),
            ("fit_y", 11),
        ]
        for name, unit in units:
            device.bind(program, name, unit)

        program["page_size"] = (page_cols, page_rows)
        # The paper names a tile by its number in the whole world, because the
        # mesh holds the whole world and the fields that change cross whole.
        program["window_size"] = (self._world.width, self._world.height)
        program["rise"] = float(stood.rise)
        program["light"] = tuple(float(band) for band in ink.LIGHT)
        # The pass that sends the tiles reads the palette from the engine and
        # sends it, and it ran above, so the palette is here.
        palette = self._palette
        if palette is None:
            message = "the palette did not reach the device before the frame"
            raise DeviceGap(message)
        program["palette_len"] = len(palette)
        program["faction_count"] = int(self._world.faction_count)
        program["draws_sky"] = 1 if self._sky else 0
        program["draws_wash"] = 1 if washes else 0
        program["cloud_step"] = max(int(page_cols * ink.CLOUD_SHADOW_STEP), 1)
        program["cloud_lift"] = int(stood.rise * ink.CLOUD_HEIGHT)
        program["fit_size"] = (fit_w, fit_h)
        program["fit_at"] = ((width - fit_w) // 2, (height - fit_h) // 2)
        program.stop()

        device.run(program, width, height, "rgba8")
        # **The rows need no turning.** The device numbers the rows of a
        # target from the bottom and reads them back in that order, so the
        # first row it gives back is the row it called zero. The shader calls
        # that row the first row of the frame, so the order already agrees.
        bands = device.read(width, height)
        packed = bands.astype(np.uint32)
        result: npt.NDArray[np.uint32] = (
            (packed[..., 0] << 16) | (packed[..., 1] << 8) | packed[..., 2]
        ).astype(np.uint32)
        return result

    def _send_fit(self, page: Page, width: int, height: int) -> tuple[int, int]:
        """Say which point of the paper each pixel of the frame shows.

        The paper is wider and shorter than the frame, because the turn
        spreads the world across it. It is scaled to fit rather than cut, so a
        watcher sees the whole world the camera covers.

        **Both renderers take this mapping from one rule.** The mapping is two
        short lists, so it crosses to the device whole rather than being
        worked out again for each pixel.

        Gives back the size of the fitted paper in pixels.
        """
        first_row, last_row, first_col, last_col = page.box
        rows = last_row - first_row
        cols = last_col - first_col
        scale = min(width / cols, height / rows)
        fit_w = max(int(cols * scale), 1)
        fit_h = max(int(rows * scale), 1)
        take_x = np.clip((np.arange(fit_w) / scale).astype(np.int32), 0, cols - 1)
        take_y = np.clip((np.arange(fit_h) / scale).astype(np.int32), 0, rows - 1)
        device = self.device
        device.upload("fit_x", (take_x + first_col)[None, :], "r32i")
        device.upload("fit_y", (take_y + first_row)[None, :], "r32i")
        return fit_w, fit_h

    def _blur_wash(self, page_cols: int, page_rows: int) -> None:
        """Blur the settled pigment at both reaches, down the paper and across.

        A box mean is the same in either order, so the two axes run as two
        passes and each pass costs the reach and not the square of it. The two
        reaches run together in one pass, because the near window lies inside
        the far one.
        """
        device = self.device
        window = (self._world.width, self._world.height)

        settle = self._program("wash_settled", source.WASH_SETTLED)
        settle.use()
        device.bind(settle, "page", 0)
        device.bind(settle, "page_grain", 1)
        device.bind(settle, "tile_flow", 2)
        device.put(settle, "page_size", (page_cols, page_rows))
        device.put(settle, "window_size", window)
        settle.stop()
        device.run(settle, page_cols, page_rows, "r32f")
        device.keep("wash_settled", "r32f")

        # The array renderer takes the mean down the page and then across it.
        # The first pass therefore runs down, and it reads one field. The
        # second runs across, and it reads the two the first produced.
        blur = self._program("wash_blur", source.WASH_BLUR)
        steps = (
            ((0, 1), 0, "wash_settled", "wash_down"),
            ((1, 0), 1, "wash_down", "wash_blurred"),
        )
        for along, paired, reads, writes in steps:
            blur.use()
            device.bind(blur, "source", 0, texture=reads)
            device.put(blur, "along", along)
            device.put(blur, "paired", paired)
            device.put(blur, "page_size", (page_cols, page_rows))
            blur.stop()
            device.run(blur, page_cols, page_rows, "rg32f")
            device.keep(writes, "rg32f")
