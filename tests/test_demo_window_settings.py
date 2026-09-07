"""The demonstration applies its video settings in the order a window takes.

The window of the demonstration cannot open in the gate, because the gate has
no display. Every test here therefore drives the parts of the window path that
hold no window: the settings, the size the surface follows, and the picture the
window draws.

A fake window stands in for the real one. It records each call it receives, and
it refuses a size while it is fullscreen, which is what the window library
does.[^1] A fake that accepted every call would prove nothing, because the
defect was a call the library refused.

The video settings live between runs in a file. Every test that touches the
file points the settings module at a directory the test owns, so no test reads
or writes the settings of the person who ran it.

References
----------
The pyglet window, ``set_size``, which raises while the window is fullscreen.
https://pyglet.readthedocs.io/en/latest/modules/window.html

ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

import sys
from pathlib import Path

import pytest

from cachette import World
from cachette.demo import settings as settings_module
from cachette.demo.app import (
    Demo,
    Picture,
    WindowPicture,
    _apply_settings,
    window_size,
)
from cachette.demo.settings import (
    OPENING_SIZE,
    SIZES,
    Settings,
    Video,
    load_video,
    save_video,
)
from cachette.demo.surface import Surface
from cachette.names import Names

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


class _Held:
    """A settings file in a directory the test owns."""

    def __init__(self, path: Path) -> None:
        """Hold the path the settings module was pointed at."""
        self.path = path


@pytest.fixture(name="held")
def held_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> _Held:
    """Point the settings module at a file inside the directory of the test.

    Every save and every load without a path reaches this file. A test that
    wrote to the real configuration directory would change the settings of the
    person who ran it.
    """
    path = tmp_path / "video.json"
    monkeypatch.setattr(settings_module, "settings_path", lambda: path)
    return _Held(path)


def test_a_saved_state_is_read_back(held: _Held) -> None:
    """The three video settings come back as the run left them."""
    video = Video(size=3, fullscreen=True, vsync=False)

    assert save_video(video)
    read = load_video()

    assert read.size_index == 3
    assert read.size == SIZES[3]
    assert read.fullscreen is True
    assert read.vsync is False


def test_a_size_chosen_by_the_menu_is_read_back(held: _Held) -> None:
    """A size the watcher stepped to with a key comes back on the next run."""
    settings_now = Settings()
    settings_now.video.next_size()
    settings_now.video.next_size()

    assert save_video(settings_now.video)

    assert load_video().size_index == settings_now.video.size_index


def test_a_missing_file_gives_the_opening_settings(held: _Held) -> None:
    """A first run finds no file, and it opens at the defaults."""
    assert not held.path.exists()

    video = load_video()

    assert video.size_index == OPENING_SIZE
    assert video.fullscreen is False
    assert video.vsync is True


@pytest.mark.parametrize(
    "body",
    [
        "",
        "{",
        "not json at all",
        "[1, 2, 3]",
        '"a string"',
        "null",
        '{"size": "large", "fullscreen": "yes", "vsync": 7}',
        '{"size": true}',
        '{"other": 1}',
    ],
)
def test_a_damaged_file_gives_the_opening_settings(held: _Held, body: str) -> None:
    """A file of the wrong shape costs the memory and never the run."""
    held.path.write_text(body, encoding="utf-8")

    video = load_video()

    assert video.size_index == OPENING_SIZE
    assert video.fullscreen is False
    assert video.vsync is True


def test_a_file_the_platform_refuses_gives_the_opening_settings(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A path that is a directory cannot be read, and the run still opens."""
    where = tmp_path / "video.json"
    where.mkdir()
    monkeypatch.setattr(settings_module, "settings_path", lambda: where)

    assert load_video().size_index == OPENING_SIZE


def test_a_saved_size_outside_the_table_is_pulled_inside(held: _Held) -> None:
    """A file from a version with more sizes holds an index this one lacks."""
    held.path.write_text(f'{{"size": {len(SIZES) + 40}}}', encoding="utf-8")

    assert load_video().size_index == len(SIZES) - 1

    held.path.write_text('{"size": -3}', encoding="utf-8")

    assert load_video().size_index == 0


def test_a_write_that_fails_is_reported_and_does_not_raise(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A file the platform refuses costs the memory and never the run."""
    where = tmp_path / "video.json"
    where.mkdir()
    monkeypatch.setattr(settings_module, "settings_path", lambda: where)

    assert save_video(Video(size=2)) is False


def test_the_settings_file_sits_in_the_configuration_directory_of_the_platform(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Each platform gets its own convention, and none writes beside the code."""
    monkeypatch.setattr(sys, "platform", "linux")
    monkeypatch.setenv("XDG_CONFIG_HOME", str(tmp_path / "config"))
    assert (
        settings_module.settings_path()
        == tmp_path / "config" / "cachette" / "video.json"
    )

    monkeypatch.delenv("XDG_CONFIG_HOME")
    assert (
        settings_module.settings_path()
        == Path.home() / ".config" / "cachette" / "video.json"
    )

    monkeypatch.setattr(sys, "platform", "darwin")
    assert settings_module.settings_path().parent.parent.name == "Application Support"

    monkeypatch.setattr(sys, "platform", "win32")
    monkeypatch.setenv("APPDATA", str(tmp_path / "roaming"))
    assert (
        settings_module.settings_path()
        == tmp_path / "roaming" / "cachette" / "video.json"
    )


def test_a_saved_size_larger_than_the_display_is_lowered() -> None:
    """A size saved on a large monitor opens on a small one."""
    video = Video(size=len(SIZES) - 1)
    widest, tallest = SIZES[-1]

    video.fit_within(widest - 1, tallest - 1)

    width, height = video.size
    assert width <= widest - 1
    assert height <= tallest - 1
    assert video.size_index < len(SIZES) - 1


def test_a_size_that_fits_is_left_alone() -> None:
    """A large display never enlarges the size the watcher chose."""
    video = Video(size=1)

    video.fit_within(7680, 4320)

    assert video.size_index == 1


def test_a_display_smaller_than_every_size_gets_the_smallest() -> None:
    """The window must still open on a display no size fits."""
    video = Video(size=len(SIZES) - 1)

    video.fit_within(320, 240)

    assert video.size_index == 0


def test_a_window_opened_at_a_typed_size_is_given_no_size() -> None:
    """A size on the command line governs the run, so nothing overrides it."""
    settings_now = Settings()
    settings_now.video.choose_size(4)
    window = FakeWindow(1400, 1050)

    refused = settings_now.apply_to(window, with_size=False)

    assert refused == []
    assert "set_size" not in window.names()
    # The other two settings still reach the window. Only the size is left to
    # the caller that opened it.
    assert window.names() == ["set_fullscreen", "set_vsync"]


def _a_demo() -> Demo:
    """Build a demonstration over a world small enough to hold in a test."""
    world = World(width=32, height=32, seed=0x0123_4567_89AB_CDEF, faction_count=2)
    return Demo(world, Names(world.seed), width=320, height=240, threads=1)


def test_the_window_path_writes_the_file_when_a_setting_changes(
    held: _Held,
) -> None:
    """The caller that gives the window a setting is the caller that saves it.

    **The test drives the window path, not the file.** A test that called the
    save itself would prove that the file works and not that a key press
    reaches it. This is the deepest caller a test without a display reaches:
    the key handler lives inside the window loop, and it calls this.
    """
    demo = _a_demo()
    window = FakeWindow(*SIZES[0])

    demo.settings.video.next_size()
    _apply_settings(demo, window)

    assert held.path.exists()
    assert load_video().size_index == demo.settings.video.size_index


def test_the_window_path_saves_the_fullscreen_state(held: _Held) -> None:
    """A watcher who goes fullscreen opens the next run fullscreen."""
    demo = _a_demo()
    window = FakeWindow(*SIZES[0])

    demo.settings.video.fullscreen = True
    _apply_settings(demo, window)

    assert load_video().fullscreen is True


def test_the_window_opening_writes_no_file(held: _Held) -> None:
    """A size this display forced down never replaces the saved choice.

    A watcher saves a large size on one monitor and opens the next run on a
    smaller one. That run must open at a size the display can show, and it
    must give the large size back when the large monitor comes back.
    """
    demo = _a_demo()
    demo.settings.video.choose_size(len(SIZES) - 1)
    demo.settings.video.fit_within(640, 480)
    window = FakeWindow(*SIZES[0])

    _apply_settings(demo, window, with_size=False, remember=False)

    assert not held.path.exists()
