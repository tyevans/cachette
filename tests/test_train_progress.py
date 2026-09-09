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
    assert "held out   not measured yet" in rendered
    assert "validation -1995.5, which chose the centre" in rendered


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


# A real sharded run, read at the moment every strategy was inside generation
# 1 and none had finished it. Four strategies score each generation in two
# processes, so the trainer prints one heartbeat for each shard.
SHARDED = (LOGS / "sharded-in-flight.log").read_text(encoding="utf-8")
MID_GENERATION = "\n".join(SHARDED.splitlines()[:88]) + "\n"


def test_the_feed_says_what_each_strategy_is_doing_now() -> None:
    """A generation that runs must not read as a strategy that stopped.

    Every strategy of this real log was about seven tenths of the way through
    generation 1 on two processes each. The feed printed the generations that
    had finished and nothing at all about the one in flight, so a reader saw
    the same screen for a healthy run and a hung one.
    """
    rendered = progress_module.render(
        progress_module.parse(MID_GENERATION), 0.7723, 100
    )
    assert rendered.count("in flight  generation 1") == 4
    assert (
        "68% of 1024 worlds, 2 shards, 3478 ticks/s, 463 decisions [401s]" in rendered
    )
    assert "1 generations finished" in rendered


def test_a_pass_in_flight_reaches_no_derived_figure() -> None:
    """A generation with no result must not shorten the estimate of the cost.

    The pattern for a finished generation matches a heartbeat as well,
    because it reads the body by name. A heartbeat counted as a generation
    would report a generation that cost nothing and scored nothing, and
    every figure below divides by that count.
    """
    parsed = progress_module.parse(MID_GENERATION)
    assert parsed.generations_done == 4
    parsed.wall_clock_seconds = 4900.0
    money = progress_module.projection(parsed, 0.7723, 100)
    assert money["generations_done"] == pytest.approx(4.0)
    assert money["generations_remaining"] == pytest.approx(96.0)
    assert money["seconds_per_generation"] == pytest.approx(1225.0)

    # The pattern for a finished generation matches these heartbeats too, so
    # the count above is right only because the marker excluded them first.
    matched = [
        line
        for line in MID_GENERATION.splitlines()
        if progress_module.GENERATION.match(line)
    ]
    beats = [line for line in matched if progress_module.in_flight(line)]
    assert len(beats) == 32
    assert len(matched) - len(beats) == parsed.generations_done


def test_a_finished_generation_ends_the_pass_it_reported() -> None:
    """A frozen heartbeat must not report a pass that already ended.

    The whole log ends with one strategy that finished generation 1 while
    the others were still inside it. That strategy has nothing in flight.
    """
    strategies = {
        strategy.name: strategy
        for strategy in progress_module.parse(SHARDED).strategies
    }
    assert strategies["wonder_rush"].flight is None
    assert strategies["aggressive"].flight is not None
    rendered = progress_module.render(progress_module.parse(SHARDED), 0.7723, 100)
    assert "in flight  nothing, between passes" in rendered


# A real run of two generations that took a held-out pass at every one of
# them. The trainer prints the win share of both passes and the two behaviour
# instruments, and the parser reads each field by name.
WITH_HOLDOUT = (LOGS / "with-holdout.log").read_text(encoding="utf-8")

# The same run, cut before the strategy ended. **This is the state the paid
# run stayed in for its whole life**, because a wall clock cap ended it before
# any strategy returned.
RUNNING = WITH_HOLDOUT.split("  trained")[0]


def test_the_verdict_renders_from_a_held_out_pass_taken_during_the_run() -> None:
    """A run that never finishes a strategy still gets a verdict.

    The dashboard used to print the verdict only from the row a finished
    strategy leaves. A wall clock cap ended a paid run before any strategy
    finished, so the verdict never rendered once and the screen showed a
    shaped return for the whole run.
    """
    rendered = progress_module.render(progress_module.parse(RUNNING), 0.7628, 60)
    assert "verdict    wins 0.500 against the controller's 1.000" in rendered
    assert "loses to the controller" in rendered
    assert "from the held-out seeds at generation 1, which chose nothing" in rendered


def test_the_verdict_names_a_selection_figure_as_one() -> None:
    """A run with no held-out pass yet says which seeds its figure came from.

    The figure is then the win share of the seeds that chose the centre, so
    it is a maximum over the passes of the run. The line must not read as a
    measurement.
    """
    without = RUNNING.replace(
        " holdout     962.3 holdout-won  0.50", " holdout - holdout-won -"
    )
    rendered = progress_module.render(progress_module.parse(without), 0.7628, 60)
    assert (
        "from the validation seeds at generation 1, which chose the centre" in rendered
    )
    assert "which selects" in rendered


def test_the_verdict_can_come_out_the_other_way() -> None:
    """The same comparison must be able to say that the policy beats the bar."""
    winning = RUNNING.replace("holdout-won  0.50", "holdout-won  1.00")
    rendered = progress_module.render(progress_module.parse(winning), 0.7628, 60)
    assert "BEATS the controller" not in rendered, (
        "an equal win share is not a win, so this fixture reaches nothing"
    )
    better = RUNNING.replace("holdout-won  0.50", "holdout-won  1.00").replace(
        "'won': 1.0", "'won': 0.5"
    )
    assert "BEATS the controller" in progress_module.render(
        progress_module.parse(better), 0.7628, 60
    )


def test_the_two_instruments_reach_the_generation_records() -> None:
    """A reader of the table sees the two checks a finding named.

    A policy that answers the same row at every decision has learned a
    preference and not a policy. Both figures are instruments, so nothing
    here fails a run on them.
    """
    rows = progress_module.rows(progress_module.parse(WITH_HOLDOUT), "run-1")
    assert [row["most_common_share"] for row in rows] == [0.5, 0.45]
    assert [row["preference_varies"] for row in rows] == [1.0, 1.0]
    assert [row["holdout_won"] for row in rows] == [0.5, 0.5]
    assert [row["validation_won"] for row in rows] == [0.0, 0.0]


def test_the_holdout_heartbeat_reads_as_a_pass_in_flight() -> None:
    """The held-out pass prints a heartbeat, and the reader must place it.

    A pass whose name the reader does not know is dropped, so the dashboard
    would report a working strategy as one between passes.
    """
    beat = (
        "  conquer holdout  3 working  decisions  12 live    2/2    "
        "ticks     2400 rate   1754.8 t/s [10s]\n"
    )
    progress = progress_module.parse(RUNNING + beat)
    flight = progress.strategies[0].flight
    assert flight is not None
    assert flight.what == "holdout 3"


def test_the_held_out_row_survives_a_generation_that_took_no_pass() -> None:
    """The newest generation is not the newest held-out pass.

    The trainer takes that pass at an interval. A reader that looked only at
    the last row would report no held-out figure for every generation between
    two intervals, which is most of them.
    """
    between = RUNNING.replace(
        "valid    -140.8 valid-won  0.00 top-share  0.45 varies  1.00 "
        "holdout     962.3 holdout-won  0.50",
        "valid    -140.8 valid-won  0.00 top-share  0.45 varies  1.00 "
        "holdout - holdout-won -",
    )
    parsed = progress_module.parse(between)
    rows = parsed.strategies[0].generations
    assert rows[-1].holdout is None, "the fixture still holds a pass on the last row"
    assert rows[0].holdout is not None, "the fixture holds no earlier pass"
    rendered = progress_module.render(parsed, 0.7628, 60)
    assert "at generation 0, during the run" in rendered
    assert "from the held-out seeds at generation 0, which chose nothing" in rendered
