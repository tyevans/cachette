"""Why each episode of a generation ended, and how near the rest came.

A run reports the share of episodes a seat won. That number says nothing
about a generation that won none. A policy that reaches almost every wonder
and a policy that reaches none both report a win share of zero, and the two
call for opposite decisions: run for longer, or change the reward.

A win is a threshold event and the reward is continuous, so a win share is
expected to stand still and then jump. This module states the approach to
that jump before it happens.

# The engine already publishes the approach

Every quantity here comes from the engine. The end record of a world names
the path a game ended on, and the observation of a faction carries one share
for each win path.[^1] **This module adds no reader to the engine.** It reads
what a faction already sees, so the figures it reports are the figures the
policy trained on.

# A distribution, and never a mean

A mean over the episodes hides the case this instrument exists for. A few
games that come close and many that do not is exactly the state that predicts
a jump, and a mean reads it as the same number a uniform middle would give.
This reports the median, the ninth decile and the highest value of each path.

# The seat and the leader

Each path reports two readings. The own reading is the share the seat itself
reached. The leader reading is the share of the leading faction, which is the
seat when the seat leads and a rival when it does not. A reader that wants to
know whether the opponents are near a wonder reads the leader.

# What one means on each path

**Two of the four paths end at one, and two do not.** Read the number of the
path you are looking at.

The wonder share and the renown share are fractions of a threshold. One is
the value at which the reader fires.

The domination share is the seats the faction holds over the seats the reader
asks for, and one fires the seat clause. It has a floor: a faction that holds
only its own seat reads one over the faction count. **The other clause of
that reader stays out of the share.** The engine ends a game for a faction
that holds a unit while every rival holds none, and a share of that clause
would state the unit count of a rival the faction never observed.[^2] An
episode can therefore end on domination while the share sits at its floor.

The territory share is the held tiles over the passable tiles of the world.
The territory reader compares the factions at the tick limit and holds no
threshold, so this share never reaches one in a game of several factions.
Read it against the leader reading rather than against one.

# References

[^1]: ADR-0148, a game end is recorded once and stops the controllers,
decision D1.
``docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md``

[^2]: PRD-0001, a faction sees only what it observes.
``docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Final

import numpy as np

from cachette._core import win_paths

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from .record import EpisodeRecord
    from .signals import SignalCatalogue

# The name a report gives an episode that holds no end record.
#
# The engine writes an end record for every game a reader ended, and the
# territory reader ends every game that reaches the tick limit. An episode
# therefore reads this name only when the horizon of the learner stopped it
# before the world reached its limit, or when the readers were off.
UNFINISHED: Final = "unfinished"

# The observation signal that carries the progress of each win path, under
# the name the engine gives the path.
#
# **The engine names a path and a signal separately, so this mapping is a
# join and not a second declaration of either.** One test asserts that the
# keys are exactly the paths the engine publishes and that every value names
# a signal of the schema, so a path or a signal that moves fails a check
# rather than reading as a missing value.[^1]
#
# References
# ----------
# [^1]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
PATH_PROGRESS: Final[Mapping[str, str]] = {
    "domination": "domination_progress",
    "territory": "ground_progress",
    "wonder": "wonder_track_progress",
    "renown": "renown_progress",
}

# The observation signal that carries the same progress for the faction that
# leads the path. The leading faction is the seat itself while the seat
# leads, so a leader reading is never below the own reading.
PATH_LEADER: Final[Mapping[str, str]] = {
    "domination": "domination_leader",
    "territory": "ground_leader",
    "wonder": "wonder_track_leader",
    "renown": "renown_leader",
}


@dataclass(frozen=True)
class Reach:
    """How far a set of episodes came along one win path.

    The three entries are the median, the ninth decile and the highest value
    over the episodes. Each is the share the engine publishes for that path,
    scaled to run from zero to one.

    **What one means depends on the path.** The wonder share and the renown
    share reach one where the reader fires. The domination share reaches one
    where the seat clause fires, and it starts at one over the faction count.
    The territory share is the held ground over the passable world, and the
    reader of that path holds no threshold at all.

    A set of no episode reads zero everywhere.
    """

    median: float
    upper: float
    highest: float

    def as_dict(self) -> dict[str, float]:
        """Return this reach as plain values, for a report file."""
        return {
            "median": self.median,
            "upper": self.upper,
            "highest": self.highest,
        }


@dataclass(frozen=True)
class Endings:
    """Why a set of episodes ended, and how near the rest came.

    The shares entry holds one fraction for each win path the engine
    publishes and one for the episodes that hold no end record. The fractions
    sum to one over a set that holds at least one episode.

    The own entry holds the reach of the seat along each path, and the leader
    entry holds the reach of the faction that leads each path.
    """

    shares: Mapping[str, float]
    own: Mapping[str, Reach]
    leader: Mapping[str, Reach]

    def as_dict(self) -> dict[str, object]:
        """Return this summary as plain values, for a report file."""
        return {
            "shares": dict(self.shares),
            "own": {name: reach.as_dict() for name, reach in self.own.items()},
            "leader": {name: reach.as_dict() for name, reach in self.leader.items()},
        }

    def columns(self) -> dict[str, float]:
        """Return the flat columns a history row and a dashboard read.

        The row holds the share of each ending and the ninth decile of each
        path, under names a reader groups by prefix. **The median and the
        highest value stay out of the row**, because the row is one line of a
        table and the ninth decile is the entry that moves first when a
        policy starts to approach a threshold.
        """
        row = {f"ended_{name}": share for name, share in self.shares.items()}
        row.update({f"reach_{name}": reach.upper for name, reach in self.own.items()})
        row.update(
            {f"reach_leader_{name}": reach.upper for name, reach in self.leader.items()}
        )
        return row

    def ended_line(self) -> str:
        """Return the share of each ending, as one printed line.

        The order is the order the engine numbers the paths, and the episodes
        that hold no end record come last.
        """
        return " ".join(f"{name} {share:4.2f}" for name, share in self.shares.items())

    def reach_line(self) -> str:
        """Return the reach of the seat along each path, as one printed line.

        Each path prints its median, its ninth decile and its highest value,
        in that order. A reader watching a run sees a policy approach a
        threshold in the second and third figures long before the first
        moves.
        """
        return " ".join(
            f"{name} {reach.median:4.2f}/{reach.upper:4.2f}/{reach.highest:4.2f}"
            for name, reach in self.own.items()
        )


def end_shares(
    episodes: Sequence[EpisodeRecord], paths: Sequence[str]
) -> dict[str, float]:
    """Return the share of the episodes that ended on each path.

    The result holds one entry for every path given and one for the episodes
    that hold no end record, whether or not any episode reached it. **A path
    that no episode reached reads zero rather than being absent**, because a
    reader that compared two generations would otherwise read a missing key
    as a path that stopped existing.

    An ending the engine reports and the caller did not name is added at the
    end, so a report states what it saw rather than dropping it.
    """
    counts = dict.fromkeys((*paths, UNFINISHED), 0)
    for row in episodes:
        counts[row.end_path] = counts.get(row.end_path, 0) + 1
    if not episodes:
        return {name: 0.0 for name in counts}
    return {name: count / len(episodes) for name, count in counts.items()}


def reach_of(
    episodes: Sequence[EpisodeRecord], signal: str, catalogue: SignalCatalogue
) -> Reach:
    """Return the reach of a set of episodes along one published share.

    The share crosses as a fixed-point integer, and the engine publishes the
    value that stands for one. **This divides by that published value and
    holds no scale of its own**, because a scale written here would be a
    second copy of an engine rule with nothing to fail when the two
    disagree.[^1]

    An episode that carries no reading of the signal is out of the summary. A
    set with no reading at all reads zero everywhere.

    References
    ----------
    [^1]: Findings register, FND-701. ``docs/FINDINGS.md``
    """
    unit = _share_unit(signal, catalogue)
    values = [row.signals[signal] / unit for row in episodes if signal in row.signals]
    if not values:
        return Reach(median=0.0, upper=0.0, highest=0.0)
    held = np.asarray(values, dtype=float)
    return Reach(
        median=float(np.median(held)),
        upper=float(np.quantile(held, 0.9)),
        highest=float(held.max()),
    )


def _share_unit(signal: str, catalogue: SignalCatalogue) -> float:
    """Return the value that stands for one in a published share.

    The catalogue carries the form of every signal, and the form states the
    unit. A signal whose schema states no form is refused, because a default
    here would silently rescale every figure the instrument reports.
    """
    form = catalogue.signal(signal).form
    if form is None or not form.unit:
        message = (
            f"{signal!r} publishes no value form, so nothing states the value "
            "that stands for one. The instrument cannot scale it."
        )
        raise ValueError(message)
    return float(form.unit)


def summarise_endings(
    episodes: Sequence[EpisodeRecord], catalogue: SignalCatalogue
) -> Endings:
    """Return why a set of episodes ended and how near the rest came.

    **The set of win paths comes from the engine.** A list written here would
    drop a path the engine later added, and nothing would fail.

    The cost is one pass over the episodes for each path. It reads no world
    and steps nothing, because every figure was recorded when the episode
    ended.
    """
    paths = win_paths()
    return Endings(
        shares=end_shares(episodes, paths),
        own={
            name: reach_of(episodes, signal, catalogue)
            for name, signal in PATH_PROGRESS.items()
            if name in paths
        },
        leader={
            name: reach_of(episodes, signal, catalogue)
            for name, signal in PATH_LEADER.items()
            if name in paths
        },
    )


__all__ = [
    "PATH_LEADER",
    "PATH_PROGRESS",
    "UNFINISHED",
    "Endings",
    "Reach",
    "end_shares",
    "reach_of",
    "summarise_endings",
]
