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

ADR-0094, the caller owns the camera and the pixels, decision D5.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

import argparse
import os
import secrets
import time
from typing import TYPE_CHECKING, Protocol

from cachette import Camera, ConfigError, World
from cachette.names import Names

if TYPE_CHECKING:
    from collections.abc import Callable

    # How a caller fills one frame: a camera, a size, the pixels and the
    # settings of the frame. The drawing method of the world has this shape,
    # and so does every other renderer.
    Renderer = Callable[..., "FrameReading"]

    # How a caller builds one picture: a width, a height, a byte layout, the
    # bytes and a pitch.
    MakePicture = Callable[[int, int, str, bytes, int], "Picture"]

    # These describe the shape of a dictionary the engine returns. They live
    # in the stub beside the compiled module and not in the module itself, so
    # importing them at run time would fail.
    from cachette._core import FoundingReport, FrameReading, GameEnd
from cachette.demo.clock import SPEEDS, Clock, says
from cachette.demo.minimap import Minimap
from cachette.demo.settings import Settings, load_video, save_video
from cachette.demo.sketch import RELIEF, BoundaryGap, Sketch
from cachette.demo.surface import Surface
from cachette.demo.toasts import Announcer

# The size of the window in pixels.
WINDOW_WIDTH = 960
WINDOW_HEIGHT = 720

# The world the demonstration builds. The engine is the same engine the tests
# exercise, and these numbers only choose which world it runs.
# The weather lattice of the demonstration, as the side of one cell in tiles.
#
# **The engine default of 32 gives this world an 8 by 8 lattice**, in which the
# deepest cell sits one cell from water. A watcher then sees no weather inland,
# because the world holds no inland at that pitch. Eight tiles gives 32 by 32,
# which carries a coast, an interior and a rain shadow, and costs a few
# milliseconds a tick. A watcher who wants one cell for each tile asks for it.
WEATHER_PITCH_DEFAULT = 8

WORLD_WIDTH = 256
WORLD_HEIGHT = 256
FACTION_COUNT = 4

# The seed of the world the demonstration opens, when the watcher names none.
#
# **A watcher gets a different world on every run.** One world seen many
# times teaches a reader that world rather than the engine, and a defect that
# one seed never produces stays invisible. The demonstration therefore draws
# a seed and prints it.
#
# The run stays reproducible. The engine gives one answer for one seed at any
# thread count, so a watcher who saw something worth keeping passes the
# printed number back with `--seed`.
WORLD_SEED = 0x0123_4567_89AB_CDEF

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

# The weather cell count above which the run warns the watcher.
#
# **The cost of the weather stage follows the cell count.** The default pitch
# gives the demonstration world 64 cells, which costs a small part of a tick.
# A pitch of one tile gives it one cell for every tile, and that dominates the
# frame. A watcher who asks for a fine pitch on a large world should read what
# it costs before the run grinds.
#
# The run warns and then proceeds. The pitch is what the watcher asked for,
# and a demonstration that refused would hide the thing it was asked to show.
#
# The number is a mark, not a measurement. No cost figure in this project is
# measured, so nothing here states one.
WEATHER_CELL_WARNING = 16384

# How many times the picture may resize before it gives up.
#
# Resizing changes what the window paints, and a section a count switches on
# adds lines, so one answer can move the next. Two passes settle it in
# practice. The bound stops a panel that never settles from looping.
RESIZES = 4

# The engine steps once for each drawn frame.
FRAMES_EACH_SECOND = 30

# How many promoted people the console names on one line.
#
# A mature world promotes several people on one tick. A line that named every
# one of them would run off the screen, and the count beside it already says
# how many there were.
NAMED_PROMOTIONS = 2

# The function keys the settings hold, by name.
#
# A panel never takes one of these. Two actions on one key means a watcher
# cannot reach the second of them.
SETTINGS_KEYS = ("F10", "F11", "F12")

# The highest function key the mapping looks for.
#
# The window library names the function keys up to this number, so a deck
# that grows past the keys below it still finds one for each panel.
LAST_FUNCTION_KEY = 20


def panel_keys(key: object) -> list[tuple[int, str]]:
    """Give back one key for each panel the engine registers, in its order.

    The function keys name the panels, and the keys the settings hold are
    skipped. **The list is as long as the panels the engine registers**, so a
    panel that joins the deck gets a key with no edit here.

    Each entry holds the key symbol and the name of the key.
    """
    free = [
        (getattr(key, f"F{number}"), f"F{number}")
        for number in range(1, LAST_FUNCTION_KEY + 1)
        if hasattr(key, f"F{number}") and f"F{number}" not in SETTINGS_KEYS
    ]
    return free[: len(World.panel_names())]


# The name of the key that turns the overlay off.
#
# A watcher needs one key that means "show me the map again", and it must not
# be an overlay itself. The number keys run 1 upward, so 0 is free.
OVERLAY_OFF_KEY = "0"


def overlay_keys(key: object) -> list[tuple[int, str]]:
    """Give back one key for each overlay the engine registers, in its order.

    The number keys name the overlays. The panel deck holds the function keys,
    and two actions on one key means a watcher cannot reach the second of them.

    **The list is as long as the overlays the engine registers**, so an overlay
    that joins the deck gets a key with no edit here.

    Each entry holds the key symbol and the name of the key.
    """
    free = [
        (getattr(key, f"_{number}"), str(number))
        for number in range(1, 10)
        if hasattr(key, f"_{number}")
    ]
    # Nine digits ran out when the tenth overlay arrived, and 0 already means
    # "show the map again". These letters carry no other action in the window.
    free += [
        (getattr(key, letter.upper()), letter)
        for letter in ("y", "u", "i", "o", "p")
        if hasattr(key, letter.upper())
    ]
    return free[: len(World.overlay_names())]


class Demo:
    """The state the control plane holds between frames.

    The engine holds none of it. The camera, the window size and the founding
    report belong to the caller, and the caller hands the camera back on every
    frame.
    """

    __slots__ = (
        "announced_end",
        "announcer",
        "camera",
        "clock",
        "minimap",
        "names",
        "overlay",
        "panels",
        "pointer",
        "reference",
        "renderer",
        "seconds",
        "settings",
        "surface",
        "threads",
        "world",
    )

    def __init__(
        self,
        world: World,
        names: Names,
        width: int = WINDOW_WIDTH,
        height: int = WINDOW_HEIGHT,
        threads: int = 0,
    ) -> None:
        """Build the state the control plane holds between frames.

        The namer turns an index, an address and an identity into words. It
        comes from the caller and not from the world, because a namer holds
        more than a seed. The caller builds it from the seed of the world,
        which the world now gives back, so one number reaches both.
        """
        self.world = world
        self.names = names
        # **The renderer fills one frame and reports what it read.** The
        # engine is the one a watcher opens on, and a flag replaces it with
        # another that takes the same arguments. Nothing else in the loop
        # knows which one is here.
        self.renderer: Renderer = world.draw
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
        # The overlay the map shows, by name, or None for the plain map. The
        # engine holds the list of names, so this cannot name one that does
        # not exist.
        self.overlay: str | None = None
        # The tile the watcher pointed at, in axial coordinates. The engine
        # has no cursor, so the control plane supplies one.
        self.pointer: tuple[int, int] | None = None
        # Whether the game end was printed. The record is written once, and
        # the line is printed once.
        self.announced_end = False
        # The round wide view in the top right corner. It reads the summary
        # level of the engine and paints over the frame, and it holds its own
        # reading between frames.
        self.minimap = Minimap()
        # The lines that appear over the map, and the reader that makes them.
        self.announcer = Announcer(names)
        # Where the deck reads the wall clock. **A toast lives for a number of
        # seconds and not for a number of ticks**, because a tick lasts
        # thirty-two times longer at the slowest speed than at the fastest,
        # and a paused world runs no tick at all. A caller replaces this to
        # drive the fade from a number it holds.
        self.seconds: Callable[[], float] = time.monotonic

    def seed(self) -> list[FoundingReport]:
        """Seed the world from its seed, and give back what each faction got.

        **The demonstration calls no seeding verb.** The engine founds every
        faction and places the luxuries from the seed in one call that takes
        nothing. The engine keeps the report for the panel, and the caller
        gets a summary to print.
        """
        return self.world.seed_world()

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
        """Say when a soldier becomes a character, and name the person.

        **The control plane reacts to one fact the engine reported.** It reads
        the count the frame gave it and prints a line. It walks no entity and
        asks the engine nothing further, so this is a reaction and not a poll.

        A promotion happens on a small share of frames, so the line is rare
        enough to read and it names the moment rather than a total that went
        up.

        The line names the people the promotion log holds, up to a few of
        them. A mature world promotes several people on one tick, and a line
        that named every one of them would run off the screen. The count is
        still there, so a reader knows how many the line did not name.
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
        named = self._promoted_names()
        if named:
            print(f"  they are {named}")

    def _promoted_names(self) -> str:
        """Give back the names of the people the last step promoted.

        The log covers the last step alone. A step that promoted more people
        than the line holds ends with a count of the rest.
        """
        columns = self.world.promoted_log_columns()
        rows = len(columns["character"])
        if rows == 0:
            return ""
        shown = [
            f"{self.names.person(int(columns['character'][row]))} of "
            f"{self.names.faction(int(columns['faction'][row]))}"
            for row in range(min(rows, NAMED_PROMOTIONS))
        ]
        rest = rows - len(shown)
        if rest > 0:
            shown.append(f"and {rest} more")
        return ", ".join(shown)

    def announce_relations(self) -> None:
        """Say who declared war on whom, and who made peace, on the last step.

        **The control plane reads one log and walks no entity.** The engine
        writes one event for each ordered pair whose relation crossed the war
        edge on the last step. A band after the move below the band before
        is a declaration, and a band above is a peace. The band numbers
        count the edges at or below the value, so the line names no edge.
        """
        columns = self.world.relation_log_columns()
        for row in range(len(columns["tick"])):
            tick = int(columns["tick"][row])
            speaker = int(columns["from_faction"][row])
            other = int(columns["to_faction"][row])
            declared = int(columns["band_after"][row]) < int(
                columns["band_before"][row]
            )
            verb = "declares war on" if declared else "makes peace with"
            print(
                f"tick {tick}: {self.names.faction(speaker)} {verb} "
                f"{self.names.faction(other)}"
            )

    def announce_campaigns(self) -> None:
        """Say who marched on what, and who took what, on the last step.

        **The control plane reads one log and walks no entity.** The engine
        writes one event when a campaign is raised and one when it closes.
        A raise prints the objective and the cohort. A win prints the
        objective. A loss and an end print nothing, because a watcher wants
        the moment and not the bookkeeping.
        """
        columns = self.world.campaign_log_columns()
        for row in range(len(columns["tick"])):
            tick = int(columns["tick"][row])
            faction = int(columns["faction"][row])
            kind = int(columns["kind"][row])
            q = int(columns["objective_q"][row])
            r = int(columns["objective_r"][row])
            # **The console keeps the address beside the name.** A watcher
            # reads a name to follow the story and reads an address to point
            # the camera, and the console is where the second one belongs.
            place = f"{self.names.place(q, r)} ({q}, {r})"
            nation = self.names.faction(faction)
            if kind == 0:
                cohort = int(columns["cohort_size"][row])
                army = self.names.faction_adjective(faction)
                print(
                    f"tick {tick}: a {army} army marches on {place} "
                    f"with {cohort} soldiers"
                )
            elif kind == 1:
                print(f"tick {tick}: {nation} takes {place}")

    def announce_trade(self) -> None:
        """Say when a contract bound and when one reached full delivery.

        **The control plane reads one log and walks no entity.** The engine
        writes one entry for each speech act and for each settlement of the
        last step. An acceptance is act two and a settlement is act six. The
        line names the two parties and nothing else, because a watcher wants
        the moment and not the terms.
        """
        columns = self.world.trade_log_columns()
        for row in range(len(columns["tick"])):
            act = int(columns["act"][row])
            if act not in (2, 6):
                continue
            tick = int(columns["tick"][row])
            proposer = int(columns["proposer"][row])
            responder = int(columns["responder"][row])
            verb = "binds a contract with" if act == 2 else "completes a contract with"
            print(
                f"tick {tick}: {self.names.faction(proposer)} {verb} "
                f"{self.names.faction(responder)}"
            )

    def announce_end(self) -> GameEnd | None:
        """Say who won, once, when the game end record first appears.

        The record is engine state, and the world keeps stepping after it.
        This reads one record and prints one line the first time it is set.
        """
        end = self.world.game_end()
        if end is None or self.announced_end:
            return end
        self.announced_end = True
        # The engine names a path with underscores. A watcher reads words.
        path = end["path"].replace("_", " ")
        winner = self.names.faction(int(end["winner"]))
        print(f"tick {end['tick']}: {winner} wins by {path}")
        return end

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

    def choose_overlay(self, name: str | None) -> None:
        """Show one overlay on the map, or show the plain map.

        The engine names the overlays it can draw, so a name that no overlay
        carries is refused here rather than at the drawing.
        """
        if name is not None and name not in World.overlay_names():
            message = f"no overlay is called {name!r}"
            raise ValueError(message)
        self.overlay = name

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

        The frame takes the phase and the speed from the clock. At a speed
        below one tick for each frame a unit that moved draws between its two
        tiles, and the frame states the speed beside the tick.
        """
        now = self.seconds()
        for _ in range(self.clock.ticks_due()):
            self.world.step(self.threads)
            self.announce_relations()
            self.announce_campaigns()
            self.announce_trade()
            # The logs cover the last step alone, so the deck reads them here
            # and not after the loop. A frame that ran several ticks would
            # otherwise keep the last of them only.
            self.announcer.after_step(self.world, now)
        self.announce_end()
        # The pace is the clock's, and the engine holds no clock. The phase
        # is the share of the current tick that has elapsed, and the frame
        # draws a unit that moved between its two tiles at that share. The
        # speed reaches the frame as a number, and the viewer holds the
        # words, so no text crosses the boundary.[^5]
        #
        # The ticks above ran first, so the phase belongs to the tick the
        # world is now part way through.
        #
        # **The renderer is a choice, and the frame is one call.** The engine
        # is the renderer a watcher opens on. A flag puts another one here,
        # and everything around this line stays as it was.
        reading = self.renderer(
            self.camera,
            self.surface.width,
            self.surface.height,
            self.surface.pixels,
            reference=self.reference,
            panel=panel,
            panels=self.panels or None,
            pointer=self.pointer,
            overlay=self.overlay,
            phase=self.clock.phase,
            speed_milli=self.clock.speed_milli,
        )
        self.announce(reading)
        # The counters and the end record are state, so the deck reads them
        # once for each drawn frame.
        self.announcer.after_frame(self.world, now)
        # **The toasts go on last, over the frame the engine filled.** They
        # are chrome and not the world: nothing here reads a tile or an
        # entity, so the two drawing paths cannot disagree about the world.
        self.announcer.toasts.paint(self.surface, now)
        # **The minimap goes on last, and it stands down for the key.** The
        # engine puts a card in the top right corner while the reference key
        # is held, and two things in one corner means a watcher reads
        # neither.
        if not self.reference:
            self.minimap.paint(self.world, self.camera, self.surface)
        return reading


def draw_seed() -> int:
    """Give back a seed for a world nobody named.

    The draw is not part of the simulation. It chooses which world runs, and
    the engine then gives one answer for that world at any thread count. A
    caller that wants one world names it instead.
    """
    return secrets.randbits(64)


def build_world(
    extent: int = 0,
    factions: int = FACTION_COUNT,
    seed: int = WORLD_SEED,
    weather_pitch: int = 0,
) -> World:
    """Build the world the demonstration runs.

    The extent is the side of the world in tiles. Zero takes the default
    world, which is the one a watcher opens.

    The seed chooses which world. It keeps the stated default, so a caller
    that names no seed gets one world every time. The command line draws a
    seed before it calls this.

    The weather pitch is the side of one weather cell in tiles. Zero takes
    the pitch the engine defaults to, so this function states no default of
    its own. One gives each tile its own weather cell.
    """
    side = extent if extent > 0 else WORLD_WIDTH
    return World(
        width=side,
        height=side if extent > 0 else WORLD_HEIGHT,
        seed=seed,
        faction_count=factions,
        weather_cell_tiles=(
            weather_pitch if weather_pitch > 0 else WEATHER_PITCH_DEFAULT
        ),
    )


def weather_line(world: World) -> str:
    """Give back the line that says what pitch the weather runs at.

    **Two runs at two pitches look alike on the map.** A watcher comparing
    them needs to read which one is on the screen, so the line names the
    pitch in tiles and the cell count that follows from it.
    """
    pitch = world.weather_cell_tiles
    cells = world.weather_cell_count
    how = "one cell for each tile" if pitch == 1 else f"{pitch} tiles a side"
    return f"weather: {how}, {cells} cells"


def print_weather_pitch(world: World) -> None:
    """Print the weather pitch, and warn when the lattice is a costly one.

    The warning states the cell count against the mark and then lets the run
    proceed. The pitch is what the watcher asked for, and a demonstration
    that refused would hide the thing it was asked to show.
    """
    print(weather_line(world))
    cells = world.weather_cell_count
    if cells > WEATHER_CELL_WARNING:
        print(
            f"note: {cells} weather cells is above {WEATHER_CELL_WARNING}, "
            "so the weather stage dominates each tick and the run is slow"
        )


def print_census(world: World) -> None:
    """Print what every subsystem produced, one line each.

    The engine holds the list of subsystems in one table, and this walks the
    dictionary that table produced. No name is written here.

    No count covers one tick. A count says what the world holds now, or what
    the run has made since it started. A zero of the second kind means that
    the thing never happened.
    """
    census = world.subsystem_census()
    print(f"census of the run at tick {world.tick}")
    for name, count in census.items():
        print(f"  {name}: {count}")


def report(foundings: list[FoundingReport], names: Names) -> tuple[int, int]:
    """Print what each faction got, and give back how many were seated and fed.

    The loop is over the foundings, and the run makes one for each faction.
    The caller sets the faction count, so this loop follows that count and
    names no number of its own. It is not a loop over entities, and it reads a
    summary the engine already made.

    **The line keeps the address beside the name.** A watcher reads the name
    to follow the story and reads the address to point the camera at the
    place, and this report is where the second one belongs.
    """
    seated = 0
    carried = 0
    for founding in foundings:
        faction = founding["faction"]
        nation = names.faction(faction)
        if not founding["seated"]:
            print(f"{nation} found no place: {founding['refusal']}")
            continue
        seated += 1
        place = names.place(founding["q"], founding["r"])
        print(
            f"{nation} founds {place} at ({founding['q']}, {founding['r']}) "
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
            print(f"  this ground carries its group of {founding['people']}")
        else:
            print(
                f"  this ground carries {founding['food']} of its group "
                f"of {founding['people']}, and the rest go short"
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
    parser.add_argument(
        "--width",
        type=int,
        default=0,
        help=(
            "the window width; the run opens at the size it last saved by "
            "default, and this width governs this run without saving"
        ),
    )
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
        "--tile",
        type=float,
        default=0.0,
        help=(
            "the size of a tile in pixels; zero keeps the size a watcher "
            "opens on, and a smaller number shows a wider region"
        ),
    )
    parser.add_argument(
        "--overlay",
        default="",
        help=(
            "show one overlay on the map; the engine names the overlays it "
            "can draw, and it refuses a name it did not publish"
        ),
    )
    parser.add_argument(
        "--panels",
        default="",
        help=(
            "draw these panels of the deck, separated by commas; the engine "
            "names the panels it can draw, and it refuses a name it did not "
            "publish"
        ),
    )
    parser.add_argument(
        "--point",
        default="",
        help=(
            "point at this tile, as two numbers separated by a comma, so that "
            "the tile panel reads it without a mouse"
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
    parser.add_argument(
        "--run-to-end",
        action="store_true",
        help=(
            "open no window; step until the game ends or the tick limit "
            "passes, then print the winner, the path, the tick and the census"
        ),
    )
    parser.add_argument(
        "--tick-limit",
        type=int,
        default=0,
        help="the tick at which the territory reader fires; zero keeps the default",
    )
    parser.add_argument(
        "--extent",
        type=int,
        default=0,
        help="the side of the world in tiles; zero keeps the default world",
    )
    parser.add_argument(
        "--factions",
        type=int,
        default=FACTION_COUNT,
        help="how many factions the world holds",
    )
    parser.add_argument(
        "--weather-pitch",
        type=int,
        default=0,
        help=(
            "the side of one weather cell in tiles, as a power of two from 1 "
            "to 256; 1 gives each tile its own weather and is slow on a large "
            "world; zero takes the pitch the demonstration chooses"
        ),
    )
    parser.add_argument(
        "--sketch",
        action="store_true",
        help=(
            "draw the world as an isometric pencil study in ink on paper, "
            "lifted by the height of the ground, instead of as a flat map; "
            "the frame costs seconds rather than milliseconds, so a window "
            "in this mode draws slowly"
        ),
    )
    parser.add_argument(
        "--sketch-relief",
        type=float,
        default=0.0,
        help=(
            "how far the tallest ground rises in the sketch, as a share of "
            "the width of the page; zero takes the share the sketch chooses"
        ),
    )
    parser.add_argument(
        "--sketch-sky",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="draw the cloud layer over the ground in the sketch",
    )
    parser.add_argument(
        "--seed",
        type=lambda given: int(given, 0),
        default=0,
        help=(
            "the seed of the world; zero draws one, and the run prints the "
            "number it drew so that you can ask for the same world again"
        ),
    )
    arguments = parser.parse_args(argv)
    seed = arguments.seed if arguments.seed else draw_seed()

    # The panels a watcher named, and the tile a watcher named. Both are
    # read before the world is built, so a name that no panel carries stops
    # the run before it steps.
    try:
        panels = chosen_panels(arguments.panels)
        pointer = chosen_tile(arguments.point)
    except ValueError as refusal:
        # A refusal is an answer to the watcher, not a defect in the engine.
        # The message names what is wrong, and the run stops before it steps.
        print(refusal)
        return 2

    # The panel holds every section and is taller than a window a person
    # opens. A picture that used the window height would cut the last
    # sections and say so, which is honest and still less than was asked for.
    #
    # A deck cuts itself to the frame it is drawn in, so a picture of a deck
    # keeps the height of a window.
    tall = bool(arguments.picture) and not panels
    default_height = PICTURE_HEIGHT if tall else WINDOW_HEIGHT
    # **One number builds the world and names the things in it.** The namer
    # now takes the seed from the world, so the number is declared once. A
    # world built from one seed and a namer built from another would name a
    # story that did not happen, and nothing would fail.
    # The engine holds the rule for what describes a world, and it refuses
    # here rather than in a traceback. A watcher who typed a weather pitch of
    # three reads one sentence and tries again.
    try:
        world = build_world(
            arguments.extent,
            arguments.factions,
            seed,
            arguments.weather_pitch,
        )
    except ConfigError as refusal:
        print(f"the engine refused the world: {refusal}")
        return 2
    # **A run opens in the video state the last run saved.** A file that is
    # absent or damaged gives the opening state back, so the memory can never
    # stop the run.
    saved = load_video()
    # A picture and a run to the end open no window, so the saved window state
    # governs neither. Both keep the sizes they always had.
    opens_window = not arguments.picture and not arguments.run_to_end
    # **A size on the command line governs this run, and it saves nothing.**
    # The watcher who types a size asks for one run at that size. The size the
    # watcher last chose by hand stays in the file, and the next run without a
    # size opens at it.
    typed_size = arguments.width > 0 or arguments.height > 0
    width = arguments.width or WINDOW_WIDTH
    height = arguments.height or default_height
    if opens_window and not typed_size:
        # A saved size can be larger than the display of this run. The window
        # library knows the display, and it gives no size without one.
        display = screen_size()
        if display is not None:
            saved.fit_within(*display)
        width, height = saved.size
    demo = Demo(
        world,
        Names(world.seed),
        width=width,
        height=height,
        threads=arguments.threads,
    )
    demo.settings.video = saved
    if arguments.tick_limit > 0:
        demo.world.set_tick_limit(arguments.tick_limit)
    # The seed comes first, before any other line. A run that ends badly is
    # worth repeating, and the number that repeats it must already be on the
    # screen when it does.
    print(f"seed 0x{seed:016x}")
    # The pitch comes before the founding, because a run at a fine pitch on a
    # large world is slow from the first tick and the warning is worth
    # nothing after it.
    print_weather_pitch(demo.world)
    foundings = demo.seed()
    seated, _ = report(foundings, demo.names)
    if seated == 0:
        print("no faction found a place, so there is nothing to watch")
        return 1
    demo.open_on(opening_place(foundings))

    if arguments.run_to_end:
        return _run_to_end(demo)

    if arguments.tile > 0:
        demo.camera = Camera(tile_size=arguments.tile)
        demo.open_on(opening_place(foundings))

    if arguments.sketch:
        # **The sketch is a renderer, not a second demonstration.** It takes
        # the place of the engine at the one call that fills a frame, and the
        # clock, the panels, the keys and the window memory stay shared.
        print(
            "the sketch renderer draws one frame in seconds, not in "
            "milliseconds, so the window will feel slow"
        )
        try:
            demo.renderer = Sketch(
                demo.world,
                relief=arguments.sketch_relief or RELIEF,
                sky=arguments.sketch_sky,
            )
        except BoundaryGap as gap:
            print(f"the sketch renderer cannot run: {gap}")
            return 2

    if arguments.overlay:
        demo.choose_overlay(arguments.overlay)

    for name in panels:
        demo.toggle_panel(name)
    demo.pointer = pointer

    if arguments.picture:
        status = _write_picture(demo, arguments.picture, arguments.ticks)
        print_census(demo.world)
        return status

    print(
        f"cachette: {demo.world.width} by {demo.world.height} tiles, "
        f"{demo.world.soldier_count} people, {demo.threads} threads"
    )
    print("arrow keys or WASD scroll, minus and equals zoom")
    print("hold tab to name the colours")
    print("m shows and hides the minimap")
    print("space pauses, full stop steps one tick, brackets change the speed")
    print(f"the speeds are {', '.join(says(speed) for speed in SPEEDS)}")
    print(
        f"{SETTINGS_KEYS[0]} opens the settings, {SETTINGS_KEYS[1]} fullscreen, "
        f"{SETTINGS_KEYS[2]} window size"
    )
    print(f"the panels are {_panel_key_line()}")
    print(f"the overlays are {_overlay_key_line()}")
    print("click a tile to point at it")
    print("close the window or press escape to stop")

    status = _run_window(demo, arguments.frames, restore_size=not typed_size)
    print_census(demo.world)
    return status


def _panel_key_line() -> str:
    """Give back the line that names the key of each panel.

    **The line and the keys come from one mapping.** A line written from a
    second count would name a key that does nothing the moment the deck
    grows.

    The import is here and not at the top of the module, so a caller that
    only wants a frame in memory needs no window library.
    """
    from pyglet.window import key

    pairs = zip(panel_keys(key), World.panel_names(), strict=True)
    return ", ".join(f"{label} {name}" for (_, label), name in pairs)


def _overlay_key_line() -> str:
    """Give back the line that names the key of each overlay.

    **The line and the keys come from one mapping.** A line written from a
    second count would name a key that does nothing the moment the deck grows.

    The import is here and not at the top of the module, so a caller that only
    wants a frame in memory needs no window library.
    """
    from pyglet.window import key

    keys = overlay_keys(key)
    names = World.overlay_names()
    # The two lists may differ in length, and the line below names what is
    # left over, so the pairing stops at the shorter of the two on purpose.
    named = ", ".join(
        f"{label} {name}" for (_, label), name in zip(keys, names, strict=False)
    )
    # An overlay past the last free key is still real, and a watcher who cannot
    # see it named would believe the renderer holds fewer than it does.
    spare = names[len(keys) :]
    if spare:
        named += ", no key for " + ", ".join(spare)
    return f"{named}, {OVERLAY_OFF_KEY} none"


def _run_to_end(demo: Demo) -> int:
    """Step the world until the game ends, then print the end and the census.

    This presenter needs no window library and no display. It drives no
    verb: the world seeds itself, the controller inside the step plays, and
    this reads the record when it appears. The tick limit bounds the run, so
    a game that no reader ends stops at the limit and says so.
    """
    limit = demo.world.tick_limit
    end = None
    while end is None and demo.world.tick < limit:
        demo.world.step(demo.threads)
        demo.announce_relations()
        demo.announce_campaigns()
        demo.announce_trade()
        end = demo.announce_end()
    if end is None:
        print(f"no game ended by the tick limit of {limit}")
    else:
        winner = demo.names.faction(int(end["winner"]))
        print(
            f"the game ended at tick {end['tick']}: {winner} "
            f"won by {end['path']}, holding {demo.world.score(end['winner'])} tiles"
        )
    print_census(demo.world)
    return 0


def chosen_panels(named: str) -> list[str]:
    """Give back the panels a watcher named on the command line.

    The names are separated by commas. An empty text names no panel, and the
    frame then draws the cards.

    The engine holds the list of panels, so a name that no panel carries is
    refused here and the message names the panels that exist.
    """
    wanted = [name.strip() for name in named.split(",") if name.strip()]
    known = World.panel_names()
    for name in wanted:
        if name not in known:
            message = f"no panel is called {name!r}; the panels are {', '.join(known)}"
            raise ValueError(message)
    return wanted


def chosen_tile(named: str) -> tuple[int, int] | None:
    """Give back the tile a watcher named on the command line.

    The address is two whole numbers separated by a comma. An empty text names
    no tile, and the tile panel then says that nobody pointed.
    """
    if not named.strip():
        return None
    parts = named.split(",")
    if len(parts) != 2:
        message = f"a tile is two numbers separated by a comma, not {named!r}"
        raise ValueError(message)
    return (int(parts[0]), int(parts[1]))


def _write_picture(demo: Demo, path: str, frames: int) -> int:
    """Step the world, then write one frame with the whole panel to a file.

    This presenter needs no window library and no display. It is the same
    frame command the window uses, so the picture holds what the window would
    have shown, with the sections the cards leave out.

    A watcher who named a deck gets that deck. The deck cuts itself to the
    frame it is drawn in, so the picture keeps the height it was given and the
    resize below is for the whole panel only.
    """
    deck = bool(demo.panels)
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
    reading = demo.advance(panel=not deck)

    # Ask the panel how tall it needed to be, and draw again at that height.
    # The loop ends when the picture is tall enough for the panel it drew.
    for attempt in range(0 if deck else RESIZES):
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

    The keys come from the one mapping, so the key a watcher reads at the
    start of a run is the key that works.
    """
    names = World.panel_names()
    for at, (bound, _) in enumerate(panel_keys(key)):
        if symbol == bound and at < len(names):
            demo.toggle_panel(names[at])
            shown = ", ".join(demo.panels) if demo.panels else "none"
            print(f"panels: {shown}")
            return


def _choose_overlay_key(demo: Demo, symbol: int, key: object) -> None:
    """Put one overlay on the map, or take the overlay off.

    The keys come from the one mapping, so the key a watcher reads at the start
    of a run is the key that works.

    The line names the overlay when it changes. A watcher who switched the map
    must be told what the map now shows, because the colours alone do not say.
    """
    if hasattr(key, "_0") and symbol == key._0:
        demo.choose_overlay(None)
        print("overlay: none")
        return
    names = World.overlay_names()
    for at, (bound, _) in enumerate(overlay_keys(key)):
        if symbol == bound and at < len(names):
            demo.choose_overlay(names[at])
            print(f"overlay: {names[at]}")
            return


def window_size(window: object, fallback: tuple[int, int]) -> tuple[int, int]:
    """Give back the size the window reports, or the fallback size.

    **The size the settings hold is the windowed size, and a fullscreen window
    is the size of the screen.** A surface built from the setting would then
    fill a small part of a large window, and the picture would stay small.

    The window library ships no type information, so this reads the two
    attributes and falls back when either is missing.
    """
    width = getattr(window, "width", None)
    height = getattr(window, "height", None)
    if type(width) is int and type(height) is int and width > 0 and height > 0:
        return (width, height)
    return fallback


def screen_size() -> tuple[int, int] | None:
    """Give back the size of the display in pixels, or None without one.

    The import is here and not at the top of the module, so a caller that only
    wants a frame in memory needs no window library.

    A box with no display gives None. The window library reaches the display
    through the operating system, and it raises in several ways when there is
    none, so this catches the failure rather than the type of it.
    """
    try:
        from pyglet.display import get_display

        screen = get_display().get_default_screen()
        return (int(screen.width), int(screen.height))
    except Exception:
        return None


def _apply_settings(
    demo: Demo,
    window: object,
    with_size: bool = True,
    remember: bool = True,
) -> None:
    """Give the window the video settings, and resize the pixels to match.

    The surface is the memory the engine fills. A window of a new size needs a
    surface of that size, so the two are changed together.

    The surface follows the size the window reports, not the size the settings
    hold. The two differ while the window is fullscreen.

    **Only a key press writes the file.** This is the one place a video setting
    reaches the window, so it is the one place the file needs a write. A key
    press writes the file at once, so a run that stops badly keeps the change
    the watcher just made. No frame writes the file, because no frame comes
    through here.

    The caller that opens the window passes False for the memory. A size that
    this display forced down, and a size that the command line named, both
    govern this run alone. Neither replaces the size the watcher last chose by
    hand.
    """
    refused = demo.settings.apply_to(window, with_size=with_size)
    if refused:
        print(f"the window refused: {', '.join(refused)}")
    if remember and not save_video(demo.settings.video):
        print("the video settings were not saved")
    width, height = window_size(window, demo.settings.video.size)
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


class Picture(Protocol):
    """What the demonstration needs of the picture a window draws.

    The window library ships no type information, so this states the two
    methods the loop calls rather than naming a class of that library.
    """

    def set_data(self, layout: str, pitch: int, data: bytes) -> None:
        """Take a new frame."""

    def blit(self, x: int, y: int) -> None:
        """Draw the frame at this place in the window."""


class WindowPicture:
    """The picture a window draws, and the surface size it was built for.

    **The size of the surface is declared here once.** The picture holds a
    width, a height and a pitch, and each follows the size of the surface. A
    picture that a new surface outgrew takes a buffer of the wrong length, and
    the window library reports a length it cannot use.

    The engine writes the first row of the frame first, and the window numbers
    its rows from the bottom. A negative pitch says so.

    The caller passes a maker, because the window library ships no type
    information and a test has no display to open a window on. The maker takes
    a width, a height, a format, the bytes and a pitch.
    """

    __slots__ = ("_make", "image", "pitch", "size")

    image: Picture
    pitch: int
    size: tuple[int, int]

    def __init__(self, make: MakePicture, surface: Surface) -> None:
        """Build the first picture from this surface."""
        self._make = make
        self._build(surface)

    def _build(self, surface: Surface) -> None:
        """Build a picture for the size of this surface."""
        self.size = (surface.width, surface.height)
        self.pitch = -surface.width * 4
        self.image = self._make(
            surface.width,
            surface.height,
            "BGRA",
            surface.to_bytes(),
            self.pitch,
        )

    def update(self, surface: Surface) -> None:
        """Put the frame in the picture, and rebuild it on a new size."""
        if (surface.width, surface.height) != self.size:
            self._build(surface)
            return
        self.image.set_data("BGRA", self.pitch, surface.to_bytes())


def _run_window(demo: Demo, frame_limit: int, restore_size: bool = True) -> int:
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

    def make_image(
        width: int, height: int, layout: str, data: bytes, pitch: int
    ) -> Picture:
        image: Picture = pyglet.image.ImageData(
            width, height, layout, data, pitch=pitch
        )
        return image

    # The window opens at the size the caller chose, so the size is already
    # right and only the other two settings need a call. A fullscreen window
    # takes no size at all, and the settings order the fullscreen call before
    # the size call for that reason.
    _apply_settings(demo, window, with_size=restore_size, remember=False)
    picture = WindowPicture(make_image, demo.surface)
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
        picture.update(demo.surface)
        counted[0] += 1
        if frame_limit and counted[0] >= frame_limit:
            pyglet.app.exit()

    def on_draw() -> None:
        window.clear()
        picture.image.blit(0, 0)

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
        if symbol == key.M:
            shown = demo.minimap.toggle()
            print(f"the minimap is {'on' if shown else 'off'}")
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
        _choose_overlay_key(demo, symbol, key)
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
