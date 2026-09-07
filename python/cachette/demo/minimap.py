"""The round minimap the control plane paints over the frame.

The minimap shows a wide view of the world around the camera. It is chrome,
not the world: it reads the engine, mixes colours into the pixel buffer the
engine already filled, and writes nothing back. Nothing here touches the
step, a simulated value or the state hash.

**The minimap reads the summary level, not the tiles.** The engine holds a
pyramid over the world. Level 0 holds the tiles. Level 1 summarises a square
block of them, and it exists so that a caller reads a wide view without
walking every tile. The minimap asks the summary level for one entry for each
block.[^1]

**The summary level does not say which faction holds a block.** It counts the
tiles a faction holds and states plainly that it does not name the faction.
The minimap therefore takes the colour of a holder from the tile holder
raster, which is one bulk array and no loop, and it takes the strength of the
tint from the summary. A reader who removes the raster still sees which ground
is held, and loses only the name of the holder.

**A black rim closes the disc.** The disc used to fade to nothing at its
edge, and a fade gives an instrument no edge, so the map appeared to leak
into the frame under it. The rim is polished black: a key light from the
upper left catches its outer shoulder, a weaker bounced light lifts the far
side, and a thin lit line runs the whole way round. The rim throws a short
shadow onto the map, so it sits above the map rather than on it.

References
----------
ADR-0022, level 0 is the only truth, and every level above it is derived,
decision D1.
``docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING, NamedTuple

import numpy as np

from cachette import Camera, faction_colours
from cachette.demo.page import INK
from cachette.demo.page import PAPER as PAGE_PAPER

if TYPE_CHECKING:
    from cachette import World
    from cachette.demo.surface import Surface

# How wide the disc is, in pixels.
#
# **A minimap that shows everything shows nothing.** At this size one level 1
# block covers about thirty pixels in the demonstration world, which carries a
# coast and a slope and carries no detail below that. The layers stop at
# three for the same reason.
#
# The disc was 168 pixels across and a watcher could not read the holders on
# it. Every number that shapes the disc derives from this one, and the stride
# of the coarse grid divides it, so the size is the only edit the change needs.
DIAMETER = 252

# How far the disc sits from the top right corner of the window, in pixels.
#
# This is the margin of the minimap and not the margin of a card. The engine
# owns where a card goes, and this module owns where the disc goes.
MARGIN = 14

# The share of the radius that the black rim takes.
#
# The disc used to fade to nothing at its edge. A fade reads as a window and
# it gives the instrument no edge, so the map appeared to leak into the frame
# below it. A rim gives the disc a body and states where it stops.
BEZEL_SHARE = 0.115

# How far the rim reaches over the map as a shadow, as a share of the radius.
#
# A rim that sits on the map with no shadow reads as a ring painted onto the
# picture. A short shadow under its inner edge lifts the rim off the map.
BEZEL_SHADOW = 0.055

# How much of the map colour the shadow of the rim keeps at its darkest.
SHADOW_KEEP = 0.34

# The value of the rim where no light reaches it, from zero to one.
#
# The rim is black and not pure black. A pure black ring against a dark frame
# has no shape at all, and the eye reads a hole.
BEZEL_BLACK = 0.045

# How much the key light adds to the rim, from zero to one.
#
# The light comes from the upper left, which is where a viewer expects a light
# to be. It catches the outer shoulder of the rim and makes it read as round.
BEZEL_KEY = 0.86

# How much the bounced light adds to the rim, from zero to one.
#
# **One highlight reads as a flat ring with a bright spot.** A polished thing
# also carries a weaker light on the side away from the key, because the
# ground around it throws light back. The second light is what makes the rim
# read as polished rather than as painted.
BEZEL_BOUNCE = 0.30

# How much the thin line at the outer edge of the rim adds, from zero to one.
#
# The line runs the whole way round, so the disc keeps a silhouette on the
# side the key light does not reach.
BEZEL_LIP = 0.16

# The blue that the key light of the rim leans by, from zero to one.
#
# A polished black surface takes the colour of what it reflects. The lean is
# small, and it is what separates the rim from a grey ring.
BEZEL_COOL = 0.10

# The share of a pixel over which the outer edge of the rim softens.
#
# A circle drawn on a square grid with no softening reads as a staircase. One
# pixel is enough, and more would bring back the fade the rim replaced.
BEZEL_SOFTEN = 1.4

# How much of the minimap colour reaches the frame at the middle of the disc.
#
# A fully opaque disc reads as a hole in the window. A little of the frame
# below it keeps the disc part of the picture.
STRENGTH = 0.88

# How many level 1 blocks the disc spans.
#
# **The span is stated in blocks, because the summary level is what the disc
# draws.** A span in tiles would state the block edge a second time, and the
# engine already owns that number.
SPAN_CELLS = 6

# How many frames pass between two reads of the summary level.
#
# The ground never changes and the holders change over many ticks, so a read
# on every frame would pay the whole cost of the summary for a picture that
# does not move. The camera still moves every frame, and the disc follows it
# every frame.
REBUILD_FRAMES = 20

# The colour of water, of the lowest land and of the highest land.
WATER_COLOUR = 0x14324F
LOW_COLOUR = 0x3C6B40
HIGH_COLOUR = 0xD8CCA6

# The same three colours for a page of ink on paper.
#
# **The disc belongs to the drawing it sits on.** The sketch draws the world
# as ink on one paper, and a polished instrument in saturated colour over that
# page reads as a widget from another program. On paper the water is a pale
# wash, the low ground is near the paper itself, and the high ground is the
# ground the pencil worked hardest.
#
# The faction colours are not repeated here. The engine states them, the
# sketch washes the ground with them, and the pips take the same table.
PAPER_WATER_COLOUR = 0xB9CAD8
PAPER_LOW_COLOUR = 0xEBE3D1
PAPER_HIGH_COLOUR = 0x9C917C
PAPER_OUTSIDE_COLOUR = 0xD9D1BF

# The rim and the outline of the view are drawn in the ink of the page, and
# the page declares it. Nothing here holds a second copy of that colour.
PAPER_VIEW_COLOUR = INK
PAPER_INK = INK

# How much ink the drawn rim of the paper disc takes at its edges.
#
# The rim is two rules with paper between them. A drawn instrument on a page
# has an edge and no shine, and two rules give it that edge.
PAPER_RIM_INK = 0.85
PAPER_RIM_INNER = 0.34

# The colour of the outline that marks where the camera is looking.
VIEW_COLOUR = 0xF2F2F2

# The colour of the ground outside the world.
#
# The disc is centred on the camera and is not held inside the world, so a
# camera near an edge shows ground that does not exist. This paints it, so a
# watcher reads the edge of the world rather than a repeated block.
OUTSIDE_COLOUR = 0x0B1118

# The holder entry that means that nobody holds the tile.
NOBODY = 0xFFFF

# How wide the outline of the camera view is, in pixels.
VIEW_WIDTH = 1.6

# The smallest and the largest pip that marks a held block, as a share of the
# block.
#
# **A held block is rarely a held block.** A young city holds about a seventh
# of the thousand tiles of its block, so a wash over the whole block reads as
# nothing. A pip at the middle of the block reads at this size, and its size
# says how much of the block the faction holds.
PIP_LEAST = 0.09
PIP_MOST = 0.30

# How many pixels lie between two readings of the summary.
#
# **The summary is coarse, so reading it at every pixel is waste.** One block
# covers about thirty pixels of the disc, and the three smooth layers cross a
# block as a gradient. A reading every few pixels therefore carries the whole
# shape, and the pixels between two readings are filled by a stretch, which
# costs a fraction of a reading. The pips and the outline are drawn after the
# stretch, at every pixel, because an edge must stay sharp.
STRIDE = 4

# The share of a block that must be water before the disc draws water.
#
# A block of a thousand tiles is rarely all land or all water, so a threshold
# names the coast. Half is the value that puts the coast where a watcher of
# the main map sees it.
WATER_SHARE = 0.5


class Palette(NamedTuple):
    """The colours one disc is drawn in.

    Two palettes exist. One belongs to the flat map the engine draws, and one
    belongs to the page of ink on paper the sketch draws. **The palette
    crosses as one value**, so a caller cannot take the water of one and the
    rim of the other.
    """

    water: int
    low: int
    high: int
    view: int
    outside: int
    paper: bool


# The palette of the flat map, and the palette of the drawn page.
SCREEN = Palette(
    WATER_COLOUR, LOW_COLOUR, HIGH_COLOUR, VIEW_COLOUR, OUTSIDE_COLOUR, False
)
PAPER = Palette(
    PAPER_WATER_COLOUR,
    PAPER_LOW_COLOUR,
    PAPER_HIGH_COLOUR,
    PAPER_VIEW_COLOUR,
    PAPER_OUTSIDE_COLOUR,
    True,
)


def _block_edge(world: World) -> int:
    """Give back the edge of one level 1 block, in tiles.

    The engine owns the block edge and does not state it. It states how many
    blocks lie across the world, and it answers about the block that covers a
    tile. This derives the edge from the tile count of the first block, and
    then checks the derivation against the block count the engine gives. A
    derivation that disagrees with the engine raises, so the two cannot drift
    apart in silence.
    """
    across = int(world.cells_wide)
    first = int(world.region_summary(0, 0)["tiles"])
    edge = math.isqrt(first)
    if edge < 1 or -(-int(world.width) // edge) != across:
        message = (
            f"cannot derive the block edge: the first block holds {first} tiles "
            f"and the world is {world.width} tiles across in {across} blocks"
        )
        raise ValueError(message)
    return edge


class Summary:
    """The level 1 lattice of one world, as one array for each layer.

    Every array holds one entry for each block, in row order. The water array
    holds the share of the block that admits no unit. The height array holds
    the mean height of the block, scaled to run from zero to one over the
    world. The held array holds the share of the block that a faction holds.
    The holder array names the faction that holds the most tiles of the block,
    or nobody.
    """

    __slots__ = (
        "across",
        "down",
        "edge",
        "held",
        "holder",
        "layers",
        "marks",
        "table",
        "water",
    )

    def __init__(
        self,
        across: int,
        down: int,
        edge: int,
        water: np.ndarray,
        height: np.ndarray,
        held: np.ndarray,
        holder: np.ndarray,
    ) -> None:
        """Hold one lattice."""
        self.across = across
        self.down = down
        self.edge = edge
        self.water = water
        self.held = held
        self.holder = holder
        # The three layers a pixel reads together, as one block with a layer
        # for its third axis. A reading of one address wants all three, and
        # one array asks for them once.
        stacked = np.stack([water, height, held], axis=-1)
        self.layers = stacked.astype(np.float32).reshape(down, across, 3)
        # The two layers a pip reads together: how much of a block is held,
        # and who holds it. A pip must not blur across a block boundary, so
        # these are read block by block and not mixed.
        self.marks = np.stack(
            [held.astype(np.float32), holder.astype(np.float32)], axis=-1
        ).reshape(down, across, 2)
        # The colour of each faction, from the engine.
        self.table = _table()


def _holders(world: World, across: int, down: int, edge: int) -> np.ndarray:
    """Give back the faction that holds the most tiles of each block.

    **The summary level does not carry this.** It counts the held tiles of a
    block and states that it does not name the faction, so the colour of a
    holder can only come from the tile holder raster. The raster crosses the
    boundary once as one array, and this reduces it with array operations. No
    step of it loops over a tile.

    The world is not always a whole number of blocks across, so the raster is
    padded with the value that means nobody before it is folded.
    """
    width, height = int(world.width), int(world.height)
    raster = np.asarray(world.tile_holders()).reshape(height, width)
    padded = np.full((down * edge, across * edge), NOBODY, dtype=np.uint16)
    padded[:height, :width] = raster
    blocks = padded.reshape(down, edge, across, edge)
    best = np.full((down, across), NOBODY, dtype=np.uint16)
    most = np.zeros((down, across), dtype=np.int64)
    for faction in range(int(world.faction_count)):
        count = (blocks == faction).sum(axis=(1, 3))
        wins = count > most
        most = np.where(wins, count, most)
        best = np.where(wins, np.uint16(faction), best)
    return best.reshape(-1)


def read_summary(world: World) -> Summary:
    """Read the level 1 lattice of the world.

    The engine answers about one block for each call, so this makes one call
    for each block. **The block count is small because the level is a
    summary**: it is the tile count divided by the square of the block edge.

    **The engine offers no call that answers about every block at once.** The
    one array of block entries it gives is the observation of a faction, and
    that one is masked to what the faction has seen, which is not what a
    watcher of the whole world wants. A world at the target scale holds
    sixteen thousand blocks, and a loop of that length belongs behind one
    call. The loop below is therefore right for the demonstration world and
    wrong for a large one, and the reading is kept between frames because of
    it.

    The height of a block is scaled against the lowest and the highest block
    of the land, so a flat world still shows its relief.
    """
    across = int(world.cells_wide)
    edge = _block_edge(world)
    down = -(-int(world.height) // edge)
    water = np.zeros(across * down, dtype=np.float64)
    height = np.zeros(across * down, dtype=np.float64)
    held = np.zeros(across * down, dtype=np.float64)
    for row in range(down):
        for column in range(across):
            cell = world.region_summary(column * edge, row * edge)
            tiles = float(cell["tiles"])
            at = row * across + column
            if tiles <= 0.0:
                continue
            water[at] = 1.0 - float(cell["open_tiles"]) / tiles
            height[at] = float(cell["height_total"]) / tiles / 65536.0
            held[at] = float(cell["held_tiles"]) / tiles
    land = water < WATER_SHARE
    if land.any():
        low = float(height[land].min())
        high = float(height[land].max())
        if high > low:
            height = (height - low) / (high - low)
        else:
            height = np.full_like(height, 0.5)
    return Summary(
        across,
        down,
        edge,
        water,
        np.clip(height, 0.0, 1.0),
        held,
        _holders(world, across, down, edge),
    )


def _channel(colour: int, shift: int) -> float:
    """Give back one channel of a packed colour, from zero to one."""
    return float((colour >> shift) & 0xFF) / 255.0


def _corners(camera: Camera, width: int, height: int) -> np.ndarray:
    """Give back the tile address at each corner of a view, as four rows.

    The engine turns a pixel into a tile address. The turn is affine, so four
    corners state the whole map and a reader interpolates between them
    exactly. This asks the engine four times and never once for each pixel.
    """
    places = ((0.0, 0.0), (float(width), 0.0), (0.0, float(height)))
    return np.array(
        [camera.tile_at(x, y) for x, y in places],
        dtype=np.float32,
    )


class Minimap:
    """The round wide view, and whether the watcher wants it.

    The control plane holds one of these between frames. It holds the lattice
    it last read and how old that reading is, and nothing else.
    """

    __slots__ = (
        "_age",
        "_colour",
        "_key",
        "_summary",
        "_turn",
        "_weight",
        "palette",
        "visible",
    )

    def __init__(self, visible: bool = True, palette: Palette = SCREEN) -> None:
        """Build a minimap that has read nothing yet.

        The palette says which page the disc belongs to. A caller that swaps
        the renderer swaps this, and the disc is built again on the next
        frame because the palette is part of what the built disc was built
        for.
        """
        self.visible = visible
        self.palette = palette
        self._summary: Summary | None = None
        self._age = 0
        # Which reading of the lattice the built disc came from. A number and
        # not the lattice itself, because a new lattice can sit where the old
        # one sat and a comparison would then miss the change.
        self._turn = 0
        # What the built disc was built for, and the disc itself. The disc is
        # a function of the lattice and of the two cameras, and nothing else,
        # so a frame in which none of them moved reuses it.
        self._key: tuple[object, ...] | None = None
        self._colour: np.ndarray | None = None
        self._weight: np.ndarray | None = None

    def toggle(self) -> bool:
        """Show the minimap if it is hidden, hide it if it is shown.

        Gives back whether it is now shown.
        """
        self.visible = not self.visible
        return self.visible

    def forget(self) -> None:
        """Drop the lattice, so that the next frame reads it again."""
        self._summary = None
        self._age = 0
        self._key = None

    def summary(self, world: World) -> Summary:
        """Give back the lattice, and read it again when it is old."""
        if self._summary is None or self._age >= REBUILD_FRAMES:
            self._summary = read_summary(world)
            self._age = 0
            self._turn += 1
        self._age += 1
        return self._summary

    def paint(self, world: World, camera: Camera, surface: Surface) -> bool:
        """Paint the disc over the frame, and say whether it painted.

        A hidden minimap paints nothing and gives back ``False``. A disc that
        does not fit in the window paints nothing as well.

        **A frame in which nothing moved reuses the disc it drew last time.**
        The disc is a function of the lattice and of the two cameras, and the
        engine reports a camera as whole tiles, so a camera that has not
        crossed a tile gives the same disc. The frame below it still changes
        every frame, so the mixing runs every frame.
        """
        if not self.visible:
            return False
        if surface.width < DIAMETER + MARGIN or surface.height < DIAMETER + MARGIN:
            return False
        summary = self.summary(world)
        wide, zoom = self._window(world, camera, surface.width, surface.height)
        seen = _corners(camera, surface.width, surface.height)
        key = (
            self._turn,
            self.palette,
            _corners(wide, DIAMETER, DIAMETER).tobytes(),
            seen.tobytes(),
        )
        if key != self._key or self._colour is None or self._weight is None:
            self._colour, self._weight = _build(
                world, summary, wide, zoom, seen, self.palette
            )
            self._key = key
        left = surface.width - MARGIN - DIAMETER
        _mix(surface, left, MARGIN, self._colour, self._weight)
        return True

    def _window(
        self, world: World, camera: Camera, width: int, height: int
    ) -> tuple[Camera, float]:
        """Build the camera that the disc looks through, and how far it zooms.

        The camera of the disc looks at the tile in the middle of the window,
        so the disc follows the watcher.

        **The engine holds a camera to a smallest tile size**, and the span of
        the minimap is below it. The disc therefore takes the map of the
        engine and stretches it about its own middle. A stretch about a point
        is exact on an affine map, so the stretched map is still the map the
        engine states, at a scale the engine will not hold.
        """
        summary = self.summary(world)
        span = SPAN_CELLS * summary.edge
        wide = Camera()
        middle = camera.tile_at(width / 2.0, height / 2.0)
        wide.look_at(middle[0], middle[1], DIAMETER, DIAMETER)
        return wide, span * wide.tile_width / DIAMETER

    def looks_at(
        self, world: World, camera: Camera, width: int, height: int
    ) -> tuple[int, int]:
        """Give back the tile at the middle of the disc.

        A caller reads this to check that the disc follows the camera.
        """
        wide, _ = self._window(world, camera, width, height)
        return wide.tile_at(DIAMETER / 2.0, DIAMETER / 2.0)


def _build(
    world: World,
    summary: Summary,
    wide: Camera,
    zoom: float,
    seen: np.ndarray,
    palette: Palette = SCREEN,
) -> tuple[np.ndarray, np.ndarray]:
    """Build the colour of every pixel of the disc, and how much reaches.

    The colour comes from three layers of the summary: water against land,
    the height of the land, and the faction that holds the ground. A black rim
    goes round the outside, and the weight stops at the outer edge of it.

    The colour comes back as one byte for each of blue, green and red, in the
    order the frame holds them. The weight comes back as a whole number from
    zero to 256. Both are what the mixing wants, so the mixing does no
    conversion on a frame that reuses the disc.
    """
    rows, columns = _grid()
    q, r, step = _address_field(wide, columns + 0.5, rows + 0.5, zoom)
    coarse = _address_field(wide, *_coarse_grid(), zoom)
    colour = _paint_layers(world, summary, q, r, coarse[0], coarse[1], step, palette)
    _outline(colour, seen, q, r, palette)
    # The rim goes on last, so it covers the map and the camera outline. A rim
    # that the outline crossed would read as a broken ring.
    if palette.paper:
        _paper_bezel(colour, rows, columns)
    else:
        _bezel(colour, rows, columns)
    # The frame holds blue, green and red in that order, and the layers give
    # red, green and blue in that order.
    packed = np.clip(colour[..., ::-1] * 255.0, 0.0, 255.0).astype(np.uint16)
    weight = np.clip(_fade(rows, columns) * 256.0, 0.0, 256.0).astype(np.uint16)[
        ..., None
    ]
    # **The colour is scaled by its weight now, not on every frame.** The disc
    # holds still between two frames and the frame below it does not, so the
    # part of the mixing that only the disc decides belongs here.
    return packed * weight, (256 - weight).astype(np.uint16)


def _grid() -> tuple[np.ndarray, np.ndarray]:
    """Give back the row and the column of every pixel of the disc."""
    rows, columns = np.mgrid[0:DIAMETER, 0:DIAMETER]
    return rows.astype(np.float32), columns.astype(np.float32)


def _coarse_grid() -> tuple[np.ndarray, np.ndarray]:
    """Give back the place of every reading of the summary.

    The readings sit on a square grid whose step is the stride. The grid runs
    one place past the disc in each direction, so the stretch that follows
    has a reading on both sides of every pixel.
    """
    steps = DIAMETER // STRIDE + 1
    line = np.arange(steps, dtype=np.float32) * STRIDE + 0.5
    return line[None, :] * np.ones((steps, 1), dtype=np.float32), line[
        :, None
    ] * np.ones((1, steps), dtype=np.float32)


def _stretch(coarse: np.ndarray) -> np.ndarray:
    """Stretch a grid of readings to one value for every pixel of the disc.

    Each row and each column of the readings is repeated by the stride, and
    the two neighbours of a pixel are then mixed by how far it lies between
    them. **A repeat copies, and it does not look a value up**, so this costs
    a fraction of a reading for each pixel.
    """
    steps = (coarse.shape[0] - 1) * STRIDE
    share = ((np.arange(steps, dtype=np.float32) % STRIDE) / STRIDE)[:, None, None]
    top = np.repeat(coarse[:-1], STRIDE, axis=0)
    tall = top + (np.repeat(coarse[1:], STRIDE, axis=0) - top) * share
    left = np.repeat(tall[:, :-1], STRIDE, axis=1)
    right = np.repeat(tall[:, 1:], STRIDE, axis=1)
    wide: np.ndarray = left + (right - left) * share.reshape(1, steps, 1)
    return wide[:DIAMETER, :DIAMETER]


def _address_field(
    wide: Camera, x: np.ndarray, y: np.ndarray, zoom: float
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Give back the tile address of every pixel of the disc, and the step.

    The step is the two by two table that turns a step of one pixel across
    and a step of one pixel down into a step in tile addresses. A caller that
    must measure a distance on the screen inverts it, so no caller repeats the
    map that the engine owns.

    The engine turns a pixel into a tile address, and the turn is affine. This
    reads three corners from the engine and interpolates the rest, so the
    field costs three calls and no loop.

    The zoom stretches the field about the middle of the disc, which widens
    the view without moving what the middle of the disc looks at.
    """
    corners = _corners(wide, DIAMETER, DIAMETER)
    origin = corners[0]
    across = (corners[1] - origin) / DIAMETER
    down = (corners[2] - origin) / DIAMETER
    middle = DIAMETER / 2.0
    dx = (x - middle) * zoom + middle
    dy = (y - middle) * zoom + middle
    q = origin[0] + across[0] * dx + down[0] * dy
    r = origin[1] + across[1] * dx + down[1] * dy
    step = np.array(
        [[across[0] * zoom, down[0] * zoom], [across[1] * zoom, down[1] * zoom]],
        dtype=np.float32,
    )
    return q, r, step


def _sample(
    field: np.ndarray, summary: Summary, q: np.ndarray, r: np.ndarray
) -> np.ndarray:
    """Read the layers of the lattice at every tile address, smoothly.

    The lattice is coarse, so a nearest reading gives a field of squares. This
    mixes the four blocks around the address, which turns the squares into a
    gradient and keeps the coast where the summary puts it.

    **The layers are read together, not one at a time.** The four blocks
    around an address are the same four for every layer, so one reading
    serves them all.
    """
    column = np.clip(q / summary.edge - 0.5, 0.0, summary.across - 1.0)
    row = np.clip(r / summary.edge - 0.5, 0.0, summary.down - 1.0)
    left = column.astype(np.intp)
    top = row.astype(np.intp)
    right = np.minimum(left + 1, summary.across - 1)
    bottom = np.minimum(top + 1, summary.down - 1)
    fx = (column - left)[..., None]
    fy = (row - top)[..., None]
    # **The blocks are read from a flat array by one index.** A reading of a
    # block by its row and its column asks the array library for a step over
    # two axes, and that costs several times as much as one step over one.
    flat = field.reshape(-1, field.shape[-1])
    top_row = top * summary.across
    bottom_row = bottom * summary.across
    upper = flat[top_row + left] * (1.0 - fx) + flat[top_row + right] * fx
    lower = flat[bottom_row + left] * (1.0 - fx) + flat[bottom_row + right] * fx
    mixed: np.ndarray = upper * (1.0 - fy) + lower * fy
    return mixed


def _nearest(
    values: np.ndarray, summary: Summary, q: np.ndarray, r: np.ndarray
) -> np.ndarray:
    """Read one layer of the lattice at every tile address, block by block.

    A faction number is a name and not a quantity, so mixing two of them would
    give a third faction. This takes the block the address falls in.
    """
    field = values.reshape(summary.down * summary.across, -1)
    column = np.clip((q / summary.edge).astype(np.intp), 0, summary.across - 1)
    row = np.clip((r / summary.edge).astype(np.intp), 0, summary.down - 1)
    taken: np.ndarray = field[row * summary.across + column]
    return taken[..., 0] if values.ndim == 1 else taken


def _table() -> np.ndarray:
    """Give back one row of red, green and blue for each faction, and nobody.

    The engine states the faction colours, so the control plane holds no
    second table of them. The last row stands for a block that nobody holds,
    and nothing reads it, because a pip needs a holder.
    """
    colours = [*faction_colours(), 0]
    rows = [[_channel(colour, shift) for shift in (16, 8, 0)] for colour in colours]
    return np.array(rows, dtype=np.float32)


def _flat(colour: int) -> np.ndarray:
    """Give back one packed colour as three numbers from zero to one."""
    return np.array([_channel(colour, shift) for shift in (16, 8, 0)], dtype=np.float32)


def _paint_layers(
    world: World,
    summary: Summary,
    q: np.ndarray,
    r: np.ndarray,
    coarse_q: np.ndarray,
    coarse_r: np.ndarray,
    step: np.ndarray,
    palette: Palette = SCREEN,
) -> np.ndarray:
    """Mix the three layers of the summary into one colour for every pixel.

    The land runs from the low colour to the high colour by its height. The
    water is one colour. A faction that holds part of a block gets a pip at
    the middle of the block and a wash over it, so a heartland reads stronger
    than a frontier.

    **The colour is mixed on the coarse grid and then stretched.** The ground
    and the wash are gradients that cross a block, so a pixel between two
    readings holds no shape of its own. The pip, the outline and the edge of
    the world are drawn after the stretch, at every pixel, because each of
    them is an edge and an edge must stay sharp.

    **The three channels are one array with a third axis**, not three arrays
    in a loop. The work is the same for each channel, so a loop over the
    three would run the whole pipeline three times.
    """
    layers = _sample(summary.layers, summary, coarse_q, coarse_r)
    holder = _nearest(summary.holder, summary, coarse_q, coarse_r)
    height = layers[..., 1:2]
    wet = np.clip((layers[..., 0] - WATER_SHARE) * 6.0 + 0.5, 0.0, 1.0)[..., None]
    low = _flat(palette.low)
    ground = (low + height * (_flat(palette.high) - low)) * (1.0 - wet) + _flat(
        palette.water
    ) * wet
    tint = summary.table[np.where(holder == NOBODY, summary.table.shape[0] - 1, holder)]
    wash = np.where(holder == NOBODY, 0.0, layers[..., 2])[..., None] * (1.0 - wet)
    share = np.sqrt(wash) * 0.45
    # The stretch runs once over the body and the tint together, because the
    # pip takes the tint at full size and the two grids are the same grid.
    stretched = _stretch(
        np.concatenate([ground * (1.0 - share) + tint * share, tint], axis=-1)
    )
    marks = _nearest(summary.marks, summary, q, r)
    pip = _pips(summary, marks[..., 0], marks[..., 1], q, r, step)
    painted = np.where(pip[..., None], stretched[..., 3:], stretched[..., :3])
    outside = ((q < 0.0) | (r < 0.0) | (q >= world.width) | (r >= world.height))[
        ..., None
    ]
    return np.where(outside, _flat(palette.outside), painted)


def _pips(
    summary: Summary,
    held: np.ndarray,
    holder: np.ndarray,
    q: np.ndarray,
    r: np.ndarray,
    step: np.ndarray,
) -> np.ndarray:
    """Say which pixels of the disc fall inside the pip of a held block.

    The pip sits at the middle of its block and grows with the share of the
    block the faction holds. This measures every pixel against the middle of
    the block it falls in, so no step of it looks up a pixel for a block.

    **The map is a hex grid, so a circle in tile addresses is not a circle on
    the screen.** The step table turns a distance in tile addresses back into
    a distance in pixels, and the pip is round because the measurement is.
    """
    edge = float(summary.edge)
    middle_q = (np.floor(q / edge) + 0.5) * edge
    middle_r = (np.floor(r / edge) + 0.5) * edge
    determinant = step[0, 0] * step[1, 1] - step[0, 1] * step[1, 0]
    if abs(determinant) < 1e-9:
        return np.zeros(q.shape, dtype=np.bool_)
    dq = q - middle_q
    dr = r - middle_r
    dx = (dq * step[1, 1] - dr * step[0, 1]) / determinant
    dy = (dr * step[0, 0] - dq * step[1, 0]) / determinant
    across = DIAMETER / SPAN_CELLS
    away = np.hypot(dx, dy) / across
    radius = PIP_LEAST + (PIP_MOST - PIP_LEAST) * np.sqrt(np.clip(held, 0.0, 1.0))
    inside: np.ndarray = (away < radius) & (held > 0.0) & (holder != float(NOBODY))
    return inside


def _outline(
    colour: np.ndarray,
    seen: np.ndarray,
    q: np.ndarray,
    r: np.ndarray,
    palette: Palette = SCREEN,
) -> None:
    """Draw the edge of what the window shows onto the disc.

    **A wide view with no mark of the current view is half a tool.** The
    window covers a parallelogram of tile addresses, because the map is a hex
    grid. This measures every pixel against the four sides of that
    parallelogram and paints the pixels near a side.

    The camera of the window states its own corners, so nothing here repeats
    the projection the engine owns. A window that covers the whole disc draws
    no outline, because the outline would then sit under the rim.

    The corners come from the caller, which read them from the engine, so
    this makes no call of its own.
    """
    # The three corners give the origin and the two sides of the view.
    origin = seen[0]
    across = seen[1] - origin
    down = seen[2] - origin
    detail = np.stack([q - origin[0], r - origin[1]], axis=-1)
    grid = np.array([across, down], dtype=np.float64)
    determinant = grid[0, 0] * grid[1, 1] - grid[0, 1] * grid[1, 0]
    if abs(determinant) < 1e-9:
        return
    u = (detail[..., 0] * grid[1, 1] - detail[..., 1] * grid[1, 0]) / determinant
    v = (detail[..., 1] * grid[0, 0] - detail[..., 0] * grid[0, 1]) / determinant
    # One side of the disc in the same units as u and v. A pixel within that
    # much of a side of the view is on the outline.
    step_u = abs(u[0, 1] - u[0, 0]) + abs(u[1, 0] - u[0, 0])
    step_v = abs(v[0, 1] - v[0, 0]) + abs(v[1, 0] - v[0, 0])
    side_u = (np.abs(u) < step_u * VIEW_WIDTH) | (np.abs(u - 1.0) < step_u * VIEW_WIDTH)
    side_v = (np.abs(v) < step_v * VIEW_WIDTH) | (np.abs(v - 1.0) < step_v * VIEW_WIDTH)
    near = side_u & (v > -step_v) & (v < 1.0 + step_v)
    near |= side_v & (u > -step_u) & (u < 1.0 + step_u)
    if not near.any():
        return
    for at, shift in enumerate((16, 8, 0)):
        channel = colour[..., at]
        colour[..., at] = np.where(near, _channel(palette.view, shift), channel)


def _polar(
    rows: np.ndarray, columns: np.ndarray
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    """Give back where each pixel of the disc sits, about the middle.

    The first value is the distance from the middle, where one is the rim. The
    other two are the step across and the step down from the middle, each as a
    share of the radius.

    **The rim and the weight both need this, and it is computed once.** Two
    copies of the geometry of one circle would drift, and nothing would fail
    when they did.[^1]

    References
    ----------
    Recurring Defect Shapes, shape 1.
    ``.agents/rules/recurring-defects.md``
    """
    middle = (DIAMETER - 1) / 2.0
    radius = DIAMETER / 2.0
    across = (columns - middle) / radius
    down = (rows - middle) / radius
    return np.hypot(across, down), across, down


def _bezel(colour: np.ndarray, rows: np.ndarray, columns: np.ndarray) -> None:
    """Paint the black rim of the disc over the colour, in place.

    The rim is a ring of polished black around the map. Two lights give it a
    shape: a key light from the upper left that catches its outer shoulder,
    and a weaker bounced light on the far side. A thin lit line runs round the
    outer edge, so the disc keeps a silhouette all the way round.

    The rim also throws a short shadow onto the map under its inner edge, so
    it sits above the map rather than on it.

    The colour holds red, green and blue in that order, each from zero to one.
    """
    away, across, down = _polar(rows, columns)
    inner = 1.0 - BEZEL_SHARE
    # The shadow the rim throws over the map, just inside the rim.
    shade = np.clip((away - (inner - BEZEL_SHADOW)) / BEZEL_SHADOW, 0.0, 1.0)
    colour *= (1.0 - shade * (1.0 - SHADOW_KEEP))[..., None]

    # Where a pixel sits across the rim. Zero is the inner edge and one is the
    # outer edge.
    band = np.clip((away - inner) / BEZEL_SHARE, 0.0, 1.0)
    # How much the key light reaches a pixel. The light comes from the upper
    # left, so the value is largest where the step across and the step down
    # are both negative.
    lit = np.clip(-(across + down) / 1.4142, -1.0, 1.0)
    # The outer shoulder of the rim, and the inner valley of it. A rim with
    # one bright band reads as a painted ring, and two bands read as a turned
    # edge.
    shoulder = np.exp(-(((band - 0.74) / 0.20) ** 2))
    valley = np.exp(-(((band - 0.28) / 0.24) ** 2))
    key = np.clip(lit, 0.0, 1.0) ** 3 * shoulder
    bounce = np.clip(-lit, 0.0, 1.0) ** 2 * valley
    lip = np.exp(-(((band - 0.97) / 0.10) ** 2))
    value = BEZEL_BLACK + BEZEL_KEY * key + BEZEL_BOUNCE * bounce + BEZEL_LIP * lip
    value = np.clip(value, 0.0, 1.0)
    # The key light leans blue, because a polished black surface takes the
    # colour of what it reflects.
    channels = (
        value * (1.0 - BEZEL_COOL * key),
        value * (1.0 - BEZEL_COOL * key * 0.4),
        np.clip(value + BEZEL_COOL * key, 0.0, 1.0),
    )
    on_rim = away >= inner
    for at, channel in enumerate(channels):
        colour[..., at] = np.where(on_rim, channel, colour[..., at])


def _paper_bezel(colour: np.ndarray, rows: np.ndarray, columns: np.ndarray) -> None:
    """Draw the rim of the disc as ink on paper, in place.

    **A drawn instrument has an edge and no shine.** The polished rim of the
    flat map reads as a widget over a pencil drawing, so on paper the rim is
    the paper itself between two ink rules. The outer rule is the heavier of
    the two, in the way a person inks the outside of a circle first.

    The colour holds red, green and blue in that order, each from zero to one.
    """
    away, _, _ = _polar(rows, columns)
    inner = 1.0 - BEZEL_SHARE
    band = np.clip((away - inner) / BEZEL_SHARE, 0.0, 1.0)
    paper = _flat(PAGE_PAPER)
    ink = _flat(PAPER_INK)
    # Two rules across the rim: a heavy one at the outer edge and a light one
    # at the inner edge, with paper between them.
    outer = np.exp(-(((band - 0.90) / 0.13) ** 2)) * PAPER_RIM_INK
    edge = np.exp(-(((band - 0.06) / 0.10) ** 2)) * PAPER_RIM_INNER
    weight = np.clip(outer + edge, 0.0, 1.0)[..., None]
    rim = paper * (1.0 - weight) + ink * weight
    on_rim = (away >= inner)[..., None]
    colour[...] = np.where(on_rim, rim, colour)


def _fade(rows: np.ndarray, columns: np.ndarray) -> np.ndarray:
    """Give back how much of the disc colour reaches the frame at each pixel.

    The map keeps a little of the frame below it, so the disc stays part of
    the picture. **The rim keeps none of it.** A rim that let the frame
    through would not read as black, and the border the disc needs would
    depend on what lay under it.

    The weight falls to nothing over one pixel at the outer edge, which takes
    the staircase off the circle and nothing else.
    """
    away, _, _ = _polar(rows, columns)
    inner = 1.0 - BEZEL_SHARE
    soften = BEZEL_SOFTEN / (DIAMETER / 2.0)
    edge = np.clip((1.0 - away) / soften, 0.0, 1.0)
    return np.where(away < inner, STRENGTH, edge)


def _mix(
    surface: Surface, left: int, top: int, colour: np.ndarray, kept: np.ndarray
) -> None:
    """Mix a block of colours into the frame by a weight for each pixel.

    The colour comes in already scaled by its weight, and the kept share is
    what the frame below keeps. The frame holds one byte for each of blue,
    green and red, and one byte the window reads as opacity. This reads the
    three colour bytes as bytes rather than taking them out of a packed value
    with a shift, so the mixing is whole number work over a small block.

    **This is a few array operations over a block, not a loop over pixels.**
    """
    frame = surface.pixels.view(np.uint8).reshape(surface.height, surface.width, 4)
    block = frame[top : top + DIAMETER, left : left + DIAMETER, :3]
    mixed = block * kept
    mixed += colour
    mixed >>= 8
    block[:] = mixed.astype(np.uint8)
