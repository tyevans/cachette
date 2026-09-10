"""A measurement pass, played one episode at a time over the shared queue.

A training run measures as much as it trains. It plays the built-in
controller on the held-out seeds for the bar, it plays its centre on the
validation seeds to choose a checkpoint, and it plays the published centre on
the held-out seeds for the figure the run is judged on. A plan of two styles
at population 32 over 6 seeds, 40 generations, validation of 128 every second
generation and a holdout of 256 every fifth counts 6,144 measurement episodes
against 7,680 training episodes.

**Those passes stepped many worlds in one process, and that shape does not
fill a machine.** Between two decisions one interpreter stacks the
observations, builds the action masks and runs the policy matrix products,
and every engine worker of the process waits for it. A measurement on a world
of 128 columns put that section at about a seventh of the wall clock of a
pass, which bounds one process at about seven busy cores however many workers
it asks for. A pass over 256 worlds on a machine of 64 cores reached about a
fifth of it.

The training generation already answers this. One task is one episode, a pool
of single-threaded worker processes takes any task from one queue, and the
combination sorts on a stable key.[^1] A measurement episode is the same
shape without the candidate dimension, so this module puts it in the same
queue.

# One task is one episode, and the key is the policy, the repeat and the seed

A task names one policy, one repeat of the seed set and one seed. The
combination sorts on those three and builds the arrays in that order.
**Nothing reads the order in which the workers answered.**[^2] An episode is
a pure function of the policy and the seed, and that is what makes a queue
admissible here at all.

The combination refuses a set of results that does not cover every policy on
every repeat of every seed exactly once. A pass that lost an episode would
otherwise report a mean over a cell that no episode filled.

# One play answers for every objective

A baseline return separates into two things. The episodes are the games the
seat plays, and they come from the world, the seed set and the engine build.
The objective weights the readings of those games. A task therefore carries
every objective of the pass and its worker plays one episode and scores it
once for each, so a pass of six objectives plays one set of games.

# The policy is named, and never carried

The weight matrix of a policy on a wide world holds about seven megabytes. A
task that carried it would send that array once for each episode, which is
nearly two gigabytes for a pass of 256 episodes, through the one queue that
every strategy shares. The pool writes the policy once and the task names
it.[^3]

# A drawing policy draws its own stream

The engine is deterministic, so only a policy that draws gains anything from
a repeat of the seed set. A pass in one process gave every world of a batch
one stream, and a repeat continued it. A queued pass plays each episode in a
worker process, and every one of them would start from the same state.

So a queued pass keys the stream on the policy, the repeat and the seed
position. Every episode then draws its own stream, the pass gives the same
answer at any pool size, and the repeat narrows the baseline in the way it
was meant to. **A queued pass and a pass in one process therefore report
different numbers for a drawing policy**, and each is repeatable on its own
terms. No policy this project trains draws.

# References

[^1]: ADR-0194, a generation is scored one episode at a time, and combined in
candidate order, decisions D1 and D3.
``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D2.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``
[^3]: ADR-0194, a generation is scored one episode at a time, and combined in
candidate order, decision D2.
``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
"""

from __future__ import annotations

from dataclasses import dataclass, replace
from typing import TYPE_CHECKING, cast

import numpy as np

from .record import PopulationRecord
from .rollout import run_objectives
from .shard import THREADS_FOR_ONE_EPISODE, SharedValue, shared_value

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    from .env import EnvConfig
    from .policy import Policy
    from .record import EpisodeRecord
    from .reward import Scoring
    from .shard import Pending, ShardPool

# The name a pass of one objective gives that objective. A caller that wants
# one record rather than one for each name never sees it, and the record it
# gets names no scoring, which is what a pass of one objective reports.
SOLE = ""


@dataclass(frozen=True)
class MeasurementTask:
    """What one worker process needs in order to play one measured episode.

    The policy entry names where the pool wrote the policy. It is a name and
    not the policy, because the policy of a wide world is far larger than
    everything else in this record put together.

    The scorings entry holds every objective of the pass, in the order the
    caller gave. The first one is the primary, and the episode it records
    names no scoring, in the way a pass of one objective records it.

    The policy index, the repeat and the seed position are the stable key the
    combination sorts on. The repeat plays the seed set again, which changes
    an episode only for a policy that draws.
    """

    env_config: EnvConfig
    scorings: tuple[tuple[str, Scoring], ...]
    policy: SharedValue
    policy_index: int
    repeat: int
    seed: int
    seed_position: int


@dataclass(frozen=True)
class MeasurementScore:
    """What one measured episode earned, under every objective of the pass.

    The returns entry holds the return of the episode under each objective,
    and the episodes entry holds its record under each. Both are keyed on the
    name the caller of the pass gave.

    The policy index, the repeat and the seed position are the key the
    combination orders this result by.
    """

    policy_index: int
    repeat: int
    seed_position: int
    returns: Mapping[str, float]
    episodes: Mapping[str, EpisodeRecord]
    ticks: int

    @property
    def key(self) -> tuple[int, int, int]:
        """The stable key the combination orders this result by."""
        return (self.policy_index, self.repeat, self.seed_position)


def measurement_tasks(
    env_config: EnvConfig,
    scorings: Mapping[str, Scoring],
    policies: Sequence[SharedValue],
    seeds: Sequence[int],
    repeats: int = 1,
) -> list[MeasurementTask]:
    """Return one task for each episode of one measurement pass.

    **The task count reads the policies, the repeats and the seeds, and never
    the worker count.** A pool of any size plays the same tasks, so the size
    of the pool reaches how long the pass takes and reaches nothing it
    answers.

    The tasks come back in policy order, then repeat order, then seed order.
    That order decides which episode a free worker starts first and it
    decides nothing else, because the combination sorts on the key of each
    result.
    """
    held = tuple(scorings.items())
    return [
        MeasurementTask(
            env_config=env_config,
            scorings=held,
            policy=policy,
            policy_index=index,
            repeat=repeat,
            seed=int(seed),
            seed_position=position,
        )
        for index, policy in enumerate(policies)
        for repeat in range(repeats)
        for position, seed in enumerate(seeds)
    ]


def play_measurement(task: MeasurementTask) -> MeasurementScore:
    """Play one measured episode, and score it under every objective.

    This runs in a worker process. It reads the policy the pool shared, gives
    that policy the stream of this episode when it draws, and plays one
    world.

    **The engine runs one thread here.** One episode holds one world, and
    more threads for one step is slower at every extent this project plays
    on, so the worker asks for one whatever the run asked for.
    """
    held = cast("Policy", shared_value(task.policy))
    policy = drawing_from(held, task.repeat, task.seed_position)
    names = [name for name, _ in task.scorings]
    played = run_objectives(
        task.env_config,
        dict(task.scorings),
        [policy],
        [task.seed],
        THREADS_FOR_ONE_EPISODE,
    )
    return MeasurementScore(
        policy_index=task.policy_index,
        repeat=task.repeat,
        seed_position=task.seed_position,
        returns={name: float(played[name].returns[0, 0]) for name in names},
        episodes={name: played[name].episodes[0] for name in names},
        ticks=played[names[0]].ticks,
    )


def drawing_from(policy: Policy, repeat: int, position: int) -> Policy:
    """Return the policy on the stream that one episode owns.

    A policy that states no stream is given back as it stands. Every policy
    this project trains is such a policy: it reads the observation and
    answers, and it draws nothing.

    A policy that draws answers this by building itself again on a stream
    keyed on the repeat and the seed position. Two episodes of one pass
    therefore draw different actions, and a repeat of the whole pass gives
    the same two.
    """
    reseed = getattr(policy, "reseed", None)
    if reseed is None:
        return policy
    drawn: Policy = reseed(repeat, position)
    return drawn


def combine_measurements(
    scores: Sequence[MeasurementScore],
    names: Sequence[str],
    policies: int,
    seeds: Sequence[int],
    repeats: int = 1,
) -> dict[str, list[PopulationRecord]]:
    """Put the episode scores in key order, and give back one record for each.

    **The order comes from the policy index, the repeat and the seed
    position, and from nothing else.** The workers may answer in any order,
    and the answer must not move.

    The result holds one entry for each objective, under the name the caller
    gave, and each entry holds one record for each repeat of the seed set. A
    caller that asked for one repeat therefore reads a list of one, which is
    what a pass in one process gives back.

    Raises ``ValueError`` when the results do not cover every policy on every
    repeat of every seed exactly once. A pass that lost an episode would
    otherwise report a mean over a cell that no episode filled.
    """
    if policies < 1 or not seeds or repeats < 1:
        message = (
            f"a measurement pass plays at least one policy on one seed once, "
            f"and this names {policies} policies on {len(seeds)} seeds over "
            f"{repeats} repeats"
        )
        raise ValueError(message)
    ordered = sorted(scores, key=lambda score: score.key)
    _refuse_a_gap(ordered, names, policies, len(seeds), repeats)
    # **One walk fills every repeat, and the walk is over the sorted list.**
    # The list is in key order, so each repeat collects its own episodes in
    # policy order and then in seed order, which is the index order a batch in
    # one process reports.
    ticks = [0] * repeats
    held: list[list[MeasurementScore]] = [[] for _ in range(repeats)]
    for score in ordered:
        ticks[score.repeat] += score.ticks
        held[score.repeat].append(score)
    return {
        name: [
            PopulationRecord(
                seeds=tuple(int(seed) for seed in seeds),
                returns=np.array(
                    [score.returns[name] for score in held[repeat]]
                ).reshape(policies, len(seeds)),
                episodes=tuple(
                    replace(score.episodes[name], candidate=score.policy_index)
                    for score in held[repeat]
                ),
                ticks=ticks[repeat],
            )
            for repeat in range(repeats)
        ]
        for name in names
    }


def _refuse_a_gap(
    ordered: Sequence[MeasurementScore],
    names: Sequence[str],
    policies: int,
    seeds: int,
    repeats: int,
) -> None:
    """Refuse a set of results that does not cover the pass exactly once.

    A result that falls outside the pass, a cell that two results wrote, and
    a cell that no result wrote are all failures here. A mean over a cell
    that nobody wrote reads as a score, and nothing else would say so.

    A result that holds another set of objectives than the pass asked for is
    refused as well, because the combination would otherwise read a name that
    the worker never scored.
    """
    filled: set[tuple[int, int, int]] = set()
    wanted = set(names)
    for score in ordered:
        index, repeat, position = score.key
        if not (0 <= index < policies and 0 <= repeat < repeats):
            message = (
                f"an episode of policy {index} on repeat {repeat} falls "
                f"outside a pass of {policies} policies over {repeats} repeats"
            )
            raise ValueError(message)
        if not 0 <= position < seeds:
            message = (
                f"an episode at seed position {position} falls outside a pass "
                f"of {seeds} seeds"
            )
            raise ValueError(message)
        if score.key in filled:
            message = (
                f"two episodes scored policy {index} on repeat {repeat} at "
                f"seed position {position}, so one of them would be lost"
            )
            raise ValueError(message)
        if set(score.returns) != wanted:
            message = (
                f"an episode scored {sorted(score.returns)} and the pass asks "
                f"for {sorted(wanted)}"
            )
            raise ValueError(message)
        filled.add(score.key)
    missing = policies * repeats * seeds - len(filled)
    if missing:
        for index in range(policies):
            for repeat in range(repeats):
                for position in range(seeds):
                    if (index, repeat, position) not in filled:
                        message = (
                            f"the episodes do not cover the pass: policy "
                            f"{index} played no episode on repeat {repeat} at "
                            f"seed position {position}, and {missing} of "
                            f"{policies * repeats * seeds} episodes are missing"
                        )
                        raise ValueError(message)


class MeasurementPass:
    """One measurement pass that a pool holds, and the records it will give.

    **A caller may submit a pass and read it later.** The controller baseline
    is the pass that needs this: it produces the bar a report states, and
    nothing that trains a weight reads it. A run that waited for it spent
    about twenty minutes before its first generation and left the queue empty
    for all of it.

    The pass owns what the pool shared for it, and it releases that when it
    gives its records back.
    """

    def __init__(
        self,
        pool: ShardPool,
        pending: Pending[MeasurementScore],
        shared: Sequence[SharedValue],
        names: Sequence[str],
        policies: int,
        seeds: Sequence[int],
        repeats: int,
    ) -> None:
        """Hold what one submitted pass needs in order to combine itself."""
        self._pool = pool
        self._pending = pending
        self._shared = list(shared)
        self._names = list(names)
        self._policies = policies
        self._seeds = list(seeds)
        self._repeats = repeats

    def __len__(self) -> int:
        """How many episodes this pass holds."""
        return len(self._pending)

    @property
    def done(self) -> bool:
        """Whether every episode of this pass has finished."""
        return self._pending.done

    def records(self, label: str = "") -> dict[str, list[PopulationRecord]]:
        """Wait for every episode, and return one record for each objective.

        The label names the pass in a progress line. A call that gives one
        reports what has finished while it waits.
        """
        try:
            scores = self._pending.results(label)
        finally:
            for shared in self._shared:
                self._pool.release(shared)
            self._shared = []
        return combine_measurements(
            scores, self._names, self._policies, self._seeds, self._repeats
        )


def start_measurement(
    pool: ShardPool,
    env_config: EnvConfig,
    scorings: Mapping[str, Scoring],
    policies: Sequence[Policy],
    seeds: Sequence[int],
    repeats: int = 1,
) -> MeasurementPass:
    """Submit every episode of one measurement pass, and return before it runs.

    The caller reads the records when it needs them. A caller whose own work
    does not depend on them submits here and reads them later, and the queue
    then holds the measurement beside whatever else the run is playing.
    """
    if not scorings:
        message = "a measurement pass plays at least one objective"
        raise ValueError(message)
    shared = [pool.share(policy) for policy in policies]
    tasks = measurement_tasks(env_config, scorings, shared, seeds, repeats)
    return MeasurementPass(
        pool=pool,
        pending=pool.start(play_measurement, tasks),
        shared=shared,
        names=list(scorings),
        policies=len(policies),
        seeds=seeds,
        repeats=repeats,
    )


def queued_objectives(
    pool: ShardPool,
    env_config: EnvConfig,
    scorings: Mapping[str, Scoring],
    policies: Sequence[Policy],
    seeds: Sequence[int],
    label: str = "",
) -> dict[str, PopulationRecord]:
    """Play the episodes once over the queue, and score them for each objective.

    This is what a pass in one process returns, over the same episodes and
    under the same names, in the order the caller gave.
    """
    pass_ = start_measurement(pool, env_config, scorings, policies, seeds)
    return {name: held[0] for name, held in pass_.records(label).items()}


def queued_population(
    pool: ShardPool,
    env_config: EnvConfig,
    scoring: Scoring,
    policies: Sequence[Policy],
    seeds: Sequence[int],
    label: str = "",
) -> PopulationRecord:
    """Play every policy on every seed over the queue, and return one record."""
    return queued_objectives(pool, env_config, {SOLE: scoring}, policies, seeds, label)[
        SOLE
    ]


def queued_repeats(
    pool: ShardPool,
    env_config: EnvConfig,
    scoring: Scoring,
    policy: Policy,
    seeds: Sequence[int],
    repeats: int = 1,
    label: str = "",
) -> list[PopulationRecord]:
    """Play one policy on a seed set several times over the queue.

    **Every repeat of every seed is one task of the one queue.** A pass that
    played the repeats one after another would drain the queue at the end of
    each of them, and a repeat exists to narrow a drawing baseline rather
    than to order the work.
    """
    pass_ = start_measurement(
        pool, env_config, {SOLE: scoring}, [policy], seeds, max(1, repeats)
    )
    return pass_.records(label)[SOLE]


__all__ = [
    "SOLE",
    "MeasurementPass",
    "MeasurementScore",
    "MeasurementTask",
    "combine_measurements",
    "drawing_from",
    "measurement_tasks",
    "play_measurement",
    "queued_objectives",
    "queued_population",
    "queued_repeats",
    "start_measurement",
]
