"""A policy that reads the shape of the observation and pays little for it.

The observation is not a flat vector. It holds an egocentric ring stack, four
sets of entity tokens, and a few hundred scalars.[^1] This policy treats each
of the three according to its own shape, then combines them.

# Why a dense layer over the observation is the wrong answer

This project trains by evolution strategies. One measurement governs the whole
design: the cosine between the step one generation takes and the direction it
is trying to find is near the square root of the pair count divided by the
trainable count.[^2] Every trainable weight therefore costs alignment at a
fixed population.

A dense first layer over the whole observation costs the width of the
observation for each feature it gives. At the current layout that is over one
hundred thousand weights, and the step it buys points almost nowhere.

The answer is weight sharing. A structure that uses the geometry of the
observation sees the geometry and pays for far fewer weights.

# The three towers

**The ring tower** views the stack as rings by sectors by channels. The sector
axis wraps, so one kernel over that axis serves every direction at once. The
policy learns "push toward the sector that holds the most unclaimed land" one
time instead of one time for each direction.

The ring axis does not wrap and its resolution falls with distance, so no
kernel runs along it. Each ring keeps its own row, and one learned matrix
mixes the rings into a few bands. The policy therefore chooses its own notion
of near and far, and it never shares a weight between two rings that cover
different amounts of ground.

The record says that the axes of the ring frame do not turn between
decisions.[^4] The sharing is therefore a prior over situations and never a
re-framing at run time: the policy states one rule for every direction,
rather than reading the world in a frame that moves.

The stack is not rectangular. Ring 0 holds one cell and has no direction, and
ring 1 holds six cells against twelve in every wider ring. The tower repeats
the inner cells to fill the sector axis. The repetition costs no weight, it
keeps the sector axis uniform, and a rotation by one hex direction stays a
shift on that axis in every ring. The alternative is a second path for the
inner rings, which states the same rule twice.

**The token tower** runs one shared encoder over every token of a set, then
pools the results with a mean and a maximum. A token position names no seat
and carries no identity, so this is the only form the record admits.[^3] It
costs one encoder for each set rather than one weight for each slot.

An absent token reads zero in every channel, including its validity channel.
The encoder sees that channel, so it can learn to answer a constant for an
absent token.

**The scalar tower** is a dense trainable layer, and it is the one place
this policy pays position by position. The scalar blocks hold no structure to
share a weight across, so no kernel and no pool applies to them. Its width is
therefore the largest single term in the trainable count, and it is the knob
that buys reading power against alignment.[^2]

**No layer of this policy is frozen.** A layer the trainer never moves states
a rule the run cannot revise, and this project rejects that shape. Every layer
except the readout starts at a draw, because a layer of zeros behind another
layer of zeros gives no change under any perturbation.

# The trunk and the readout

One small layer joins the three towers, and one readout scores each row of the
action table. The readout starts at zero, so an untrained policy takes the
no-op at every decision and is the baseline every trained policy is measured
against.

References
----------
[^1]: Report 42, what a policy should be able to see, sections 5, 9 and 11.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^2]: Findings register, FND-668. ``docs/FINDINGS.md``

[^3]: ADR-0195, the observation of a faction is a fixed-width scale-free
table, decision D4.
``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``

[^4]: ADR-0195, decision D3.
``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

import numpy as np

from .layout import ObservationLayout, RingBlock, TokenBlock
from .picture import RingStack
from .policy import (
    FeatureNormalizer,
    PolicyFitError,
    encode,
    encode_many,
    masked_choices,
)

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence
    from pathlib import Path

    import numpy.typing as npt

    from .signals import SignalCatalogue


STRUCTURED_KIND = "structured"

# The seed the start of this policy is drawn from. The trainer and every
# worker process build the same shell, so the draw must not depend on
# anything a process holds.
SHELL_SEED = 20260908

# How many statistics each pool gives. A mean carries the whole set and a
# maximum carries its extreme, and neither implies the other.
POOL_STATISTICS = 2


class ShapeError(ValueError):
    """A weight vector or a weight file does not fit this architecture."""


@dataclass(frozen=True)
class StructuredShape:
    """Every width of the architecture, and nothing about one world.

    Each width trades what the policy can state against how well one
    generation points the right way. The cosine between the step one
    generation takes and the direction it looks for is near the square root of
    the pair count divided by the trainable count.[^1]

    **The scalar width is the dominant term.** The scalar tower holds one
    weight for each unstructured position and each feature, and every other
    tower shares its weights. A caller that wants a better aligned step lowers
    this width first.

    **The default holds the whole trainable count below the length of the
    observation.** That is the claim this architecture makes against a dense
    layer, and a wider scalar tower breaks it. A test asserts it against the
    layout the engine publishes.

    References
    ----------
    [^1]: Findings register, FND-668. ``docs/FINDINGS.md``
    """

    scalar_width: int = 4
    ring_width: int = 4
    ring_bands: int = 3
    sector_kernel: int = 3
    token_width: int = 4
    trunk_width: int = 12

    def __post_init__(self) -> None:
        """Refuse a width that cannot describe a network."""
        for name in (
            "scalar_width",
            "ring_width",
            "ring_bands",
            "sector_kernel",
            "token_width",
            "trunk_width",
        ):
            value = int(getattr(self, name))
            if value < 1:
                message = f"{name} is at least one, and this states {value}"
                raise ShapeError(message)
        if self.sector_kernel % 2 == 0:
            message = (
                f"the sector kernel holds an odd width, so that it centres on "
                f"a sector, and this states {self.sector_kernel}"
            )
            raise ShapeError(message)


def _split(
    flat: npt.NDArray[np.float64], shapes: Sequence[tuple[int, ...]]
) -> list[np.ndarray]:
    """Cut one flat vector into arrays of the given shapes, in order."""
    wanted = int(sum(int(np.prod(shape)) for shape in shapes))
    if flat.size != wanted:
        message = (
            f"this architecture holds {wanted} trainable weights and was "
            f"given a vector of {flat.size}"
        )
        raise ShapeError(message)
    parts = []
    walked = 0
    for shape in shapes:
        size = int(np.prod(shape))
        parts.append(flat[walked : walked + size].reshape(shape))
        walked += size
    return parts


def _pool(values: npt.NDArray[np.float64], axis: int) -> npt.NDArray[np.float64]:
    """Reduce one axis of a feature map to a mean and a maximum.

    Both statistics are symmetric in the axis, so the result does not depend
    on the order of the entries along it.
    """
    return np.concatenate([values.mean(axis=axis), values.max(axis=axis)], axis=-1)


def _agree(stored: Mapping[str, np.ndarray], key: str, held: int) -> None:
    """Refuse a file whose fit disagrees with the architecture it holds.

    The fit of a world names the observation length and the action row count.
    The architecture names both a second time, because a file must rebuild
    without a world. A second declaration needs a check that fails when the
    copies disagree.
    """
    if key not in stored:
        return
    written = int(np.asarray(stored[key]).reshape(-1)[0])
    if written != held:
        message = (
            f"this weight file states {key} as {written} and the architecture "
            f"it holds states {held}. The file is not consistent with itself."
        )
        raise ShapeError(message)


class ScalarTower:
    """The trainable dense layer over every unstructured position.

    The scalar blocks hold a few hundred positions with no structure, so no
    weight can be shared across them. This layer therefore states one weight
    for each position and each feature, and it holds the largest single term
    of the trainable count.

    **The trainer moves every weight of this layer.** An earlier design held
    the layer at a fixed draw and trained nothing in it, which stated a rule
    that no run could revise.
    """

    def __init__(self, positions: npt.NDArray[np.int64], weights: np.ndarray) -> None:
        """Take the positions this reads and the weights it reads them with."""
        self.positions = np.asarray(positions, dtype=np.int64)
        self.weights = np.asarray(weights, dtype=np.float64)

    @classmethod
    def initial(
        cls, layout: ObservationLayout, width: int, rng: np.random.Generator
    ) -> ScalarTower:
        """Build the layer a fresh run starts from, over one layout.

        The caller owns the stream of draws, so every initial weight of one
        policy comes from one draw sequence. Two streams from one seed would
        give this layer and the first tower the same numbers.

        **This layer starts at a draw and not at zero.** A layer of zeros
        gives a zero change under every perturbation of the layer above it, so
        a network of zero layers never leaves the origin.

        The bias position of the encoded observation joins the scalar
        positions, so the layer carries an offset without a second array.
        """
        positions = np.asarray([*layout.scalars, layout.length], dtype=np.int64)
        scale = 1.0 / np.sqrt(positions.size)
        return cls(positions, rng.standard_normal((width, positions.size)) * scale)

    @property
    def shapes(self) -> tuple[tuple[int, ...], ...]:
        """The shape of each trainable array, in the order the vector holds them."""
        return (self.weights.shape,)

    def arrays(self) -> tuple[np.ndarray, ...]:
        """Every trainable array, in the order the vector holds them."""
        return (self.weights,)

    def with_arrays(self, arrays: Sequence[np.ndarray]) -> ScalarTower:
        """Give back a layer over these positions with the given weights."""
        return ScalarTower(self.positions, *arrays)

    @property
    def parameter_count(self) -> int:
        """How many weights this layer trains."""
        return int(sum(array.size for array in self.arrays()))

    @property
    def width(self) -> int:
        """How many features this gives the trunk."""
        return int(self.weights.shape[0])

    def features(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Give one feature row for each encoded observation row."""
        return np.tanh(encoded[:, self.positions] @ self.weights.T)


class RingTower:
    """The trainable tower over the egocentric ring stack.

    The tower runs one kernel along the sector axis with wraparound, mixes the
    channels, pools each ring over its sectors, then mixes the rings into a
    few bands.

    Every weight of the first two steps is shared across every ring and every
    sector. That sharing is the reason this architecture can read 3775 ring
    positions for a few hundred weights.
    """

    def __init__(
        self,
        block: RingBlock,
        sector_kernel: np.ndarray,
        channel_bias: np.ndarray,
        channel_mix: np.ndarray,
        mix_bias: np.ndarray,
        bands: np.ndarray,
    ) -> None:
        """Take the block this reads and the five weight arrays it reads it with."""
        self.block = block
        self.sector_kernel = np.asarray(sector_kernel, dtype=np.float64)
        self.channel_bias = np.asarray(channel_bias, dtype=np.float64)
        self.channel_mix = np.asarray(channel_mix, dtype=np.float64)
        self.mix_bias = np.asarray(mix_bias, dtype=np.float64)
        self.bands = np.asarray(bands, dtype=np.float64)
        self._cells = block.gather().reshape(-1)
        self._rectangle = block.rectangle().reshape(-1)

    @classmethod
    def initial(
        cls, block: RingBlock, shape: StructuredShape, rng: np.random.Generator
    ) -> RingTower:
        """Build the tower a fresh run starts from.

        **No trainable layer starts at zero except the readout.** A layer of
        zeros gives a zero change under every perturbation of the layer above
        it, so a network of zero layers never leaves the origin.
        """
        channels = block.channels
        return cls(
            block,
            rng.standard_normal((channels, shape.sector_kernel))
            / np.sqrt(shape.sector_kernel),
            np.zeros(channels),
            rng.standard_normal((shape.ring_width, channels)) / np.sqrt(channels),
            np.zeros(shape.ring_width),
            rng.standard_normal((shape.ring_bands, block.rings)) / np.sqrt(block.rings),
        )

    @property
    def shapes(self) -> tuple[tuple[int, ...], ...]:
        """The shape of each trainable array, in the order the vector holds them."""
        return (
            self.sector_kernel.shape,
            self.channel_bias.shape,
            self.channel_mix.shape,
            self.mix_bias.shape,
            self.bands.shape,
        )

    def arrays(self) -> tuple[np.ndarray, ...]:
        """Every trainable array, in the order the vector holds them."""
        return (
            self.sector_kernel,
            self.channel_bias,
            self.channel_mix,
            self.mix_bias,
            self.bands,
        )

    def with_arrays(self, arrays: Sequence[np.ndarray]) -> RingTower:
        """Give back a tower of this geometry with the given weights."""
        return RingTower(self.block, *arrays)

    @property
    def parameter_count(self) -> int:
        """How many weights this tower trains."""
        return int(sum(array.size for array in self.arrays()))

    @property
    def width(self) -> int:
        """How many features this gives the trunk."""
        bands = int(self.bands.shape[0])
        return bands * POOL_STATISTICS * int(self.channel_mix.shape[0])

    def rectangle(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Give the rectangular view of the stack, as rings by sectors by channels.

        The inner rings hold fewer cells than the widest ring, so this repeats
        each of their cells to fill the sector axis.
        """
        rows = encoded.shape[0]
        cells = encoded[:, self._cells].reshape(
            rows, self.block.cells, self.block.channels
        )
        return cells[:, self._rectangle, :].reshape(
            rows, self.block.rings, self.block.sectors, self.block.channels
        )

    def feature_map(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Give the feature map before any pool, as rings by sectors by features.

        **A rotation of the world by one hex direction rolls this map along
        the sector axis and changes nothing else.** That equivariance is the
        property the whole design rests on, and it is the property to test.
        """
        grid = self.rectangle(encoded)
        width = self.sector_kernel.shape[1]
        centre = width // 2
        acc = np.broadcast_to(self.channel_bias, grid.shape).copy()
        for step in range(width):
            rolled = np.roll(grid, centre - step, axis=2)
            acc += rolled * self.sector_kernel[:, step]
        mixed = np.einsum("brsc,gc->brsg", np.tanh(acc), self.channel_mix)
        return np.asarray(np.tanh(mixed + self.mix_bias), dtype=np.float64)

    def features(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Give one feature row for each encoded observation row.

        The pool over the sector axis makes this invariant to a rotation of
        the world. No row of the action table names a hex direction, so the
        readout needs no direction and loses nothing to the pool.
        """
        pooled = _pool(self.feature_map(encoded), axis=2)
        joined = np.einsum("brf,er->bef", pooled, self.bands)
        return np.asarray(joined.reshape(encoded.shape[0], -1), dtype=np.float64)


class TokenTower:
    """The trainable tower over one entity token set.

    One encoder runs over every token, and a mean and a maximum pool the
    results. The output therefore does not depend on the order of the tokens,
    which is what the record requires of a reader.[^1]

    References
    ----------
    [^1]: ADR-0195, the observation of a faction is a fixed-width scale-free
    table, decision D4.
    ``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``
    """

    def __init__(
        self, block: TokenBlock, embed: np.ndarray, embed_bias: np.ndarray
    ) -> None:
        """Take the set this reads and the encoder it reads every token with."""
        self.block = block
        self.embed = np.asarray(embed, dtype=np.float64)
        self.embed_bias = np.asarray(embed_bias, dtype=np.float64)
        self._positions = block.gather().reshape(-1)

    @classmethod
    def initial(
        cls, block: TokenBlock, shape: StructuredShape, rng: np.random.Generator
    ) -> TokenTower:
        """Build the tower a fresh run starts from."""
        return cls(
            block,
            rng.standard_normal((shape.token_width, block.channels))
            / np.sqrt(block.channels),
            np.zeros(shape.token_width),
        )

    @property
    def shapes(self) -> tuple[tuple[int, ...], ...]:
        """The shape of each trainable array, in the order the vector holds them."""
        return (self.embed.shape, self.embed_bias.shape)

    def arrays(self) -> tuple[np.ndarray, ...]:
        """Every trainable array, in the order the vector holds them."""
        return (self.embed, self.embed_bias)

    def with_arrays(self, arrays: Sequence[np.ndarray]) -> TokenTower:
        """Give back a tower of this set with the given weights."""
        return TokenTower(self.block, *arrays)

    @property
    def parameter_count(self) -> int:
        """How many weights this tower trains."""
        return int(sum(array.size for array in self.arrays()))

    @property
    def width(self) -> int:
        """How many features this gives the trunk."""
        return int(self.embed.shape[0]) * POOL_STATISTICS

    def embeddings(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Give one embedding for each token of each encoded observation row."""
        rows = encoded.shape[0]
        tokens = encoded[:, self._positions].reshape(
            rows, self.block.tokens, self.block.channels
        )
        mixed = np.einsum("btc,ec->bte", tokens, self.embed) + self.embed_bias
        return np.asarray(np.tanh(mixed), dtype=np.float64)

    def features(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Give one feature row for each encoded observation row."""
        return _pool(self.embeddings(encoded), axis=1)


class StructuredPolicy:
    """The block-aware policy: three towers, one trunk, one readout.

    The policy holds the layout it reads. A stored weight file holds the same
    layout, and the reader refuses a file whose layout differs, because the
    length alone does not separate two layouts.

    The policy also holds the feature normalizer its towers read through. A
    tower reads a standardized feature, so the normalizer is part of what the
    weights mean, and it travels into the weight file beside them. A policy
    that holds none reads the plain squash.
    """

    # **A positive scaling of the weights moves the choice.** Every tower ends
    # in a ``tanh`` and the trunk does too. Scaling the weight vector puts each
    # ``tanh`` at another place on its curve, and a saturating function is not
    # linear. The bias arrays are a second reason: a bias meets a constant
    # feature of value one, so it scales while what it is added to does not
    # scale by the same factor.
    #
    # The scores are therefore not the old scores times one positive number,
    # and the highest-scoring legal row can change. The search reads this and
    # never normalises the centre of this kind.
    CHOICE_SURVIVES_SCALING = False

    def __init__(
        self,
        layout: ObservationLayout,
        shape: StructuredShape,
        scalars: ScalarTower,
        ring: RingTower,
        tokens: Sequence[TokenTower],
        trunk: np.ndarray,
        readout: np.ndarray,
        normalizer: FeatureNormalizer | None = None,
    ) -> None:
        """Take the layout, the widths, the towers, the matrices and the normalizer."""
        self.layout = layout
        self.shape = shape
        self.scalars = scalars
        self.ring = ring
        self.tokens = tuple(tokens)
        self.trunk = np.asarray(trunk, dtype=np.float64)
        self.readout = np.asarray(readout, dtype=np.float64)
        self.normalizer = normalizer

    @classmethod
    def zeros(
        cls,
        action_length: int,
        layout: ObservationLayout,
        shape: StructuredShape | None = None,
        seed: int = SHELL_SEED,
        normalizer: FeatureNormalizer | None = None,
    ) -> StructuredPolicy:
        """Build the policy a fresh run starts from.

        **The readout is zero and nothing else is.** A zero readout scores
        every row of the action table at zero, so this policy takes the no-op
        at every decision, in the way the untrained linear policy does. Every
        earlier layer holds a draw from one fixed seed, because a layer of
        zeros would never leave the origin.

        The first generation therefore moves the readout alone, and the towers
        start to move once the readout is not zero.
        """
        chosen = shape or StructuredShape()
        rng = np.random.default_rng(seed)
        scalars = ScalarTower.initial(layout, chosen.scalar_width, rng)
        ring = RingTower.initial(layout.ring, chosen, rng)
        towers = [TokenTower.initial(block, chosen, rng) for block in layout.tokens]
        width = scalars.width + ring.width + sum(tower.width for tower in towers)
        trunk = rng.standard_normal((chosen.trunk_width, width + 1)) / np.sqrt(
            width + 1
        )
        readout = np.zeros((action_length, chosen.trunk_width + 1))
        return cls(layout, chosen, scalars, ring, towers, trunk, readout, normalizer)

    @classmethod
    def of_catalogue(
        cls,
        action_length: int,
        catalogue: SignalCatalogue,
        shape: StructuredShape | None = None,
        normalizer: FeatureNormalizer | None = None,
    ) -> StructuredPolicy:
        """Build the policy a fresh run starts from, reading the schema.

        Raises ``LayoutError`` when the schema states no ring geometry and no
        token set, and the message names the entries to add.
        """
        layout = ObservationLayout.of_catalogue(catalogue)
        return cls.zeros(action_length, layout, shape, normalizer=normalizer)

    @property
    def action_length(self) -> int:
        """How many rows the action table holds."""
        return int(self.readout.shape[0])

    @property
    def trunk_width(self) -> int:
        """How many features the trunk gives the readout."""
        return int(self.trunk.shape[0])

    @property
    def feature_width(self) -> int:
        """How many features the three towers give the trunk."""
        return (
            self.scalars.width
            + self.ring.width
            + sum(tower.width for tower in self.tokens)
        )

    def counts(self) -> dict[str, int]:
        """How many weights each part trains, and how many the whole trains.

        A test asserts this, so a later change that quietly adds a weight
        fails rather than costing alignment nobody accounted for.
        """
        parts = {
            "scalars": self.scalars.parameter_count,
            "ring": self.ring.parameter_count,
            "tokens": int(sum(tower.parameter_count for tower in self.tokens)),
            "trunk": int(self.trunk.size),
            "readout": int(self.readout.size),
        }
        parts["total"] = int(sum(parts.values()))
        return parts

    @property
    def parameter_count(self) -> int:
        """How many weights the whole policy trains."""
        return self.counts()["total"]

    def _shapes(self) -> tuple[tuple[int, ...], ...]:
        """Give the shape of every trainable array, in the order of the vector."""
        shapes: list[tuple[int, ...]] = [*self.scalars.shapes, *self.ring.shapes]
        for tower in self.tokens:
            shapes.extend(tower.shapes)
        shapes.append(self.trunk.shape)
        shapes.append(self.readout.shape)
        return tuple(shapes)

    def flat(self) -> npt.NDArray[np.float64]:
        """Return every trainable weight as one vector.

        Every layer of the policy is in it. A trainer perturbs this vector and
        rebuilds a policy from it.
        """
        arrays: list[np.ndarray] = [*self.scalars.arrays(), *self.ring.arrays()]
        for tower in self.tokens:
            arrays.extend(tower.arrays())
        arrays.append(self.trunk)
        arrays.append(self.readout)
        return np.concatenate([array.reshape(-1) for array in arrays])

    def rebuild(self, flat: npt.NDArray[np.float64]) -> StructuredPolicy:
        """Return a policy of this layout and these widths with the given weights.

        **The normalizer travels with the rebuilt policy.** The search
        rebuilds every candidate of every generation through the shell, so a
        normalizer that stopped here would reach no candidate the run scored.
        """
        parts = _split(np.asarray(flat, dtype=np.float64), self._shapes())
        walked = 0
        scalars = self.scalars.with_arrays(
            parts[walked : walked + len(self.scalars.shapes)]
        )
        walked += len(self.scalars.shapes)
        ring = self.ring.with_arrays(parts[walked : walked + len(self.ring.shapes)])
        walked += len(self.ring.shapes)
        towers = []
        for tower in self.tokens:
            size = len(tower.shapes)
            towers.append(tower.with_arrays(parts[walked : walked + size]))
            walked += size
        return StructuredPolicy(
            self.layout,
            self.shape,
            scalars,
            ring,
            towers,
            parts[walked],
            parts[walked + 1],
            self.normalizer,
        )

    def with_readout(self, readout: np.ndarray) -> StructuredPolicy:
        """Return this policy with another readout, and every tower unchanged.

        The readout is the one layer whose row count is the row count of the
        action table. A reader that carries a stored policy onto another
        table rebuilds this layer and keeps the rest.[^1]

        References
        ----------
        [^1]: ADR-0200, a stored policy names each row of the action table by
        its verb and its candidate coordinates, decision D3.
        ``docs/adrs/draft/adr-0200-a-stored-policy-names-each-action-row-by-verb-and-coordinates.md``
        """
        return StructuredPolicy(
            self.layout,
            self.shape,
            self.scalars,
            self.ring,
            self.tokens,
            self.trunk,
            readout,
            self.normalizer,
        )

    def features(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Join the three towers into the row the trunk reads."""
        parts = [self.scalars.features(encoded), self.ring.features(encoded)]
        parts.extend(tower.features(encoded) for tower in self.tokens)
        parts.append(np.ones((encoded.shape[0], 1)))
        return np.concatenate(parts, axis=1)

    def scores(self, encoded: npt.NDArray[np.float64]) -> npt.NDArray[np.float64]:
        """Return one score for each action row of each encoded observation row."""
        hidden = np.tanh(self.features(encoded) @ self.trunk.T)
        with_bias = np.concatenate([hidden, np.ones((hidden.shape[0], 1))], axis=1)
        return with_bias @ self.readout.T

    def choose(self, observation: np.ndarray, mask: np.ndarray) -> int:
        """Return the action integer of the highest-scoring legal row."""
        scores = self.scores(encode(observation, self.normalizer)[None, :])
        return masked_choices(scores, np.asarray(mask)[None, :])[0]

    def scores_many(self, observations: np.ndarray) -> npt.NDArray[np.float64]:
        """Return one unmasked score for each action row of each observation.

        The choice masks the result, and the instrument that reads the
        unmasked preference reads the same matrix.
        """
        return self.scores(encode_many(observations, self.normalizer))

    def choose_many(self, observations: np.ndarray, masks: np.ndarray) -> list[int]:
        """Return one action for each row of a stack of observations."""
        return masked_choices(self.scores_many(observations), masks)

    def save(self, path: Path, meta: Mapping[str, object]) -> None:
        """Write the weights, the layout, the widths and the fit.

        The layout goes into the file because two layouts of one length mean
        different things. A reader that only compared the length would load a
        file trained against another shape of stack and raise nothing.

        **Every key of this policy carries a prefix.** The fit of the world
        names a length and a version of its own, and a key that collided with
        one of those would put two declarations of one number in one file.

        **The normalizer goes in beside the weights.** Every tower reads a
        standardized feature, so a file without its normalizer states nothing
        a reader can play.
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        stored: dict[str, np.ndarray] = {
            "flat": self.flat(),
            "layout_ring_cells": np.asarray(
                self.layout.ring.stack.ring_cells, dtype=np.int64
            ),
            "layout_ring_positions": self.layout.ring.gather(),
            "layout_length": np.asarray(self.layout.length, dtype=np.int64),
            "layout_token_names": np.asarray(
                [block.name for block in self.layout.tokens]
            ),
            "architecture": np.asarray(
                [
                    self.shape.scalar_width,
                    self.shape.ring_width,
                    self.shape.ring_bands,
                    self.shape.sector_kernel,
                    self.shape.token_width,
                    self.shape.trunk_width,
                    self.action_length,
                ],
                dtype=np.int64,
            ),
            "kind": np.asarray(STRUCTURED_KIND),
        }
        if self.normalizer is not None:
            stored.update(self.normalizer.as_arrays())
        for index, block in enumerate(self.layout.tokens):
            stored[f"layout_token_positions_{index}"] = block.gather()
        # numpy declares ``allow_pickle`` beside its own keyword arguments, so
        # a mapping keyed on ``str`` can never unpack cleanly.
        np.savez(
            path,
            **stored,  # type: ignore[arg-type]
            **{key: np.array(value) for key, value in meta.items()},  # type: ignore[arg-type]
        )

    @classmethod
    def restore(cls, stored: Mapping[str, np.ndarray]) -> StructuredPolicy:
        """Rebuild the policy one weight file holds.

        The file states the layout, the widths and the action row count under
        its own keys. It may also state the fit of the world, which names the
        observation length and the action row count a second time. This
        compares the two rather than choosing a winner, so a file whose
        copies disagree fails here.

        A file that states no normalizer rebuilds a policy that reads the
        plain squash, so a policy published before the normalizer existed
        still runs.
        """
        names = [str(name) for name in stored["layout_token_names"]]
        blocks = []
        for index, name in enumerate(names):
            grid = np.asarray(stored[f"layout_token_positions_{index}"], dtype=np.int64)
            blocks.append(
                TokenBlock(
                    name,
                    int(grid.shape[1]),
                    tuple(tuple(int(v) for v in row) for row in grid),
                )
            )
        ring_grid = np.asarray(stored["layout_ring_positions"], dtype=np.int64)
        ring = RingBlock(
            RingStack(tuple(int(count) for count in stored["layout_ring_cells"])),
            int(ring_grid.shape[1]),
            tuple(tuple(int(v) for v in row) for row in ring_grid),
        )
        length = int(stored["layout_length"])
        layout = ObservationLayout(length=length, ring=ring, tokens=tuple(blocks))
        architecture = [int(value) for value in stored["architecture"]]
        shape = StructuredShape(*architecture[:-1])
        actions = architecture[-1]
        _agree(stored, "observation_length", length)
        _agree(stored, "action_length", actions)
        shell = cls.zeros(
            actions, layout, shape, normalizer=FeatureNormalizer.read(stored)
        )
        return shell.rebuild(np.asarray(stored["flat"], dtype=np.float64))

    def check_layout(self, wanted: ObservationLayout, path: Path | None = None) -> None:
        """Refuse when this policy reads another layout than a caller asked for.

        Raises ``PolicyFitError`` naming both layouts. The schema version
        catches a change the engine made, and this catches a change a caller
        made to the geometry it states.
        """
        if self.layout.fingerprint() == wanted.fingerprint():
            return
        where = f" at {path}" if path is not None else ""
        message = (
            f"the stored policy{where} reads another observation layout. "
            f"The file states {self.layout.describe()}. The world states "
            f"{wanted.describe()}. Train a policy against this layout."
        )
        raise PolicyFitError(message)


def layout_of(policy: object) -> ObservationLayout | None:
    """Give the layout one policy reads, or nothing when it reads none.

    The linear policy reads a flat vector and holds no layout. A caller that
    wants to compare layouts asks here rather than testing the type of a
    policy.
    """
    found = getattr(policy, "layout", None)
    return found if isinstance(found, ObservationLayout) else None


__all__ = [
    "POOL_STATISTICS",
    "SHELL_SEED",
    "STRUCTURED_KIND",
    "RingTower",
    "ScalarTower",
    "ShapeError",
    "StructuredPolicy",
    "StructuredShape",
    "TokenTower",
    "layout_of",
]
