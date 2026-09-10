"""A measurement pass plays one episode in each task of the shared queue.

A training run measures as much as it trains. The passes that measure it
stepped many worlds in one process, and that shape does not fill a machine:
the section one interpreter runs between two decisions holds every engine
worker of the process, and a measurement put that section at about a seventh
of the wall clock of a pass.

These tests hold the properties that make the queue admissible and that make
it worth having.

**The answer must not move.** A pass gives the same records at any pool size,
and the records agree with the pass in one process, position for position.
The combination sorts on the policy, the repeat and the seed position, and it
reads no completion order.[^1]

**No task may hold more than one episode.** That is the whole of the change,
and it is what keeps a worker from waiting on the longest game of a block
somebody gave it. The uneven fixture below holds the case: its episodes end
at tick counts that differ by more than a factor of two, so a pass that gave
each worker a block would wait for the longest of them.

**One play answers for every objective.** A pass of two objectives plays the
same games as a pass of one, and it plays them once.

No test here reads a clock.[^2]

References
----------
[^1]: ADR-0194, a generation is scored one episode at a time, and combined in
candidate order, decisions D1, D3 and D4.
``docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md``
[^2]: Testing Rules, sections 1, 2a and 3. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from dataclasses import replace

import numpy as np
import pytest

from cachette.learn import Env, EnvConfig, Weighting
from cachette.learn.measure import (
    SOLE,
    MeasurementScore,
    combine_measurements,
    drawing_from,
    measurement_tasks,
    queued_objectives,
    queued_population,
    queued_repeats,
)
from cachette.learn.policy import LinearPolicy, RandomPolicy
from cachette.learn.rollout import run_objectives, run_population
from cachette.learn.shard import ShardPool
from cachette.learn.train import evaluate

# Two objectives over the same episodes. The first reads the ground the seat
# holds and the second reads the people it feeds, so the two weight one set
# of games differently and neither one can be read as the other.
GROUND = Weighting(
    terms={"held_tiles": 1.0},
    levels={"live_units": 0.25},
    won=100.0,
    lost=-100.0,
    drawn=0.0,
)
PEOPLE = Weighting(terms={"population": 1.0}, won=10.0, lost=-10.0, drawn=0.0)

# A small world with a short episode. The tests drive whole decisions, so the
# cost of the fixture is the cost of the suite.
LEARNER = EnvConfig(
    width=32,
    height=32,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=6,
    decision_interval=4,
)

# The same world with the seat given back to the built-in controller, which
# is the world the controller baseline plays.
CONTROLLER = replace(LEARNER, controlled=False)

SEEDS = [11, 12, 13]

# **The uneven world lets an episode end early, and the flat one does not.**
# The horizon of the world above covers its tick limit, so every episode of
# it runs the whole limit and the pass is perfectly even. A pass over even
# episodes measures nothing about a queue, because a block of them would
# finish together. This world raises the tick limit far above the horizon, so
# a game that resolves ends at its own tick and a game that does not runs to
# the horizon.
UNEVEN = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=20_000,
    horizon=40,
    decision_interval=6,
    controlled=False,
)

UNEVEN_SEEDS = [3, 5, 7, 9, 11, 13, 17, 19]

# **A world where a random action changes nothing measures the fixture.** The
# world above gives the seat six decisions, and a random policy driven from
# three different streams earned exactly the same return on every seed of it.
# The tests of a drawing pass therefore play this world, which gives the seat
# forty decisions over 600 ticks. Three streams part company on it at every
# seed.
DRAWING = EnvConfig(
    width=32,
    height=32,
    faction_count=3,
    seat=0,
    tick_limit=600,
    horizon=40,
    decision_interval=8,
)


@pytest.fixture(name="probe", scope="module")
def probe_env() -> Env:
    """One environment, for the observation and action lengths of the world."""
    return Env(LEARNER, GROUND)


@pytest.fixture(name="zeros", scope="module")
def zero_policy(probe: Env) -> LinearPolicy:
    """Return the untrained policy, which takes the no-op at every decision."""
    return LinearPolicy.zeros(probe.action_length, probe.observation_length)


def rows(record: object) -> list[dict[str, float]]:
    """Return the reading of every episode of one record, in index order."""
    return [row.as_row() for row in record.episodes]  # type: ignore[attr-defined]


def test_a_queued_pass_gives_the_same_records_at_every_pool_size(
    zeros: LinearPolicy,
) -> None:
    """The pool size changes the spread of the work and nothing else.

    This is the property the whole change rests on. A run whose figures moved
    with the size of its pool could not be repeated.
    """
    with ShardPool(2) as pool:
        small = queued_population(pool, LEARNER, GROUND, [zeros], SEEDS)
    with ShardPool(3) as pool:
        large = queued_population(pool, LEARNER, GROUND, [zeros], SEEDS)

    assert small.ticks == large.ticks
    assert np.array_equal(small.returns, large.returns)
    assert rows(small) == rows(large)


def test_a_queued_pass_agrees_with_a_pass_in_one_process(
    zeros: LinearPolicy,
) -> None:
    """The queue moves where an episode runs, and moves no number it earns."""
    alone = run_population(LEARNER, GROUND, [zeros], SEEDS, workers=1)
    with ShardPool(3) as pool:
        queued = queued_population(pool, LEARNER, GROUND, [zeros], SEEDS)

    assert queued.seeds == alone.seeds
    assert queued.ticks == alone.ticks
    assert np.array_equal(queued.returns, alone.returns)
    assert rows(queued) == rows(alone)


def test_a_queued_pass_plays_each_episode_once_for_every_objective(
    zeros: LinearPolicy,
) -> None:
    """Two objectives weight one set of games, and the pass plays it once.

    The tick count of a pass is the simulated cost it paid. A pass that
    played the games twice would pay twice, so the tick count is what says
    that it played them once.
    """
    with ShardPool(3) as pool:
        one = queued_objectives(pool, CONTROLLER, {"ground": GROUND}, [zeros], SEEDS)
        two = queued_objectives(
            pool, CONTROLLER, {"ground": GROUND, "people": PEOPLE}, [zeros], SEEDS
        )

    assert list(two) == ["ground", "people"]
    assert two["ground"].ticks == one["ground"].ticks
    assert two["people"].ticks == one["ground"].ticks
    assert np.array_equal(two["ground"].returns, one["ground"].returns)
    # The two objectives read one set of games, so every episode ends at the
    # same tick with the same outcome under both names.
    ends = [(row.end_tick, row.outcome) for row in two["ground"].episodes]
    assert ends == [(row.end_tick, row.outcome) for row in two["people"].episodes]
    # **The two must weight the games differently**, or the test would pass
    # for a pass that scored one objective twice under two names.
    assert not np.array_equal(two["ground"].returns, two["people"].returns)


def test_a_queued_pass_agrees_with_a_shared_pass_in_one_process(
    zeros: LinearPolicy,
) -> None:
    """A pass of several objectives gives what one process gives, name by name."""
    named = {"ground": GROUND, "people": PEOPLE}
    alone = run_objectives(CONTROLLER, named, [zeros], SEEDS, 1)
    with ShardPool(2) as pool:
        queued = queued_objectives(pool, CONTROLLER, named, [zeros], SEEDS)

    assert list(queued) == list(alone)
    for name in named:
        assert queued[name].ticks == alone[name].ticks
        assert np.array_equal(queued[name].returns, alone[name].returns)
        assert rows(queued[name]) == rows(alone[name])


def test_no_task_of_a_pass_holds_more_than_one_episode(
    zeros: LinearPolicy,
) -> None:
    """One task is one episode, whatever the pool size is.

    A task that held a block of episodes would end when its own longest game
    ended, and its worker would idle until then. The task count therefore
    reads the policies, the repeats and the seeds, and never the pool.
    """
    with ShardPool(2) as pool:
        shared = [pool.share(zeros)]
        tasks = measurement_tasks(LEARNER, {SOLE: GROUND}, shared, SEEDS, repeats=3)
        pool.release(shared[0])

    assert len(tasks) == len(SEEDS) * 3
    assert len({(task.policy_index, task.repeat, task.seed) for task in tasks}) == len(
        tasks
    )
    assert sorted({task.seed for task in tasks}) == sorted(SEEDS)


def test_the_uneven_fixture_holds_episodes_of_very_different_length() -> None:
    """The fixture reaches the case a queue exists for.

    **Put the block shape back and this is what fails.** A pass whose
    episodes all ran the same number of ticks would finish a block of them
    together, so it would measure nothing about the queue. This asserts that
    the fixture really does spread its episodes, before any test reads a
    result from it.
    """
    probe = Env(UNEVEN, GROUND)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    with ShardPool(3) as pool:
        played = queued_population(pool, UNEVEN, GROUND, [zeros], UNEVEN_SEEDS)

    ends = sorted(row.end_tick for row in played.episodes)
    assert len(ends) == len(UNEVEN_SEEDS)
    assert min(ends) > 0
    assert max(ends) >= 2 * min(ends)


def test_an_uneven_pass_gives_the_same_records_at_every_pool_size() -> None:
    """An uneven pass is where a block shape loses, and it must still repeat.

    The episodes of this fixture end at tick counts that differ by more than
    a factor of two, so the workers of two pools of different sizes take them
    in different orders. The records must not notice.
    """
    probe = Env(UNEVEN, GROUND)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    with ShardPool(2) as pool:
        small = queued_population(pool, UNEVEN, GROUND, [zeros], UNEVEN_SEEDS)
    with ShardPool(4) as pool:
        large = queued_population(pool, UNEVEN, GROUND, [zeros], UNEVEN_SEEDS)

    assert small.ticks == large.ticks
    assert np.array_equal(small.returns, large.returns)
    assert rows(small) == rows(large)


def test_each_episode_of_a_pass_draws_its_own_stream() -> None:
    """The stream of an episode depends on the repeat and on the seed position.

    **This tests each field of the key, not only that the draw repeats.** A
    stream keyed on the repeat alone would give every seed of one repeat the
    same actions, and a stream keyed on the seed position alone would give
    every repeat of one seed the same actions. Both would pass a test that
    only asked whether the pass repeats.
    """
    first = drawing_from(RandomPolicy(seed=4), 0, 0)
    later = drawing_from(RandomPolicy(seed=4), 1, 0)
    beside = drawing_from(RandomPolicy(seed=4), 0, 1)
    again = drawing_from(RandomPolicy(seed=4), 0, 0)
    masks = np.ones((12, 8), dtype=np.uint8)
    watching = np.zeros((12, 4))

    assert first.choose_many(watching, masks) == again.choose_many(watching, masks)
    assert first.choose_many(watching, masks) != later.choose_many(watching, masks)
    assert first.choose_many(watching, masks) != beside.choose_many(watching, masks)


def test_a_policy_that_draws_nothing_is_given_back_as_it_stands(
    zeros: LinearPolicy,
) -> None:
    """A trained policy states no stream, so no episode reseeds it."""
    assert drawing_from(zeros, 3, 7) is zeros


def test_a_repeated_drawing_pass_keeps_its_repeat_count() -> None:
    """A repeat of the seed set is a repeat, and the repeats are not one repeat.

    The engine is deterministic, so only a policy that draws gains anything
    from a repeat. **Every episode of a queued pass therefore draws its own
    stream**, keyed on the repeat and the seed position. A pass whose workers
    all started from one state would report the first repeat three times.
    """
    with ShardPool(3) as pool:
        thrice = queued_repeats(
            pool, DRAWING, GROUND, RandomPolicy(seed=4), SEEDS, repeats=3
        )
        again = queued_repeats(
            pool, DRAWING, GROUND, RandomPolicy(seed=4), SEEDS, repeats=3
        )

    assert len(thrice) == 3
    for record in thrice:
        assert record.returns.shape == (1, len(SEEDS))
    assert not np.array_equal(thrice[0].returns, thrice[1].returns)
    assert not np.array_equal(thrice[1].returns, thrice[2].returns)
    # The same call gives the same three repeats, so the pass repeats on its
    # own terms even though it draws.
    for held, repeated in zip(thrice, again, strict=True):
        assert np.array_equal(held.returns, repeated.returns)


def test_the_random_baseline_keeps_its_repeat_count_through_evaluate() -> None:
    """The pass a run calls reports the episodes of every repeat it asked for.

    This drives the caller a run drives, rather than the queue underneath it.
    """
    with ShardPool(3) as pool:
        summary = evaluate(
            DRAWING,
            GROUND,
            RandomPolicy(seed=4),
            SEEDS,
            workers=1,
            repeats=3,
            pool=pool,
        )
        once = evaluate(
            DRAWING, GROUND, RandomPolicy(seed=4), SEEDS, workers=1, pool=pool
        )

    assert summary["episodes"] == float(3 * len(SEEDS))
    assert once["episodes"] == float(len(SEEDS))


def test_a_pass_that_lost_an_episode_fails(zeros: LinearPolicy) -> None:
    """A combination refuses a set of results that does not cover the pass.

    A mean over a cell that no episode filled reads as a score, and nothing
    else in the pass would say so.
    """
    scores = [
        MeasurementScore(
            policy_index=0,
            repeat=0,
            seed_position=position,
            returns={SOLE: 1.0},
            episodes={},
            ticks=1,
        )
        for position in range(len(SEEDS) - 1)
    ]
    with pytest.raises(ValueError, match="do not cover the pass"):
        combine_measurements(scores, [SOLE], 1, SEEDS)


def test_a_pass_that_scored_one_episode_twice_fails() -> None:
    """Two results at one key would lose one of them, so the pass fails."""
    twice = [
        MeasurementScore(
            policy_index=0,
            repeat=0,
            seed_position=0,
            returns={SOLE: 1.0},
            episodes={},
            ticks=1,
        )
    ] * 2
    with pytest.raises(ValueError, match="so one of them would be lost"):
        combine_measurements(twice, [SOLE], 1, [11])
