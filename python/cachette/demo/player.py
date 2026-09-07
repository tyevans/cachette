"""One faction in the hands of a person, and the turn that holds the clock.

**A person and a policy choose through one surface.** The engine publishes an
action table, a mask that says which row of it is legal now, and one verb that
takes a row. The learner reads that mask and calls that verb. A person at the
keyboard reads the same mask and calls the same verb, and this module is the
whole of the difference between them: it stops the clock while the person
reads.

The turn is a number of ticks. The world runs that many ticks, then freezes,
and it stays frozen until the person ends the turn. **Time does not run while
the person is choosing.** A frozen world owes no tick, and the ticks a frozen
frame would have run are dropped rather than saved, so a person who thinks for
a minute does not get a minute of world when they finish.

Nothing here loops over an entity. The mask comes from the engine as one
array, the schema comes from the engine as one dictionary, and an action
crosses back as one whole number.

References
----------
ADR-0176, the action table is flat and bounded.
``docs/adrs/draft/adr-0176-the-action-table-is-flat-and-bounded.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING, NamedTuple

if TYPE_CHECKING:
    import numpy as np

    from cachette import World

# How many ticks one turn runs, when nobody says.
#
# **Twenty ticks is short enough to feel like a move and long enough to see
# one.** A gather order or a build order takes several ticks to show anything
# at all, so a turn of one tick would report nothing back to the person who
# gave it. A turn of two hundred would run a small war between two choices.
TURN_TICKS = 20

# The shortest and the longest turn a person may ask for.
#
# A turn of no ticks never advances, and the world would stand still whatever
# the person did. The upper bound is a guard on the menu and not a law of the
# engine.
LEAST_TURN = 1
MOST_TURN = 200

# The turn lengths the menu offers, in ticks.
TURN_CHOICES = (1, 5, 10, 20, 40, 80, 200)


class Choice(NamedTuple):
    """One row of the action table, as a person reads it.

    The action is the whole number the engine takes. The verb is the name the
    engine gave that verb. The argument names the position the row varies, and
    it is empty for a verb the engine resolves by itself.
    """

    action: int
    verb: str
    argument: str
    legal: bool

    def says(self) -> str:
        """Give back the line a card shows for this row."""
        words = self.verb.replace("_", " ").upper()
        if not self.argument:
            return words
        return f"{words} {self.argument}"


def read_table(world: World) -> list[Choice]:
    """Give back one entry for each row of the action table, in its order.

    **The schema is the only declaration of the table.** No number here says
    where a verb starts, how many rows it holds, or what its argument means. A
    verb the engine adds appears on the card with no edit to this file.

    The engine publishes no words for the argument of a verb, so a row names
    the position and the number the engine gave it. A person reads ``GATHER
    RESOURCE 1`` rather than the name of that resource. A name table on the
    engine side is the one change that would improve this line, and inventing
    the words here would put a second declaration of them in the control
    plane.

    Every row is marked legal. A caller that holds a mask calls ``with_mask``
    to mark them against it.
    """
    schema = world.action_schema()
    rows: list[Choice] = []
    for verb in schema["verbs"]:
        name = str(verb["name"])
        first = int(verb["first"])
        positions = list(verb["positions"])
        if not positions:
            rows.append(Choice(first, name, "", True))
            continue
        for offset in range(int(verb["rows"])):
            words = " ".join(
                f"{str(position['candidate']).replace('_', ' ').upper()} "
                f"{(offset // int(position['stride'])) % int(position['bound'])}"
                for position in positions
            )
            rows.append(Choice(first + offset, name, words, True))
    rows.sort(key=lambda choice: choice.action)
    return rows


def with_mask(rows: list[Choice], mask: np.ndarray) -> list[Choice]:
    """Mark each row against the legality mask the engine gave."""
    return [
        choice._replace(
            legal=bool(mask[choice.action]) if choice.action < len(mask) else False
        )
        for choice in rows
    ]


class Seat:
    """The faction a person holds, and the turn they are inside.

    The seat is the one thing that tells a run in which a person plays from a
    run in which the engine plays every faction. Nothing else in the loop
    changes shape.
    """

    __slots__ = (
        "_marked",
        "_read_at",
        "_table",
        "_world",
        "choosing",
        "faction",
        "left",
        "taken",
        "turn",
        "turn_ticks",
    )

    def __init__(
        self, world: World, faction: int, turn_ticks: int = TURN_TICKS
    ) -> None:
        """Take a faction out of the hands of the engine controller.

        The engine gives that faction no evaluation while the flag is set, so
        the faction does nothing that the person did not ask for.
        """
        self._world = world
        self.faction = faction
        self.turn_ticks = _held(turn_ticks)
        world.set_externally_controlled(faction, True)
        # The rows of the action table never change inside one run, so they
        # are read once. The mask changes every tick, and it is read fresh.
        self._table = read_table(world)
        # **A run opens with the world frozen.** The person takes the seat and
        # then reads it. A world that ran a turn before they had chosen
        # anything would spend their first turn for them.
        self.choosing = True
        self.left = self.turn_ticks
        self.turn = 1
        # The actions this turn already took, as the words a card shows.
        self.taken: list[str] = []
        # The rows as they were last marked against the mask, and what the
        # world looked like when they were marked. **A frame draws the card,
        # lays it out and answers a click**, and each of those asks which rows
        # are legal. One crossing for each of them would ask the engine four
        # times for one answer that cannot change inside a frame.
        self._marked: list[Choice] = []
        self._read_at: tuple[int, int] = (-1, -1)

    @property
    def frozen(self) -> bool:
        """Say whether the clock is held while the person chooses."""
        return self.choosing

    def choose_turn_ticks(self, ticks: int) -> None:
        """Set how many ticks one turn runs.

        The change reaches the turn the person is inside, so a shorter turn
        never leaves more ticks owed than a whole turn holds.
        """
        self.turn_ticks = _held(ticks)
        self.left = min(self.left, self.turn_ticks)

    def actions(self) -> list[Choice]:
        """Give back every row of the action table, marked against the mask.

        The mask is fog scoped. A row that names ground the faction has never
        seen is refused here in the same way it is refused for a policy.

        The answer is held until the world moves or the person acts, because
        nothing else can change it.
        """
        now = (int(self._world.tick), len(self.taken))
        if now != self._read_at:
            self._marked = with_mask(
                self._table, self._world.legal_actions(self.faction)
            )
            self._read_at = now
        return self._marked

    def take(self, action: int) -> bool:
        """Send one action to the engine, and say whether the verb took it.

        The action crosses as one whole number through the verb the learner
        uses. A refusal is an answer and not a failure: the engine records it,
        and the card says so.
        """
        took = bool(self._world.act(self.faction, action))
        for choice in self._table:
            if choice.action == action:
                mark = "" if took else " (REFUSED)"
                self.taken.append(choice.says() + mark)
                break
        return took

    def end_turn(self) -> None:
        """Let the clock run again for the length of one turn."""
        self.choosing = False
        self.left = self.turn_ticks
        self.taken = []

    def hold(self, due: int) -> int:
        """Give back how many of the ticks a frame owes the turn allows.

        **A frozen turn allows none.** The ticks the frame owed are dropped.
        A turn that saved them would run the whole of a person's thinking time
        the moment they pressed the key.
        """
        if self.choosing:
            return 0
        return max(0, min(due, self.left))

    def tick_ran(self) -> None:
        """Take note that the world ran one tick, and freeze at the end.

        The caller calls this once for each tick it ran, so a frame that ran
        several ticks reaches the end of the turn on the right one.
        """
        if self.choosing:
            return
        self.left -= 1
        if self.left <= 0:
            self.left = 0
            self.choosing = True
            self.turn += 1

    def release(self) -> None:
        """Give the faction back to the engine controller."""
        self._world.set_externally_controlled(self.faction, False)


def _held(ticks: int) -> int:
    """Give back a turn length inside the bounds."""
    return max(LEAST_TURN, min(MOST_TURN, ticks))
