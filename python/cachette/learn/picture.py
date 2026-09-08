"""Draw the observation of one faction, so a reader looks at it.

A policy reads one flat array of integers. Nothing in this project drew that
array, so every question about what the policy sees was answered by reading
engine source, and several of those answers were wrong. This module draws the
array instead.

# The schema is the only layout this module knows

The engine owns the layout of the observation and states it in a schema.[^1]
This module holds no position, no width, no ring count, no sector count and no
channel count. It reads the schema through the signal catalogue, which is the
one reader of that schema in the control plane.[^2] A field the engine adds
therefore appears in the picture with no change here.

# One page holds every group

A reader compares fog against held ground against food at one moment, so the
tool puts every spatial group on one page and the scalar groups under them. A
directory of one file for each channel answers no question a reader has.

# Absent is not zero

A position that carries no value yet must not read as the value zero. The
design of the new observation states this twice: a ring cell outside the world
reads zero in an area channel that gates the rest, and a missing entity token
reads zero in a validity channel.[^3] A picture that shades both the same way
destroys the distinction that those channels exist to carry. This module draws
an absent position as a hatch and a zero position as the light end of the
ramp.

The tool learns which positions are absent from three places. A position that
no field covers is absent. A signal that names a gate is absent where the gate
reads zero. A caller may name the gate on the command line, for a schema that
does not state one yet.

# What the constants of this module are, and what they are not

Every drawing size here is a drawing size. The layout constants of the
observation are not here, and this module reads all of them from the schema.

``LATTICE_PREFIX`` is the prefix the published lattice fields carry today. The
tool applies it only when the schema marks no field with a space, so that the
tool draws the current observation as well as the one that replaces it.

``RING_ZERO_CELLS`` and ``RING_ONE_CELLS`` are the cell counts of the two
innermost rings. The design fixes both from the geometry of a hex ring and not
from a knob: the ring at distance zero is one tile and has no direction, and
the ring at distance one is six tiles.[^3] The tool applies the two only when
the schema states a ring count and a sector count and no cell list.

The one-hue ramp draws a quantity that never goes below zero. The two-hue ramp
draws a quantity that does. The middle stop of the two-hue ramp is the light
end of the one-hue ramp, so a zero reads the same way in both.

# Two spatial shapes, because the engine is between two layouts

The published layout today holds a square lattice of summary cells, and each
lattice signal holds one value for each cell. The new design holds an
egocentric ring stack: rings at geometric hex distance from the centroid of
the faction, sectors inside each ring, and one value for each cell of each
channel.[^3] The tool draws a lattice signal as a square grid and a ring
signal as a polar plot. It selects between them from the schema, and it falls
back to the lattice for a signal whose name carries the lattice prefix.

The polar plot draws each ring as a band of equal width. The hex distance
bands of the design are geometric, so a plot with proportional bands would
draw ring 0 as one pixel and ring 7 as half the picture. Equal bands keep
every ring readable, and the ring index sits on a spoke so that no reader
counts bands.

# References

[^1]: ADR-0154, the observation and the action of a faction are
schema-declared bounded tables the engine owns, decision D1.
``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
[^2]: The one reader of the observation schema.
``python/cachette/learn/signals.py``
[^3]: Report 42, what a policy should be able to see, sections 5 and 9.
``docs/research/reports/42-what-a-policy-should-be-able-to-see.md``
"""

from __future__ import annotations

import argparse
import math
from dataclasses import dataclass
from math import isqrt
from pathlib import Path
from typing import TYPE_CHECKING

import numpy as np

from cachette._core import World

from .signals import Signal, SignalCatalogue

if TYPE_CHECKING:  # pragma: no cover - the import is for the type checker
    from collections.abc import Mapping, Sequence

    import numpy.typing as npt


RING_SPACE = "ring"
GRID_SPACE = "grid"

LATTICE_PREFIX = "cell_"

RING_ZERO_CELLS = 1
RING_ONE_CELLS = 6

MISSING_RING_GEOMETRY = (
    "the schema marks {names} as ring space and states no ring geometry, so "
    "this tool cannot place a cell. Add 'ring_cells' to the schema as the "
    "cell count of each ring, in ring order, for example "
    "[1, 6, 12, 12, 12, 12, 12, 12]. State 'rings' and 'sectors' instead and "
    "this tool derives the same list, because ring 0 holds one cell, ring 1 "
    "holds six cells, and every further ring holds 'sectors' cells."
)

MISSING_CHANNEL_ORDER = (
    "{name!r} names {count} channels over {cells} cells and the schema does "
    "not say which of the two runs first, so this tool cannot cut the "
    "channels apart. Publish one field for each channel, which needs no "
    "order at all. Add 'channel_order' to the schema instead, as "
    "'channel_major' when every value of one channel is adjacent, or as "
    "'cell_major' when every channel of one cell is adjacent."
)

CHANNEL_MAJOR = "channel_major"
CELL_MAJOR = "cell_major"

PANEL_WIDTH = 258
PANEL_HEIGHT = 292
PANEL_RADIUS = 96
PANEL_COLUMNS = 5

BAR_HEIGHT = 13
BAR_LABEL_WIDTH = 168
BAR_TRACK_WIDTH = 118
BAR_VALUE_WIDTH = 104
BAR_COLUMN_WIDTH = BAR_LABEL_WIDTH + BAR_TRACK_WIDTH + BAR_VALUE_WIDTH + 18
BAR_COLUMN_ROWS = 34

HEADER_HEIGHT = 86
BLOCK_HEADING_HEIGHT = 24
SECTION_GAP = 18
MARGIN = 20

SEQUENTIAL_RAMP = (
    (0xF4, 0xF7, 0xF6),
    (0xCC, 0xE5, 0xDF),
    (0x96, 0xCD, 0xC2),
    (0x53, 0xAC, 0x9C),
    (0x25, 0x7F, 0x72),
    (0x0E, 0x4D, 0x45),
)
DIVERGING_RAMP = (
    (0x7A, 0x33, 0x0A),
    (0xC0, 0x69, 0x2B),
    (0xE8, 0xBE, 0x93),
    (0xF4, 0xF7, 0xF6),
    (0x96, 0xCD, 0xC2),
    (0x25, 0x7F, 0x72),
    (0x0E, 0x4D, 0x45),
)

ABSENT_FILL = "url(#absent)"
INK = "#16211f"
FAINT = "#8d9a97"
PAPER = "#ffffff"


@dataclass(frozen=True)
class RingStack:
    """The ring and sector geometry of an egocentric ring stack.

    The cell count of each ring sits in ring order. A published cell index
    walks the rings in ascending order and the sectors of each ring in
    ascending order, so this class turns an index into a ring and a sector.
    """

    ring_cells: tuple[int, ...]

    @property
    def cells(self) -> int:
        """How many cells the whole stack holds."""
        return sum(self.ring_cells)

    @property
    def rings(self) -> int:
        """How many rings the stack holds."""
        return len(self.ring_cells)

    def place(self, index: int) -> tuple[int, int]:
        """Give the ring and the sector of one published cell index."""
        if not 0 <= index < self.cells:
            message = f"cell {index} is outside a stack of {self.cells} cells"
            raise IndexError(message)
        walked = index
        for ring, count in enumerate(self.ring_cells):
            if walked < count:
                return ring, walked
            walked -= count
        raise AssertionError  # pragma: no cover - the bound above covers this

    def sectors(self, ring: int) -> int:
        """How many sectors one ring holds."""
        return self.ring_cells[ring]


@dataclass(frozen=True)
class Lattice:
    """The geometry of a square lattice of summary cells.

    A lattice signal holds one value for each cell of a square block grid, so
    the side comes from the position count of the signal.
    """

    side: int

    @property
    def cells(self) -> int:
        """How many cells the lattice holds."""
        return self.side * self.side

    def place(self, index: int) -> tuple[int, int]:
        """Give the row and the column of one published cell index.

        The engine publishes the cells in one order and the schema does not
        state which axis runs first. The panel labels every cell with its
        index, so a reader checks the guess against the picture.
        """
        if not 0 <= index < self.cells:
            message = f"cell {index} is outside a lattice of {self.cells} cells"
            raise IndexError(message)
        return index // self.side, index % self.side


@dataclass(frozen=True)
class Panel:
    """One channel of the spatial part, ready to draw.

    The values entry holds one number for each cell. The present entry says
    which of those cells carry a value at all. The positions entry holds the
    observation position each cell came from, so that the page reports what it
    drew.
    """

    title: str
    values: npt.NDArray[np.float64]
    present: npt.NDArray[np.bool_]
    positions: npt.NDArray[np.int64]

    @property
    def scale(self) -> float:
        """The value that reads as full shade.

        The scale is the largest magnitude the panel holds. The channels of
        one observation run over ranges that differ by seven orders, so a
        shared scale would draw every small channel as empty. The panel prints
        its own scale, so a reader never compares two shades across panels
        without seeing that the scales differ.
        """
        live = np.abs(self.values[self.present])
        if live.size == 0:
            return 1.0
        top = float(live.max())
        return top if top > 0.0 else 1.0

    @property
    def signed(self) -> bool:
        """Whether the panel holds a value below zero."""
        live = self.values[self.present]
        return bool(live.size and live.min() < 0.0)

    @property
    def subtitle(self) -> str:
        """What the panel says about its own scale, under its title."""
        return f"full shade {_number(self.scale)}"


@dataclass(frozen=True)
class Bar:
    """One position of a scalar group, ready to draw."""

    label: str
    value: float
    present: bool
    positions: tuple[int, ...]


@dataclass(frozen=True)
class Block:
    """One named group of scalar positions."""

    name: str
    bars: tuple[Bar, ...]

    @property
    def scale(self) -> float:
        """The magnitude that fills the track of every bar of this block."""
        live = [abs(bar.value) for bar in self.bars if bar.present]
        top = max(live) if live else 0.0
        return top if top > 0.0 else 1.0


@dataclass(frozen=True)
class Page:
    """Everything one picture holds."""

    caption: str
    notes: tuple[str, ...]
    panels: tuple[Panel, ...]
    blocks: tuple[Block, ...]
    geometry: RingStack | Lattice | None
    length: int

    def drawn(self) -> npt.NDArray[np.bool_]:
        """Say which positions of the observation this page draws.

        A test compares this against the length the schema states. A field the
        engine adds and this page misses then fails a check rather than going
        unnoticed.
        """
        mask = np.zeros(self.length, dtype=bool)
        for panel in self.panels:
            mask[panel.positions] = True
        for block in self.blocks:
            for bar in block.bars:
                for position in bar.positions:
                    mask[position] = True
        return mask


def ring_stack_of(geometry: Mapping[str, object], names: Sequence[str]) -> RingStack:
    """Read the ring geometry from the schema, and refuse to guess it.

    The schema states the cell count of each ring, or it states the ring count
    and the sector count and this function applies the rule of the design. It
    states neither and this function fails with the message that says what to
    add.
    """
    declared = geometry.get("ring_cells")
    if declared is not None:
        if not isinstance(declared, list | tuple):
            message = (
                f"'ring_cells' holds the cell count of each ring as a list, "
                f"and the schema holds {declared!r}"
            )
            raise TypeError(message)
        return RingStack(tuple(int(count) for count in declared))
    rings = geometry.get("rings")
    sectors = geometry.get("sectors")
    if not isinstance(rings, int) or not isinstance(sectors, int):
        raise ValueError(MISSING_RING_GEOMETRY.format(names=sorted(names)))
    counts = [RING_ZERO_CELLS, RING_ONE_CELLS][:rings]
    counts.extend(sectors for _ in range(rings - len(counts)))
    return RingStack(tuple(counts))


def _lattice_of(signals: Sequence[Signal]) -> Lattice:
    """Read the lattice side from the position count of a lattice signal."""
    widths = {signal.positions for signal in signals}
    if len(widths) != 1:
        message = (
            f"the lattice signals hold different position counts {sorted(widths)}, "
            f"so they do not share one grid"
        )
        raise ValueError(message)
    cells = widths.pop()
    side = isqrt(cells)
    if side * side != cells:
        message = (
            f"a lattice signal holds {cells} positions, which is not a square, "
            f"so this tool cannot lay it out as a grid"
        )
        raise ValueError(message)
    return Lattice(side)


SELF_BLOCK = "self"


def _blocks_by_name(signals: Sequence[Signal]) -> dict[str, str]:
    """Say which group each scalar signal belongs to.

    The schema names the block of a signal when it can, and this function
    then takes that name. A schema that names no block falls back to the part
    of the name before the first underscore, and it takes that prefix only
    when two or more signals share it. One signal alone is not a group, and a
    page of one-bar groups hides the groups that are real.
    """
    named = {
        signal.name: signal.block for signal in signals if signal.block is not None
    }
    heads: dict[str, int] = {}
    for signal in signals:
        if signal.name in named:
            continue
        head, _, tail = signal.name.partition("_")
        if tail:
            heads[head] = heads.get(head, 0) + 1
    groups: dict[str, str] = {}
    for signal in signals:
        block = named.get(signal.name)
        if block is not None:
            groups[signal.name] = block
            continue
        head, _, tail = signal.name.partition("_")
        groups[signal.name] = head if tail and heads[head] > 1 else SELF_BLOCK
    return groups


def _channel_windows(
    signal: Signal, cells: int, order: str | None
) -> tuple[tuple[str, slice], ...]:
    """Cut one signal into the windows of its channels.

    A signal with no channel list is one channel. A signal with a channel list
    needs the schema to say which axis runs first, because a wrong guess draws
    a picture that looks plausible and is not the observation.
    """
    if not signal.channels:
        return ((signal.name, slice(0, cells)),)
    count = len(signal.channels)
    if count * cells != signal.positions:
        message = (
            f"{signal.name!r} names {count} channels over {cells} cells, "
            f"which needs {count * cells} positions, and it holds "
            f"{signal.positions}"
        )
        raise ValueError(message)
    if order is None:
        raise ValueError(
            MISSING_CHANNEL_ORDER.format(name=signal.name, count=count, cells=cells)
        )
    windows = []
    for index, channel in enumerate(signal.channels):
        if order == CHANNEL_MAJOR:
            windows.append((channel, slice(index * cells, (index + 1) * cells)))
        elif order == CELL_MAJOR:
            windows.append((channel, slice(index, signal.positions, count)))
        else:
            message = (
                f"the schema states the channel order {order!r}, and this tool "
                f"knows {CHANNEL_MAJOR!r} and {CELL_MAJOR!r}"
            )
            raise ValueError(message)
    return tuple(windows)


def _spatial_split(
    catalogue: SignalCatalogue,
) -> tuple[tuple[Signal, ...], tuple[Signal, ...], str, bool]:
    """Split the signals into the spatial ones and the rest.

    The schema marks a spatial field with a space. A schema that marks none
    falls back to the lattice prefix the engine publishes today, and the
    caller reports that fallback on the page.
    """
    marked = tuple(
        signal for signal in catalogue if signal.space in {RING_SPACE, GRID_SPACE}
    )
    if marked:
        spaces = sorted({str(signal.space) for signal in marked})
        if len(spaces) != 1:
            message = (
                f"the schema mixes the spatial shapes {spaces} in one "
                f"observation, and one page draws one shape"
            )
            raise ValueError(message)
        space = spaces[0]
        rest = tuple(signal for signal in catalogue if signal not in marked)
        return marked, rest, space, False
    guessed = tuple(
        signal for signal in catalogue if signal.name.startswith(LATTICE_PREFIX)
    )
    rest = tuple(signal for signal in catalogue if signal not in guessed)
    return guessed, rest, GRID_SPACE, bool(guessed)


def _gate_name(
    catalogue: SignalCatalogue, spatial: Sequence[Signal], chosen: str | None
) -> str | None:
    """Name the signal that says which spatial cells carry a value.

    The caller wins, then the schema entry that names one gate for the whole
    spatial part, then a gate named on a spatial field itself.
    """
    if chosen is not None:
        return catalogue.signal(chosen).name
    declared = catalogue.geometry.get("spatial_gate")
    if declared is not None:
        return str(declared)
    named = {signal.gate for signal in spatial if signal.gate is not None}
    if len(named) > 1:
        message = (
            f"the spatial signals name the gates {sorted(named)}, and one page "
            f"draws one gate"
        )
        raise ValueError(message)
    return named.pop() if named else None


def read_page(
    catalogue: SignalCatalogue,
    observation: npt.NDArray[np.int64],
    caption: str,
    gate: str | None = None,
) -> Page:
    """Turn one observation and its schema into everything a picture holds."""
    if observation.shape[-1] != catalogue.observation_length:
        message = (
            f"this catalogue reads an observation of "
            f"{catalogue.observation_length} positions and was given one of "
            f"{observation.shape[-1]}"
        )
        raise ValueError(message)
    values = np.asarray(observation, dtype=np.float64)
    spatial, scalar, space, guessed = _spatial_split(catalogue)
    notes: list[str] = []

    geometry: RingStack | Lattice | None = None
    if spatial:
        if space == RING_SPACE:
            geometry = ring_stack_of(
                catalogue.geometry, [signal.name for signal in spatial]
            )
        else:
            geometry = _lattice_of(spatial)
    if guessed:
        notes.append(
            f"The schema marks no field as spatial. The tool drew every field "
            f"named {LATTICE_PREFIX}* as a square grid."
        )

    gate_name = _gate_name(catalogue, spatial, gate)
    present_cells: npt.NDArray[np.bool_] | None = None
    if gate_name is not None and geometry is not None:
        window = catalogue.signal(gate_name)
        present_cells = values[window.start : window.start + geometry.cells] > 0.0
        notes.append(
            f"{gate_name!r} gates the spatial cells. A cell it reads as zero "
            f"draws as absent and not as zero."
        )
    elif geometry is not None:
        notes.append(
            "No signal gates the spatial cells, so every cell draws as "
            "present. Name one with the gate argument to tell an empty cell "
            "from a cell that holds zero."
        )

    panels = tuple(
        _panels_of(signal, values, geometry, present_cells, catalogue.geometry)
        for signal in spatial
    )
    flat = tuple(panel for group in panels for panel in group)
    blocks = _blocks_of(scalar, values)
    blocks = blocks + _unclaimed_blocks(flat, blocks, catalogue.observation_length)
    return Page(
        caption=caption,
        notes=tuple(notes),
        panels=flat,
        blocks=blocks,
        geometry=geometry,
        length=catalogue.observation_length,
    )


def _panels_of(
    signal: Signal,
    values: npt.NDArray[np.float64],
    geometry: RingStack | Lattice | None,
    present: npt.NDArray[np.bool_] | None,
    schema_geometry: Mapping[str, object],
) -> tuple[Panel, ...]:
    """Build one panel for each channel of one spatial signal."""
    if geometry is None:  # pragma: no cover - a spatial signal implies geometry
        raise AssertionError
    order = schema_geometry.get("channel_order")
    windows = _channel_windows(
        signal, geometry.cells, None if order is None else str(order)
    )
    positions = np.arange(signal.start, signal.start + signal.positions, dtype=np.int64)
    built = []
    for channel, window in windows:
        cell_positions = positions[window]
        cell_values = values[cell_positions]
        mask = np.ones(cell_values.shape, dtype=bool) if present is None else present
        built.append(
            Panel(
                title=channel,
                values=cell_values,
                present=mask,
                positions=cell_positions,
            )
        )
    return tuple(built)


def _blocks_of(
    signals: Sequence[Signal], values: npt.NDArray[np.float64]
) -> tuple[Block, ...]:
    """Group every scalar position into the blocks the schema names."""
    grouped: dict[str, list[Bar]] = {}
    groups = _blocks_by_name(signals)
    for signal in signals:
        bars = grouped.setdefault(groups[signal.name], [])
        for offset in range(signal.positions):
            position = signal.start + offset
            label = signal.name if signal.scalar else f"{signal.name}[{offset}]"
            bars.append(
                Bar(
                    label=label,
                    value=float(values[position]),
                    present=True,
                    positions=(position,),
                )
            )
    return tuple(Block(name=name, bars=tuple(bars)) for name, bars in grouped.items())


def _unclaimed_blocks(
    panels: Sequence[Panel],
    blocks: Sequence[Block],
    length: int,
) -> tuple[Block, ...]:
    """Draw every position that no field of the schema covers.

    A reserve position is not a position that holds zero. The page shows each
    run of them as one hatched bar, so that a reader sees the reserve and the
    page still reports that it covered the whole array.
    """
    seen = np.zeros(length, dtype=bool)
    for panel in panels:
        seen[panel.positions] = True
    for block in blocks:
        for bar in block.bars:
            for position in bar.positions:
                seen[position] = True
    if bool(seen.all()):
        return ()
    bars = []
    position = 0
    while position < length:
        if seen[position]:
            position += 1
            continue
        run = position
        while run < length and not seen[run]:
            run += 1
        span = tuple(range(position, run))
        label = (
            f"position {position}"
            if len(span) == 1
            else f"positions {position} to {run - 1}"
        )
        bars.append(Bar(label=label, value=0.0, present=False, positions=span))
        position = run
    return (Block(name="no field covers these", bars=tuple(bars)),)


def _ramp(stops: Sequence[tuple[int, int, int]], fraction: float) -> str:
    """Sample one colour ramp, and give back a hex colour."""
    clamped = min(max(fraction, 0.0), 1.0)
    span = clamped * (len(stops) - 1)
    low = min(int(span), len(stops) - 2)
    weight = span - low
    channels = (
        round(stops[low][index] + weight * (stops[low + 1][index] - stops[low][index]))
        for index in range(3)
    )
    return "#" + "".join(f"{value:02x}" for value in channels)


def _shade(value: float, scale: float, signed: bool) -> str:
    """Give the fill of one cell or one bar."""
    if signed:
        return _ramp(DIVERGING_RAMP, 0.5 + value / (2.0 * scale))
    return _ramp(SEQUENTIAL_RAMP, value / scale)


def _number(value: float) -> str:
    """Write one value short enough for a label."""
    if value == 0.0:
        return "0"
    magnitude = abs(value)
    if magnitude >= 1e6 or magnitude < 1e-3:
        return f"{value:.3g}"
    if float(value).is_integer():
        return str(int(value))
    return f"{value:.4g}"


def _escape(text: str) -> str:
    """Make one string safe inside an element of the picture."""
    return text.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def _point(cx: float, cy: float, radius: float, angle: float) -> tuple[float, float]:
    """Place one point at a radius and an angle, with the angle at the right."""
    return cx + radius * math.cos(angle), cy + radius * math.sin(angle)


def _wedge(
    cx: float,
    cy: float,
    inner: float,
    outer: float,
    start: float,
    end: float,
) -> str:
    """Write the path of one annulus wedge."""
    large = 1 if end - start > math.pi else 0
    ax, ay = _point(cx, cy, outer, start)
    bx, by = _point(cx, cy, outer, end)
    if inner <= 0.0:
        return (
            f"M {cx:.2f} {cy:.2f} L {ax:.2f} {ay:.2f} "
            f"A {outer:.2f} {outer:.2f} 0 {large} 1 {bx:.2f} {by:.2f} Z"
        )
    dx, dy = _point(cx, cy, inner, end)
    ex, ey = _point(cx, cy, inner, start)
    return (
        f"M {ax:.2f} {ay:.2f} "
        f"A {outer:.2f} {outer:.2f} 0 {large} 1 {bx:.2f} {by:.2f} "
        f"L {dx:.2f} {dy:.2f} "
        f"A {inner:.2f} {inner:.2f} 0 {large} 0 {ex:.2f} {ey:.2f} Z"
    )


def _text(
    x: float, y: float, body: str, size: float, fill: str = INK, anchor: str = "start"
) -> str:
    """Write one label."""
    return (
        f'<text x="{x:.2f}" y="{y:.2f}" font-size="{size}" fill="{fill}" '
        f'text-anchor="{anchor}">{_escape(body)}</text>'
    )


def _halo_text(x: float, y: float, body: str, size: float) -> str:
    """Write one label that must stay readable over any shade."""
    return (
        f'<text x="{x:.2f}" y="{y:.2f}" font-size="{size}" fill="{INK}" '
        f'text-anchor="middle" stroke="{PAPER}" stroke-width="2.4" '
        f'paint-order="stroke">{_escape(body)}</text>'
    )


def _draw_ring_panel(panel: Panel, stack: RingStack, x: float, y: float) -> list[str]:
    """Draw one channel as a polar plot of rings and sectors."""
    cx = x + PANEL_WIDTH / 2
    cy = y + 40 + PANEL_RADIUS
    band = PANEL_RADIUS / stack.rings
    scale = panel.scale
    signed = panel.signed
    parts = [
        _text(x + 8, y + 14, panel.title, 10.5),
        _text(x + 8, y + 27, panel.subtitle, 8.5, FAINT),
    ]
    for index in range(stack.cells):
        ring, sector = stack.place(index)
        count = stack.sectors(ring)
        inner = ring * band
        outer = (ring + 1) * band
        width = 2 * math.pi / count
        start = sector * width - width / 2
        fill = (
            _shade(float(panel.values[index]), scale, signed)
            if bool(panel.present[index])
            else ABSENT_FILL
        )
        if count == 1:
            shape = (
                f'<circle cx="{cx:.2f}" cy="{cy:.2f}" r="{outer:.2f}" '
                f'fill="{fill}" stroke="{PAPER}" stroke-width="0.7"/>'
            )
        else:
            path = _wedge(cx, cy, inner, outer, start, start + width)
            shape = (
                f'<path d="{path}" fill="{fill}" stroke="{PAPER}" '
                f'stroke-width="0.7"/>'
            )
        parts.append(shape)
    outermost = stack.sectors(stack.rings - 1)
    for sector in range(outermost):
        angle = sector * 2 * math.pi / outermost
        lx, ly = _point(cx, cy, PANEL_RADIUS + 11, angle)
        parts.append(_text(lx, ly + 3, str(sector), 7.5, FAINT, "middle"))
    for ring in range(stack.rings):
        ry = cy - (ring + 0.5) * band
        parts.append(_halo_text(cx, ry + 2.6, str(ring), 7.0))
    return parts


def _draw_grid_panel(panel: Panel, lattice: Lattice, x: float, y: float) -> list[str]:
    """Draw one channel as a square grid of lattice cells."""
    side = 2 * PANEL_RADIUS
    cell = side / lattice.side
    left = x + (PANEL_WIDTH - side) / 2
    top = y + 40
    scale = panel.scale
    signed = panel.signed
    parts = [
        _text(x + 8, y + 14, panel.title, 10.5),
        _text(x + 8, y + 27, panel.subtitle, 8.5, FAINT),
    ]
    for index in range(lattice.cells):
        row, column = lattice.place(index)
        fill = (
            _shade(float(panel.values[index]), scale, signed)
            if bool(panel.present[index])
            else ABSENT_FILL
        )
        cx = left + column * cell
        cy = top + row * cell
        parts.append(
            f'<rect x="{cx:.2f}" y="{cy:.2f}" width="{cell:.2f}" '
            f'height="{cell:.2f}" fill="{fill}" stroke="{PAPER}" '
            f'stroke-width="1"/>'
        )
        parts.append(_halo_text(cx + cell / 2, cy + cell / 2 - 3, str(index), 8.0))
        body = (
            _number(float(panel.values[index]))
            if bool(panel.present[index])
            else "absent"
        )
        parts.append(_halo_text(cx + cell / 2, cy + cell / 2 + 9, body, 7.5))
    return parts


def _draw_block(block: Block, x: float, y: float) -> tuple[list[str], float]:
    """Draw one scalar group as labelled bars, and give back its height."""
    parts = [_text(x, y + 12, block.name, 11.5)]
    scale = block.scale
    signed = any(bar.value < 0 for bar in block.bars if bar.present)
    rows = min(BAR_COLUMN_ROWS, len(block.bars))
    top = y + BLOCK_HEADING_HEIGHT
    for index, bar in enumerate(block.bars):
        column, row = divmod(index, BAR_COLUMN_ROWS)
        bx = x + column * BAR_COLUMN_WIDTH
        by = top + row * BAR_HEIGHT
        parts.append(_text(bx, by + 9, bar.label, 8.5))
        track = bx + BAR_LABEL_WIDTH
        parts.append(
            f'<rect x="{track}" y="{by + 1}" width="{BAR_TRACK_WIDTH}" '
            f'height="{BAR_HEIGHT - 3}" fill="#f2f4f3" stroke="#e2e6e5" '
            f'stroke-width="0.6"/>'
        )
        if not bar.present:
            parts.append(
                f'<rect x="{track}" y="{by + 1}" width="{BAR_TRACK_WIDTH}" '
                f'height="{BAR_HEIGHT - 3}" fill="{ABSENT_FILL}"/>'
            )
            parts.append(
                _text(track + BAR_TRACK_WIDTH + 6, by + 9, "absent", 8.5, FAINT)
            )
            continue
        share = min(abs(bar.value) / scale, 1.0)
        if signed:
            middle = track + BAR_TRACK_WIDTH / 2
            length = share * BAR_TRACK_WIDTH / 2
            start = middle if bar.value >= 0 else middle - length
        else:
            start = track
            length = share * BAR_TRACK_WIDTH
        parts.append(
            f'<rect x="{start:.2f}" y="{by + 1}" width="{max(length, 0.6):.2f}" '
            f'height="{BAR_HEIGHT - 3}" fill="'
            f'{_shade(bar.value, scale, signed)}"/>'
        )
        parts.append(
            _text(track + BAR_TRACK_WIDTH + 6, by + 9, _number(bar.value), 8.5, INK)
        )
    columns = math.ceil(len(block.bars) / BAR_COLUMN_ROWS)
    parts.append(
        _text(
            x + columns * BAR_COLUMN_WIDTH - 8,
            y + 12,
            f"full track {_number(scale)}",
            8.5,
            FAINT,
            "end",
        )
    )
    return parts, BLOCK_HEADING_HEIGHT + rows * BAR_HEIGHT + 12


def _legend(x: float, y: float) -> list[str]:
    """Draw what a shade and a hatch mean."""
    parts = [_text(x, y + 9, "shade", 8.5, FAINT)]
    steps = 24
    for step in range(steps):
        parts.append(
            f'<rect x="{x + 42 + step * 4}" y="{y}" width="4" height="10" '
            f'fill="{_ramp(SEQUENTIAL_RAMP, step / (steps - 1))}"/>'
        )
    parts.append(_text(x + 42, y + 20, "zero", 7.5, FAINT))
    parts.append(_text(x + 42 + steps * 4, y + 20, "full", 7.5, FAINT, "end"))
    parts.append(
        f'<rect x="{x + 158}" y="{y}" width="26" height="10" '
        f'fill="{ABSENT_FILL}" stroke="#e2e6e5" stroke-width="0.6"/>'
    )
    parts.append(_text(x + 190, y + 9, "absent, which is not zero", 8.5, FAINT))
    return parts


def render(page: Page) -> str:
    """Write one page of the observation as a picture."""
    body: list[str] = []
    body.append(_text(MARGIN, 26, page.caption, 15))
    line = 44
    for note in page.notes:
        body.append(_text(MARGIN, line, note, 9, FAINT))
        line += 12
    body.extend(_legend(MARGIN, line + 2))
    cursor = float(line + 34)

    columns = min(PANEL_COLUMNS, max(1, len(page.panels)))
    for index, panel in enumerate(page.panels):
        row, column = divmod(index, columns)
        x = MARGIN + column * PANEL_WIDTH
        y = cursor + row * PANEL_HEIGHT
        if isinstance(page.geometry, RingStack):
            body.extend(_draw_ring_panel(panel, page.geometry, x, y))
        elif isinstance(page.geometry, Lattice):
            body.extend(_draw_grid_panel(panel, page.geometry, x, y))
    if page.panels:
        rows = math.ceil(len(page.panels) / columns)
        cursor += rows * PANEL_HEIGHT + SECTION_GAP

    widest = columns * PANEL_WIDTH
    for block in page.blocks:
        parts, height = _draw_block(block, float(MARGIN), cursor)
        body.extend(parts)
        cursor += height
        widest = max(
            widest, math.ceil(len(block.bars) / BAR_COLUMN_ROWS) * BAR_COLUMN_WIDTH
        )

    width = widest + 2 * MARGIN
    height = int(cursor) + MARGIN
    head = (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" '
        f'height="{height}" viewBox="0 0 {width} {height}" '
        f'font-family="Inter, Helvetica, Arial, sans-serif">'
        f'<defs><pattern id="absent" width="6" height="6" '
        f'patternUnits="userSpaceOnUse" patternTransform="rotate(45)">'
        f'<rect width="6" height="6" fill="#f6f6f6"/>'
        f'<line x1="0" y1="0" x2="0" y2="6" stroke="#bcc4c2" '
        f'stroke-width="1.7"/></pattern></defs>'
        f'<rect width="{width}" height="{height}" fill="{PAPER}"/>'
    )
    return head + "".join(body) + "</svg>"


def geometry_of(catalogue: SignalCatalogue) -> RingStack | Lattice | None:
    """Read the spatial geometry of one layout, or nothing when it has none."""
    spatial, _, space, _ = _spatial_split(catalogue)
    if not spatial:
        return None
    if space == RING_SPACE:
        return ring_stack_of(catalogue.geometry, [signal.name for signal in spatial])
    return _lattice_of(spatial)


def picture_name(
    width: int, height: int, factions: int, seed: int, faction: int, tick: int
) -> str:
    """Name one picture from the world, the faction and the tick."""
    return (
        f"obs-{width}x{height}-f{factions}-seed{seed}"
        f"-faction{faction}-tick{tick:06d}.svg"
    )


def build_world(
    width: int, height: int, factions: int, seed: int, tick_limit: int
) -> World:
    """Build one world and let the built-in controller play every seat.

    The tool draws what a faction sees. It does not train, so it takes no seat
    and it leaves every seat to the built-in controller. The world then moves
    on its own and the pictures show a game rather than a still world.
    """
    world = World(width=width, height=height, seed=seed, faction_count=factions)
    world.seed_world()
    world.set_win_readers_enabled(True)
    world.set_tick_limit(tick_limit)
    return world


def _animation(names: Sequence[str], seconds: float) -> str:
    """Write one page that shows the frames in order, with no script.

    The page holds one image for each frame and one keyframe rule. Each image
    carries a delay of its own, so exactly one image is opaque at a time. A
    reader opens the page in a browser and watches the observation change.
    """
    total = max(seconds * len(names), 0.001)
    share = 100.0 / len(names)
    frames = "".join(
        f'<img src="{_escape(name)}" style="animation-delay:'
        f'{index * seconds - total:.3f}s">'
        for index, name in enumerate(names)
    )
    return (
        '<!doctype html><meta charset="utf-8">'
        "<title>the observation over time</title>"
        "<style>body{margin:0;background:#fff}"
        ".stack{position:relative}"
        "img{position:absolute;top:0;left:0;opacity:0;"
        f"animation:flip {total:.3f}s steps(1,end) infinite}}"
        "@keyframes flip{0%{opacity:1}"
        f"{share:.4f}%{{opacity:0}}100%{{opacity:0}}}}"
        f'</style><div class="stack">{frames}</div>'
    )


def sequence(
    directory: Path,
    width: int,
    height: int,
    factions: int,
    seed: int,
    faction: int,
    ticks: int,
    decision_interval: int,
    threads: int = 1,
    gate: str | None = None,
    animate: bool = True,
) -> list[Path]:
    """Run one world and write one picture for each decision.

    The function writes a picture at tick zero and after each interval, so a
    reader watches the observation change over the run. It stops early when
    the game ends, because a faction publishes no further change after that.
    """
    directory.mkdir(parents=True, exist_ok=True)
    world = build_world(width, height, factions, seed, ticks)
    catalogue = SignalCatalogue.of_world(world)
    written: list[Path] = []
    tick = 0
    while True:
        observation = np.asarray(world.faction_observation(faction))
        caption = (
            f"faction {faction} of {factions}, world {width} by {height}, "
            f"seed {seed}, tick {tick}, "
            f"{catalogue.observation_length} positions, schema version "
            f"{catalogue.geometry.get('version')}"
        )
        page = read_page(catalogue, observation, caption, gate)
        name = picture_name(width, height, factions, seed, faction, tick)
        path = directory / name
        path.write_text(render(page), encoding="utf-8")
        written.append(path)
        if tick >= ticks or world.game_end() is not None:
            break
        for _ in range(min(decision_interval, ticks - tick)):
            world.step(threads)
        tick = int(world.tick)
    if animate and len(written) > 1:
        page_path = directory / (
            f"obs-{width}x{height}-f{factions}-seed{seed}"
            f"-faction{faction}-sequence.html"
        )
        page_path.write_text(
            _animation([path.name for path in written], 0.5), encoding="utf-8"
        )
        written.append(page_path)
    return written


def main(argv: Sequence[str] | None = None) -> int:
    """Run one world and write the pictures of one faction."""
    parser = argparse.ArgumentParser(
        prog="python -m cachette.learn.picture",
        description="Draw what one faction sees, one picture for each decision.",
    )
    parser.add_argument("--width", type=int, default=48)
    parser.add_argument("--height", type=int, default=48)
    parser.add_argument("--factions", type=int, default=3)
    parser.add_argument("--seed", type=int, default=7)
    parser.add_argument("--faction", type=int, default=0)
    parser.add_argument("--ticks", type=int, default=300)
    parser.add_argument("--decision-interval", type=int, default=50)
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument(
        "--gate",
        default=None,
        help=(
            "the signal that says which spatial cells hold a value. A cell it "
            "reads as zero draws as absent."
        ),
    )
    parser.add_argument("--out", type=Path, default=Path("pictures"))
    parser.add_argument("--no-animation", action="store_true")
    args = parser.parse_args(argv)
    written = sequence(
        directory=args.out,
        width=args.width,
        height=args.height,
        factions=args.factions,
        seed=args.seed,
        faction=args.faction,
        ticks=args.ticks,
        decision_interval=args.decision_interval,
        threads=args.threads,
        gate=args.gate,
        animate=not args.no_animation,
    )
    for path in written:
        print(path)
    return 0


if __name__ == "__main__":  # pragma: no cover - the command line entry
    raise SystemExit(main())
