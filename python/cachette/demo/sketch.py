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

The cloud hatch runs with the wind, because the phase of its lines is the
distance across the wind of the tile under them. Its weight grows with the
cloud share. A watcher therefore reads the circulation and the cover from one
set of marks. The sky carries the lightest marks on the page, so the land
holds the weight.

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

from cachette.demo.view import View

if TYPE_CHECKING:
    from collections.abc import Sequence

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

# How many tiles of margin the window takes around the tiles the camera names.
#
# The rows of a hex grid interlock, so a rectangle of pixels covers a slanted
# band of tiles. The margin keeps the corners of that band inside the window.
WINDOW_MARGIN = 3

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
CLOUD_SPACING = 8.0

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
CLOUD_SHADOW_DEPTH = 0.16

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

# How much bare paper stays around the ground, in pixels.
MARGIN = 24

# How far the mask of the panels grows before the page goes under it, in
# pixels. A letter of a panel that matched the map under it would speckle the
# card without this.
PANEL_GROWTH = 2

# How far apart the pixels stand that the wash asks the engine to name, in
# pixels. A tile of the demonstration world covers several pixels, so a coarse
# grid still names every tile the camera shows.
SAMPLE_STEP = 3


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


def _grown(mask: np.ndarray, radius: int) -> np.ndarray:
    """Grow a mask by a short reach, so that a hole inside it closes."""
    wide = mask.copy()
    for step in range(1, radius + 1):
        wide[:, step:] |= mask[:, :-step]
        wide[:, :-step] |= mask[:, step:]
        wide[step:, :] |= mask[:-step, :]
        wide[:-step, :] |= mask[step:, :]
    return wide


def _box(marked: np.ndarray) -> tuple[int, int, int, int]:
    """Give back the part of a page that holds the ground, with a margin.

    The turn spreads the world across a rectangle, and the world fills the
    diamond inside it. The corners hold nothing, so a page that kept them
    would show the world small in the middle of the frame.

    **The ground sets the edge, and not the marks.** The sky reaches past the
    ground, and a page cut to the sky would show the land small. A sketch lets
    the sky run off the sheet.
    """
    rows = np.nonzero(marked.any(axis=1))[0]
    cols = np.nonzero(marked.any(axis=0))[0]
    if not rows.size or not cols.size:
        return 0, marked.shape[0], 0, marked.shape[1]
    return (
        max(int(rows[0]) - MARGIN, 0),
        min(int(rows[-1]) + MARGIN + 1, marked.shape[0]),
        max(int(cols[0]) - MARGIN, 0),
        min(int(cols[-1]) + MARGIN + 1, marked.shape[1]),
    )


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
        "drawn",
        "held",
        "outline",
        "page_x",
        "page_y",
        "sample",
        "shape",
        "sheet",
        "stand",
        "take",
        "tone",
        "water",
        "window",
    )

    # The window of tiles this page covers, the size of the frame it was built
    # for, and the two angles the view stood at. The rest is what the build
    # put on the page.
    window: tuple[int, int, int, int]
    shape: tuple[int, int]
    stand: tuple[float, float]
    sample: tuple[np.ndarray, np.ndarray] | None
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

    def __init__(
        self,
        window: tuple[int, int, int, int],
        shape: tuple[int, int],
        stand: tuple[float, float],
    ) -> None:
        """Record which window, which size and which angles built this page."""
        self.window = window
        self.shape = shape
        self.stand = stand
        # Where the engine drew each tile of the window on its own flat map.
        # It is built only when a wash needs it.
        self.sample = None


class Sketch:
    """A renderer that draws the world as a pencil study.

    A caller builds one of these from the world, then calls it exactly as it
    calls the drawing method of the world. The call fills the pixels it is
    given and gives back what the drawing pass read.
    """

    __slots__ = (
        "_grain",
        "_grain_size",
        "_ground",
        "_heights",
        "_kinds",
        "_level",
        "_relief",
        "_scratch",
        "_sky",
        "_water",
        "_whole_sky",
        "_world",
        "view",
    )

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
        for name in ("tile_heights", "tile_kinds", "cloud_shares", "tile_winds"):
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
        self._ground: Ground | None = None
        self._scratch: dict[str, npt.NDArray[np.uint32]] = {}
        self._grain: np.ndarray | None = None
        self._grain_size = (0, 0)

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
        page = self._draw(ground, overlay, camera, width, height)
        kept = self._panels(pixels, camera, width, height, overlay, phase)
        frame = pixels.reshape(height, width)
        frame[...] = np.where(kept, frame, self._fit(page, ground.box, width, height))
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
        """Give back the tiles the camera covers, as a window of the world.

        **The engine answers which tile a pixel shows.** The rows of a hex
        grid interlock, so a rectangle of pixels covers a slanted band of
        tiles, and the corners of the frame therefore name the band. A margin
        keeps the whole band inside the window.
        """
        corners = [
            camera.tile_at(float(x), float(y))
            for x in (0.0, width / 2.0, width - 1.0)
            for y in (0.0, height / 2.0, height - 1.0)
        ]
        columns = [place[0] for place in corners]
        rows = [place[1] for place in corners]
        first_q = max(min(columns) - WINDOW_MARGIN, 0)
        last_q = min(max(columns) + WINDOW_MARGIN + 1, self._world.width)
        first_r = max(min(rows) - WINDOW_MARGIN, 0)
        last_r = min(max(rows) + WINDOW_MARGIN + 1, self._world.height)
        return first_q, first_r, max(last_q, first_q + 1), max(last_r, first_r + 1)

    def _for(self, camera: Camera, width: int, height: int) -> Ground:
        """Give back the page of the ground, building it if the view moved.

        **The page follows the two angles as well as the window.** A turn or a
        lean moves every point of the page, so a page built at one pair of
        angles says nothing about another pair.
        """
        window = self._window(camera, width, height)
        stand = (self.view.turn, self.view.lean)
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
        stand: tuple[float, float],
    ) -> Ground:
        """Turn the window of tiles into a page, and hatch the ground on it.

        The pass reads the world backwards. For each point of the turned page
        it names the tile that stands there, so the page holds no hole and
        nothing is scattered.
        """
        ground = Ground(window, (height, width), stand)
        first_q, first_r, last_q, last_r = window
        heights = self._heights[first_r:last_r, first_q:last_q]
        water = self._water[first_r:last_r, first_q:last_q]
        # The land starts at the level of the water. The water lies flat.
        land = np.clip(heights - self._level, 0.0, None) / max(1.0 - self._level, 1e-3)
        deep = np.clip(1.0 - heights / max(self._level, 1e-3), 0.0, 1.0)
        raised = _smooth(np.where(water, 0.0, land), SMOOTHING)
        raised = np.where(water, 0.0, raised).astype(np.float32)

        take, held = self._turn(window, width, height)
        ground.held = held
        plan = np.where(held, raised.ravel()[take], 0.0).astype(np.float32)
        wet = held & (water.ravel()[take])
        depth = np.where(held, deep.ravel()[take], 0.0).astype(np.float32)

        cols = plan.shape[1]
        rise = max(int(cols * self._relief), 1)
        drawn, cliff, lifted = self._lift(plan, held, rise)
        ground.take = np.where(drawn, take.ravel()[lifted], -1)
        ground.drawn = drawn
        ground.cliff = cliff
        ground.water = drawn & wet.ravel()[lifted]
        held_height = np.where(drawn, plan.ravel()[lifted], 0.0).astype(np.float32)
        depth_here = np.where(drawn, depth.ravel()[lifted], 0.0).astype(np.float32)
        page_rows, page_cols = drawn.shape
        ground.page_x = np.broadcast_to(
            np.arange(page_cols, dtype=np.float32), (page_rows, page_cols)
        )
        ground.page_y = np.broadcast_to(
            np.arange(page_rows, dtype=np.float32)[:, None], (page_rows, page_cols)
        )
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
        ground.box = _box(drawn)
        return ground

    def _turn(
        self, window: tuple[int, int, int, int], width: int, height: int
    ) -> tuple[np.ndarray, np.ndarray]:
        """Turn the window about the up direction and lean it away from the page.

        A world seen square to the page reads as a flat map however far the
        ground is lifted, so the page turns before it lifts.

        **The two angles come from the view.** The turn is how far the ground
        is rotated about the up direction, and the lean is how far the page
        tips away from the watcher. The view opens at an eighth of a circle
        and a lean of one half, which is the isometric drawing this page was
        first written as.

        **The pass reads backwards, one tile for each point of the page.** A
        pass that scattered the tiles forward would leave a hole between two
        of them, and a page of holes is a page of dust.

        Returns the tile of the window that stands at each point of the turned
        page, as an index into the window, and the mask of the points that
        hold one.
        """
        first_q, first_r, last_q, last_r = window
        across = last_q - first_q
        down = last_r - first_r
        # The window in the plan of the world, before the turn. A row of a hex
        # grid steps half a column across and less than a row down.
        plan_wide = across + down * 0.5
        plan_tall = down * ROW_PITCH
        lean = self.view.lean
        turn_x, turn_y = math.cos(self.view.turn), math.sin(self.view.turn)
        # How far the turned plan reaches across the page and down it. A
        # rectangle turned through an angle covers this much of each
        # direction, and the page is cut to fit it.
        span_x = plan_wide * abs(turn_x) + plan_tall * abs(turn_y)
        span_y = plan_wide * abs(turn_y) + plan_tall * abs(turn_x)
        scale = min(
            width / max(span_x, 1e-3),
            height / max(span_y * lean, 1e-3),
        )
        page_cols = max(int(span_x * scale) + 2, 2)
        page_rows = max(int(span_y * scale * lean) + 2, 2)
        column = (np.arange(page_cols, dtype=np.float32) - page_cols / 2.0)[None, :]
        row = ((np.arange(page_rows, dtype=np.float32) - page_rows / 2.0) / lean)[
            :, None
        ]
        # Turn the point of the page back into the plan of the world. The lean
        # is already taken out of the row above, so this is a plain rotation.
        plan_x = (column * turn_x + row * turn_y) / scale + plan_wide / 2.0
        plan_y = (row * turn_x - column * turn_y) / scale + plan_tall / 2.0
        tile_r = plan_y / ROW_PITCH
        tile_q = plan_x - tile_r * 0.5
        take_r = np.rint(tile_r).astype(np.int32)
        take_q = np.rint(tile_q).astype(np.int32)
        held = (take_q >= 0) & (take_q < across) & (take_r >= 0) & (take_r < down)
        take = np.clip(take_r, 0, down - 1) * across + np.clip(take_q, 0, across - 1)
        return take.astype(np.int32), held

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
        page_column = np.broadcast_to(np.arange(cols, dtype=np.int32), owner.shape)
        take = np.where(highest >= 0, owner[np.clip(highest, 0, None), page_column], -1)
        foot = np.clip(take, 0, None) // cols + rise + 4
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
    ) -> np.ndarray:
        """Put the layers that change over the ground the build drew.

        The ground is a copy. The cloud, the wind, the faction that holds each
        tile and a named overlay all change with the world, so they are read
        and drawn here.
        """
        page = ground.sheet.copy()
        page = self._holders(page, ground)
        rise = max(int(ground.drawn.shape[1] * self._relief), 1)
        if self._sky:
            page = self._sky_over(page, ground, rise)
        page = self._wash(page, ground, overlay, camera, width, height)
        page = (
            page * (1.0 - ground.coverage[..., None]) + INK * ground.coverage[..., None]
        )
        edge = ground.outline[..., None]
        inked: np.ndarray = np.clip(page * (1.0 - edge) + INK * edge, 0.0, 255.0)
        return inked

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
        colour. It asks the engine for the frame with the overlay and for the
        frame without it, and the difference between the two is the pigment
        the overlay laid down.

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
        bare = self._buffer("bare", width, height)
        tinted = self._buffer("tinted", width, height)
        self._world.draw(camera, width, height, bare)
        self._world.draw(camera, width, height, tinted, overlay=overlay)
        plain = _channels(bare, width, height)
        painted = _channels(tinted, width, height)
        # The wash follows the ground, so it is read at the tile the page
        # shows and not at the pixel the flat map put it on.
        flow = self._by_tile(
            ground,
            np.abs(painted - plain).max(axis=-1) / 255.0,
            camera,
            width,
            height,
        )
        hue = np.stack(
            [
                self._by_tile(ground, painted[..., band], camera, width, height)
                for band in range(3)
            ],
            axis=-1,
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

    def _by_tile(
        self, ground: Ground, frame: np.ndarray, camera: Camera, width: int, height: int
    ) -> np.ndarray:
        """Read a field the engine painted on the flat map, tile by tile.

        The engine paints an overlay on its own map. The page shows the same
        tiles in another place, so the reading finds where the engine drew
        each tile and then puts the value where the page shows that tile.

        **The engine answers where a tile is.** The pass asks it over a coarse
        grid of pixels and takes the middle of the pixels that named each
        tile, so this module holds no layout of its own for the flat map.
        """
        first_q, first_r, last_q, last_r = ground.window
        across = last_q - first_q
        down = last_r - first_r
        if ground.sample is None:
            columns = np.arange(0, width, SAMPLE_STEP)
            rows = np.arange(0, height, SAMPLE_STEP)
            found_q = np.zeros((down, across), dtype=np.int64)
            found_y = np.zeros((down, across), dtype=np.int64)
            counted = np.zeros((down, across), dtype=np.int64)
            for y in rows:
                for x in columns:
                    tile_q, tile_r = camera.tile_at(float(x), float(y))
                    at_q = tile_q - first_q
                    at_r = tile_r - first_r
                    if 0 <= at_q < across and 0 <= at_r < down:
                        found_q[at_r, at_q] += x
                        found_y[at_r, at_q] += y
                        counted[at_r, at_q] += 1
            safe = np.clip(counted, 1, None)
            ground.sample = (
                np.clip(found_q // safe, 0, width - 1).astype(np.int32),
                np.clip(found_y // safe, 0, height - 1).astype(np.int32),
            )
        at_x, at_y = ground.sample
        window = frame[at_y, at_x]
        whole = np.zeros((self._world.height, self._world.width), dtype=window.dtype)
        whole[first_r:last_r, first_q:last_q] = window
        return self._gather(ground, whole)

    def _sky_over(self, page: np.ndarray, ground: Ground, rise: int) -> np.ndarray:
        """Put the cloud over the ground, as a layer and not as a tint.

        **The cloud is a layer above the ground.** It sits higher on the page
        than the tallest ground, so a mass crosses the paper over a mountain
        and a watcher reads that the mountain stands under it. A tint on the
        ground would say nothing about height.

        **The hatch runs with the wind.** The phase of its lines is the
        distance across the wind, so a line runs along the wind and a watcher
        reads the circulation. The weight of the lines is the cloud.

        **The sky carries the lightest marks on the page.** A sky drawn as
        heavily as the land buries the land.
        """
        rows, columns = self._world.height, self._world.width
        cloud = self._world.cloud_shares().reshape(rows, columns).astype(
            np.float32
        ) / max(self._whole_sky, 1.0)
        winds = self._world.tile_winds()
        wind_q = winds["q"].reshape(rows, columns).astype(np.float32)
        wind_r = winds["r"].reshape(rows, columns).astype(np.float32)
        # The wind stands on the axes of the hex grid. The page turns those
        # axes, so the wind turns with them. This is the forward turn, and the
        # page is built from the backward one, so the two use one pair of
        # angles and cannot part company.
        plan_x = wind_q + wind_r * 0.5
        plan_y = wind_r * ROW_PITCH
        along_turn = math.cos(self.view.turn)
        across_turn = math.sin(self.view.turn)
        page_dx = plan_x * along_turn - plan_y * across_turn
        page_dy = (plan_x * across_turn + plan_y * along_turn) * self.view.lean
        length = np.hypot(page_dx, page_dy)
        moving = length > 0.0
        page_dx = np.where(moving, page_dx / np.where(moving, length, 1.0), 1.0)
        page_dy = np.where(moving, page_dy / np.where(moving, length, 1.0), 0.0)

        share = self._gather(ground, cloud)
        across_x = self._gather(ground, page_dy)
        across_y = self._gather(ground, -page_dx)
        step = max(int(ground.drawn.shape[1] * CLOUD_SHADOW_STEP), 1)
        under = np.clip((np.roll(share, step, axis=1) - CLOUD_FLOOR), 0.0, 1.0)
        page = page * (1.0 - under[..., None] * CLOUD_SHADOW_DEPTH)

        lift = int(rise * CLOUD_HEIGHT)
        above = np.roll(share, -lift, axis=0)
        turn_x = np.roll(across_x, -lift, axis=0)
        turn_y = np.roll(across_y, -lift, axis=0)
        thick = np.clip((above - CLOUD_FLOOR) / (1.0 - CLOUD_FLOOR), 0.0, 1.0)
        # The distance across the wind. A line of constant phase runs along
        # the wind, so the hatch runs with the circulation.
        phase = ground.page_x * turn_x + ground.page_y * turn_y
        along = _lines(phase, CLOUD_SPACING, thick * 0.42)
        marks = (along * 0.50)[..., None]
        clouded: np.ndarray = page * (1.0 - marks) + SKY_INK * marks
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
        width: int,
        height: int,
    ) -> npt.NDArray[np.uint32]:
        """Put the page into the frame, whole, on paper, and pack the bytes.

        The page is wider and shorter than the frame it goes into, because the
        turn spreads the world across the paper. It is scaled to fit rather
        than cropped, so a watcher sees the whole world the camera covers.
        """
        first_row, last_row, first_col, last_col = box
        page = page[first_row:last_row, first_col:last_col]
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


def _channels(pixels: npt.NDArray[np.uint32], width: int, height: int) -> np.ndarray:
    """Split one frame into red, green and blue, as three planes."""
    frame = pixels.reshape(height, width)
    split: np.ndarray = np.stack(
        [(frame >> 16) & 0xFF, (frame >> 8) & 0xFF, frame & 0xFF], axis=-1
    ).astype(np.float32)
    return split
