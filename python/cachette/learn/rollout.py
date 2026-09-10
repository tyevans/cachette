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

# One play answers for several objectives

A baseline return separates into two things. The episodes are the games the
seat plays, and they depend on the world shape and the seed set. The
weighting turns the readings of those games into one number, and only it
differs between two objectives.

This module therefore holds a pass that plays the episodes once and scores
them once for each objective. A run of six objectives plays one batch, and a
seventh objective then costs the arithmetic of one more scorer.

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
from collections.abc import Sequence
from dataclasses import dataclass, field, replace
from typing import TYPE_CHECKING

import numpy as np

from .env import VectorEnv
from .league import run_seated_population
from .policy import masked_choices, preferred_rows
from .record import (
    ActionTally,
    EpisodeRecord,
    PopulationRecord,
    episode_records,
)

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping

    from .config import TrainConfig
    from .env import EnvConfig
    from .policy import Policy
    from .reward import Scoring

# How often a long call says that it is still working. A generation of the
# usual size takes several minutes, so a reader needs a line inside it. Thirty
# seconds is often enough to tell a slow generation from a stopped one, and
# rare enough to leave the log readable.
HEARTBEAT_SECONDS = 30.0


def run_population(
    config: EnvConfig,
    scoring: Scoring | Sequence[Scoring],
    policies: Sequence[Policy],
    seeds: Sequence[int],
    workers: int,
    label: str = "",
) -> PopulationRecord:
    """Play every policy on every seed, and return one record of the batch.

    The scoring entry is what the seat is rewarded for. **A caller may give
    one scoring for each seed position, and it may not give one for each
    candidate.** Every candidate then plays position zero under the same
    objective, position one under the same objective, and so on, so two
    candidates of one batch are always comparable. An evolution strategy
    ranks the candidates against each other, and a rank over two objectives
    carries no information about either policy.

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

    **The loop takes the observation from the step and never asks a world for
    it.** The reward of a decision reads the array of the state after the
    ticks, and that state is the state the next decision hands the policy. So
    the step already holds the array the loop needs, and a loop that asked a
    world again built the same numbers a second time. Every one of those
    builds ran in this interpreter with every engine worker idle.[^1]

    References
    ----------
    [^1]: Report 38, where the training time goes, section 10.2.
    ``docs/research/reports/38-where-the-training-time-goes.md``
    """
    ordered = [int(seed) for seed in seeds]
    pairs = _pairs(len(policies), len(ordered))
    positions = _each_position(scoring, len(ordered))
    vector = VectorEnv(
        config,
        [positions[position] for _, position in pairs],
        count=len(pairs),
        workers=workers,
    )
    played = _drive(vector, policies, ordered, pairs, label)
    return _record_of(vector, played, policies, ordered, None)


def run_objectives(
    config: EnvConfig,
    scorings: Mapping[str, Scoring],
    policies: Sequence[Policy],
    seeds: Sequence[int],
    workers: int,
    label: str = "",
) -> dict[str, PopulationRecord]:
    """Play the episodes once, and return one record for each objective.

    **The episodes do not depend on the objective.** The world comes from the
    configuration and the seed. The action comes from the policy, and a world
    the built-in controller holds takes no action at all. The end of an
    episode comes from the outcome reader, which reads the observation of the
    faction and the recorded end of the game. A scoring weights the readings
    of an episode and never moves it, so one play answers for every scoring.

    A caller that wants six numbers therefore plays one batch rather than six.
    Each environment holds one scorer for each objective, and every scorer
    reads the same decision.

    The result holds one record for each name, under the name the caller gave.
    **The names keep the order of the mapping the caller passed**, so the
    order of the result comes from a key the caller stated and never from
    which world or worker finished first.

    A caller that gives one objective gets the same answer as a call that
    plays that objective alone, which is the property the tests fix.
    """
    if not scorings:
        message = "a shared pass plays at least one objective"
        raise ValueError(message)
    ordered = [int(seed) for seed in seeds]
    pairs = _pairs(len(policies), len(ordered))
    names = list(scorings)
    primary = names[0]
    vector = VectorEnv(
        config,
        scorings[primary],
        count=len(pairs),
        workers=workers,
        also={name: scorings[name] for name in names[1:]},
    )
    played = _drive(vector, policies, ordered, pairs, label)
    records = {primary: _record_of(vector, played, policies, ordered, None)}
    for name in names[1:]:
        shared = replace(played, returns=played.also[name])
        records[name] = _record_of(vector, shared, policies, ordered, name)
    return records


def _pairs(candidates: int, seeds: int) -> list[tuple[int, int]]:
    """Return the candidate and seed position of each world, in index order."""
    return [(c, s) for c in range(candidates) for s in range(seeds)]


@dataclass(frozen=True)
class _Played:
    """What one batch earned, before any record is built from it.

    The returns entry holds the return of each world under the primary
    scoring, in index order. The also entry holds the same for each further
    scoring, under the name the caller gave.

    The tallies entry holds the two behaviour instruments of each world, in
    the same index order.
    """

    returns: np.ndarray
    also: Mapping[str, np.ndarray]
    chosen: list[int]
    refused: list[int]
    tallies: list[ActionTally]


def _drive(
    vector: VectorEnv,
    policies: Sequence[Policy],
    seeds: Sequence[int],
    pairs: Sequence[tuple[int, int]],
    label: str,
) -> _Played:
    """Reset the batch, run it to the end, and total what each world earned.

    **This is the one declaration of the batch loop.** Two callers need it:
    one that plays a population under one objective, and one that plays it
    once under several. A second copy of the loop would be one rule stored
    twice, with nothing that fails when the copies disagree.

    **The loop instruments what each policy answered, not only what it
    earned.** It keeps the action each world emitted and the row each policy
    preferred over the unmasked scores. Those two say whether a policy
    answered one row at every decision, and no figure of a run said so.[^1]

    A policy that publishes a score matrix pays nothing for the instrument.
    The loop masks that matrix itself, which is the arithmetic the policy
    would have done, and it reads the unmasked argmax of the same matrix. A
    policy that publishes none reports no preference, and the tally then says
    that the preference was never read.

    **A pass whose seat the built-in controller holds asks no policy for a
    choice.** The environment of such a world discards an action, so every
    number this loop reports is the same without one.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    observations = vector.reset([seeds[s] for _, s in pairs])
    names = vector.envs[0].also_names
    returns = np.zeros(len(pairs))
    also = {name: np.zeros(len(pairs)) for name in names}
    chosen = [0] * len(pairs)
    refused = [0] * len(pairs)
    emitted: list[list[int]] = [[] for _ in pairs]
    preferred: list[list[int]] = [[] for _ in pairs]
    started = time.perf_counter()
    spoke = started
    told = 0
    decisions = 0
    while not vector.done:
        actions, rows = _choices(vector, policies, len(seeds), observations)
        results = vector.step(actions)
        observations = np.stack([result.observation for result in results])
        for index, result in enumerate(results):
            returns[index] += result.reward
            paid = result.info["also"]
            for name in names:
                also[name][index] += paid[name]
            applied = result.info.get("applied")
            if applied is None:
                continue
            chosen[index] += 1
            if not applied:
                refused[index] += 1
            emitted[index].append(actions[index])
            if rows[index] >= 0:
                preferred[index].append(rows[index])
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

    return _Played(
        returns=returns,
        also=also,
        chosen=chosen,
        refused=refused,
        tallies=[
            ActionTally.of_actions(emitted[index], preferred[index])
            for index in range(len(pairs))
        ],
    )


def _choices(
    vector: VectorEnv,
    policies: Sequence[Policy],
    seeds: int,
    observations: np.ndarray,
) -> tuple[list[int], list[int]]:
    """Return the action of each world, and the row each policy preferred.

    **A world whose seat the built-in controller holds reads no action**, so
    this asks no policy for one. The environment of such a world answers
    nothing from its apply step, the loop counts no decision for it, and the
    instrument reports no preference. Every one of those answers is the same
    answer this gives, and the pass no longer pays for the choice behind it.

    The choice is the largest term of the section that one interpreter runs
    between two decisions, and every engine worker of the process waits for
    that section. A controller pass over a wide world spent about a twelfth
    of its wall clock building a score matrix that no seat read.

    A row of minus one says that nothing read the preference of that world.
    A policy that publishes no score matrix reports the same, because a
    uniform draw and a fixed no-op prefer no row.
    """
    count = len(vector)
    actions = [0] * count
    rows = [-1] * count
    if not vector.controlled:
        return actions, rows
    masks = vector.action_masks()
    # Each candidate scores its own worlds. The rows of one candidate are
    # contiguous, so one matrix product answers for all of them.
    for candidate, policy in enumerate(policies):
        first = candidate * seeds
        last = first + seeds
        scores = _score_matrix(policy, observations[first:last])
        if scores is None:
            actions[first:last] = policy.choose_many(
                observations[first:last], masks[first:last]
            )
        else:
            actions[first:last] = masked_choices(scores, masks[first:last])
            rows[first:last] = preferred_rows(scores)
    return actions, rows


def _score_matrix(policy: Policy, observations: np.ndarray) -> np.ndarray | None:
    """Return the unmasked scores of one policy, or nothing when it has none.

    The two baselines publish no score. A random draw and a fixed no-op have
    no preference over the action rows, so the instrument reports none for
    them rather than inventing one.
    """
    scored = getattr(policy, "scores_many", None)
    if scored is None:
        return None
    return np.asarray(scored(observations))


def _record_of(
    vector: VectorEnv,
    played: _Played,
    policies: Sequence[Policy],
    seeds: Sequence[int],
    scoring_name: str | None,
) -> PopulationRecord:
    """Build the record of one objective over a batch that already ran."""
    return PopulationRecord(
        seeds=tuple(seeds),
        returns=played.returns.reshape(len(policies), len(seeds)),
        episodes=episode_records(
            vector.envs,
            seeds,
            played.returns,
            played.chosen,
            played.refused,
            scoring_name,
            played.tallies,
        ),
        ticks=vector.world_ticks,
    )


def _each_position(
    scoring: Scoring | Sequence[Scoring], seeds: int
) -> tuple[Scoring, ...]:
    """Return the scoring of each seed position of one batch.

    One scoring answers for every position. A sequence answers for one
    position each, and it holds exactly one entry for each seed.

    **The result is indexed by the seed position and never by the
    candidate.** The batch repeats it for every candidate, so a caller cannot
    give one candidate an objective that another candidate did not play.
    """
    if isinstance(scoring, Sequence):
        held = tuple(scoring)
        if len(held) != seeds:
            message = f"the batch plays {seeds} seeds and holds {len(held)} scorings"
            raise ValueError(message)
        return held
    return (scoring,) * seeds


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

    The objectives entry holds the mean of each objective over the episodes.
    It is empty for a generation scored by a weighting over single fields,
    which holds no objective vector.
    """

    ranked: np.ndarray
    absolute: np.ndarray
    won: float
    ticks: int
    chosen: int = 0
    refused: int = 0
    episodes: tuple[EpisodeRecord, ...] = ()
    objectives: Mapping[str, float] = field(default_factory=dict)


def one_scoring(scoring: Scoring | Sequence[Scoring]) -> Scoring:
    """Return the one scoring of a batch that admits no variation.

    A seated league builds its own worlds, so it takes one scoring for the
    whole batch. A caller that varies the scoring by seed position therefore
    cannot use that path yet, and this says so rather than scoring the league
    under the first entry of the schedule.
    """
    if not isinstance(scoring, Sequence):
        return scoring
    held = tuple(scoring)
    if len(held) == 1:
        return held[0]
    message = (
        "a seated league scores one batch under one objective, and this batch "
        f"holds {len(held)}. A league builds its own worlds, so it has no seed "
        "position to vary on."
    )
    raise ValueError(message)


def score_generation(
    env_config: EnvConfig,
    scoring: Scoring | Sequence[Scoring],
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
            env_config, scoring, candidates, seeds, train_config.workers, label
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
            objectives=played.objectives,
        )
    result = run_seated_population(
        env_config,
        one_scoring(scoring),
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
    "one_scoring",
    "run_objectives",
    "run_population",
    "score_generation",
]
