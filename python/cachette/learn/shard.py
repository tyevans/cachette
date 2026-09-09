"""One generation, played one episode at a time, combined in candidate order.

A generation plays a population of candidates on a set of seeds. Every one of
those episodes is independent, so the whole set could use the whole machine.
**One interpreter cannot use it.** Between two decisions, one process stacks
the observations, builds the action masks and runs the policy matrix products,
and every engine worker of that process waits. That section is per process, so
more processes fill the machine and more threads inside one process do not.

A measurement on the target platform states the size of it.[^1]

# One task is one episode

**A task is one strategy, one candidate and one seed.** A generation of
twenty-four candidates over six seeds is one hundred and forty-four tasks, and
two strategies of that shape are two hundred and eighty-eight. The pool holds
a queue, so any worker takes any episode of any strategy, and a worker that
finished a short episode takes the next one rather than waiting.

An earlier version of this module split the candidates into one contiguous
block for each worker process. That shape cost three things, and all three
were measured on a paid run. The block count could not pass the pair count,
which is half the population, so a run asking for sixteen processes at a
population of twenty-four received twelve and four processes received no work
at all. A block ended when its slowest episode ended, and an episode of the
audited world ends anywhere between tick 1,271 and tick 3,311, so every block
waited on its longest game while its cores idled. Each process then ran many
engine threads, which serialise on the one interpreter that chooses their
actions.

**Parallelism is now bounded by the episode count and not by the pair
count.** A seated group is the one task that holds more than one candidate,
because the candidates of a group share the worlds they play.

# The dependency is per strategy, and there is no barrier across strategies

A generation of one strategy needs every episode of the generation before it,
and it needs no episode of another strategy. So one strategy queues its next
generation while another is still finishing the one it is on, and the queue
stays full while any strategy holds work. **This module states no barrier
across the strategies**, and one that waited for every strategy of a run would
give the whole gain back.

# A worker holds the engine to one thread

More threads for one world step is slower at every extent this project trains
on. One thread reaches 439 ticks a second at extent 128 and sixteen threads
reach 215.[^2] So a worker gives up no engine parallelism by running one
thread, and a pool of single-threaded workers is the shape that fills a
machine.

# A worker rebuilds its candidate, and never receives it

The trainer builds one candidate as the centre plus or minus one perturbation,
and the perturbation comes from a generator keyed on the run seed and the
generation number.[^3] A worker that holds the centre, the generation number
and the candidate index it owns therefore builds exactly the candidate the
trainer would have built. It sends back one score, which is a few numbers.

# The combination is ordered by the candidate and by the seed

**Nothing here reads which worker answered first.** Each result reports the
strategy it played, the candidate index it started at and the seed position it
played, and the combination sorts on that key.[^4] An episode is a pure
function of the policy and the seed, so the completion order cannot reach a
score, and that is the reason a queue is admissible here at all.

The combination also checks that the results cover every candidate of one
strategy on every seed exactly once, so a generation that lost an episode
fails rather than training on a subset.

# A worker process runs one matrix thread

Each matrix library starts one thread for each core unless an environment
variable says otherwise. Five processes on a 64 core machine then start 320
threads, and those threads fight for the cores the engine needs. The pool sets
the four variables before it starts a process, so a worker inherits them.

# References

[^1]: Target platform costs, the trainer process measurement.
``docs/reference/graviton-costs.md``
[^2]: Findings register, FND-714. ``docs/FINDINGS.md``
[^3]: ADR-0194, a generation is scored one episode at a time, and combined in
candidate order, decision D2.
``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
[^4]: ADR-0001, one binary gives one answer at any thread count, decision D2.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``
"""

from __future__ import annotations

import os
import time
from concurrent.futures import ProcessPoolExecutor, as_completed
from dataclasses import dataclass, replace
from multiprocessing import get_context
from typing import TYPE_CHECKING

import numpy as np

from .env import Env
from .record import EpisodeRecord
from .rollout import HEARTBEAT_SECONDS, Generation, score_generation
from .search import generation_noise, pair_candidates, shell_policy

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence
    from concurrent.futures import Future
    from types import TracebackType

    from .config import TrainConfig
    from .env import EnvConfig
    from .policy import FeatureNormalizer
    from .reward import Scoring

# The variables that hold a matrix library to one thread. Each library reads
# its own, and a library that reads none starts one thread for each core.
MATRIX_THREAD_VARS = (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
)

# The test-only switch that fails one episode. The value names the candidate
# index of the episode that must raise. **A worker runs in another process, so
# a test reaches it by no other door.** The switch proves that a lost episode
# fails the generation loudly, which is what a silent partial generation would
# hide.
SHARD_FAULT = "CACHETTE_SHARD_FAULT"

# How many engine threads one worker gives the world of one episode. One
# episode holds one world, and more threads for one step is slower at every
# extent this project trains on, so a worker that asked for more would pay for
# the barrier and gain nothing.
THREADS_FOR_ONE_EPISODE = 1


@dataclass(frozen=True)
class EpisodeTask:
    """What one worker process needs in order to play one episode.

    The centre is the only array that crosses. The perturbations do not
    cross, because the worker draws them from the run seed and the generation
    number, and that draw reads nothing else.

    The strategy entry names what the episode plays for. One queue holds the
    episodes of every strategy of a run, so the name travels with the task
    and returns with the score.

    The candidate range names what the task plays. **An unseated task holds
    one candidate**, so it plays one world and one episode. A seated task
    holds the whole group of candidates that share its worlds, because a task
    that held part of a group would build a world with a learner seat that
    nobody filled.

    The seed entry is the seed of the world. The seed position is where that
    seed sits in the set the generation plays, and the combination reads it as
    the last part of its sort key.

    **The normalizer travels in the task.** It comes from a reference sample
    of played episodes, and a worker that derived one of its own would give
    its candidate a different feature transform from the candidates of every
    other task. Two candidates of one generation that read two transforms are
    not comparable, so the rank over them would say nothing.
    """

    env_config: EnvConfig
    scoring: Scoring
    train_config: TrainConfig
    generation: int
    seed: int
    seed_position: int
    centre: np.ndarray
    kind: str
    first_candidate: int
    last_candidate: int
    strategy: str = ""
    normalizer: FeatureNormalizer | None = None


@dataclass(frozen=True)
class EpisodeScore:
    """What one task scored, and where it sits in the generation.

    The strategy, the first candidate entry and the seed position are the
    stable key the combination sorts on. The length of the score arrays says
    how many candidates the task held, so the combination can say whether the
    tasks cover the generation.

    The wins and the games are counts and not a share. A share cannot be
    combined without the games behind it, and a run whose tasks held
    different candidate counts would report a mean that the single-process
    run does not report. The chosen and the refused entries are counts for
    the same reason.

    The episodes entry holds one record for each episode the task played, in
    candidate order. The combination concatenates them in key order, which is
    candidate order and then seed order.
    """

    first_candidate: int
    seed_position: int
    ranked: np.ndarray
    absolute: np.ndarray
    wins: int
    games: int
    ticks: int
    chosen: int = 0
    refused: int = 0
    episodes: tuple[EpisodeRecord, ...] = ()
    strategy: str = ""

    @property
    def key(self) -> tuple[str, int, int]:
        """The stable key the combination orders this result by."""
        return (self.strategy, self.first_candidate, self.seed_position)


def candidate_stride(train_config: TrainConfig) -> int:
    """Return how many candidates one task holds.

    **An unseated generation puts one candidate in one world, so one task is
    one episode.** That is the whole of this change: the task count is then
    the episode count, and no worker waits on the slowest episode of a block
    it was given.

    A seated generation puts one candidate in each learner seat of a world,
    and it needs both halves of every antithetic pair it seats. A group is
    therefore two candidates for each learner seat, and a task cannot hold
    less than a group.
    """
    seats = len(train_config.learner_seats)
    return 1 if seats == 0 else 2 * seats


def episode_tasks(
    env_config: EnvConfig,
    scoring: Scoring,
    train_config: TrainConfig,
    seeds: Sequence[int],
    generation: int,
    centre: np.ndarray,
    kind: str = "linear",
    normalizer: FeatureNormalizer | None = None,
    strategy: str = "",
) -> list[EpisodeTask]:
    """Return one task for each episode of one generation of one strategy.

    **The task count reads the population and the seeds, and never the worker
    count.** A pool of any size plays the same tasks, so the number of workers
    reaches how long a generation takes and reaches nothing it answers.

    The tasks come back in candidate order and then in seed order. That order
    decides which episode a free worker starts first and it decides nothing
    else, because the combination sorts on the key of each result.
    """
    candidates = 2 * train_config.pairs
    stride = candidate_stride(train_config)
    return [
        EpisodeTask(
            env_config=env_config,
            scoring=scoring,
            train_config=train_config,
            generation=generation,
            seed=int(seed),
            seed_position=position,
            centre=centre,
            kind=kind,
            first_candidate=first,
            last_candidate=min(first + stride, candidates),
            strategy=strategy,
            normalizer=normalizer,
        )
        for first in range(0, candidates, stride)
        for position, seed in enumerate(seeds)
    ]


def play_episode(task: EpisodeTask) -> EpisodeScore:
    """Rebuild the candidate of one task, play its episode, and score it.

    This runs in a worker process. It builds the same policy shell the
    trainer holds, draws the same perturbations, and takes the candidate the
    task owns.

    **The engine runs one thread here.** The task holds one world in the
    unseated case, and more threads for one step is slower at every extent
    this project trains on, so the worker asks for one whatever the run asked
    for. The worker count of the configuration reaches the passes that no
    queue splits, which are the validation, the held-out and the baseline
    passes.
    """
    fault = os.environ.get(SHARD_FAULT)
    if fault is not None and int(fault) == task.first_candidate:
        message = (
            f"the fault switch failed the episode of candidate "
            f"{task.first_candidate} on seed {task.seed}"
        )
        raise RuntimeError(message)
    config = replace(task.train_config, workers=THREADS_FOR_ONE_EPISODE)
    probe = Env(task.env_config, task.scoring)
    shell = shell_policy(task.kind, probe, task.normalizer)
    noise = generation_noise(
        config.seed, task.generation, config.pairs, shell, task.centre
    )
    # The candidate index carries the sign: candidate ``2 * pair`` is the plus
    # half of its pair and ``2 * pair + 1`` is the minus half. A task may
    # start at either half, so it builds the pairs that cover its range and
    # takes the candidates inside that range.
    first_pair = task.first_candidate // 2
    last_pair = -(-task.last_candidate // 2)
    built = pair_candidates(
        shell, task.centre, noise, config.sigma, first_pair, last_pair
    )
    held = built[
        task.first_candidate - 2 * first_pair : task.last_candidate - 2 * first_pair
    ]
    played = score_generation(task.env_config, task.scoring, held, [task.seed], config)
    # **A record names the candidate of the generation and not the candidate
    # of the task.** The runner numbers the policies it received from zero, so
    # every task would report candidate zero and a reader of the episode rows
    # could not tell one candidate from another.
    numbered = tuple(
        replace(row, candidate=task.first_candidate + row.candidate)
        for row in played.episodes
    )
    # A candidate plays one world for each seed, whether it plays that world
    # alone or beside another learner seat. The games of a task are therefore
    # its candidates times its one seed, and the win count follows from the
    # share the runner reported.
    games = len(held)
    return EpisodeScore(
        first_candidate=task.first_candidate,
        seed_position=task.seed_position,
        ranked=played.ranked,
        absolute=played.absolute,
        wins=round(played.won * games),
        games=games,
        ticks=played.ticks,
        chosen=played.chosen,
        refused=played.refused,
        episodes=numbered,
        strategy=task.strategy,
    )


def combine_episodes(
    scores: Sequence[EpisodeScore],
    candidates: int,
    seeds: int,
    strategy: str = "",
) -> Generation:
    """Put the episode scores in candidate order, and give back one generation.

    **The order comes from the strategy, the candidate index and the seed
    position, and from nothing else.** The workers may answer in any order,
    and the answer must not move. An episode is a pure function of the policy
    and the seed, so the order a worker happens to finish in carries no
    information about the score it carries.

    One combination answers for one generation of one strategy. A result of
    another strategy fails here rather than joining a population it was not
    drawn for.

    The score of a candidate is the mean of its episodes over the seeds. This
    builds the same array of episode returns that a single process builds, in
    the same positions, and takes the mean of it in the same call, so the
    answer is the same number and not a number near it.

    Raises ``ValueError`` when the results do not cover every candidate on
    every seed exactly once. A generation that lost an episode would
    otherwise train on a subset, and nothing would say so.
    """
    if candidates < 1 or seeds < 1:
        message = (
            f"a generation holds at least one candidate on one seed, and this "
            f"names {candidates} candidates on {seeds} seeds"
        )
        raise ValueError(message)
    other = sorted({score.strategy for score in scores} - {strategy})
    if other:
        message = (
            f"a generation of {strategy!r} holds the episodes of {other} as "
            "well, and one combination answers for one strategy"
        )
        raise ValueError(message)
    ordered = sorted(scores, key=lambda score: score.key)
    ranked = np.zeros((candidates, seeds))
    absolute = np.zeros((candidates, seeds))
    filled = np.zeros((candidates, seeds), dtype=bool)
    for score in ordered:
        _place(score, ranked, absolute, filled)
    missing = np.argwhere(~filled)
    if missing.size:
        candidate, seed = (int(value) for value in missing[0])
        message = (
            f"the episodes do not cover the generation: candidate {candidate} "
            f"played no episode on seed position {seed}, and "
            f"{len(missing)} of {candidates * seeds} episodes are missing"
        )
        raise ValueError(message)
    games = sum(score.games for score in ordered)
    return Generation(
        ranked=ranked.mean(axis=1),
        absolute=absolute.mean(axis=1),
        won=sum(score.wins for score in ordered) / games if games else 0.0,
        ticks=sum(score.ticks for score in ordered),
        chosen=sum(score.chosen for score in ordered),
        refused=sum(score.refused for score in ordered),
        episodes=tuple(episode for score in ordered for episode in score.episodes),
    )


def _place(
    score: EpisodeScore,
    ranked: np.ndarray,
    absolute: np.ndarray,
    filled: np.ndarray,
) -> None:
    """Write one result into the arrays of the generation, at its own key.

    A result that falls outside the generation, or onto a cell another result
    already wrote, fails here. Both would leave a cell that no episode filled,
    and a mean over a zero that nobody wrote reads as a score.
    """
    first = score.first_candidate
    last = first + len(score.ranked)
    seed = score.seed_position
    if first < 0 or last > ranked.shape[0] or not 0 <= seed < ranked.shape[1]:
        message = (
            f"an episode of candidates {first} to {last} on seed position "
            f"{seed} falls outside a generation of {ranked.shape[0]} "
            f"candidates on {ranked.shape[1]} seeds"
        )
        raise ValueError(message)
    if filled[first:last, seed].any():
        message = (
            f"two episodes scored candidates {first} to {last} on seed "
            f"position {seed}, so one of them would be lost"
        )
        raise ValueError(message)
    ranked[first:last, seed] = score.ranked
    absolute[first:last, seed] = score.absolute
    filled[first:last, seed] = True


class ShardPool:
    """The worker processes of one run, and the threads they are held to.

    The pool holds the processes for the whole run, so a generation pays no
    start cost. It sets the matrix thread variables while it is open, and it
    puts back what was there when it closes. A process inherits the
    environment of the process that started it, so a worker reads the value
    the pool set.

    **The pool is a queue and never an assignment.** Every task of a
    generation is submitted at once, and a worker takes the next one when it
    is free. So a worker that drew a short episode starts another instead of
    waiting for the longest episode of a block that somebody gave it.

    **One pool serves every strategy of a run.** Each strategy submits its own
    generation and waits for its own results, so the queue holds the episodes
    of any strategy that has work. A strategy that finishes first queues its
    next generation while another is still finishing.

    **A worker that dies fails the generation.** The pool waits on each
    result in turn and lets the failure through. It catches nothing.
    """

    def __init__(self, processes: int) -> None:
        """Declare a pool of one or more worker processes."""
        if processes < 1:
            message = f"a pool holds at least one process, and this names {processes}"
            raise ValueError(message)
        self._processes = processes
        self._executor: ProcessPoolExecutor | None = None
        self._restore: dict[str, str | None] = {}

    @property
    def processes(self) -> int:
        """How many worker processes the pool holds."""
        return self._processes

    def __enter__(self) -> ShardPool:
        """Pin the matrix libraries, then start the worker processes."""
        self._restore = {name: os.environ.get(name) for name in MATRIX_THREAD_VARS}
        for name in MATRIX_THREAD_VARS:
            os.environ[name] = "1"
        # A spawned process starts a fresh interpreter and imports the
        # package again, so it reads the environment as it stands here. That
        # is why the variables are set before this line: a matrix library
        # reads them as it loads, and an initialiser would run too late. A
        # forked process would carry the engine's own threads into the child.
        self._executor = ProcessPoolExecutor(
            max_workers=self._processes, mp_context=get_context("spawn")
        )
        return self

    def __exit__(
        self,
        kind: type[BaseException] | None,
        value: BaseException | None,
        trace: TracebackType | None,
    ) -> None:
        """Stop the worker processes, then put the environment back."""
        del kind, value, trace
        if self._executor is not None:
            self._executor.shutdown(wait=True)
            self._executor = None
        for name, held in self._restore.items():
            if held is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = held
        self._restore = {}

    def play(self, tasks: Sequence[EpisodeTask], label: str = "") -> list[EpisodeScore]:
        """Submit every task at once, and give back what the workers scored.

        The results come back in the order the tasks were submitted, whatever
        order the workers answered in. The combination sorts them on their own
        key as well, so neither this list nor the completion order that filled
        it can reach a score.

        Two callers may hold one pool at once, because a submission takes the
        lock of the executor and a caller waits only on futures of its own.
        """
        if self._executor is None:
            message = "the pool is not open. Use it as a context manager."
            raise RuntimeError(message)
        futures = [self._executor.submit(play_episode, task) for task in tasks]
        if label:
            _report_progress(futures, label)
        return [future.result() for future in futures]


def _report_progress(futures: Sequence[Future[EpisodeScore]], label: str) -> None:
    """Print what a generation has finished, while it runs.

    **A generation says it is working while it works.** A generation of the
    paid size takes several minutes, and without a line inside it a slow
    generation and a stopped one look the same from outside.

    The line holds the marker and the fields that the progress reader takes
    by name. The live count is the episodes that have not finished, and the
    rate is the simulated ticks a second since the last line. **A rate taken
    over the whole elapsed time falls as the queue empties**, and that reads
    as a machine slowing down when it is only running fewer episodes.

    **This reads the completion order, and only the printing reads it.** The
    count of finished episodes reaches a line of the log and reaches no
    score, and the clock decides when to print. A failed episode is counted
    here and raised by the caller that reads the results.
    """
    total = len(futures)
    started = time.perf_counter()
    spoke = started
    finished = 0
    ticks = 0
    told = 0
    for future in as_completed(futures):
        finished += 1
        if future.exception() is None:
            ticks += future.result().ticks
        now = time.perf_counter()
        if now - spoke < HEARTBEAT_SECONDS and finished < total:
            continue
        window = now - spoke
        rate = (ticks - told) / window if window > 0.0 else 0.0
        spoke = now
        told = ticks
        print(
            f"  {label} working  episodes {finished:5d} "
            f"live {total - finished:4d}/{total:<4d} "
            f"ticks {ticks:9d} rate {rate:8.1f} t/s "
            f"[{now - started:.0f}s]",
            flush=True,
        )


def run_sharded_generation(
    env_config: EnvConfig,
    scoring: Scoring,
    train_config: TrainConfig,
    seeds: list[int],
    generation: int,
    centre: np.ndarray,
    pool: ShardPool,
    kind: str = "linear",
    label: str = "",
    normalizer: FeatureNormalizer | None = None,
    strategy: str = "",
) -> Generation:
    """Score one generation across the pool, and combine it in candidate order.

    The answer holds the numbers the single-process runner gives for the same
    centre, the same generation, the same seeds and the same normalizer. The
    worker count changes how the work is spread, and it changes nothing else.
    """
    tasks = episode_tasks(
        env_config,
        scoring,
        train_config,
        seeds,
        generation,
        centre,
        kind,
        normalizer,
        strategy,
    )
    return combine_episodes(
        pool.play(tasks, label),
        candidates=2 * train_config.pairs,
        seeds=len(seeds),
        strategy=strategy,
    )


__all__ = [
    "MATRIX_THREAD_VARS",
    "SHARD_FAULT",
    "THREADS_FOR_ONE_EPISODE",
    "EpisodeScore",
    "EpisodeTask",
    "ShardPool",
    "candidate_stride",
    "combine_episodes",
    "episode_tasks",
    "play_episode",
    "run_sharded_generation",
]
