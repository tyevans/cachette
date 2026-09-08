"""Where the structured parts of one observation sit, read from the schema.

A policy that exploits the shape of the observation must know that shape. The
engine owns the layout and states it in a schema, so this module reads the
schema and states no position, no width, no ring count, no sector count and
no channel count of its own.[^1]

# The three shapes one observation holds

The observation holds three kinds of part, and each kind needs a different
treatment from a policy.

The ring stack is an egocentric cylinder. It holds rings at geometric hex
distance from the centroid of the faction, sectors inside each ring, and one
value for each channel of each cell.[^2] The sector axis wraps, so one weight
set serves every direction.

An entity token set is a set. A token position names no seat and carries no
identity, so a reader must treat two orders of the same tokens as one
input.[^3]

Every other position is a scalar with no spatial structure.

# This module refuses to guess

A ring stack of 3775 positions is 151 cells of 25 channels, and it is also
25 cells of 151 channels. Nothing in the position count separates the two. A
reader that guesses reads a plausible observation that does not exist, so
this module fails with a message that names the schema entries to add.

The vocabulary of those entries is the one the drawing tool already
established, so the schema carries one set of entries and not two.[^4] The
space entry marks a field as a ring cell or as a token. The channels entry
names the channels of one field. The channel order entry says which axis runs
first. The ring geometry states the cell count of each ring, or the ring
count and the sector count.

# A caller may state a geometry the schema does not

A layout takes its blocks as values. A caller that holds a schema which
states no geometry can therefore build the blocks itself and pass them. The
layout then checks every block against the length the schema states, so a
stated geometry that does not fit fails at once.

References
----------
[^1]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``

[^2]: Report 42, what a policy should be able to see, sections 5 and 9.5.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``

[^3]: ADR-0195, the observation of a faction is a fixed-width scale-free
table, decision D4.
``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``

[^4]: The drawing tool of the observation. ``python/cachette/learn/picture.py``
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING

import numpy as np

from .picture import (
    CELL_MAJOR,
    CHANNEL_MAJOR,
    RING_SPACE,
    RingStack,
    ring_stack_of,
)

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Sequence

    import numpy.typing as npt

    from .signals import Signal, SignalCatalogue


TOKEN_SPACE = "token"

HEX_DIRECTIONS = 6

MISSING_RING_FIELD = (
    "the schema marks no field as ring space, so this layout cannot find the "
    "ring stack. Add 'space' to the ring field of the schema with the value "
    f"{RING_SPACE!r}, or state the ring block yourself and pass it."
)

MISSING_TOKEN_FIELDS = (
    "the schema marks no field as token space, so this layout cannot find the "
    "entity tokens. Publish one field for each token set, because the sets "
    "hold different channel counts and one field cannot state four shapes. "
    f"Mark each with 'space' as {TOKEN_SPACE!r} and give it a 'channels' "
    "list. State the token blocks yourself and pass them instead."
)

MISSING_TOKEN_CHANNELS = (
    "{name!r} is token space and names no channels, so this layout cannot "
    "say how many tokens it holds. Add 'channels' to the field as the list "
    "of channel names of one token."
)


class LayoutError(ValueError):
    """A layout does not describe the observation a caller holds.

    The layout of the structured parts is a function of the schema. A block
    that runs past the end of the observation, a block that overlaps another,
    or a schema that states no geometry all raise this.
    """


def _windows(start: int, outer: int, inner: int, *, inner_first: bool) -> np.ndarray:
    """Give the observation position of every entry of a two-axis block.

    The result holds one row for each outer entry and one column for each
    inner entry. The inner-first flag says that the inner axis runs fastest
    in the published array, which is what the cell-major channel order
    states.
    """
    outer_index = np.arange(outer, dtype=np.int64)[:, None]
    inner_index = np.arange(inner, dtype=np.int64)[None, :]
    if inner_first:
        return start + outer_index * inner + inner_index
    return start + inner_index * outer + outer_index


def _rows(grid: np.ndarray) -> tuple[tuple[int, ...], ...]:
    """Freeze a two-axis position grid into rows of plain integers."""
    return tuple(tuple(int(value) for value in row) for row in grid)


@dataclass(frozen=True)
class RingBlock:
    """The ring stack of one observation, and where every value of it sits.

    The stack entry holds the cell count of each ring, in ring order. The
    positions entry holds one row for each cell and one column for each
    channel, so a reader gathers the whole stack in one step.
    """

    stack: RingStack
    channels: int
    positions: tuple[tuple[int, ...], ...]

    def __post_init__(self) -> None:
        """Refuse a block whose positions do not match its own geometry."""
        if self.channels < 1:
            message = (
                f"a ring block holds at least one channel, and this states "
                f"{self.channels}"
            )
            raise LayoutError(message)
        if len(self.positions) != self.stack.cells:
            message = (
                f"this ring block states {self.stack.cells} cells and holds "
                f"{len(self.positions)} rows of positions"
            )
            raise LayoutError(message)
        for row in self.positions:
            if len(row) != self.channels:
                message = (
                    f"this ring block states {self.channels} channels and "
                    f"holds a row of {len(row)} positions"
                )
                raise LayoutError(message)
        for count in self.stack.ring_cells:
            if count != 1 and count % HEX_DIRECTIONS != 0:
                message = (
                    f"a ring holds one cell or a multiple of {HEX_DIRECTIONS} "
                    f"cells, because a rotation by one hex direction must "
                    f"permute the cells of every ring, and this stack states "
                    f"{list(self.stack.ring_cells)}"
                )
                raise LayoutError(message)

    @classmethod
    def contiguous(
        cls, start: int, stack: RingStack, channels: int, *, cell_major: bool = True
    ) -> RingBlock:
        """Build a block whose values run in one unbroken window.

        The cell-major flag says that every channel of one cell is adjacent,
        which is the order the drawing tool names ``cell_major``.
        """
        grid = _windows(start, stack.cells, channels, inner_first=cell_major)
        return cls(stack, channels, _rows(grid))

    @property
    def cells(self) -> int:
        """How many ring cells the stack holds."""
        return self.stack.cells

    @property
    def rings(self) -> int:
        """How many rings the stack holds."""
        return self.stack.rings

    @property
    def sectors(self) -> int:
        """How many sectors the widest ring holds.

        This is the sector count of the rectangular view. The inner rings
        hold fewer cells, and the rectangular view repeats each of them.
        """
        return max(self.stack.ring_cells)

    @property
    def slots(self) -> int:
        """How many observation positions the whole stack holds."""
        return self.cells * self.channels

    def gather(self) -> npt.NDArray[np.int64]:
        """Give the positions as one array of cells by channels."""
        return np.asarray(self.positions, dtype=np.int64)

    def rectangle(self) -> npt.NDArray[np.int64]:
        """Give the cell index of every place of the rectangular view.

        The result holds one row for each ring and one column for each sector
        of the widest ring. Ring 0 holds one cell and has no direction, so
        every sector of it names that cell. Ring 1 holds six cells, so each
        of them fills two sectors. A wider ring fills one sector each.

        The repetition costs no weight and it makes the sector axis uniform,
        so one wrapped kernel serves every ring. The alternative is a second
        path for the inner rings, which states the same rule twice.
        """
        sectors = self.sectors
        rows = []
        base = 0
        for count in self.stack.ring_cells:
            rows.append(
                [base + (sector * count) // sectors for sector in range(sectors)]
            )
            base += count
        return np.asarray(rows, dtype=np.int64)

    def rotation(self, steps: int) -> npt.NDArray[np.int64]:
        """Give the source cell of every cell after a rotation of the world.

        One step is one hex direction, which is a sixth of a turn. The
        rotation carries the content of a sector into the sector that many
        directions further round, in every ring at once. The result is a
        gather index: the value of cell ``c`` after the rotation is the value
        that cell ``result[c]`` held before it.

        A rotation by one hex direction is the coarsest rotation the hex grid
        admits, so it is the symmetry a sector kernel must respect.
        """
        source = []
        base = 0
        for count in self.stack.ring_cells:
            shift = steps * count // HEX_DIRECTIONS
            for sector in range(count):
                source.append(base + (sector - shift) % count)
            base += count
        return np.asarray(source, dtype=np.int64)


@dataclass(frozen=True)
class TokenBlock:
    """One entity token set, and where every channel of every token sits.

    A token position names no seat, so a reader must not give one token a
    weight that another does not have.[^1]

    References
    ----------
    [^1]: ADR-0195, the observation of a faction is a fixed-width scale-free
    table, decision D4.
    ``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``
    """

    name: str
    channels: int
    positions: tuple[tuple[int, ...], ...]

    def __post_init__(self) -> None:
        """Refuse a set whose rows do not all hold the stated channels."""
        if self.channels < 1:
            message = (
                f"{self.name!r} holds at least one channel, and it states "
                f"{self.channels}"
            )
            raise LayoutError(message)
        if not self.positions:
            message = f"{self.name!r} holds no token"
            raise LayoutError(message)
        for row in self.positions:
            if len(row) != self.channels:
                message = (
                    f"{self.name!r} states {self.channels} channels and holds "
                    f"a token of {len(row)} positions"
                )
                raise LayoutError(message)

    @classmethod
    def contiguous(
        cls,
        name: str,
        start: int,
        tokens: int,
        channels: int,
        *,
        token_major: bool = True,
    ) -> TokenBlock:
        """Build a set whose values run in one unbroken window.

        The token-major flag says that every channel of one token is
        adjacent, which is the order the drawing tool names ``cell_major``.
        """
        grid = _windows(start, tokens, channels, inner_first=token_major)
        return cls(name, channels, _rows(grid))

    @property
    def tokens(self) -> int:
        """How many tokens the set holds."""
        return len(self.positions)

    @property
    def slots(self) -> int:
        """How many observation positions the whole set holds."""
        return self.tokens * self.channels

    def gather(self) -> npt.NDArray[np.int64]:
        """Give the positions as one array of tokens by channels."""
        return np.asarray(self.positions, dtype=np.int64)


@dataclass(frozen=True)
class ObservationLayout:
    """The three structured parts of one observation layout.

    The ring entry and the token entry name the parts that carry structure.
    The scalar entry holds every remaining position, in ascending order, and
    it is derived rather than stated. A position therefore belongs to exactly
    one part, and no position is left out.
    """

    length: int
    ring: RingBlock
    tokens: tuple[TokenBlock, ...]
    scalars: tuple[int, ...] = field(init=False, default=())

    def __post_init__(self) -> None:
        """Derive the scalar positions, and refuse a block that does not fit."""
        claimed = np.zeros(self.length, dtype=bool)
        for name, grid in self._structured():
            flat = grid.reshape(-1)
            if flat.size and (flat.min() < 0 or int(flat.max()) >= self.length):
                message = (
                    f"{name} reaches position {int(flat.max())} of an "
                    f"observation of {self.length} positions"
                )
                raise LayoutError(message)
            if claimed[flat].any():
                message = f"{name} claims a position another part already claims"
                raise LayoutError(message)
            claimed[flat] = True
        rest = np.flatnonzero(~claimed)
        object.__setattr__(self, "scalars", tuple(int(value) for value in rest))

    def _structured(self) -> list[tuple[str, npt.NDArray[np.int64]]]:
        """Name every structured part and give its positions."""
        parts = [("the ring stack", self.ring.gather())]
        parts.extend(
            (f"the token set {block.name!r}", block.gather()) for block in self.tokens
        )
        return parts

    @property
    def scalar_slots(self) -> int:
        """How many positions carry no spatial and no set structure."""
        return len(self.scalars)

    def fingerprint(self) -> tuple[object, ...]:
        """Give the value that two layouts must share to mean the same thing.

        A stored weight file holds a layout, and a caller holds another. The
        two must agree before the weights mean anything, and the length alone
        does not separate them: one ring stack of 3775 positions is 151 cells
        of 25 channels and another is 25 cells of 151 channels.

        **The positions are part of it, and not only the counts.** Two
        layouts of one cell count and one channel count still differ when one
        holds the channels of a cell adjacent and the other holds the cells of
        a channel adjacent. Nothing in the counts separates those two.
        """
        return (
            self.length,
            self.ring.stack.ring_cells,
            self.ring.channels,
            self.ring.positions,
            tuple(
                (block.name, block.channels, block.positions) for block in self.tokens
            ),
        )

    def describe(self) -> str:
        """Return one line that names every part of the layout."""
        sets = ", ".join(
            f"{block.name}={block.tokens}x{block.channels}" for block in self.tokens
        )
        return (
            f"length={self.length}, ring_cells={list(self.ring.stack.ring_cells)}, "
            f"ring_channels={self.ring.channels}, scalars={self.scalar_slots}, "
            f"tokens[{sets}]"
        )

    @classmethod
    def of_catalogue(cls, catalogue: SignalCatalogue) -> ObservationLayout:
        """Read the layout from the schema, and refuse to guess it.

        Raises ``LayoutError`` when the schema states no ring geometry or no
        token set. The message names the entries to add.
        """
        return cls(
            length=catalogue.observation_length,
            ring=_ring_of_catalogue(catalogue),
            tokens=_tokens_of_catalogue(catalogue),
        )


def _marked(catalogue: SignalCatalogue, space: str) -> tuple[Signal, ...]:
    """Give every signal the schema marks with one space."""
    return tuple(signal for signal in catalogue if signal.space == space)


def _channel_order(catalogue: SignalCatalogue, name: str) -> bool:
    """Say whether the channels of one place are adjacent in the published array.

    The schema states the order, and this refuses to guess it. A wrong guess
    reads a plausible observation that does not exist.
    """
    order = catalogue.geometry.get("channel_order")
    if order == CELL_MAJOR:
        return True
    if order == CHANNEL_MAJOR:
        return False
    message = (
        f"{name!r} holds several channels over several places and the schema "
        f"does not say which axis runs first. Add 'channel_order' to the "
        f"schema as {CELL_MAJOR!r} when every channel of one place is "
        f"adjacent, or as {CHANNEL_MAJOR!r} when every place of one channel "
        f"is adjacent."
    )
    raise LayoutError(message)


def _ring_of_catalogue(catalogue: SignalCatalogue) -> RingBlock:
    """Read the ring block from the schema."""
    marked = _marked(catalogue, RING_SPACE)
    if len(marked) != 1:
        if not marked:
            raise LayoutError(MISSING_RING_FIELD)
        message = (
            f"the schema marks {len(marked)} fields as ring space, and this "
            f"layout reads one ring stack. Publish the stack as one field."
        )
        raise LayoutError(message)
    signal = marked[0]
    try:
        stack = ring_stack_of(catalogue.geometry, [signal.name])
    except (ValueError, TypeError) as error:
        raise LayoutError(str(error)) from error
    channels, remainder = divmod(signal.positions, stack.cells)
    if remainder or channels < 1:
        message = (
            f"{signal.name!r} holds {signal.positions} positions over "
            f"{stack.cells} ring cells, which is not a whole channel count"
        )
        raise LayoutError(message)
    declared = len(signal.channels)
    if declared and declared != channels:
        message = (
            f"{signal.name!r} names {declared} channels and holds "
            f"{signal.positions} positions over {stack.cells} cells, which "
            f"needs {channels}"
        )
        raise LayoutError(message)
    cell_major = _channel_order(catalogue, signal.name) if channels > 1 else True
    return RingBlock.contiguous(signal.start, stack, channels, cell_major=cell_major)


def _tokens_of_catalogue(catalogue: SignalCatalogue) -> tuple[TokenBlock, ...]:
    """Read every token set from the schema."""
    marked = _marked(catalogue, TOKEN_SPACE)
    if not marked:
        raise LayoutError(MISSING_TOKEN_FIELDS)
    blocks = []
    for signal in marked:
        channels = len(signal.channels)
        if channels < 1:
            raise LayoutError(MISSING_TOKEN_CHANNELS.format(name=signal.name))
        tokens, remainder = divmod(signal.positions, channels)
        if remainder or tokens < 1:
            message = (
                f"{signal.name!r} names {channels} channels and holds "
                f"{signal.positions} positions, which is not a whole token count"
            )
            raise LayoutError(message)
        token_major = _channel_order(catalogue, signal.name) if channels > 1 else True
        blocks.append(
            TokenBlock.contiguous(
                signal.name, signal.start, tokens, channels, token_major=token_major
            )
        )
    return tuple(blocks)


def token_blocks(
    name: str, start: int, shapes: Sequence[tuple[str, int, int]]
) -> tuple[TokenBlock, ...]:
    """Cut one published field into the token sets it holds, in order.

    The engine publishes the four sets as one field today, and the schema
    states no shape for them. A caller that knows the shapes states them
    here, as a name, a token count and a channel count for each set. The
    sets follow one another in the order given, and each set holds every
    channel of one token adjacent.

    The name argument names the field the sets came from, so a failure says
    where the caller was reading.
    """
    blocks = []
    walked = start
    for set_name, tokens, channels in shapes:
        blocks.append(
            TokenBlock.contiguous(f"{name}.{set_name}", walked, tokens, channels)
        )
        walked += tokens * channels
    return tuple(blocks)


__all__ = [
    "HEX_DIRECTIONS",
    "TOKEN_SPACE",
    "LayoutError",
    "ObservationLayout",
    "RingBlock",
    "TokenBlock",
    "token_blocks",
]
