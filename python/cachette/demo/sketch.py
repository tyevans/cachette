"""A sketchbook renderer: the world as a pencil study in ink on paper.

The demonstration fills one frame through one call. This module offers a
second renderer at that same call, so a flag chooses a renderer and the clock,
the panels, the keys and the window memory stay shared.[^1]

**This renderer reads the world. It never writes to it.**[^2] It reads the
fields the engine publishes in bulk, one array for each field, and it draws
them. It touches no simulated value, no state hash and no step.

What the sketch shows
---------------------

The picture is a view of the ground from above and to one side. The page turns
the world about the up direction, leans it away from the watcher, and lifts
every tile by the height of the ground under it. A tile that stands high moves
up the page, and the face below it becomes a cliff. The flat view cannot show
this, and the world holds real terrain.

**The turn and the lean come from the view, and a mouse drives them.** The page
opens at an eighth of a circle and a lean of one half, which is an isometric
drawing. A drag with the right button or the middle button moves both.

What the hatching carries
-------------------------

**Each set of marks carries one quantity. No set is decoration.**

The contour hatch follows a line of constant height. The lines are evenly
spaced in height, so they crowd where the ground is steep and they open where
it is flat. The density of this set is the slope.

The shadow hatch runs at a fixed angle to the page. It appears where the
ground turns away from the light, and its weight grows as the ground turns
further. The weight of this set is the aspect against the light.

The cross hatch runs at the opposite angle, in the deepest shade alone. Its
presence is the depth of the shadow.

The cliff hatch runs down the page, on the faces the lift exposes. Its
presence names a vertical face.

The water hatch runs across the page. Its weight grows with the water. The
weight of this set is the depth.

The cloud is not a set of marks. It is a mass, and it stands where a field
of noise rises over a mark that the cover sets. A cover of a third therefore
paints cloud over a third of the sky and open sky beside it, which is what a
broken sky is. The mass drifts along the wind and a second, slower field
bends it as it goes, so a watcher reads the circulation from the shape and
the motion rather than from a stripe.

The deep of the sky is the top of the cover range. A sky that stands there
darkens, its cloud closes over, and the page turns from the ink of a fair sky
to the ink of a deep one.

What costs what
---------------

**The ground is built once for each view.** The terrain never moves, so the
page that carries it is a function of the window of tiles the camera covers,
the two angles the view stands at, and the size of the frame. The pass that
turns the world, lifts it and hatches it therefore runs when the view moves,
and a frame that only steps the world reuses it.

**A drag moves the view on every frame, so a drag pays that pass on every
frame.** The cost is the reason the console warns that this renderer draws
slowly. It is a property of drawing each pixel on the processor.

**The layers that change are read and drawn every frame.** The cloud, the wind
and the faction that holds each tile change with the world, so they cross the
boundary on every frame and they are drawn over the ground.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0067, the viewer reads the world and never writes to it, decision D3.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING, Any

import numpy as np

from cachette.demo.view import TINY, View

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

    import numpy.typing as npt

    from cachette import World
    from cachette._core import Camera, FrameReading

# The number the engine gives the kind of a tile that holds open water.
#
# **The engine numbers the kinds and this module reads that numbering.** A
# test pins the number against a property of water that no renumbering
# survives: water lies lower than every other kind.
WATER_KIND = 0

# The scale of a fixed point value the engine reports.
#
# The engine reports a height as a raw Q16.16 value, so a whole unit of height
# is this number. The page divides by it once and works in shares of one.
FIXED_POINT = 65536.0

# The paper, the ink, the ink of a fair sky and the ink of a deep one, as
# red, green and blue.
#
# A fair sky is lighter than the land on purpose. In a pencil study a fair
# cloud is among the lightest marks on the page. A deep sky is not: the
# heaviest ink on the page is the ink of the weather, because that is what a
# watcher sees when the weather turns.
PAPER = np.array([0xF2, 0xEC, 0xDD], dtype=np.float32)
INK = np.array([0x25, 0x23, 0x20], dtype=np.float32)
SKY_INK = np.array([0x5A, 0x62, 0x6E], dtype=np.float32)
STORM_INK = np.array([0x2B, 0x2E, 0x3A], dtype=np.float32)

# Where the light stands. It comes from the upper left, which is where a right
# handed person holding a pencil puts it.
LIGHT = np.array([-0.55, -0.62, 0.56], dtype=np.float32)

# How far the page turns the ground, and how far it leans away from the
# watcher, live on the view. **This module holds no copy of either.** They are
# what a person drives with a mouse, so a second copy here would be read back
# correctly and would show a page that stood somewhere else.

# How far the tallest ground rises, as a share of the width of the page.
RELIEF = 0.115

# The height of one row of a hex grid against the width of one column.
#
# A pointy top hexagon is taller than it is wide by this ratio, and the rows
# interlock, so a row stands this far below the row behind it.
ROW_PITCH = 0.8660254

# How many spare rows the page keeps above the lift and below the foot.
#
# The lift moves a point up the page by the height of the ground under it, and
# the face below it reaches down to where the point stood. The page therefore
# needs room at both ends, and this is the room it keeps.
LIFT_MARGIN = 4

# How much finer than the frame the page may be drawn.
#
# The page holds the whole world, and the camera magnifies it. A page drawn at
# the size of the frame goes soft as soon as a watcher zooms in, so the page is
# drawn finer as the camera zooms.
#
# **A page costs the square of this, in the memory of the graphics device.**
# The page carries four real numbers for each of its points, so a frame of
# 2256 by 1504 asks the device for about 200 megabytes at two and about 800 at
# four. A machine that shares its memory with the display cannot hold the
# second. The page therefore stops here and a closer view softens.
PAGE_DETAIL_CAP = 2

# The spacing of the contour hatch, in shares of the full height range.
#
# **The lines are evenly spaced in height, not on the page.** That is what
# makes their density on the page the slope of the ground.
CONTOUR_STEP = 0.055

# How thick a contour line is, as a share of the spacing between two of them.
CONTOUR_WEIGHT = 0.16

# The spacing of the hatch sets that belong to the page, in pixels.
#
# **These spacings belong to the page and not to the ground.** A pencil study
# keeps the weight of its marks while the subject moves, because the marks are
# on the paper.
HATCH_SPACING = 6.0
CLIFF_SPACING = 3.0
WATER_SPACING = 7.0

# The tone at which the shadow hatch starts, and at which the cross hatch
# starts. A tone runs from zero in full light to one in full shade.
SHADOW_START = 0.34
CROSS_START = 0.66

# The share of the width of the page that the cloud shadow moves.
CLOUD_SHADOW_STEP = 0.030

# The share of the sky below which the page draws no cloud, how far the cloud
# layer stands over the tallest ground as a share of the rise of the ground,
# and how far a cloud darkens the ground it covers.
#
# The cloud sits above the tallest ground on purpose. A mass that crossed the
# paper behind a mountain would say nothing about the height of the mountain.
CLOUD_FLOOR = 0.18
CLOUD_HEIGHT = 1.6
CLOUD_SHADOW_DEPTH = 0.30

# How far the mean of the cover reaches, as a share of the pitch of the
# weather lattice.
#
# **The weather stands on a lattice coarser than the tiles.** Every tile of
# one cell reports one cover, so the raw field is a staircase with a step at
# each cell edge. A cloud drawn from the raw field carries that staircase, and
# a watcher reads a straight edge that no weather has. The engine publishes
# the pitch, so this is a share of that pitch and never a count of tiles.[^1]
#
# **The mean runs over a box, and this is half its width.** A half therefore
# makes the box exactly one cell wide, which is the width that turns a
# staircase of that pitch into a slope with no step left in it. A wider box
# takes the tops off the field as well, and the deep of the sky is the top of
# the field.
#
# [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
SKY_SMOOTHING = 0.5

# The lattice of values that the cloud noise reads, and the number it draws
# from.
#
# **The lattice comes from an integer hash and never from a sine.** A hash
# built on a sine gives one answer on the processor and another on the
# graphics device, and the two renderers then draw two pictures. An integer
# hash gives one answer everywhere. The array renderer builds the lattice and
# the device renderer is given it, so one table serves both.
#
# The side is a power of two, so a cell number folds into the table with one
# mask. The cloud repeats over this many cells, which is far wider than a
# page.
CLOUD_TABLE = 256
CLOUD_SEED = 0x1B2F_A5C7

# How wide one cell of the coarsest cloud octave is, in page points, how many
# octaves the cloud carries, how much each finer octave weighs, and how far
# each octave is shifted from the one before it.
#
# **The octaves are what makes a mass read as a mass.** One octave gives a
# blob. Three give a body with a ragged edge and detail inside it. The shift
# keeps the octaves from lining up at the corner of the page.
CLOUD_GRAIN = 44.0
CLOUD_OCTAVES = 3
CLOUD_OCTAVE_FALL = 0.52
CLOUD_OCTAVE_SHIFT = 19.0

# How far the swirl moves a reading, in cells of the coarsest octave, how wide
# one cell of the swirl field is against a cloud cell, and how far apart the
# two readings of that field are taken.
#
# **The swirl is what makes the sky read as weather rather than as wool.** The
# cloud field is read at a place that a second, coarser field moves. A mass
# therefore bends, curls and tears along a line the second field draws, and
# that is the shape a front has.
CLOUD_SWIRL = 1.45
CLOUD_SWIRL_GRAIN = 4.0
CLOUD_SWIRL_APART = 31.0

# How far the cloud moves along the wind in one tick, in page points, and how
# much slower the swirl field moves than the cloud.
#
# **The motion belongs to the renderer.** The sky over a place holds one
# narrow band and it does not leave it, so a picture that waited for the
# engine to move the weather would never move.[^1] The cover says how much
# cloud stands over a place. The drift says where the masses that make up
# that cover are at this moment.
#
# The two rates differ on purpose. The swirl field crosses the page more
# slowly than the cloud does, so a mass runs through the swirl and deforms as
# it goes. Two fields at one rate would slide together and nothing would
# change shape.
#
# [^1]: Findings register, FND-715. `docs/FINDINGS.md`
CLOUD_DRIFT = 2.4
CLOUD_GUST = 0.35

# The largest tick the cloud drifts on.
#
# The drift is a distance in page points, and it grows with the tick. A real
# number holds a large distance coarsely, so the tick folds into this range
# and the picture keeps its detail however long a world runs. The range is a
# power of two, so the fold is exact.
CLOUD_CLOCK_MASK = 0xF_FFFF

# The two marks that the cover puts the cloud field against, how soft the edge
# of a mass is, and the cover over which a cloud opens from nothing.
#
# **A cover share and an opacity are two quantities, and the engine carries
# one.** A broken sky is one part of the sky at full opacity beside a part at
# none. The cover therefore sets a mark, and the cloud stands where the field
# is above that mark. A cover of a third then paints cloud over a third of the
# sky, and not thin cloud everywhere.[^1]
#
# The two marks span the range that the field reaches in practice, so a low
# cover leaves the sky open and a high cover closes it.
#
# [^1]: Findings register, FND-715. `docs/FINDINGS.md`
CLOUD_MARK_HIGH = 0.86
CLOUD_MARK_LOW = 0.13
CLOUD_EDGE = 0.11
CLOUD_OPEN = 0.10

# How far the cloud reads toward the light, in cells of the coarsest octave,
# how hard that reading turns into a face, and how dark the two faces of a
# mass draw.
#
# **A mass has a lit side and a shaded side, and that is what gives it
# body.** The page reads the coarsest octave at the point and again a short
# step toward the light. A mass that falls away toward the light is turned
# into the light, and it draws pale. A mass that rises toward the light stands
# in its own shade, and it draws heavy.
#
# The light is where the light of the ground is. This module holds one light
# and the sky reads it, so the cloud and the hill cannot be lit from two
# places.
CLOUD_LIGHT_STEP = 0.55
CLOUD_RELIEF = 3.2
CLOUD_FACE_LIT = 0.20
CLOUD_FACE_DARK = 0.74

# The cover at which the sky begins to go deep, and how far a deep sky darkens
# the whole of itself.
#
# **A deep sky is the top of the cover range, and it is measured.** Over four
# hundred ticks, 522 cells of 9216 stood overcast at every tick and the set of
# overcast cells turned over.[^1] The deep of the sky is therefore a real
# thing that moves, and not a mark this module invented.
#
# **The engine holds a storm as an object, and this is not that object.** A
# storm puts a pressure deficit on the cells it reaches. The engine holds that
# deficit and the Python boundary does not publish it for each tile, so this
# module cannot read it. When it does, the deep of the sky reads the deficit
# and this mark goes.
#
# [^1]: Findings register, FND-715. `docs/FINDINGS.md`
SKY_DEEP_MARK = 0.84
SKY_GLOOM = 0.46

# How far the height of the ground is smoothed before it is lifted, in tiles.
#
# One tile of the world is many pixels of the page, so a page that lifted the
# raw field would draw a staircase of tile edges rather than a hill. The
# smoothing is over the tiles, not over the page, so it costs the tile count.
SMOOTHING = 2

# How much of the colour of a faction survives into the wash of its ground.
#
# **The sketch stays close to monochrome.** The colour says who holds the
# ground and nothing else, so the ink carries the drawing.
HOLDER_WASH = 0.30

# How the wash of an overlay behaves as paint.
#
# The gain turns the strength the overlay painted into the strength of the
# wash. The granulation is how much the grain of the paper carries the
# pigment. The rim gain deepens the colour where a wash ends, which is what a
# wash of water colour does as it dries. The lift keeps the glaze off black,
# because a glaze passes light rather than blocking it.
WASH_GAIN = 2.4
GRANULATION = 0.45
RIM_GAIN = 1.6
GLAZE_LIFT = 1.25

# The number the grain of the paper draws from.
#
# **The grain belongs to the paper, not to the world.** One number here gives
# every run the same sheet, so two pictures of two worlds differ in the world
# alone.
GRAIN_SEED = 0x5C4E_7C48

# How wide a road draws, as a share of the width of a tile, by level.
#
# **A road is a way and not a tile.** It runs from somewhere to somewhere, it
# joins another road at a junction, it bends, and it ends. The ribbon runs
# through the middle of a tile and out to the middle of each edge it shares
# with a road beside it, so the two halves of a join meet exactly.
#
# The first entry is a tile that carries no road. The second is a way that is
# still being made. The third and the fourth are the two levels that stand.
# **The width carries the level**, so a watcher tells a better road from a
# poorer one without reading a legend.
WAY_WIDTH = (0.0, 0.14, 0.28, 0.44)

# The level at and above which a road draws a line down its middle.
#
# **The crown is the second channel of the level.** A width reads against a
# neighbour, and a road with no neighbour to compare against would say
# nothing. A line down the middle needs no comparison.
CROWNED_LEVEL = 3

# How heavy a road mark is, in page points.
#
# **The mark belongs to the page and not to the ground**, in the same way that
# the hatch does. A pencil study keeps the weight of its marks while the
# subject moves.
WAY_INK = 2.2

# How dark the two edges of a road are, how dark the ground between them is,
# and how dark the line down the middle of the best road is.
#
# **The edges carry the way and the ground between them carries the surface.**
# Two lines on their own read as a long thin loop when a watcher looks
# closely. A light tone between them reads as a made surface, and the edges
# still lead the eye along it.
#
# The middle is lighter than the edges, because the edges are what a watcher
# follows and the middle only says which road it is.
WAY_EDGE_INK = 0.92
WAY_FILL_INK = 0.20
WAY_CROWN_INK = 0.55

# How dark a road that is still being made is.
#
# A way under work draws one light line down the middle and no edges, so a
# watcher reads marked-out ground rather than a road that carries traffic.
WAY_PLANNED_INK = 0.35

# How much bare paper stays around the ground, in pixels.
MARGIN = 24

# How far the mask of the panels grows before the page goes under it, in
# pixels. A letter of a panel that matched the map under it would speckle the
# card without this.
PANEL_GROWTH = 2


class BoundaryGap(Exception):
    """The engine does not publish something this renderer needs."""


def _lines(phase: np.ndarray, spacing: float, weight: np.ndarray) -> np.ndarray:
    """Give back how much of each pixel one set of ruled lines covers.

    The phase says where a point falls between two lines. The weight says how
    thick a line is, from nothing to the full spacing.
    """
    across = np.abs(((phase / spacing) % 1.0) - 0.5) * 2.0
    ruled: np.ndarray = np.clip(
        (weight - across) / np.clip(weight, 1e-3, None), 0.0, 1.0
    ).astype(np.float32)
    return ruled


def _slid(field: np.ndarray, step: int, axis: int) -> np.ndarray:
    """Move a field along an axis, and leave nothing where it came from.

    **The world does not wrap.** A neighbour outside the world is absent, and
    the edge of the world is an edge.[^1] A field rolled off one side of the
    page and back onto the other therefore says that a place is next to a
    place it is not next to. The sky did that, and a watcher read a hatch and
    a shadow shaped like ground that stood on the far side of the map.[^2]

    A positive step moves the field towards the higher index.

    [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial,
    decision D2.
    ``adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md``

    [^2]: Findings register, FND-627. ``docs/FINDINGS.md``
    """
    moved = np.zeros_like(field)
    reach = abs(step)
    if reach >= field.shape[axis]:
        return moved
    if step == 0:
        return field.copy()
    if axis == 1:
        if step > 0:
            moved[:, step:] = field[:, :-step]
        else:
            moved[:, :-reach] = field[:, reach:]
        return moved
    if step > 0:
        moved[step:, :] = field[:-step, :]
    else:
        moved[:-reach, :] = field[reach:, :]
    return moved


def _smooth(field: np.ndarray, radius: int) -> np.ndarray:
    """Give back the mean of each point and the points around it.

    The mean runs across and then down, and each pass sums over a running
    total, so the cost follows the number of points and not the square of the
    radius.
    """
    if radius <= 0:
        return field.astype(np.float32)
    wide = np.pad(field.astype(np.float32), radius, mode="edge")
    for axis in (0, 1):
        total = np.cumsum(wide, axis=axis)
        lead = np.take(total, np.arange(2 * radius, wide.shape[axis]), axis=axis)
        trail = np.take(total, np.arange(0, wide.shape[axis] - 2 * radius), axis=axis)
        wide = (lead - trail) / (2.0 * radius)
    return wide[: field.shape[0], : field.shape[1]].astype(np.float32)


def _grain(width: int, height: int, seed: int) -> np.ndarray:
    """Give back the grain of the paper, as a field of light and dark.

    The grain is presentation and not simulation. It draws from a generator
    this module seeds, so one seed gives one page.
    """
    engine = np.random.default_rng(seed & 0xFFFF_FFFF)
    coarse = engine.random((height // 3 + 2, width // 3 + 2), dtype=np.float32)
    grown = np.repeat(np.repeat(coarse, 3, axis=0), 3, axis=1)
    return grown[:height, :width]


def _cloud_table(seed: int) -> np.ndarray:
    """Give back the lattice of values that the cloud noise reads.

    The answer is a square of real numbers from nothing to one, one for each
    corner of the cloud lattice. The side is ``CLOUD_TABLE``.

    **The value comes from an integer hash of the two cell numbers.** A hash
    built on a sine gives one answer on the processor and another on the
    graphics device, because the two round a sine differently. The two
    renderers would then draw two skies, and only the frame comparison would
    notice. Integer operations give one answer on every machine.

    **One table serves both renderers.** The array renderer builds it here and
    the device renderer uploads this same table, so the two read one set of
    numbers rather than each hashing its own.[^1]

    The value keeps 24 bits, which a real number holds exactly, so the table
    is the same to the last bit on both sides.

    [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    """
    side = np.arange(CLOUD_TABLE, dtype=np.uint32)
    cell_y, cell_x = np.meshgrid(side, side, indexing="ij")
    mixed = (
        cell_x * np.uint32(0x27D4_EB2D)
        + cell_y * np.uint32(0x9E37_79B1)
        + np.uint32(seed & 0xFFFF_FFFF)
    )
    mixed ^= mixed >> np.uint32(15)
    mixed *= np.uint32(0x85EB_CA6B)
    mixed ^= mixed >> np.uint32(13)
    mixed *= np.uint32(0xC2B2_AE35)
    mixed ^= mixed >> np.uint32(16)
    kept: np.ndarray = (mixed >> np.uint32(8)).astype(np.float32) / 16777216.0
    return kept


def _cloud_noise(table: np.ndarray, at_x: np.ndarray, at_y: np.ndarray) -> np.ndarray:
    """Give back the value of the cloud field at a place on its lattice.

    The place is given in cells of the lattice. The answer runs from nothing
    to one, and it is smooth across a cell edge, so a last bit of difference
    in the place gives a last bit of difference in the answer.

    The cell number folds into the table with a mask, so the field repeats
    over ``CLOUD_TABLE`` cells and a page never reaches that far.
    """
    fold = np.int32(CLOUD_TABLE - 1)
    base_x = np.floor(at_x)
    base_y = np.floor(at_y)
    part_x = at_x - base_x
    part_y = at_y - base_y
    ease_x = part_x * part_x * (3.0 - 2.0 * part_x)
    ease_y = part_y * part_y * (3.0 - 2.0 * part_y)
    cell_x = base_x.astype(np.int32) & fold
    cell_y = base_y.astype(np.int32) & fold
    next_x = (cell_x + 1) & fold
    next_y = (cell_y + 1) & fold
    # The table is read as one row of numbers. A read of two axes costs about
    # four times a read of one, and the two give the same value.
    flat = table.reshape(-1)
    row = cell_y * np.int32(CLOUD_TABLE)
    row_next = next_y * np.int32(CLOUD_TABLE)
    here = np.take(flat, row + cell_x)
    right = np.take(flat, row + next_x)
    under = np.take(flat, row_next + cell_x)
    across = np.take(flat, row_next + next_x)
    near = here + (right - here) * ease_x
    far = under + (across - under) * ease_x
    mixed: np.ndarray = near + (far - near) * ease_y
    return mixed


def _cloud_cover(share: np.ndarray) -> np.ndarray:
    """Say how much of the sky closes, from nothing to all of it.

    The share is how much of the sky the engine reports as cloud. The floor is
    the share below which the page draws no cloud at all, and the answer opens
    from nothing at that floor.

    **One place holds this ramp.** The mark and the mass both read the cover,
    and a second ramp would be one rule in two places.[^1]

    [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    """
    opened: np.ndarray = np.clip((share - CLOUD_FLOOR) / (1.0 - CLOUD_FLOOR), 0.0, 1.0)
    return opened


def _cloud_mark(cover: np.ndarray) -> np.ndarray:
    """Say what the cloud field must reach for a cloud to stand here.

    The answer runs from the high mark at a clear sky to the low mark at a
    closed one. The two marks span the range that the field reaches, so a low
    cover leaves the sky open and a high cover closes it.
    """
    marked: np.ndarray = CLOUD_MARK_HIGH + (CLOUD_MARK_LOW - CLOUD_MARK_HIGH) * cover
    return marked


def cloud_mass_of(density: np.ndarray, share: np.ndarray) -> np.ndarray:
    """Say how much cloud stands at a place, from none to all of it.

    The density is the value of the cloud field. The share is how much of the
    sky the engine reports as cloud.

    **The cover sets a mark and the mass stands over it.** A cover share and
    an opacity are two quantities, and the engine carries one. A renderer that
    turned the share into a weight painted thin cloud everywhere, and a broken
    sky cannot be drawn that way.[^1] A cover of a third therefore paints
    cloud over a third of the sky and open sky beside it.

    The cover also opens the cloud from nothing over the first part of its
    range, so a sky the engine calls clear draws no wisp.

    **Both renderers hold this rule and the device renderer repeats it in the
    shader.** A test compares the frames the two draw.[^2]

    [^1]: Findings register, FND-715. `docs/FINDINGS.md`
    [^2]: The two renderers, and the bound they agree within.
    `tests/test_demo_sketch_gl.py`
    """
    cover = _cloud_cover(share)
    raw = np.clip((density - _cloud_mark(cover)) / CLOUD_EDGE + 1.0, 0.0, 1.0)
    eased = raw * raw * (3.0 - 2.0 * raw)
    massed: np.ndarray = eased * np.clip(cover / CLOUD_OPEN, 0.0, 1.0)
    return massed


def _toward_light() -> tuple[float, float]:
    """Give back the direction of the light on the page, of length one.

    **This module holds one light and every layer reads it.** A cloud lit from
    one place and a hill lit from another would read as two drawings. The
    shader normalises the same two bands of the same light, so the two
    renderers hold no second copy of the direction.[^1]

    [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
    """
    across = float(LIGHT[0])
    down = float(LIGHT[1])
    length = math.hypot(across, down)
    return (across / length, down / length)


def _grown(mask: np.ndarray, radius: int) -> np.ndarray:
    """Grow a mask by a short reach, so that a hole inside it closes."""
    wide = mask.copy()
    for step in range(1, radius + 1):
        wide[:, step:] |= mask[:, :-step]
        wide[:, :-step] |= mask[:, step:]
        wide[step:, :] |= mask[:-step, :]
        wide[:-step, :] |= mask[step:, :]
    return wide


def _edge_below(
    holds: Callable[[float], bool],
    place: float,
    pitch: float,
    halvings: int = 24,
) -> float:
    """Say how far a place stands past the edge below it, in shares of a pitch.

    The caller gives a test that answers whether a place still lies in the
    same tile as the place it asks about. The edge below stands within one
    pitch, and halving the range finds it.

    **This asks the engine where the edge is.** A module that worked the edge
    out from the size of a tile would hold a second copy of a layout the
    engine owns, and the two would part company without anything failing.
    """
    low, high = place - pitch, place
    for _ in range(halvings):
        middle = (low + high) / 2.0
        if holds(middle):
            high = middle
        else:
            low = middle
    return min(max((place - high) / max(pitch, TINY), 0.0), 1.0)


class Projection:
    """Where each tile of the window falls on the page, before the lift.

    **One object holds the projection, and both renderers read it.** The array
    renderer reads the page backwards, one tile for each point. The device
    renderer draws the tiles forwards, as a mesh. A projection worked out
    twice would be one value declared twice, and the two would part company
    without anything failing.

    The turn is how far the page rotates the ground about the up direction.
    The lean is how far the page tips away from the watcher. A world seen
    square to the page reads as a flat map however far the ground is lifted,
    so the page turns before it lifts.
    """

    __slots__ = (
        "cols",
        "flat_rows",
        "lean",
        "plan_tall",
        "plan_wide",
        "rise",
        "rows",
        "scale",
        "turn",
        "window",
    )

    # The window of tiles, the two angles, and the numbers that follow from
    # them: how far the plan of the window reaches, how many page points one
    # tile covers, and how large the page is before and after the lift.
    window: tuple[int, int, int, int]
    turn: float
    lean: float
    plan_wide: float
    plan_tall: float
    scale: float
    cols: int
    flat_rows: int
    rise: int
    rows: int

    def __init__(
        self,
        window: tuple[int, int, int, int],
        width: int,
        height: int,
        turn: float,
        lean: float,
        relief: float,
    ) -> None:
        """Work out the page this window, this frame and these angles give."""
        first_q, first_r, last_q, last_r = window
        across = last_q - first_q
        down = last_r - first_r
        self.window = window
        self.turn = turn
        self.lean = lean
        # The window in the plan of the world, before the turn. A row of a hex
        # grid steps half a column across and less than a row down.
        self.plan_wide = across + down * 0.5
        self.plan_tall = down * ROW_PITCH
        turn_x, turn_y = math.cos(turn), math.sin(turn)
        # How far the turned plan reaches across the page and down it. A
        # rectangle turned through an angle covers this much of each
        # direction, and the page is cut to fit it.
        span_x = self.plan_wide * abs(turn_x) + self.plan_tall * abs(turn_y)
        span_y = self.plan_wide * abs(turn_y) + self.plan_tall * abs(turn_x)
        self.scale = min(
            width / max(span_x, 1e-3),
            height / max(span_y * lean, 1e-3),
        )
        self.cols = max(int(span_x * self.scale) + 2, 2)
        self.flat_rows = max(int(span_y * self.scale * lean) + 2, 2)
        # How far the tallest ground rises, in page points, and how tall the
        # page is once the lift has moved every point up by its own height.
        self.rise = max(int(self.cols * relief), 1)
        self.rows = self.flat_rows + self.rise + LIFT_MARGIN * 2

    def to_page(
        self, tile_q: np.ndarray, tile_r: np.ndarray
    ) -> tuple[np.ndarray, np.ndarray]:
        """Say where a place in the grid falls on the page, before the lift.

        The place is given in tiles of the world, and it may lie between two
        tiles. The answer is the column of the page and the row of the flat
        page, both as real numbers.
        """
        first_q, first_r = self.window[0], self.window[1]
        rel_q = tile_q - first_q
        rel_r = tile_r - first_r
        plan_x = rel_q + rel_r * 0.5
        plan_y = rel_r * ROW_PITCH
        turn_x, turn_y = math.cos(self.turn), math.sin(self.turn)
        across = (plan_x - self.plan_wide / 2.0) * self.scale
        down = (plan_y - self.plan_tall / 2.0) * self.scale
        column = across * turn_x - down * turn_y + self.cols / 2.0
        row = (across * turn_y + down * turn_x) * self.lean + self.flat_rows / 2.0
        return column, row


class Ground:
    """The page the terrain draws on, for one camera and one frame size.

    **The terrain never moves.** The turn, the lift and the hatch of the
    ground are therefore a function of the window of tiles the camera covers
    and of the size of the frame. This holds the result, so a frame that only
    steps the world draws the ground again without building it again.
    """

    __slots__ = (
        "box",
        "cliff",
        "coverage",
        "depth",
        "drawn",
        "held",
        "held_height",
        "outline",
        "page_x",
        "page_y",
        "rise",
        "shape",
        "sheet",
        "stand",
        "take",
        "tone",
        "water",
        "window",
        "within_q",
        "within_r",
    )

    # The window of tiles this page covers, the size of the frame it was built
    # for, and the two angles the view stood at. The rest is what the build
    # put on the page.
    window: tuple[int, int, int, int]
    shape: tuple[int, int]
    stand: tuple[float, float, float]
    held: np.ndarray
    take: np.ndarray
    drawn: np.ndarray
    cliff: np.ndarray
    water: np.ndarray
    tone: np.ndarray
    coverage: np.ndarray
    outline: np.ndarray
    sheet: np.ndarray
    box: tuple[int, int, int, int]
    page_x: np.ndarray
    page_y: np.ndarray
    # The height of the ground under each point of the lifted page, and how
    # deep the water is there. Every set of marks is a function of these two
    # and of the place on the page, so a renderer that draws its own marks
    # needs these and needs nothing else the build made.
    held_height: np.ndarray
    depth: np.ndarray
    rise: int
    # Where inside its own tile each point of the page stands, in tiles, on
    # each axis of the grid. Both run from minus one half to one half. A mark
    # that runs through a tile rather than filling it needs this, and nothing
    # else the build makes carries it.
    within_q: np.ndarray
    within_r: np.ndarray

    def __init__(
        self,
        window: tuple[int, int, int, int],
        shape: tuple[int, int],
        stand: tuple[float, float, float],
    ) -> None:
        """Record which window, which size and which angles built this page."""
        self.window = window
        self.shape = shape
        self.stand = stand


class Sketch:
    """A renderer that draws the world as a pencil study.

    A caller builds one of these from the world, then calls it exactly as it
    calls the drawing method of the world. The call fills the pixels it is
    given and gives back what the drawing pass read.
    """

    __slots__ = (
        "_cloud",
        "_deep",
        "_grain",
        "_grain_size",
        "_ground",
        "_heights",
        "_kinds",
        "_level",
        "_raised",
        "_relief",
        "_scratch",
        "_sky",
        "_water",
        "_whole_sky",
        "_world",
        "view",
    )

    # Whether the build draws the marks of the ground on the page.
    #
    # **This renderer draws them, and it is the reference for one that does
    # not.** A renderer that composites every pixel elsewhere sets this to
    # false, takes the page the build made, and draws the same marks there.
    # A test holds the two together by comparing the frames they draw.
    _marks = True

    def __init__(
        self,
        world: World,
        *,
        view: View | None = None,
        relief: float = RELIEF,
        sky: bool = True,
    ) -> None:
        """Build the renderer for one world.

        The relief is how far the tallest ground rises, as a share of the
        width of the page.

        **The view says where the watcher stands, and this module holds no
        angle of its own.** The turn and the lean are read from it on every
        frame, so a mouse that moves the view moves the page. A caller that
        names no view gets one at the opening angles.

        **The terrain crosses the boundary once.** The engine generates the
        ground from the seed and never changes it, so the height and the kind
        of every tile are read here and never read again.

        Raises ``BoundaryGap`` when the engine publishes no bulk reader for
        the fields the page draws.
        """
        for name in (
            "tile_heights",
            "tile_kinds",
            "cloud_shares",
            "tile_winds",
            "weather_cell_tiles",
            "tick",
            "overlay_paint",
            "road_ways",
        ):
            if not hasattr(world, name):
                message = (
                    f"the engine publishes no {name!r} reader, and the sketch "
                    f"renderer reads the world through it"
                )
                raise BoundaryGap(message)
        self._world = world
        self._relief = relief
        self.view = view if view is not None else View()
        self._sky = sky
        rows, columns = world.height, world.width
        self._heights = (
            world.tile_heights().reshape(rows, columns).astype(np.float32) / FIXED_POINT
        )
        self._kinds = world.tile_kinds().reshape(rows, columns)
        self._water = self._kinds == WATER_KIND
        self._whole_sky = float(world.cloud_share_whole)
        # **The water stands at one level, and the land starts there.** The
        # engine reports the height of the ground over the whole range it
        # generates, and the water covers the lower part of that range. A page
        # that put the water at zero would draw a cliff along every coast.
        #
        # The level is read once, from the whole world, so it cannot move when
        # the camera moves.
        under = self._heights[self._water]
        self._level = float(under.max()) if under.size else 0.0
        # **The lifted height of the ground crosses the boundary once.** The
        # land starts at the level of the water and the water lies flat, so
        # the field below is a function of the terrain alone. The device
        # renderer uploads it once as vertex data, and a field smoothed over a
        # window would hold a different value at the edge of every window.
        land = np.clip(self._heights - self._level, 0.0, None) / max(
            1.0 - self._level, 1e-3
        )
        raised = _smooth(np.where(self._water, 0.0, land), SMOOTHING)
        self._raised = np.where(self._water, 0.0, raised).astype(np.float32)
        self._deep = np.clip(
            1.0 - self._heights / max(self._level, 1e-3), 0.0, 1.0
        ).astype(np.float32)
        self._ground: Ground | None = None
        self._scratch: dict[str, npt.NDArray[np.uint32]] = {}
        self._grain: np.ndarray | None = None
        self._grain_size = (0, 0)
        self._cloud: np.ndarray | None = None

    def window(self) -> tuple[int, int, int, int] | None:
        """Give back the window of tiles the last frame drew, or nothing.

        A caller reads this to learn which part of the world the page covers.
        It answers nothing before the first frame.
        """
        return None if self._ground is None else self._ground.window

    def _buffer(self, name: str, width: int, height: int) -> npt.NDArray[np.uint32]:
        """Give back a buffer the size of the frame, built once for each size."""
        held = self._scratch.get(name)
        if held is None or held.size != width * height:
            held = np.zeros(width * height, dtype=np.uint32)
            self._scratch[name] = held
        return held

    def __call__(
        self,
        camera: Camera,
        width: int,
        height: int,
        pixels: npt.NDArray[np.uint32],
        reference: bool = False,
        panel: bool = False,
        panels: Sequence[str] | None = None,
        pointer: tuple[int, int] | None = None,
        overlay: str | None = None,
        phase: float = 0.0,
        speed_milli: int = 0,
    ) -> FrameReading:
        """Fill one frame with the sketch, and report what the pass read.

        The arguments are the arguments of the drawing method of the world, so
        a caller swaps one renderer for the other and changes nothing else.

        **The panels stay the engine's.** The pass draws the frame the window
        would have drawn, keeps the pixels that the panels wrote, and puts the
        sketch under them.
        """
        reading = self._world.draw(
            camera,
            width,
            height,
            pixels,
            reference=reference,
            panel=panel,
            panels=panels,
            pointer=pointer,
            overlay=overlay,
            phase=phase,
            speed_milli=speed_milli,
        )
        ground = self._for(camera, width, height)
        page = self._draw(ground, overlay, camera, width, height, phase)
        kept = self._panels(pixels, camera, width, height, overlay, phase)
        frame = pixels.reshape(height, width)
        stood = self.projection(ground.window, width, height, int(ground.stand[2]))
        frame[...] = np.where(
            kept, frame, self._fit(page, ground.box, stood, camera, width, height)
        )
        return reading

    def _panels(
        self,
        pixels: npt.NDArray[np.uint32],
        camera: Camera,
        width: int,
        height: int,
        overlay: str | None,
        phase: float,
    ) -> np.ndarray:
        """Give back the pixels a panel, a card or the clock painted.

        The frame with the panels and the frame without them differ where a
        panel painted, and nowhere else. A pixel of a panel that happens to
        match the map under it would speckle the card, so the mask grows over
        a short reach before it is used.
        """
        plain = self._buffer("plain", width, height)
        self._world.draw(camera, width, height, plain, overlay=overlay, phase=phase)
        differs = pixels.reshape(height, width) != plain.reshape(height, width)
        return _grown(differs, PANEL_GROWTH)

    def _window(
        self, camera: Camera, width: int, height: int
    ) -> tuple[int, int, int, int]:
        """Give back the tiles the page covers: every tile of the world.

        **The page holds the whole world, and the camera does not choose
        it.** A page cut to the tiles the camera covers made the camera pick
        an extent rather than a view. Panning towards the edge of the world
        then widened the extent, and the page drew the world smaller instead
        of moving it, so a drag with the left button did the opposite of what
        a hand expects.

        The camera now moves the view of the page, and the extent is the
        world.
        """
        del camera, width, height
        return 0, 0, self._world.width, self._world.height

    def detail_of(self, camera: Camera, width: int, height: int) -> int:
        """Say how much finer than the frame the page is drawn.

        The page holds the whole world and the camera magnifies it, so a page
        drawn at the size of the frame goes soft as soon as a watcher zooms
        in. The page is therefore drawn finer as the camera zooms, and the
        cost of a page is the square of this, so it stops at a bound.
        """
        plain = self.projection(self._window(camera, width, height), width, height)
        zoom = float(camera.tile_width) / max(plain.scale, TINY)
        return int(min(max(round(zoom), 1), PAGE_DETAIL_CAP))

    def page_middle(
        self, stood: Projection, camera: Camera, width: int, height: int
    ) -> tuple[float, float]:
        """Say which point of the page the middle of the frame shows.

        **The engine answers which tile stands at a pixel, and this asks it.**
        The answer is a whole tile, and the page needs the place inside that
        tile as well. The engine draws its flat map at a fixed number of
        pixels for each column and each row, so the edge between two tiles
        stands at a fixed place. This finds that edge by halving, which costs
        a few dozen questions and states no layout of its own.
        """
        middle_x, middle_y = width / 2.0, height / 2.0
        column, row = camera.tile_at(middle_x, middle_y)
        into_x = _edge_below(
            lambda place: camera.tile_at(place, middle_y)[0] == column,
            middle_x,
            float(camera.tile_width),
        )
        into_y = _edge_below(
            lambda place: camera.tile_at(middle_x, place)[1] == row,
            middle_y,
            float(camera.tile_height),
        )
        tile_q = column + into_x
        tile_r = row + into_y
        at_column, at_row = stood.to_page(
            np.array([tile_q], dtype=np.float64), np.array([tile_r], dtype=np.float64)
        )
        # The projection answers the flat page. The lift moves every point up
        # by the height of the ground, and ground with no height stands this
        # far down the lifted page.
        return float(at_column[0]), float(at_row[0]) + stood.rise + LIFT_MARGIN

    def ground_step(
        self, camera: Camera, across: float, down: float
    ) -> tuple[float, float]:
        """Turn a step across the frame into a step across the flat map.

        The page turns the ground about the up direction and then leans it
        away from the watcher. A hand that drags across the frame therefore
        asks the ground to move in another direction, and this is the inverse
        of the page: the answer is the step the flat map must take so that the
        drawing moves the way the hand went.

        **The two angles come from the view, which is the one place that holds
        them.** This module keeps no copy of either, so a page that stands
        somewhere else cannot be dragged as though it stood here.
        """
        lean = max(self.view.lean, TINY)
        turn_x, turn_y = math.cos(self.view.turn), math.sin(self.view.turn)
        # The lean squashes the page down, so a step down the frame asks for a
        # longer step across the ground before the turn is taken out.
        deep = down / lean
        plan_x = across * turn_x + deep * turn_y
        plan_y = -across * turn_y + deep * turn_x
        # The flat map spaces its rows by the height of a tile, and the plan
        # of a hex grid spaces them closer, so the two differ by the pitch.
        height_of = max(float(camera.tile_height), TINY)
        width_of = max(float(camera.tile_width), TINY)
        return plan_x, plan_y * width_of / (height_of * ROW_PITCH)

    def fit_lists(
        self,
        stood: Projection,
        box: tuple[int, int, int, int],
        camera: Camera,
        width: int,
        height: int,
    ) -> tuple[np.ndarray, np.ndarray]:
        """Say which point of the page each pixel of the frame shows.

        The answer is one list for the columns of the frame and one for the
        rows. A value below nought names a pixel the page does not reach, and
        a caller draws bare paper there.

        **Both renderers take this mapping from one rule.** A mapping worked
        out twice is one value declared twice: the two divide by the same
        scale at different widths, and near a boundary they land on
        neighbouring points of a hatched page.
        """
        first_row, last_row, first_col, last_col = box
        # **The page says how many of its points one tile covers, and the
        # camera says how many pixels one tile covers.** The ratio is
        # therefore how many pixels one point of the page covers, and a drag
        # moves the drawing by the pixels the hand moved.
        scale = max(float(camera.tile_width) / max(stood.scale, TINY), TINY)
        middle_col, middle_row = self.page_middle(stood, camera, width, height)
        take_x = np.floor(
            middle_col + (np.arange(width) + 0.5 - width / 2.0) / scale
        ).astype(np.int32)
        take_y = np.floor(
            middle_row + (np.arange(height) + 0.5 - height / 2.0) / scale
        ).astype(np.int32)
        take_x = np.where((take_x >= first_col) & (take_x < last_col), take_x, -1)
        take_y = np.where((take_y >= first_row) & (take_y < last_row), take_y, -1)
        return take_x, take_y

    def _for(self, camera: Camera, width: int, height: int) -> Ground:
        """Give back the page of the ground, building it if the view moved.

        **The page follows the two angles as well as the window.** A turn or a
        lean moves every point of the page, so a page built at one pair of
        angles says nothing about another pair.
        """
        window = self._window(camera, width, height)
        detail = self.detail_of(camera, width, height)
        stand = (self.view.turn, self.view.lean, float(detail))
        held = self._ground
        if (
            held is not None
            and held.window == window
            and held.shape == (height, width)
            and held.stand == stand
        ):
            return held
        built = self._build(window, width, height, stand)
        self._ground = built
        return built

    def _build(
        self,
        window: tuple[int, int, int, int],
        width: int,
        height: int,
        stand: tuple[float, float, float],
    ) -> Ground:
        """Turn the window of tiles into a page, and hatch the ground on it.

        The pass reads the world backwards. For each point of the turned page
        it names the tile that stands there, so the page holds no hole and
        nothing is scattered.
        """
        ground = Ground(window, (height, width), stand)
        first_q, first_r, last_q, last_r = window
        water = self._water[first_r:last_r, first_q:last_q]
        raised = self._raised[first_r:last_r, first_q:last_q]
        deep = self._deep[first_r:last_r, first_q:last_q]

        stood = self.projection(window, width, height, int(stand[2]))
        take, held, within_q, within_r = self._turn(stood)
        ground.held = held
        plan = np.where(held, raised.ravel()[take], 0.0).astype(np.float32)
        wet = held & (water.ravel()[take])
        depth = np.where(held, deep.ravel()[take], 0.0).astype(np.float32)

        rise = stood.rise
        drawn, cliff, lifted = self._lift(plan, held, rise)
        ground.take = np.where(drawn, take.ravel()[lifted], -1)
        ground.drawn = drawn
        ground.cliff = cliff
        ground.water = drawn & wet.ravel()[lifted]
        held_height = np.where(drawn, plan.ravel()[lifted], 0.0).astype(np.float32)
        depth_here = np.where(drawn, depth.ravel()[lifted], 0.0).astype(np.float32)
        # The lift moves a point up the page, and a pixel under it belongs to
        # the face the lift exposed. The offset inside a tile follows the
        # point and not the pixel, in the same way that the height does.
        ground.within_q = np.where(drawn, within_q.ravel()[lifted], 0.0).astype(
            np.float32
        )
        ground.within_r = np.where(drawn, within_r.ravel()[lifted], 0.0).astype(
            np.float32
        )
        page_rows, page_cols = drawn.shape
        ground.page_x = np.broadcast_to(
            np.arange(page_cols, dtype=np.float32), (page_rows, page_cols)
        )
        ground.page_y = np.broadcast_to(
            np.arange(page_rows, dtype=np.float32)[:, None], (page_rows, page_cols)
        )
        ground.held_height = held_height
        ground.depth = depth_here
        ground.rise = rise
        ground.box = self.box_of(stood)
        if not self._marks:
            # A renderer that draws its own marks takes the page here. The
            # tone, the hatch, the silhouette and the sheet are every point of
            # the page several times over, and they are the cost of the build.
            return ground
        ground.tone = self._tone(held_height, rise)
        ground.coverage = self._hatch(
            held_height,
            ground.tone,
            drawn & ~ground.water,
            ground.water,
            cliff,
            depth_here,
            ground.page_x,
            ground.page_y,
        )
        ground.outline = self._outline(drawn, cliff)
        # **The sheet is drawn once.** The grain of the paper follows the size
        # of the page alone, so the sheet is built here and copied on every
        # frame after this one.
        #
        # The hatch and the silhouette are not baked into it. **The ink goes
        # on last**, over every wash, because that is the order a pencil study
        # is made in and because a wash laid over ink changes the colour of
        # the ink.
        ground.sheet = self._paper(page_rows, page_cols)
        return ground

    def projection(
        self,
        window: tuple[int, int, int, int],
        width: int,
        height: int,
        detail: int = 1,
    ) -> Projection:
        """Give back where each tile of the window falls on the page.

        **The two angles come from the view.** The turn is how far the ground
        is rotated about the up direction, and the lean is how far the page
        tips away from the watcher. The view opens at an eighth of a circle
        and a lean of one half, which is an isometric drawing.
        """
        return Projection(
            window,
            width * detail,
            height * detail,
            self.view.turn,
            self.view.lean,
            self._relief,
        )

    def box_of(self, stood: Projection) -> tuple[int, int, int, int]:
        """Give back the part of the page that holds the ground, with a margin.

        The turn spreads the world across a rectangle, and the world fills a
        slanted band inside it. The corners hold nothing, so a page that kept
        them would show the world small in the middle of the frame.

        **The projection sets the edge, and not the marks.** Both renderers
        take the box from here. A box read off the marks of one renderer would
        put the two pages at two scales, and every pixel would then differ
        because the two cut the paper differently.

        The sky reaches past the ground, and a page cut to the sky would show
        the land small. A sketch lets the sky run off the sheet.
        """
        first_q, first_r, last_q, last_r = stood.window
        # The ground covers one square of the grid for each tile, so the whole
        # window covers the rectangle between the outer edges of those squares.
        corner_q = np.array(
            [first_q - 0.5, last_q - 0.5, first_q - 0.5, last_q - 0.5],
            dtype=np.float64,
        )
        corner_r = np.array(
            [first_r - 0.5, first_r - 0.5, last_r - 0.5, last_r - 0.5],
            dtype=np.float64,
        )
        columns, rows = stood.to_page(corner_q, corner_r)
        # The lift moves the top of the ground up the page by the height under
        # it, and the face below reaches down to where the ground stood. The
        # foot is therefore the flat row, and the top is the flat row of the
        # tallest ground less its own rise.
        tile_q, tile_r = self._corners(stood)
        _, top_rows = stood.to_page(tile_q, tile_r)
        raised = self._raised[first_r:last_r, first_q:last_q].ravel()[None, :]
        lifted = top_rows - raised * stood.rise
        first_row = math.floor(float(lifted.min())) + stood.rise + LIFT_MARGIN
        last_row = math.ceil(float(rows.max())) + stood.rise + LIFT_MARGIN
        return (
            max(first_row - MARGIN, 0),
            min(last_row + MARGIN + 1, stood.rows),
            max(math.floor(float(columns.min())) - MARGIN, 0),
            min(math.ceil(float(columns.max())) + MARGIN + 1, stood.cols),
        )

    @staticmethod
    def _corners(stood: Projection) -> tuple[np.ndarray, np.ndarray]:
        """Give back the four corners of the square of every tile of a window.

        The answer is two arrays of four rows. One row holds one corner of
        every tile, in the order the window numbers the tiles.
        """
        first_q, first_r, last_q, last_r = stood.window
        columns = np.arange(first_q, last_q, dtype=np.float64)
        rows = np.arange(first_r, last_r, dtype=np.float64)
        grid_q, grid_r = np.meshgrid(columns, rows)
        steps = np.array([[-0.5, -0.5], [0.5, -0.5], [-0.5, 0.5], [0.5, 0.5]])
        tile_q = grid_q.ravel()[None, :] + steps[:, 0:1]
        tile_r = grid_r.ravel()[None, :] + steps[:, 1:2]
        return tile_q, tile_r

    def _turn(
        self, stood: Projection
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """Name the tile of the window that stands at each point of the page.

        **The pass reads backwards, one tile for each point of the page.** A
        pass that scattered the tiles forward would leave a hole between two
        of them, and a page of holes is a page of dust.

        Returns four arrays: the tile of the window that stands at each point
        of the turned page, as an index into the window; the mask of the
        points that hold one; and where inside that tile the point stands, on
        each axis of the grid.

        **The offset inside a tile falls out of the same arithmetic.** The
        pass already turns a point back into a place in the grid and rounds it
        to the nearest tile. What the rounding threw away is the offset, so a
        mark that runs through a tile costs a subtraction here and no second
        pass.
        """
        first_q, first_r, last_q, last_r = stood.window
        across = last_q - first_q
        down = last_r - first_r
        lean = stood.lean
        turn_x, turn_y = math.cos(stood.turn), math.sin(stood.turn)
        column = (np.arange(stood.cols, dtype=np.float32) - stood.cols / 2.0)[None, :]
        row = (
            (np.arange(stood.flat_rows, dtype=np.float32) - stood.flat_rows / 2.0)
            / lean
        )[:, None]
        # Turn the point of the page back into the plan of the world. The lean
        # is already taken out of the row above, so this is a plain rotation.
        plan_x = (column * turn_x + row * turn_y) / stood.scale + stood.plan_wide / 2.0
        plan_y = (row * turn_x - column * turn_y) / stood.scale + stood.plan_tall / 2.0
        tile_r = plan_y / ROW_PITCH
        tile_q = plan_x - tile_r * 0.5
        take_r = np.rint(tile_r).astype(np.int32)
        take_q = np.rint(tile_q).astype(np.int32)
        held = (take_q >= 0) & (take_q < across) & (take_r >= 0) & (take_r < down)
        take = np.clip(take_r, 0, down - 1) * across + np.clip(take_q, 0, across - 1)
        within_q = (tile_q - take_q).astype(np.float32)
        within_r = (tile_r - take_r).astype(np.float32)
        return take.astype(np.int32), held, within_q, within_r

    @staticmethod
    def _lift(
        plan: np.ndarray, solid: np.ndarray, rise: int
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
        """Move every point of the page up by the height of the ground on it.

        The nearest row of the page is the last row of the array, so a scatter
        in the order of the array leaves the nearest row on top and nothing
        needs sorting.

        A pixel that no point reached belongs to the face below the point
        above it. That face is the cliff the lift exposed, and it ends where
        the ground it stands on ends.

        Returns the mask of the pixels the lift covered, the mask of the
        pixels that are a cliff, and the point of the page each pixel takes
        its numbers from.
        """
        rows, cols = plan.shape
        page_rows = rows + rise + LIFT_MARGIN * 2
        landing = np.arange(rows, dtype=np.int32)[:, None] + rise + LIFT_MARGIN
        landing = np.clip(landing - (plan * rise).astype(np.int32), 0, page_rows - 1)
        column = np.broadcast_to(np.arange(cols, dtype=np.int32), plan.shape)
        owner = np.full((page_rows, cols), -1, dtype=np.int32)
        place = solid.ravel()
        owner[landing.ravel()[place], column.ravel()[place]] = np.arange(
            plan.size, dtype=np.int32
        )[place]
        page_row = np.arange(page_rows, dtype=np.int32)[:, None]
        marked = np.where(owner >= 0, page_row, np.int32(-1))
        highest = np.maximum.accumulate(marked, axis=0)
        page_column = np.broadcast_to(np.arange(cols, dtype=np.int32), owner.shape)
        take = np.where(highest >= 0, owner[np.clip(highest, 0, None), page_column], -1)
        foot = np.clip(take, 0, None) // cols + rise + LIFT_MARGIN
        take = np.where(page_row <= foot, take, -1)
        drawn = take >= 0
        cliff = drawn & (highest != page_row)
        return drawn, cliff, np.clip(take, 0, None)

    @staticmethod
    def _tone(held: np.ndarray, rise: float) -> np.ndarray:
        """Give back the tone of the light on the ground.

        The slope of the height and the way the ground faces give a normal,
        and the light gives the tone. A tone of zero is full light and a tone
        of one is full shade.
        """
        scale = max(rise, 1.0)
        slope_x = np.gradient(held, axis=1) * scale
        slope_y = np.gradient(held, axis=0) * scale
        normal = np.stack(
            [-slope_x, -slope_y, np.ones_like(held)], axis=-1, dtype=np.float32
        )
        normal /= np.clip(np.linalg.norm(normal, axis=-1, keepdims=True), 1e-4, None)
        lit = np.clip((normal * LIGHT).sum(axis=-1), 0.0, 1.0)
        shade: np.ndarray = np.clip(1.0 - lit * 1.3, 0.0, 1.0).astype(np.float32)
        return shade

    @staticmethod
    def _hatch(
        held: np.ndarray,
        tone: np.ndarray,
        land: np.ndarray,
        water: np.ndarray,
        cliff: np.ndarray,
        depth: np.ndarray,
        page_x: np.ndarray,
        page_y: np.ndarray,
    ) -> np.ndarray:
        """Draw every set of marks the ground carries.

        Each set carries one quantity, and the module docstring names which.
        """
        coverage = np.zeros_like(held)
        # The contour hatch. Evenly spaced in height, so it crowds on a slope.
        contour = _lines(held, CONTOUR_STEP, np.full_like(held, CONTOUR_WEIGHT))
        coverage = np.maximum(coverage, np.where(land, contour * 0.60, 0.0))
        # The shadow hatch. Fixed to the page, weighted by the aspect.
        shade = np.clip((tone - SHADOW_START) / (1.0 - SHADOW_START), 0.0, 1.0)
        first = _lines(page_x * 0.5 + page_y * 0.866, HATCH_SPACING, shade * 0.42)
        coverage = np.maximum(coverage, np.where(land, first * 0.82, 0.0))
        # The cross hatch. The deepest shade alone.
        deep = np.clip((tone - CROSS_START) / (1.0 - CROSS_START), 0.0, 1.0)
        second = _lines(page_x * 0.5 - page_y * 0.866, HATCH_SPACING, deep * 0.45)
        coverage = np.maximum(coverage, np.where(land, second * 0.86, 0.0))
        # The cliff hatch. It runs down the page on the faces the lift made.
        down = _lines(page_x, CLIFF_SPACING, np.full_like(page_x, 0.38))
        coverage = np.maximum(coverage, np.where(cliff, down * 0.75, 0.0))
        # The water hatch. It runs across the page, and its weight is depth.
        across = _lines(page_y, WATER_SPACING, 0.16 + depth * 0.5)
        wet: np.ndarray = np.maximum(coverage, np.where(water, across * 0.52, 0.0))
        return wet

    @staticmethod
    def _outline(drawn: np.ndarray, cliff: np.ndarray) -> np.ndarray:
        """Give back an ink line on every silhouette edge."""
        edge = np.zeros(drawn.shape, dtype=np.float32)
        for axis in (0, 1):
            for shift in (1, -1):
                edge = np.maximum(
                    edge, (drawn ^ np.roll(drawn, shift, axis=axis)).astype(np.float32)
                )
        top = (cliff & ~np.roll(cliff, 1, axis=0)).astype(np.float32)
        lined: np.ndarray = np.maximum(edge, top * 0.85) * 0.85
        return lined

    def _draw(
        self,
        ground: Ground,
        overlay: str | None,
        camera: Camera,
        width: int,
        height: int,
        phase: float,
    ) -> np.ndarray:
        """Put the layers that change over the ground the build drew.

        The ground is a copy. The cloud, the wind, the faction that holds each
        tile and a named overlay all change with the world, so they are read
        and drawn here.

        The phase is the share of the current tick that has elapsed. The cloud
        drifts on it, so a sky moves smoothly at every speed of the world.
        """
        page = ground.sheet.copy()
        page = self._holders(page, ground)
        rise = max(int(ground.drawn.shape[1] * self._relief), 1)
        if self._sky:
            page = self._sky_over(page, ground, rise, phase)
        page = self._wash(page, ground, overlay, camera, width, height)
        page = (
            page * (1.0 - ground.coverage[..., None]) + INK * ground.coverage[..., None]
        )
        # The ways go over the hatch. A road is a made thing, and a mark that
        # the hatch of the ground crossed would read as ground.
        stood = self.projection(ground.window, width, height, int(ground.stand[2]))
        page = self._ways(page, ground, stood.scale)
        edge = ground.outline[..., None]
        inked: np.ndarray = np.clip(page * (1.0 - edge) + INK * edge, 0.0, 255.0)
        return inked

    def road_lattice(self) -> tuple[np.ndarray, np.ndarray]:
        """Give back the joins and the level of a road, for every tile.

        The answer is two arrays shaped like the world. The first holds one
        bit for each of the six neighbours of a tile, and bit ``i`` is set
        when the neighbour in direction ``i`` carries a road. The second holds
        nothing where no road runs, one where a road is still being made, and
        the level above that where a road stands.

        **The joins are the engine's own, and this names no neighbour.** A
        renderer that worked out which tiles touch would hold a second copy of
        a rule the engine already applies, and nothing would fail when the two
        disagreed.[^1]

        **The crossing follows the roads and not the world.** The engine holds
        an upgrade sparsely and answers the whole road network in one call, so
        a world of sixteen million tiles with a hundred roads costs a hundred
        entries. This module scatters them onto the lattice that the drawing
        reads, in one array write and never in a loop.[^2]

        [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
        [^2]: ADR-0040, Python is a control plane, not a data plane,
        decisions D1 and D2.
        `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
        """
        ways = self._world.road_ways()
        rows, columns = self._world.height, self._world.width
        joins = np.zeros((rows, columns), dtype=np.uint8)
        # Nothing stands for a tile with no road, so a level that stands moves
        # up by one. A road under work stands at level nought in the engine,
        # and nought is also the tile that carries nothing.
        level = np.zeros((rows, columns), dtype=np.uint8)
        column, row = ways["q"], ways["r"]
        if column.size:
            joins[row, column] = ways["joins"]
            level[row, column] = ways["level"] + 1
        return joins, level

    def _ways(self, page: np.ndarray, ground: Ground, scale: float) -> np.ndarray:
        """Draw every road on the page, as a ribbon and not as a filled cell.

        **A road is a way.** It runs from somewhere to somewhere, it joins
        another road at a junction, it bends, and it ends. A coloured cell
        says that a tile carries the road property. It does not draw a road.

        The ribbon runs from the middle of a tile out to the middle of the
        edge it shares with each neighbour that carries a road. The neighbour
        draws to the same point from its own side, so the two halves meet and
        the way is continuous. A tile that joins one neighbour ends there. A
        tile that joins two bends. A tile that joins three or more is a
        junction. A tile that joins nothing draws a mark, because a made thing
        that nothing reaches is still a made thing.

        **The marks are the two edges of the way, and not a fill.** That is
        how a road is drawn on a map and in a study: a watcher follows two
        lines and reads the ground between them. The best road adds a line
        down the middle, so a watcher tells the two levels apart without
        comparing one road against another. A way that is still being made
        draws the middle line alone.

        **The pass costs the road and not the page.** The distance to the
        ribbon is worked out at the points that show a road and nowhere else,
        so a world with no road pays one test.
        """
        joins_of, level_of = self.road_lattice()
        if not level_of.any():
            return page
        level = self._gather(ground, level_of.astype(np.int32))
        # A road lies on the top of the ground. The face that the lift exposed
        # is a cliff below that top, so the ribbon does not run down it.
        here = ground.drawn & ~ground.cliff & (level > 0)
        if not here.any():
            return page
        joins = self._gather(ground, joins_of.astype(np.int32))[here]
        level = level[here]
        # Where the point stands inside its tile, in the plan of the world. A
        # row of the grid steps half a column across, so the offset on the
        # page is not the offset on the grid.
        off_q = ground.within_q[here]
        off_r = ground.within_r[here]
        plan_x = off_q + off_r * 0.5
        plan_y = off_r * ROW_PITCH
        # The distance to the ribbon. It opens at the distance to the middle
        # of the tile, which is the cap of a way that ends here and the round
        # of a junction that turns here.
        near = np.hypot(plan_x, plan_y)
        # The directions are the engine's own. This module states no order,
        # so a renumbering in the engine cannot leave a second order here.
        from cachette import World as WorldType

        for direction, (step_q, step_r) in enumerate(WorldType.direction_offsets()):
            joined = ((joins >> direction) & 1).astype(bool)
            if not joined.any():
                continue
            # The middle of the shared edge is half way between the two tile
            # middles, so the neighbour reaches the same point from its side.
            reach_x = (step_q + step_r * 0.5) * 0.5
            reach_y = step_r * ROW_PITCH * 0.5
            span = reach_x * reach_x + reach_y * reach_y
            along = np.clip((plan_x * reach_x + plan_y * reach_y) / span, 0.0, 1.0)
            arm = np.hypot(plan_x - along * reach_x, plan_y - along * reach_y)
            near = np.where(joined, np.minimum(near, arm), near)
        # The weight of a mark belongs to the page, so the width of a line is
        # a count of page points turned into a share of a tile.
        line = WAY_INK / max(scale, 1e-3)
        half = np.take(np.array(WAY_WIDTH, dtype=np.float32), level) * 0.5
        # A road that is still being made carries the middle line alone.
        planned = level == 1
        edge = np.clip(1.0 - np.abs(near - half) / line, 0.0, 1.0)
        middle = np.clip(1.0 - near / line, 0.0, 1.0)
        surface = np.clip((half - near) / line, 0.0, 1.0)
        laid_ink = np.maximum(edge * WAY_EDGE_INK, surface * WAY_FILL_INK)
        ink = np.where(planned, middle * WAY_PLANNED_INK, laid_ink)
        crowned = level >= CROWNED_LEVEL
        ink = np.where(crowned, np.maximum(ink, middle * WAY_CROWN_INK), ink)
        weight = np.zeros(ground.drawn.shape, dtype=np.float32)
        weight[here] = ink
        laid: np.ndarray = page * (1.0 - weight[..., None]) + INK * weight[..., None]
        return laid

    def _gather(self, ground: Ground, field: np.ndarray) -> np.ndarray:
        """Give every pixel of the page the number of the tile it shows."""
        first_q, first_r, last_q, last_r = ground.window
        window = field[first_r:last_r, first_q:last_q].ravel()
        taken = window[np.clip(ground.take, 0, None)]
        return np.where(ground.drawn, taken, 0)

    def _holders(self, page: np.ndarray, ground: Ground) -> np.ndarray:
        """Lay a light wash of the colour of the faction that holds the ground.

        **The colour says who holds the ground and nothing else.** The engine
        publishes one colour for each faction, so this module names none.
        """
        from cachette import faction_colours

        palette = np.array(
            [[(c >> 16) & 0xFF, (c >> 8) & 0xFF, c & 0xFF] for c in faction_colours()],
            dtype=np.float32,
        )
        rows, columns = self._world.height, self._world.width
        holders = self._world.tile_holders().reshape(rows, columns)
        # The engine reports a number no faction can carry for ground that
        # nobody holds, and it publishes the faction count, so the test is
        # against that count and never against the number itself.
        held = self._gather(ground, holders.astype(np.int32))
        owned = ground.drawn & (held < self._world.faction_count)
        colour = palette[np.clip(held, 0, len(palette) - 1) % len(palette)]
        tint = np.clip(colour / 255.0, 0.0, 1.0)
        weight = owned[..., None] * HOLDER_WASH
        washed: np.ndarray = page * (1.0 - weight) + page * tint * weight * 2.0
        return washed

    def overlay_paint(self, overlay: str) -> tuple[np.ndarray, np.ndarray]:
        """Give back the pigment of one overlay, for every tile of the world.

        The answer is two arrays shaped like the world. The first is the
        strength the overlay paints at, from none to one. The second is the
        colour it paints in, as three bands of 255.

        **The colours are the engine's own.** This module names no overlay
        colour and holds no ramp from a value to a strength. It asks the
        engine for both, over the whole world in one crossing.

        **The pigment belongs to a tile and not to a pixel.** The answer
        stands on the tile order of the world, so no camera and no frame size
        reaches it. A reading taken at the pixel the engine drew a tile on
        walks across the ground as a zoom moves the camera.[^4]

        [^4]: Findings register, FND-610. `docs/FINDINGS.md`
        """
        paint = self._world.overlay_paint(overlay)
        rows, columns = self._world.height, self._world.width
        flow = paint["strength"].reshape(rows, columns).astype(np.float32) / 255.0
        packed = paint["colour"].reshape(rows, columns).astype(np.uint32)
        hue = np.stack(
            [((packed >> shift) & 0xFF).astype(np.float32) for shift in (16, 8, 0)],
            axis=-1,
        )
        return flow, hue

    def _wash(
        self,
        page: np.ndarray,
        ground: Ground,
        overlay: str | None,
        camera: Camera,
        width: int,
        height: int,
    ) -> np.ndarray:
        """Lay a named overlay on the ground as a wash of water colour.

        **The colours are the engine's own.** This module names no overlay
        colour. It asks the engine what pigment the overlay lays on each tile,
        for the whole world in one crossing, and puts that pigment where the
        page shows the tile.

        **The wash is a glaze and not a coat.** The page multiplies the paper
        by the colour rather than replacing it, so the grain of the paper and
        the marks under the wash both stay visible.

        The grain of the paper carries the pigment, because pigment settles in
        the tooth of a sheet. The edge of a wash dries darker than the middle,
        and the page finds that edge as the difference between a near view and
        a far view of the same wash.
        """
        if not overlay:
            return page
        over_world, colour = self.overlay_paint(overlay)
        # The wash follows the ground, so it is read at the tile the page
        # shows and not at the pixel the flat map put it on.
        flow = self._gather(ground, over_world)
        hue = np.stack(
            [self._gather(ground, colour[..., band]) for band in range(3)], axis=-1
        )
        if not flow.any():
            return page
        rows, cols = ground.drawn.shape
        grain = self._paper_grain(rows, cols)
        settled = flow * (1.0 - GRANULATION + GRANULATION * 2.0 * grain)
        rim = np.clip(_smooth(settled, 4) - _smooth(settled, 14), 0.0, 1.0)
        weight = np.clip(settled * WASH_GAIN + rim * RIM_GAIN, 0.0, 1.0) * ground.drawn
        tint = np.clip(hue / 255.0 * GLAZE_LIFT, 0.0, 1.0)
        glazed: np.ndarray = (
            page * (1.0 - weight[..., None]) + page * tint * weight[..., None]
        )
        return glazed

    def sky_fields(self) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
        """Give back the cover and the heading of the wind, for every tile.

        The answer is three arrays shaped like the world. The first is how
        much of the sky the engine reports as cloud, from nothing to one. The
        second and the third are the direction the wind blows on the page, as
        a vector of length one.

        **This is the one reader of the sky, and both renderers call it.** The
        array renderer gathers these at the pixel. The device renderer uploads
        the same arrays to the graphics device. A renderer that worked the
        turn of the wind out again would hold a second copy of it, and nothing
        would fail when the two disagreed.[^1]

        **The weather stands on a lattice coarser than the tiles.** Every tile
        of one cell reports one cover and one wind, so the raw fields step at
        a cell edge and a cloud drawn from them carries a straight edge that
        no weather has. The mean over the pitch of that lattice turns the step
        into a slope. The engine publishes the pitch.

        **The wind stands on the axes of the hex grid.** The page turns those
        axes, so the wind turns with them. This is the forward turn, and the
        page is built from the backward one, so the two use one pair of angles
        and cannot part company.

        [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
        """
        rows, columns = self._world.height, self._world.width
        reach = max(int(self._world.weather_cell_tiles * SKY_SMOOTHING), 1)
        raw = self._world.cloud_shares().reshape(rows, columns).astype(np.float32)
        cover = _smooth(raw / max(self._whole_sky, 1.0), reach)
        winds = self._world.tile_winds()
        wind_q = _smooth(winds["q"].reshape(rows, columns).astype(np.float32), reach)
        wind_r = _smooth(winds["r"].reshape(rows, columns).astype(np.float32), reach)
        plan_x = wind_q + wind_r * 0.5
        plan_y = wind_r * ROW_PITCH
        along_turn = math.cos(self.view.turn)
        across_turn = math.sin(self.view.turn)
        page_dx = plan_x * along_turn - plan_y * across_turn
        page_dy = (plan_x * across_turn + plan_y * along_turn) * self.view.lean
        length = np.hypot(page_dx, page_dy)
        moving = length > 0.0
        held = np.where(moving, length, 1.0)
        return (
            cover.astype(np.float32),
            np.where(moving, page_dx / held, 1.0).astype(np.float32),
            np.where(moving, page_dy / held, 0.0).astype(np.float32),
        )

    def sky_clock(self, phase: float) -> float:
        """Give back the clock that the cloud drifts on, in ticks.

        The phase is the share of the current tick that has elapsed, so the
        answer moves smoothly and a slow world still shows a moving sky.

        **The tick folds into a range.** The drift is a distance that grows
        with the clock, and a real number holds a large distance coarsely. The
        range is a power of two, so the fold costs nothing and the picture
        keeps its detail however long a world runs.

        **Both renderers call this.** A clock worked out twice would be one
        value in two places, and the two skies would part company on a frame
        where the tick advanced between the two readings.[^1]

        [^1]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
        """
        return float(self._world.tick & CLOUD_CLOCK_MASK) + phase

    def cloud_lattice(self) -> np.ndarray:
        """Give back the lattice of values the cloud noise reads, built once.

        The device renderer uploads this same table, so one set of numbers
        serves both renderers.
        """
        if self._cloud is None:
            self._cloud = _cloud_table(CLOUD_SEED)
        return self._cloud

    def cloud_body(
        self,
        place_x: np.ndarray,
        place_y: np.ndarray,
        along_x: np.ndarray,
        along_y: np.ndarray,
        clock: float,
        lean: float,
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
        """Read the cloud field at a place on the page.

        The place is a point of the page. The heading is where the wind blows
        there, on the page. The clock is in ticks and the lean is how far the
        page tips away from the watcher.

        Returns the density of the whole field, the value of its coarsest
        octave, and the two numbers of the place on the cloud lattice that the
        drift and the swirl carried the reading to. The caller reads the last
        two again to find the face of a mass against the light.

        **The motion belongs to the renderer.** The sky over a place holds one
        narrow band and does not leave it, so a picture that waited for the
        engine to move the weather would stand still.[^1] The drift carries
        the whole field along the wind at a fixed rate.

        **The swirl is a second field, and it crosses the page more slowly.**
        A mass therefore runs through it and deforms as it goes. Two fields
        at one rate would slide together and nothing would change shape.

        **The cloud layer leans with the page.** A layer seen from the side is
        shorter down the page than it is across it, so the reading divides the
        row by the lean and the cells of the field lean with the ground.

        **This declares the arithmetic and the shader repeats it.** A test
        compares the frames that the two renderers draw.[^2]

        [^1]: Findings register, FND-715. `docs/FINDINGS.md`
        [^2]: The two renderers, and the bound they agree within.
        `tests/test_demo_sketch_gl.py`
        """
        table = self.cloud_lattice()
        tall = CLOUD_GRAIN * max(lean, 1e-3)
        run = clock * CLOUD_DRIFT
        gust = run * CLOUD_GUST
        swirl_x = (place_x - along_x * gust) / (CLOUD_GRAIN * CLOUD_SWIRL_GRAIN)
        swirl_y = (place_y - along_y * gust) / (tall * CLOUD_SWIRL_GRAIN)
        turn_u = _cloud_noise(table, swirl_x, swirl_y) - 0.5
        turn_v = (
            _cloud_noise(
                table, swirl_x + CLOUD_SWIRL_APART, swirl_y + CLOUD_SWIRL_APART
            )
            - 0.5
        )
        at_x = (place_x - along_x * run) / CLOUD_GRAIN + turn_u * CLOUD_SWIRL
        at_y = (place_y - along_y * run) / tall + turn_v * CLOUD_SWIRL
        total = np.zeros_like(at_x)
        coarse = np.zeros_like(at_x)
        weight = 0.0
        scale = 1.0
        part = 1.0
        for octave in range(CLOUD_OCTAVES):
            shift = octave * CLOUD_OCTAVE_SHIFT
            value = _cloud_noise(table, at_x * scale + shift, at_y * scale + shift)
            if octave == 0:
                coarse = value
            total = total + value * part
            weight = weight + part
            scale = scale * 2.0
            part = part * CLOUD_OCTAVE_FALL
        return total / weight, coarse, at_x, at_y

    def _sky_over(
        self, page: np.ndarray, ground: Ground, rise: int, phase: float
    ) -> np.ndarray:
        """Put the cloud over the ground, as a mass and not as a tint.

        **The cloud is a layer above the ground.** It sits higher on the page
        than the tallest ground, so a mass crosses the paper over a mountain
        and a watcher reads that the mountain stands under it. A tint on the
        ground would say nothing about height.

        **The cover sets a mark and the mass stands over it.** A cover share
        and an opacity are two quantities, and the engine carries one. A
        renderer that turned the share into a weight painted thin cloud
        everywhere, and a broken sky cannot be drawn that way.[^3] The cover
        therefore says how much of the sky closes, and a field of noise says
        which part of it.

        **A mass has a lit side and a shaded side.** The page reads the
        coarsest octave again a short step toward the light, and the
        difference is the face. That is what gives a mass body.

        **The deep of the sky is the top of the cover range.** A sky that
        stands there darkens the whole of itself and closes its cloud over.
        The engine holds a storm as an object with a pressure deficit, and the
        Python boundary does not publish that deficit for each tile, so this
        reads the cover instead.[^3]

        **The shadow of a mass is that mass, moved.** The cloud drawn at a
        point of the page stands over the ground a lift below it, so the
        shadow at a point of the ground is the mass drawn a lift above it and
        a step across it. The array renderer moves the field it already drew.

        **The page has an edge and the sky stops at it.** The world does not
        wrap, so a point off the page is not a point of another part of the
        world.[^1] It carries no cloud, and it casts no shadow.

        **The cloud crosses the paper and its shadow does not.** A cloud
        stands above the ground and a watcher reads it over bare paper. A
        shadow falls on something, and there is nothing outside the map for it
        to fall on.[^2]

        [^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial,
        decision D2.
        `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
        [^2]: Findings register, FND-627. `docs/FINDINGS.md`
        [^3]: Findings register, FND-715. `docs/FINDINGS.md`
        """
        cover, heading_x, heading_y = self.sky_fields()
        clock = self.sky_clock(phase)
        lean = float(self.view.lean)
        share = self._gather(ground, cover)
        along_x = self._gather(ground, heading_x)
        along_y = self._gather(ground, heading_y)
        # **The page has an edge, and the sky stops at it.** A field rolled
        # off one side and back onto the other draws the weather of one part
        # of the world over another part, and a watcher reads a mark shaped
        # like ground that is somewhere else.
        lift = int(rise * CLOUD_HEIGHT)
        step = max(int(ground.drawn.shape[1] * CLOUD_SHADOW_STEP), 1)
        above = _slid(share, -lift, 0)
        heads_x = _slid(along_x, -lift, 0)
        heads_y = _slid(along_y, -lift, 0)
        stands = above > CLOUD_FLOOR
        mass = np.zeros(above.shape, dtype=np.float32)
        face = np.zeros(above.shape, dtype=np.float32)
        if stands.any():
            density, coarse, at_x, at_y = self.cloud_body(
                ground.page_x[stands],
                ground.page_y[stands],
                heads_x[stands],
                heads_y[stands],
                clock,
                lean,
            )
            toward_x, toward_y = _toward_light()
            ahead = _cloud_noise(
                self.cloud_lattice(),
                at_x + toward_x * CLOUD_LIGHT_STEP,
                at_y + toward_y * CLOUD_LIGHT_STEP,
            )
            mass[stands] = cloud_mass_of(density, above[stands])
            face[stands] = np.clip(0.5 - (coarse - ahead) * CLOUD_RELIEF, 0.0, 1.0)
        # **A shadow falls on something.** The cloud is drawn above the
        # ground, so it crosses bare paper, and a shadow that crossed the
        # paper with it drew a grey copy of the ground beside the ground. A
        # watcher read a shape like the terrain that stood in the wrong
        # place.[^2]
        under = _slid(_slid(mass, lift, 0), step, 1) * ground.drawn
        page = page * (1.0 - under[..., None] * CLOUD_SHADOW_DEPTH)
        deep = np.clip((above - SKY_DEEP_MARK) / (1.0 - SKY_DEEP_MARK), 0.0, 1.0)
        sky = SKY_INK + (STORM_INK - SKY_INK) * deep[..., None]
        gloom = (deep * SKY_GLOOM)[..., None]
        page = page * (1.0 - gloom) + sky * gloom
        ink = mass * (CLOUD_FACE_LIT + (CLOUD_FACE_DARK - CLOUD_FACE_LIT) * face)
        clouded: np.ndarray = page * (1.0 - ink[..., None]) + sky * ink[..., None]
        return clouded

    def _paper_grain(self, rows: int, cols: int) -> np.ndarray:
        """Give back the grain of the paper at this size, built once."""
        if self._grain is None or self._grain_size != (rows, cols):
            self._grain = _grain(cols, rows, GRAIN_SEED)
            self._grain_size = (rows, cols)
        return self._grain

    def _paper(self, rows: int, cols: int) -> np.ndarray:
        """Give back the page, as paper with its grain, at this size."""
        grain = self._paper_grain(rows, cols)
        page = np.broadcast_to(PAPER, (rows, cols, 3)).copy()
        return page * (0.955 + grain * 0.070)[..., None]

    def _fit(
        self,
        page: np.ndarray,
        box: tuple[int, int, int, int],
        stood: Projection,
        camera: Camera,
        width: int,
        height: int,
    ) -> npt.NDArray[np.uint32]:
        """Put the page into the frame under the camera, and pack the bytes.

        The page holds the whole world. The camera says which point of the
        page the middle of the frame shows and how far the page is magnified,
        so a drag moves the drawing and a zoom makes it larger. A pixel the
        page does not reach carries bare paper.
        """
        take_x, take_y = self.fit_lists(stood, box, camera, width, height)
        frame = np.broadcast_to(PAPER * 0.98, (height, width, 3)).copy()
        inside = (take_y >= 0)[:, None] & (take_x >= 0)[None, :]
        drawn = page[np.clip(take_y, 0, None)][:, np.clip(take_x, 0, None)]
        frame = np.where(inside[..., None], drawn, frame)
        packed: Any = frame.astype(np.uint32)
        bytes_of: np.ndarray = (
            (packed[..., 0] << 16) | (packed[..., 1] << 8) | packed[..., 2]
        ).astype(np.uint32)
        return bytes_of


def _channels(pixels: npt.NDArray[np.uint32], width: int, height: int) -> np.ndarray:
    """Split one frame into red, green and blue, as three planes."""
    frame = pixels.reshape(height, width)
    split: np.ndarray = np.stack(
        [(frame >> 16) & 0xFF, (frame >> 8) & 0xFF, frame & 0xFF], axis=-1
    ).astype(np.float32)
    return split
