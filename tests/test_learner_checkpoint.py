"""A training run leaves a resume point behind after every generation.

A run of this trainer takes hours. **The claim that it checkpoints every
generation was believed for a whole session and was false.** The trainer
wrote the weights only when a validation pass found a better centre, so a run
with no validation seeds wrote nothing at all until it ended, and a run with
them lost everything since the last improvement.

These tests drive the trainer and then read the file from disk. A test that
asserted the save method was called would have passed against the defect,
because the save method was called, just not often enough.

The world here is small and the generations are few, because a run of the
real size takes hours.[^1]

References
----------
[^1]: Testing Rules, section 2a, on what a fixture must supply.
``.agents/rules/testing.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING

import numpy as np
import pytest

if TYPE_CHECKING:
    from pathlib import Path

from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.policy import load_policy
from cachette.learn.reward import Weighting
from cachette.learn.train import TrainConfig, train

# A world that resolves quickly, so a test pays for a whole game rather than
# a truncated one. The extent is small and the tick limit is short.
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

TRAIN = TrainConfig(
    generations=2, population=4, seeds_per_generation=1, workers=4, seed=0
)


@pytest.fixture
def pool() -> list[int]:
    """Return a training seed pool for the small world."""
    return viable_seeds(WORLD, 6, 900)


def test_the_latest_file_exists_with_no_validation_seeds(
    tmp_path: Path, pool: list[int]
) -> None:
    """A run that validates nothing still leaves a resume point.

    This is the case the defect broke completely. With no validation seeds
    the trainer had no improvement to gate on, so it wrote the weights once,
    after the last generation. A run stopped before then lost everything.
    """
    train("t", WORLD, WEIGHTING, TRAIN, tmp_path, pool, validation=[])
    latest = tmp_path / "t-latest.npz"
    assert latest.exists(), "a run with no validation seeds wrote no resume point"
    _, meta = load_policy(latest)
    assert meta["generation"] == TRAIN.generations - 1
    # The best file is the latest one when nothing can tell centres apart.
    assert (tmp_path / "t.npz").exists()


def test_the_latest_file_names_the_generation_and_the_spread(
    tmp_path: Path, pool: list[int]
) -> None:
    """A file found after a crash can say where it came from.

    The generation places the file. The spread says whether the search still
    had a population to rank when it stopped, which is the signal the first
    full run lost without anyone seeing it.
    """
    train("t", WORLD, WEIGHTING, TRAIN, tmp_path, pool, validation=[])
    _, meta = load_policy(tmp_path / "t-latest.npz")
    assert "generation" in meta
    assert "spread" in meta
    spread = meta["spread"]
    assert isinstance(spread, float)
    assert np.isfinite(spread)
    # The versions still travel, so a stale file fails loudly rather than
    # acting on the wrong columns.
    #
    # **The two numbers come from the engine and not from a literal here.** A
    # test that names the number is a second declaration of it, and it fails
    # for the wrong reason on the tick the engine moves.
    probe = Env(WORLD, WEIGHTING)
    assert meta["action_version"] == probe.action_version
    assert meta["observation_version"] == probe.observation_version


def test_a_resumed_run_continues_rather_than_restarting(
    tmp_path: Path, pool: list[int]
) -> None:
    """Stop after one generation, resume, and the run picks up at the next.

    A resumed run that started again at generation zero would look identical
    from the outside: it would finish, and it would leave a file. The history
    is what separates the two, so this reads it.
    """
    first = train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=[],
    )
    assert [row["generation"] for row in first["history"]] == [0]
    _, meta = load_policy(tmp_path / "t-latest.npz")
    assert meta["generation"] == 0

    second = train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 3}),
        tmp_path,
        pool,
        validation=[],
        resume=True,
    )
    # The resumed run ran generations 1 and 2, and did not repeat 0.
    assert [row["generation"] for row in second["history"]] == [1, 2]
    _, meta = load_policy(tmp_path / "t-latest.npz")
    assert meta["generation"] == 2


def test_a_resumed_run_starts_from_the_stored_centre(
    tmp_path: Path, pool: list[int]
) -> None:
    """The centre a resume reads is the centre the earlier run left."""
    train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=[],
    )
    stopped, _ = load_policy(tmp_path / "t-latest.npz")
    # A resume of zero further generations must return the same weights.
    train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=[],
        resume=True,
    )
    again, _ = load_policy(tmp_path / "t-latest.npz")
    assert np.allclose(np.asarray(stopped.flat()), np.asarray(again.flat()))


def test_the_best_file_and_the_latest_file_are_not_the_same_file(
    tmp_path: Path, pool: list[int]
) -> None:
    """The best centre and the newest centre are different things.

    The first full run stored a centre taken from inside a region where the
    population had collapsed, because the newest centre was the only one it
    kept. The two files must therefore be able to disagree.
    """
    validation = viable_seeds(WORLD, 2, 20_000)
    result = train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 4}),
        tmp_path,
        pool,
        validation=validation,
        validate_every=1,
    )
    assert (tmp_path / "t.npz").exists()
    assert (tmp_path / "t-latest.npz").exists()
    _, best = load_policy(tmp_path / "t.npz")
    _, latest = load_policy(tmp_path / "t-latest.npz")
    assert latest["generation"] == 3
    assert best["generation"] == result["best_generation"]
    # The report names both, so a reader loads the one it means.
    assert result["weights"].endswith("t.npz")
    assert result["latest_weights"].endswith("t-latest.npz")


def test_a_run_with_no_validation_seeds_stores_no_best_score(
    tmp_path: Path, pool: list[int]
) -> None:
    """The file says that nothing chose this centre, rather than scoring it.

    A run with no validation seeds has no way to tell one centre from
    another, so its best file is its latest file and the score entry holds
    the quiet value. This states the input the resume test needs.
    """
    train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=[],
    )
    _, meta = load_policy(tmp_path / "t.npz")
    stored = meta["best_score"]
    assert isinstance(stored, float)
    assert math.isnan(stored), "a run that chose no centre scored one"


def test_a_resume_that_adds_validation_seeds_can_still_choose_a_centre(
    tmp_path: Path, pool: list[int]
) -> None:
    """A resume must not inherit a best score that no score can beat.

    The first run has no validation seeds, so it stores the quiet value for
    the best score. A resume that read that value as it stands would compare
    every later score against it, and no comparison against it is ever true.
    The best centre would never move, and the run would say nothing: it
    prints its generations and writes its checkpoints as usual.

    **The two runs must differ in their validation setting.** A resume that
    keeps the setting of the first run never reaches the case, so it would
    measure the fixture rather than the trainer.
    """
    train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 1}),
        tmp_path,
        pool,
        validation=[],
    )
    result = train(
        "t",
        WORLD,
        WEIGHTING,
        TrainConfig(**{**vars(TRAIN), "generations": 3}),
        tmp_path,
        pool,
        validation=viable_seeds(WORLD, 2, 20_000),
        validate_every=1,
        resume=True,
    )
    assert result["best_generation"] >= 0, "the resumed run chose no centre"
    chosen = result["best_validation"]
    assert chosen is not None
    assert math.isfinite(chosen), "the resumed run kept the quiet value"


def test_a_written_file_names_no_hidden_width_and_still_loads(
    tmp_path: Path, pool: list[int]
) -> None:
    """The trainer dropped a stored key, and the reader takes the file anyway.

    A file used to carry a hidden width. One policy kind read that width, the
    project deleted the kind, and the trainer then wrote the key as a constant
    zero that nothing read back. Dropping it changes the shape of every file a
    run writes, so this drives a real run and reads the file from disk.

    **A file that stopped loading over this would be the defect.** The reader
    builds what a file names from the keys the file holds, so an absent key
    costs it nothing. The stored index covers the other direction, where a
    file written by an older run still carries the key.[^1]

    References
    ----------
    [^1]: The stored policy index test. ``tests/test_stored_policy_index.py``
    """
    train("t", WORLD, WEIGHTING, TRAIN, tmp_path, pool, validation=[])
    for name in ("t.npz", "t-latest.npz"):
        policy, meta = load_policy(tmp_path / name)
        assert policy is not None, f"{name} did not load"
        assert "hidden" not in meta, f"{name} still names a hidden width"
        assert "generation" in meta, f"{name} lost the generation counter"
        assert meta["kind"] == "linear"
        assert meta["width"] == WORLD.width, f"{name} lost the fit of its world"
        assert meta["faction_count"] == WORLD.faction_count
