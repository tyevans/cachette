"""The measurement must find a rise where nothing rewards it, and refuse a guess.

The question the tool answers is whether population rises under an objective
that never pays for it. That is a paired comparison against the built-in
controller, so the tests below fix three things: the paired count, the win
path and end tick summary, and the refusal a name that no world publishes
earns.

# The fixture supplies extremes, not a typical world

Every synthetic reading in this file is built by hand, and none of it comes
from the demonstration world. A world chosen to look right supplies no
extreme, so an assertion over it never receives the input that would fail.
The readings here therefore hold, on purpose:

- a seed on which the arm reads exactly the baseline, which is neither above
  nor below and must count as neither
- a seed on which the arm reads zero population, which is the elimination
  case the survival rule names
- a seed the arm played and the baseline did not, which must leave the paired
  count alone
- an end tick set that piles up at the limit and holds one early end, which
  is the shape a mean hides and a quartile shows

One test starts the engine. It plays a small world for a few decisions, so it
costs a second or two rather than the minutes a held-out pass costs.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path
from types import ModuleType

import pytest

ROOT = Path(__file__).resolve().parent.parent


def _load(name: str) -> ModuleType:
    """Import a script by path, because `scripts/` is not a package."""
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


measure = _load("instrumental_population")


def _reading(
    arm: str,
    seed: int,
    population: float,
    held: float = 0.0,
    settlements: float = 1.0,
    path: str = "territory",
    end_tick: int = 2500,
    reached_limit: bool = True,
) -> object:
    """Build one reading by hand, so a test states the extreme it needs."""
    return measure.Reading(
        arm=arm,
        seed=seed,
        outcome="lost",
        decisions=250,
        path=path,
        end_tick=end_tick,
        winner=1,
        reached_limit=reached_limit,
        signals={
            "population": population,
            "settlements": settlements,
            "held_tiles": held,
        },
    )


def test_a_rise_counts_the_seeds_above_and_ignores_a_tie() -> None:
    """A tie is neither above nor below, and it must not be counted as either."""
    baseline = [
        _reading("controller", seed=1, population=100.0),
        _reading("controller", seed=2, population=100.0),
        _reading("controller", seed=3, population=100.0),
    ]
    arm = [
        _reading("conquer", seed=1, population=150.0),
        _reading("conquer", seed=2, population=100.0),
        _reading("conquer", seed=3, population=0.0),
    ]
    rises = {
        rise.signal: rise
        for rise in measure.compare(arm, baseline, ["population"], rewarded=())
    }
    population = rises["population"]
    assert population.paired == 3
    assert population.above == 1
    assert population.above_share == pytest.approx(1.0 / 3.0)
    assert population.arm_mean == pytest.approx(250.0 / 3.0)
    assert population.baseline_mean == pytest.approx(100.0)
    assert population.difference == pytest.approx(250.0 / 3.0 - 100.0)
    assert population.rewarded is False


def test_a_seed_only_one_side_played_leaves_the_paired_count_alone() -> None:
    """The comparison is paired, so an unshared seed cannot enter the count."""
    baseline = [_reading("controller", seed=1, population=10.0)]
    arm = [
        _reading("people", seed=1, population=20.0),
        _reading("people", seed=9, population=999.0),
    ]
    rise = measure.compare(arm, baseline, ["population"], rewarded=("population",))[0]
    assert rise.paired == 1
    assert rise.above == 1
    assert rise.rewarded is True
    assert rise.arm_mean == pytest.approx(509.5)
    assert rise.baseline_mean == pytest.approx(10.0)


def test_the_win_path_and_the_end_tick_come_from_the_readings() -> None:
    """The path names come from the engine, so nothing here may list them."""
    readings = [
        _reading("controller", seed=1, population=1.0, path="domination", end_tick=310),
        _reading("controller", seed=2, population=1.0, path="territory"),
        _reading("controller", seed=3, population=1.0, path="territory"),
        _reading("controller", seed=4, population=1.0, path="territory"),
    ]
    assert measure.path_counts(readings) == {"domination": 1, "territory": 3}
    assert measure.outcome_counts(readings) == {"lost": 4}
    spread = measure.Spread.of([float(row.end_tick) for row in readings])
    assert spread.count == 4
    assert spread.lowest == pytest.approx(310.0)
    assert spread.highest == pytest.approx(2500.0)
    assert spread.middle == pytest.approx(2500.0)
    assert spread.mean == pytest.approx((310.0 + 2500.0 * 3) / 4)


def test_a_game_no_reader_ended_gets_the_absent_path_name() -> None:
    """A game that ran out of ticks holds no record, and needs a column."""
    readings = [_reading("controller", seed=1, population=1.0, path=measure.NO_PATH)]
    assert measure.path_counts(readings) == {measure.NO_PATH: 1}


def test_a_spread_of_nothing_is_refused() -> None:
    """A quartile over an empty set states a number nothing measured."""
    with pytest.raises(ValueError, match="at least one number"):
        measure.Spread.of([])


def test_a_weighting_names_the_quantities_it_pays_for() -> None:
    """The rewarded set comes from the objective, so a term of zero is not in it."""
    from cachette.learn.reward import Weighting

    weighting = Weighting(
        terms={"population": 3.0, "held_tiles": 0.25, "store_total": 0.0},
        won=1.0,
        lost=-1.0,
        drawn=0.0,
    )
    assert measure.rewarded_signals(weighting) == ("population", "held_tiles")


def test_every_strategy_of_the_run_names_a_quantity_the_schema_publishes() -> None:
    """A rewarded name the engine does not publish must fail, not read as zero."""
    from cachette.learn.__main__ import STRATEGIES, WORLD
    from cachette.learn.env import Env
    from cachette.learn.train import first_scoring

    probe = Env(WORLD, measure.probe_scoring())
    for name, (_config, scoring, _kind) in STRATEGIES.items():
        rewarded = measure.rewarded_signals(first_scoring(scoring))
        assert rewarded, f"the strategy {name!r} rewards no named signal"
        assert measure.require_signals(probe.signals, rewarded) == rewarded


def test_a_quantity_the_world_does_not_publish_is_refused_by_name() -> None:
    """The failure this replaces read a missing name as zero for every episode."""
    from cachette.learn.__main__ import WORLD
    from cachette.learn.env import Env

    probe = Env(WORLD, measure.probe_scoring())
    with pytest.raises(KeyError, match="names no signal of this world"):
        measure.require_signals(probe.signals, ["tick"])


def test_a_compound_quantity_is_refused_because_it_states_no_position() -> None:
    """A signal of many positions cannot be one column of this report."""
    from cachette.learn.__main__ import WORLD
    from cachette.learn.env import Env

    probe = Env(WORLD, measure.probe_scoring())
    compound = probe.signals.compound()[0]
    with pytest.raises(ValueError, match="reads one number for each quantity"):
        measure.require_signals(probe.signals, [compound.name])


def test_a_shorter_tick_limit_carries_the_horizon_with_it() -> None:
    """A horizon shorter than the limit truncates, and a truncation reports nothing."""
    from cachette.learn.__main__ import CONTROLLER_WORLD, STRATEGIES, WORLD
    from cachette.learn.train import first_scoring

    strategies = {
        name: (config, first_scoring(scoring), kind)
        for name, (config, scoring, kind) in STRATEGIES.items()
    }
    world, controller, moved = measure.at_tick_limit(
        WORLD, CONTROLLER_WORLD, strategies, 1500
    )
    assert world.tick_limit == 1500
    assert world.horizon == 1500 // world.decision_interval
    assert controller.tick_limit == 1500
    assert controller.controlled is False
    for config, _scoring, _kind in moved.values():
        assert config.tick_limit == 1500
        assert config.horizon == 1500 // config.decision_interval


def test_a_tick_limit_of_nothing_is_refused() -> None:
    """A world of zero ticks measures no game."""
    with pytest.raises(ValueError, match="one tick or more"):
        measure.at_tick_limit(None, None, {}, 0)


def test_a_stored_policy_names_a_row_of_the_strategy_table() -> None:
    """The world of a stored policy comes from the table, never the command line."""
    from cachette.learn.__main__ import STRATEGIES

    found = measure.stored_policies(["people=runs/people.npz"], STRATEGIES)
    assert found == {"people": Path("runs/people.npz")}
    with pytest.raises(ValueError, match="names no strategy"):
        measure.stored_policies(["peeple=runs/x.npz"], STRATEGIES)
    with pytest.raises(ValueError, match="names no path"):
        measure.stored_policies(["people"], STRATEGIES)


def test_the_controller_plays_a_small_world_and_reports_its_end() -> None:
    """The reading path must work end to end, from the engine to the report.

    This drives the real batch. The world is small and the limit is short, so
    the test costs a second or two.
    """
    from cachette.learn.env import EnvConfig, viable_seeds

    config = EnvConfig(
        width=24,
        height=24,
        faction_count=3,
        seat=0,
        tick_limit=200,
        horizon=20,
        decision_interval=10,
        threads=1,
        controlled=False,
    )
    seeds = viable_seeds(config, 2, 1_000)
    arm = measure.Arm(name="controller", config=config, policy=measure.IdleSeat())
    readings = measure.play(arm, seeds, workers=1, scoring=measure.probe_scoring())

    assert [row.seed for row in readings] == seeds
    for row in readings:
        assert row.arm == "controller"
        assert "population" in row.signals
        assert "settlements" in row.signals
        assert "held_tiles" in row.signals
        assert row.end_tick > 0
        assert row.path in {
            measure.NO_PATH,
            "domination",
            "territory",
            "wonder",
            "renown",
        }
        assert (row.path == measure.NO_PATH) == (row.winner is None)

    counts = measure.path_counts(readings)
    assert sum(counts.values()) == len(readings)
    summary = measure.report(
        config, seeds, 1_000, {"controller": readings}, {}, ["population"]
    )
    assert summary["arms"]["controller"]["end_tick"]["count"] == float(len(readings))
    assert summary["seeds"] == seeds
