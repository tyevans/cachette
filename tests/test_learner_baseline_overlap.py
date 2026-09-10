"""The bar plays beside the training, and never in front of it.

A training run reports every policy against the built-in controller playing
the learner's own seat. That is the bar, and it is a reporting quantity: the
search ranks the candidates of a generation against each other by win share,
and nothing that trains a weight reads a baseline.

A run measured that bar before its first generation and waited for it. On a
rented machine of 64 cores it spent about twenty minutes there, with the
queue empty, for a number no generation reads.

These tests hold the two halves of the change. **A baseline pass is started
and read later**, and other work of the run runs between the two. **A judge
that starts its yardstick does not hold the answer when it returns**, and the
answer it takes later is the answer a waiting judge takes.

A run whose inputs match a stored baseline still reads that store and
enqueues nothing, so the overlap costs the cache nothing.

No test here reads a clock.[^1]

References
----------
[^1]: Testing Rules, section 3. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from dataclasses import replace
from typing import TYPE_CHECKING

import pytest

from cachette.learn import Env, EnvConfig, Weighting
from cachette.learn.baseline import BaselineCache, start_controller_baselines
from cachette.learn.config import TrainConfig
from cachette.learn.measure import queued_population
from cachette.learn.policy import LinearPolicy
from cachette.learn.shard import ShardPool
from cachette.learn.train import Validator, train

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from pathlib import Path

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)

# A small world with a short episode. The tests drive whole runs, so the cost
# of the fixture is the cost of the suite.
SMALL = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

# The same world with the seat given back to the built-in controller.
CONTROLLER = replace(SMALL, controlled=False)

HOLDOUT = [101, 102, 103, 104]
VALIDATION = [201, 202]
POOL = [301, 302]

# The schema version the pass keys its cache on. No test here reads a stored
# baseline, so the value only has to be one number.
SCHEMA = 1


def test_a_baseline_pass_is_started_and_read_after_other_work() -> None:
    """The run submits the bar, plays its own episodes, then reads the bar.

    **This is the shape the change buys.** The pass holds no worker of its
    own, so the episodes between the two calls below reach the same queue and
    the same workers.
    """
    probe = Env(CONTROLLER, WEIGHTING)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    with ShardPool(3) as pool:
        started = start_controller_baselines(
            CONTROLLER,
            {"ground": WEIGHTING},
            zeros,
            HOLDOUT,
            1,
            SCHEMA,
            "probe baseline",
            BaselineCache(directory=None),
            pool,
        )
        assert started.measuring == ("ground",)

        # The work a generation does, submitted after the bar and finished
        # before anybody asks the bar for an answer.
        played = queued_population(pool, SMALL, WEIGHTING, [zeros], POOL)
        assert played.ticks > 0

        found = started.results()

    summary, source = found["ground"]
    assert source == "measured"
    assert summary["episodes"] == float(len(HOLDOUT))
    # A second read gives the same answer and plays nothing, so a run of
    # several strategies pays for one set of games.
    assert not started.measuring
    assert started.results()["ground"][0] == summary


def test_a_cached_baseline_pass_enqueues_nothing(tmp_path: Path) -> None:
    """A run whose inputs match a stored baseline reads it and submits nothing.

    The cache is why a repeated run pays nothing for its bar, and the overlap
    must not cost that. A pass that finds every number measures none of them,
    and it is finished the moment it starts.
    """
    probe = Env(CONTROLLER, WEIGHTING)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    cache = BaselineCache(directory=tmp_path, engine_key="a probe build")

    def started(pool: ShardPool) -> object:
        """Start one pass over the same inputs, against the same cache."""
        return start_controller_baselines(
            CONTROLLER,
            {"ground": WEIGHTING},
            zeros,
            HOLDOUT,
            1,
            SCHEMA,
            "probe baseline",
            cache,
            pool,
        )

    with ShardPool(2) as pool:
        first = started(pool)
        measured, source = first.results()["ground"]  # type: ignore[attr-defined]
        assert source == "measured"

        second = started(pool)
        assert not second.measuring  # type: ignore[attr-defined]
        assert second.done  # type: ignore[attr-defined]
        stored, again = second.results()["ground"]  # type: ignore[attr-defined]

    assert again == "cached"
    assert stored == measured


def a_judge(pool: ShardPool | None) -> Validator:
    """Return one judge over the small world, holding this queue or none."""
    probe = Env(CONTROLLER, WEIGHTING)
    zeros = LinearPolicy.zeros(probe.action_length, probe.observation_length)
    return Validator(
        name="bar",
        env_config=SMALL,
        scoring=WEIGHTING,
        workers=1,
        seeds=list(VALIDATION),
        best_policy=zeros,
        pool=pool,
    )


def test_a_judge_with_a_queue_starts_the_yardstick_and_does_not_hold_it() -> None:
    """The bar is submitted, and the judge returns without the answer.

    **Nothing that trains a weight reads the yardstick.** It reaches a report
    row and a printed line, so the judge submits the pass and takes the
    answer later. A judge that had the answer when this call returned would
    have waited for every episode of it, and a run of 128 validation seeds
    waits minutes there with the rest of the queue empty.
    """
    with ShardPool(3) as pool:
        judge = a_judge(pool)
        judge.measure_controller()

        assert judge.yardstick is None
        assert judge.started_yardstick is not None

        judge.collect_controller(wait=True)
        assert judge.yardstick is not None
        assert judge.started_yardstick is None
        queued = judge.yardstick

    waiting = a_judge(None)
    waiting.measure_controller()

    assert waiting.yardstick is not None
    assert queued.mean == waiting.yardstick.mean
    assert queued.won == waiting.yardstick.won


def test_a_run_with_a_queue_still_reports_its_yardstick(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A run states its bar before it ends, however the pass was played.

    The loop takes the answer whatever state the pass is in when the last
    generation ends, so a run that a wall clock cap stopped early still
    leaves the bar behind. This drives the run, rather than the judge
    underneath it.
    """
    train(
        "bar",
        SMALL,
        WEIGHTING,
        TrainConfig(
            generations=1,
            population=4,
            seeds_per_generation=1,
            workers=1,
            pool=3,
            seed=0,
        ),
        tmp_path,
        POOL,
        validation=VALIDATION,
        validate_every=1,
        holdout=[],
    )

    assert "bar controller yardstick" in capsys.readouterr().out
