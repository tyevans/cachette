"""A policy file names its own kind, and a reader gets back what was written.

The learner stores its weights in a file and a later run reads them. Two
kinds of policy share that file format, so the file must say which kind it
holds. A file that named no kind would load as the wrong kind and would
choose a legal action every time, which is a failure nothing reports.[^1]

This module covers the linear policy and the file that names no kind. The
tests of the structured policy sit beside the layout helpers that build it,
because that policy reads a layout and not a length.[^2]

**No test in this file states that a policy reads the world.** The tests here
are about the file format, the mask and the untrained baseline. Every one of
them passes for a policy that ignores the observation. The assertions that
separate a policy from a fixed preference order sit in their own file, with
the guard that requires them to fail under a blinded encoder.[^3]

# The shapes

The behavioural tests draw the shape the engine publishes. They draw it with a
majority of positions that never move, and with a small minority of legal
action rows. A fixture whose every position moves, and whose every row is
legal, is the inverse of the world on both axes that decide this. A defect in
the constant subspace cannot exist in such a fixture.[^4] [^5]

The file-format tests keep a small shape. A round trip through an archive is
arithmetic over any two lengths, and a large shape costs time for nothing.

References
----------
[^1]: Recurring Defect Shapes, section 1.
``.agents/rules/recurring-defects.md``

[^2]: The tests of the structured policy.
``tests/test_learner_structured_policy.py``

[^3]: The sensitivity tests and the blinding guard.
``tests/test_learner_policy_sensitivity.py``

[^4]: Findings register, FND-709. ``docs/FINDINGS.md``

[^5]: Testing Rules, section 2a, on what a fixture must supply.
``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np

from cachette.learn.policy import LinearPolicy, load_policy

if TYPE_CHECKING:
    from pathlib import Path

    from learner_shapes import EngineShapes, WorldShapedInputs

ACTIONS = 29
FEATURES = 176
"""Two lengths for the file-format tests of this module.

**They state no layout of the engine.** A round trip through an archive is
arithmetic over any two lengths, and the schema of a world is the only
declaration of the real ones.
"""


def _archive_inputs(rows: int = 4) -> tuple[np.ndarray, np.ndarray]:
    """Return a small observation stack and a mask stack that allows every row.

    A test that uses this shape reads a file back and compares two policies
    over one input. The shape says nothing about play, and no test that uses
    it may claim to.
    """
    rng = np.random.default_rng(7)
    observations = rng.integers(0, 5000, (rows, FEATURES)).astype(np.int64)
    masks = np.ones((rows, ACTIONS), dtype=np.uint8)
    return observations, masks


def test_an_untrained_linear_policy_takes_the_no_op(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Every weight starts at zero, so every score is zero.

    The first legal row of the table is the no-op, and the choice takes the
    highest score. An untrained policy therefore does nothing, and that is
    the baseline every trained policy is measured against.
    """
    linear = LinearPolicy.zeros(
        engine_shapes.action_length, engine_shapes.observation_length
    )
    rows = world_shaped_inputs.observations.shape[0]
    chosen = linear.choose_many(
        world_shaped_inputs.observations, world_shaped_inputs.masks
    )
    assert chosen == [0] * rows


def test_a_perturbed_linear_policy_chooses_something_other_than_the_no_op(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """A perturbation moves the choice off the row an untrained policy takes.

    **This is not the property the trainer depends on**, and the docstring of
    this test used to say that it was. A fixed preference order over the
    action rows satisfies it completely. Such a policy answers a row other
    than the no-op at every decision, and it reads nothing.[^1]

    What this test states is narrower, and it is still worth stating. A
    perturbation that left every candidate on the no-op would give a
    generation one score. The run would then report a flat curve, and no
    reader could see a defect in it.

    The property that separates a policy from a preference order sits in
    another file, with the guard that requires it to fail under a blinded
    encoder.[^2]

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``

    [^2]: The sensitivity tests and the blinding guard.
    ``tests/test_learner_policy_sensitivity.py``
    """
    linear = LinearPolicy.zeros(
        engine_shapes.action_length, engine_shapes.observation_length
    )
    rng = np.random.default_rng(1)
    moved = linear.rebuild(rng.standard_normal(linear.flat().size))
    rows = world_shaped_inputs.observations.shape[0]
    chosen = moved.choose_many(
        world_shaped_inputs.observations, world_shaped_inputs.masks
    )
    assert chosen != [0] * rows


def test_a_stored_linear_policy_reads_back_as_the_kind_that_wrote_it(
    tmp_path: Path,
) -> None:
    """The kind round trips through a file, and so do the choices."""
    observations, masks = _archive_inputs()
    rng = np.random.default_rng(3)
    linear = LinearPolicy.zeros(ACTIONS, FEATURES)
    linear = linear.rebuild(rng.standard_normal(linear.flat().size) * 0.1)

    path = tmp_path / "lin.npz"
    linear.save(path, {"observation_length": FEATURES, "action_length": ACTIONS})
    read, meta = load_policy(path)

    assert type(read) is LinearPolicy
    assert meta["kind"] == "linear"
    assert read.choose_many(observations, masks) == linear.choose_many(
        observations, masks
    )


def test_a_file_that_names_no_kind_reads_back_as_a_linear_policy(
    tmp_path: Path,
) -> None:
    """The runs of the first night wrote no kind entry, and they still load."""
    path = tmp_path / "old.npz"
    weights = np.zeros((ACTIONS, FEATURES + 1))
    np.savez(path, weights=weights, observation_length=np.array(FEATURES))
    read, meta = load_policy(path)
    assert isinstance(read, LinearPolicy)
    assert meta["kind"] == "linear"


def test_the_mask_removes_a_row_from_a_linear_policy(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """A policy never returns a row the engine called illegal.

    The mask decides before the weights do. A policy that scored an illegal
    row highest would send it, and the engine would refuse it silently.

    **This test states legality, and it states nothing about play.** The
    register measured that the mask supplies what looks like situational play,
    so a policy that reads nothing passes this test.[^1] The assertions about
    play sit in another file.[^2]

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``

    [^2]: The sensitivity tests and the blinding guard.
    ``tests/test_learner_policy_sensitivity.py``
    """
    observations = world_shaped_inputs.observations
    masks = np.zeros_like(world_shaped_inputs.masks)
    masks[:, 0] = 1
    rng = np.random.default_rng(5)
    linear = LinearPolicy(
        rng.standard_normal(
            (engine_shapes.action_length, engine_shapes.observation_length + 1)
        )
    )
    assert linear.choose_many(observations, masks) == [0] * observations.shape[0]
