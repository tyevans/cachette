"""A run that trains the readout alone holds every other weight where it started.

The structured policy holds three towers, a trunk and a readout. A run may ask
the search to move the readout alone. The towers and the trunk then keep the
seeded draw of the shell for the whole run, and every figure that counts
trainable weights counts the readout.[^1]

The run tests drive the trainer and read the centres from disk. **A test that
built the search and called it once would pass against a trainer that never
passed the setting on**, so those tests start at the trainer.[^2] The search
tests beside them reach what a short run cannot: a gradient outside the
readout, and a readout long enough to pass the norm ceiling of the centre.

The world is small and the generations are few, because a run of the real
size takes hours.

References
----------
[^1]: The search, a run may train the readout alone.
``python/cachette/learn/search.py``

[^2]: Testing Rules, section 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import math
import subprocess
import sys
from dataclasses import replace
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette._core import World
from cachette.learn.env import Env, EnvConfig, viable_seeds
from cachette.learn.layout import ObservationLayout
from cachette.learn.policy import LinearPolicy, PolicyFitError, load_policy
from cachette.learn.reward import Weighting
from cachette.learn.search import (
    EvolutionStrategy,
    generation_noise,
    step_alignment,
    trainable_count,
)
from cachette.learn.signals import SignalCatalogue
from cachette.learn.structured import STRUCTURED_KIND, StructuredPolicy
from cachette.learn.train import TrainConfig, first_scoring, train

if TYPE_CHECKING:
    from pathlib import Path

    from cachette.learn.train import TrainResult

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

WEIGHTING = Weighting(terms={"held_tiles": 1.0}, won=100.0, lost=-100.0, drawn=0.0)

TRAIN = TrainConfig(
    generations=3, population=4, seeds_per_generation=1, workers=4, seed=0
)

# The layout the search tests build a shell over, without playing a game.
LAYOUT_SEED = 7
STRUCTURED_ACTIONS = 9
SEARCH_PAIRS = 3

# Scores of a population of three pairs in which the plus half of every pair
# wins, so the generation agrees and the search takes a step.
AGREED_SCORES = np.array([6.0, 1.0, 5.0, 2.0, 4.0, 3.0])


@pytest.fixture
def pool() -> list[int]:
    """Return a training seed pool for the small world."""
    return viable_seeds(WORLD, 6, 100)


def a_run(
    out: Path,
    pool: list[int],
    readout_only: bool,
    generations: int = TRAIN.generations,
    resume: bool = False,
    kind: str = STRUCTURED_KIND,
    workers: int = 1,
) -> TrainResult:
    """Train one small run, and return what the trainer reports."""
    return train(
        "t",
        WORLD,
        WEIGHTING,
        replace(
            TRAIN, generations=generations, readout_only=readout_only, pool=workers
        ),
        out,
        pool,
        kind=kind,
        resume=resume,
        validation=[],
    )


def the_shell() -> StructuredPolicy:
    """Build the shell a structured run of the small world starts from.

    **The shell comes from one fixed seed**, so this is the generation 0
    centre of every structured run of this world, whatever the run seed.
    """
    probe = Env(WORLD, WEIGHTING)
    return StructuredPolicy.of_catalogue(probe.action_length, probe.signals)


def frozen_mask(shell: StructuredPolicy) -> np.ndarray:
    """Return true at every coordinate of the flat vector outside the readout."""
    frozen = np.ones(shell.flat().size, dtype=bool)
    frozen[shell.readout_span] = False
    return frozen


def stored_centre(path: Path) -> np.ndarray:
    """Read the flat centre one weight file holds."""
    stored, _ = load_policy(path)
    return np.asarray(stored.flat())


def a_search_shell() -> StructuredPolicy:
    """Build a structured shell over the published layout, with no game."""
    world = World(width=24, height=24, faction_count=3, seed=LAYOUT_SEED)
    layout = ObservationLayout.of_catalogue(SignalCatalogue.of_world(world))
    return StructuredPolicy.zeros(STRUCTURED_ACTIONS, layout)


def a_readout_search(shell: StructuredPolicy) -> EvolutionStrategy:
    """Build a search that trains the readout of this shell alone."""
    return EvolutionStrategy(
        shell=shell,
        pairs=SEARCH_PAIRS,
        sigma=0.5,
        learning_rate=0.3,
        seed=LAYOUT_SEED,
        readout_only=True,
    )


def a_moved_centre(shell: StructuredPolicy, readout_scale: float = 0.05) -> np.ndarray:
    """Return the shell with a drawn readout, the way a first generation leaves it.

    A zero readout takes row zero at every decision, so a test of the step
    over it would measure the no-op and not the arithmetic under test.
    """
    centre = shell.flat().copy()
    span = shell.readout_span
    draw = np.random.default_rng(3).standard_normal(span.stop - span.start)
    centre[span] = readout_scale * draw
    return centre


def test_a_readout_run_holds_every_weight_outside_the_readout(
    tmp_path: Path, pool: list[int]
) -> None:
    """Every centre of the run holds the seeded towers, bit for bit.

    The run keeps the centre of every generation. Each one must hold the
    towers and the trunk of the shell, to the last bit, and the readout of
    the last one must have moved. A run whose readout never moved would pass
    the first half for a reason that says nothing.
    """
    a_run(tmp_path, pool, readout_only=True)
    shell = the_shell()
    start = shell.flat()
    frozen = frozen_mask(shell)
    for generation in range(TRAIN.generations):
        centre = stored_centre(tmp_path / f"t-gen{generation:03d}.npz")
        assert centre[frozen].tobytes() == start[frozen].tobytes(), (
            f"generation {generation} moved a weight outside the readout"
        )
    last = stored_centre(tmp_path / "t-latest.npz")
    span = shell.readout_span
    assert not np.array_equal(last[span], start[span]), "the readout never moved"


def test_a_run_of_every_weight_moves_the_weights_outside_the_readout(
    tmp_path: Path, pool: list[int]
) -> None:
    """The control: the same run without the setting moves the towers.

    Without this, the test above could pass because the run moves nothing
    outside the readout in any configuration, for example because every
    generation of this world carried no information.
    """
    a_run(tmp_path, pool, readout_only=False)
    shell = the_shell()
    frozen = frozen_mask(shell)
    last = stored_centre(tmp_path / "t-latest.npz")
    assert not np.array_equal(last[frozen], shell.flat()[frozen])


def test_a_readout_run_reports_the_readout_as_its_trainable_count(
    tmp_path: Path, pool: list[int], capsys: pytest.CaptureFixture[str]
) -> None:
    """The trainer counts the readout, in its result and on every line it prints.

    The expected figures come from the shell and from the law, and never from
    the trainer, so a trainer that counted every weight fails here.
    """
    result = a_run(tmp_path, pool, readout_only=True, generations=1)
    shell = the_shell()
    readout = int(shell.readout.size)
    total = int(shell.flat().size)
    alignment = math.sqrt(TRAIN.pairs / readout)
    assert readout < total
    assert result["trainable"] == readout
    assert result["parameters"] == total
    assert result["readout_only"] is True
    printed = capsys.readouterr().out
    assert (
        f"one step of {TRAIN.pairs} pairs over {readout} trainable weights "
        f"aligns {alignment:.4f}"
    ) in printed
    assert f"aligned {alignment:6.4f}" in printed
    assert f"this run trains {readout} of the {total} weights" in printed


@pytest.mark.parametrize("written", [True, False])
def test_a_resume_refuses_a_checkpoint_of_the_other_setting(
    tmp_path: Path, pool: list[int], written: bool
) -> None:
    """A resume across the two settings continues neither run, so it fails.

    The file names the setting it was written under, and the resume reads it
    back. Both directions fail, and neither writes a generation first.
    """
    a_run(tmp_path, pool, readout_only=written, generations=1)
    latest = tmp_path / "t-latest.npz"
    before = latest.read_bytes()
    with pytest.raises(PolicyFitError, match="continues neither run"):
        a_run(tmp_path, pool, readout_only=not written, generations=2, resume=True)
    assert latest.read_bytes() == before


def test_a_resume_of_the_same_setting_continues_and_holds_the_towers(
    tmp_path: Path, pool: list[int]
) -> None:
    """A resumed readout run continues, and its towers are still the draw.

    This is the positive half of the refusal above. A resume that refused
    every readout file would pass the refusal test and train nothing.
    """
    a_run(tmp_path, pool, readout_only=True, generations=1)
    second = a_run(tmp_path, pool, readout_only=True, generations=2, resume=True)
    assert [row["generation"] for row in second["history"]] == [1]
    shell = the_shell()
    frozen = frozen_mask(shell)
    last = stored_centre(tmp_path / "t-latest.npz")
    assert last[frozen].tobytes() == shell.flat()[frozen].tobytes()


def test_a_readout_run_reaches_one_centre_at_any_worker_count(
    tmp_path: Path, pool: list[int]
) -> None:
    """A worker process draws the candidates the trainer draws.

    A worker reads the setting from the configuration it receives. A worker
    that ignored it would perturb the towers of its candidate, so its score
    and the step after it would differ from a run of one process.
    """
    alone = a_run(tmp_path / "alone", pool, readout_only=True, generations=2)
    queued = a_run(
        tmp_path / "queued", pool, readout_only=True, generations=2, workers=2
    )
    assert alone["history"] and queued["history"]
    one = stored_centre(tmp_path / "alone" / "t-latest.npz")
    two = stored_centre(tmp_path / "queued" / "t-latest.npz")
    assert one.tobytes() == two.tobytes()


def test_the_linear_policy_refuses_to_train_its_readout_alone(
    tmp_path: Path, pool: list[int]
) -> None:
    """The linear policy has no readout block, so the trainer refuses the setting.

    The refusal comes before the run plays anything, so it writes nothing.
    """
    with pytest.raises(ValueError, match="no readout block"):
        a_run(tmp_path, pool, readout_only=True, kind="linear")
    assert not any(tmp_path.iterdir())
    with pytest.raises(ValueError, match="no readout block"):
        EvolutionStrategy(
            shell=LinearPolicy.zeros(STRUCTURED_ACTIONS, 4),
            pairs=SEARCH_PAIRS,
            sigma=0.5,
            learning_rate=0.3,
            seed=LAYOUT_SEED,
            readout_only=True,
        )


def a_plan(*arguments: str) -> subprocess.CompletedProcess[str]:
    """Ask the command line for the plan of a run, as the launcher asks it."""
    return subprocess.run(
        [sys.executable, "-m", "cachette.learn", "--print-plan", *arguments],
        capture_output=True,
        text=True,
        check=False,
    )


def test_the_command_line_refuses_the_flag_for_a_linear_strategy() -> None:
    """The launcher asks for the plan before it rents, so the refusal is free."""
    answer = a_plan("--only", "wealth", "--train-readout-only")
    assert answer.returncode == 2
    assert "cannot train its readout alone" in answer.stderr


def test_the_plan_states_the_readout_as_the_trainable_count() -> None:
    """The plan names the readout count with the flag and every weight without.

    The expected counts come from a shell of the world the command line
    plays, so the test states no count of its own.
    """
    from cachette.learn.__main__ import STRATEGIES

    env_config, scoring, kind = STRATEGIES["wealth-structured"]
    assert kind == STRUCTURED_KIND
    probe = Env(env_config, first_scoring(scoring))
    shell = StructuredPolicy.of_catalogue(probe.action_length, probe.signals)
    pairs = TrainConfig(population=24).pairs
    readout = int(shell.readout.size)
    total = int(shell.flat().size)
    common = ("--only", "wealth-structured", "--population", "24")

    trained = a_plan(*common, "--train-readout-only")
    assert trained.returncode == 0, trained.stderr
    assert f"one step of {pairs} pairs over {readout} trainable weights" in (
        trained.stdout
    )
    assert f"this run trains {readout} of the {total} weights" in trained.stdout

    every = a_plan(*common)
    assert every.returncode == 0, every.stderr
    assert f"one step of {pairs} pairs over {total} trainable weights" in every.stdout


def test_the_readout_span_names_the_readout_of_the_rebuilt_policy() -> None:
    """The span the search reads is where the rebuild puts the readout.

    A span that named the trunk would freeze the readout and train the
    trunk, and every other test of this file would still see a span.
    """
    shell = a_search_shell()
    marked = np.zeros(shell.flat().size)
    marked[shell.readout_span] = 1.0
    rebuilt = shell.rebuild(marked)
    assert np.all(rebuilt.readout == 1.0)
    assert not np.any(rebuilt.trunk)
    assert trainable_count(shell, readout_only=True) == shell.readout.size
    assert trainable_count(shell) == shell.flat().size


def test_the_noise_of_a_readout_search_is_the_full_draw_held_to_the_readout() -> None:
    """The draw is keyed as a run of every weight keys it, and zero elsewhere.

    The generator reads the run seed and the generation alone. The readout
    is one layer, so it takes one weight throughout, and each row is the
    draw of that generation held to the readout and scaled to unit length.
    """
    shell = a_search_shell()
    centre = a_moved_centre(shell)
    frozen = frozen_mask(shell)
    noise = generation_noise(LAYOUT_SEED, 2, SEARCH_PAIRS, shell, centre, True)
    assert not np.any(noise[:, frozen])
    assert np.allclose(np.linalg.norm(noise, axis=1), 1.0)
    draw = np.random.default_rng([LAYOUT_SEED, 2]).standard_normal(noise.shape)
    draw[:, frozen] = 0.0
    held = draw / np.linalg.norm(draw, axis=1, keepdims=True)
    assert np.allclose(noise, held)
    other = generation_noise(LAYOUT_SEED, 3, SEARCH_PAIRS, shell, centre, True)
    assert not np.allclose(noise, other)


def test_every_candidate_of_a_readout_search_holds_the_frozen_layers() -> None:
    """A candidate differs from its centre in the readout and nowhere else."""
    shell = a_search_shell()
    search = a_readout_search(shell)
    centre = a_moved_centre(shell)
    frozen = frozen_mask(shell)
    for candidate in search.propose(centre, 0):
        moved = candidate.flat()
        assert moved[frozen].tobytes() == centre[frozen].tobytes()
        assert not np.array_equal(moved[~frozen], centre[~frozen])


def test_the_step_writes_the_readout_alone_whatever_the_gradient() -> None:
    """A gradient outside the readout reaches no frozen weight.

    The noise of this search is zero outside the readout, so a run never
    hands the step such a gradient. The step holds the frozen weights by its
    own contract as well, so a later change to the draw cannot reach them.
    """
    shell = a_search_shell()
    search = a_readout_search(shell)
    centre = a_moved_centre(shell)
    frozen = frozen_mask(shell)
    gradient = np.random.default_rng(5).standard_normal(centre.size)
    moved = search.step(centre, gradient, 1.0)
    assert moved[frozen].tobytes() == centre[frozen].tobytes()
    assert not np.array_equal(moved[~frozen], centre[~frozen])


def test_a_long_readout_scales_no_frozen_weight() -> None:
    """The norm ceiling of the centre never scales the towers of a readout run.

    The readout here is long enough to take the whole centre past the
    ceiling. A search that applied the ceiling to the whole centre would
    shorten the towers with it.
    """
    shell = a_search_shell()
    search = a_readout_search(shell)
    centre = a_moved_centre(shell, readout_scale=100.0)
    assert np.linalg.norm(centre) > search.norm_ceiling
    frozen = frozen_mask(shell)
    update = search.update(centre, 0, AGREED_SCORES)
    assert update.informative
    assert update.centre[frozen].tobytes() == centre[frozen].tobytes()


def test_a_readout_search_aligns_over_the_readout() -> None:
    """The alignment of every update counts the readout weights alone."""
    shell = a_search_shell()
    search = a_readout_search(shell)
    readout = int(shell.readout.size)
    assert search.trainable == readout
    expected = step_alignment(SEARCH_PAIRS, readout)
    centre = a_moved_centre(shell)
    assert search.update(centre, 0, AGREED_SCORES).alignment == pytest.approx(expected)
    quiet = search.update(centre, 0, np.full(search.population, 3.0))
    assert quiet.alignment == pytest.approx(expected)
