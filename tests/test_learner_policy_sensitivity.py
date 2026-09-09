"""A choice must depend on the observation, and the suite must see when it does not.

A paid run published four policies that cannot play. Each one is a fixed
preference order over the action rows, and the legality answer of the engine
supplies what looks like situational play.[^1] The cause sits at the feature
layer: most encoded positions never move, so a weight over one of them adds a
fixed offset to one action row and does nothing else.[^2]

**The suite could not see it.** A worker replaced the feature encoder of the
policy modules with a function that returns a constant row, which reproduces
that defect exactly, and ran the learner suite. The baseline passed 236 items
of 236, and 230 of the 236 still passed under the blinded encoder.[^3]

This file holds the assertions that separate a policy which reads the world
from one which ignores it, and it holds the guard that keeps them honest.

# 1. Every sensitivity claim is a named helper

Each claim is a function of the drawn input and the engine shapes. A test
below calls one helper, and the meta-test calls every helper under the blinded
encoder and requires each one to raise. **A contributor who weakens a helper
makes the meta-test fail**, which is the whole reason the set is named.

The set is checked against the tree as well: every name in it must have a test
function of its own in this module, so a helper cannot be added without a test
that runs it unblinded.

# 2. The blinding is the project rule applied to the suite

The testing rule says to put the defect back and watch the test stay green.
That rule was written for one fixture at a time. This file applies it to a set
of tests: it blinds the encoder and requires the set to go red.[^4]

# 3. The stored files hold one preference order, and this says so

The only test that touched the four published files asserted that they load.
It built a real world and a loaded policy, and it asked nothing about what the
weights do.[^3] The test below asks. It plays the world forward and reads the
highest-scoring row with the mask out of it, and every stored file answers one
row at every decision.

**It does not assert that a stored file plays well, because none of them
does.** It asserts the shape the register measured, and it names the detector
that measured it. A stored file that reads the world would fail this test, and
that failure is progress rather than a defect.

References
----------
[^1]: Findings register, FND-707. ``docs/FINDINGS.md``

[^2]: Findings register, FND-708. ``docs/FINDINGS.md``

[^3]: Findings register, FND-709. ``docs/FINDINGS.md``

[^4]: Testing Rules, sections 1 and 2a. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import contextlib
import sys
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette import World
from cachette.learn import imitate as imitate_module
from cachette.learn import policy as policy_module
from cachette.learn import structured as structured_module
from cachette.learn.policy import (
    FeatureNormalizer,
    LinearPolicy,
    load_policy,
    masked_choices,
    preferred_rows,
)
from cachette.learn.signals import SignalCatalogue
from cachette.learn.structured import StructuredPolicy

if TYPE_CHECKING:
    from collections.abc import Callable, Iterator
    from types import ModuleType

    from learner_shapes import EngineShapes, WorldShapedInputs

DRAWS = 5
"""How many weight sets each sensitivity claim is stated over.

The claim is a property of the arithmetic and not of one draw, so each helper
draws several weight sets and requires the property of each.
"""

READOUT_SCALE = 0.2
"""How far a drawn structured policy sits from the untrained one.

The untrained structured policy holds a zero readout, so every score is zero
and no observation can move it. A first generation moves the readout, and this
is the scale of that move.
"""

STRUCTURED_SEED = 1000
"""The seed the drawn structured weights come from.

It sits away from the seeds the linear draws use, so the two kinds do not
share a draw.
"""

CHECKPOINTS = Path(__file__).resolve().parent.parent / "checkpoints"
"""Where the stored policies live.

The path comes from this file and not from the directory a runner started in.
"""

TICKS_BETWEEN_DECISIONS = 25
"""How many ticks pass between two observations of the stored-file test.

A decision interval of a run is of this order, so the stack this test reads is
the stack a player would drive the stored file over.
"""

STORED_ROWS = 8
"""How many decisions the stored-file test reads.

The register measured sixty decisions of one changing world and found one
preference row at every one of them. Eight decisions over two hundred ticks
reach the same answer and cost a fraction of the ticks.
"""


def stored_files() -> list[Path]:
    """Return every stored weight file, in a fixed order."""
    return sorted(CHECKPOINTS.rglob("*.npz"))


def blind_encode(
    observation: np.ndarray, normalizer: FeatureNormalizer | None = None
) -> np.ndarray:
    """Encode one observation as a constant row, whatever the observation holds.

    The body reads zero and the trailing bias entry reads one, so a policy
    scores from its weights alone. That is the recorded feature-layer defect
    reproduced exactly.[^1]

    References
    ----------
    [^1]: Findings register, FND-708. ``docs/FINDINGS.md``
    """
    del normalizer
    held = np.asarray(observation)
    return np.concatenate([np.zeros(held.shape[-1]), np.ones(1)])


def blind_encode_many(
    observations: np.ndarray, normalizer: FeatureNormalizer | None = None
) -> np.ndarray:
    """Encode a stack of observations as one constant row for every row."""
    del normalizer
    held = np.asarray(observations)
    body = np.zeros((held.shape[0], held.shape[1]))
    return np.concatenate([body, np.ones((held.shape[0], 1))], axis=1)


BLINDED = {
    "encode": blind_encode,
    "encode_many": blind_encode_many,
}
"""The two encoder names the blinding replaces."""

ENCODER_MODULES = (policy_module, structured_module, imitate_module)
"""The modules that hold a name the blinding replaces.

Each of the three imports the encoder into its own namespace, so a blinding
that patched one module would leave the other two reading the world.
"""


@contextlib.contextmanager
def blinded_encoder() -> Iterator[None]:
    """Replace the feature encoder of every policy module while the block runs.

    The replacement returns a constant row, so every policy scores from its
    weights alone and no observation can reach a score.
    """
    held: list[tuple[ModuleType, str, object]] = []
    for module in ENCODER_MODULES:
        for name, blind in BLINDED.items():
            if hasattr(module, name):
                held.append((module, name, getattr(module, name)))
                setattr(module, name, blind)
    try:
        yield
    finally:
        for module, name, original in held:
            setattr(module, name, original)


def a_drawn_linear_policy(
    action_length: int,
    observation_length: int,
    draw: int,
    normalizer: FeatureNormalizer | None = None,
) -> LinearPolicy:
    """Return a linear policy whose weights come from one draw."""
    rng = np.random.default_rng(draw)
    return LinearPolicy(
        rng.standard_normal((action_length, observation_length + 1)), normalizer
    )


def a_drawn_structured_policy(
    shapes: EngineShapes, draw: int, normalizer: FeatureNormalizer | None = None
) -> StructuredPolicy:
    """Return a structured policy over the published layout, moved off zero."""
    catalogue = SignalCatalogue.of_world(shapes.world)
    centre = StructuredPolicy.of_catalogue(
        shapes.action_length, catalogue, normalizer=normalizer
    )
    rng = np.random.default_rng(STRUCTURED_SEED + draw)
    return centre.rebuild(rng.standard_normal(centre.flat().size) * READOUT_SCALE)


def a_reference_normalizer(observations: np.ndarray) -> FeatureNormalizer:
    """Derive the normalizer of a stack, which is what a run stores.

    A position that never moves reads exactly zero after the subtraction, so
    its weight reaches no score. The remaining positions carry the score, and
    a choice can then depend on the world.[^1]

    References
    ----------
    [^1]: Findings register, FND-708. ``docs/FINDINGS.md``
    """
    return FeatureNormalizer.of_observations(observations)


def preference_count(scores: np.ndarray) -> int:
    """Count the distinct highest-scoring rows of a score stack, unmasked.

    **This is the detector.** The engine's legality answer is out of it, so a
    count of one says that the observation reached no score, whatever the
    masked choices did.[^1]

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    return len(set(preferred_rows(scores)))


def assert_the_rows_of_a_score_stack_differ(scores: np.ndarray, who: str) -> None:
    """Fail when every row of a score stack holds the same numbers.

    The stack comes from observations that differ, so two equal score rows say
    that the observation reached no score at all.
    """
    assert not np.allclose(scores, scores[0]), (
        f"{who} scored every observation of the stack alike, so its scores do "
        "not depend on the observation"
    )


def a_linear_score_moves_with_the_observation(
    inputs: WorldShapedInputs, shapes: EngineShapes
) -> None:
    """Require the unmasked linear score to differ between two observations.

    **The claim is on the unmasked score.** A mask can hide a constant, because
    a fixed preference order still answers many different rows when the legal
    set moves under it.
    """
    for draw in range(DRAWS):
        policy = a_drawn_linear_policy(
            shapes.action_length, shapes.observation_length, draw
        )
        assert_the_rows_of_a_score_stack_differ(
            policy.scores_many(inputs.observations), f"the linear draw {draw}"
        )


def a_structured_score_moves_with_the_observation(
    inputs: WorldShapedInputs, shapes: EngineShapes
) -> None:
    """Require the unmasked structured score to differ between two observations."""
    for draw in range(DRAWS):
        policy = a_drawn_structured_policy(shapes, draw)
        assert_the_rows_of_a_score_stack_differ(
            policy.scores_many(inputs.observations), f"the structured draw {draw}"
        )


def a_linear_choice_moves_with_the_observation(
    inputs: WorldShapedInputs, shapes: EngineShapes
) -> None:
    """Require the row a linear policy prefers to move with the observation.

    The policy reads through the normalizer a run stores, because that is the
    policy a run now trains. The mask is the same in every row of the stack, so
    a choice that moves is the observation moving it and not the legal set.
    """
    normalizer = a_reference_normalizer(inputs.observations)
    mask = np.tile(inputs.masks[0], (inputs.observations.shape[0], 1))
    moved = 0
    for draw in range(DRAWS):
        policy = a_drawn_linear_policy(
            shapes.action_length, shapes.observation_length, draw, normalizer
        )
        scores = policy.scores_many(inputs.observations)
        assert preference_count(scores) > 1, (
            f"the linear draw {draw} preferred one row at every observation of "
            "the stack, which is a preference order and not a policy"
        )
        moved += len(set(masked_choices(scores, mask))) > 1
    assert moved > 0, (
        "no linear draw changed the row it sent under one fixed mask, so the "
        "legal set is doing the whole of the choosing"
    )


def a_structured_choice_moves_with_the_observation(
    inputs: WorldShapedInputs, shapes: EngineShapes
) -> None:
    """Require the row a structured policy prefers to move with the observation."""
    normalizer = a_reference_normalizer(inputs.observations)
    mask = np.tile(inputs.masks[0], (inputs.observations.shape[0], 1))
    moved = 0
    for draw in range(DRAWS):
        policy = a_drawn_structured_policy(shapes, draw, normalizer)
        scores = policy.scores_many(inputs.observations)
        assert preference_count(scores) > 1, (
            f"the structured draw {draw} preferred one row at every observation "
            "of the stack, which is a preference order and not a policy"
        )
        moved += len(set(masked_choices(scores, mask))) > 1
    assert moved > 0, (
        "no structured draw changed the row it sent under one fixed mask, so "
        "the legal set is doing the whole of the choosing"
    )


def a_reading_policy_is_not_a_fixed_preference_order(
    inputs: WorldShapedInputs, shapes: EngineShapes
) -> None:
    """Require the detector to report several preferred rows for a reading policy.

    The stored files each answer one row, and the detector says so. **A
    detector that answered one row for every policy would say it too**, so the
    detector needs a policy that reads the world to prove it can separate the
    two.
    """
    normalizer = a_reference_normalizer(inputs.observations)
    policy = a_drawn_linear_policy(
        shapes.action_length, shapes.observation_length, 0, normalizer
    )
    scores = policy.scores_many(inputs.observations)
    assert preference_count(scores) > 1, (
        "the detector reported one preferred row for a policy that reads the "
        "world, so it cannot separate a preference order from a policy"
    )


SENSITIVITY_ASSERTIONS: dict[str, Callable[[WorldShapedInputs, EngineShapes], None]] = {
    "a_linear_score_moves_with_the_observation": (
        a_linear_score_moves_with_the_observation
    ),
    "a_structured_score_moves_with_the_observation": (
        a_structured_score_moves_with_the_observation
    ),
    "a_linear_choice_moves_with_the_observation": (
        a_linear_choice_moves_with_the_observation
    ),
    "a_structured_choice_moves_with_the_observation": (
        a_structured_choice_moves_with_the_observation
    ),
    "a_reading_policy_is_not_a_fixed_preference_order": (
        a_reading_policy_is_not_a_fixed_preference_order
    ),
}
"""The named set the blinding must turn red.

A helper that no longer fails under the blinded encoder no longer separates a
policy that reads the world from one that ignores it, whatever else it still
asserts.
"""


def test_the_world_shaped_stack_supplies_the_distribution_it_claims(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Read the fixture, and require the two shares the world was measured to hold.

    Every assertion in this file rests on the input. A fixture that drew every
    position independently, or that marked every row legal, would be the
    inverse of the world on both axes, and no assertion here would reach the
    case it exists for.[^1]

    References
    ----------
    [^1]: Findings register, FND-709. ``docs/FINDINGS.md``
    """
    observations = world_shaped_inputs.observations
    moved = np.flatnonzero((observations != observations[0]).any(axis=0))
    assert set(moved.tolist()) == set(world_shaped_inputs.varying_positions.tolist())
    assert world_shaped_inputs.constant_positions.size > (
        engine_shapes.observation_length // 2
    ), "the stack holds no constant majority, so a bias weight has nowhere to live"

    legal = world_shaped_inputs.legal_rows
    assert legal[0] == 0, "the engine always permits the no-op, and the mask must"
    assert legal.size < engine_shapes.action_length // 4, (
        "the mask permits too many rows, so the legal set states nothing"
    )
    for row in world_shaped_inputs.masks:
        assert np.array_equal(np.flatnonzero(row), legal)


def test_a_linear_score_moves_with_the_observation(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Vary the observation and require the unmasked linear score to move."""
    a_linear_score_moves_with_the_observation(world_shaped_inputs, engine_shapes)


def test_a_structured_score_moves_with_the_observation(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Vary the observation and require the unmasked structured score to move."""
    a_structured_score_moves_with_the_observation(world_shaped_inputs, engine_shapes)


def test_a_linear_choice_moves_with_the_observation(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Vary the observation and require the linear choice to move under one mask."""
    a_linear_choice_moves_with_the_observation(world_shaped_inputs, engine_shapes)


def test_a_structured_choice_moves_with_the_observation(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Vary the observation and require the structured choice to move."""
    a_structured_choice_moves_with_the_observation(world_shaped_inputs, engine_shapes)


def test_a_reading_policy_is_not_a_fixed_preference_order(
    world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """The detector separates a reading policy from a preference order."""
    a_reading_policy_is_not_a_fixed_preference_order(world_shaped_inputs, engine_shapes)


@pytest.mark.parametrize("name", sorted(SENSITIVITY_ASSERTIONS))
def test_every_named_sensitivity_assertion_fails_under_a_blinded_encoder(
    name: str, world_shaped_inputs: WorldShapedInputs, engine_shapes: EngineShapes
) -> None:
    """Put the recorded defect back, and require the named set to go red.

    This is the guard the register asked for. A suite earns no confidence from
    passing; it earns confidence from failing when the defect is present.[^1]

    References
    ----------
    [^1]: Findings register, FND-709. ``docs/FINDINGS.md``
    """
    with blinded_encoder(), pytest.raises(AssertionError):
        SENSITIVITY_ASSERTIONS[name](world_shaped_inputs, engine_shapes)


def test_the_blinding_puts_the_encoder_back() -> None:
    """A blinded encoder that outlived its block would blind the whole session."""
    before = [getattr(module, "encode_many", None) for module in ENCODER_MODULES]
    with blinded_encoder():
        assert policy_module.encode_many is blind_encode_many
    assert [
        getattr(module, "encode_many", None) for module in ENCODER_MODULES
    ] == before


def test_every_named_assertion_has_a_test_of_its_own() -> None:
    """Derive the tests from the tree, and compare against the named set.

    A helper with no test of its own runs only under the blinding, so nothing
    would say it passes when the encoder reads the world.
    """
    module = sys.modules[__name__]
    for name in SENSITIVITY_ASSERTIONS:
        assert hasattr(module, f"test_{name}"), (
            f"the named set holds {name!r} and this module holds no test that "
            "runs it unblinded"
        )


def stated_number(meta: dict[str, object], key: str) -> int:
    """Read one whole number a stored file states about its world."""
    value = meta[key]
    assert isinstance(value, int), f"the file states {key} as {value!r}"
    return value


def a_world_of(meta: dict[str, object], seed: int) -> World:
    """Build the world a stored file states, seeded so it publishes its schemas.

    **The file states the world.** A test that named one extent of its own
    would state a second declaration of the world a file was fitted on.
    """
    world = World(
        width=stated_number(meta, "width"),
        height=stated_number(meta, "height"),
        seed=seed,
        faction_count=stated_number(meta, "faction_count"),
    )
    list(world.seed_world())
    return world


def decisions_of_one_world(world: World, seat: int = 0) -> np.ndarray:
    """Return the observations one seat reads over several decisions of one world.

    The world runs forward between two reads, so the stack holds a changing
    world rather than a repeated one.
    """
    rows = []
    for _ in range(STORED_ROWS):
        rows.append(np.asarray(world.faction_observation(seat)))
        world.step(TICKS_BETWEEN_DECISIONS)
    return np.stack(rows)


@pytest.mark.parametrize("path", stored_files(), ids=lambda path: path.stem)
def test_every_stored_policy_holds_one_fixed_preference_order(path: Path) -> None:
    """Read what a stored file does, and not only that it loads.

    Every stored file answers one row at every decision of a changing world,
    with the mask out of it. That is the recorded shape, and this test is the
    statement of it.[^1]

    **A file that fails this reads the world.** Move its name out of the
    expectation and say so in the commit, because that is the repair this
    register asked for and not a defect in this test.

    References
    ----------
    [^1]: Findings register, FND-707. ``docs/FINDINGS.md``
    """
    policy, meta = load_policy(path)
    world = a_world_of(meta, seed=3)
    scores = policy.scores_many(decisions_of_one_world(world))
    assert preference_count(scores) == 1, (
        f"{path.name} preferred more than one row over the stack, so it reads "
        "the world. Move it out of the expectation of this test"
    )
