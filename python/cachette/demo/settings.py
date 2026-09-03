"""The settings a watcher changes, and the video section of them.

**Every setting here is one the window library honours.** The demonstration
opens its window with pyglet, and pyglet sets the window size, the fullscreen
state and the vertical synchronisation of an open window.[^1] A control that
changed nothing would be a capability nobody invokes, which is a defect shape
this project records.[^2]

The settings hold no window. The caller owns the window and applies a setting
to it, in the same way the caller owns the camera and the pixels.[^3]

A setting that this module refuses is refused with a reason. A silent refusal
looks the same as a setting that did nothing.

References
----------
The pyglet window, ``set_size``, ``set_fullscreen`` and ``set_vsync``.
https://pyglet.readthedocs.io/en/latest/modules/window.html

Recurring Defect Shapes, shape 3. ``.claude/rules/recurring-defects.md``

ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterator

# The window sizes the video section offers, in pixels.
#
# Each is a width and a height. The demonstration draws one engine pixel to
# one window pixel, so the size is the number of tiles a watcher sees and not
# only the size of the frame on the desk.
SIZES: tuple[tuple[int, int], ...] = (
    (960, 720),
    (1280, 720),
    (1280, 960),
    (1600, 900),
    (1920, 1080),
)

# The size a run opens at, as an index into the sizes above.
OPENING_SIZE = 0


class Video:
    """The video section of the settings.

    It holds a window size, a fullscreen state and a vertical synchronisation
    state. Each of the three is a setting the window library applies to an
    open window.
    """

    __slots__ = ("_size", "fullscreen", "vsync")

    def __init__(
        self,
        size: int = OPENING_SIZE,
        fullscreen: bool = False,
        vsync: bool = True,
    ) -> None:
        """Build the video settings a run opens with."""
        self._size = self._held(size)
        self.fullscreen = fullscreen
        self.vsync = vsync

    @staticmethod
    def _held(size: int) -> int:
        """Give back a size index inside the set."""
        if size < 0:
            return 0
        if size >= len(SIZES):
            return len(SIZES) - 1
        return size

    @property
    def size(self) -> tuple[int, int]:
        """Give back the window size in pixels."""
        return SIZES[self._size]

    @property
    def size_index(self) -> int:
        """Give back which of the sizes the settings hold."""
        return self._size

    def choose_size(self, size: int) -> None:
        """Choose one of the window sizes by its number."""
        self._size = self._held(size)

    def next_size(self) -> None:
        """Choose the next window size, and wrap at the end of the set."""
        self._size = (self._size + 1) % len(SIZES)

    def rows(self) -> Iterator[tuple[str, str]]:
        """Give back the section as a label and a value for each setting.

        A menu draws these. The values are text, because a menu shows text.
        """
        width, height = self.size
        yield ("window size", f"{width} x {height}")
        yield ("fullscreen", "on" if self.fullscreen else "off")
        yield ("vertical sync", "on" if self.vsync else "off")


class Settings:
    """Every setting a watcher changes.

    The video section is the only section today. A second section is a second
    attribute here and a second block in the menu.
    """

    __slots__ = ("open", "video")

    def __init__(self, video: Video | None = None) -> None:
        """Build the settings a run opens with, closed."""
        self.video = video if video is not None else Video()
        # Whether the menu is on the screen. The menu is drawn by the caller
        # that owns the window, so this is a request and not a window state.
        self.open = False

    def toggle(self) -> None:
        """Show the menu, or hide it."""
        self.open = not self.open

    def sections(self) -> Iterator[tuple[str, list[tuple[str, str]]]]:
        """Give back each section, as a name and its rows."""
        yield ("VIDEO", list(self.video.rows()))

    def apply_to(self, window: object) -> list[str]:
        """Apply every video setting to an open window.

        Returns the name of each setting the window could not take. The
        window library ships no type information, so this checks for the
        method before it calls it, and it names what it could not do rather
        than failing in silence.
        """
        refused: list[str] = []
        width, height = self.video.size
        for name, method, argument in (
            ("window size", "set_size", (width, height)),
            ("fullscreen", "set_fullscreen", (self.video.fullscreen,)),
            ("vertical sync", "set_vsync", (self.video.vsync,)),
        ):
            call = getattr(window, method, None)
            if call is None:
                refused.append(name)
                continue
            call(*argument)
        return refused
