"""Selection measures play, and a published figure says which seeds gave it.

A paid run published four policies that cannot play. Each one takes a single
unit and wanders, and every number the run produced said the policies were
strong. Two defects made that possible.

**The quantity that measures play was computed at the moment of selection and
thrown away.** The population record counted the episodes that ended in a win
and divided by the episode count. The validator reduced the same record to the
mean of the shaped return and never read the win share. So the trainer knew
the win share of every candidate centre and selected on a proxy.[^1]

**The published figure was not a holdout.** It was the maximum over the
validation passes of the run, on the very seeds that chose the centre. The
held-out pass ran only after a strategy ended, and a wall clock cap meant it
never ran at all.

The fixtures below are built to produce the distribution each assertion needs.
The world of the demonstration binary would supply none of it: a policy that
always wins gives one win share, and a test over one win share measures the
fixture.[^2]

**The weighting inverts the shaped signal on purpose.** It pays a negative
weight for held ground and nothing at all for the outcome, so a policy that
takes more ground scores a lower shaped return. That is the only way to build
two centres whose win share and shaped return disagree, and the disagreement is
the case the defect lived in.

References
----------
[^1]: Findings register, FND-707. ``docs/FINDINGS.md``

[^2]: Testing Rules, section 2a, on what a fixture must supply.
``.agents/rules/testing.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.policy import (
    LinearPolicy,
    PolicyFitError,
    RandomPolicy,
    load_policy,
)
from cachette.learn.reward import Weighting
from cachette.learn.rollout import run_population
from cachette.learn.train import (
    Checkpoint,
    TrainConfig,
    Validator,
    shell_policy,
    train,
)

if TYPE_CHECKING:
    from pathlib import Path

    from cachette.learn.search import Trainable

# A world where the untrained centre does not always win. The win share of the
# no-op policy here is neither zero nor one, so a comparison of two centres on
# the win share can come out either way.
MIXED = EnvConfig(
    width=24,
    height=24,
    faction_count=4,
    seat=0,
    tick_limit=400,
    horizon=40,
    decision_interval=10,
)

# A world where two centres reach the same win share. The tie is what the
# shaped return must break, and a fixture whose win shares differ never
# reaches that branch.
TIED = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=40,
    decision_interval=10,
)

# The shaped signal, inverted. Held ground pays a negative weight and no
# outcome pays anything, so a centre that plays better scores lower here.
INVERTED = Weighting(terms={"held_tiles": -1.0}, won=0.0, lost=0.0, drawn=0.0)

# An ordinary weighting, for the tests that only need a run to happen.
PLAIN = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)

TRAIN = TrainConfig(
    generations=2, population=4, seeds_per_generation=1, workers=4, seed=0
)


def a_centre(world: EnvConfig, weights: str) -> LinearPolicy:
    """Return one linear centre of the shape this world publishes.

    The zeros centre scores every action row at zero, so it takes the first
    legal row at every decision. That row is the no-op, which is always
    legal, so the centre emits one action for the whole episode.

    The fixed centre weights the bias entry alone, rising with the row
    number. Its scores therefore do not depend on the observation at all, and
    it holds one preference order over the rows. **The row it prefers is
    often illegal**, so the engine's legality answer makes it emit many
    different actions. That is the shape the four published policies held.

    The drawn centre scores the rows apart from the features, so the row it
    prefers moves with the observation.
    """
    probe = Env(world, INVERTED)
    if weights == "zeros":
        return LinearPolicy.zeros(probe.action_length, probe.observation_length)
    if weights == "fixed":
        rows = np.zeros((probe.action_length, probe.observation_length + 1))
        rows[:, -1] = np.linspace(0.0, 1.0, probe.action_length)
        return LinearPolicy(rows)
    drawn = np.random.default_rng(7).normal(
        size=(probe.action_length, probe.observation_length + 1)
    )
    return LinearPolicy(drawn)


def a_judge(world: EnvConfig, seeds: list[int], start: Trainable) -> Validator:
    """Return the judge of a run over one seed set, choosing nothing yet."""
    return Validator(
        name="t",
        env_config=world,
        scoring=INVERTED,
        workers=4,
        seeds=seeds,
        best_policy=start,
    )


def test_a_higher_win_share_selects_over_a_higher_shaped_return() -> None:
    """The centre that wins more is kept, whatever the shaped return says.

    This is the test the trainer failed. The drawn centre wins more games and
    scores a lower shaped return under the inverted weighting. The zeros
    centre is the reverse. Selection on the mean shaped return keeps the
    zeros centre and publishes a policy that plays worse.
    """
    seeds = viable_seeds(MIXED, 8, 20_000)
    zeros = a_centre(MIXED, "zeros")
    drawn = a_centre(MIXED, "drawn")

    quiet = a_judge(MIXED, seeds, zeros)
    on_zeros = quiet.score(zeros, "")
    on_drawn = quiet.score(drawn, "")
    assert on_drawn.won > on_zeros.won, "the fixture gives no win share to compare"
    assert on_drawn.mean < on_zeros.mean, "the fixture gives no disagreement"

    judge = a_judge(MIXED, seeds, zeros)
    judge.check(zeros, 0)
    judge.check(drawn, 1)
    assert judge.best_generation == 1, "the judge kept the centre that wins less"
    assert judge.best is not None
    assert judge.best.won == pytest.approx(on_drawn.won)


def test_an_equal_win_share_falls_to_the_shaped_return() -> None:
    """A tie on the win share is broken by the mean shaped return.

    The order of the two passes must not decide the answer, so this plays
    them both ways round. A judge that only compared the win share would keep
    whichever centre it saw first.
    """
    seeds = viable_seeds(TIED, 8, 20_000)
    zeros = a_centre(TIED, "zeros")
    drawn = a_centre(TIED, "drawn")

    quiet = a_judge(TIED, seeds, zeros)
    on_zeros = quiet.score(zeros, "")
    on_drawn = quiet.score(drawn, "")
    assert on_drawn.won == pytest.approx(on_zeros.won), "the fixture holds no tie"
    assert on_drawn.mean > on_zeros.mean, "the fixture gives no return to break it"

    rising = a_judge(TIED, seeds, zeros)
    rising.check(zeros, 0)
    rising.check(drawn, 1)
    assert rising.best_generation == 1, "the tie did not fall to the higher return"

    falling = a_judge(TIED, seeds, drawn)
    falling.check(drawn, 0)
    falling.check(zeros, 1)
    assert falling.best_generation == 0, "the tie fell to the later pass"


def test_the_most_common_action_share_reads_one_for_one_row() -> None:
    """A policy that answers one row reads one, and another reads less.

    The zeros centre takes the first legal row at every decision, so every
    decision it takes falls on one action. The random policy draws a legal
    row at each decision, so its share is far below one.
    """
    seeds = viable_seeds(MIXED, 4, 20_000)
    zeros = a_centre(MIXED, "zeros")
    one_row = run_population(MIXED, INVERTED, [zeros], seeds, 4)
    assert one_row.most_common_share == pytest.approx(1.0)

    drawing = run_population(MIXED, INVERTED, [RandomPolicy(seed=1)], seeds, 4)
    assert drawing.most_common_share < 1.0


def test_the_unmasked_argmax_reports_no_change_for_a_fixed_order() -> None:
    """A fixed preference order never changes inside an episode.

    **The mask is out of this reading, and that is the whole point.** The
    fixed centre emits many different actions, because the engine's legality
    answer removes the rows it cannot take. Its emitted actions therefore
    look situational, and the share of its most common action is well below
    one. The row it prefers over the unmasked scores never moves.

    The two assertions together are what reach the case. A fixture whose
    emitted actions did not vary would pass a reading taken from the masked
    choice, so it would measure nothing.
    """
    seeds = viable_seeds(MIXED, 4, 20_000)
    fixed = run_population(MIXED, INVERTED, [a_centre(MIXED, "fixed")], seeds, 4)
    assert fixed.chosen > 0, "the fixture took no decision to read"
    assert fixed.most_common_share < 1.0, (
        "the fixture emits one action, so a reading taken after the mask "
        "would look like a policy too"
    )
    assert fixed.preference_varies == pytest.approx(0.0)

    moving = run_population(MIXED, INVERTED, [a_centre(MIXED, "drawn")], seeds, 4)
    assert moving.preference_varies > 0.0


def test_the_generation_line_names_both_instruments(
    tmp_path: Path, capsys: pytest.CaptureFixture[str]
) -> None:
    """A run reports the two instruments on the line a dashboard reads.

    An instrument nothing prints is an instrument nobody reads. This drives a
    whole run and reads what it printed.
    """
    pool = viable_seeds(TIED, 4, 900)
    train(
        "t",
        TIED,
        PLAIN,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=viable_seeds(TIED, 2, 20_000),
        validate_every=1,
    )
    printed = capsys.readouterr().out
    assert "top-share" in printed
    assert "varies" in printed
    assert "valid-won" in printed


def test_the_held_out_pass_runs_at_its_interval(tmp_path: Path) -> None:
    """A run cut short still leaves a held-out figure behind.

    The run below never reaches an end. It stops at the generation count it
    was given, in the way a wall clock cap stops a paid run, and the held-out
    pass at the interval is the only honest figure it can leave.
    """
    pool = viable_seeds(TIED, 4, 900)
    result = train(
        "t",
        TIED,
        PLAIN,
        TrainConfig(**{**vars(TRAIN), "generations": 2}),
        tmp_path,
        pool,
        validation=viable_seeds(TIED, 2, 20_000),
        validate_every=1,
        holdout=viable_seeds(TIED, 2, 50_000),
        holdout_every=1,
    )
    assert result["held_out_won"] is not None, "the run left no held-out win share"
    assert result["held_out_return"] is not None
    assert result["held_out_generation"] == 1
    measured = [row["holdout_won"] for row in result["history"]]
    assert all(value is not None for value in measured), (
        f"a generation took no held-out pass at an interval of one: {measured}"
    )


def test_a_run_that_never_reaches_the_interval_holds_no_held_out_figure(
    tmp_path: Path,
) -> None:
    """The held-out entries stay empty rather than holding a selection figure.

    This is the state the paid run ended in. A result that filled the
    held-out entries from the validation pass would state a measurement that
    nothing measured.
    """
    pool = viable_seeds(TIED, 4, 900)
    result = train(
        "t",
        TIED,
        PLAIN,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=viable_seeds(TIED, 2, 20_000),
        validate_every=1,
        holdout=viable_seeds(TIED, 2, 50_000),
        holdout_every=5,
    )
    assert result["held_out_won"] is None
    assert result["held_out_return"] is None
    assert result["best_selection_won"] is not None


def test_a_manifest_names_which_figure_selected_and_which_did_not(
    tmp_path: Path,
) -> None:
    """The weight file says which seeds gave each figure it carries.

    A file used to carry one entry called the validation score. A manifest
    built from it was published as a mean shaped return over held-out seeds,
    which is the opposite of what the number was.
    """
    pool = viable_seeds(TIED, 4, 900)
    result = train(
        "t",
        TIED,
        PLAIN,
        TrainConfig(**{**vars(TRAIN), "generations": 2}),
        tmp_path,
        pool,
        validation=viable_seeds(TIED, 2, 20_000),
        validate_every=1,
        holdout=viable_seeds(TIED, 2, 50_000),
        holdout_every=1,
    )
    _, meta = load_policy(tmp_path / "t.npz")
    assert "validation_score" not in meta, "the file still names an unqualified figure"
    assert "best_score" not in meta
    for key in (
        "selection_won",
        "selection_return",
        "best_selection_won",
        "best_selection_return",
        "held_out_won",
        "held_out_return",
    ):
        assert key in meta, f"the file names no {key}"
        assert isinstance(meta[key], float)
    assert meta["best_selection_won"] == pytest.approx(result["best_selection_won"])
    assert math.isfinite(float(meta["held_out_won"]))  # type: ignore[arg-type]
    assert meta["held_out_episodes"] == 2


def test_a_resume_refuses_a_checkpoint_that_states_no_normalizer(
    tmp_path: Path,
) -> None:
    """A centre trained through no transform is not read through a new one.

    The reader takes a file that states no normalizer, so a policy published
    before the normalizer existed still plays. A resume must not take the
    same door: every weight of such a centre scores a plain squash, and every
    weight this run trains scores a standardized feature.

    This writes the resume point through the writer a run uses, with no
    normalizer, and then asks a run to resume it.
    """
    probe = Env(TIED, PLAIN)
    before = Checkpoint(
        name="t",
        out_dir=tmp_path,
        env_config=TIED,
        probe=probe,
        kind="linear",
        normalizer=None,
    )
    shell = shell_policy("linear", probe, None)
    before.write(shell, before.latest_path, 0, 0.0, None, None)
    assert before.latest_path.exists()

    with pytest.raises(PolicyFitError, match="states no feature normalizer"):
        train(
            "t",
            TIED,
            PLAIN,
            TrainConfig(**{**vars(TRAIN), "generations": 1}),
            tmp_path,
            viable_seeds(TIED, 4, 900),
            validation=[],
            resume=True,
        )
