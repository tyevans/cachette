"""The sketch renderer, and the flag that puts it at the frame.

The demonstration fills one frame through one call. These tests drive that
call, and the flag that chooses which renderer answers it.
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import Camera, World
from cachette.demo import sketch as ink
from cachette.demo.app import Demo, main
from cachette.demo.mouse import LEFT_BUTTON, RIGHT_BUTTON, Controls
from cachette.demo.sketch import BoundaryGap, Sketch
from cachette.demo.surface import Surface
from cachette.demo.view import View
from cachette.names import Names

# A small world and a small frame. The sketch reads the frame and not the
# tiles, so its cost follows the frame, and a small frame keeps a test quick.
SIDE = 48
WIDTH = 240
HEIGHT = 180
SEED = 0x0123_4567_89AB_CDEF

# The share of a frame that may be the colour of the paper and still be a map
# of the world. The engine paints no paper, so a map stays far under this and
# a sketch stands far over it.
PAPER_SHARE_OF_A_MAP = 0.10

# How dark a pixel must be before the test counts it as a mark, and how far
# the top of the drawing must move between one column and the next before the
# test accepts that the page carries height.
MARK_LIGHT = 200
SKYLINE_RANGE = 4.0


def build() -> tuple[World, Camera]:
    """Give back a seeded world and a camera that fits it."""
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=2)
    world.seed_world()
    for _ in range(8):
        world.step(1)
    return world, Camera.fitting(world, WIDTH, HEIGHT)


def frame_of(renderer: object, world: World, camera: Camera) -> np.ndarray:
    """Fill one frame with this renderer and give back a copy of the pixels."""
    surface = Surface(WIDTH, HEIGHT)
    renderer(camera, WIDTH, HEIGHT, surface.pixels)  # type: ignore[operator]
    return surface.pixels.copy()


def paper_share(pixels: np.ndarray) -> float:
    """Give back the share of the frame that is close to the paper colour."""
    red = (pixels >> 16) & 0xFF
    green = (pixels >> 8) & 0xFF
    blue = pixels & 0xFF
    near = (
        (np.abs(red.astype(np.int32) - int(ink.PAPER[0])) < 24)
        & (np.abs(green.astype(np.int32) - int(ink.PAPER[1])) < 24)
        & (np.abs(blue.astype(np.int32) - int(ink.PAPER[2])) < 24)
    )
    return float(near.mean())


def test_the_demonstration_opens_on_the_engine_renderer() -> None:
    """A watcher who asks for nothing gets the map the engine draws.

    The frame the engine draws is a map of the world. The sketch draws on
    paper, so the share of the frame that is paper tells the two apart
    without naming a class.
    """
    world, camera = build()
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.camera = camera
    demo.advance()
    assert not isinstance(demo.renderer, Sketch)
    assert paper_share(demo.surface.pixels) < PAPER_SHARE_OF_A_MAP


def test_the_sketch_answers_the_frame_when_a_caller_puts_it_there() -> None:
    """The renderer takes the place of the engine at the one drawing call."""
    world, camera = build()
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.camera = camera
    demo.clock.pause()
    demo.renderer = Sketch(world)
    reading = demo.advance()
    engine = frame_of(world.draw, world, camera)
    assert not np.array_equal(demo.surface.pixels, engine)
    # The reading is the engine's own, so the panels and the console read the
    # same numbers under either renderer.
    assert reading["tick"] == world.tick
    assert paper_share(demo.surface.pixels) > paper_share(engine)


def test_the_sketch_writes_no_value_of_the_world() -> None:
    """The renderer reads the world. It never writes to it."""
    world, camera = build()
    before = world.state_hash
    tick = world.tick
    sketch = Sketch(world)
    frame_of(sketch, world, camera)
    assert world.state_hash == before
    assert world.tick == tick


def test_the_sketch_lifts_the_ground_off_the_flat_map() -> None:
    """The height of the ground moves the marks up the page.

    The engine paints the world into a rectangle, so the top of its map is a
    straight line across the frame. The sketch lifts every point by the height
    under it, so the top of its drawing follows the hills.

    The test measures how far the topmost mark of each column moves. A flat
    map moves it not at all.
    """
    world, camera = build()
    engine = frame_of(world.draw, world, camera).reshape(HEIGHT, WIDTH)
    drawn = frame_of(Sketch(world), world, camera).reshape(HEIGHT, WIDTH)

    def top_edge(frame: np.ndarray) -> np.ndarray:
        marked = (frame & 0xFF) < MARK_LIGHT
        rows = [
            int(np.nonzero(marked[:, column])[0][0])
            for column in range(WIDTH)
            if marked[:, column].any()
        ]
        return np.array(rows, dtype=np.float64)

    assert top_edge(engine).std() < 1.0
    assert top_edge(drawn).std() > SKYLINE_RANGE


def test_the_sketch_paints_a_named_overlay_as_a_wash_of_its_own_colour() -> None:
    """An overlay reaches the page as colour, and no overlay leaves it grey."""
    world, camera = build()
    sketch = Sketch(world)
    surface = Surface(WIDTH, HEIGHT)

    def colour_spread(pixels: np.ndarray) -> float:
        red = ((pixels >> 16) & 0xFF).astype(np.int32)
        green = ((pixels >> 8) & 0xFF).astype(np.int32)
        blue = (pixels & 0xFF).astype(np.int32)
        stacked = np.stack([red, green, blue], axis=-1)
        return float((stacked.max(axis=-1) - stacked.min(axis=-1)).mean())

    sketch(camera, WIDTH, HEIGHT, surface.pixels)
    plain = colour_spread(surface.pixels)
    sketch(camera, WIDTH, HEIGHT, surface.pixels, overlay="height")
    washed = colour_spread(surface.pixels)
    assert washed > plain


class Silent:
    """A world that publishes none of the readers the renderer needs."""


def test_the_sketch_refuses_an_engine_that_publishes_no_bulk_reader() -> None:
    """The renderer reads the world through the readers the engine publishes."""
    with pytest.raises(BoundaryGap):
        Sketch(Silent())  # type: ignore[arg-type]


def test_water_is_the_kind_the_renderer_takes_it_for() -> None:
    """The page reads one kind number as water, and the engine numbers them.

    **The number is a second copy of a fact the engine holds.** This is the
    check that fails when the two disagree. Water lies under every other kind,
    so the kind the page calls water must report the lowest ground of all.
    """
    world, _ = build()
    heights = world.tile_heights().astype(np.float64)
    kinds = world.tile_kinds()
    present = sorted(set(kinds.tolist()))
    means = [heights[kinds == kind].mean() for kind in present]
    assert means[0] == min(means)
    assert ink.WATER_KIND == present[0]


def test_the_page_lifts_the_ground_the_engine_reports() -> None:
    """The window the page builds holds the tallest ground of the world.

    A page built from a field of its own would still draw hills, so the test
    asks where the engine reports the tallest ground and then asks whether the
    page covers that place.
    """
    world, camera = build()
    sketch = Sketch(world)
    frame_of(sketch, world, camera)
    heights = world.tile_heights()
    highest = int(np.argmax(heights))
    window = sketch.window()
    assert window is not None
    first_q, first_r, last_q, last_r = window
    assert first_r <= highest // world.width < last_r
    assert first_q <= highest % world.width < last_q


def test_the_flag_chooses_the_sketch_and_the_default_does_not(
    tmp_path: object,
) -> None:
    """The flag selects the renderer, and the run without it draws the map.

    The test drives the command line, because the flag is the thing under
    test and a caller that built the renderer itself would not exercise it.
    """
    plain = f"{tmp_path}/plain.png"
    drawn = f"{tmp_path}/sketch.png"
    shared = [
        "--picture",
        "",
        "--ticks",
        "2",
        "--extent",
        str(SIDE),
        "--width",
        str(WIDTH),
        "--height",
        str(HEIGHT),
        "--panels",
        "statistics",
        "--seed",
        hex(SEED),
    ]
    shared[1] = plain
    assert main(list(shared)) == 0
    shared[1] = drawn
    assert main([*shared, "--sketch"]) == 0
    first = Surface(WIDTH, HEIGHT)
    second = Surface(WIDTH, HEIGHT)
    read_png(plain, first)
    read_png(drawn, second)
    assert not np.array_equal(first.pixels, second.pixels)
    assert paper_share(second.pixels) > paper_share(first.pixels)


def read_png(path: str, into: Surface) -> None:
    """Read a picture this package wrote back into a surface."""
    import struct
    import zlib

    with open(path, "rb") as source:
        data = source.read()
    at = 8
    width = height = 0
    body = b""
    while at < len(data):
        length = struct.unpack(">I", data[at : at + 4])[0]
        kind = data[at + 4 : at + 8]
        chunk = data[at + 8 : at + 8 + length]
        if kind == b"IHDR":
            width, height = struct.unpack(">2I", chunk[:8])
        elif kind == b"IDAT":
            body += chunk
        at += length + 12
    raw = np.frombuffer(zlib.decompress(body), dtype=np.uint8)
    rows = raw.reshape(height, width * 3 + 1)[:, 1:].reshape(height, width, 3)
    packed = rows.astype(np.uint32)
    into.pixels[:] = (
        (packed[..., 0] << 16) | (packed[..., 1] << 8) | packed[..., 2]
    ).ravel()


def test_a_mouse_drag_turns_and_leans_the_page_the_sketch_draws() -> None:
    """The sketch reads the view, so the mouse reaches the page.

    **This drives the handler the window library calls.** A test that set the
    angle on the renderer would prove that the renderer can turn. It would not
    prove that a drag reaches it.
    """
    world, camera = build()
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.camera = camera
    demo.clock.pause()
    demo.renderer = Sketch(world, view=demo.view)
    demo.advance()
    was = demo.surface.pixels.copy()
    stood = demo.view.turn, demo.view.lean

    controls = Controls(demo)
    controls.on_mouse_press(200, 120, RIGHT_BUTTON, 0)
    controls.on_mouse_drag(260, 90, 60, -30, RIGHT_BUTTON, 0)
    controls.on_mouse_release(260, 90, RIGHT_BUTTON, 0)
    assert (demo.view.turn, demo.view.lean) != stood

    demo.advance()
    assert not np.array_equal(demo.surface.pixels, was)


def test_the_page_holds_no_second_copy_of_the_angles_it_stands_at() -> None:
    """One object says where the watcher stands.

    A page built at one pair of angles says nothing about another pair, so the
    ground it holds is rebuilt when either angle moves.
    """
    world, camera = build()
    view = View()
    sketch = Sketch(world, view=view)
    frame_of(sketch, world, camera)
    first = sketch._ground
    assert first is not None
    frame_of(sketch, world, camera)
    assert sketch._ground is first, "a still view rebuilt the ground"

    view.tilt_by(0.2)
    frame_of(sketch, world, camera)
    assert sketch._ground is not first, "a lean did not rebuild the ground"


def test_the_lean_changes_how_far_the_page_reaches_down_the_frame() -> None:
    """A view near the ground draws a short page. A plan draws a tall one."""
    world, camera = build()
    view = View()
    sketch = Sketch(world, view=view)
    heights = []
    for lean in (0.15, 0.5, 1.0):
        view.lean = lean
        frame_of(sketch, world, camera)
        ground = sketch._ground
        assert ground is not None
        heights.append(ground.drawn.shape[0])
    assert heights[0] < heights[1] < heights[2]


# ----------------------------------------------------------------------
# The wash of an overlay against the ground it colours.
#
# **The colour and the ink are one page, and a zoom must not part them.** The
# engine paints an overlay on its own flat map. The sketch reads that paint
# tile by tile, at the pixel the engine drew each tile on. That pixel is a
# function of the camera and of the size of the frame, and of nothing else.
#
# A reading held against the window of whole tiles instead is reused at every
# camera that covers the same whole tiles. A zoom moves the camera in small
# steps and the window in whole tiles, so between two steps of the window each
# tile takes the paint of a neighbour, and the colour walks across the ink.
# The register holds what this cost.[^1]
#
# [^1]: Findings register, FND-610. `docs/FINDINGS.md`

# The overlay these tests colour the world with. It paints a field that
# changes from tile to tile, so a reading that slips by one tile reads a
# different colour.
DRIFT_OVERLAY = "height"

# The zoom steps the tests run. **Two of these hold the window of whole tiles
# still.** A test that only compared two frames at one zoom, or at two zooms
# far enough apart to move the window, would pass while the colour slid.
DRIFT_ZOOMS = (1.0, 1.06, 1.13, 1.9, 3.1)


def zoomed(world: World, factor: float) -> Camera:
    """Give back a camera at this factor of the size that fits the world."""
    fitted = Camera.fitting(world, WIDTH, HEIGHT)
    camera = Camera(tile_size=fitted.tile_width * factor)
    camera.origin_x = fitted.origin_x
    camera.origin_y = fitted.origin_y
    camera.clamp(world, WIDTH, HEIGHT)
    return camera


def washed(renderer: object, camera: Camera) -> np.ndarray:
    """Fill one frame with an overlay laid over the sketch."""
    surface = Surface(WIDTH, HEIGHT)
    renderer(  # type: ignore[operator]
        camera, WIDTH, HEIGHT, surface.pixels, overlay=DRIFT_OVERLAY
    )
    return surface.pixels.copy()


def test_every_tile_takes_its_wash_from_the_pixel_the_engine_drew_it_at() -> None:
    """The reading of a tile stands on that tile, at every zoom.

    **This asks the engine, and it does not repeat the engine.** The renderer
    says which pixel it read a tile at. The engine says which tile stands at
    that pixel. The two must name the same tile, or the tile wears the colour
    of a neighbour.
    """
    world, _ = build()
    sketch = Sketch(world, view=View())
    for factor in DRIFT_ZOOMS:
        camera = zoomed(world, factor)
        washed(sketch, camera)
        at_x, at_y, seen = sketch.tile_pixels(camera, WIDTH, HEIGHT)
        rows, columns = np.nonzero(seen)
        wrong = [
            (int(q), int(r))
            for r, q in zip(rows, columns, strict=True)
            if camera.tile_at(float(at_x[r, q]), float(at_y[r, q])) != (int(q), int(r))
        ]
        assert not wrong, (
            f"at zoom {factor} the sketch read {len(wrong)} of {len(rows)} tiles "
            f"at a pixel that shows another tile, the first being {wrong[0]}"
        )


def test_a_zoom_does_not_walk_the_wash_across_the_ground() -> None:
    """The frame follows the camera it is drawn at, not the one before it.

    **A wash is a property of a tile, so the road to a camera cannot change
    it.** One renderer goes straight to a camera. Another reaches the same
    camera through every zoom step before it. The two frames must agree.

    A reading held against the window of whole tiles fails here, because the
    second renderer keeps the reading it made at the first zoom and every tile
    then wears the colour of a neighbour.
    """
    world, _ = build()
    travelled = Sketch(world, view=View())
    for factor in DRIFT_ZOOMS:
        washed(travelled, zoomed(world, factor))

    for factor in DRIFT_ZOOMS:
        camera = zoomed(world, factor)
        direct = washed(Sketch(world, view=View()), camera)
        after = washed(travelled, camera)
        apart = float((direct != after).mean())
        assert apart == 0.0, (
            f"at zoom {factor} the frame drawn after a zoom differs from the "
            f"frame drawn at that camera alone on {apart:.4%} of its pixels"
        )


# ----------------------------------------------------------------------
# The camera against the page.
#
# **The page holds the whole world, and the camera moves the view of it.** A
# page cut to the tiles the camera covered made the camera choose an extent
# rather than a view. Panning towards the edge of the world then widened the
# extent, and the page drew the world smaller instead of moving it, so a drag
# with the left button did the opposite of what a hand expects.

# How far the test drags, in pixels, and how far the drawing may fall short of
# the hand and still count as following it.
#
# **The drawing follows the hand, one pixel for one pixel, and it does not
# travel sideways.** The page turns the ground and leans it, so the step the
# camera takes is not the step the hand made. The renderer inverts its own
# angles, so the drawing moves the way the hand went. The slack below is for
# the whole page point that each pixel of the frame lands on.
DRAG_ACROSS = 40
DRAG_SLACK = 3

# How much of the change a drag makes must be a move of the drawing.
#
# The correct page scores about 0.47 on the fixture below. The page that was
# cut to the camera scores nothing at all, because its best shift is no shift:
# a drag resized the drawing rather than moving it.
#
# **A drag that resized the drawing leaves no shift that explains it.** The
# page used to be cut to the tiles the camera covered and then fitted to the
# frame, so a drag towards the edge of the world widened the cut and drew the
# world smaller. Under that page the best shift explains almost nothing.
MOVE_EXPLAINS = 0.30


def travel_between(before: np.ndarray, after: np.ndarray, reach: int) -> tuple:
    """Say how far the drawing moved, and how much of the change that explains.

    The answer is the shift across, the shift down, and the share of the
    difference between the two frames that the shift takes away. A positive
    shift across means the drawing moved to the right.
    """
    first = ((before >> 8) & 0xFF).astype(np.int32)
    second = ((after >> 8) & 0xFF).astype(np.int32)
    rows, cols = first.shape
    edge = reach + 1
    still = float(
        np.abs(
            second[edge : rows - edge, edge : cols - edge]
            - first[edge : rows - edge, edge : cols - edge]
        ).mean()
    )
    best, at = None, (0, 0)
    for down in range(-reach, reach + 1):
        for across in range(-reach, reach + 1):
            cut = second[
                edge + down : rows - edge + down, edge + across : cols - edge + across
            ]
            gap = np.abs(cut - first[edge : rows - edge, edge : cols - edge])
            cost = float(gap.mean())
            if best is None or cost < best:
                best, at = cost, (across, down)
    explains = 0.0 if still <= 0.0 else 1.0 - (best or 0.0) / still
    return at[0], at[1], explains


def test_a_left_drag_moves_the_drawing_the_way_the_hand_went() -> None:
    """The sketch follows the hand, and the drawing keeps its size.

    **This asserts on the frame the renderer drew, not on the view.** A test
    that read the camera back after a drag passes while the renderer ignores
    the camera, and that is how a page that resized under every drag reached a
    watcher. The page now holds the whole world, so nothing but the placing of
    the page in the frame can change the drawing.
    """
    world, _ = build()
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.camera = Camera.fitting(world, WIDTH, HEIGHT)
    demo.clock.pause()
    demo.renderer = Sketch(world, view=demo.view)
    # Zoom in first, so that the drag has room before the world meets the edge
    # of the frame.
    demo.view.zoom_at(2.0, WIDTH / 2.0, HEIGHT / 2.0)
    demo.advance()
    before = demo.surface.pixels.reshape(HEIGHT, WIDTH).copy()

    controls = Controls(demo)
    controls.on_mouse_press(WIDTH // 2, HEIGHT // 2, LEFT_BUTTON, 0)
    for step in range(4):
        controls.on_mouse_drag(
            WIDTH // 2 + (step + 1) * (DRAG_ACROSS // 4),
            HEIGHT // 2,
            DRAG_ACROSS // 4,
            0,
            LEFT_BUTTON,
            0,
        )
    controls.on_mouse_release(WIDTH // 2 + DRAG_ACROSS, HEIGHT // 2, LEFT_BUTTON, 0)
    demo.advance()
    after = demo.surface.pixels.reshape(HEIGHT, WIDTH).copy()

    assert not np.array_equal(before, after), "a drag drew the same frame"
    across, down, explains = travel_between(before, after, DRAG_ACROSS)
    assert abs(across - DRAG_ACROSS) <= DRAG_SLACK, (
        f"a drag of {DRAG_ACROSS} pixels to the right moved the drawing "
        f"{across} across; the map must follow the hand"
    )
    assert abs(down) <= DRAG_SLACK, (
        f"a drag straight across moved the drawing {down} down the frame; "
        f"the map must not travel sideways under the hand"
    )
    assert explains >= MOVE_EXPLAINS, (
        f"a shift of ({across}, {down}) takes away only {explains:.2%} of the "
        f"change the drag made, so the drag resized the drawing rather than "
        f"moving it"
    )


def test_the_page_holds_every_tile_of_the_world_at_every_camera() -> None:
    """The sketch draws the whole map, and the camera does not cut it.

    A page cut to the camera made a pan widen the extent. The extent is now
    the world at every camera, so nothing a mouse does can drop a tile.
    """
    world, _ = build()
    sketch = Sketch(world, view=View())
    whole = (0, 0, world.width, world.height)
    for factor in (1.0, 1.7, 3.0):
        camera = zoomed(world, factor)
        frame_of(sketch, world, camera)
        assert sketch.window() == whole, (
            f"at zoom {factor} the page held {sketch.window()} and not {whole}"
        )


# ----------------------------------------------------------------------
# The compass.
#
# **A control nobody finds is a control that does not exist.** The right drag
# and the middle drag turn the page and lean it, and nothing on the frame said
# so. The compass says that the ground turns, and it says how far it is turned
# now, which a line of instructions cannot.

# The corner the compass must stay out of, as a share of the frame. The
# minimap holds the upper right, and the engine puts a card there while the
# reference key is held.
KEPT_CORNER = 0.34

# The frame the compass tests draw in.
#
# **The compass stands down in a frame too small to leave it a margin**, and
# the frame the other tests use is that small. This is the size a watcher
# opens.
ROOM_WIDTH = 640
ROOM_HEIGHT = 480


def in_a_window(world: World) -> Demo:
    """Give back a paused demonstration drawing the sketch in a real window."""
    demo = Demo(world, Names(SEED), width=ROOM_WIDTH, height=ROOM_HEIGHT, threads=1)
    demo.camera = Camera.fitting(world, ROOM_WIDTH, ROOM_HEIGHT)
    demo.clock.pause()
    demo.renderer = Sketch(world, view=demo.view)
    return demo


def test_the_compass_shows_the_turn_the_page_stands_at() -> None:
    """The needle moves when the view turns, and it moves nothing else.

    **This drives the frame the renderer drew.** A test that read the compass
    directly would prove that a compass can be drawn. It would not prove that
    a watcher sees one.
    """
    world, _ = build()
    demo = in_a_window(world)
    demo.advance()
    before = demo.surface.pixels.reshape(ROOM_HEIGHT, ROOM_WIDTH).copy()

    controls = Controls(demo)
    controls.on_mouse_press(320, 240, RIGHT_BUTTON, 0)
    controls.on_mouse_drag(380, 240, 60, 0, RIGHT_BUTTON, 0)
    controls.on_mouse_release(380, 240, RIGHT_BUTTON, 0)
    demo.advance()
    after = demo.surface.pixels.reshape(ROOM_HEIGHT, ROOM_WIDTH).copy()

    corner = (slice(ROOM_HEIGHT - 120, ROOM_HEIGHT), slice(0, 120))
    assert not np.array_equal(before[corner], after[corner]), (
        "the compass drew the same needle after the view turned"
    )


def test_the_compass_keeps_out_of_the_corner_the_minimap_holds() -> None:
    """Two things in one corner means a watcher reads neither."""
    world, _ = build()
    demo = in_a_window(world)
    demo.advance()
    with_compass = demo.surface.pixels.reshape(ROOM_HEIGHT, ROOM_WIDTH).copy()
    demo.compass.visible = False
    demo.advance()
    without = demo.surface.pixels.reshape(ROOM_HEIGHT, ROOM_WIDTH).copy()

    changed = with_compass != without
    assert changed.any(), "the compass painted nothing at all"
    rows, columns = np.nonzero(changed)
    assert rows.min() > ROOM_HEIGHT * (1.0 - KEPT_CORNER), (
        "the compass reaches the upper part of the frame"
    )
    assert columns.max() < ROOM_WIDTH * KEPT_CORNER, (
        "the compass reaches the right of the frame, where the minimap stands"
    )


def test_the_flat_map_carries_no_compass() -> None:
    """The flat map stands square to the ground, so it has no angle to show.

    A compass over it would point at nothing and would name a gesture that
    does nothing.
    """
    world, camera = build()
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.camera = camera
    demo.clock.pause()
    demo.advance()
    shown = demo.surface.pixels.copy()
    demo.compass.visible = False
    demo.advance()
    assert np.array_equal(demo.surface.pixels, shown), "the flat map drew a compass"
