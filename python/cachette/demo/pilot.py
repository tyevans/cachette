"""One faction in the hands of a stored policy, at the cadence it learned.

A score says that a policy is better than the built-in controller. It does not
say what the policy does. **This module puts a stored policy on a faction of
the demonstration world**, so a watcher sees the play rather than the number.

# The policy acts through the seat a person acts through

The engine publishes an action table, a mask that says which row is legal now,
and one verb that takes a row. A person at the keyboard reaches all three
through the seat.[^1] A policy reaches them through the same seat, and the one
call that sends an action to the engine is the one the person uses. A second
copy of the acting rule would be a second declaration of it, and nothing would
fail when the two disagreed.[^2]

# The file states the cadence, and the pilot keeps it

A learner acts once, then lets the world run a fixed number of ticks, then
acts again.[^3] That number is the decision interval, and it is a property of
the run that trained the weights. A policy asked to act at another cadence
plays a game it never learned, and nothing raises.

**The pilot therefore reads the interval from the checkpoint and counts its
own ticks.** It holds no constant of its own, and it refuses a checkpoint that
names no cadence rather than guessing one. The turn of a person is a separate
mechanism over the same clock, so a policy and a person may hold two factions
of one world, and each keeps its own cadence.

Two things change the cadence, and both are honest. A person who is choosing
freezes the clock, so no tick runs and no pilot decides. The speed of the
clock changes how many ticks a second of wall time carries, and a pilot counts
ticks and never seconds.

# A file that does not fit this world stops the run

The observation length counts the cells of a lattice, so two worlds of
different extents can hold the same length and mean something else at every
position.[^4] The reader therefore checks the extent and the faction count
beside the two lengths and the two schema versions, and it names what
disagreed. **Nothing falls back to the built-in controller**, because a
faction that silently reverted would be watched as if it were the policy.

# References

[^1]: The player seat of the demonstration. ``python/cachette/demo/player.py``
[^2]: Recurring defect shapes, shape 1.
``.agents/rules/recurring-defects.md``
[^3]: The learner environment, the decision interval.
``python/cachette/learn/env.py``
[^4]: The stored policy and what it was trained against.
``python/cachette/learn/policy.py``
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING

from cachette.demo.player import Seat
from cachette.learn.policy import PolicyFit, PolicyFitError, load_policy

if TYPE_CHECKING:
    from cachette import World


class PolicyChoiceError(ValueError):
    """A caller named a policy seat that this world cannot give.

    A faction outside the world, a faction two policies asked for, and a
    weight file that is not there all reach here. Each is an answer to the
    watcher and not a defect in the engine, so the run stops with one
    sentence rather than a traceback.
    """


class Pilot:
    """One faction, one stored policy, and the tick count between decisions.

    The pilot holds a seat, and the seat holds the faction. Nothing here calls
    the acting verb of the engine: the seat does that, and the person at the
    keyboard uses the same call.
    """

    __slots__ = (
        "_choose",
        "_interval",
        "_since",
        "_world",
        "decisions",
        "name",
        "seat",
    )

    def __init__(
        self,
        world: World,
        faction: int,
        policy: object,
        name: str,
        interval: int,
    ) -> None:
        """Take a faction out of the hands of the engine controller.

        The seat sets the external control flag, so the engine gives the
        faction no evaluation of its own while the policy holds it.

        **The pilot decides once as it takes the seat.** The training loop
        applies an action and then runs the interval, so a pilot that waited
        an interval before its first decision would begin one decision behind
        the loop it learned in.
        """
        self._world = world
        self.seat = Seat(world, faction)
        self._choose = policy
        self.name = name
        self._interval = max(1, int(interval))
        self._since = 0
        self.decisions = 0
        self.decide()

    @property
    def faction(self) -> int:
        """The faction this policy holds."""
        return self.seat.faction

    @property
    def interval(self) -> int:
        """How many ticks the world runs between two decisions."""
        return self._interval

    @property
    def taken(self) -> list[str]:
        """The words of the actions the last decision sent."""
        return self.seat.taken

    def decide(self) -> None:
        """Read the world as this faction sees it, choose one row, and act.

        The observation and the mask are the two the learner reads, and both
        are fog scoped. The choice is a pure function of those two arrays, so
        one world gives one action however many threads ran it.
        """
        observation = self._world.faction_observation(self.faction)
        mask = self._world.legal_actions(self.faction)
        action = int(self._choose.choose(observation, mask))  # type: ignore[attr-defined]
        # The seat keeps the last decision alone, so a card shows what the
        # policy just did rather than everything it ever did.
        self.seat.taken = []
        self.seat.take(action)
        self.decisions += 1
        self._since = 0

    def tick_ran(self) -> None:
        """Take note that the world ran one tick, and decide at the interval.

        The caller calls this once for each tick it ran, so a frame that ran
        several ticks gives the pilot every decision those ticks earned.
        """
        self._since += 1
        if self._since >= self._interval:
            self.decide()

    def release(self) -> None:
        """Give the faction back to the engine controller."""
        self.seat.release()


def read_manifest(path: Path) -> dict[str, object]:
    """Give back what the manifest beside a weight file says, or nothing.

    A training run writes the weights and a manifest of the same stem. The
    manifest states what the run scored and what it was fitted at. A file that
    is absent or damaged gives an empty answer, and the caller then reads the
    weight file alone.
    """
    beside = path.with_suffix(".json")
    try:
        text = beside.read_text(encoding="utf-8")
    except OSError:
        return {}
    try:
        held = json.loads(text)
    except ValueError:
        return {}
    if not isinstance(held, dict):
        return {}
    return held


def _whole(value: object) -> int | None:
    """Give back a whole number, or nothing for anything else.

    A boolean is a whole number in Python, and it is not one here.
    """
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return None
    return int(value)


def read_interval(path: Path, meta: dict[str, object]) -> int:
    """Give back the cadence a checkpoint names, and refuse when it names none.

    **A checkpoint states its cadence in two places, and both are read.** The
    weight file carries it beside the fit, and the manifest of the same stem
    carries it too. A run that wrote only one of the two is still placed, and
    a run that wrote both and disagrees with itself is refused. A comment that
    named the winner would leave the loser reachable and silent.[^1]

    **A checkpoint that names no cadence is refused.** A guessed interval
    plays the policy at a rhythm it never learned, and the play would look
    wrong for a reason nothing on the screen could show.

    References
    ----------
    [^1]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """
    stored = _whole(meta.get("decision_interval"))
    written = _whole(read_manifest(path).get("decision_interval"))
    if stored is not None and written is not None and stored != written:
        message = (
            f"the checkpoint at {path} states two cadences. The weight file "
            f"says {stored} ticks and the manifest beside it says {written}. "
            "Repair the checkpoint, because nothing here can choose between "
            "them."
        )
        raise PolicyChoiceError(message)
    named = stored if stored is not None else written
    if named is None or named < 1:
        message = (
            f"the checkpoint at {path} names no decision interval, so nothing "
            "knows how often it was trained to act. A policy played at a "
            "guessed cadence plays a game it never learned. Write "
            "decision_interval into the weight file or into the manifest "
            "beside it."
        )
        raise PolicyChoiceError(message)
    return named


def read_seat(path: Path, meta: dict[str, object], world: World) -> int:
    """Give back the faction a checkpoint was trained in.

    A caller that names no faction gets this one. The weight file is read
    first, then the manifest beside it. A checkpoint that names none, or that
    names one outside this world, gives faction zero.
    """
    named = _whole(meta.get("seat"))
    if named is None:
        named = _whole(read_manifest(path).get("seat"))
    if named is None or named < 0 or named >= world.faction_count:
        return 0
    return named


def open_pilot(world: World, path: Path, faction: int | None = None) -> Pilot:
    """Seat a stored policy on one faction of this world.

    The faction is the one the caller named, or the one the file was trained
    in when the caller named none.

    Raises ``PolicyChoiceError`` when the file is not there or the faction is
    outside the world. Raises ``PolicyFitError`` when the file was trained
    against another world, and the message names every entry that disagreed.
    """
    if not path.is_file():
        message = (
            f"there is no weight file at {path}. Name a file that a training "
            "run wrote, or train a policy against this world."
        )
        raise PolicyChoiceError(message)
    policy, meta = load_policy(path, PolicyFit.of_world(world))
    seat = read_seat(path, meta, world) if faction is None else faction
    if seat < 0 or seat >= world.faction_count:
        message = (
            f"the world holds {world.faction_count} factions, so there is no "
            f"faction {seat} for the policy at {path}"
        )
        raise PolicyChoiceError(message)
    return Pilot(world, seat, policy, path.stem, read_interval(path, meta))


def chosen_policies(named: list[str]) -> list[tuple[int | None, Path]]:
    """Give back the faction and the file of each policy a watcher named.

    One entry is a path, and the policy then takes the faction its file names.
    An entry of the form ``2=path`` puts the policy on faction two instead, so
    a watcher seats several policies in one world and compares them.

    Raises ``PolicyChoiceError`` for a weight file that is not there, and for
    two policies on one faction.
    """
    chosen: list[tuple[int | None, Path]] = []
    for entry in named:
        faction: int | None = None
        text = entry
        head, sign, tail = entry.partition("=")
        if sign and head.strip().lstrip("-").isdigit():
            faction = int(head.strip())
            text = tail
        if not text.strip():
            message = f"the policy {entry!r} names no weight file"
            raise PolicyChoiceError(message)
        path = Path(text.strip())
        # **A path is checked here, before anything builds a world.** A
        # watcher who mistyped a name waits for one sentence rather than for a
        # world nobody will watch.
        if not path.is_file():
            message = (
                f"there is no weight file at {path}. Name a file that a "
                "training run wrote, or train a policy against this world."
            )
            raise PolicyChoiceError(message)
        chosen.append((faction, path))
    named_factions = [seat for seat, _ in chosen if seat is not None]
    if len(set(named_factions)) != len(named_factions):
        message = (
            "two policies ask for one faction. A faction holds one policy, "
            "so name a different faction for each file."
        )
        raise PolicyChoiceError(message)
    return chosen


def seat_policies(world: World, chosen: list[tuple[int | None, Path]]) -> list[Pilot]:
    """Seat every policy a watcher named, and refuse a faction taken twice.

    Every file is read and checked against this world before any faction
    leaves the hands of the engine controller, so a run that refuses one file
    leaves the world as it found it.
    """
    pilots: list[Pilot] = []
    try:
        for faction, path in chosen:
            pilot = open_pilot(world, path, faction)
            if any(other.faction == pilot.faction for other in pilots):
                pilot.release()
                message = (
                    f"faction {pilot.faction} already holds a policy. Name a "
                    f"faction for {path} with a leading number, as in "
                    f"1={path}."
                )
                raise PolicyChoiceError(message)
            pilots.append(pilot)
    except (PolicyChoiceError, PolicyFitError):
        for pilot in pilots:
            pilot.release()
        raise
    return pilots
