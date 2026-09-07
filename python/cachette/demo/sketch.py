"""A sketchbook renderer: the world as a pencil study in ink on paper.

The demonstration draws one frame through one call. This module offers a
second renderer at that same call, so a flag chooses a renderer and the clock,
the panels, the keys and the window memory stay shared.[^1]

**This renderer reads the world. It never writes to it.**[^2] It asks the
engine for pictures of the world through the drawing call the window already
uses, and it composes those pictures into a sketch. It touches no simulated
value, no state hash and no step.

What the sketch shows
---------------------

The picture is an isometric view of the ground. The engine draws the world
from above. This module turns that picture an eighth of a circle, leans it
away from the watcher, and lifts every point by the height of the ground under
it. A point that stands high moves up the page, and the face below it becomes
a cliff. The flat view cannot show this, and the world holds real terrain.

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

The cloud hatch follows the edge of a cloud mass, and its weight grows with
the cloud share. The weight of this set is the cloud. The sky carries the
lightest marks on the page, so the land holds the weight.

Where the numbers come from
---------------------------

The height of the ground reaches this module through the engine's own height
overlay. The cloud share reaches it through the engine's own cloud overlay.
**This module never maps a tile to a weather cell.** It reads the picture the
engine painted, and the engine answered that mapping.[^3]

A second world of the same seed supplies the height picture. That world is
never seeded and never stepped, so it holds the same ground, and it holds no
faction colour, no upgrade and no unit to mix into the height reading.

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

ADR-0067, the viewer reads the world and never writes to it, decision D3.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``

Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import numpy as np

from cachette import Camera, World

if TYPE_CHECKING:
    from collections.abc import Sequence

    import numpy.typing as npt

    from cachette._core import FrameReading

# The name of the overlay that reports the height of the ground, and the name
# of the overlay that reports the cloud in the sky.
#
# The engine publishes the overlay names. This module refuses a name the
# engine does not publish, so a renamed overlay stops the run rather than
# drawing a flat world under a clear sky.
HEIGHT_OVERLAY = "height"
CLOUD_OVERLAY = "cloud"

# The colour the height overlay paints, and the colour the cloud overlay paints,
# as red, green and blue.
#
# **These are the one place this module states a colour of the engine.** The
# reading below solves for the strength the engine mixed, and it needs the
# colour that was mixed in.
HEIGHT_INK = np.array([0xF0, 0xE2, 0xA8], dtype=np.float32)
CLOUD_INK = np.array([0xD8, 0xE8, 0xF8], dtype=np.float32)

# The strength the engine paints just above the low end of an overlay span,
# and the strength it paints at the high end, of 255.
#
# The engine lifts the first step off the ground so that a watcher tells an
# empty tile from a nearly empty one. The reading undoes that lift.
LEAST_STRENGTH = 24.0
FULL_STRENGTH = 210.0

# How light a pixel must be before this module reads it as the ground of a
# tile. The engine paints one dark colour outside the world.
INSIDE_LIGHT = 34.0

# How far apart the ground and an overlay colour must stand in one channel
# before the reading trusts that channel, and how far two channels may
# disagree about the strength before the reading refuses the pixel.
#
# **A mix moves the three channels together.** A pixel the engine painted over
# rather than mixed into moves them apart, so the disagreement names it.
CHANNEL_GAP = 24.0
CHANNEL_TOLERANCE = 20.0

# How thick a letter the engine writes on the frame may be, in pixels.
#
# The ground of a world is wider than a letter, so shrinking the ground mask
# by this much drops every word and keeps the world.
STROKE = 2

# How far the page looks for sound ground when it mends a refused reading, in
# pixels. The key of an overlay is the widest thing the engine draws over the
# map, so the reach must pass it.
MENDING = 48

# How wide a card the engine writes on the map may be, in pixels.
#
# The page closes the ground mask over this reach, so a gap narrower than
# twice this reach fills and the coast stays where it is.
CARD_REACH = 64

# How far blue must lead red before this module reads a pixel as water.
#
# **The test is that blue leads, not that a pixel holds one named colour.** A
# named colour would be a second copy of a number the view crate holds, and
# nothing would fail when that number changed.
WATER_LEAD = 24.0

# The paper, the ink, and the ink of the sky, as red, green and blue.
#
# The sky is lighter than the land on purpose. In a pencil study the clouds
# are the lightest marks on the page.
PAPER = np.array([0xF2, 0xEC, 0xDD], dtype=np.float32)
INK = np.array([0x25, 0x23, 0x20], dtype=np.float32)
SKY_INK = np.array([0x5A, 0x62, 0x6E], dtype=np.float32)

# Where the light stands. It comes from the upper left, which is where a right
# handed person holding a pencil puts it.
LIGHT = np.array([-0.55, -0.62, 0.56], dtype=np.float32)

# How far the page leans away from the watcher.
#
# One is a plan seen from above and zero is a plan seen edge on. This is the
# two to one of an isometric drawing.
LEAN = 0.5

# How far the tallest ground rises, as a share of the width of the page.
RELIEF = 0.075

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
CLOUD_SPACING = 9.0

# The tone at which the shadow hatch starts, and at which the cross hatch
# starts. A tone runs from zero in full light to one in full shade.
SHADOW_START = 0.34
CROSS_START = 0.66

# The share of the width of the page that the cloud shadow moves.
CLOUD_SHADOW_STEP = 0.030

# The share of the cloud field below which the sky holds no mark.
CLOUD_FLOOR = 0.12

# How far the cloud layer stands over the tallest ground, as a share of the
# rise of the ground, and how far a cloud darkens the ground it covers.
#
# The cloud sits above the tallest ground on purpose. A mass that crossed the
# paper behind a mountain would say nothing about the height of the mountain.
CLOUD_HEIGHT = 1.6

# How many lines of constant cloud the hatch draws over the whole range.
CLOUD_CONTOURS = 14.0
CLOUD_SHADOW_DEPTH = 0.16

# How much of the engine's own colour survives into the wash.
#
# **The sketch stays close to monochrome.** The picture the engine draws
# already carries the terrain, the faction that holds the ground, the upgrades
# and the units as colour. The wash keeps a trace of that colour, so a watcher
# reads who holds what, and drops the rest so that the ink carries the
# drawing.
WASH_COLOUR = 0.30
WASH_WEIGHT = 0.26

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

# How far the reading of an overlay is smoothed, in pixels.
#
# The engine carries the value of an overlay in eight bits of one channel, so
# the reading steps. A step in the height is a cliff on the page, and a field
# of steps draws a field of cliffs that the ground does not hold.
SMOOTHING = 6

# How many pixels of water a frame must hold before it states a water level,
# and the share of those heights the level takes.
#
# A frame with a little water in one corner reports the height of that corner
# and not the level of the sea, so the reading waits for enough of it.
WATER_SAMPLE = 4096
WATER_QUANTILE = 0.92

# The number the grain of the paper draws from.
#
# **The grain belongs to the paper, not to the world.** One number here gives
# every run the same sheet, so two pictures of two worlds differ in the world
# alone.
GRAIN_SEED = 0x5C4E_7C48

# How much darker than the paper a pixel must be before the page counts it as
# a mark, and how much bare paper stays around the marks, in pixels.
#
# The grain of the paper darkens a pixel a little, so the test sits below the
# grain and above nothing.
MARKED = 0.92
MARGIN = 24


class BoundaryGap(Exception):
    """The engine does not publish something this renderer needs."""


def _channels(pixels: npt.NDArray[np.uint32], width: int, height: int) -> np.ndarray:
    """Split one frame into red, green and blue, as three planes."""
    frame = pixels.reshape(height, width)
    split: np.ndarray = np.stack(
        [(frame >> 16) & 0xFF, (frame >> 8) & 0xFF, frame & 0xFF], axis=-1
    ).astype(np.float32)
    return split


def _share(
    base: np.ndarray, over: np.ndarray, ink: np.ndarray
) -> tuple[np.ndarray, np.ndarray]:
    """Solve for the share of an overlay that the engine mixed into a picture.

    The engine mixes one colour into the ground at a strength, so a pixel of
    the overlay picture is the ground plus the strength times the difference
    between the overlay colour and the ground. Two pictures of one place
    therefore give the strength back.

    **Every channel must give the same strength, and the answer checks that.**
    A mix moves the three channels together. A pixel that the engine painted
    over rather than mixed into, such as a letter of the overlay key, moves
    them apart, and the check finds it. Without the check the key of the
    overlay reads as a plateau, and the letters of a message read as ground.

    Returns the share of the span of the overlay, from zero to one, and the
    mask of the pixels whose channels agree.
    """
    gap = ink - base
    lift = over - base
    usable = np.abs(gap) > CHANNEL_GAP
    strength = 255.0 * lift / np.where(usable, gap, 1.0)
    counted = usable.sum(axis=-1)
    total = np.where(usable, strength, 0.0).sum(axis=-1)
    mean = total / np.clip(counted, 1, None)
    apart = np.where(usable, np.abs(strength - mean[..., None]), 0.0).max(axis=-1)
    agreed = (counted >= 2) & (apart < CHANNEL_TOLERANCE)
    share = np.clip(
        (mean - LEAST_STRENGTH) / (FULL_STRENGTH - LEAST_STRENGTH), 0.0, 1.0
    )
    return share.astype(np.float32), agreed


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


def _grain(width: int, height: int, seed: int) -> np.ndarray:
    """Give back the grain of the paper, as a field of light and dark.

    The grain is presentation and not simulation. It draws from a generator
    this module seeds from the seed of the world, so one world gives one page.
    """
    engine = np.random.default_rng(seed & 0xFFFF_FFFF)
    coarse = engine.random((height // 3 + 2, width // 3 + 2), dtype=np.float32)
    grown = np.repeat(np.repeat(coarse, 3, axis=0), 3, axis=1)
    return grown[:height, :width]


def _smooth(field: np.ndarray, radius: int) -> np.ndarray:
    """Give back the mean of each point and the points around it.

    The mean runs across the page and then down it, and each pass sums over a
    running total, so the cost follows the number of pixels and not the square
    of the radius.
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


def _mended(field: np.ndarray, sound: np.ndarray, radius: int) -> np.ndarray:
    """Fill the points a reading refused, from the points around them.

    The mean runs over the sound points alone and then divides by how many of
    them it found, so a point beside a hole takes the value of its neighbours
    and not a value pulled toward zero by the hole itself.
    """
    keep = sound.astype(np.float32)
    found = _smooth(keep, radius)
    filled = _smooth(field * keep, radius) / np.clip(found, 1e-3, None)
    return np.where(sound, field, filled).astype(np.float32)


def _trim(page: np.ndarray) -> np.ndarray:
    """Cut the bare paper from the edge of a page.

    The turn spreads the plan across a rectangle, and the world fills the
    diamond inside it. The corners hold nothing, so a page that kept them
    would show the world small in the middle of the frame.

    A margin of paper stays, because a drawing needs one.
    """
    marked = page.min(axis=-1) < PAPER.min() * MARKED
    rows = np.nonzero(marked.any(axis=1))[0]
    cols = np.nonzero(marked.any(axis=0))[0]
    if not rows.size or not cols.size:
        return page
    edge = MARGIN
    first_row = max(int(rows[0]) - edge, 0)
    last_row = min(int(rows[-1]) + edge + 1, page.shape[0])
    first_col = max(int(cols[0]) - edge, 0)
    last_col = min(int(cols[-1]) + edge + 1, page.shape[1])
    return page[first_row:last_row, first_col:last_col]


def _eroded(mask: np.ndarray, radius: int) -> np.ndarray:
    """Shrink a mask, so that a thin stroke inside it disappears.

    The engine writes words on the frame: the clock, the key of an overlay and
    the hint line. A letter is light, so a test on brightness alone reads a
    letter as ground and the page then lifts the word into the drawing. A
    letter is also thin, and the ground of a world is not.
    """
    small = mask.copy()
    for step in range(1, radius + 1):
        small[:, step:] &= mask[:, :-step]
        small[:, :-step] &= mask[:, step:]
        small[step:, :] &= mask[:-step, :]
        small[:-step, :] &= mask[step:, :]
    return small


def _closed(mask: np.ndarray, across: int, down: int) -> np.ndarray:
    """Grow a mask over a short run, so that a hole inside it closes."""
    wide = mask.copy()
    for step in range(1, across + 1):
        wide[:, step:] |= mask[:, :-step]
        wide[:, :-step] |= mask[:, step:]
    tall = wide.copy()
    for step in range(1, down + 1):
        tall[step:, :] |= wide[:-step, :]
        tall[:-step, :] |= wide[step:, :]
    return tall


class Sketch:
    """A renderer that draws the world as a pencil study.

    A caller builds one of these from the world, then calls it exactly as it
    calls the drawing method of the world. The call fills the pixels it is
    given and gives back what the drawing pass read.
    """

    __slots__ = (
        "_grain",
        "_grain_size",
        "_lean",
        "_relief",
        "_scratch",
        "_sky",
        "_twin",
        "_water_level",
        "_world",
    )

    def __init__(
        self,
        world: World,
        *,
        seed: int | None = None,
        relief: float = RELIEF,
        lean: float = LEAN,
        sky: bool = True,
    ) -> None:
        """Build the renderer for one world.

        The seed is the seed of the world. **The world is the source of it**,
        and a caller that names none takes it from the world, so the two
        cannot disagree. A caller names it only when it holds a world that
        does not report one.

        The relief is how far the tallest ground rises, as a share of the
        width of the page. The lean is how far the page tips away from the
        watcher, where one is a plan and zero is an edge.

        Raises ``BoundaryGap`` when the engine publishes no height overlay or
        no cloud overlay.
        """
        published = World.overlay_names()
        for name in (HEIGHT_OVERLAY, CLOUD_OVERLAY):
            if name not in published:
                message = (
                    f"the engine publishes no {name!r} overlay, and the "
                    f"sketch renderer reads the world through it"
                )
                raise BoundaryGap(message)
        self._world = world
        self._relief = relief
        self._lean = lean
        self._sky = sky
        # A second world of the same size and seed holds the same ground, and
        # it holds no faction colour, no upgrade and no unit. It is never
        # seeded and never stepped, so it cannot drift from the ground it
        # copies, and it cannot report a fact of the present frame.
        #
        # **The twin names no faction count.** The ground follows the size and
        # the seed, and a world that seeds no faction holds no faction, so a
        # count here would be a number that changes nothing.
        self._twin = World(
            width=world.width,
            height=world.height,
            seed=world.seed if seed is None else seed,
        )
        self._scratch: dict[str, npt.NDArray[np.uint32]] = {}
        self._grain: np.ndarray | None = None
        self._grain_size = (0, 0)
        self._water_level: float | None = None

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
        plain = self._buffer("plain", width, height)
        lifted = self._buffer("height", width, height)
        ground = self._buffer("ground", width, height)
        # The frame the window would have drawn. It carries the panels, the
        # clock and the pointer, and its reading is the reading this call
        # gives back, so no number here is a second copy of a number there.
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
        # The same frame without the panels. The two differ where a panel
        # painted, and nowhere else.
        self._world.draw(camera, width, height, plain, phase=phase)
        self._twin.draw(camera, width, height, ground)
        self._twin.draw(camera, width, height, lifted, overlay=HEIGHT_OVERLAY)
        base = _channels(plain, width, height)
        terrain = _channels(ground, width, height)
        share, agreed = _share(terrain, _channels(lifted, width, height), HEIGHT_INK)
        cloud = self._read_cloud(camera, width, height, base, phase)
        glaze = self._read_overlay(camera, width, height, overlay, phase)
        page, _written = self._page(
            base, terrain, share, agreed, ground, cloud, glaze, width, height
        )
        # **The panels and the clock stay the engine's.** A panel differs
        # from the frame without it, and the chrome the engine writes over the
        # map belongs on top of the sketch rather than under it.
        # The frame the caller asked for, without the panels. A named overlay
        # tints the whole map, so a comparison against the plain frame would
        # call every tinted pixel a panel.
        asked = base if glaze is None else glaze
        kept = _closed(
            np.any(np.abs(_channels(pixels, width, height) - asked) > 0.5, axis=-1),
            STROKE,
            STROKE,
        )
        frame = pixels.reshape(height, width)
        drawn = self._fit(page, width, height)
        frame[...] = np.where(kept, frame, drawn)
        return reading

    def _read_overlay(
        self,
        camera: Camera,
        width: int,
        height: int,
        overlay: str | None,
        phase: float,
    ) -> np.ndarray | None:
        """Give back the frame the engine painted with one overlay on it.

        **The colours are the engine's own.** This module names no overlay
        colour. It asks the engine for the frame with the overlay and for the
        frame without it, and the difference between the two is the pigment
        the overlay laid down.
        """
        if not overlay:
            return None
        washed = self._buffer("glaze", width, height)
        self._world.draw(camera, width, height, washed, overlay=overlay, phase=phase)
        return _channels(washed, width, height)

    def _read_cloud(
        self,
        camera: Camera,
        width: int,
        height: int,
        base: np.ndarray,
        phase: float,
    ) -> np.ndarray:
        """Give back the cloud share of every pixel, from the cloud overlay.

        **This module never maps a tile to a weather cell.** The engine paints
        the cloud overlay at the resolution of a tile and answers that mapping
        itself, so a reading of the painted picture is a reading at the pitch
        the engine chose.
        """
        if not self._sky:
            return np.zeros((height, width), dtype=np.float32)
        sky = self._buffer("cloud", width, height)
        self._world.draw(camera, width, height, sky, overlay=CLOUD_OVERLAY, phase=phase)
        share, agreed = _share(base, _channels(sky, width, height), CLOUD_INK)
        return np.where(agreed, share, 0.0).astype(np.float32)

    def _page(
        self,
        base: np.ndarray,
        terrain: np.ndarray,
        share: np.ndarray,
        agreed: np.ndarray,
        packed: npt.NDArray[np.uint32],
        cloud: np.ndarray,
        glaze: np.ndarray | None,
        width: int,
        height: int,
    ) -> tuple[np.ndarray, np.ndarray]:
        """Turn the pictures the engine drew into one page of ink on paper.

        Returns the page, and the mask of the pixels the engine wrote over the
        map. The caller puts those back on top of the sketch.
        """
        # A pixel is ground when the engine painted ground there and when the
        # height reading of that pixel holds together. The second test refuses
        # the key of the overlay and every letter the engine wrote on the map.
        # The engine writes on the frame. It draws the clock, a message and
        # the key of an overlay, and each of those hides the ground under it.
        # **A word is not the world.** The page reads no ground from a word.
        ground = _eroded(terrain.mean(axis=-1) > INSIDE_LIGHT, STROKE)
        # **A word is not the world.** The engine writes the clock and a
        # message over the map in a dark card, and the card is darker than any
        # ground. A page that read the card as open paper would cut a
        # rectangle out of the hills.
        #
        # The page closes the ground mask instead. It grows the mask and then
        # shrinks it by the same reach, which fills a gap narrower than that
        # reach and leaves the coast where it was. Nothing here reads a
        # colour, so no card the engine restyles can defeat it.
        # The reach never passes a sixth of the frame, so a small frame keeps
        # its world. A card the engine writes is small beside the frame it
        # writes on.
        reach = max(min(CARD_REACH, min(width, height) // 6), 1)
        inside = _eroded(_closed(ground, reach, reach), reach)
        written = inside & ~ground
        sound = inside & ~written
        share = _mended(share, sound & agreed, MENDING)
        base = np.stack(
            [_mended(base[..., band], sound, MENDING) for band in range(3)], axis=-1
        )
        terrain = np.stack(
            [_mended(terrain[..., band], sound, MENDING) for band in range(3)], axis=-1
        )
        water = inside & (terrain[..., 2] > terrain[..., 0] + WATER_LEAD)
        level = self._level(share, water)
        # The water sits at one level, and the height under it is the depth.
        depth = np.where(water, np.clip(1.0 - share / max(level, 1e-3), 0.0, 1.0), 0.0)
        # **The land starts at the level of the water and not at zero.** The
        # engine reports the height of the ground over the whole range it
        # generates, and the water covers the lower part of that range. A page
        # that put the water at zero would draw a cliff along every coast.
        share = np.clip(share - level, 0.0, None) / max(1.0 - level, 1e-3)
        # **The reading is smoothed before it is lifted.** The overlay carries
        # the height in eight bits of one channel, so the reading steps. A
        # step in the height is a cliff on the page, and a field of steps
        # draws a field of cliffs that the ground does not hold.
        eased = _smooth(np.where(inside & ~water, share, 0.0), SMOOTHING)
        plan = np.where(inside & ~water, eased, 0.0).astype(np.float32)
        # The overlay, as a wash of colour the engine chose. The pigment is
        # what the overlay added to the ground, so its weight is the strength
        # the overlay painted and its colour is the colour of the overlay.
        if glaze is None:
            pigment = np.zeros_like(base)
            flow = np.zeros(base.shape[:2], dtype=np.float32)
        else:
            # The rim of the world and the cards the engine writes carry no
            # ground, so the wash takes its colour from the ground beside them
            # in the same way the drawing does.
            pigment = np.stack(
                [_mended(glaze[..., band], sound, MENDING) for band in range(3)],
                axis=-1,
            )
            flow = np.where(
                inside, np.abs(pigment - base).max(axis=-1) / 255.0, 0.0
            ).astype(np.float32)
        fields: dict[str, np.ndarray] = {
            "wash": base,
            "hue": pigment,
            "flow": flow,
            "water": water.astype(np.float32),
            "inside": inside.astype(np.float32),
            "depth": depth.astype(np.float32),
            "cloud": _smooth(np.where(inside, cloud, 0.0), SMOOTHING),
        }
        turned, carried, held = self._turn(plan, fields, width, height)
        solid = held & (carried["inside"] > 0.5)
        return self._ink(turned, carried, solid), written

    def _level(self, share: np.ndarray, water: np.ndarray) -> float:
        """Give back the height at which the water stands, from zero to one.

        **The level is read once and then held.** A level read again on every
        frame would move as the camera moved, and the coast would rise and
        fall while the world stood still.

        The reading takes a high share of the heights the water covers, so one
        deep lake cannot pull the level down.
        """
        if self._water_level is None:
            found = share[water]
            if found.size < WATER_SAMPLE:
                return 0.0
            self._water_level = float(np.quantile(found, WATER_QUANTILE))
        return self._water_level

    def _turn(
        self,
        plan: np.ndarray,
        fields: dict[str, np.ndarray],
        width: int,
        height: int,
    ) -> tuple[np.ndarray, dict[str, np.ndarray], np.ndarray]:
        """Turn the plan an eighth of a circle and lean it away from the page.

        The plan the engine draws is square to the page. A square plan drawn
        straight reads as a flat map however far the ground is lifted, so the
        page turns before it lifts.

        The turn reads the plan backwards, one source point for each point of
        the turned picture, so the turned picture holds no hole.
        """
        scale = 0.70710678
        across_size = int((width + height) * scale) + 2
        down_size = int((width + height) * scale * self._lean) + 2
        across = (np.arange(across_size, dtype=np.float32) - across_size / 2.0)[None, :]
        down = (
            (np.arange(down_size, dtype=np.float32) - down_size / 2.0) / self._lean
        )[:, None]
        source_x = (across + down) / (2.0 * scale) + width / 2.0
        source_y = (down - across) / (2.0 * scale) + height / 2.0
        held = (source_x >= 0.0) & (source_x < width)
        held &= (source_y >= 0.0) & (source_y < height)
        take_x = np.clip(source_x, 0.0, width - 1).astype(np.int32)
        take_y = np.clip(source_y, 0.0, height - 1).astype(np.int32)
        turned = np.where(held, plan[take_y, take_x], 0.0).astype(np.float32)
        carried: dict[str, np.ndarray] = {}
        for name, field in fields.items():
            picked = field[take_y, take_x]
            keep = held[..., None] if picked.ndim == 3 else held
            carried[name] = np.where(keep, picked, 0.0).astype(np.float32)
        return turned, carried, held

    def _ink(
        self, plan: np.ndarray, fields: dict[str, np.ndarray], solid: np.ndarray
    ) -> np.ndarray:
        """Lift the turned plan by its height, and draw it in ink on paper.

        The solid mask names the points that hold ground. A point that holds
        none is not lifted and casts no face, so the paper stays bare beyond
        the edge of the world.
        """
        rise = max(int(plan.shape[1] * self._relief), 1)
        drawn, cliff, take = self._lift(plan, solid)
        # The shadow of a cloud lies on the ground under the cloud, so it
        # moves with the plan and then drapes over the relief with it.
        step = max(int(plan.shape[1] * CLOUD_SHADOW_STEP), 1)
        fields["shadow"] = np.roll(
            np.roll(fields["cloud"], step, axis=1), step * 2, axis=0
        )
        gathered = {
            name: self._gather(field, take, drawn) for name, field in fields.items()
        }
        held = self._gather(plan, take, drawn)
        water = gathered["water"] > 0.5
        inside = gathered["inside"] > 0.5
        land = drawn & inside & ~water
        rows, cols = held.shape
        page_x = np.broadcast_to(np.arange(cols, dtype=np.float32), (rows, cols))
        page_y = np.broadcast_to(
            np.arange(rows, dtype=np.float32)[:, None], (rows, cols)
        )
        tone, steep = self._tone(held, cols * self._relief)
        coverage = self._hatch(
            held, tone, steep, land, water, cliff & inside, gathered, page_x, page_y
        )
        page = self._paper(rows, cols)
        page = self._wash(page, gathered["wash"], drawn, land, tone)
        if self._sky:
            # The ground under a cloud stands in its shadow. The shadow
            # drapes over the relief, because it came through the same lift.
            under = np.clip(
                (gathered["shadow"] - CLOUD_FLOOR) / (1.0 - CLOUD_FLOOR), 0.0, 1.0
            )
            page = page * (1.0 - (under * drawn)[..., None] * CLOUD_SHADOW_DEPTH)
        page = self._colour_wash(
            page, gathered["hue"], gathered["flow"], drawn, rows, cols
        )
        page = page * (1.0 - coverage[..., None]) + INK * coverage[..., None]
        page = self._outline(page, drawn, cliff)
        if self._sky:
            page = self._sky_over(
                page, fields["cloud"], rise, page_x, page_y, plan.shape[0]
            )
        return np.clip(page, 0.0, 255.0)

    def _lift(
        self, plan: np.ndarray, solid: np.ndarray
    ) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
        """Move every point of the plan up the page by the height under it.

        The nearest row of the plan is the last row of the array, so a scatter
        in the order of the array leaves the nearest row on top and nothing
        needs sorting.

        A pixel that no point of the plan reached belongs to the face below
        the point above it. That face is the cliff the lift exposed.

        Returns the mask of the pixels the lift covered, the mask of the
        pixels that are a cliff, and the point of the plan each pixel takes
        its numbers from.
        """
        rows, cols = plan.shape
        rise = max(int(cols * self._relief), 1)
        page_rows = rows + rise + 8
        landing = np.arange(rows, dtype=np.int32)[:, None] + rise + 4
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
        page_column = np.broadcast_to(
            np.arange(cols, dtype=np.int32), (page_rows, cols)
        )
        take = np.where(highest >= 0, owner[np.clip(highest, 0, None), page_column], -1)
        # **A face ends where the ground it stands on would end.** The earth
        # under a point reaches down to the level of the water, so the face
        # below that point is as deep as the point is high. Without the bound
        # the face of the near edge of the world hangs to the foot of the
        # page, because no row in front of it ever stops the fill.
        foot = np.clip(take, 0, None) // cols + rise + 4
        take = np.where(page_row <= foot, take, -1)
        drawn = take >= 0
        cliff = drawn & (highest != page_row)
        return drawn, cliff, np.clip(take, 0, None)

    @staticmethod
    def _gather(field: np.ndarray, take: np.ndarray, drawn: np.ndarray) -> np.ndarray:
        """Give every pixel of the page the numbers of the point it shows."""
        flat = field.reshape(-1, field.shape[2]) if field.ndim == 3 else field.ravel()
        taken = flat[take]
        keep = drawn[..., None] if taken.ndim == 3 else drawn
        return np.where(keep, taken, 0.0).astype(np.float32)

    @staticmethod
    def _tone(held: np.ndarray, rise: float) -> tuple[np.ndarray, np.ndarray]:
        """Give back the tone of the light on the ground, and its steepness.

        The slope of the height and the way the ground faces give a normal,
        and the light gives the tone. A tone of zero is full light and a tone
        of one is full shade.
        """
        rise = max(rise, 1.0)
        slope_x = np.gradient(held, axis=1) * rise
        slope_y = np.gradient(held, axis=0) * rise
        normal = np.stack(
            [-slope_x, -slope_y, np.ones_like(held)], axis=-1, dtype=np.float32
        )
        normal /= np.clip(np.linalg.norm(normal, axis=-1, keepdims=True), 1e-4, None)
        lit = np.clip((normal * LIGHT).sum(axis=-1), 0.0, 1.0)
        tone = np.clip(1.0 - lit * 1.3, 0.0, 1.0)
        return tone.astype(np.float32), np.clip(np.hypot(slope_x, slope_y), 0.0, 6.0)

    @staticmethod
    def _hatch(
        held: np.ndarray,
        tone: np.ndarray,
        steep: np.ndarray,
        land: np.ndarray,
        water: np.ndarray,
        cliff: np.ndarray,
        fields: dict[str, np.ndarray],
        page_x: np.ndarray,
        page_y: np.ndarray,
    ) -> np.ndarray:
        """Draw every set of marks, and give back how much ink each pixel takes.

        Each set carries one quantity, and the module docstring names which.
        """
        coverage = np.zeros_like(held)
        # The contour hatch. Evenly spaced in height, so it crowds on a slope.
        contour = _lines(held, CONTOUR_STEP, np.full_like(held, CONTOUR_WEIGHT))
        coverage = np.maximum(coverage, np.where(land, contour * 0.55, 0.0))
        # The shadow hatch. Fixed to the page, weighted by the aspect.
        shade = np.clip((tone - SHADOW_START) / (1.0 - SHADOW_START), 0.0, 1.0)
        first = _lines(page_x * 0.5 + page_y * 0.866, HATCH_SPACING, shade * 0.42)
        coverage = np.maximum(coverage, np.where(land, first * 0.82, 0.0))
        # The cross hatch. The deepest shade alone.
        deep = np.clip((tone - CROSS_START) / (1.0 - CROSS_START), 0.0, 1.0)
        second = _lines(page_x * 0.5 - page_y * 0.866, HATCH_SPACING, deep * 0.45)
        coverage = np.maximum(coverage, np.where(land, second * 0.86, 0.0))
        # The cliff hatch. It runs down the page on the faces the lift made.
        down = _lines(page_x, CLIFF_SPACING, np.full_like(page_x, 0.5))
        coverage = np.maximum(coverage, np.where(cliff, down * 0.9, 0.0))
        # The water hatch. It runs across the page, and its weight is depth.
        across = _lines(page_y, WATER_SPACING, 0.16 + fields["depth"] * 0.5)
        coverage = np.maximum(coverage, np.where(water, across * 0.52, 0.0))
        # A slope steep enough to read gets a second contour between the
        # first, so a cliff edge on the land does not vanish into the shade.
        crowd = _lines(held, CONTOUR_STEP * 0.5, np.clip(steep - 1.2, 0.0, 0.4))
        return np.maximum(coverage, np.where(land, crowd * 0.5, 0.0))

    def _colour_wash(
        self,
        page: np.ndarray,
        hue: np.ndarray,
        flow: np.ndarray,
        drawn: np.ndarray,
        rows: int,
        cols: int,
    ) -> np.ndarray:
        """Lay the overlay on the ground as a wash of water colour.

        **The wash is a glaze and not a coat.** The page multiplies the paper
        by the colour rather than replacing it, so the grain of the paper and
        the marks under the wash both stay visible. That is what a wash of
        water colour does to a sheet.

        Three things make it read as paint rather than as a tint.

        The grain of the paper carries the pigment. Pigment settles in the
        tooth of a sheet, so the wash is stronger where the sheet is rough.

        The edge of a wash dries darker than the middle. The page finds that
        edge as the difference between a near view and a far view of the same
        wash, and it deepens the colour there.

        The wash follows the ground. It came through the same lift as the
        ground, so it drapes over a hill instead of lying flat across it.
        """
        if not flow.any():
            return page
        grain = self._paper_grain(rows, cols)
        settled = flow * (1.0 - GRANULATION + GRANULATION * 2.0 * grain)
        rim = np.clip(_smooth(settled, 4) - _smooth(settled, 14), 0.0, 1.0)
        weight = np.clip(settled * WASH_GAIN + rim * RIM_GAIN, 0.0, 1.0) * drawn
        tint = np.clip(hue / 255.0 * GLAZE_LIFT, 0.0, 1.0)
        glazed: np.ndarray = (
            page * (1.0 - weight[..., None]) + page * tint * weight[..., None]
        )
        return glazed

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

    @staticmethod
    def _wash(
        page: np.ndarray,
        colour: np.ndarray,
        drawn: np.ndarray,
        land: np.ndarray,
        tone: np.ndarray,
    ) -> np.ndarray:
        """Lay a light wash of the engine's own colour under the marks.

        The wash keeps a trace of the colour so that a watcher reads who holds
        the ground, and drops the rest so that the ink carries the drawing.
        """
        grey = colour.mean(axis=-1, keepdims=True)
        muted = np.clip((grey + WASH_COLOUR * (colour - grey)) / 255.0, 0.0, 1.0)
        washed = page * (1.0 - WASH_WEIGHT) + page * muted * (WASH_WEIGHT * 2.0)
        page = np.where(drawn[..., None], washed, page)
        return np.where(land[..., None], page * (1.0 - tone * 0.15)[..., None], page)

    @staticmethod
    def _outline(page: np.ndarray, drawn: np.ndarray, cliff: np.ndarray) -> np.ndarray:
        """Put an ink line on every silhouette edge."""
        edge = np.zeros(drawn.shape, dtype=np.float32)
        for axis in (0, 1):
            for shift in (1, -1):
                edge = np.maximum(
                    edge, (drawn ^ np.roll(drawn, shift, axis=axis)).astype(np.float32)
                )
        top = (cliff & ~np.roll(cliff, 1, axis=0)).astype(np.float32)
        edge = np.maximum(edge, top * 0.85)
        weight = (edge * 0.85)[..., None]
        lined: np.ndarray = page * (1.0 - weight) + INK * weight
        return lined

    @staticmethod
    def _sky_over(
        page: np.ndarray,
        cloud: np.ndarray,
        rise: int,
        page_x: np.ndarray,
        page_y: np.ndarray,
        rows: int,
    ) -> np.ndarray:
        """Put the cloud over the ground, as a layer and not as a tint.

        **The cloud is a layer above the ground.** It sits higher on the page
        than the tallest ground, so a mass crosses the paper over a mountain
        and a watcher reads that the mountain stands under it. A tint on the
        ground would say nothing about height.

        **The sky carries the lightest marks on the page.** A sky drawn as
        heavily as the land buries the land.

        The hatch of the cloud follows the edge of the mass, because the phase
        of the lines is the cloud share itself. A line therefore runs along a
        line of constant cloud, and a watcher reads the shape of a mass.
        """
        page_rows = page.shape[0]
        lifted = np.zeros((page_rows, page.shape[1]), dtype=np.float32)
        top = max(rise + 4 - int(rise * CLOUD_HEIGHT), 0)
        lifted[top : top + rows] = cloud[: page_rows - top]
        thick = np.clip((lifted - CLOUD_FLOOR) / (1.0 - CLOUD_FLOOR), 0.0, 1.0)
        along = _lines(lifted * CLOUD_CONTOURS, 1.0, thick * 0.40)
        veil = _lines(page_x * 0.94 + page_y * 0.34, CLOUD_SPACING, thick * 0.30)
        marks = np.maximum(along * 0.50, veil * 0.34)[..., None]
        clouded: np.ndarray = page * (1.0 - marks) + SKY_INK * marks
        return clouded

    def _fit(self, page: np.ndarray, width: int, height: int) -> np.ndarray:
        """Put the page into the frame, whole, on paper, and pack the bytes.

        The page is wider and shorter than the frame it goes into, because the
        turn spreads the plan across the paper. It is scaled to fit rather
        than cropped, so a watcher sees the whole world the camera covers.
        """
        page = _trim(page)
        rows, cols = page.shape[:2]
        scale = min(width / cols, height / rows)
        fit_w = max(int(cols * scale), 1)
        fit_h = max(int(rows * scale), 1)
        take_x = np.clip((np.arange(fit_w) / scale).astype(np.int32), 0, cols - 1)
        take_y = np.clip((np.arange(fit_h) / scale).astype(np.int32), 0, rows - 1)
        frame = np.broadcast_to(PAPER * 0.98, (height, width, 3)).copy()
        left = (width - fit_w) // 2
        top = (height - fit_h) // 2
        frame[top : top + fit_h, left : left + fit_w] = page[take_y][:, take_x]
        packed: Any = frame.astype(np.uint32)
        bytes_of: np.ndarray = (
            (packed[..., 0] << 16) | (packed[..., 1] << 8) | packed[..., 2]
        ).astype(np.uint32)
        return bytes_of
