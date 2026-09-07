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

**The video settings live between runs in one small file.** The file holds the
three video settings as JSON, and it sits in the user configuration directory
of the platform.[^5] A run reads it when it opens and writes it when a setting
changes. A file that is absent, unreadable or damaged gives the opening
settings back, because a missing convenience must never stop a run.

References
----------
The pyglet window, ``set_size``, ``set_fullscreen`` and ``set_vsync``.
https://pyglet.readthedocs.io/en/latest/modules/window.html

Recurring Defect Shapes, shape 3. ``.claude/rules/recurring-defects.md``

ADR-0094, the caller owns the camera and the pixels, decision D2.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

The pyglet window, ``set_size``, which raises while the window is fullscreen.
https://pyglet.readthedocs.io/en/latest/modules/window.html

The XDG Base Directory Specification, version 0.8, ``XDG_CONFIG_HOME``.
https://specifications.freedesktop.org/basedir-spec/latest/
"""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
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

# The directory and the file that hold the video settings between runs.
#
# The directory sits under the user configuration directory of the platform,
# so the settings of one person never reach another person and never reach the
# repository. The name of the file says which section it holds, so a second
# section can take a second file without a schema.
SETTINGS_DIRECTORY = "cachette"
SETTINGS_FILE = "video.json"


def settings_home() -> Path:
    """Give back the user configuration directory of this platform.

    macOS gets the application support directory. Windows gets the roaming
    application data directory, and falls back to the profile when the
    environment names none. Every other platform gets the XDG configuration
    directory, which is ``~/.config`` when the environment names none.
    """
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support"
    if sys.platform == "win32":
        named = os.environ.get("APPDATA")
        if named:
            return Path(named)
        return Path.home() / "AppData" / "Roaming"
    named = os.environ.get("XDG_CONFIG_HOME")
    if named:
        return Path(named)
    return Path.home() / ".config"


def settings_path() -> Path:
    """Give back the file that holds the video settings between runs."""
    return settings_home() / SETTINGS_DIRECTORY / SETTINGS_FILE


class Video:
    """The video section of the settings.

    It holds a window size, a fullscreen state and a vertical synchronisation
    state. Each of the three is a setting the window library applies to an
    open window.
    """

    __slots__ = ("_size", "fullscreen", "sketch", "vsync")

    def __init__(
        self,
        size: int = OPENING_SIZE,
        fullscreen: bool = False,
        vsync: bool = True,
        sketch: bool = True,
    ) -> None:
        """Build the video settings a run opens with.

        **The sketch is the renderer a run opens on.** The engine renderer
        stays, because it is the reference the sketch is read against and
        because a machine that cannot draw the sketch must still show the
        world. A watcher who chose it keeps it, because this setting is
        written to the file with the rest.
        """
        self._size = self._held(size)
        self.fullscreen = fullscreen
        self.vsync = vsync
        self.sketch = sketch

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

    def fit_within(self, width: int, height: int) -> None:
        """Lower the size until it fits inside a display of this size.

        **A saved size can be larger than the display of this run.** A watcher
        saves a large size on one monitor and opens the next run on a smaller
        one. The size is an index into a fixed table, so the saved value is
        always a size the demonstration offers, but it is not always a size
        the display can show.

        This lowers the index to the largest size that fits, and it never
        raises the index. A watcher who chose a small size keeps it. A display
        that is smaller than every size in the table gets the smallest size,
        because the window must still open.
        """
        for index in range(self._size, -1, -1):
            wide, tall = SIZES[index]
            if wide <= width and tall <= height:
                self._size = index
                return
        self._size = 0

    def state(self) -> dict[str, object]:
        """Give back the section as the values that a file holds."""
        return {
            "size": self._size,
            "fullscreen": self.fullscreen,
            "vsync": self.vsync,
            "sketch": self.sketch,
        }

    @classmethod
    def from_state(cls, state: object) -> Video:
        """Build the section from the values that a file held.

        **A value this cannot read gives the opening value back.** The file is
        a convenience that a person may edit, another version may have
        written, or a half-finished write may have truncated. Each value is
        read on its own, so one bad value costs one setting and not the file.
        """
        if not isinstance(state, dict):
            return cls()
        return cls(
            size=_whole(state.get("size"), OPENING_SIZE),
            fullscreen=_yes_or_no(state.get("fullscreen"), False),
            vsync=_yes_or_no(state.get("vsync"), True),
            sketch=_yes_or_no(state.get("sketch"), True),
        )

    def rows(self) -> Iterator[tuple[str, str]]:
        """Give back the section as a label and a value for each setting.

        A menu draws these. The values are text, because a menu shows text.
        """
        width, height = self.size
        yield ("window size", f"{width} x {height}")
        yield ("fullscreen", "on" if self.fullscreen else "off")
        yield ("vertical sync", "on" if self.vsync else "off")
        yield ("renderer", "sketch" if self.sketch else "engine")


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

    def _calls(self, with_size: bool) -> list[tuple[str, str, tuple[object, ...]]]:
        """Give back each window call in the order the window accepts them.

        **The fullscreen state goes first, and the size follows it.** A window
        library refuses a size while the window is fullscreen, because a
        fullscreen window is the size of the screen.[^4] A run that set the
        size first therefore failed as it left fullscreen.

        A fullscreen window takes no size at all, so this omits the size call
        while the fullscreen state is on. The caller reads the size the window
        reports rather than the size held here.

        A caller that opened the window at a size of its own passes False for
        the size, and keeps the size it opened with.
        """
        width, height = self.video.size
        calls: list[tuple[str, str, tuple[object, ...]]] = [
            ("fullscreen", "set_fullscreen", (self.video.fullscreen,)),
        ]
        if with_size and not self.video.fullscreen:
            calls.append(("window size", "set_size", (width, height)))
        calls.append(("vertical sync", "set_vsync", (self.video.vsync,)))
        return calls

    def apply_to(self, window: object, with_size: bool = True) -> list[str]:
        """Apply every video setting to an open window.

        Returns the name of each setting the window could not take. The
        window library ships no type information, so this checks for the
        method before it calls it, and it names what it could not do rather
        than failing in silence.

        **A window that raises is a refusal too.** The library refuses a call
        it holds by raising, and an uncaught raise leaves the window half
        set and stops the key press that asked for the change.
        """
        refused: list[str] = []
        for name, method, argument in self._calls(with_size):
            call = getattr(window, method, None)
            if call is None:
                refused.append(name)
                continue
            try:
                call(*argument)
            except Exception:
                refused.append(name)
        return refused


def _whole(value: object, fallback: int) -> int:
    """Give back a whole number, or the fallback for anything else.

    A boolean is a whole number in Python, and it is not one here. A file that
    holds ``true`` for a size holds a damaged size.
    """
    if isinstance(value, bool) or not isinstance(value, int):
        return fallback
    return value


def _yes_or_no(value: object, fallback: bool) -> bool:
    """Give back a boolean, or the fallback for anything else."""
    if not isinstance(value, bool):
        return fallback
    return value


def load_video(path: Path | None = None) -> Video:
    """Read the video settings a run last saved.

    **A file this cannot read gives the opening settings back.** A run that
    refused to start because of a settings file would be worse than a run with
    no memory at all. A file that is absent, that the platform refuses, that
    holds no JSON, or that holds JSON of the wrong shape all reach the same
    answer.
    """
    where = path if path is not None else settings_path()
    try:
        text = where.read_text(encoding="utf-8")
    except OSError:
        return Video()
    try:
        state = json.loads(text)
    except ValueError:
        return Video()
    return Video.from_state(state)


def save_video(video: Video, path: Path | None = None) -> bool:
    """Write the video settings, and say whether the write succeeded.

    **A write that fails is not an error of the run.** A read-only home
    directory or a full disk costs the memory of the window state, and it must
    not stop the world.

    The write goes to a file beside the target and then replaces it, so a run
    that stops in the middle leaves the last good file rather than half of a
    new one.
    """
    where = path if path is not None else settings_path()
    beside = where.with_name(where.name + ".part")
    try:
        where.parent.mkdir(parents=True, exist_ok=True)
        beside.write_text(json.dumps(video.state(), indent=2) + "\n", encoding="utf-8")
        beside.replace(where)
    except OSError:
        try:
            beside.unlink(missing_ok=True)
        except OSError:
            pass
        return False
    return True
