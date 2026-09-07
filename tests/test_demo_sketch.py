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
from cachette.demo.mouse import RIGHT_BUTTON, Controls
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
