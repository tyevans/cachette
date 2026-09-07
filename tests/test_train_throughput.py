"""The throughput probe must count ticks exactly and divide them by the price.

Every throughput figure this project holds comes from a development machine
on x86-64. A figure taken on the target platform is worth having only if the
count behind it is exact, so the tests below fix the count, the figure for
each worker, and the cost of a million ticks.

The probe drives the real batch, so the first test starts an engine. It runs
one small world for a few decisions, which costs about a second.
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


throughput = _load("train_throughput")


def test_the_ticks_a_second_divide_by_the_workers() -> None:
    """The figure for each worker is what says whether the work divides."""
    row = throughput.Measurement(
        workers=64, worlds=64, decisions=20, ticks=12_800, seconds=2.0
    )
    assert row.ticks_per_second == pytest.approx(6400.0)
    assert row.ticks_per_second_per_worker == pytest.approx(100.0)


def test_the_cost_of_a_million_ticks_follows_the_price_and_the_rate() -> None:
    """Dollars an hour compares two prices. This compares two machines."""
    row = throughput.Measurement(
        workers=64, worlds=64, decisions=20, ticks=12_800, seconds=2.0
    )
    # A million ticks at 6400 a second takes 156.25 seconds, and the machine
    # bills that fraction of an hour.
    assert row.dollars_per_million_ticks(0.7628) == pytest.approx(
        0.7628 / 3600.0 * (1_000_000.0 / 6400.0)
    )


def test_a_measurement_with_no_time_reports_no_rate() -> None:
    """A probe that took no time must not divide by zero."""
    row = throughput.Measurement(workers=8, worlds=8, decisions=0, ticks=0, seconds=0.0)
    assert row.ticks_per_second == 0.0
    assert row.dollars_per_million_ticks(1.0) == 0.0


def test_the_scaling_line_names_a_bottleneck_and_names_a_clean_divide() -> None:
    """The verdict must be able to come out either way."""
    clean = [
        throughput.Measurement(1, 1, 20, 200, 1.0),
        throughput.Measurement(16, 16, 20, 3200, 1.0),
    ]
    assert "the work divides" in throughput.scaling(clean)

    # The same worker counts, but the larger one reached only twice the
    # throughput rather than sixteen times it.
    stalled = [
        throughput.Measurement(1, 1, 20, 200, 1.0),
        throughput.Measurement(16, 16, 20, 400, 1.0),
    ]
    assert "bottleneck" in throughput.scaling(stalled)


def test_one_worker_count_claims_nothing_about_scaling() -> None:
    """Silence is not evidence that the work divides."""
    one = [throughput.Measurement(8, 8, 20, 1600, 1.0)]
    assert "Nothing here says" in throughput.scaling(one)


def test_the_table_names_the_world_beside_every_figure() -> None:
    """Ticks a second does not compare across world sizes."""
    config = throughput.EnvConfig(
        width=48,
        height=48,
        faction_count=3,
        seat=0,
        tick_limit=2500,
        horizon=250,
        decision_interval=10,
    )
    rendered = throughput.table(
        [throughput.Measurement(4, 4, 5, 200, 1.0)],
        config,
        0.7628,
        {"machine": "aarch64"},
    )
    assert "# extent\t48x48" in rendered
    assert "# faction_count\t3" in rendered
    assert "ticks_per_second_per_worker" in rendered


def test_the_probe_counts_every_tick_it_ran() -> None:
    """The count is the live world count times the decision interval.

    This drives the real batch rather than a model of it, because the count
    is only worth anything if it matches what the engine actually stepped.
    """
    config = throughput.EnvConfig(
        width=16,
        height=16,
        faction_count=2,
        seat=0,
        tick_limit=600,
        horizon=60,
        decision_interval=10,
    )
    row = throughput.measure(config, worlds=2, workers=2, decisions=3)
    # Three crossings, two worlds still running, ten ticks in each crossing.
    assert row.decisions == 3
    assert row.ticks == 3 * 2 * 10
    assert row.seconds > 0.0
