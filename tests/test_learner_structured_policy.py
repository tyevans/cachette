"""The structured policy reads the shape of the observation, and pays little.

The observation holds an egocentric ring stack, four sets of entity tokens,
and a few hundred scalars. A dense layer over the whole array would cost over
one hundred thousand trainable weights, and one generation of an evolution
strategy points the right way in proportion to the square root of the pair
count divided by the trainable count.[^1] Weight sharing is therefore the
whole design, and these tests check the two symmetries the sharing rests on.

**Two of these tests carry their own proof that they can fail.** Each of them
builds a tower with the defect put back and asserts that the property then
breaks. A test of a symmetry that never sees an asymmetric tower measures
nothing.[^2]

References
----------
[^1]: Findings register, FND-668. ``docs/FINDINGS.md``

[^2]: Testing Rules, sections 1 and 2. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette._core import World
from cachette.learn.env import Env, EnvConfig
from cachette.learn.layout import (
    LayoutError,
    ObservationLayout,
    RingBlock,
    TokenBlock,
    token_blocks,
)
from cachette.learn.picture import RingStack
from cachette.learn.policy import PolicyFit, PolicyFitError, encode_many, load_policy
from cachette.learn.reward import Weighting
from cachette.learn.search import shell_policy
from cachette.learn.signals import Signal, SignalCatalogue
from cachette.learn.structured import (
    POOL_STATISTICS,
    RingTower,
    StructuredPolicy,
    StructuredShape,
    TokenTower,
)

if TYPE_CHECKING:
    from pathlib import Path

# The world these tests play. It is small, because every assertion here is
# about arithmetic over one observation and not about the play.
WIDTH = 24
HEIGHT = 24
FACTIONS = 3
SEED = 7
READER = 0

# The field the engine marks as ring space. **The tests here hold no ring
# count, no sector count and no channel count.** The schema states every one of
# them, and a test that stated one would be a second declaration of the thing
# under test.
RING_FIELD = "ring_stack"


def _world() -> World:
    """Build the world every engine-driven test here plays."""
    return World(width=WIDTH, height=HEIGHT, faction_count=FACTIONS, seed=SEED)


def _layout_of(world: World) -> ObservationLayout:
    """Read the layout from the schema the world publishes.

    The schema is the one declaration of the layout, so this asks it and
    states nothing of its own. It marks the ring stack, names the channels of
    every field, states the cells of each ring, states which axis runs first,
    and publishes one field for each token set.
    """
    return ObservationLayout.of_catalogue(SignalCatalogue.of_world(world))


def _small_layout() -> ObservationLayout:
    """Build a layout that no engine publishes, for the arithmetic tests.

    The rings hold 1, 6 and 12 cells, which is the shape of the real stack in
    miniature: one centre with no direction, one ring of six, and one ring of
    twelve. A test that used the real layout would measure the engine.
    """
    stack = RingStack((1, 6, 12))
    ring = RingBlock.contiguous(0, stack, 4)
    tokens = token_blocks("tokens", ring.slots, (("first", 3, 5), ("second", 2, 7)))
    length = ring.slots + sum(block.slots for block in tokens) + 11
    return ObservationLayout(length=length, ring=ring, tokens=tokens)


def _rows(layout: ObservationLayout, rows: int = 4, seed: int = 3) -> np.ndarray:
    """Draw a stack of observation rows of one layout."""
    rng = np.random.default_rng(seed)
    return rng.integers(-40000, 40000, (rows, layout.length)).astype(np.int64)


class _PositionDependentTokenTower(TokenTower):
    """A token encoder that reads the slot a token sits in.

    This is the defect the permutation test exists to catch. A token position
    names no seat, so an encoder that reads the position states a rule the
    record forbids.
    """

    def embeddings(self, encoded: np.ndarray) -> np.ndarray:
        """Embed every token, and add an offset that names its slot."""
        base = super().embeddings(encoded)
        slots = np.arange(base.shape[1], dtype=np.float64)[None, :, None]
        return base + slots


class _FlatSectorRingTower(RingTower):
    """A ring tower whose sector kernel does not wrap.

    This is the defect the rotation test exists to catch. The sector axis is
    a circle, so a kernel that treats sector 0 and sector 11 as edges states
    that one direction is special.
    """

    def feature_map(self, encoded: np.ndarray) -> np.ndarray:
        """Run the same kernel with zeros outside the sector axis."""
        grid = self.rectangle(encoded)
        width = self.sector_kernel.shape[1]
        centre = width // 2
        acc = np.broadcast_to(self.channel_bias, grid.shape).copy()
        for step in range(width):
            shift = centre - step
            shifted = np.zeros_like(grid)
            if shift == 0:
                shifted = grid
            elif shift > 0:
                shifted[:, :, shift:, :] = grid[:, :, :-shift, :]
            else:
                shifted[:, :, :shift, :] = grid[:, :, -shift:, :]
            acc += shifted * self.sector_kernel[:, step]
        mixed = np.einsum("brsc,gc->brsg", np.tanh(acc), self.channel_mix)
        return np.asarray(np.tanh(mixed + self.mix_bias), dtype=np.float64)


def _shuffled(encoded: np.ndarray, block: TokenBlock, seed: int) -> np.ndarray:
    """Return the encoded rows with the tokens of one set put in a new order."""
    grid = block.gather()
    order = np.random.default_rng(seed).permutation(block.tokens)
    assert not np.array_equal(order, np.arange(block.tokens))
    moved = encoded.copy()
    moved[:, grid.reshape(-1)] = encoded[:, grid[order].reshape(-1)]
    return moved


def test_the_token_encoder_gives_one_answer_for_every_order_of_the_tokens() -> None:
    """A token position names no seat, so the pooled features must not move.

    The record says that a token position carries no identity, so a shuffle
    of the tokens is the same input. The encoder is shared and the pool is
    symmetric, so the features are the same to the last bit that a reordered
    sum can hold.
    """
    layout = _small_layout()
    rng = np.random.default_rng(11)
    block = layout.tokens[0]
    tower = TokenTower.initial(block, StructuredShape(), rng)
    encoded = encode_many(_rows(layout))
    before = tower.features(encoded)
    after = tower.features(_shuffled(encoded, block, seed=5))
    np.testing.assert_allclose(after, before, rtol=0.0, atol=1e-12)


def test_a_token_encoder_that_reads_the_slot_breaks_the_permutation_test() -> None:
    """Put the defect back, and the permutation assertion must fail.

    This is the proof that the test above can fail. The encoder here adds an
    offset that names the slot of a token, which is exactly the rule the
    record forbids, and the pooled features then move under a shuffle.
    """
    layout = _small_layout()
    rng = np.random.default_rng(11)
    block = layout.tokens[0]
    tower = _PositionDependentTokenTower(
        block, *TokenTower.initial(block, StructuredShape(), rng).arrays()
    )
    encoded = encode_many(_rows(layout))
    before = tower.features(encoded)
    after = tower.features(_shuffled(encoded, block, seed=5))
    assert not np.allclose(after, before, rtol=0.0, atol=1e-12)


def _rotated(encoded: np.ndarray, block: RingBlock, steps: int) -> np.ndarray:
    """Return the encoded rows with the ring stack turned by whole directions."""
    grid = block.gather()
    source = block.rotation(steps)
    moved = encoded.copy()
    moved[:, grid.reshape(-1)] = encoded[:, grid[source].reshape(-1)]
    return moved


def test_the_sector_kernel_turns_with_the_world() -> None:
    """A turn of one hex direction rolls the feature map and changes nothing else.

    The sector axis of the ring stack is a circle, and the kernel wraps
    around it. One weight set therefore serves every direction, which is the
    saving the whole architecture is built on. The map after a turn is the map
    before it, rolled by the sectors that one hex direction covers.

    The equality is exact. A roll is a reindex, and the kernel sums its taps
    in one order, so the arithmetic is the same arithmetic on the same values.
    """
    layout = _small_layout()
    block = layout.ring
    tower = RingTower.initial(block, StructuredShape(), np.random.default_rng(13))
    encoded = encode_many(_rows(layout))
    before = tower.feature_map(encoded)
    steps = 1
    after = tower.feature_map(_rotated(encoded, block, steps))
    shift = steps * block.sectors // 6
    np.testing.assert_array_equal(after, np.roll(before, shift, axis=2))


def test_the_pooled_ring_features_do_not_move_when_the_world_turns() -> None:
    """The readout reads a turn-invariant summary of the stack.

    No row of the action table names a hex direction, so the tower pools the
    sector axis away. The pooled features are therefore the same after a turn
    of the world, to the last bit that a reordered mean can hold.
    """
    layout = _small_layout()
    block = layout.ring
    tower = RingTower.initial(block, StructuredShape(), np.random.default_rng(13))
    encoded = encode_many(_rows(layout))
    before = tower.features(encoded)
    after = tower.features(_rotated(encoded, block, 1))
    np.testing.assert_allclose(after, before, rtol=0.0, atol=1e-12)


def test_a_sector_kernel_that_does_not_wrap_breaks_the_rotation_test() -> None:
    """Put the defect back, and the rotation assertion must fail.

    This is the proof that the two tests above can fail. The kernel here pads
    the sector axis with zeros instead of wrapping it, so sector 0 and the
    last sector become edges and one direction becomes special.
    """
    layout = _small_layout()
    block = layout.ring
    arrays = RingTower.initial(block, StructuredShape(), np.random.default_rng(13))
    tower = _FlatSectorRingTower(block, *arrays.arrays())
    encoded = encode_many(_rows(layout))
    before = tower.feature_map(encoded)
    after = tower.feature_map(_rotated(encoded, block, 1))
    shift = block.sectors // 6
    assert not np.allclose(after, np.roll(before, shift, axis=2))
    assert not np.allclose(
        tower.features(_rotated(encoded, block, 1)), tower.features(encoded)
    )


def _expected_count(
    layout: ObservationLayout, actions: int, shape: StructuredShape
) -> dict[str, int]:
    """Compute the trainable count of each part from the widths alone.

    This states the claim of the architecture as arithmetic. A later change
    that adds an array fails here rather than costing alignment nobody
    accounted for.
    """
    channels = layout.ring.channels
    scalars = shape.scalar_width * (layout.scalar_slots + 1)
    ring = (
        channels * shape.sector_kernel
        + channels
        + shape.ring_width * channels
        + shape.ring_width
        + shape.ring_bands * layout.ring.rings
    )
    tokens = sum((block.channels + 1) * shape.token_width for block in layout.tokens)
    features = (
        shape.scalar_width
        + shape.ring_bands * POOL_STATISTICS * shape.ring_width
        + len(layout.tokens) * POOL_STATISTICS * shape.token_width
    )
    trunk = shape.trunk_width * (features + 1)
    readout = actions * (shape.trunk_width + 1)
    return {
        "scalars": scalars,
        "ring": ring,
        "tokens": tokens,
        "trunk": trunk,
        "readout": readout,
        "total": scalars + ring + tokens + trunk + readout,
    }


def test_the_stated_layout_trains_the_count_the_design_claims() -> None:
    """The parts and the whole hold the counts the arithmetic gives.

    The exact integers below are a property of the layout this test states
    and of the default widths. They pin both, so a change to either is a
    change a reader sees.
    """
    layout = _small_layout()
    shape = StructuredShape()
    policy = StructuredPolicy.zeros(9, layout, shape)
    assert policy.counts() == _expected_count(layout, 9, shape)
    assert policy.counts() == {
        "scalars": 48,
        "ring": 45,
        "tokens": 56,
        "trunk": 540,
        "readout": 117,
        "total": 806,
    }
    assert policy.flat().size == policy.parameter_count


def test_the_engine_layout_trains_the_count_the_design_claims() -> None:
    """The count over the real observation follows the same arithmetic.

    The engine owns the layout, so this states no integer of it. It states
    the formula, which is the claim the architecture makes.

    **The bound is a dense read of the layout for every action row.** The
    readout holds one weight for each row it scores, so this count follows
    the action table. The layout length alone was the bound while the
    table held one row for each verb.
    """
    world = _world()
    layout = _layout_of(world)
    actions = int(world.action_schema()["length"])
    shape = StructuredShape()
    policy = StructuredPolicy.zeros(actions, layout, shape)
    assert policy.counts() == _expected_count(layout, actions, shape)
    assert policy.parameter_count < layout.length * actions


def test_every_position_of_the_engine_layout_belongs_to_exactly_one_part() -> None:
    """The three parts cover the observation and never overlap.

    The layout derives the scalar positions rather than stating them, so a
    field the engine adds joins the scalar tower rather than going unread.
    """
    layout = _layout_of(_world())
    covered = (
        layout.ring.slots
        + sum(block.slots for block in layout.tokens)
        + layout.scalar_slots
    )
    assert covered == layout.length


def test_the_untrained_policy_takes_the_no_op_on_the_engine_observation() -> None:
    """A fresh policy scores every row at zero and takes the first legal row.

    This drives the engine and reads the observation and the mask through the
    public interface of the world, so it proves that the architecture reaches
    a real observation rather than a synthetic array.
    """
    world = _world()
    layout = _layout_of(world)
    actions = int(world.action_schema()["length"])
    policy = StructuredPolicy.zeros(actions, layout)
    observation = np.asarray(world.faction_observation(READER), dtype=np.int64)
    mask = np.asarray(world.legal_actions(READER), dtype=np.uint8)
    assert observation.size == layout.length
    assert policy.choose(observation, mask) == 0
    stack = np.stack([observation, observation])
    masks = np.stack([mask, mask])
    assert policy.choose_many(stack, masks) == [0, 0]


def test_a_trained_policy_chooses_a_legal_row_of_the_engine_action_table() -> None:
    """A policy with a non-zero readout still chooses inside the mask.

    The mask decides before the weights do, so a policy that scores an
    illegal row highest must never take it.
    """
    world = _world()
    layout = _layout_of(world)
    actions = int(world.action_schema()["length"])
    shell = StructuredPolicy.zeros(actions, layout)
    rng = np.random.default_rng(19)
    policy = shell.rebuild(rng.standard_normal(shell.parameter_count))
    observation = np.asarray(world.faction_observation(READER), dtype=np.int64)
    mask = np.asarray(world.legal_actions(READER), dtype=np.uint8)
    chosen = policy.choose(observation, mask)
    assert mask[chosen] > 0


def test_a_rebuilt_policy_holds_the_weights_it_was_given() -> None:
    """The flat view and the rebuild are inverse, which the search needs.

    The search perturbs the flat vector and rebuilds a candidate from it, so
    a rebuild that lost a weight would train a policy nobody scored.
    """
    layout = _small_layout()
    shell = StructuredPolicy.zeros(9, layout)
    weights = np.random.default_rng(23).standard_normal(shell.parameter_count)
    rebuilt = shell.rebuild(weights)
    np.testing.assert_array_equal(rebuilt.flat(), weights)
    encoded = encode_many(_rows(layout))
    assert not np.allclose(rebuilt.scores(encoded), shell.scores(encoded))


def test_the_vector_holds_the_scalar_tower_and_the_scores_read_it() -> None:
    """The scalar tower is trainable, so the flat vector carries it.

    **This is the test the earlier design could not pass.** That design held
    the scalar layer at a fixed draw, kept it out of the vector, and trained
    nothing in it. A change to the scalar weights then reached no score,
    whatever the trainer did.

    The readout of a fresh policy is zero, so every score of it is zero and
    no change to an earlier layer can move a choice. The policy below
    therefore starts from a full vector of draws, which is what a run holds
    after its first generation.
    """
    layout = _small_layout()
    shell = StructuredPolicy.zeros(9, layout)
    scalars = shell.counts()["scalars"]
    assert scalars == shell.shape.scalar_width * (layout.scalar_slots + 1)
    np.testing.assert_array_equal(
        shell.flat()[:scalars], shell.scalars.weights.reshape(-1)
    )

    rng = np.random.default_rng(29)
    trained = shell.rebuild(rng.standard_normal(shell.parameter_count) * 0.5)
    moved = trained.flat().copy()
    moved[:scalars] += 1.5
    encoded = encode_many(_rows(layout))

    assert not np.allclose(
        trained.rebuild(moved).scores(encoded), trained.scores(encoded)
    )
    np.testing.assert_array_equal(
        trained.rebuild(moved).flat()[scalars:], trained.flat()[scalars:]
    )


def test_a_rebuild_refuses_a_vector_of_another_size() -> None:
    """A vector of the wrong length names another architecture."""
    shell = StructuredPolicy.zeros(9, _small_layout())
    with pytest.raises(ValueError, match="trainable weights"):
        shell.rebuild(np.zeros(shell.parameter_count + 1))


def _fit(observation_version: int) -> PolicyFit:
    """Build a fit that names one observation layout version."""
    return PolicyFit(
        observation_version=observation_version,
        action_version=1,
        observation_length=_small_layout().length,
        action_length=9,
        width=WIDTH,
        height=HEIGHT,
        faction_count=FACTIONS,
    )


def test_the_reader_refuses_a_structured_file_of_another_layout_version(
    tmp_path: Path,
) -> None:
    """A weight file is a function of one observation version, and says so.

    A new field or a new bound moves every position of the observation, so a
    file written under one version means something else under the next.
    """
    layout = _small_layout()
    policy = StructuredPolicy.zeros(9, layout)
    path = tmp_path / "structured.npz"
    policy.save(path, _fit(observation_version=5).as_meta())
    read, meta = load_policy(path, _fit(observation_version=5))
    assert meta["kind"] == "structured"
    assert isinstance(read, StructuredPolicy)
    np.testing.assert_array_equal(read.flat(), policy.flat())
    with pytest.raises(PolicyFitError, match="observation_version"):
        load_policy(path, _fit(observation_version=6))


def test_the_reader_refuses_a_structured_file_of_another_ring_geometry(
    tmp_path: Path,
) -> None:
    """One length holds many layouts, so the reader compares the layout too.

    A stack of 3775 positions is 151 cells of 25 channels, and it is also 25
    cells of 151 channels. The version and the length agree in both, so
    neither separates them. A stack of one cell count and one channel count
    still differs when the two axes run the other way round, so the layout
    compares the positions and not only the counts.
    """
    layout = _small_layout()
    policy = StructuredPolicy.zeros(9, layout)
    path = tmp_path / "structured.npz"
    policy.save(path, _fit(observation_version=5).as_meta())
    turned = ObservationLayout(
        length=layout.length,
        ring=RingBlock.contiguous(0, RingStack((1, 6, 12)), 4, cell_major=False),
        tokens=layout.tokens,
    )
    assert turned.fingerprint() != layout.fingerprint()
    with pytest.raises(PolicyFitError, match="another observation layout"):
        load_policy(path, _fit(observation_version=5), turned)
    regrouped = ObservationLayout(
        length=layout.length,
        ring=RingBlock.contiguous(0, RingStack((1, 6, 6, 6)), 4),
        tokens=token_blocks("tokens", 76, (("first", 3, 5), ("second", 2, 7))),
    )
    with pytest.raises(PolicyFitError, match="another observation layout"):
        load_policy(path, _fit(observation_version=5), regrouped)
    read, _ = load_policy(path, _fit(observation_version=5), layout)
    np.testing.assert_array_equal(read.flat(), policy.flat())


def _catalogue(geometry: dict[str, object], signals: list[Signal]) -> SignalCatalogue:
    """Build a catalogue of one stated layout, without an engine."""
    length = max(signal.start + signal.positions for signal in signals)
    return SignalCatalogue(signals, length, geometry)


def test_the_layout_refuses_a_schema_that_marks_no_ring_field() -> None:
    """A layout states no position, so it fails rather than guessing one.

    The message names the schema entry to add, because a reader that guessed
    would read a plausible observation that does not exist.
    """
    catalogue = _catalogue({}, [Signal(name="ring_stack", start=0, positions=48)])
    with pytest.raises(LayoutError, match="marks no field as ring space"):
        ObservationLayout.of_catalogue(catalogue)


def test_the_layout_refuses_a_ring_field_with_no_ring_geometry() -> None:
    """A marked field without a ring count states no cell count."""
    catalogue = _catalogue(
        {}, [Signal(name="ring_stack", start=0, positions=48, space="ring")]
    )
    with pytest.raises(LayoutError, match="ring_cells"):
        ObservationLayout.of_catalogue(catalogue)


def test_the_layout_refuses_a_ring_field_with_no_channel_order() -> None:
    """A field of several channels over several cells needs the axis order."""
    catalogue = _catalogue(
        {"rings": 3, "sectors": 12},
        [Signal(name="ring_stack", start=0, positions=76, space="ring")],
    )
    with pytest.raises(LayoutError, match="channel_order"):
        ObservationLayout.of_catalogue(catalogue)


def test_the_layout_refuses_a_schema_that_marks_no_token_field() -> None:
    """A layout needs the token sets, and one field cannot state four shapes."""
    catalogue = _catalogue(
        {"rings": 3, "sectors": 12, "channel_order": "cell_major"},
        [Signal(name="ring_stack", start=0, positions=76, space="ring")],
    )
    with pytest.raises(LayoutError, match="marks no field as token space"):
        ObservationLayout.of_catalogue(catalogue)


def test_the_layout_reads_a_schema_that_states_every_entry() -> None:
    """A schema that carries the entries needs no statement from a caller.

    This is the layout the engine must publish. When it does, the fallback in
    these tests stops running and nothing else changes.
    """
    channels = tuple(f"channel_{index}" for index in range(4))
    catalogue = _catalogue(
        {"rings": 3, "sectors": 12, "channel_order": "cell_major"},
        [
            Signal(
                name="ring_stack",
                start=0,
                positions=76,
                space="ring",
                channels=channels,
            ),
            Signal(
                name="own_settlements",
                start=76,
                positions=15,
                space="token",
                channels=("a", "b", "c", "d", "e"),
            ),
            Signal(name="tick_share", start=91, positions=1),
        ],
    )
    layout = ObservationLayout.of_catalogue(catalogue)
    assert layout.ring.stack.ring_cells == (1, 6, 12)
    assert layout.ring.channels == 4
    assert [block.tokens for block in layout.tokens] == [3]
    assert layout.scalars == (91,)


def test_the_layout_refuses_two_parts_that_claim_one_position() -> None:
    """A position belongs to one part, so an overlap is a defect not a warning."""
    stack = RingStack((1, 6))
    with pytest.raises(LayoutError, match="already claims"):
        ObservationLayout(
            length=60,
            ring=RingBlock.contiguous(0, stack, 4),
            tokens=token_blocks("tokens", 20, (("first", 2, 5),)),
        )


def test_the_search_builds_the_structured_shell_from_the_probe_schema() -> None:
    """The trainer builds this policy through one seam, and this drives it.

    The trainer and every worker process call the same builder, so a kind the
    builder does not know would train nothing. This drives that builder with
    a real probe environment.

    The schema of the engine carries the ring geometry, so the builder gives
    a policy. A schema that stopped carrying it would raise here, and the
    message would name the entries to add.

    **The bound is a dense read of the observation for every action row.**
    The readout holds one weight for each row it scores, so the count of
    this policy follows the action table. The observation length alone was
    the bound while the table held one row for each verb, and a place
    argument gives it one row for each cell of the frame.
    """
    config = EnvConfig(
        width=WIDTH,
        height=HEIGHT,
        faction_count=FACTIONS,
        seat=READER,
        tick_limit=40,
        horizon=4,
        decision_interval=10,
    )
    probe = Env(config, Weighting(terms={}, won=0.0, lost=0.0, drawn=0.0))
    policy = shell_policy("structured", probe, 32)
    assert isinstance(policy, StructuredPolicy)
    assert policy.parameter_count < probe.observation_length * probe.action_length
