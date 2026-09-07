"""A policy file names its own kind, and a reader gets back what was written.

The learner stores its weights in a file and a later run reads them. Two
kinds of policy now share that file format, so the file must say which kind
it holds. A file that named no kind would load as the wrong kind and would
choose a legal action every time, which is a failure nothing reports.[^1]

References
----------
[^1]: Recurring Defect Shapes, section 1.
``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np

from cachette.learn.policy import LinearPolicy, MLPPolicy, load_policy

if TYPE_CHECKING:
    from pathlib import Path

# Three shapes for the arrays of this test. **They state no layout of the
# engine.** A policy is arithmetic over whatever length the caller gives it,
# and the schema of a world is the only declaration of the real length.
ACTIONS = 29
FEATURES = 176
HIDDEN = 8


def _inputs(rows: int = 4) -> tuple[np.ndarray, np.ndarray]:
    """Return an observation stack and a mask stack that allows every row."""
    rng = np.random.default_rng(7)
    observations = rng.integers(0, 5000, (rows, FEATURES)).astype(np.int64)
    masks = np.ones((rows, ACTIONS), dtype=np.uint8)
    return observations, masks


def test_an_untrained_network_takes_the_no_op() -> None:
    """The second layer starts at zero, so every score is zero.

    The first legal row of the table is the no-op, and the choice takes the
    highest score. An untrained network therefore does nothing, in the way
    an untrained linear policy does. That makes the two comparable as one
    baseline.
    """
    observations, masks = _inputs()
    network = MLPPolicy.zeros(ACTIONS, FEATURES, HIDDEN)
    linear = LinearPolicy.zeros(ACTIONS, FEATURES)
    assert network.choose_many(observations, masks) == [0, 0, 0, 0]
    assert linear.choose_many(observations, masks) == [0, 0, 0, 0]


def test_the_projection_of_an_untrained_network_is_not_zero() -> None:
    """A network of two zero layers would never leave the origin.

    Every perturbation of the second layer of such a network multiplies a
    zero hidden vector, so every candidate of a generation scores the same
    and the evolution strategy has no direction. The first layer therefore
    holds a fixed random projection.
    """
    network = MLPPolicy.zeros(ACTIONS, FEATURES, HIDDEN)
    assert np.any(network.first != 0.0)
    assert np.all(network.second == 0.0)


def test_a_perturbed_network_chooses_something_other_than_the_no_op() -> None:
    """The trainer perturbs the flat vector, and the choice must move.

    This is the property the trainer depends on. A perturbation that never
    changed a choice would give every candidate one score, and the run would
    report a flat curve with no defect anyone could see.
    """
    observations, masks = _inputs()
    network = MLPPolicy.zeros(ACTIONS, FEATURES, HIDDEN)
    rng = np.random.default_rng(1)
    moved = network.rebuild(network.flat() + rng.standard_normal(network.flat().size))
    assert moved.choose_many(observations, masks) != [0, 0, 0, 0]


def test_a_stored_policy_reads_back_as_the_kind_that_wrote_it(
    tmp_path: Path,
) -> None:
    """Both kinds round trip through a file, and both keep their choices."""
    observations, masks = _inputs()
    rng = np.random.default_rng(3)

    network = MLPPolicy.zeros(ACTIONS, FEATURES, HIDDEN)
    network = network.rebuild(rng.standard_normal(network.flat().size))
    linear = LinearPolicy.zeros(ACTIONS, FEATURES)
    linear = linear.rebuild(rng.standard_normal(linear.flat().size) * 0.1)

    for name, policy in (("net", network), ("lin", linear)):
        path = tmp_path / f"{name}.npz"
        policy.save(path, {"observation_length": FEATURES, "action_length": ACTIONS})
        read, meta = load_policy(path)
        assert type(read) is type(policy)
        assert meta["kind"] == ("mlp" if name == "net" else "linear")
        assert read.choose_many(observations, masks) == policy.choose_many(
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


def test_the_mask_removes_a_row_from_every_kind() -> None:
    """A policy never returns a row the engine called illegal.

    The mask decides before the weights do. A policy that scored an illegal
    row highest would send it, and the engine would refuse it silently.
    """
    observations, masks = _inputs()
    masks[:, 0] = 1
    masks[:, 1:] = 0
    rng = np.random.default_rng(5)
    network = MLPPolicy.zeros(ACTIONS, FEATURES, HIDDEN)
    network = network.rebuild(rng.standard_normal(network.flat().size))
    linear = LinearPolicy(rng.standard_normal((ACTIONS, FEATURES + 1)))
    assert network.choose_many(observations, masks) == [0, 0, 0, 0]
    assert linear.choose_many(observations, masks) == [0, 0, 0, 0]
