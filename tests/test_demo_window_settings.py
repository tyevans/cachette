"""The demonstration applies its video settings in the order a window takes.

The window of the demonstration cannot open in the gate, because the gate has
no display. Every test here therefore drives the parts of the window path that
hold no window: the settings, the size the surface follows, and the picture the
window draws.

A fake window stands in for the real one. It records each call it receives, and
it refuses a size while it is fullscreen, which is what the window library
does.[^1] A fake that accepted every call would prove nothing, because the
defect was a call the library refused.

References
----------
The pyglet window, ``set_size``, which raises while the window is fullscreen.
https://pyglet.readthedocs.io/en/latest/modules/window.html

ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

from cachette.demo.app import Picture, WindowPicture, window_size
from cachette.demo.settings import SIZES, Settings
from cachette.demo.surface import Surface

# The size of the screen the fake window fills when it goes fullscreen. It is
# not one of the sizes the settings offer, so a test cannot pass by reading the
# setting instead of the window.
SCREEN = (2560, 1440)


class FakeWindowError(Exception):
    """The fake window refuses a call, in the way the window library does."""


class FakeWindow:
    """A window that records its calls and refuses a size while fullscreen."""

    def __init__(self, width: int, height: int) -> None:
        """Build a windowed fake of this size."""
        self.calls: list[tuple[str, tuple[object, ...]]] = []
        self.fullscreen = False
        self.width = width
        self.height = height

    def set_size(self, width: int, height: int) -> None:
        """Take a new size, unless the window is fullscreen."""
        self.calls.append(("set_size", (width, height)))
        if self.fullscreen:
            message = "Cannot set size of fullscreen window."
            raise FakeWindowError(message)
        self.width = width
        self.height = height

    def set_fullscreen(self, state: bool) -> None:
        """Go fullscreen, or leave it, and take the size that follows."""
        self.calls.append(("set_fullscreen", (state,)))
        self.fullscreen = state
        if state:
            self.width, self.height = SCREEN
        else:
            self.width, self.height = SIZES[0]

    def set_vsync(self, state: bool) -> None:
        """Take the vertical synchronisation state."""
        self.calls.append(("set_vsync", (state,)))

    def names(self) -> list[str]:
        """Give back the name of each call, in the order they arrived."""
        return [name for name, _ in self.calls]


def test_fullscreen_is_applied_before_the_size() -> None:
    settings = Settings()
    window = FakeWindow(*SIZES[0])

    settings.apply_to(window)

    names = window.names()
    assert names.index("set_fullscreen") < names.index("set_size")


def test_a_fullscreen_window_is_given_no_size() -> None:
    settings = Settings()
    settings.video.fullscreen = True
    window = FakeWindow(*SIZES[0])

    refused = settings.apply_to(window)

    assert "set_size" not in window.names()
    assert refused == []


def test_leaving_fullscreen_sets_the_size_and_is_not_refused() -> None:
    settings = Settings()
    settings.video.fullscreen = True
    window = FakeWindow(*SIZES[0])
    settings.apply_to(window)

    settings.video.fullscreen = False
    refused = settings.apply_to(window)

    assert refused == []
    assert ("set_size", SIZES[0]) in window.calls
    assert window.width == SIZES[0][0]


def test_a_window_that_raises_is_named_and_does_not_raise() -> None:
    class Stubborn(FakeWindow):
        def set_vsync(self, state: bool) -> None:
            message = "no vertical synchronisation here"
            raise FakeWindowError(message)

    settings = Settings()
    window = Stubborn(*SIZES[0])

    refused = settings.apply_to(window)

    assert refused == ["vertical sync"]


def test_a_missing_method_is_still_named() -> None:
    settings = Settings()

    refused = settings.apply_to(object())

    assert refused == ["fullscreen", "window size", "vertical sync"]


def test_the_surface_follows_the_window_and_not_the_setting() -> None:
    settings = Settings()
    settings.video.fullscreen = True
    window = FakeWindow(*SIZES[0])
    settings.apply_to(window)

    assert window_size(window, settings.video.size) == SCREEN
    assert settings.video.size != SCREEN


def test_a_window_that_reports_no_size_gives_the_setting_back() -> None:
    assert window_size(object(), SIZES[1]) == SIZES[1]


class FakeImage:
    """An image that records the buffers it is given."""

    def __init__(self, width: int, height: int, pitch: int, data: bytes) -> None:
        """Build a fake image of this size."""
        self.width = width
        self.height = height
        self.pitch = pitch
        self.data = data
        self.updates: list[int] = []
        self.drawn: tuple[int, int] | None = None

    def set_data(self, _layout: str, pitch: int, data: bytes) -> None:
        """Take a new frame, and refuse a buffer of the wrong length."""
        if len(data) != abs(pitch) * self.height:
            want = abs(pitch) * self.height
            message = f"the buffer holds {len(data)} bytes, not {want}"
            raise ValueError(message)
        self.updates.append(len(data))
        self.data = data

    def blit(self, x: int, y: int) -> None:
        """Record where the window drew the frame."""
        self.drawn = (x, y)


def _picture(surface: Surface) -> tuple[WindowPicture, list[FakeImage]]:
    """Build a picture over a fake image, and give back every image it built."""
    built: list[FakeImage] = []

    def make(width: int, height: int, _layout: str, data: bytes, pitch: int) -> Picture:
        image = FakeImage(width, height, pitch, data)
        built.append(image)
        return image

    return WindowPicture(make, surface), built


def test_the_picture_takes_a_frame_of_the_same_size() -> None:
    surface = Surface(64, 32)
    picture, built = _picture(surface)

    picture.update(surface)

    assert len(built) == 1
    assert built[0].updates == [64 * 32 * 4]


def test_the_picture_is_rebuilt_when_the_surface_changes_size() -> None:
    picture, built = _picture(Surface(64, 32))

    picture.update(Surface(128, 96))

    assert len(built) == 2
    assert (built[1].width, built[1].height) == (128, 96)
    assert built[1].pitch == -128 * 4
    assert picture.pitch == -128 * 4
    assert picture.image is built[1]
