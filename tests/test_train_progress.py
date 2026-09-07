"""The progress feed must answer the four questions a spending owner asks.

A training run on a rented machine spends money every minute. The feed is
what tells its owner whether to keep paying, so each test below fixes one of
the answers it gives: what the policy must beat, whether the search is still
searching, what it has cost, and how long is left.

The fixtures are real trainer output. A fixture invented here would prove
that the parser reads the fixture, and nothing about whether it reads the
trainer.
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


progress_module = _load("train_progress")

LOGS = Path(__file__).resolve().parent / "fixtures" / "train-logs"

# One strategy that ran to the end, with the controller baseline above it.
FINISHED = (LOGS / "finished.log").read_text(encoding="utf-8")

# The older trainer prints no spread and no validation figure. The feed must
# still read it, because a log is the interface and an old log is still one.
OLD = (LOGS / "no-spread.log").read_text(encoding="utf-8")

# The failure that cost most of a real run: every candidate scored the same,
# so the ranking ranked nothing and the update carried no information.
COLLAPSED = (LOGS / "collapsed.log").read_text(encoding="utf-8")


def test_the_bar_is_the_controller_and_the_feed_names_it() -> None:
    """The controller baseline is the figure that decides the spend."""
    progress = progress_module.parse(FINISHED)
    assert progress.controller["won"] == pytest.approx(0.375)
    rendered = progress_module.render(progress, 0.7628, 60)
    assert "controller wins 0.375" in rendered


def test_a_policy_that_loses_to_the_controller_is_named_as_losing() -> None:
    """A held-out row below the bar must not read as a result."""
    progress = progress_module.parse(FINISHED)
    strategy = progress.strategies[0]
    assert strategy.baselines["trained"]["won"] == pytest.approx(0.21)
    rendered = progress_module.render(progress, 0.7628, 60)
    assert "loses to the controller" in rendered
    assert "BEATS the controller" not in rendered


def test_a_policy_above_the_bar_is_named_as_beating_it() -> None:
    """The same comparison must be able to come out the other way."""
    text = FINISHED.replace(
        "  trained     return    -1153.1 tiles   136.0 won  0.21 lost  0.79",
        "  trained     return     -100.1 tiles   300.0 won  0.62 lost  0.38",
    )
    rendered = progress_module.render(progress_module.parse(text), 0.7628, 60)
    assert "BEATS the controller" in rendered


def test_a_collapsed_search_is_named_and_a_live_one_is_not() -> None:
    """A spread that reached zero must be visible at once, and must not fire early."""
    collapsed = progress_module.parse(COLLAPSED)
    assert collapsed.collapsed is True
    assert "SEARCH STOPPED" in progress_module.render(collapsed, 0.7628, 60)

    healthy = progress_module.parse(FINISHED)
    assert healthy.collapsed is False
    assert "SEARCH STOPPED" not in progress_module.render(healthy, 0.7628, 60)


def test_a_log_with_no_spread_column_never_claims_a_collapse() -> None:
    """An older trainer reports no spread, and silence is not evidence."""
    progress = progress_module.parse(OLD)
    assert progress.strategies[0].generations[0].mean == pytest.approx(256.1)
    assert progress.strategies[0].generations[0].spread is None
    assert progress.collapsed is False


def test_the_money_follows_the_elapsed_time_and_the_price() -> None:
    """The running total is what answers whether to keep paying."""
    progress = progress_module.parse(FINISHED)
    money = progress_module.projection(progress, 0.7628, 60)
    # The last generation ended at 871 seconds, and the price is an hourly
    # rate, so the charge is that fraction of an hour.
    assert money["elapsed_seconds"] == pytest.approx(871.0)
    assert money["dollars_spent"] == pytest.approx(871.0 / 3600.0 * 0.7628)
    assert money["generations_done"] == 3
    assert money["generations_remaining"] == 57
    # The rate comes from the generations already run, so the projection of
    # the whole run is the rate multiplied by the whole count.
    assert money["dollars_total"] == pytest.approx(871.0 / 3.0 * 60.0 / 3600.0 * 0.7628)


def test_the_projection_survives_a_log_with_no_generation() -> None:
    """A run that has not finished a generation must not divide by zero."""
    money = progress_module.projection(progress_module.parse(""), 0.7628, 60)
    assert money["dollars_spent"] == 0.0
    assert money["dollars_per_generation"] == 0.0


def test_the_feed_refuses_to_present_the_mean_as_progress() -> None:
    """The seed set moves every generation, so the mean is not a curve."""
    rendered = progress_module.render(progress_module.parse(FINISHED), 0.7628, 60)
    assert "not a learning curve" in rendered


def test_a_strategy_still_running_says_the_held_out_figure_is_not_in_yet() -> None:
    """A validation figure must never be read as the held-out result."""
    running = FINISHED.split("  trained")[0]
    rendered = progress_module.render(progress_module.parse(running), 0.7628, 60)
    assert "not measured until this strategy ends" in rendered
    assert "validation -1995.5" in rendered


def test_every_generation_becomes_one_record() -> None:
    """A finished run leaves a table, and not only prose."""
    rows = progress_module.rows(progress_module.parse(FINISHED), "run-1")
    assert len(rows) == 3
    assert rows[0]["strategy"] == "conquer"
    assert rows[0]["kind"] == "linear"
    assert rows[1]["validation"] == pytest.approx(-1995.5)
    assert rows[2]["spread"] == pytest.approx(754.1)


def test_an_unreadable_line_is_skipped_rather_than_fatal() -> None:
    """A trainer that gains a column must not break the feed."""
    text = FINISHED + "\n  conquer generation 99 mean nonsense best [1s]\n"
    progress = progress_module.parse(text)
    assert len(progress.strategies[0].generations) == 3


def test_the_parser_reads_a_generation_line_that_carries_the_tick_count() -> None:
    """The trainer prints world-ticks, and the parser must not go blind.

    The parser reads the stdout of the trainer, and that seam has no other
    guard. A field added between two the parser already knew broke every
    generation line in one edit, and the parser reported a run with no
    generations rather than an error.
    """
    text = (LOGS / "with-ticks.log").read_text(encoding="utf-8")
    progress = progress_module.parse(text)
    rows = progress.strategies[0].generations
    assert [row.generation for row in rows] == [0, 1, 2]
    assert [row.ticks for row in rows] == [121248, 118904, 124016]
    # The spread and the validation still arrive beside it.
    assert rows[0].spread == 1350.1
    assert rows[1].validation == -656.7
    assert rows[0].validation is None


def test_a_log_written_before_the_tick_count_still_parses() -> None:
    """An older log names no ticks, and that is not zero work.

    A reader that took a missing count for zero would report a run that ran
    no ticks, which is worse than reporting that it does not know.
    """
    progress = progress_module.parse(FINISHED)
    rows = progress.strategies[0].generations
    assert rows, "the fixture holds no generation"
    assert all(row.ticks is None for row in rows)
