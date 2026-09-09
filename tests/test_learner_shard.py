"""A generation queued one episode at a time gives what one process gives.

The trainer scores a population of candidates on a set of seeds. Those
episodes are independent, so a run plays them in a pool of worker processes.
**One task is one episode**, so any worker takes any episode, and a worker
that finished a short game takes the next one rather than waiting for the
longest game of a block that somebody gave it.

**The split must not reach the answer.** A run at five workers and a run at
one worker start from the same seed, so they must end at the same weights,
position for position.

The property has three halves, and all three are tested here.

**The combination is ordered by the strategy, the candidate index and the
seed position.** A result reports all three, and the combination sorts on
them. Nothing reads which worker answered first.

**A worker rebuilds its candidate.** It receives the centre, the generation
number and one candidate index, and it draws the perturbations from the run
seed and the generation number. No candidate crosses to a worker, so a worker
that drew from another stream would score a different population and the
weights would move.

**One queue holds every strategy of a run.** Two strategies submit into one
pool at the same time, so their episodes interleave. Each strategy combines
its own results, and the interleaving must not move either answer.

**The comparison must be able to fail.** One test perturbs the combination
order behind the trainer and requires the weights to move. A second test
fails one episode on purpose and requires the generation to raise, because a
generation that lost an episode would otherwise train on a subset and say
nothing.

# There is no assertion on time here

A queue is worth having because it keeps the cores busy, and this file
asserts nothing about how long anything takes. A timing assertion is flaky on
a loaded machine.[^1] The tests assert the mechanism instead: the task count
is the episode count, no task holds a range of candidates, and a generation of
very uneven episodes is still one task for each episode.

# References

[^1]: Testing Rules, do not assert on time. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import io
import json
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from dataclasses import replace
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette.learn import Weighting
from cachette.learn.config import TrainConfig
from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.journal import strategy_logs
from cachette.learn.policy import load_policy
from cachette.learn.rollout import score_generation
from cachette.learn.search import generation_noise, pair_candidates, shell_policy
from cachette.learn.shard import (
    SHARD_FAULT,
    EpisodeScore,
    ShardPool,
    candidate_stride,
    combine_episodes,
    episode_tasks,
    run_sharded_generation,
)
from cachette.learn.train import train

if TYPE_CHECKING:
    from collections.abc import Sequence

    from cachette.learn.rollout import Generation

# The repository root, so a test that drives the command line runs where a
# person runs it.
ROOT = Path(__file__).resolve().parents[1]

# A world small enough to play many times in a test, and long enough that a
# candidate reaches a different score from its mirror.
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

# A world whose episodes end at very different ticks. **A fixture of equal
# episodes cannot show the defect this change fixes**, because a block of
# equal episodes wastes nothing. Two factions on a small map resolve most
# games early and take one to the tick limit, and the test asserts that
# spread rather than trusting it.
UNEVEN_WORLD = EnvConfig(
    width=14,
    height=14,
    faction_count=2,
    seat=0,
    tick_limit=3000,
    horizon=300,
    decision_interval=10,
)

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


def a_config(
    population: int = 6,
    seeds_per_generation: int = 2,
    pool: int = 1,
    seed: int = 3,
    learner_seats: tuple[int, ...] = (),
) -> TrainConfig:
    """Build a training configuration of the size these tests play."""
    return TrainConfig(
        generations=1,
        population=population,
        seeds_per_generation=seeds_per_generation,
        workers=1,
        pool=pool,
        seed=seed,
        learner_seats=learner_seats,
    )


def a_centre(world: EnvConfig) -> np.ndarray:
    """Return the untrained centre of the linear policy of one world."""
    return np.asarray(shell_policy("linear", Env(world, WEIGHTING)).flat())


def weights_of_a_run(out: Path, pool: int) -> np.ndarray:
    """Train one policy at one worker count, and return the centre it reached."""
    seed_pool = viable_seeds(WORLD, 4, 900)
    result = train(
        f"pool-{pool}",
        WORLD,
        WEIGHTING,
        TrainConfig(
            generations=2,
            population=8,
            seeds_per_generation=2,
            workers=2,
            pool=pool,
            seed=3,
        ),
        out,
        seed_pool,
        kind="linear",
    )
    stored, _ = load_policy(out / f"{result['name']}-latest.npz")
    return np.asarray(stored.flat())


def score(candidate: int, seed: int, value: float, strategy: str = "") -> EpisodeScore:
    """Build one episode result over a made-up score."""
    array = np.asarray([value], dtype=float)
    return EpisodeScore(
        first_candidate=candidate,
        seed_position=seed,
        ranked=array,
        absolute=array,
        wins=1,
        games=1,
        ticks=1,
        strategy=strategy,
    )


def a_generation_of(values: Sequence[Sequence[float]]) -> list[EpisodeScore]:
    """Build one result for each candidate and seed of a made-up generation."""
    return [
        score(candidate, seed, value)
        for candidate, row in enumerate(values)
        for seed, value in enumerate(row)
    ]


# --------------------------------------------------------- the task count


def test_a_generation_holds_one_task_for_each_episode() -> None:
    """A population of N over S seeds is N times S tasks, and never a block.

    This is the defect. The block scheme cut the population into one block
    for each worker process, and it could make no more blocks than the
    generation held pairs. A run of sixteen processes at a population of
    twenty-four therefore received twelve blocks, and four processes received
    nothing.
    """
    config = a_config(population=24, seeds_per_generation=6)
    tasks = episode_tasks(
        WORLD, WEIGHTING, config, list(range(6)), 0, a_centre(WORLD), "linear"
    )

    assert len(tasks) == 24 * 6
    assert {task.last_candidate - task.first_candidate for task in tasks} == {1}


def test_the_task_count_does_not_depend_on_the_worker_count() -> None:
    """The worker count decides how long a generation takes and nothing else.

    The block scheme took the process count as an argument, and the tasks it
    built therefore moved with it. This builder takes no worker count at all,
    so the same generation is the same tasks on any machine.
    """
    seeds = [11, 12, 13]
    counts = {
        pool: len(
            episode_tasks(
                WORLD,
                WEIGHTING,
                a_config(population=8, pool=pool),
                seeds,
                0,
                a_centre(WORLD),
                "linear",
            )
        )
        for pool in (1, 2, 7, 64)
    }

    assert set(counts.values()) == {8 * 3}


def test_no_task_is_bound_to_a_range_of_candidates() -> None:
    """Any worker may take any episode, so no task names a range.

    A task that held a contiguous range of candidates would bind whichever
    worker took it to that range. Every task of an unseated generation holds
    one candidate on one seed, and the whole set covers the generation
    exactly once.
    """
    config = a_config(population=6, seeds_per_generation=3)
    tasks = episode_tasks(
        WORLD, WEIGHTING, config, [1, 2, 3], 0, a_centre(WORLD), "linear"
    )

    assert candidate_stride(config) == 1
    keys = [(task.first_candidate, task.seed_position) for task in tasks]
    assert sorted(keys) == [(c, s) for c in range(6) for s in range(3)]
    assert len(set(keys)) == len(keys)


def test_a_seated_group_stays_inside_one_task() -> None:
    """A task that held part of a group would leave a learner seat empty.

    A league run of two learner seats seats two candidates in one world, and
    it needs both halves of every antithetic pair it seats. The group is
    therefore four candidates, and that is the one task that holds more than
    one episode.
    """
    seated = a_config(population=8, learner_seats=(0, 1))
    tasks = episode_tasks(
        WORLD, WEIGHTING, seated, [1, 2], 0, a_centre(WORLD), "linear"
    )

    assert candidate_stride(seated) == 4
    assert [task.first_candidate for task in tasks] == [0, 0, 4, 4]
    for task in tasks:
        assert task.first_candidate % 4 == 0
        assert task.last_candidate - task.first_candidate == 4


# ---------------------------------------------------------- the combination


def test_the_combination_orders_by_the_candidate_and_the_seed() -> None:
    """The workers may answer in any order, and the answer must not move."""
    results = a_generation_of([[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]])
    combined = combine_episodes(list(reversed(results)), candidates=3, seeds=2)

    assert np.array_equal(combined.ranked, np.array([1.5, 3.5, 5.5]))
    assert np.array_equal(combined.absolute, np.array([1.5, 3.5, 5.5]))


def test_the_order_assertion_can_fail() -> None:
    """A combination that took the arrival order would give another answer.

    The test above compares against one fixed array. This one takes the same
    results, gives each of them the key of another, and requires the answer
    to move. So the fixed array is sensitive to the key and is not satisfied
    by any arrangement of the same numbers.
    """
    results = a_generation_of([[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]])
    keys = [(held.first_candidate, held.seed_position) for held in results]
    shuffled = [
        replace(held, first_candidate=candidate, seed_position=seed)
        for held, (candidate, seed) in zip(reversed(results), keys, strict=True)
    ]

    assert not np.array_equal(
        combine_episodes(shuffled, candidates=3, seeds=2).ranked,
        combine_episodes(results, candidates=3, seeds=2).ranked,
    )


def test_a_missing_episode_fails_the_generation() -> None:
    """A generation that lost an episode would otherwise train on a subset."""
    results = a_generation_of([[1.0, 2.0], [3.0, 4.0]])
    with pytest.raises(ValueError, match="do not cover the generation"):
        combine_episodes(results[:-1], candidates=2, seeds=2)


def test_two_episodes_of_one_candidate_and_seed_fail_the_generation() -> None:
    """A repeated key would lose one of the two results in silence."""
    results = a_generation_of([[1.0, 2.0], [3.0, 4.0]])
    doubled = [*results, score(0, 0, 9.0)]
    with pytest.raises(ValueError, match="two episodes scored"):
        combine_episodes(doubled, candidates=2, seeds=2)


def test_an_episode_of_another_strategy_fails_the_generation() -> None:
    """One queue holds every strategy, so a combination checks the name.

    Two strategies of one run submit into one pool. A result that reached the
    wrong combination would join a population it was not drawn for, and the
    coverage check alone would not see it.
    """
    mine = [score(0, 0, 1.0, "conquer"), score(1, 0, 2.0, "conquer")]
    theirs = [score(0, 0, 1.0, "conquer"), score(1, 0, 2.0, "renown")]

    combined = combine_episodes(mine, candidates=2, seeds=1, strategy="conquer")
    assert np.array_equal(combined.ranked, np.array([1.0, 2.0]))
    with pytest.raises(ValueError, match="holds the episodes of"):
        combine_episodes(theirs, candidates=2, seeds=1, strategy="conquer")


def test_a_candidate_scores_the_mean_of_its_episodes() -> None:
    """The score of a candidate is the mean over the seeds it played."""
    combined = combine_episodes(
        a_generation_of([[2.0, 4.0, 6.0], [1.0, 1.0, 1.0]]), candidates=2, seeds=3
    )

    assert np.array_equal(combined.ranked, np.array([4.0, 1.0]))


# ----------------------------------------------------------- the determinism


def played_at(pool_size: int, config: TrainConfig, seeds: list[int]) -> Generation:
    """Score one generation over a pool of this many worker processes."""
    with ShardPool(pool_size) as pool:
        return run_sharded_generation(
            WORLD,
            WEIGHTING,
            config,
            seeds,
            0,
            a_centre(WORLD),
            pool,
            "linear",
            strategy="conquer",
        )


def same_generation(one: Generation, other: Generation) -> bool:
    """Say whether two generations hold the same numbers, exactly."""
    return (
        np.array_equal(one.ranked, other.ranked)
        and np.array_equal(one.absolute, other.absolute)
        and one.won == other.won
        and one.ticks == other.ticks
        and one.chosen == other.chosen
        and one.refused == other.refused
        and [(row.candidate, row.seed) for row in one.episodes]
        == [(row.candidate, row.seed) for row in other.episodes]
    )


def test_a_generation_is_byte_identical_at_several_worker_counts() -> None:
    """This is the property the whole change rests on.

    One worker plays every episode in turn. Five workers play them in
    whatever order they finish. The scores must be equal number for number,
    because the combination sorts on a key that no worker decides.
    """
    config = a_config(population=6, seeds_per_generation=2)
    seeds = viable_seeds(WORLD, 2, 900)
    alone = played_at(1, config, seeds)

    assert np.count_nonzero(alone.ranked) > 0, "an all-zero score compares equal"
    assert len(set(alone.ranked.tolist())) > 1, "a flat population compares equal"
    assert same_generation(alone, played_at(2, config, seeds))
    assert same_generation(alone, played_at(5, config, seeds))


def test_a_queued_generation_is_the_generation_one_process_scores() -> None:
    """The queue changes how the work spreads and changes nothing it answers.

    The single-process runner is the reference. It plays the whole population
    in one batch of worlds, and the queue plays one episode in each task, so
    the two paths share no arithmetic beyond the policy and the engine.
    """
    config = a_config(population=6, seeds_per_generation=2)
    seeds = viable_seeds(WORLD, 2, 900)
    centre = a_centre(WORLD)
    shell = shell_policy("linear", Env(WORLD, WEIGHTING))
    noise = generation_noise(config.seed, 0, config.pairs, shell, centre)
    candidates = pair_candidates(shell, centre, noise, config.sigma, 0, config.pairs)
    alone = score_generation(WORLD, WEIGHTING, candidates, seeds, config)

    queued = played_at(3, config, seeds)

    assert np.array_equal(alone.ranked, queued.ranked)
    assert np.array_equal(alone.absolute, queued.absolute)
    assert alone.won == queued.won
    assert alone.chosen == queued.chosen
    assert alone.refused == queued.refused
    assert [(row.candidate, row.seed) for row in alone.episodes] == [
        (row.candidate, row.seed) for row in queued.episodes
    ]


def test_two_strategies_interleave_in_one_queue_and_neither_moves() -> None:
    """One queue holds every strategy, and the interleaving reaches nothing.

    Both strategies submit into one pool at the same time, so a worker takes
    an episode of one and then an episode of the other. Each strategy is then
    compared against the generation it scores when it holds the pool alone.
    """
    seeds = viable_seeds(WORLD, 2, 900)
    first = a_config(population=6, seeds_per_generation=2, seed=3)
    second = a_config(population=6, seeds_per_generation=2, seed=8)
    alone = {"first": played_at(2, first, seeds), "second": played_at(2, second, seeds)}

    centre = a_centre(WORLD)
    with ShardPool(3) as pool:

        def scored(name: str, config: TrainConfig) -> Generation:
            """Score one generation of one strategy over the shared pool."""
            return run_sharded_generation(
                WORLD,
                WEIGHTING,
                config,
                seeds,
                0,
                centre,
                pool,
                "linear",
                strategy=name,
            )

        with ThreadPoolExecutor(max_workers=2) as threads:
            running = {
                "first": threads.submit(scored, "first", first),
                "second": threads.submit(scored, "second", second),
            }
            together = {name: future.result() for name, future in running.items()}

    assert not np.array_equal(alone["first"].ranked, alone["second"].ranked), (
        "two strategies that score the same numbers cannot show a mix-up"
    )
    assert same_generation(alone["first"], together["first"])
    assert same_generation(alone["second"], together["second"])


def test_a_generation_of_uneven_episodes_is_one_task_for_each_episode() -> None:
    """A block waits on its longest game, and a queue does not.

    The fixture is the point of this test. Its episodes end between a few
    hundred ticks and the tick limit, so a block of them would hold a worker
    on its longest game while the rest of the block finished. The test
    asserts the spread, so a fixture that became uniform fails here rather
    than measuring nothing.
    """
    config = a_config(population=4, seeds_per_generation=4)
    seeds = viable_seeds(UNEVEN_WORLD, 4, 900)
    with ShardPool(2) as pool:
        played = run_sharded_generation(
            UNEVEN_WORLD,
            WEIGHTING,
            config,
            seeds,
            0,
            a_centre(UNEVEN_WORLD),
            pool,
            "linear",
            strategy="conquer",
        )

    ends = np.array([row.end_tick for row in played.episodes], dtype=float)
    assert len(played.episodes) == 4 * 4
    assert ends.max() / ends.min() > 2.0, (
        f"the fixture must hold uneven episodes, and these ran {ends.tolist()}"
    )


# ------------------------------------------------------------ through a run


def test_a_queued_run_reaches_the_weights_the_unqueued_run_reaches(
    tmp_path: Path,
) -> None:
    """The run at a pool of one scores every generation in this process.

    The runs at two and five worker processes score them in a queue, and each
    worker rebuilds the candidate of the episode it took. All three start
    from one seed, so all three must end at one centre, position for
    position.

    **Five workers is more than the generation holds pairs.** The block
    scheme could not use them, and this asserts that the answer holds when
    the worker count passes that bound.
    """
    alone = weights_of_a_run(tmp_path, pool=1)
    assert np.count_nonzero(alone) > 0, "an all-zero centre would compare equal"
    assert np.array_equal(alone, weights_of_a_run(tmp_path, pool=2))
    assert np.array_equal(alone, weights_of_a_run(tmp_path, pool=5))


def test_the_weight_comparison_can_fail(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Perturb the combination order, and the equivalence test must go red.

    The test above compares two runs and finds them equal. That proves
    nothing unless a wrong combination gives a different answer. This test
    puts one in, behind a switch that only a test can reach, and requires the
    weights to move.
    """
    from cachette.learn import shard

    honest = shard.combine_episodes

    def by_arrival(
        scores: Sequence[EpisodeScore],
        candidates: int,
        seeds: int,
        strategy: str = "",
    ) -> Generation:
        """Give each result the candidate index of another result.

        This is the defect the sort exists to stop: the score of one episode
        lands on another candidate. The generation is covered, so the
        coverage check passes and only the weights say that anything moved.
        """
        keys = [held.first_candidate for held in scores]
        return honest(
            [
                replace(held, first_candidate=key)
                for held, key in zip(reversed(scores), keys, strict=True)
            ],
            candidates,
            seeds,
            strategy,
        )

    alone = weights_of_a_run(tmp_path, pool=1)
    monkeypatch.setattr(shard, "combine_episodes", by_arrival)
    assert not np.array_equal(alone, weights_of_a_run(tmp_path, pool=2))


def test_a_dead_episode_fails_the_run(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A silent partial generation would train on a subset and say nothing.

    The switch runs in the worker process, because a test in this process
    cannot reach one. A worker that raises must carry the failure back
    through the pool and end the run.
    """
    monkeypatch.setenv(SHARD_FAULT, "0")
    with pytest.raises(RuntimeError, match="fault switch"):
        weights_of_a_run(tmp_path, pool=2)


def test_a_run_of_two_strategies_holds_one_queue_and_writes_both_logs(
    tmp_path: Path,
) -> None:
    """Drive the run the launcher drives, and read what it left behind.

    **The queue and the per-strategy log have one obligated caller, and it is
    the run.** A test that opened the pool itself would prove that the pool
    works and prove nothing about whether a run reaches it. This drives the
    command line the launcher calls.

    The run writes one log for each strategy, under the name the dashboard
    opens, and one report for the whole run. It also prints the heartbeat that
    the progress reader parses, so the line is asserted here by shape.
    """
    finished = subprocess.run(
        [
            sys.executable,
            "-m",
            "cachette.learn",
            "--only",
            "conquer,land",
            "--out",
            str(tmp_path),
            "--pool",
            "3",
            "--generations",
            "1",
            "--population",
            "4",
            "--seeds",
            "1",
            "--holdout",
            "2",
            "--validation",
            "2",
            "--world-extent",
            "14",
            "--factions",
            "2",
            "--tick-limit",
            "300",
            "--decision-interval",
            "10",
        ],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    )

    report = json.loads((tmp_path / "report.json").read_text(encoding="utf-8"))
    assert sorted(report["strategies"]) == ["conquer", "land"]
    assert report["pool"] == 3

    for name in ("conquer", "land"):
        log = (tmp_path / f"{name}.log").read_text(encoding="utf-8")
        assert f"=== {name} (" in log
        assert f"{name} generation  0 " in log
        # A log of one strategy holds no line of the other, because the
        # thread that printed it is the strategy.
        other = "land" if name == "conquer" else "conquer"
        assert f"=== {other} (" not in log

    # The heartbeat the progress reader parses. It names the episodes that
    # finished, the episodes still live and the ticks a second since the last
    # line.
    assert "working  episodes" in finished.stdout
    assert "live    0/4" in finished.stdout


def test_a_line_of_one_strategy_never_holds_the_text_of_another(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Two strategies print at once, and a reader matches a line by shape.

    The print function writes the text and the line break in two calls, so a
    thread that wrote between them puts its own text inside the line of
    another. The combined log is the file the dashboard reads, and a spliced
    line matches nothing there.

    This drives the two writes of one print from two threads, in the order
    that splices a line, and requires the combined stream to hold whole lines
    in the order they were completed.
    """
    combined = io.StringIO()
    monkeypatch.setattr(sys, "stdout", combined)
    log = tmp_path / "first.log"
    with strategy_logs() as journal, log.open("w", encoding="utf-8") as file:
        journal.register(file)
        journal.write("first half")
        # Another thread writes between the two calls of one print. It must
        # be another thread, because the route reads the thread.
        other = ThreadPoolExecutor(max_workers=1)
        other.submit(journal.write, "second whole\n").result()
        journal.write(" and the rest\n")
        other.shutdown()
        journal.forget()

    assert combined.getvalue() == "second whole\nfirst half and the rest\n"
    assert log.read_text(encoding="utf-8") == "first half and the rest\n"
