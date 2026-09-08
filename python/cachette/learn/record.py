"""What one training run produces, kept so a later reader can measure it.

A run that keeps only a mean throws away the measurements that answer the next
question. This module holds three records. An episode record holds what one
candidate did on one seed. A population record holds every episode of one
batch. A generation record holds what a generation scored and what the search
made of it.

Three gaps in the earlier reporting are the reason each field is here.

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

# References

[^1]: Findings register, FND-670. ``docs/FINDINGS.md``
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING

import numpy as np

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from .env import Env

# The outcome names the engine records for one episode, and the name each one
# takes in a reading. **The reading reports one column for each outcome rather
# than the name itself**, so a mean over episodes is the share of episodes
# that ended that way.
OUTCOME_COLUMNS: dict[str, str] = {
    "won": "won",
    "lost": "lost",
    "drawn": "drawn",
    "running": "unresolved",
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
    """

    candidate: int
    seed: int
    total_reward: float
    outcome: str
    decisions: int
    chosen: int
    refused: int
    signals: Mapping[str, float] = field(default_factory=dict)

    @classmethod
    def of_env(
        cls,
        env: Env,
        candidate: int,
        seed: int,
        total_reward: float,
        chosen: int,
        refused: int,
    ) -> EpisodeRecord:
        """Read the record of a finished episode from its environment.

        The signals come from the catalogue the environment built out of the
        schema of the engine. **This module names no position and no field.**
        A tuple of names written here would be a second declaration of what
        the engine publishes, and a name that the schema stopped carrying
        would read as a missing value rather than fail.
        """
        return cls(
            candidate=candidate,
            seed=seed,
            total_reward=total_reward,
            outcome=env.outcome,
            decisions=env.decisions,
            chosen=chosen,
            refused=refused,
            signals=env.signals.read_scalars(np.asarray(env.observation())),
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
        and the tick of the end under the name a report uses. **The engine
        spells the tick of the end ``tick`` and a report spells it
        ``end_tick``**, and both names are in the row so that no caller has to
        translate between them.
        """
        row = {name: float(value) for name, value in self.signals.items()}
        for outcome, column in OUTCOME_COLUMNS.items():
            row[column] = 1.0 if self.outcome == outcome else 0.0
        row["end_tick"] = float(self.signals.get("tick", 0.0))
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
            "signals": dict(self.signals),
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

    @property
    def refusal_share(self) -> float:
        """The fraction of the chosen actions that the verbs refused."""
        if self.chosen == 0:
            return 0.0
        return self.refused / self.chosen

    def summary(
        self,
        validation: float | None,
        yardstick: float | None,
        seconds: float,
    ) -> dict[str, float | None]:
        """Return the one row a run prints and stores for this generation.

        The validation entry is absent on a generation that took no validation
        pass. The yardstick entry is what the built-in controller scored on
        the validation seeds, and the run reports every validation score
        against it, because a relative score cannot say whether the whole
        population improved.
        """
        return {
            "generation": float(self.generation),
            "best": max(self.ranked),
            "mean": float(np.mean(self.ranked)),
            "worst": min(self.ranked),
            "spread": self.spread,
            # One when the generation carried no information and the centre
            # did not move. A reader of the report finds the wasted
            # generations by this field alone.
            "degenerate": 0.0 if self.informative else 1.0,
            "absolute_spread": self.absolute_spread,
            "absolute_mean": float(np.mean(self.absolute)),
            "world_ticks": float(self.ticks),
            "won": self.won,
            "chosen": float(self.chosen),
            "refused": float(self.refused),
            "refusal_share": self.refusal_share,
            "validation": validation,
            "yardstick": yardstick,
            "above_controller": (
                None
                if validation is None or yardstick is None
                else validation - yardstick
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
            "episodes": [row.as_dict() for row in self.episodes],
        }


def episode_records(
    envs: Sequence[Env],
    seeds: Sequence[int],
    returns: np.ndarray,
    chosen: Sequence[int],
    refused: Sequence[int],
) -> tuple[EpisodeRecord, ...]:
    """Read one record for each world of a finished batch, in index order.

    The world at index ``candidate * len(seeds) + seed`` belongs to that pair,
    because the batch reports in index order and keeps it for every step.
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
            )
        )
    return tuple(records)


__all__ = [
    "OUTCOME_COLUMNS",
    "EpisodeRecord",
    "GenerationRecord",
    "PopulationRecord",
    "episode_records",
]
