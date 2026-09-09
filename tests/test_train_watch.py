"""The watch screen must never report a figure the log does not support.

An owner watches a paid training run on this screen and decides from it
whether to keep paying. Three figures on it were wrong at once, and each
test below fixes one of them.

The fixtures hold real trainer line shapes, and they hold them at the values
that expose the defect. A fixture built from a healthy moment of a real run
supplies no finished pass and no silent worker, so it would measure the
fixture rather than the reader.[^1]

No test here reads a clock. The screen takes every fact about the machine
from its caller, so a test hands those facts in and the reading of a stored
log stays fixed at any hour.[^2]

# References

[^1]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^2]: Testing Rules, do not assert on time. `.agents/rules/testing.md`
"""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path
from types import ModuleType

import pytest

ROOT = Path(__file__).resolve().parent.parent
LOGS = Path(__file__).resolve().parent / "fixtures" / "train-logs"


def _load(name: str) -> ModuleType:
    """Import a script by path, because `scripts/` is not a package."""
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / f"{name}.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


watch = _load("train_watch")
progress_module = _load("train_progress")

# Four strategies measure the controller baseline, and all four finish it.
# Every one leaves its last heartbeat in the log at 1288 ticks a second.
BASELINE_DONE = (LOGS / "watch-baseline-done.log").read_text(encoding="utf-8")

# Four strategies train. Three heartbeat at 1000, 2000 and 3000 ticks a
# second. The fourth said 9999 once and then stopped speaking.
MIXED = (LOGS / "watch-mixed.log").read_text(encoding="utf-8")

ESCAPE = re.compile(r"\x1b\[[0-9;]*m")


def facts(**changed: object) -> object:
    """Return the machine facts a caller brings, with these fields changed."""
    base = {
        "heading": "run-1 c7g.16xlarge",
        "link": "live",
        "asked": 0.0,
        "log_quiet": 4.0,
        "elapsed": 3600.0,
        "price": 1.0,
        "generations": 40,
        "load": 48.0,
        "cores": 64,
    }
    base.update(changed)
    return watch.Facts(**base)  # type: ignore[arg-type]


def screen(text: str, **changed: object) -> str:
    """Return the rendered screen of this log, in plain text."""
    return watch.render(watch.read(text), facts(**changed))


def test_a_finished_baseline_pass_is_not_a_live_process() -> None:
    """A frozen last line must not count as work, and must not add a rate.

    This is the defect exactly. Four baseline passes had ended, and each one
    left its last heartbeat in the log. The screen summed those four frozen
    rates and reported 5152 ticks a second across four live processes, on a
    machine where nothing was playing a world.
    """
    reading = watch.read(BASELINE_DONE)
    assert sorted(reading.strategies) == ["alpha", "beta", "delta", "gamma"]
    rendered = screen(BASELINE_DONE)
    assert "0 ticks/s across 0 working processes" in rendered
    assert "5152" not in rendered
    assert "4 working" not in rendered
    for name in ("alpha", "beta", "gamma", "delta"):
        assert f"{name:<19}" in rendered
    assert rendered.count("ended baseline, nothing started since") == 4


def test_a_strategy_that_stopped_speaking_leaves_the_machine_rate() -> None:
    """A quiet worker is named as quiet, and its frozen rate is dropped.

    Three strategies heartbeat at 1000, 2000 and 3000 ticks a second, so the
    machine reached 6000. The fourth claimed 9999 and then said nothing for
    four whole rounds of the others. Its frozen figure stays on its own row,
    marked as the last thing it said, and never reaches the machine rate.
    """
    rendered = screen(MIXED)
    assert "6000 ticks/s across 3 working processes" in rendered
    assert "15999" not in rendered
    quiet_row = next(row for row in rendered.splitlines() if row.startswith("  delta"))
    assert "QUIET, last said" in quiet_row
    assert "9999t/s" in quiet_row

    reading = watch.read(MIXED)
    states = {
        name: watch.state_of(reading, strategy, 4.0)
        for name, strategy in reading.strategies.items()
    }
    assert states == {
        "alpha": "working",
        "beta": "working",
        "gamma": "working",
        "delta": "quiet",
    }


def test_a_quiet_log_makes_every_strategy_quiet() -> None:
    """A run that wrote nothing for minutes has no working process.

    The caller measures the age of the newest line on the instance, so this
    needs no clock here.
    """
    rendered = screen(MIXED, log_quiet=600.0)
    assert "0 ticks/s across 0 working processes" in rendered
    assert "the run last wrote 600s ago" in rendered


def test_each_strategy_is_shown_against_its_own_yardstick() -> None:
    """Above the bar must be visible without reading the whole screen.

    The bar is the return the built-in controller took on the validation
    seeds, and the trainer prints one for each strategy. Two strategies of a
    real run were above their own bar, and the screen did not say so where a
    reader looks.
    """
    reading = watch.read(MIXED)
    assert reading.strategies["alpha"].yardstick == pytest.approx(1000.0)
    assert reading.strategies["alpha"].margin == pytest.approx(200.4)
    assert reading.strategies["beta"].margin == pytest.approx(-499.1)
    assert reading.strategies["gamma"].margin is None

    rendered = screen(MIXED)
    assert "+200" in rendered
    assert "-499" in rendered
    assert "above the bar 1000 by 200" in rendered
    assert "below the bar 2000 by 499" in rendered
    assert "above the bar 4000 by 400" in rendered


def test_no_validation_win_share_is_invented() -> None:
    """Validation publishes a return only, so the screen shows a return only.

    The trainer prints no win share for the validation seeds. A screen that
    printed one would be printing a figure it derived from nothing.
    """
    rendered = screen(MIXED)
    assert "validation wins" not in rendered
    assert "valid won" not in rendered


def test_the_training_seed_columns_are_marked_as_not_comparable() -> None:
    """The seed set moves every generation, so those columns are not a curve.

    Every strategy of one real run rose to the same win share at one
    generation and fell together at the next. That was the seed set moving.
    Two readers took it for learning.
    """
    rendered = screen(MIXED)
    assert "mean*" in rendered
    assert "best*" in rendered
    assert "spread*" in rendered
    assert "won*" in rendered
    assert "does not compare across them" in rendered
    assert "the validation seeds, which never move" in rendered


def test_the_screen_names_the_phase() -> None:
    """A reader must be able to see which phase the run is in."""
    assert "training generation 2" in screen(MIXED)

    baseline = BASELINE_DONE.split("  alpha controller measured")[0]
    assert "measuring the controller baseline" in screen(baseline)

    assert "building the engine" in screen("")
    probe = "  measuring 1 process(es) of 10 workers over 512 worlds each\n"
    assert "measuring the ticks a second" in screen(probe)


def test_the_three_reasons_for_stale_data_read_differently() -> None:
    """Not asking, asking and getting nothing, and a lost instance differ.

    These are spot instances, so a reclaim is a real outcome. A screen that
    said the same thing for all three trained its reader to ignore it.
    """
    quiet_asked = screen(MIXED, link="cached", asked=3.0)
    assert "no read due yet" in quiet_asked
    assert "gone" not in quiet_asked

    silent = screen(MIXED, link="silent", asked=22.0)
    assert "the instance did not answer" in silent
    assert "22s old" in silent
    assert "the instance is gone" not in silent

    lost = screen(MIXED, link="gone", asked=95.0)
    assert "the instance is gone" in lost

    ended = screen(MIXED, link="local")
    assert "the run ended" in ended
    assert "did not answer" not in ended


def test_the_spend_follows_the_wall_clock_and_not_the_generation_count() -> None:
    """A run spends money before its first generation ends.

    One reader summed the per-generation seconds of every strategy, so it
    read zero for the whole engine build and the whole baseline phase, and
    reported that the run had spent nothing.
    """
    baseline = screen(BASELINE_DONE, elapsed=1800.0, price=2.0)
    assert "elapsed 0h30m" in baseline
    assert "spent $1.00" in baseline
    assert "generations 0/40" in baseline

    mixed = screen(MIXED, elapsed=3600.0, price=1.0)
    assert "elapsed 1h00m" in mixed
    assert "spent $1.00" in mixed
    assert "generations 8/40" in mixed


def test_the_two_readers_of_a_log_agree_about_what_it_says() -> None:
    """Two readers parse one log, so a check must fail when they disagree.

    The progress feed and the watch screen each hold a pattern for the
    generation line. One of them fixed the order of the fields and stopped
    matching every real log, and nothing failed, because a reader that finds
    no generation reports a run with no generation.
    """
    feed = {
        strategy.name: [
            (row.generation, row.mean, row.best, row.spread, row.won, row.validation)
            for row in strategy.generations
        ]
        for strategy in progress_module.parse(MIXED).strategies
    }
    seen = {
        name: [
            (row.generation, row.mean, row.best, row.spread, row.won, row.validation)
            for row in strategy.done
        ]
        for name, strategy in watch.read(MIXED).strategies.items()
    }
    assert feed == seen
    assert sum(len(rows) for rows in feed.values()) == 8


def test_the_progress_feed_reads_the_same_wall_clock() -> None:
    """The two readers of a run must not disagree about the elapsed time.

    Four strategies of the fixture each reached 1200 cumulative seconds. They
    ran at the same time, so the run took 1200 seconds and not 4800. A run
    with no finished generation still spends money, so the wall clock the
    caller measures wins over anything the log holds.
    """
    parsed = progress_module.parse(MIXED)
    assert len(parsed.strategies) == 4
    assert parsed.elapsed_seconds == pytest.approx(1200.0)

    parsed.wall_clock_seconds = 3600.0
    assert parsed.elapsed_seconds == pytest.approx(3600.0)
    money = progress_module.projection(parsed, 1.0, 40)
    assert money["dollars_spent"] == pytest.approx(1.0)

    early = progress_module.parse(BASELINE_DONE)
    early.wall_clock_seconds = 1800.0
    assert progress_module.projection(early, 2.0, 40)["dollars_spent"] == pytest.approx(
        1.0
    )


def test_the_load_average_is_shown_against_the_core_count() -> None:
    """A busy box and an idle box must not look the same."""
    rendered = screen(MIXED, load=48.0, cores=64)
    assert "load 48.0 of 64 cores (75%)" in rendered
    assert "load" not in screen(MIXED, load=-1.0, cores=0).splitlines()[2]


def test_the_screen_carries_the_same_words_with_and_without_colour() -> None:
    """Colour must add meaning and never be the only carrier of it."""
    reading = watch.read(MIXED)
    plain = watch.render(reading, facts(), 15, watch.Paint(False))
    painted = watch.render(reading, facts(), 15, watch.Paint(True))
    assert "\x1b[" not in plain
    assert "\x1b[" in painted
    assert ESCAPE.sub("", painted) == plain


def test_no_colour_wins_over_every_other_setting(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """A reader who asked for no colour asked last."""

    class Terminal:
        """A stream that claims to be a terminal."""

        def isatty(self) -> bool:
            """Say that this stream is a terminal."""
            return True

    class Pipe:
        """A stream that is not a terminal, which is what `watch` gives."""

        def isatty(self) -> bool:
            """Say that this stream is not a terminal."""
            return False

    monkeypatch.delenv("NO_COLOR", raising=False)
    monkeypatch.delenv("CLICOLOR_FORCE", raising=False)
    assert watch.wants_colour("auto", Terminal()) is True
    assert watch.wants_colour("auto", Pipe()) is False
    assert watch.wants_colour("always", Pipe()) is True
    assert watch.wants_colour("never", Terminal()) is False

    monkeypatch.setenv("CLICOLOR_FORCE", "1")
    assert watch.wants_colour("auto", Pipe()) is True

    monkeypatch.setenv("NO_COLOR", "1")
    assert watch.wants_colour("always", Terminal()) is False
    assert watch.wants_colour("auto", Terminal()) is False


def test_a_trainer_that_gains_a_column_still_parses() -> None:
    """The log is the interface, and the fields are read by name.

    A reader that fixed the order of the fields stopped matching every
    generation line the moment one field appeared between two it knew, and it
    reported an empty run rather than an error.
    """
    text = MIXED.replace("won  0.30 ticks", "won  0.30 novel  1.0 ticks")
    reading = watch.read(text)
    assert len(reading.strategies["alpha"].done) == 2
    assert reading.strategies["alpha"].done[-1].won == pytest.approx(0.30)
