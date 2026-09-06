"""Short lines that appear over the world when something happens.

A toast is one sentence a watcher reads without looking away from the map. It
appears at the moment of the thing, it stays for a few seconds and it fades.
It is not the events panel, which is a list a watcher must choose to open.

**A toast lives for a number of seconds, not for a number of ticks.** The
demonstration runs from a quarter of a tick each frame to eight ticks each
frame, so one tick is thirty-two times longer at the slow end than at the
fast end. A toast measured in ticks would sit on the screen for half a minute
at quarter speed and flash past at eight times speed. A paused world runs no
tick at all, so a toast measured in ticks would never leave. A second is the
rate a person reads at, and it is the same at every speed.

**Every one of these comes from an event log the engine writes.** A log holds
what happened since the last step began, so the caller reads it after each
step and a caller that misses a step misses the events. The engine publishes
the logs by name, so this module asks for a log by its name and holds no
method for each one.

**A toast is for a moment, not for a rate.** A thing that happens every few
ticks in a mature world belongs in the events panel and not over the map. A
watcher learns to ignore a screen that always has a line on it.

References
----------
ADR-0067, the viewer reads the world and never writes to it, decision D2.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from cachette import faction_colours
from cachette.demo.text import paint_block, paint_text, text_height, text_width

if TYPE_CHECKING:
    from cachette import World
    from cachette.demo.surface import Surface

# One colour for each faction, so a watcher tells who without reading.
#
# **The viewer states these once and this module reads them.** The demo held
# its own copy of the six numbers, so the table stood in two places and
# nothing failed when they disagreed.
FACTION_COLOURS: tuple[int, ...] = tuple(faction_colours())

# The colour of a line that names no faction.
NEUTRAL_COLOUR = 0xF0E8D8

# The holder value that names nobody. It sits above the faction ceiling, so
# no faction collides with it.
NOBODY = 65535

# The upgrade category numbers, as the engine numbers them, and the word a
# watcher reads for each one.
CATEGORY_WONDER = 2
CATEGORY_WORDS: dict[int, str] = {
    0: "road",
    1: "terrace",
    2: "wonder",
    3: "storehouse",
    4: "wall",
    5: "lodging",
}

# What ended an upgrade, as the engine numbers it.
CAUSE_WORDS: dict[int, str] = {
    1: "the weather",
    2: "an army",
    3: "the weather and an army",
    4: "an order",
}

# The counters a rise is read from, and the words for one and for several.
#
# **A subsystem that publishes a log is read from the log, not from here.** A
# counter says that a thing happened and it does not say who did it or where.
# This tuple is empty, because every subsystem the deck watches now publishes
# a log. It stays, so a counter that has no log yet has a home.
WATCHED_COUNTERS: tuple[tuple[str, str, str], ...] = ()

# The colour behind a line, so the words read over any ground.
BACKING_COLOUR = 0x101014

# How much of the backing colour covers the map behind a line.
BACKING_WEIGHT = 0.72

# How long a line stays on the screen, in seconds, and how much of that time
# it spends fading. A watcher reads a short sentence in about two seconds, so
# five holds it long enough to catch a line that landed while the eye was
# elsewhere.
LIFETIME_SECONDS = 5.0
FADE_SECONDS = 1.5

# How many lines the screen holds at once.
#
# **A busy tick must not cover the map.** Four lines take a fifth of the
# height of the window a watcher opens, and the map stays readable under
# them. A fifth line takes the place of the oldest of the four. **Nothing
# waits in a queue**: a queue would show a line seconds after its moment, and
# a toast that arrives late says the wrong thing about now. A line the deck
# has no room for is dropped, and the events panel still holds it.
MOST_SHOWN = 4

# How long a line is safe from eviction, in seconds.
#
# **A busy world must not push a line off before anybody reads it.** At eight
# ticks for each frame the world makes events far faster than a person reads
# them, and a deck that always took the newest would flicker. A new line that
# finds every slot younger than this is dropped rather than queued, unless it
# outranks the oldest line. The events panel still holds everything.
MINIMUM_SECONDS = 1.2

# The two ranks a line carries.
#
# A moment is a thing that happened between two factions or to a campaign. A
# milestone is a total that crossed a round number. A moment pushes a
# milestone off the screen, and a milestone never pushes a moment off.
RANK_MILESTONE = 0
RANK_MOMENT = 1

# How long the deck refuses a second copy of the same sentence, in seconds.
#
# A counter that rises on many ticks in a row would otherwise write one line
# for each tick. The second copy refreshes the line that is already there, so
# the thing stays on the screen without stacking.
SAME_AGAIN_SECONDS = 4.0

# How many people a faction gains between two population lines.
#
# A line for each birth would be a stream. A round number is a milestone, and
# this one fires a few times for each faction over a long run.
POPULATION_MARK = 25

# The text scale, and the spacing of the lines, in pixels.
SCALE = 2
PADDING = 5
LINE_GAP = 4

# How far the lowest line sits above the foot of the frame.
#
# The engine writes a strip of key names along the foot of the window, and a
# line over it would hide the strip.
FOOT_MARGIN = 34


def faction_colour(faction: int) -> int:
    """Give back the colour the viewer paints this faction in.

    A faction beyond the table wraps to a colour it shares, in the same way
    the viewer wraps. That is a display limit and not a simulation one.
    """
    return FACTION_COLOURS[faction % len(FACTION_COLOURS)]


class Toast:
    """One line, its colour and the second it appeared."""

    __slots__ = ("born", "colour", "key", "rank", "text")

    def __init__(
        self, text: str, colour: int, key: str, born: float, rank: int
    ) -> None:
        """Build one line that appeared at this second."""
        self.text = text
        self.colour = colour
        self.key = key
        self.born = born
        self.rank = rank

    def weight(self, now: float) -> float:
        """Give back how much of the colour reaches the pixels now.

        A line is at full colour until the fade starts, and it falls to
        nothing at the end of its life.
        """
        age = now - self.born
        if age <= LIFETIME_SECONDS - FADE_SECONDS:
            return 1.0
        left = LIFETIME_SECONDS - age
        if left <= 0.0:
            return 0.0
        return left / FADE_SECONDS


class Toasts:
    """The lines that are on the screen, and the painting of them.

    The deck holds no world and reads none. A caller hands it a sentence, and
    it decides whether the sentence is new, how long it stays and where it
    sits.
    """

    __slots__ = ("_said", "_shown")

    def __init__(self) -> None:
        """Build an empty deck."""
        self._shown: list[Toast] = []
        # When each sentence was last shown, so a counter that rises on many
        # ticks in a row writes one line rather than one for each tick.
        self._said: dict[str, float] = {}

    def __len__(self) -> int:
        """Give back how many lines the deck holds."""
        return len(self._shown)

    def texts(self) -> list[str]:
        """Give back the lines the deck holds, oldest first."""
        return [toast.text for toast in self._shown]

    def show(
        self,
        text: str,
        colour: int,
        now: float,
        key: str = "",
        rank: int = RANK_MOMENT,
    ) -> None:
        """Put one line on the screen, or refresh the copy already there.

        The key names the thing the line is about. Two lines with one key are
        the same thing said twice, and the second refreshes the first rather
        than stacking under it. An empty key takes the text itself, which is
        right when the sentence names the moment exactly.

        A full deck drops the oldest line to make room. It drops nothing while
        every line is younger than the minimum, and it then refuses the new
        line unless the new line outranks the oldest one.
        """
        named = key or text
        when = self._said.get(named)
        if when is not None and now - when < SAME_AGAIN_SECONDS:
            self._said[named] = now
            for toast in self._shown:
                if toast.key == named:
                    toast.born = now
                    toast.text = text
                    return
        if len(self._shown) >= MOST_SHOWN:
            oldest = self._shown[0]
            if now - oldest.born < MINIMUM_SECONDS and rank <= oldest.rank:
                return
            del self._shown[0]
        self._said[named] = now
        self._shown.append(Toast(text, colour, named, now, rank))

    def forget_old(self, now: float) -> None:
        """Drop the lines whose time is up, and the keys nobody needs.

        The keys are dropped as well as the lines. A key that stayed would
        hold a sentence out of a run that lasts hours.
        """
        self._shown = [toast for toast in self._shown if toast.weight(now) > 0.0]
        self._said = {
            key: when
            for key, when in self._said.items()
            if now - when < max(LIFETIME_SECONDS, SAME_AGAIN_SECONDS)
        }

    def paint(self, surface: Surface, now: float) -> int:
        """Paint the lines over the frame, and give back how many were drawn.

        The lines sit at the foot of the frame, in the middle, with the newest
        at the bottom. The corners of the window hold the cards of the deck,
        and the middle of the foot holds none of them.
        """
        self.forget_old(now)
        if not self._shown:
            return 0
        tall = text_height(SCALE) + PADDING * 2
        painted = 0
        for above, toast in enumerate(reversed(self._shown)):
            weight = toast.weight(now)
            if weight <= 0.0:
                continue
            wide = text_width(toast.text, SCALE) + PADDING * 2
            left = (surface.width - wide) // 2
            top = surface.height - FOOT_MARGIN - tall - above * (tall + LINE_GAP)
            paint_block(
                surface, left, top, wide, tall, BACKING_COLOUR, BACKING_WEIGHT * weight
            )
            paint_text(
                surface,
                left + PADDING,
                top + PADDING,
                toast.text,
                toast.colour,
                SCALE,
                weight,
            )
            painted += 1
        return painted


class Announcer:
    """Turns what the engine published into sentences for the deck.

    **This reads logs and counters. It walks no entity.** The engine writes
    one row for each thing that happened on the last step, so the step methods
    run after each step and the frame methods run once for each drawn frame.
    """

    __slots__ = ("_census", "_charactered", "_ended", "_population", "toasts")

    def __init__(self, toasts: Toasts | None = None) -> None:
        """Build an announcer with an empty deck and no history."""
        self.toasts = toasts if toasts is not None else Toasts()
        # The counters as they were at the last reading. A rise between two
        # readings is the event.
        self._census: dict[str, int] = {}
        # The population of each faction at the last reading, so a line lands
        # when a faction passes a round number.
        self._population: list[int] = []
        # The factions that already have a character, so only the first one
        # of each faction gets a line.
        self._charactered: set[int] = set()
        # Whether the end of the game was said. The record is written once.
        self._ended = False

    def after_step(self, world: World, now: float) -> None:
        """Read the logs of the step that just ran.

        A log covers the last step alone, so the next step empties it. A frame
        that ran several ticks therefore has to call this after each of them.
        """
        self._relations(world, now)
        self._campaigns(world, now)
        self._first_characters(world, now)
        self._collapses(world, now)
        self._foundings(world, now)
        self._wonders(world, now)

    def after_frame(self, world: World, now: float) -> None:
        """Read what the counters and the records say after a drawn frame.

        The counters are cumulative, so a rise since the last reading names a
        thing that happened. This runs once for each drawn frame rather than
        once for each tick, because a counter carries no tick and a rise over
        several ticks is still one rise.
        """
        self._counters(world, now)
        self._population_marks(world, now)
        self._end(world, now)

    def _relations(self, world: World, now: float) -> None:
        """Say who declared war on whom, and who made peace.

        The engine writes one row for each ordered pair whose relation crossed
        the war edge. A band after below the band before is a declaration, and
        a band above is a peace.
        """
        columns = world.log("relation_crossed")
        for row in range(len(columns["tick"])):
            speaker = int(columns["from_faction"][row])
            other = int(columns["to_faction"][row])
            declared = int(columns["band_after"][row]) < int(
                columns["band_before"][row]
            )
            verb = "declares war on" if declared else "makes peace with"
            self.toasts.show(
                f"Faction {speaker} {verb} faction {other}",
                faction_colour(speaker),
                now,
            )

    def _campaigns(self, world: World, now: float) -> None:
        """Say who marched, who took a place and who was thrown back.

        The engine writes one row when a campaign is raised and one when it
        closes. A close by a holder change to a third party is bookkeeping, so
        it gets no line.
        """
        columns = world.log("campaign_event")
        for row in range(len(columns["tick"])):
            faction = int(columns["faction"][row])
            kind = int(columns["kind"][row])
            q = int(columns["objective_q"][row])
            r = int(columns["objective_r"][row])
            place = f"({q}, {r})"
            if kind == 0:
                cohort = int(columns["cohort_size"][row])
                text = f"Faction {faction} marches on {place} with {cohort} soldiers"
            elif kind == 1:
                text = f"Faction {faction} takes {place}"
            elif kind == 2:
                text = f"Faction {faction} is thrown back from {place}"
            else:
                continue
            self.toasts.show(text, faction_colour(faction), now)

    def _first_characters(self, world: World, now: float) -> None:
        """Say when a faction gains the first character of its run.

        **Only the first one gets a line.** A mature faction promotes a
        soldier every few ticks, so a line for each promotion would be half
        of everything on the screen. The first is a milestone, and the panel
        and the console report the rest.
        """
        columns = world.log("unit_promoted")
        for row in range(len(columns["faction"])):
            faction = int(columns["faction"][row])
            if faction in self._charactered:
                continue
            self._charactered.add(faction)
            self.toasts.show(
                f"Faction {faction} gains its first character",
                faction_colour(faction),
                now,
                key=f"first character {faction}",
                rank=RANK_MILESTONE,
            )

    def _collapses(self, world: World, now: float) -> None:
        """Say when an upgrade wore away to nothing.

        An upgrade collapses when the weather and a hostile army take the last
        of its condition. The entry is then gone and the tile returns to the
        ground the generator made, so the log is the only record of it.

        A collapse is a moment and not a rate. A world with no storm and no
        war has none at all.
        """
        columns = world.log("upgrade_collapsed")
        for row in range(len(columns["tick"])):
            holder = int(columns["holder"][row])
            category = int(columns["category"][row])
            cause = int(columns["cause"][row])
            what = CATEGORY_WORDS.get(category, "upgrade")
            blame = CAUSE_WORDS.get(cause, "wear")
            if holder == NOBODY:
                text = f"A {what} falls to {blame}"
                colour = NEUTRAL_COLOUR
            else:
                text = f"Faction {holder} loses a {what} to {blame}"
                colour = faction_colour(holder)
            self.toasts.show(text, colour, now)

    def _foundings(self, world: World, now: float) -> None:
        """Say who founded a settlement, and where.

        The engine once published a count of the settlements and nothing else.
        The count falls when a settlement is lost, so a founding and a loss in
        one tick cancelled and the watcher saw neither.
        """
        columns = world.log("settlement_founded")
        for row in range(len(columns["tick"])):
            faction = int(columns["faction"][row])
            self.toasts.show(
                f"Faction {faction} founds a settlement",
                faction_colour(faction),
                now,
                rank=RANK_MILESTONE,
            )

    def _wonders(self, world: World, now: float) -> None:
        """Say when a wonder finishes.

        **Only a wonder gets a line.** A road, a terrace, a store, a wall and
        a lodging finish every few ticks in a mature world, which is a rate
        and not a moment. A wonder claims the wealth-or-wonder end, so it is
        the one level that changes the run.
        """
        columns = world.log("upgrade_finished")
        for row in range(len(columns["tick"])):
            if int(columns["category"][row]) != CATEGORY_WONDER:
                continue
            holder = int(columns["holder"][row])
            level = int(columns["level"][row])
            whose = "A" if holder == NOBODY else f"Faction {holder}"
            verb = "stands" if holder == NOBODY else "finishes"
            colour = NEUTRAL_COLOUR if holder == NOBODY else faction_colour(holder)
            self.toasts.show(
                f"{whose} {verb} a wonder at level {level}",
                colour,
                now,
                rank=RANK_MILESTONE,
            )

    def _counters(self, world: World, now: float) -> None:
        """Say when one of the watched counters rose since the last reading.

        **A line from a counter carries no faction colour.** A counter says
        that a thing happened and it does not say who did it. A counter is
        therefore the last source to reach for, and a subsystem that publishes
        a log is read from the log.

        Only a rare counter belongs here. A counter that rises on most ticks
        would write a line on most ticks, and the deck would then hold nothing
        else.
        """
        if not WATCHED_COUNTERS:
            # The census reads every row of every subsystem. A deck that
            # watches no counter must not pay for that on each frame.
            return
        census = world.subsystem_census()
        was = self._census
        self._census = dict(census)
        if not was:
            # The first reading has nothing to compare against. A world that
            # was founded with four settlements would otherwise open with a
            # line about four settlements being founded.
            return
        for name, one, many in WATCHED_COUNTERS:
            rise = int(census.get(name, 0)) - int(was.get(name, 0))
            if rise <= 0:
                continue
            text = one if rise == 1 else f"{rise} {many}"
            self.toasts.show(text, NEUTRAL_COLOUR, now, key=name, rank=RANK_MILESTONE)

    def _population_marks(self, world: World, now: float) -> None:
        """Say when a faction passes a round number of people.

        A line for each birth would be a stream. The mark is the round number,
        and a faction that crosses one gets one line.
        """
        population = list(world.faction_population())
        was = self._population
        self._population = population
        if len(was) != len(population):
            return
        for faction, count in enumerate(population):
            before = was[faction] // POPULATION_MARK
            after = count // POPULATION_MARK
            if after <= before or after == 0:
                continue
            self.toasts.show(
                f"Faction {faction} passes {after * POPULATION_MARK} people",
                faction_colour(faction),
                now,
                key=f"population {faction}",
                rank=RANK_MILESTONE,
            )

    def _end(self, world: World, now: float) -> None:
        """Say who won, once, when the end record first appears.

        The record is engine state and the world keeps stepping after it. The
        line is shown the first time the record is set.
        """
        end = world.game_end()
        if end is None or self._ended:
            return
        self._ended = True
        winner = int(end["winner"])
        # The engine names a path with underscores. A watcher reads words.
        path = end["path"].replace("_", " ")
        self.toasts.show(
            f"Faction {winner} wins by {path}",
            faction_colour(winner),
            now,
        )
