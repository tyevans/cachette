"""One decision builds the observation of its state once.

The observation array of a faction is the dearest thing the control plane
asks the engine for between two decisions. A decision used to ask for it four
times: the loop asked for the policy input, the reward asked for its terms,
the outcome reader asked for its three fields, and the step result carried a
fourth build that no caller read.

Every one of those builds ran in one interpreter while every engine worker of
the process waited. The four are now one.

**The scores must not move.** The four builds read one world state, so they
gave one set of numbers, and one build gives the same numbers. These tests
hold that: a decision reports the array of the state after its ticks, a
finished episode reports the array of its last decision, and a population
gives the same returns at any worker count.

References
----------
[^1]: Report 38, where the training time goes, section 10.2.
``docs/research/reports/38-where-the-training-time-goes.md``
[^2]: Testing Rules, sections 1 and 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import numpy as np

from cachette.learn import Env, EnvConfig, Weighting
from cachette.learn.env import viable_seeds
from cachette.learn.policy import LinearPolicy
from cachette.learn.reward import Reward
from cachette.learn.rollout import run_population

# A weighting with every weight set, so the reward runs. The values are the
# test's own and they state no rule of the downstream game.
WEIGHTING = Weighting(
    terms={"held_tiles": 1.0, "population": 0.5},
    won=100.0,
    lost=-100.0,
    drawn=0.0,
)

# A small world with a short episode. The tests drive whole decisions, so the
# cost of the fixture is the cost of the suite.
CONFIG = EnvConfig(
    width=32,
    height=32,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=10,
    decision_interval=4,
)


def a_policy(env: Env, seed: int) -> LinearPolicy:
    """Return a policy whose weights differ from every other seed.

    **A policy of zero weights chooses one action in every world.** Every
    world then walks the same path, and a test over it measures the fixture
    rather than the code. The weights here come from a fixed generator, so
    the worlds diverge and the run still repeats.
    """
    rng = np.random.default_rng(seed)
    shell = LinearPolicy.zeros(env.action_length, env.observation_length)
    return shell.rebuild(rng.standard_normal(shell.flat().size))


def test_a_decision_reports_the_observation_of_the_state_after_its_ticks() -> None:
    """The array in the result belongs to the state the ticks left.

    The reward of a decision reads the state after the ticks, and the next
    decision hands the policy that same state. So one array serves both. An
    array taken before the ticks would score the decision on the state that
    the decision started from.
    """
    seed = viable_seeds(CONFIG, 1, start=0)[0]
    env = Env(CONFIG, WEIGHTING)
    before = np.asarray(env.reset(seed))

    first = env.step(0)
    after_first = np.asarray(env.observation())
    assert np.array_equal(np.asarray(first.observation), after_first), (
        "the result of a decision must carry the array of the state it left"
    )
    assert not np.array_equal(np.asarray(first.observation), before), (
        "four ticks of this world must change the observation of the seat"
    )

    second = env.step(0)
    assert np.array_equal(
        np.asarray(second.observation), np.asarray(env.observation())
    ), "the second decision must carry its own array and not the first one"
    assert not np.array_equal(
        np.asarray(second.observation), np.asarray(first.observation)
    ), "two decisions of this world must not report one array"


def test_a_reset_forgets_the_array_of_the_episode_before_it() -> None:
    """A new episode reports the array of its own world.

    The environment keeps the array of the last decision, so that a finished
    episode answers without another build. A reset that kept it would report
    the world of the run before this one.
    """
    seeds = viable_seeds(CONFIG, 2, start=0)
    env = Env(CONFIG, WEIGHTING)
    env.reset(seeds[0])
    env.step(0)
    first = np.asarray(env.idle().observation)

    env.reset(seeds[1])
    second = np.asarray(env.idle().observation)
    assert np.array_equal(second, np.asarray(env.observation())), (
        "a reset environment must report the array of its new world"
    )
    assert not np.array_equal(first, second), (
        "two worlds of two seeds must not report one array"
    )


def test_a_finished_episode_reports_the_array_of_its_last_decision() -> None:
    """A world that has ended stands still, so its array stands still.

    A finished episode leaves the batch and its world takes no further tick.
    The environment therefore answers every later row with the array of the
    last decision, and that array still equals a fresh reading.
    """
    seed = viable_seeds(CONFIG, 1, start=0)[0]
    env = Env(CONFIG, WEIGHTING)
    env.reset(seed)
    while not env.done:
        env.step(0)
    held = np.asarray(env.idle().observation)
    assert np.array_equal(held, np.asarray(env.observation())), (
        "the array of a finished episode must equal a fresh reading of it"
    )
    assert np.array_equal(held, np.asarray(env.idle().observation)), (
        "two idle rows of one finished episode must report one array"
    )


def test_a_reward_reads_the_same_numbers_from_an_array_a_caller_holds() -> None:
    """A passed array and a fresh build give one reading.

    The reward reads named positions of the array. The array a caller holds
    and the array the world would build are the same numbers at one state, so
    the two readings agree in every field.
    """
    seed = viable_seeds(CONFIG, 1, start=0)[0]
    env = Env(CONFIG, WEIGHTING)
    env.reset(seed)
    world = env.world
    fresh = Reward(world, CONFIG.seat, WEIGHTING)
    passed = Reward(world, CONFIG.seat, WEIGHTING)
    for _ in range(3):
        env.apply(0)
        for _ in range(CONFIG.decision_interval):
            world.step(1)
        held = env.observation()
        by_world = fresh.read(world)
        by_array = passed.read(world, held)
        assert by_array.value == by_world.value, (
            "a reward must pay the same value from a passed array"
        )
        assert by_array.terms == by_world.terms, (
            "a reward must report the same terms from a passed array"
        )
        assert by_array.outcome == by_world.outcome, (
            "an outcome must not depend on who built the array"
        )


def test_a_population_scores_the_same_at_any_worker_count() -> None:
    """The returns of a batch do not depend on how the work was spread.

    The batch spreads its worlds over its workers and sorts every result by
    the index of the world. A run at one worker and a run at four workers
    therefore report one set of returns for one set of seeds.

    **This is the test that guards the change.** One array now serves the
    policy, the reward and the result. A stale array or an array of the wrong
    state would move a return, and a moved return shows here.
    """
    seeds = viable_seeds(CONFIG, 3, start=0)
    probe = Env(CONFIG, WEIGHTING)
    policies = [a_policy(probe, 7), a_policy(probe, 8)]

    alone = run_population(CONFIG, WEIGHTING, policies, seeds, workers=1)
    spread = run_population(CONFIG, WEIGHTING, policies, seeds, workers=4)

    assert np.array_equal(alone.returns, spread.returns), (
        "a return must not depend on the worker count"
    )
    assert alone.ticks == spread.ticks, (
        "a batch must run the same ticks at any worker count"
    )
    assert [record.outcome for record in alone.episodes] == [
        record.outcome for record in spread.episodes
    ], "an outcome must not depend on the worker count"


def test_the_worker_count_comparison_can_fail() -> None:
    """A run whose seeds differ must not read as one run.

    The comparison above passes when two runs agree. This proves that it can
    disagree: two runs over two seed sets report different returns, so the
    assertion has something to catch.
    """
    first = viable_seeds(CONFIG, 3, start=0)
    second = viable_seeds(CONFIG, 3, start=first[-1] + 1)
    probe = Env(CONFIG, WEIGHTING)
    policies = [a_policy(probe, 7)]

    one = run_population(CONFIG, WEIGHTING, policies, first, workers=1)
    other = run_population(CONFIG, WEIGHTING, policies, second, workers=4)
    assert not np.array_equal(one.returns, other.returns), (
        "two seed sets must not give one set of returns"
    )
