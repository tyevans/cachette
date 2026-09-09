"""What one training run produces, kept so a later reader can measure it.

A run that keeps only a mean throws away the measurements that answer the next
question. This module holds three records. An episode record holds what one
candidate did on one seed. A population record holds every episode of one
batch. A generation record holds what a generation scored and what the search
made of it.

It also holds two readings that a record carries rather than earns. A tally
says what a policy answered at each decision. A score says what one pass over
a fixed seed set measured, and which of its two figures selects.

Each field below answers a gap in the earlier reporting.

# A paired comparison needs the outcome of each seed

Two policies compared over a shared seed set can be compared seed by seed,
and a paired comparison holds far more power than an unpaired one. A run that
stored only the mean over the seeds threw that away, so every later comparison
had to be unpaired. An episode record therefore names its own seed.

# A reward experiment needs the score vector

The score of each candidate is what the search ranks. A run that logged only
the highest, the mean and the lowest of them cannot be read again for any
other question. A generation record therefore holds the whole vector.

# A refusal is a measurement, and nothing measured it

The engine says whether a verb accepted an action, and a batch used to throw
that answer away. A policy whose actions the engine mostly refuses is close to
a no-op whatever it chooses, and no figure of the run said so. An episode
record therefore counts what the policy chose and what the engine refused.

# A return does not say what the policy answered

The four policies of one paid run are each a fixed preference order over the
action rows, and the engine's legality answer supplies what looks like
situational play. Every figure the run reported was a return, and no return
can separate a preference from a policy. A tally therefore holds two
readings: the share of decisions on the most common action, and whether the
argmax over the unmasked rows ever moves.[^3]

# The quantity that measures play was computed and thrown away

A population record counts the episodes that ended in a win. The judge of the
run reduced the same record to the mean of the shaped return and never read
the win share, so the selection ran on a proxy. A score therefore carries
both figures under their own names, and a caller cannot use one where it
meant the other.[^4]

# The tick of the end is not a signal

How long a game ran is the strongest single answer a run reports about the
game itself. The engine publishes no observation field that carries it, so a
record read the signals under the name ``tick`` and took a default of zero.
Every episode the project recorded carried a zero in that column, and nothing
failed. An episode record therefore holds the end tick as a field of its own,
read from the world.[^2]

# References

[^1]: Findings register, FND-670. ``docs/FINDINGS.md``
[^2]: Findings register, FND-689. ``docs/FINDINGS.md``
[^3]: Findings register, FND-707. ``docs/FINDINGS.md``
[^4]: What is wrong with training and evaluation, item 1.
``docs/research/what-is-wrong-with-training-and-evaluation.md``
"""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass, field
from itertools import pairwise
from typing import TYPE_CHECKING, Protocol

import numpy as np

from .reward import OUTCOMES, RUNNING

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from cachette._core import GameEnd

    from .env import Env


# What a column of an objective is called in a report row. One prefix keeps
# an objective apart from a signal of the engine of the same name.
OBJECTIVE_PREFIX = "objective."


@dataclass(frozen=True)
class ActionTally:
    """The two instruments over the decisions of one episode.

    A finding named two cheap checks and the project had neither. The first is
    the share of decisions on which the policy emits its most common action.
    The second is whether the highest-scoring row over the **unmasked** action
    rows ever changes inside the episode.[^1]

    Both separate a preference order from a policy. A fixed preference order
    over the action rows still emits many different actions, because the
    engine's legality answer removes the rows it cannot take, so the emitted
    actions look situational. The unmasked argmax does not: it moves only when
    the observation moved the scores.

    **These are instruments and not gates.** A run reports them and no run
    fails on them.[^2]

    The decisions entry counts the decisions the tally saw. The highest count
    entry is how many of them fell on the most common action. The preferences
    entry counts the decisions whose unmasked preference was read, which is
    zero for a policy that publishes no score. The changes entry counts how
    often that preference differed from the one before it.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``

    [^2]: ADR-0202, a run selects on the win share, and
    every published figure names its seed set, decision D5.
    ``docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md``
    """

    decisions: int = 0
    most_common: int = -1
    highest_count: int = 0
    preferences: int = 0
    changes: int = 0

    @classmethod
    def of_actions(
        cls, actions: Sequence[int], preferences: Sequence[int]
    ) -> ActionTally:
        """Count one episode's emitted actions and its unmasked preferences.

        The preferences may be shorter than the actions, or empty. A policy
        that publishes no score reports none, and the tally then says that the
        preference was never read rather than that it never changed.
        """
        counts = Counter(int(value) for value in actions)
        common, highest = counts.most_common(1)[0] if counts else (-1, 0)
        read = [int(value) for value in preferences]
        changes = sum(1 for before, after in pairwise(read) if before != after)
        return cls(
            decisions=len(actions),
            most_common=int(common),
            highest_count=int(highest),
            preferences=len(read),
            changes=changes,
        )

    @property
    def most_common_share(self) -> float:
        """The share of the decisions that fell on the most common action.

        An episode that took no decision reads zero, because it emitted no
        action to be common.
        """
        if self.decisions == 0:
            return 0.0
        return self.highest_count / self.decisions

    @property
    def preference_varies(self) -> bool | None:
        """Whether the unmasked argmax changed inside the episode.

        Nothing when no preference was read. A policy that publishes no score
        has no answer here, and a false would state that its preference held.
        """
        if self.preferences == 0:
            return None
        return self.changes > 0


@dataclass(frozen=True)
class ValidationScore:
    """What one pass over a fixed seed set measured, and which figure selects.

    **The win share selects, and the mean shaped return breaks a tie.** The
    shaped return is the training signal, because it is dense and it drives
    the search. It is not a measure of play. A run that selected its centre on
    the mean shaped return published four policies that take one unit and
    wander, and the win share of every one of those passes was already
    computed and thrown away.[^1]

    A win is always defined, so the win share is a real quantity over any
    pass. The engine's territory reader compares held ground at the tick
    limit and names a winner there, so an episode that runs out of ticks ends
    won or lost rather than drawn.[^2]

    **This type exists so that a caller cannot use one figure where it meant
    the other.** The pass used to give back a bare float, and the one caller
    that selected on it read the shaped mean without saying so.[^3]

    The two instrument entries carry what the pass measured about the
    behaviour of the policy rather than about its result.

    References
    ----------
    [^1]: What is wrong with training and evaluation, item 1.
    ``docs/research/what-is-wrong-with-training-and-evaluation.md``

    [^2]: The game end readers of the engine.
    ``crates/cachette-core/src/world/victory.rs``

    [^3]: ADR-0202, a run selects on the win share, and
    every published figure names its seed set, decisions D1 and D2.
    ``docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md``
    """

    won: float
    mean: float
    episodes: int = 0
    most_common_share: float = 0.0
    preference_varies: float = 0.0

    @classmethod
    def of_record(cls, record: PopulationRecord) -> ValidationScore:
        """Read the score of one pass from the record of its batch."""
        return cls(
            won=record.won,
            mean=record.mean(),
            episodes=len(record.episodes),
            most_common_share=record.most_common_share,
            preference_varies=record.preference_varies,
        )

    @property
    def key(self) -> tuple[float, float]:
        """The order between two passes: the win share, then the mean return.

        A caller that keeps the best centre compares this and nothing else.
        """
        return (self.won, self.mean)

    def beats(self, other: ValidationScore | None) -> bool:
        """Whether this pass selects over another one, or over nothing."""
        return other is None or self.key > other.key


class EndedWorld(Protocol):
    """What this module needs of a world: its clock and its end record."""

    @property
    def tick(self) -> int:
        """Give back the tick the world stands at."""

    def game_end(self) -> GameEnd | None:
        """Give back how the game ended, or nothing while it runs."""


def end_tick_of(world: EndedWorld) -> int:
    """Return the tick the game of one finished episode ended at.

    **The end record is the source when the world holds one, and the clock of
    the world is the source when it does not.** The two answer different
    endings, and neither answers both.

    A win reader fires inside the interval that one decision runs, so the
    clock stands past the end by up to that interval. The end record holds
    the tick the reader fired at, which is the tick the game ended at. An
    episode that runs to the tick limit also holds an end record, because the
    engine compares held ground at the limit and records a winner there.

    An episode that the horizon truncated holds no end record. The clock is
    then the only statement of how far the episode ran.

    **This function names no signal.** The engine publishes no observation
    field called ``tick``, so a reading of the signals under that name is a
    missing name and never a tick.[^1]

    References
    ----------
    [^1]: Findings register, FND-689. ``docs/FINDINGS.md``
    """
    end = world.game_end()
    if end is None:
        return int(world.tick)
    return int(end["tick"])


def outcome_columns(outcome: str) -> dict[str, float]:
    """Return one column for each outcome, with a one in the column that holds.

    A reading reports one column for each outcome rather than the name
    itself, so a mean over many episodes is the share of episodes that ended
    that way.

    **The set of outcomes comes from the reward module, which declares it.**
    A second list here would be one fact stored twice, with nothing that
    fails when the copies disagree. The engine calls the unfinished state
    ``running`` and a report has always called that column ``unresolved``, so
    this is the one place that translates between the two.
    """
    if outcome not in OUTCOMES:
        message = f"{outcome!r} names no outcome. The reward module holds {OUTCOMES}."
        raise ValueError(message)
    reported = {RUNNING: "unresolved"}
    columns = {reported.get(name, name): 0.0 for name in OUTCOMES}
    columns[reported.get(outcome, outcome)] = 1.0
    return columns


def objective_columns(objectives: Mapping[str, float]) -> dict[str, float]:
    """Return one column for each objective, under a name a reader can group.

    A report holds one row for each episode, and the columns of a row come
    from several sources. The prefix keeps an objective apart from a signal
    of the engine that happens to carry the same name.
    """
    return {
        f"{OBJECTIVE_PREFIX}{name}": float(value) for name, value in objectives.items()
    }


@dataclass(frozen=True)
class EpisodeRecord:
    """What one candidate did on one seed.

    The candidate entry is the index of the policy in its population, and the
    seed entry names the world it played. Both are needed to compare two runs
    seed by seed.

    The chosen entry is how many actions the policy sent to a verb, and the
    refused entry is how many of those the verb did not take. The signals
    entry holds every one-position quantity the engine published at the end of
    the episode, under the name the engine gives it.

    The objectives entry holds what the episode scored on each named
    objective. **A run that stored only the return could not say what the
    episode traded away**, because one number cannot separate a candidate
    that won by taking ground from one that won by fighting. The entry is
    empty for a run scored by a weighting over single fields, which holds no
    objective vector.

    The end tick entry is the tick the game ended at. It is a fact of the
    world and never a signal, because the engine publishes no observation
    field that carries it.[^1]

    The tally entry holds the two behaviour instruments over the decisions of
    this episode. They say whether the policy answered one row at every
    decision, which the return of the episode cannot say.[^2]

    References
    ----------
    [^1]: Findings register, FND-689. ``docs/FINDINGS.md``

    [^2]: Findings register, FND-707. ``docs/FINDINGS.md``
    """

    candidate: int
    seed: int
    total_reward: float
    outcome: str
    decisions: int
    chosen: int
    refused: int
    end_tick: int
    signals: Mapping[str, float] = field(default_factory=dict)
    objectives: Mapping[str, float] = field(default_factory=dict)
    tally: ActionTally = field(default_factory=ActionTally)

    @classmethod
    def of_env(
        cls,
        env: Env,
        candidate: int,
        seed: int,
        total_reward: float,
        chosen: int,
        refused: int,
        scoring_name: str | None = None,
        tally: ActionTally | None = None,
    ) -> EpisodeRecord:
        """Read the record of a finished episode from its environment.

        The signals come from the catalogue the environment built out of the
        schema of the engine. **This module names no position and no field.**
        A tuple of names written here would be a second declaration of what
        the engine publishes, and a name that the schema stopped carrying
        would read as a missing value rather than fail.

        The scoring name asks the environment for the objectives of one of
        the further scorings it read. A name of ``None`` asks the primary
        scoring, which is what one play under one objective wants.

        The end tick comes from the world of the episode, through the one
        reader this module holds for it. **It does not come from the
        signals.** The engine publishes no signal that carries it.
        """
        return cls(
            candidate=candidate,
            seed=seed,
            total_reward=total_reward,
            outcome=env.outcome,
            decisions=env.decisions,
            chosen=chosen,
            refused=refused,
            end_tick=end_tick_of(env.world),
            signals=env.signals.read_scalars(np.asarray(env.observation())),
            objectives=dict(env.objectives_under(scoring_name)),
            tally=ActionTally() if tally is None else tally,
        )

    @property
    def applied(self) -> int:
        """How many chosen actions the verb took."""
        return self.chosen - self.refused

    @property
    def refusal_share(self) -> float:
        """The fraction of the chosen actions that the verb refused.

        An episode that chose nothing refuses nothing, and the share is zero.
        That is the case of the baseline that leaves the seat to the built-in
        controller.
        """
        if self.chosen == 0:
            return 0.0
        return self.refused / self.chosen

    def as_row(self) -> dict[str, float]:
        """Return the reading of this episode, as one flat row of numbers.

        The row holds every one-position signal, one column for each outcome,
        one column for each objective, and the tick the game ended at under
        the name ``end_tick``.

        **The end tick is a fact of the world and never a signal.** This
        column read the signals under the name ``tick`` with a default of
        zero, and the engine publishes no signal of that name. Every episode
        the project recorded therefore carried a zero here, and nothing
        failed.[^1]

        References
        ----------
        [^1]: Findings register, FND-689. ``docs/FINDINGS.md``
        """
        row = {name: float(value) for name, value in self.signals.items()}
        row.update(outcome_columns(self.outcome))
        row["end_tick"] = float(self.end_tick)
        row.update(objective_columns(self.objectives))
        return row

    def as_dict(self) -> dict[str, object]:
        """Return this record as plain values, for a report file."""
        return {
            "candidate": self.candidate,
            "seed": self.seed,
            "return": self.total_reward,
            "outcome": self.outcome,
            "decisions": self.decisions,
            "chosen": self.chosen,
            "refused": self.refused,
            "end_tick": self.end_tick,
            "signals": dict(self.signals),
            "objectives": dict(self.objectives),
            "most_common_share": self.tally.most_common_share,
            "preference_varies": self.tally.preference_varies,
        }


@dataclass(frozen=True)
class PopulationRecord:
    """Every episode of one batch, and the returns they earned.

    The returns array holds one row for each candidate and one column for each
    seed. The episodes hold the same set in one flat sequence, at index
    ``candidate * len(seeds) + seed``, which is the index order the batch
    reports in. **Nothing here reads which worker finished first.**

    The ticks entry is how many world ticks the batch ran, which a run reports
    as the sample cost of the batch.
    """

    seeds: tuple[int, ...]
    returns: np.ndarray
    episodes: tuple[EpisodeRecord, ...]
    ticks: int

    @property
    def candidates(self) -> int:
        """How many policies the batch played."""
        return int(self.returns.shape[0])

    def mean(self) -> float:
        """Return the mean return over every episode of the batch."""
        return float(self.returns.mean())

    @property
    def won(self) -> float:
        """The share of episodes that ended in a win."""
        if not self.episodes:
            return 0.0
        wins = sum(1 for row in self.episodes if row.outcome == "won")
        return wins / len(self.episodes)

    @property
    def chosen(self) -> int:
        """How many actions the batch sent to a verb."""
        return sum(row.chosen for row in self.episodes)

    @property
    def refused(self) -> int:
        """How many of the chosen actions the verbs refused."""
        return sum(row.refused for row in self.episodes)

    @property
    def refusal_share(self) -> float:
        """The fraction of the chosen actions that the verbs refused."""
        if self.chosen == 0:
            return 0.0
        return self.refused / self.chosen

    @property
    def most_common_share(self) -> float:
        """The mean over the episodes of the most common action share."""
        return most_common_share(self.episodes)

    @property
    def preference_varies(self) -> float:
        """The share of the episodes whose unmasked argmax changed."""
        return preference_varies(self.episodes)

    @property
    def objectives(self) -> dict[str, float]:
        """The mean of each objective over every episode of the batch.

        A batch of no episode holds no objective. A batch scored by a
        weighting over single fields holds none either, because such a
        weighting has no objective vector.
        """
        return mean_objectives(self.episodes)

    def rows(self) -> list[dict[str, float]]:
        """Return the reading of every episode, in index order."""
        return [row.as_row() for row in self.episodes]

    def by_seed(self) -> dict[int, tuple[EpisodeRecord, ...]]:
        """Group the episodes by the seed they played.

        A comparison of two policies over a shared seed set pairs on this. The
        keys keep the order the seeds were given in, so a walk over the
        mapping is stable.
        """
        grouped: dict[int, list[EpisodeRecord]] = {seed: [] for seed in self.seeds}
        for row in self.episodes:
            grouped.setdefault(row.seed, []).append(row)
        return {seed: tuple(rows) for seed, rows in grouped.items()}


@dataclass(frozen=True)
class GenerationRecord:
    """What one generation scored, and what the search made of it.

    The ranked entry is the quantity the search ranked, one score for each
    candidate in candidate order. The absolute entry is the mean return of
    each candidate, which a run reports whatever the search ranked, so that a
    reader sees both instruments on every generation.

    The spread entry is the highest ranked score minus the lowest one. The
    informative entry is false when that spread was zero, which means the
    score did not depend on the candidate and the centre did not move.

    The episodes entry is empty for a generation that a seated league played,
    because that path builds its worlds itself and reports no episode.
    """

    generation: int
    seeds: tuple[int, ...]
    ranked: tuple[float, ...]
    absolute: tuple[float, ...]
    spread: float
    absolute_spread: float
    informative: bool
    won: float
    ticks: int
    chosen: int
    refused: int
    episodes: tuple[EpisodeRecord, ...] = ()
    objectives: Mapping[str, float] = field(default_factory=dict)

    @property
    def refusal_share(self) -> float:
        """The fraction of the chosen actions that the verbs refused."""
        if self.chosen == 0:
            return 0.0
        return self.refused / self.chosen

    def summary(
        self,
        validation: ValidationScore | None,
        yardstick: ValidationScore | None,
        seconds: float,
        holdout: ValidationScore | None = None,
    ) -> dict[str, float | None]:
        """Return the one row a run prints and stores for this generation.

        The validation entry is absent on a generation that took no validation
        pass. The yardstick entry is what the built-in controller scored on
        the validation seeds, and the run reports every validation score
        against it, because a relative score cannot say whether the whole
        population improved.

        **The row names which figure selected and which did not.** The
        validation entries come from the seeds that choose the centre, so they
        select. The holdout entries come from seeds that never influenced the
        choice, so they measure. A row that gave one number for both let a
        selection maximum be read as an unbiased measurement.[^1]

        The win entries are the quantity that measures play, and the return
        entries are the shaped training signal. The two are kept apart under
        their own names.

        The degenerate entry is one when the generation carried no
        information and the centre did not move. A reader of the report finds
        the wasted generations by that entry alone.

        References
        ----------
        [^1]: What is wrong with training and evaluation, items 1 and 2.
        ``docs/research/what-is-wrong-with-training-and-evaluation.md``
        """
        return {
            "generation": float(self.generation),
            "best": max(self.ranked),
            "mean": float(np.mean(self.ranked)),
            "worst": min(self.ranked),
            "spread": self.spread,
            "degenerate": 0.0 if self.informative else 1.0,
            "absolute_spread": self.absolute_spread,
            "absolute_mean": float(np.mean(self.absolute)),
            "world_ticks": float(self.ticks),
            "won": self.won,
            "chosen": float(self.chosen),
            "refused": float(self.refused),
            "refusal_share": self.refusal_share,
            "validation": None if validation is None else validation.mean,
            "validation_won": None if validation is None else validation.won,
            "most_common_share": (
                None if validation is None else validation.most_common_share
            ),
            "preference_varies": (
                None if validation is None else validation.preference_varies
            ),
            "holdout": None if holdout is None else holdout.mean,
            "holdout_won": None if holdout is None else holdout.won,
            "yardstick": None if yardstick is None else yardstick.mean,
            "yardstick_won": None if yardstick is None else yardstick.won,
            **objective_columns(self.objectives),
            "above_controller": (
                None
                if validation is None or yardstick is None
                else validation.mean - yardstick.mean
            ),
            "above_controller_won": (
                None
                if validation is None or yardstick is None
                else validation.won - yardstick.won
            ),
            "seconds": seconds,
        }

    def as_dict(self) -> dict[str, object]:
        """Return this record as plain values, for a report file."""
        return {
            "generation": self.generation,
            "seeds": list(self.seeds),
            "ranked": list(self.ranked),
            "absolute": list(self.absolute),
            "spread": self.spread,
            "absolute_spread": self.absolute_spread,
            "degenerate": not self.informative,
            "won": self.won,
            "world_ticks": self.ticks,
            "chosen": self.chosen,
            "refused": self.refused,
            "refusal_share": self.refusal_share,
            "objectives": dict(self.objectives),
            "episodes": [row.as_dict() for row in self.episodes],
        }


def most_common_share(episodes: Sequence[EpisodeRecord]) -> float:
    """Return the mean most common action share over a set of episodes.

    A policy that answers one row at every decision reads one here. **A run
    that reported only a return could not say that**, because the engine's
    legality answer makes a fixed preference order emit many different
    actions.[^1]

    An episode that took no decision is out of the mean, because it emitted
    no action to be common. A set of only such episodes reads zero, and that
    is the state of the built-in controller, which takes no decision through
    this seat.

    **This is the one declaration of the reading.** The record of a batch and
    the summary of a held-out pass both need it, and two copies would be one
    rule stored twice with nothing that fails when they disagree.[^2]

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``

    [^2]: Recurring defect shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """
    held = [row.tally for row in episodes if row.tally.decisions]
    if not held:
        return 0.0
    return float(np.mean([tally.most_common_share for tally in held]))


def preference_varies(episodes: Sequence[EpisodeRecord]) -> float:
    """Return the share of the episodes whose unmasked argmax changed.

    The mask is out of this reading. A policy that holds one fixed preference
    order over the action rows reads zero here whatever it emitted, because
    the row it prefers never moves.[^1]

    An episode whose preference nothing read is out of the share. A set of
    episodes of a policy that publishes no score therefore reads zero.

    **This is the one declaration of the reading**, for the reason the share
    above states.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    held = [row.tally for row in episodes if row.tally.preferences]
    if not held:
        return 0.0
    return sum(1 for tally in held if tally.changes) / len(held)


def mean_objectives(episodes: Sequence[EpisodeRecord]) -> dict[str, float]:
    """Return the mean of each objective over a set of episodes.

    **The objectives come from the episodes and never from a list written
    here.** A list of objective names in this module would be a second
    declaration of the objective vector of the run, and nothing would fail
    when a researcher added an objective and this did not.

    An episode that holds a different objective set than another is refused,
    because a mean over a name that only some episodes carry reads a missing
    value as nothing.
    """
    held = [row for row in episodes if row.objectives]
    if not held:
        return {}
    names = tuple(held[0].objectives)
    for row in held:
        if tuple(row.objectives) != names:
            message = (
                f"these episodes hold different objectives: {names} against "
                f"{tuple(row.objectives)}"
            )
            raise ValueError(message)
    return {
        name: float(np.mean([row.objectives[name] for row in held])) for name in names
    }


def episode_records(
    envs: Sequence[Env],
    seeds: Sequence[int],
    returns: np.ndarray,
    chosen: Sequence[int],
    refused: Sequence[int],
    scoring_name: str | None = None,
    tallies: Sequence[ActionTally] | None = None,
) -> tuple[EpisodeRecord, ...]:
    """Read one record for each world of a finished batch, in index order.

    The world at index ``candidate * len(seeds) + seed`` belongs to that pair,
    because the batch reports in index order and keeps it for every step.

    The scoring name asks each environment for the objectives of one of the
    further scorings it read. A name of ``None`` asks the primary scoring.
    """
    records = []
    for index, env in enumerate(envs):
        candidate, position = divmod(index, len(seeds))
        records.append(
            EpisodeRecord.of_env(
                env,
                candidate=candidate,
                seed=int(seeds[position]),
                total_reward=float(returns[index]),
                chosen=int(chosen[index]),
                refused=int(refused[index]),
                scoring_name=scoring_name,
                tally=None if tallies is None else tallies[index],
            )
        )
    return tuple(records)


__all__ = [
    "OBJECTIVE_PREFIX",
    "ActionTally",
    "EndedWorld",
    "EpisodeRecord",
    "GenerationRecord",
    "PopulationRecord",
    "ValidationScore",
    "end_tick_of",
    "episode_records",
    "mean_objectives",
    "most_common_share",
    "objective_columns",
    "outcome_columns",
    "preference_varies",
]
