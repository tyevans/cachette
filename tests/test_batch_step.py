"""The batch step gives what the single step gives, at every thread count.

A batch of worlds steps in one call. The batch must leave each world in the
state the single step would have left it, and it must report in index order
and never in the order a worker finished.[^1]

A test that steps one batch and compares it against itself proves nothing, so
each test here compares a batch against a run the batch did not produce.[^2]

References
----------
[^1]: ADR-0155, a batch of worlds steps in one call, in index order.
``docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md``
[^2]: Testing Rules, section 1. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import pytest

from cachette import ConfigError, StepError, World
from cachette._core import Batch

# The worlds of one batch. A small side keeps the test quick, and four worlds
# over three workers gives one worker two worlds, so the strided assignment is
# exercised rather than assumed.
SEEDS = (11, 22, 33, 44)
SIDE = 24
FACTIONS = 3
TICKS = 6


def build(seed: int) -> World:
    """Build and seed one world."""
    world = World(width=SIDE, height=SIDE, seed=seed, faction_count=FACTIONS)
    world.seed_world()
    return world


def run_alone(seed: int, ticks: int, threads: int) -> tuple[int, list[int]]:
    """Step one world by itself, and report its hash and its event counts."""
    world = build(seed)
    counts = [world.step(threads) for _ in range(ticks)]
    return world.state_hash(), counts


@pytest.mark.parametrize("workers", [1, 2, 3, 8])
@pytest.mark.parametrize("threads", [1, 2])
def test_a_batch_gives_what_the_worlds_give_alone(workers: int, threads: int) -> None:
    """Four worlds in one batch equal four worlds stepped one at a time.

    The comparison is the state hash of each world and the event count of
    each step. Both come from a run that used no batch at all, so the batch
    is compared against something else and not against itself.
    """
    alone = [run_alone(seed, TICKS, threads) for seed in SEEDS]

    worlds = [build(seed) for seed in SEEDS]
    batch = Batch(worlds)
    assert len(batch) == len(SEEDS)

    batched_counts: list[list[int]] = [[] for _ in SEEDS]
    for _ in range(TICKS):
        rows = batch.step(workers, threads)
        # The rows arrive in index order, whatever order the workers took.
        assert [row.index for row in rows] == list(range(len(SEEDS)))
        for row in rows:
            assert row.error is None
            batched_counts[row.index].append(row.events)

    for index, seed in enumerate(SEEDS):
        expected_hash, expected_counts = alone[index]
        assert worlds[index].state_hash() == expected_hash, (
            f"the world of seed {seed} at index {index} ended at a different state"
        )
        assert batched_counts[index] == expected_counts, (
            f"the world of seed {seed} at index {index} emitted different events"
        )


def test_a_batch_gives_one_answer_at_every_worker_count() -> None:
    """One worker and eight workers leave the same state hashes."""
    hashes: dict[int, list[int]] = {}
    for workers in (1, 8):
        worlds = [build(seed) for seed in SEEDS]
        batch = Batch(worlds)
        for _ in range(TICKS):
            batch.step(workers, 1)
        hashes[workers] = [world.state_hash() for world in worlds]
    assert hashes[1] == hashes[8]


def test_the_batch_reports_in_index_order_and_not_in_worker_order() -> None:
    """The rows carry ascending indices, whatever order the workers ran in.

    Two workers over five worlds gives worker zero the worlds 0, 2 and 4, and
    worker one the worlds 1 and 3. A batch that reported in the order its
    workers finished would therefore report 0, 2, 4, 1, 3. Only the sort by
    the index puts them in the order the caller gave.

    The worlds run from the largest side to the smallest, so a longer step
    cannot line the two orders up by accident.
    """
    sides = (64, 56, 48, 40, 32)
    worlds = []
    for index, side in enumerate(sides):
        world = World(width=side, height=side, seed=100 + index, faction_count=2)
        world.seed_world()
        worlds.append(world)
    batch = Batch(worlds)
    rows = batch.step(2, 1)
    assert [row.index for row in rows] == list(range(len(sides)))


def test_the_batch_hands_back_the_world_the_caller_gave_it() -> None:
    """A world of a batch is the world the caller built, and not a copy."""
    worlds = [build(seed) for seed in SEEDS]
    batch = Batch(worlds)
    for index, world in enumerate(worlds):
        assert batch.world(index) is world
    with pytest.raises(IndexError):
        batch.world(len(SEEDS))


def test_a_batch_refuses_one_world_twice() -> None:
    """Two entries of one world would put two workers on one lock."""
    world = build(SEEDS[0])
    with pytest.raises(ConfigError):
        Batch([world, world])


def test_a_batch_refuses_no_world_and_refuses_a_count_of_zero() -> None:
    """An empty batch and a zero count are refusals, not silent no-ops."""
    with pytest.raises(ConfigError):
        Batch([])
    batch = Batch([build(SEEDS[0])])
    with pytest.raises(StepError):
        batch.step(0, 1)
    with pytest.raises(StepError):
        batch.step(1, 0)
