"""One generation, split across worker processes, combined in candidate order.

A generation plays a population of candidates on a set of seeds. Every one of
those episodes is independent, so the whole set could use the whole machine.
**One interpreter cannot use it.** Between two decisions, one process stacks
the observations, builds the action masks and runs the policy matrix products,
and every engine worker of that process waits. That section is per process, so
more processes fill the machine and more workers inside one process do not.

A measurement on the target platform states the size of it.[^1]

# A worker rebuilds its candidates, and never receives them

The trainer builds one candidate as the centre plus or minus one perturbation,
and the perturbation comes from a generator keyed on the run seed and the
generation number.[^2] A worker that holds the centre, the generation number
and the pair range it owns therefore builds exactly the candidates the trainer
would have built. It sends back the scores, which are a few hundred numbers.

# The combination is ordered by the candidate index

**Nothing here reads which shard answered first.** Each shard reports the
candidate index it started at, and the combination sorts on that index.[^3]
The combination also checks that the shards cover every candidate, so a
generation that lost a shard fails rather than training on a subset.

# A worker process runs one matrix thread

Each matrix library starts one thread for each core unless an environment
variable says otherwise. Five processes on a 64 core machine then start 320
threads, and those threads fight for the cores the engine needs. The pool sets
the four variables before it starts a process, so a worker inherits them.

# References

[^1]: Target platform costs, the trainer process measurement.
``docs/reference/graviton-costs.md``
[^2]: ADR-0194, a generation is scored in shards and combined in candidate
order, decision D2. ``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D2.
``docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md``
"""

from __future__ import annotations

import os
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass
from multiprocessing import get_context
from typing import TYPE_CHECKING

import numpy as np

from .env import Env
from .record import EpisodeRecord
from .rollout import Generation, score_generation
from .search import generation_noise, pair_candidates, shell_policy

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence
    from types import TracebackType

    from .config import TrainConfig
    from .env import EnvConfig
    from .reward import Weighting

# The variables that hold a matrix library to one thread. Each library reads
# its own, and a library that reads none starts one thread for each core.
MATRIX_THREAD_VARS = (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
)

# The test-only switch that makes one shard fail. The value names the first
# pair index of the shard that must raise. **A worker runs in another process,
# so a test reaches it by no other door.** The switch proves that a lost shard
# fails the generation loudly, which is what a silent partial generation would
# hide.
SHARD_FAULT = "CACHETTE_SHARD_FAULT"


@dataclass(frozen=True)
class ShardTask:
    """What one worker process needs in order to score its own candidates.

    The centre is the only array that crosses. The perturbations do not
    cross, because the worker draws them from the run seed and the generation
    number, and that draw reads nothing else.

    The pair range names the candidates the shard owns. Candidate
    ``2 * pair`` is the plus half of a pair and ``2 * pair + 1`` is the minus
    half, which is the order the trainer builds them in.
    """

    env_config: EnvConfig
    weighting: Weighting
    train_config: TrainConfig
    generation: int
    seeds: list[int]
    centre: np.ndarray
    kind: str
    hidden: int
    first_pair: int
    last_pair: int
    label: str = ""


@dataclass(frozen=True)
class ShardScore:
    """What one worker process scored, and where it sits in the population.

    The first candidate entry is the stable key the combination sorts on. The
    count entry says how many candidates the shard held, so the combination
    can say whether the shards cover the population.

    The wins and the games are counts and not a share. A share cannot be
    combined without the games behind it, and a run whose shards hold
    different candidate counts would report a mean that the single-process
    run does not report. The chosen and the refused entries are counts for
    the same reason.

    The episodes entry holds one record for each episode the shard played, in
    candidate order. The combination concatenates them in shard order, which
    is candidate order.
    """

    first_candidate: int
    candidates: int
    ranked: np.ndarray
    absolute: np.ndarray
    wins: int
    games: int
    ticks: int
    chosen: int = 0
    refused: int = 0
    episodes: tuple[EpisodeRecord, ...] = ()


def shard_ranges(pairs: int, processes: int, width: int) -> list[tuple[int, int]]:
    """Split the pairs of one generation into contiguous blocks.

    The width entry is how many pairs share one group of worlds. A
    single-seat run puts one pair in one world, so its width is one. A league
    run of two learner seats groups two pairs into one pair of worlds, and a
    shard boundary must fall between two groups. A boundary inside a group
    would leave a world with a seat that no shard filled.

    The blocks are as even as the group count allows. A run with more
    processes than groups uses one process for each group.
    """
    if pairs < 1:
        message = "a generation holds at least one pair"
        raise ValueError(message)
    if width < 1:
        message = "a group holds at least one pair"
        raise ValueError(message)
    if processes < 1:
        message = "a run holds at least one process"
        raise ValueError(message)
    groups = -(-pairs // width)
    blocks = min(processes, groups)
    ranges: list[tuple[int, int]] = []
    for index in range(blocks):
        first = index * groups // blocks
        last = (index + 1) * groups // blocks
        ranges.append((first * width, min(last * width, pairs)))
    return ranges


def play_shard(task: ShardTask) -> ShardScore:
    """Rebuild the candidates of one shard, play them, and score them.

    This runs in a worker process. It builds the same policy shell the
    trainer holds, draws the same perturbations, and takes the slice of them
    that the shard owns.
    """
    fault = os.environ.get(SHARD_FAULT)
    if fault is not None and int(fault) == task.first_pair:
        message = f"the fault switch failed the shard at pair {task.first_pair}"
        raise RuntimeError(message)
    config = task.train_config
    probe = Env(task.env_config, task.weighting)
    shell = shell_policy(task.kind, probe, task.hidden)
    noise = generation_noise(
        config.seed, task.generation, config.pairs, task.centre.size
    )
    candidates = pair_candidates(
        shell, task.centre, noise, config.sigma, task.first_pair, task.last_pair
    )
    played = score_generation(
        task.env_config, task.weighting, candidates, task.seeds, config, task.label
    )
    # A candidate plays one world for each seed, whether it plays that world
    # alone or beside another learner seat. The games of a shard are
    # therefore its candidates times its seeds, and the win count follows
    # from the share the runner reported.
    games = len(candidates) * len(task.seeds)
    return ShardScore(
        first_candidate=2 * task.first_pair,
        candidates=len(candidates),
        ranked=played.ranked,
        absolute=played.absolute,
        wins=round(played.won * games),
        games=games,
        ticks=played.ticks,
        chosen=played.chosen,
        refused=played.refused,
        episodes=played.episodes,
    )


def combine_shards(scores: Sequence[ShardScore], candidates: int) -> Generation:
    """Put the shard scores in candidate order, and give back one generation.

    **The order comes from the candidate index and from nothing else.** The
    shards may answer in any order, and the answer must not move.

    Raises ``ValueError`` when the shards do not cover every candidate
    exactly once. A generation that lost a shard would otherwise train on a
    subset, and nothing would say so.
    """
    ordered = sorted(scores, key=lambda score: score.first_candidate)
    reached = 0
    for score in ordered:
        if score.first_candidate != reached:
            message = (
                f"the shards do not cover the population: candidate {reached} "
                f"is missing, and the next shard starts at {score.first_candidate}"
            )
            raise ValueError(message)
        reached += score.candidates
    if reached != candidates:
        message = (
            f"the shards cover {reached} candidates and the generation holds "
            f"{candidates}"
        )
        raise ValueError(message)
    games = sum(score.games for score in ordered)
    return Generation(
        ranked=np.concatenate([score.ranked for score in ordered]),
        absolute=np.concatenate([score.absolute for score in ordered]),
        won=sum(score.wins for score in ordered) / games if games else 0.0,
        ticks=sum(score.ticks for score in ordered),
        chosen=sum(score.chosen for score in ordered),
        refused=sum(score.refused for score in ordered),
        episodes=tuple(episode for score in ordered for episode in score.episodes),
    )


class ShardPool:
    """The worker processes of one run, and the threads they are held to.

    The pool holds the processes for the whole run, so a generation pays no
    start cost. It sets the matrix thread variables while it is open, and it
    puts back what was there when it closes. A process inherits the
    environment of the process that started it, so a worker reads the value
    the pool set.

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

    def play(self, tasks: Sequence[ShardTask]) -> list[ShardScore]:
        """Send one task to each worker, and give back what they scored."""
        if self._executor is None:
            message = "the pool is not open. Use it as a context manager."
            raise RuntimeError(message)
        futures = [self._executor.submit(play_shard, task) for task in tasks]
        return [future.result() for future in futures]


def run_sharded_generation(
    env_config: EnvConfig,
    weighting: Weighting,
    train_config: TrainConfig,
    seeds: list[int],
    generation: int,
    centre: np.ndarray,
    pool: ShardPool,
    kind: str = "linear",
    hidden: int = 0,
    label: str = "",
) -> Generation:
    """Score one generation across the pool, and combine it in candidate order.

    The answer holds the numbers the single-process runner gives for the same
    centre, the same generation and the same seeds. The shard count changes
    how the work is spread, and it changes nothing else.
    """
    pairs = train_config.pairs
    width = max(1, len(train_config.learner_seats))
    ranges = shard_ranges(pairs, pool.processes, width)
    tasks = [
        ShardTask(
            env_config=env_config,
            weighting=weighting,
            train_config=train_config,
            generation=generation,
            seeds=list(seeds),
            centre=centre,
            kind=kind,
            hidden=hidden,
            first_pair=first,
            last_pair=last,
            label=f"{label} shard {index + 1}/{len(ranges)}" if label else "",
        )
        for index, (first, last) in enumerate(ranges)
    ]
    return combine_shards(pool.play(tasks), candidates=2 * pairs)


__all__ = [
    "MATRIX_THREAD_VARS",
    "SHARD_FAULT",
    "ShardPool",
    "ShardScore",
    "ShardTask",
    "combine_shards",
    "play_shard",
    "run_sharded_generation",
    "shard_ranges",
]
