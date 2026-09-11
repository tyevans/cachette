"""A run may start from the centre another run stored, and it is then a new run.

The owner trains one strategy under its own reward from the weights that a run
under another reward produced. **A start from a file is not a resume.** A
resume continues the generation counter and the best score of its own run. A
start from a file begins at generation zero with no best score, because a score
under one reward says nothing under another.[^1]

The run tests drive the trainer and read the files from disk. The refusals of a
start are the refusals of a resume, and the trainer declares them in one
place.[^2] Each refusal test below gives the start a file that a resume would
refuse, and asserts that the start refuses it too.

The command line tests run the trainer up to its first seed search, through the
stand-in the launcher tests use. A check that did not fire then stops at the
seed search rather than playing a whole run.

The world is small and the generations are few, because a run of the real size
takes hours.

References
----------
[^1]: The trainer, the checkpoint of a run. ``python/cachette/learn/train.py``

[^2]: Recurring defect shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import math
import subprocess
import sys
from dataclasses import replace
from pathlib import Path

import numpy as np
import pytest

from cachette.learn import train as trainer
from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.normalize import reference_normalizer
from cachette.learn.policy import PolicyFitError, load_policy
from cachette.learn.record import ValidationScore
from cachette.learn.reward import Weighting
from cachette.learn.search import shell_policy
from cachette.learn.structured import STRUCTURED_KIND
from cachette.learn.train import Checkpoint, TrainConfig, train
from test_launcher_trainer_calls import STAND_IN_TRAINER, write_start_file

# A world that resolves quickly, so a test pays for a whole game rather than a
# truncated one. It is the world the checkpoint tests play.
WORLD = EnvConfig(
    width=24,
    height=24,
    faction_count=3,
    seat=0,
    tick_limit=200,
    horizon=20,
    decision_interval=10,
)

# The command line arguments that name the world above. The trainer derives
# the horizon from the tick limit and the interval, so the two agree.
WORLD_ARGUMENTS = ("--world-extent", "24", "--tick-limit", "200")

# The reward of the run that writes the source file.
SOURCE_REWARD = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)

# The reward of the run that starts from it. It is another reward, which is the
# case the owner needs.
TARGET_REWARD = Weighting(terms={"held_tiles": 0.5}, won=50.0, lost=-10.0, drawn=0.0)

TRAIN = TrainConfig(
    generations=2, population=4, seeds_per_generation=1, workers=4, seed=0
)

# The generation the source file states. A start that continued the counter of
# its source would begin after it.
SOURCE_GENERATION = 5

# The best score the source file states. A start that kept the best score of
# its source would report it.
SOURCE_BEST = ValidationScore(won=0.5, mean=10.0, episodes=2)

# The seed of the draw that the source centre holds.
SOURCE_DRAW_SEED = 5


@pytest.fixture(scope="module")
def pool() -> list[int]:
    """Return a training seed pool for the small world."""
    return viable_seeds(WORLD, 6, 900)


@pytest.fixture(scope="module")
def source(tmp_path_factory: pytest.TempPathFactory) -> Path:
    """Write the file a later run starts from, and return it.

    **The real checkpoint writes the file, so it holds every entry a run
    writes.** The centre is a unit draw and not the shell, so a run that
    started from the shell does not reach it. A source trained here kept its
    centre at the shell, because no generation of the small run carried
    information, and the control of the first test found that the fixture
    never reached the case.

    The file states a later generation and a best score. A start that resumed
    the source would begin after that generation and report that score.
    """
    out = tmp_path_factory.mktemp("source")
    probe = Env(WORLD, SOURCE_REWARD)
    normalizer = reference_normalizer(WORLD, SOURCE_REWARD)
    shell = shell_policy("linear", probe, normalizer)
    draw = np.random.default_rng(SOURCE_DRAW_SEED).standard_normal(shell.flat().size)
    path = out / "wonder-latest.npz"
    checkpoint = Checkpoint(
        name="wonder",
        out_dir=out,
        env_config=WORLD,
        probe=probe,
        kind="linear",
        normalizer=normalizer,
    )
    checkpoint.write(
        shell.rebuild(draw / np.linalg.norm(draw)),
        path,
        SOURCE_GENERATION,
        0.0,
        None,
        SOURCE_BEST,
    )
    return path


def centre_of(path: Path) -> np.ndarray:
    """Read the flat centre one weight file holds."""
    stored, _ = load_policy(path)
    return np.asarray(stored.flat())


def a_started_run(
    out: Path,
    pool: list[int],
    start_from: Path | None,
    generations: int = 1,
    resume: bool = False,
    world: EnvConfig = WORLD,
    kind: str = "linear",
    readout_only: bool = False,
) -> trainer.TrainResult:
    """Train the run that starts from another run, and return its result.

    **The learning rate is zero.** The centre of the first generation is then
    the centre the run started from, so a test reads the start centre from
    the first file the run writes.
    """
    return train(
        "renown",
        world,
        TARGET_REWARD,
        replace(
            TRAIN,
            generations=generations,
            learning_rate=0.0,
            readout_only=readout_only,
        ),
        out,
        pool,
        kind=kind,
        resume=resume,
        validation=[],
        start_from=start_from,
    )


def test_a_started_run_begins_at_the_stored_centre_at_generation_zero(
    tmp_path: Path,
    pool: list[int],
    source: Path,
    capsys: pytest.CaptureFixture[str],
) -> None:
    """The run takes the centre of the file, and nothing else of its run.

    The source ends at a later generation and holds a best score. A start that
    resumed the source would begin after that generation and report that
    score, so each assertion below separates a start from a resume.
    """
    _, stated = load_policy(source)
    assert stated["generation"] == SOURCE_GENERATION
    assert math.isfinite(stated["best_selection_won"]), "the source chose nothing"
    stored = centre_of(source)
    seeded = np.asarray(shell_policy("linear", Env(WORLD, TARGET_REWARD)).flat())
    assert not np.allclose(stored, seeded), "the source never moved its centre"

    capsys.readouterr()
    result = a_started_run(tmp_path, pool, source)
    printed = capsys.readouterr().out.splitlines()

    assert printed[0].startswith(
        f"  renown starts from {source} at generation 0 with no best score"
    ), printed[:3]
    assert [row["generation"] for row in result["history"]] == [0]
    assert result["best_selection_won"] is None
    assert result["best_selection_return"] is None
    first = centre_of(tmp_path / "renown-gen000.npz")
    assert np.allclose(first, stored, rtol=0.0, atol=1e-12)
    assert sorted(path.name for path in tmp_path.glob("*.npz")) == [
        "renown-gen000.npz",
        "renown-latest.npz",
        "renown.npz",
    ]


def test_every_file_of_a_started_run_names_its_source(
    tmp_path: Path, pool: list[int], source: Path
) -> None:
    """A reader of any file the run writes sees that it did not start seeded."""
    a_started_run(tmp_path, pool, source)
    for name in ("renown-latest.npz", "renown-gen000.npz", "renown.npz"):
        _, meta = load_policy(tmp_path / name)
        assert meta["strategy"] == "renown", name
        assert meta["started_from"] == "wonder-latest.npz", name
        assert meta["started_from_strategy"] == "wonder", name
        assert meta["started_from_generation"] == SOURCE_GENERATION, name
        assert math.isnan(meta["best_selection_won"]), name


def test_a_run_from_a_seeded_draw_names_no_source(source: Path) -> None:
    """The control: a file of a run that started seeded states no start file."""
    _, meta = load_policy(source)
    assert meta["strategy"] == "wonder"
    assert "started_from" not in meta
    assert "started_from_strategy" not in meta


def test_a_restart_of_a_started_run_resumes_and_keeps_naming_its_source(
    tmp_path: Path, pool: list[int], source: Path
) -> None:
    """A restart continues the started run, and its files still name the source.

    **The launcher restarts a run after a fault with the resume flag alone.**
    A resume that dropped the start point would write files after the
    restart that read like a run that started seeded.
    """
    a_started_run(tmp_path, pool, source)
    second = a_started_run(tmp_path, pool, None, generations=2, resume=True)
    assert [row["generation"] for row in second["history"]] == [1]
    _, meta = load_policy(tmp_path / "renown-latest.npz")
    assert meta["generation"] == 1
    assert meta["started_from"] == "wonder-latest.npz"
    assert meta["started_from_generation"] == SOURCE_GENERATION


def write_other_centre(
    path: Path,
    world: EnvConfig = WORLD,
    kind: str = "linear",
    readout_only: bool = False,
    normalized: bool = True,
) -> Path:
    """Write a centre that differs from the run that starts from it in one way.

    **The file carries the normalizer of the small world when it carries one.**
    The fit compares two normalizers when both sides state one, so a file of
    another normalizer would stop at the fit and never reach the refusal a
    test names.
    """
    normalizer = reference_normalizer(WORLD, SOURCE_REWARD) if normalized else None
    return write_start_file(
        path, world, kind, readout_only=readout_only, normalizer=normalizer
    )


# One case for each refusal a resume makes. Each names the file, the run that
# reads it, and the words of the refusal.
REFUSALS = {
    "another world": (
        {"world": replace(WORLD, width=32, height=32), "normalized": False},
        {},
        "does not fit this world",
    ),
    "another readout setting": (
        {"kind": STRUCTURED_KIND, "readout_only": True},
        {"kind": STRUCTURED_KIND, "readout_only": False},
        "continues neither run",
    ),
    "another limit rule": (
        {"world": replace(WORLD, limit_is_loss=True)},
        {},
        "tick limit as a loss",
    ),
    "no normalizer": (
        {"normalized": False},
        {},
        "states no feature normalizer",
    ),
    "another kind": (
        {"kind": STRUCTURED_KIND},
        {},
        "holds a structured policy",
    ),
}


@pytest.mark.parametrize("case", list(REFUSALS), ids=list(REFUSALS))
def test_a_start_refuses_every_file_a_resume_refuses(
    tmp_path: Path, pool: list[int], case: str
) -> None:
    """A start from a file that a resume would refuse fails, and writes nothing."""
    written, run, words = REFUSALS[case]
    path = write_other_centre(tmp_path / "wonder-latest.npz", **written)
    out = tmp_path / "run"
    with pytest.raises(PolicyFitError, match=words):
        a_started_run(out, pool, path, **run)
    assert not list(out.glob("*.npz")) if out.exists() else True


def test_a_run_cannot_both_resume_and_start_from_a_file(
    tmp_path: Path, pool: list[int], source: Path
) -> None:
    """A run that resumes continues its own run, so it cannot start from another."""
    with pytest.raises(ValueError, match="never both"):
        a_started_run(tmp_path, pool, source, resume=True)


@pytest.mark.parametrize(
    "content", [None, b"", b"not a weight file"], ids=["missing", "empty", "text"]
)
def test_an_unreadable_start_file_fails_before_any_episode(
    tmp_path: Path,
    pool: list[int],
    monkeypatch: pytest.MonkeyPatch,
    content: bytes | None,
) -> None:
    """A file that cannot be read ends the run before the first episode plays.

    **The first episodes of a run play the reference sample of the feature
    normalizer.** The stand-in below fails the run if that sample starts, so
    a trainer that read the file after it fails here with the wrong error.
    """
    path = tmp_path / "wonder-latest.npz"
    if content is not None:
        path.write_bytes(content)

    def no_episode(*arguments: object, **keywords: object) -> None:
        message = "an episode played before the start file was read"
        raise AssertionError(message)

    monkeypatch.setattr(trainer, "reference_normalizer", no_episode)
    out = tmp_path / "run"
    with pytest.raises((OSError, EOFError, ValueError)):
        a_started_run(out, pool, path)
    assert not out.exists()


def run_the_trainer(
    tmp_path: Path, *arguments: str
) -> subprocess.CompletedProcess[str]:
    """Run the trainer from its command line, up to its first seed search."""
    stand_in = tmp_path / "trainer.py"
    stand_in.write_text(STAND_IN_TRAINER, encoding="utf-8")
    return subprocess.run(
        [
            sys.executable,
            str(stand_in),
            *WORLD_ARGUMENTS,
            *arguments,
            "--out",
            str(tmp_path / "out"),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.fixture
def small_centre(tmp_path: Path) -> Path:
    """Write a linear centre of the small world, which a conquest run may start from."""
    return write_start_file(tmp_path / "wonder-latest.npz", WORLD, "linear")


@pytest.mark.parametrize(
    ("arguments", "words"),
    [
        (("--only", "conquer", "--resume"), "Name one of them"),
        (("--only", "conquer,land"), "one centre to one strategy"),
        ((), "one centre to one strategy"),
    ],
    ids=["resume", "two strategies", "every strategy"],
)
def test_the_command_line_refuses_a_start_it_cannot_make(
    tmp_path: Path, small_centre: Path, arguments: tuple[str, ...], words: str
) -> None:
    """A start needs one strategy and no resume, and the run ends before it plays.

    **The file fits the run.** A check that did not fire would therefore pass
    every later check and stop at the seed search with a status of zero.
    """
    finished = run_the_trainer(tmp_path, *arguments, "--start-from", str(small_centre))
    assert finished.returncode == 2, finished.stderr[-2000:]
    assert words in finished.stderr
    assert not (tmp_path / "out").exists()


@pytest.mark.parametrize(
    "content",
    [None, b"not a weight file", "another world"],
    ids=["missing", "text", "another world"],
)
def test_the_command_line_refuses_a_file_that_cannot_start_the_run(
    tmp_path: Path, content: bytes | str | None
) -> None:
    """A missing, unreadable or unfitting file ends the run before anything plays.

    **This is the check the launcher reaches before it rents a machine.** The
    file of another world fails here, and not after an instance compiled the
    engine.
    """
    path = tmp_path / "wonder-latest.npz"
    if content == "another world":
        write_start_file(path, replace(WORLD, width=32, height=32), "linear")
    elif isinstance(content, bytes):
        path.write_bytes(content)
    finished = run_the_trainer(tmp_path, "--only", "conquer", "--start-from", str(path))
    assert finished.returncode == 2, finished.stderr[-2000:]
    assert "cannot start the strategy conquer" in finished.stderr
    assert "does not fall back to a seeded draw" in finished.stderr
    assert not (tmp_path / "out").exists()


@pytest.mark.parametrize(
    ("arguments", "kind", "readout_only"),
    [
        (("--only", "conquer"), "linear", False),
        (
            ("--only", "conquer-structured", "--train-readout-only"),
            STRUCTURED_KIND,
            True,
        ),
    ],
    ids=["linear", "readout only"],
)
def test_the_command_line_takes_a_start_that_fits_and_says_so_first(
    tmp_path: Path, arguments: tuple[str, ...], kind: str, readout_only: bool
) -> None:
    """A start that fits passes every flag check of the train path.

    **A refusal of one flag has stopped a paid launch before**, because it
    fired on a path its own tests never drove. This drives the train path of
    the command line with the start beside the readout flag, and the run
    reaches its seed search. The first line it prints names the file.
    """
    path = write_start_file(
        tmp_path / "wonder-latest.npz", WORLD, kind, readout_only=readout_only
    )
    finished = run_the_trainer(tmp_path, *arguments, "--start-from", str(path))
    assert finished.returncode == 0, finished.stderr[-2000:]
    assert finished.stdout.splitlines()[0] == (
        f"the run starts from {path}, at generation 0 with no best score"
    )
