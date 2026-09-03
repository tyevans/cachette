"""The demonstration: build a world, step it, and show it.

The control plane owns the loop, the camera and the pixels. The engine owns
the drawing. One call fills a whole frame, and this module never names a tile
or an entity.[^1]

The window library is pyglet. It is maintained, it ships as a pure Python
wheel with no bundled native library, and it binds the system graphics through
ctypes, so it adds no compiled dependency to a machine that installs this
package.[^2]

The engine tick and the wall clock are separate. The window draws at its own
rate, and a clock says how many ticks the world runs between two drawings. A
paused world runs none and still draws.

Every draw still follows the steps of that frame, on one thread, which is what
the viewer record fixes.[^3] The number of steps in a frame belongs to the
caller, and this module is the caller.[^4]

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

Decisions register, the window library of the Python demonstration.
``docs/DECISIONS.md``

ADR-0067, the viewer reads the world and never writes to it, decision D4.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``

Findings register, FND-327. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import argparse
import os
from typing import TYPE_CHECKING

from cachette import Camera, World

if TYPE_CHECKING:
    # These describe the shape of a dictionary the engine returns. They live
    # in the stub beside the compiled module and not in the module itself, so
    # importing them at run time would fail.
    from cachette._core import FoundingReport, FrameReading
from cachette.demo.clock import SPEEDS, Clock
from cachette.demo.settings import Settings
from cachette.demo.surface import Surface

# The size of the window in pixels.
WINDOW_WIDTH = 960
WINDOW_HEIGHT = 720

# The world the demonstration builds. The engine is the same engine the tests
# exercise, and these numbers only choose which world it runs.
WORLD_WIDTH = 256
WORLD_HEIGHT = 256
WORLD_SEED = 0x0123_4567_89AB_CDEF
FACTION_COUNT = 4

# The number of people each faction founds with.
GROUP = 64

# The height a picture of the whole panel starts at.
#
# **This is a starting point, not the answer.** The panel grows with the
# faction count, with the number of foundings, and with every section a count
# switches on, so a constant that fits today cuts tomorrow. The picture asks
# the engine how tall the panel needs to be and resizes to it.
PICTURE_HEIGHT = 1400

# How many frames a picture steps before it draws, when nobody says.
#
# **A picture at tick 2 shows a world in which nothing has happened.** No seat
# is held, no unit carries a load, no store has rationed and no soldier has
# been promoted, so a picture taken then reports every subsystem at zero and
# each of those zeros is the fixture rather than the engine. The seats fill,
# the carrying starts and the first promotion lands well inside this count,
# so a reader sees the subsystems the panel reports.
#
# This is a default and not a bound. `--ticks` takes any count, including 0
# for a picture of the world as it was founded.
PICTURE_TICKS = 300

# How many times the picture may resize before it gives up.
#
# Resizing changes what the window paints, and a section a count switches on
# adds lines, so one answer can move the next. Two passes settle it in
# practice. The bound stops a panel that never settles from looping.
RESIZES = 4

# The engine steps once for each drawn frame.
FRAMES_EACH_SECOND = 30


class Demo:
    """The state the control plane holds between frames.

    The engine holds none of it. The camera, the window size and the founding
    report belong to the caller, and the caller hands the camera back on every
    frame.
    """

    __slots__ = (
        "camera",
        "clock",
        "panels",
        "pointer",
        "reference",
        "settings",
        "surface",
        "threads",
        "world",
    )

    def __init__(
        self,
        world: World,
        width: int = WINDOW_WIDTH,
        height: int = WINDOW_HEIGHT,
        threads: int = 0,
    ) -> None:
        """Build the state the control plane holds between frames."""
        self.world = world
        self.surface = Surface(width, height)
        self.threads = threads if threads > 0 else min(os.cpu_count() or 1, 12)
        # The reference layer names the colours while a key is held. It holds
        # no state between frames: the keyboard says what the watcher wants,
        # and the answer lives for one frame.
        self.reference = False
        self.camera = Camera()
        # The engine tick and the wall clock are separate. The window draws at
        # its own rate and this says how far the world moves between two
        # drawings.
        self.clock = Clock()
        self.settings = Settings()
        # The panels of the deck the frame draws, by name. The engine holds
        # the list of names, so this cannot name one that does not exist.
        self.panels: list[str] = []
        # The tile the watcher pointed at, in axial coordinates. The engine
        # has no cursor, so the control plane supplies one.
        self.pointer: tuple[int, int] | None = None

    def found(self) -> list[FoundingReport]:
        """Found one run for every faction, and give back what each got.

        One call seats every faction, because a founding keeps its distance
        from the foundings before it. The engine keeps the report for the
        panel, and the caller gets a summary to print.
        """
        return self.world.found_run_for_every_faction(GROUP)

    def open_on(self, place: tuple[int, int]) -> None:
        """Point the camera at a place and hold it inside the world.

        A group holds one small part of a large world, so a camera at the
        corner would show an empty map.
        """
        width, height = self.surface.width, self.surface.height
        self.camera.look_at(place[0], place[1], width, height)
        self.camera.clamp(self.world, width, height)

    def steer(self, across: float, down: float, zoom: int) -> None:
        """Move the camera by whole presses.

        A press moves the view by a share of the window, so it covers the same
        part of the picture at every zoom. The verbs live in the engine, so no
        number here is a second copy of a number there.
        """
        width, height = self.surface.width, self.surface.height
        if zoom > 0:
            self.camera.zoom_in(width, height)
        elif zoom < 0:
            self.camera.zoom_out(width, height)
        if across or down:
            self.camera.nudge(across, down, width, height)
        self.camera.clamp(self.world, width, height)

    def announce(self, reading: FrameReading) -> None:
        """Say when a soldier becomes a character, and what earned it.

        **The control plane reacts to one fact the engine reported.** It reads
        the count the frame gave it and prints a line. It walks no entity and
        asks the engine nothing further, so this is a reaction and not a poll.

        A promotion happens on a small share of frames, so the line is rare
        enough to read and it names the moment rather than a total that went
        up.
        """
        if reading["promoted_now"] <= 0:
            return
        deeds = reading["promoted_deeds"]
        earned = f" for {deeds} deeds" if deeds is not None else ""
        one = reading["promoted_now"] == 1
        who = "person" if one else "people"
        what = "a character" if one else "characters"
        print(
            f"tick {reading['tick']}: {reading['promoted_now']} {who} "
            f"became {what}{earned}, {reading['characters']} in the world"
        )

    def toggle_panel(self, name: str) -> None:
        """Add a panel of the deck to the frame, or take it off.

        The engine names the panels it can draw, so a name that no panel
        carries is refused here rather than at the drawing.
        """
        if name not in World.panel_names():
            message = f"no panel is called {name!r}"
            raise ValueError(message)
        if name in self.panels:
            self.panels.remove(name)
        else:
            self.panels.append(name)

    def point_at(self, x: float, y: float) -> None:
        """Name the tile under a place in the window.

        The engine answers which tile a pixel covers. This names one address
        and reads no tile.
        """
        self.pointer = self.camera.tile_at(x, y)

    def advance(self, panel: bool = False) -> FrameReading:
        """Step the engine as far as the clock says, then draw one frame.

        **The engine tick and the wall clock are separate.** The clock says
        how many ticks this frame owes. A paused world owes none and still
        draws, so the camera still moves and the panel still reads.

        Returns what the drawing pass read. The caller reports those numbers
        rather than starting a second pass to find them.

        The reading names what the last step logged. A frame that runs several
        ticks therefore reports the last of them, and the logs of the earlier
        ticks are gone. A watcher who wants every tick sets the speed to one.
        """
        for _ in range(self.clock.ticks_due()):
            self.world.step(self.threads)
        reading = self.world.draw(
            self.camera,
            self.surface.width,
            self.surface.height,
            self.surface.pixels,
            reference=self.reference,
            panel=panel,
            panels=self.panels or None,
            pointer=self.pointer,
        )
        self.announce(reading)
        return reading


def build_world() -> World:
    """Build the world the demonstration runs."""
    return World(
        width=WORLD_WIDTH,
        height=WORLD_HEIGHT,
        seed=WORLD_SEED,
        faction_count=FACTION_COUNT,
    )


def report(foundings: list[FoundingReport]) -> tuple[int, int]:
    """Print what each faction got, and give back how many were seated and fed.

    The loop is over factions, of which there are four. It is not a loop over
    entities, and it reads a summary the engine already made.
    """
    seated = 0
    carried = 0
    for founding in foundings:
        faction = founding["faction"]
        if not founding["seated"]:
            print(f"faction {faction} found no place: {founding['refusal']}")
            continue
        seated += 1
        print(
            f"faction {faction} founded at ({founding['q']}, {founding['r']}) "
            f"with {founding['people']} people, "
            f"chosen from {founding['considered']} places"
        )
        print(
            f"  it reaches {founding['food']} food, {founding['wood']} wood "
            f"and {founding['stone']} stone, over {founding['open_ground']} "
            f"open tiles, with {founding['water_edge']} of open water beside it"
        )
        if founding["carries_its_group"]:
            carried += 1
            print(f"  this ground carries its group of {GROUP}")
        else:
            print(
                f"  this ground carries {founding['food']} of its group "
                f"of {GROUP}, and the rest go short"
            )
    # A fixture that produces one condition everywhere measures itself. This
    # says which way the run came out rather than assuming the split.
    if seated > 0 and carried in (0, seated):
        state = "short" if carried == 0 else "fed"
        print(f"note: every seated group is {state}, so this run shows one condition")
    return seated, carried


def opening_place(foundings: list[FoundingReport]) -> tuple[int, int]:
    """Give back the place the view opens on."""
    for founding in foundings:
        if founding["seated"]:
            return (founding["q"], founding["r"])
    return (0, 0)


def main(argv: list[str] | None = None) -> int:
    """Open the window and run until the watcher closes it."""
    parser = argparse.ArgumentParser(
        prog="python -m cachette.demo",
        description="Watch the world run, from the control plane.",
    )
    parser.add_argument("--width", type=int, default=WINDOW_WIDTH)
    parser.add_argument(
        "--height",
        type=int,
        default=0,
        help="the window height; the picture mode is taller by default",
    )
    parser.add_argument("--threads", type=int, default=0)
    parser.add_argument(
        "--frames",
        type=int,
        default=0,
        help="stop the window after this many frames, for a run without a watcher",
    )
    parser.add_argument(
        "--picture",
        default="",
        help=(
            "write one frame with the whole panel to this file, and open no "
            "window; the name must end in .png or .ppm"
        ),
    )
    parser.add_argument(
        "--ticks",
        type=int,
        default=PICTURE_TICKS,
        help=(
            "how many frames the picture mode steps before it draws; the "
            "default runs far enough for the seats, the carrying and the "
            "first promotions to appear"
        ),
    )
    arguments = parser.parse_args(argv)

    # The panel holds every section and is taller than a window a person
    # opens. A picture that used the window height would cut the last
    # sections and say so, which is honest and still less than was asked for.
    default_height = PICTURE_HEIGHT if arguments.picture else WINDOW_HEIGHT
    demo = Demo(
        build_world(),
        width=arguments.width,
        height=arguments.height or default_height,
        threads=arguments.threads,
    )
    foundings = demo.found()
    seated, _ = report(foundings)
    if seated == 0:
        print("no faction found a place, so there is nothing to watch")
        return 1
    demo.open_on(opening_place(foundings))

    if arguments.picture:
        return _write_picture(demo, arguments.picture, arguments.ticks)

    print(
        f"cachette: {demo.world.width} by {demo.world.height} tiles, "
        f"{demo.world.soldier_count} people, {demo.threads} threads"
    )
    print("arrow keys or WASD scroll, minus and equals zoom")
    print("hold tab to name the colours")
    print("space pauses, full stop steps one tick, brackets change the speed")
    print(f"the speeds are {', '.join(f'x{speed}' for speed in SPEEDS)}")
    print("F10 opens the settings, F11 fullscreen, F12 window size")
    keys = ", ".join(f"F{at + 1} {name}" for at, name in enumerate(World.panel_names()))
    print(f"the panels are {keys}")
    print("click a tile to point at it")
    print("close the window or press escape to stop")

    return _run_window(demo, arguments.frames)


def _write_picture(demo: Demo, path: str, frames: int) -> int:
    """Step the world, then write one frame with the whole panel to a file.

    This presenter needs no window library and no display. It is the same
    frame command the window uses, so the picture holds what the window would
    have shown, with the sections the cards leave out.
    """
    # The world must run before it is worth drawing, and it must be drawn at
    # least once whatever the count. The steps before the last one take the
    # same path the window takes, so a promotion during the run is announced
    # here as it would be on a screen, and the picture cannot be drawn by a
    # path that the window never runs.
    #
    # **The run draws at the height of a window, not at the height of the
    # panel.** The picture is one frame and the run is hundreds, and the whole
    # panel is several times the height a watcher opens, so drawing every step
    # at that height spends most of the run on pixels that nobody keeps. The
    # surface grows for the frame that is written and for that frame only.
    written = demo.surface
    demo.surface = Surface(written.width, min(written.height, WINDOW_HEIGHT))
    for _ in range(max(frames, 0)):
        demo.advance()
    demo.surface = written
    reading = demo.advance(panel=True)

    # Ask the panel how tall it needed to be, and draw again at that height.
    # The loop ends when the picture is tall enough for the panel it drew.
    for attempt in range(RESIZES):
        needed = reading["panel_height"]
        if needed <= demo.surface.height:
            break
        if attempt + 1 == RESIZES:
            print(
                f"the panel still needs {needed} pixels after {RESIZES} "
                f"resizes, so the picture holds less than the whole panel"
            )
            break
        demo.surface = Surface(demo.surface.width, needed)
        reading = demo.advance(panel=True)

    demo.surface.write_image(path)
    print(
        f"wrote {path} at tick {reading['tick']}, "
        f"{demo.surface.width} by {demo.surface.height}, "
        f"{reading['tiles_painted']} tiles and "
        f"{reading['soldiers_painted']} people painted"
    )
    return 0


def _toggle_panel_key(demo: Demo, symbol: int, key: object) -> None:
    """Put a panel of the deck on the frame, or take it off.

    The function keys F1 upward name the panels in the order the engine
    registers them. The engine owns that order, so a panel that joins the deck
    gets a key with no edit here.
    """
    names = World.panel_names()
    first = getattr(key, "F1", 0)
    at = symbol - first
    if 0 <= at < len(names) and at < 9:
        demo.toggle_panel(names[at])
        shown = ", ".join(demo.panels) if demo.panels else "none"
        print(f"panels: {shown}")


def _apply_settings(demo: Demo, window: object) -> None:
    """Give the window the video settings, and resize the pixels to match.

    The surface is the memory the engine fills. A window of a new size needs a
    surface of that size, so the two are changed together.
    """
    refused = demo.settings.apply_to(window)
    if refused:
        print(f"the window refused: {', '.join(refused)}")
    width, height = demo.settings.video.size
    if (width, height) != (demo.surface.width, demo.surface.height):
        demo.surface = Surface(width, height)
        demo.camera.clamp(demo.world, width, height)


def _show_settings(demo: Demo, window: object) -> None:
    """Write the settings menu, or say that it closed.

    **The menu is written to the console and not over the map.** The engine
    draws the frame, and it draws what it reads from the world. It takes no
    text from the caller, so a menu over the map would need a second drawing
    path and two drawing paths disagree about the world.[^1]

    References
    ----------
    ADR-0094, the caller owns the camera and the pixels, decision D5.
    ``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
    """
    if not demo.settings.open:
        print("settings closed")
        return
    print("settings")
    for name, rows in demo.settings.sections():
        print(f"  {name}")
        for label, value in rows:
            print(f"    {label}: {value}")
    print("  F11 fullscreen, F12 window size")
    _apply_settings(demo, window)


def _run_window(demo: Demo, frame_limit: int) -> int:
    """Drives the window until it closes.

    The import is here and not at the top of the module, so that a caller that
    only wants a frame in memory needs no window library and no display.
    """
    import pyglet

    window = pyglet.window.Window(
        width=demo.surface.width,
        height=demo.surface.height,
        caption="cachette — watch the world run",
    )
    # The engine writes the first row of the frame first, and the window
    # numbers its rows from the bottom. A negative pitch says so once.
    pitch = -demo.surface.width * 4
    image = pyglet.image.ImageData(
        demo.surface.width,
        demo.surface.height,
        "BGRA",
        demo.surface.to_bytes(),
        pitch=pitch,
    )
    keys = pyglet.window.key.KeyStateHandler()
    window.push_handlers(keys)
    counted = [0]

    def frame(_delta: float) -> None:
        key = pyglet.window.key
        demo.reference = keys[key.TAB]
        across = float(keys[key.RIGHT] or keys[key.D]) - float(
            keys[key.LEFT] or keys[key.A]
        )
        down = float(keys[key.DOWN] or keys[key.S]) - float(keys[key.UP] or keys[key.W])
        zoom = int(keys[key.EQUAL]) - int(keys[key.MINUS])
        demo.steer(across, down, zoom)
        demo.advance()
        image.set_data("BGRA", pitch, demo.surface.to_bytes())
        counted[0] += 1
        if frame_limit and counted[0] >= frame_limit:
            pyglet.app.exit()

    def on_draw() -> None:
        window.clear()
        image.blit(0, 0)

    def on_mouse_press(x: int, y: int, _button: int, _modifiers: int) -> None:
        # The window numbers its rows from the bottom and the engine numbers
        # them from the top, so the height turns one into the other.
        demo.point_at(float(x), float(demo.surface.height - y))
        q, r = demo.pointer if demo.pointer is not None else (0, 0)
        print(f"pointing at tile ({q}, {r})")

    def on_key_press(symbol: int, _modifiers: int) -> None:
        key = pyglet.window.key
        if symbol == key.ESCAPE:
            pyglet.app.exit()
            return
        if symbol == key.SPACE:
            demo.clock.toggle()
            print(f"the world is {demo.clock.says()}")
            return
        if symbol == key.PERIOD:
            demo.clock.step_once()
            return
        if symbol == key.BRACKETLEFT:
            demo.clock.slower()
            print(f"speed {demo.clock.says()}")
            return
        if symbol == key.BRACKETRIGHT:
            demo.clock.faster()
            print(f"speed {demo.clock.says()}")
            return
        if symbol == key.F10:
            demo.settings.toggle()
            _show_settings(demo, window)
            return
        if symbol == key.F11:
            demo.settings.video.fullscreen = not demo.settings.video.fullscreen
            _apply_settings(demo, window)
            return
        if symbol == key.F12:
            demo.settings.video.next_size()
            _apply_settings(demo, window)
            return
        _toggle_panel_key(demo, symbol, key)

    # The handlers are registered by name rather than by decorator. The
    # library ships no type information, so a decorator from it would make
    # every function it wraps untyped.
    window.push_handlers(
        on_draw=on_draw,
        on_key_press=on_key_press,
        on_mouse_press=on_mouse_press,
    )

    pyglet.clock.schedule_interval(frame, 1.0 / FRAMES_EACH_SECOND)
    pyglet.app.run()
    window.close()

    print(f"stopped at tick {demo.world.tick}, state hash {demo.world.state_hash()}")
    return 0
