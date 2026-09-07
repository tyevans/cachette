"""A generation split across processes gives the weights one process gives.

The trainer scores a population of candidates on a set of seeds. Those
episodes are independent, so a run may play them in several worker processes.
**The split must not reach the answer.** A run at four shards and a run at one
shard start from the same seed, so they must end at the same weights, position
for position.

The property has two halves, and both are tested here.

**The combination is ordered by the candidate index.** A shard reports the
candidate index it started at, and the combination sorts on that key. Nothing
reads which shard answered first.

**A worker rebuilds its candidates.** It receives the centre, the generation
number and the pair range, and it draws the perturbations from the run seed
and the generation number. No candidate crosses to a worker, so a worker that
drew from another stream would score a different population and the weights
would move.

**The comparison must be able to fail.** One test perturbs the combination
order behind the trainer and requires the weights to move. A second test
fails one shard on purpose and requires the generation to raise, because a
generation that lost a shard would otherwise train on a subset and say
nothing.
"""

from __future__ import annotations

from dataclasses import replace
from itertools import pairwise
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette.learn import Weighting
from cachette.learn.env import EnvConfig, viable_seeds
from cachette.learn.policy import load_policy
from cachette.learn.shard import (
    SHARD_FAULT,
    ShardScore,
    combine_shards,
    shard_ranges,
)
from cachette.learn.train import TrainConfig, train

if TYPE_CHECKING:
    from collections.abc import Sequence
    from pathlib import Path

    from cachette.learn.train import Generation

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

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)


def weights_of_a_run(out: Path, shards: int) -> np.ndarray:
    """Train one policy at one shard count, and return the centre it reached."""
    pool = viable_seeds(WORLD, 4, 900)
    result = train(
        f"shard-{shards}",
        WORLD,
        WEIGHTING,
        TrainConfig(
            generations=2,
            population=8,
            seeds_per_generation=2,
            workers=2,
            shards=shards,
            seed=3,
        ),
        out,
        pool,
        kind="linear",
    )
    stored, _ = load_policy(out / f"{result['name']}-latest.npz")
    return np.asarray(stored.flat())


def score(first: int, values: Sequence[float]) -> ShardScore:
    """Build one shard result over made-up scores."""
    array = np.asarray(values, dtype=float)
    return ShardScore(
        first_candidate=first,
        candidates=len(array),
        ranked=array,
        absolute=array,
        wins=1,
        games=len(array),
        ticks=len(array),
    )


def test_shard_ranges_cover_every_pair_exactly_once() -> None:
    ranges = shard_ranges(pairs=7, processes=3, width=1)
    assert ranges[0][0] == 0
    assert ranges[-1][1] == 7
    for before, after in pairwise(ranges):
        assert before[1] == after[0]


def test_a_shard_boundary_falls_between_two_seat_groups() -> None:
    """A boundary inside a group would leave a world with an empty seat.

    A league run of two learner seats puts two pairs in one group of worlds.
    A shard that held one of those pairs would build a world whose second
    learner seat nobody filled, and the margin in that world would have
    nothing to subtract.
    """
    for first, last in shard_ranges(pairs=8, processes=3, width=2):
        assert first % 2 == 0
        assert (last - first) % 2 == 0


def test_more_processes_than_groups_use_one_process_for_each_group() -> None:
    assert shard_ranges(pairs=2, processes=8, width=1) == [(0, 1), (1, 2)]


def test_the_combination_orders_by_the_candidate_index() -> None:
    """The shards may answer in any order, and the answer must not move."""
    shards = [score(0, [1.0, 2.0]), score(2, [3.0, 4.0]), score(4, [5.0, 6.0])]
    combined = combine_shards(list(reversed(shards)), candidates=6)
    assert np.array_equal(combined.ranked, np.array([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]))


def test_the_order_assertion_can_fail() -> None:
    """A combination that took the arrival order would give another answer.

    The test above compares against one fixed array. This one shows that the
    array is sensitive to the order, so the assertion is not satisfied by any
    arrangement of the same numbers.
    """
    shards = [score(0, [1.0, 2.0]), score(2, [3.0, 4.0]), score(4, [5.0, 6.0])]
    arrived = np.concatenate([held.ranked for held in reversed(shards)])
    assert not np.array_equal(arrived, combine_shards(shards, candidates=6).ranked)


def test_a_missing_shard_fails_the_combination() -> None:
    """A generation that lost a shard would otherwise train on a subset."""
    with pytest.raises(ValueError, match="do not cover the population"):
        combine_shards([score(0, [1.0, 2.0]), score(4, [5.0, 6.0])], candidates=6)


def test_a_short_combination_fails_even_when_it_is_contiguous() -> None:
    """A lost tail shard leaves a contiguous run that is too short."""
    with pytest.raises(ValueError, match="cover 4 candidates"):
        combine_shards([score(0, [1.0, 2.0]), score(2, [3.0, 4.0])], candidates=6)


def test_a_sharded_run_reaches_the_weights_the_unsharded_run_reaches(
    tmp_path: Path,
) -> None:
    """This is the property the whole change rests on.

    The run at one shard scores the whole generation in this process. The runs
    at two and four shards score it in worker processes that rebuild their own
    candidates. All three start from one seed, so all three must end at one
    centre, position for position.
    """
    alone = weights_of_a_run(tmp_path, shards=1)
    assert np.count_nonzero(alone) > 0, "an all-zero centre would compare equal"
    assert np.array_equal(alone, weights_of_a_run(tmp_path, shards=2))
    assert np.array_equal(alone, weights_of_a_run(tmp_path, shards=4))


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

    honest = shard.combine_shards

    def reversed_order(scores: Sequence[ShardScore], candidates: int) -> Generation:
        """Give each shard the candidate index of another shard.

        This is the defect the sort exists to stop: the scores of one shard
        land on the candidates of another. The population is covered, so the
        coverage check passes and only the weights say that anything moved.
        """
        keys = [held.first_candidate for held in scores]
        return honest(
            [
                replace(held, first_candidate=key)
                for held, key in zip(reversed(scores), keys, strict=True)
            ],
            candidates,
        )

    alone = weights_of_a_run(tmp_path, shards=1)
    monkeypatch.setattr(shard, "combine_shards", reversed_order)
    assert not np.array_equal(alone, weights_of_a_run(tmp_path, shards=2))


def test_a_dead_shard_fails_the_generation(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A silent partial generation would train on a subset and say nothing.

    The switch runs in the worker process, because a test in this process
    cannot reach one. A worker that raises must carry the failure back
    through the pool and end the run.
    """
    monkeypatch.setenv(SHARD_FAULT, "0")
    with pytest.raises(RuntimeError, match="fault switch"):
        weights_of_a_run(tmp_path, shards=2)
