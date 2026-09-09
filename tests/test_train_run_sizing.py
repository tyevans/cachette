"""The launcher must size a run against the cores a strategy receives.

A paid run rents one machine and bills for every minute it lives. One run
asked for twenty generations, and the four centres it published came from
generations 3, 7, 7 and 9. The wall clock cap ended it, and nothing had said
that it would.

Two defects caused that. The launcher sized the run against the cores of the
instance, and a run gives each strategy the cores divided by the strategy
count. The launcher also called the validation passes nearly free, on a
comparison that contradicted the episode counts printed beside it.

Every test below drives the trainer through the flag the launcher calls, so
each one measures the answer a paid run would receive.

# References

[^1]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

from cachette.learn.sizing import (
    Episodes,
    RunShape,
    episodes,
    generations_inside,
    plan_of,
    seconds_for,
    workers_for_each_strategy,
)

ROOT = Path(__file__).resolve().parents[1]
LAUNCHER = ROOT / "scripts" / "graviton-train.sh"

# The shape of the run that published the four policies. It asked for twenty
# generations of twenty-four candidates over six seeds, it validated every
# second generation over one hundred and twenty-eight seeds, and it held out
# two hundred and fifty-six.
#
# **This is the fixture that exposes both defects.** A shape of one strategy
# never divides the machine, and a shape with no validation interval plays no
# measurement pass, so either one would measure the fixture.
PAID_RUN = RunShape(
    generations=20,
    population=24,
    seeds=6,
    validation=128,
    validate_every=2,
    holdout=256,
    holdout_every=5,
    tick_limit=6000,
    strategies=4,
)


def _plan(*arguments: str) -> dict[str, str]:
    """Run the flag the launcher calls, and return the fields it printed."""
    finished = subprocess.run(
        [sys.executable, "-m", "cachette.learn", "--print-plan", *arguments],
        capture_output=True,
        text=True,
        check=True,
        cwd=ROOT,
    )
    fields: dict[str, str] = {}
    for line in finished.stdout.splitlines():
        name, _, value = line.partition("\t")
        if value:
            fields[name] = value
    return fields


def test_a_strategy_receives_the_cores_divided_by_the_strategy_count() -> None:
    """Four strategies on sixty-four cores hold sixteen workers each."""
    assert workers_for_each_strategy(64, 4) == 16
    assert workers_for_each_strategy(64, 1) == 64
    # A machine with fewer cores than strategies still gives each one a
    # worker, because a process of no workers plays nothing.
    assert workers_for_each_strategy(2, 4) == 1


def test_the_estimate_reads_the_workers_a_strategy_holds() -> None:
    """The estimate must move with the strategy count and not with the cores.

    This is the defect. The reasoning that sized a run read sixty-four
    workers while each of four strategies held sixteen, so every estimate the
    launcher made was four times too fast.
    """
    alone = plan_of(RunShape(**{**vars(PAID_RUN), "strategies": 1}), 64, 0.0)
    shared = plan_of(PAID_RUN, 64, 0.0)

    assert alone.workers == 64
    assert shared.workers == 16

    # The same episodes at a quarter of the workers take four times as long.
    # The two plans above hold different episode counts, because one shared
    # baseline pass divides between the strategies, so this reads the time of
    # one shape at the two worker counts.
    whole = seconds_for(PAID_RUN, 64)
    quarter = seconds_for(PAID_RUN, 16)
    assert quarter == pytest.approx(whole * 4.0)
    assert shared.seconds == pytest.approx(quarter)


def test_a_run_that_cannot_finish_says_so_before_it_starts() -> None:
    """The plan of the paid run reports that its cap ends it."""
    fields = _plan(
        "--generations",
        "20",
        "--population",
        "24",
        "--seeds",
        "6",
        "--holdout",
        "256",
        "--validation",
        "128",
        "--validate-every",
        "2",
        "--styles",
        "aggressive,defensive_expansionist,population_growth,wonder_rush",
        "--style-kind",
        "structured",
        "--cores",
        "64",
        "--wall-minutes",
        "360",
    )

    assert fields["strategies"] == "4"
    assert fields["workers_each"] == "16"
    assert fields["fits"] == "no", (
        "the run that delivered nine of twenty generations reports that it "
        f"fits its cap: {fields}"
    )
    assert int(fields["generations_reached"]) < int(fields["generations_asked"])


def test_a_run_that_fits_says_so() -> None:
    """A short run under the same cap must not raise the warning.

    **A test that only reads the failing side measures nothing.** A plan that
    answered `no` for every configuration would pass the test above.
    """
    fields = _plan(
        "--generations",
        "2",
        "--population",
        "4",
        "--seeds",
        "2",
        "--holdout",
        "4",
        "--validation",
        "4",
        "--validate-every",
        "2",
        "--only",
        "conquer",
        "--cores",
        "64",
        "--wall-minutes",
        "360",
    )

    assert fields["fits"] == "yes"
    assert fields["generations_reached"] == fields["generations_asked"]


def test_a_third_of_the_episodes_of_the_paid_run_measure_rather_than_train() -> None:
    """The measurement share is the figure the launcher called nearly free.

    The launcher stated that a validation pass costs about a fifth of a
    generation. A generation of this shape plays 144 worlds and a validation
    pass plays 128, so a validation pass costs about nine tenths of one.

    The share is above a third and not near it, because the run now takes a
    periodic held-out pass as well.
    """
    played = episodes(PAID_RUN)

    assert played.training == 20 * 24 * 6
    # Ten validation passes, because every second generation validates and
    # the last one always does.
    assert played.validation == 10 * 128
    # Four held-out passes, at generations 4, 9, 14 and 19.
    assert played.holdout_during == 4 * 256
    assert played.measurement > played.training // 2
    assert played.measurement_share > 1.0 / 3.0


def test_a_validation_pass_costs_about_as_much_as_a_generation() -> None:
    """The comparison the launcher made must hold in worlds, not in prose."""
    generation_worlds = PAID_RUN.population * PAID_RUN.seeds

    assert PAID_RUN.validation / generation_worlds > 0.8
    assert PAID_RUN.validation / generation_worlds < 1.0


def test_the_shared_baseline_pass_is_charged_once_and_not_once_a_strategy() -> None:
    """One pass plays the held-out seeds for every strategy of a run.

    It scores one batch of episodes under every objective the run trains, so
    a count of the whole pass for each strategy would charge a run of four
    strategies four times for it.
    """
    alone = episodes(RunShape(**{**vars(PAID_RUN), "strategies": 1}))
    shared = episodes(PAID_RUN)

    assert alone.baseline == PAID_RUN.holdout
    assert shared.baseline == PAID_RUN.holdout // 4


def test_a_run_with_no_measurement_pass_plays_only_the_final_ones() -> None:
    """A shape that measures nothing periodically still measures at the end.

    **A fixture that models the typical case supplies no extreme.** A shape
    with both intervals off is the extreme that says which counts come from
    the schedule and which come from the end of a strategy.
    """
    quiet = RunShape(
        generations=4,
        population=4,
        seeds=2,
        validation=0,
        validate_every=0,
        holdout=8,
        holdout_every=0,
        tick_limit=100,
        strategies=1,
    )
    played = episodes(quiet)

    assert played.validation == 0
    assert played.yardstick == 0
    assert played.holdout_during == 0
    assert played.holdout_final == 5 * 8
    assert played.baseline == 8


def test_the_generations_a_cap_affords_rise_with_the_cap() -> None:
    """A longer cap buys more generations, and a short one buys none."""
    workers = workers_for_each_strategy(64, PAID_RUN.strategies)
    whole = seconds_for(PAID_RUN, workers)

    assert generations_inside(PAID_RUN, workers, whole) == PAID_RUN.generations
    assert generations_inside(PAID_RUN, workers, whole / 2.0) < PAID_RUN.generations
    assert generations_inside(PAID_RUN, workers, 1.0) == 0


def test_the_plan_reaches_the_launcher_before_the_confirmation() -> None:
    """The launcher must print the plan above the prompt that spends money.

    A warning under the prompt is a warning nobody reads before typing yes.
    """
    script = LAUNCHER.read_text(encoding="utf-8")

    assert "--print-plan" in script, "the launcher does not ask the trainer for a plan"
    assert "DOES NOT FINISH" in script, "the launcher states no verdict on the cap"
    assert script.index("size_warning=") < script.index("Type yes to spend"), (
        "the launcher builds its warning after the confirmation prompt"
    )
    assert "$size_warning" in script, "the launcher builds a warning it never prints"


def test_the_launcher_holds_no_copy_of_a_trainer_default() -> None:
    """The launcher reads the plan, and restates no interval of its own.

    Each expression below read an interval out of the launcher's own argument
    string, with a fallback for the case it found nothing. Every fallback was
    a second copy of a default the trainer owns, and two of them disagreed
    with it.
    """
    script = LAUNCHER.read_text(encoding="utf-8")

    for gone in (
        "${validation:-6}",
        "${validate_every:-3}",
        "${generations:-20}",
        "${population:-24}",
        "${probe_seeds:-6}",
    ):
        assert gone not in script, f"the launcher still holds the fallback {gone}"


def test_the_episode_counts_answer_a_shorter_run() -> None:
    """A run cut at a generation reports what it had played by then."""
    whole = episodes(PAID_RUN)
    part = episodes(PAID_RUN, 9)

    assert isinstance(part, Episodes)
    assert part.training < whole.training
    assert part.training == 9 * 24 * 6
