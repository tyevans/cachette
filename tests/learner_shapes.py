"""The shapes a learner test draws its input from.

A fixture supplies the input, and a uniform input hides a defect. The world
holds a majority of observation positions that never move, and it permits a
small minority of the action rows at any decision. A fixture that drew every
position independently and marked every row legal would be the inverse of the
world on both axes, and a defect that lives in the constant subspace could not
exist in it.[^1] [^2]

The lengths come from the engine schema. A test that stated a length would
hold a second declaration of something the engine already declares.[^3]

The fixtures that hand these to a test sit beside this module.[^4]

References
----------
[^1]: Findings register, FND-708 and FND-709. ``docs/FINDINGS.md``

[^2]: Testing Rules, section 2a, on what a fixture must supply.
``.agents/rules/testing.md``

[^3]: Recurring defect shapes, shape 1.
``.agents/rules/recurring-defects.md``

[^4]: The shared fixtures. ``tests/conftest.py``
"""

from __future__ import annotations

import dataclasses

import numpy as np

from cachette import World

STACK_ROWS = 6
"""How many rows one world-shaped stack holds.

Six rows stand for six decisions of one episode. That is the smallest stack on
which a preference order can be seen to move or to stand still.
"""

CONSTANT_SHARE = 0.74
"""The share of observation positions that hold one value for every row.

The world holds 3,546 positions of 4,819 that never move, which is 0.736. This
share is that figure to two places.
"""

LEGAL_SHARE = 0.086
"""The share of action rows the mask permits.

The world permits 13 to 18 rows of 180 at a decision, and the middle of that
band is 0.086.
"""

STACK_SEED = 17
"""The seed every world-shaped stack draws from."""

FIELD_CEILING = 5000
"""The highest value one observation position of a stack holds.

The encoder takes the signed logarithm of a position, so the exact ceiling
decides nothing. It is above one thousand, so several orders of magnitude
reach the encoder.
"""

FIXTURE_EXTENT = 48
FIXTURE_FACTIONS = 3
FIXTURE_SEED = 0
"""The world the shapes come from.

It is the extent and the faction count the stored policy index names, so a
fixture shape is the shape a stored file was fitted on.
"""


@dataclasses.dataclass(frozen=True)
class EngineShapes:
    """The two lengths one world publishes, and the world that published them.

    A caller that needs the signal catalogue of the world reads it from the
    world this holds, rather than building a second one.
    """

    world: World
    action_length: int
    observation_length: int


@dataclasses.dataclass(frozen=True)
class WorldShapedInputs:
    """An observation stack and a mask stack shaped like the world.

    The stack holds a majority of positions that never move, and the mask
    permits a small minority of the action rows.
    """

    observations: np.ndarray
    masks: np.ndarray
    constant_positions: np.ndarray
    varying_positions: np.ndarray
    legal_rows: np.ndarray


def engine_world() -> World:
    """Build the world the shapes come from, seeded so it publishes its schemas."""
    world = World(
        width=FIXTURE_EXTENT,
        height=FIXTURE_EXTENT,
        seed=FIXTURE_SEED,
        faction_count=FIXTURE_FACTIONS,
    )
    list(world.seed_world())
    return world


def engine_shapes_of(world: World) -> EngineShapes:
    """Read the action length and the observation length one world publishes."""
    return EngineShapes(
        world=world,
        action_length=int(world.action_schema()["length"]),
        observation_length=int(world.observation_schema()["length"]),
    )


def world_shaped_stack(
    action_length: int,
    observation_length: int,
    rows: int = STACK_ROWS,
    seed: int = STACK_SEED,
) -> WorldShapedInputs:
    """Draw an observation stack and a mask stack that look like the world.

    A stated majority of the positions holds one value for every row, and the
    rest vary from row to row. A stated minority of the action rows is legal,
    and row zero is legal in every row, because the engine always permits the
    no-op.

    **A uniform fixture cannot hold the defect this shape exists for.** A
    weight over a position that never moves adds a fixed offset to one action
    row, and it does nothing else. A fixture whose every position moves holds
    no constant subspace for such a weight to live in.[^1]

    References
    ----------
    [^1]: Findings register, FND-708 and FND-709. ``docs/FINDINGS.md``
    """
    rng = np.random.default_rng(seed)
    order = rng.permutation(observation_length)
    cut = round(CONSTANT_SHARE * observation_length)
    constant, varying = np.sort(order[:cut]), np.sort(order[cut:])

    base = rng.integers(0, FIELD_CEILING, observation_length)
    observations = np.tile(base, (rows, 1)).astype(np.int64)
    observations[:, varying] = rng.integers(0, FIELD_CEILING, (rows, varying.size))

    legal_count = max(2, round(LEGAL_SHARE * action_length))
    drawn = rng.permutation(action_length - 1)[: legal_count - 1] + 1
    masks = np.zeros((rows, action_length), dtype=np.uint8)
    masks[:, 0] = 1
    masks[:, drawn] = 1
    legal = np.sort(np.concatenate([np.zeros(1, dtype=np.int64), drawn]))
    return WorldShapedInputs(
        observations=observations,
        masks=masks,
        constant_positions=constant,
        varying_positions=varying,
        legal_rows=legal,
    )
