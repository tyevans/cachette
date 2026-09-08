"""Play a population of policies over a set of seeds, in one batch.

A generation of an evolution strategy is many episodes that differ only in the
policy that drives them. This module plays them. It builds one world for each
pair of a candidate and a seed, steps them all through one batch, and scores
each one.

**The batch orders its results by the index of the world.** The score of a
candidate therefore never depends on which worker finished first.[^1]

# The seed set is fixed inside a generation

Every candidate of one generation plays the same worlds. That removes the
variance that would otherwise drown a small population. The caller moves the
set between generations, so a policy cannot learn one map.

# A candidate may be scored against the seats it played

A generation may put more than one candidate in one world. Two candidates in
one game share the map, the weather and the opponents, so the difference
between their returns holds almost none of the variance that either return
holds on its own. A seated run then ranks that difference rather than the raw
return.

**A relative score cannot say whether the population improved.** It is zero on
average by construction, so a population that got worse together reads the
same as one that got better together. The caller therefore plays its centre
against the built-in controller on a held-out seed set as well.

# References

[^1]: ADR-0155, a batch of worlds steps in one call, in index order.
``docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md``
"""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import TYPE_CHECKING

import numpy as np

from .env import VectorEnv
from .league import run_seated_population
from .record import EpisodeRecord, PopulationRecord, episode_records

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

    from .config import TrainConfig
    from .env import EnvConfig
    from .policy import Policy
    from .reward import Weighting

# How often a long call says that it is still working. A generation of the
# usual size takes several minutes, so a reader needs a line inside it. Thirty
# seconds is often enough to tell a slow generation from a stopped one, and
# rare enough to leave the log readable.
HEARTBEAT_SECONDS = 30.0


def run_population(
    config: EnvConfig,
    weighting: Weighting,
    policies: Sequence[Policy],
    seeds: Sequence[int],
    workers: int,
    label: str = "",
) -> PopulationRecord:
    """Play every policy on every seed, and return one record of the batch.

    The label names what is playing, for example ``conquer generation  3``. A
    call that gives one reports progress while it runs. A call that gives none
    stays silent, which is what a short call wants.

    The world at index ``candidate * len(seeds) + seed`` belongs to that pair.
    The batch reports in index order, so the mapping holds for every step.

    The record counts what each world chose and what its verb refused. **The
    engine answers that on every action, and the batch used to throw the
    answer away**, so no figure of a run said how much of what a policy chose
    the engine carried out. A policy the engine mostly refuses is close to a
    no-op whatever it chooses.
    """
    ordered = [int(seed) for seed in seeds]
    pairs = [(c, s) for c in range(len(policies)) for s in range(len(ordered))]
    vector = VectorEnv(config, weighting, count=len(pairs), workers=workers)
    vector.reset([ordered[s] for _, s in pairs])

    returns = np.zeros(len(pairs))
    chosen = [0] * len(pairs)
    refused = [0] * len(pairs)
    started = time.perf_counter()
    spoke = started
    told = 0
    decisions = 0
    while not vector.done:
        observations = np.stack([env.observation() for env in vector.envs])
        masks = vector.action_masks()
        actions = [0] * len(pairs)
        # Each candidate scores its own worlds. The rows of one candidate are
        # contiguous, so one matrix product answers for all of them.
        for candidate, policy in enumerate(policies):
            first = candidate * len(ordered)
            last = first + len(ordered)
            picked = policy.choose_many(observations[first:last], masks[first:last])
            actions[first:last] = picked
        for index, result in enumerate(vector.step(actions)):
            returns[index] += result.reward
            applied = result.info.get("applied")
            if applied is None:
                continue
            chosen[index] += 1
            if not applied:
                refused[index] += 1
        decisions += 1

        # **A generation says it is working while it works.** A generation of
        # this size takes several minutes, and the row that reports it comes
        # only at the end. Without this line, a slow generation and a stopped
        # one look the same from outside, and a reader can only guess from the
        # load of the machine.
        #
        # **The clock decides when to print, and nothing else.** No simulated
        # value reads it, and the printing changes no state, so this cannot
        # move a result. It is not a time budget and it ends nothing.
        now = time.perf_counter()
        if label and now - spoke >= HEARTBEAT_SECONDS:
            # **The rate is what happened since the last line, not since the
            # start.** A world leaves the batch when its episode ends, so the
            # live count falls through a generation and a rate taken over the
            # whole elapsed time falls with it. That reads as a machine
            # slowing down when it is only running fewer worlds.
            window = now - spoke
            since = vector.world_ticks - told
            spoke = now
            told = vector.world_ticks
            live = sum(1 for env in vector.envs if not env.done)
            elapsed = now - started
            rate = since / window if window else 0.0
            print(
                f"  {label} working  decisions {decisions:5d} "
                f"live {live:4d}/{len(pairs):<4d} "
                f"ticks {vector.world_ticks:9d} rate {rate:8.1f} t/s "
                f"[{elapsed:.0f}s]",
                flush=True,
            )

    return PopulationRecord(
        seeds=tuple(ordered),
        returns=returns.reshape(len(policies), len(ordered)),
        episodes=episode_records(vector.envs, ordered, returns, chosen, refused),
        ticks=vector.world_ticks,
    )


@dataclass(frozen=True)
class Generation:
    """What one generation of candidates scored.

    The ranked entry is the quantity the search ranks. The absolute entry is
    the mean return of each candidate, which is reported whatever the search
    ranks, so that a reader sees both instruments on every generation.

    The chosen and refused entries count the actions of the whole generation
    and how many of them a verb did not take.

    The episodes entry holds one record for each world the generation played.
    **It is empty for a generation that a seated league played**, because that
    path builds its own worlds and reports no episode.
    """

    ranked: np.ndarray
    absolute: np.ndarray
    won: float
    ticks: int
    chosen: int = 0
    refused: int = 0
    episodes: tuple[EpisodeRecord, ...] = ()


def score_generation(
    env_config: EnvConfig,
    weighting: Weighting,
    candidates: Sequence[Policy],
    seeds: Sequence[int],
    train_config: TrainConfig,
    label: str = "",
) -> Generation:
    """Play one generation, and return the score the update ranks.

    A run with no learner seats plays one candidate in one world, and the
    score is the mean return over the seeds. A run with two or more learner
    seats puts that many candidates in one world, and the score is the mean
    margin against the other seats of the same world.
    """
    if not train_config.learner_seats:
        played = run_population(
            env_config, weighting, candidates, seeds, train_config.workers, label
        )
        absolute = played.returns.mean(axis=1)
        return Generation(
            ranked=absolute,
            absolute=absolute,
            won=played.won,
            ticks=played.ticks,
            chosen=played.chosen,
            refused=played.refused,
            episodes=played.episodes,
        )
    result = run_seated_population(
        env_config,
        weighting,
        candidates,
        list(seeds),
        train_config.learner_seats,
        train_config.workers,
    )
    absolute = result.absolute.mean(axis=1)
    ranked = result.relative.mean(axis=1) if train_config.relative else absolute
    return Generation(
        ranked=ranked, absolute=absolute, won=result.won, ticks=result.ticks
    )


__all__ = [
    "HEARTBEAT_SECONDS",
    "Generation",
    "run_population",
    "score_generation",
]
