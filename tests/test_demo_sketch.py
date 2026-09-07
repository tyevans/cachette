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
from cachette.demo.sketch import BoundaryGap, Sketch
from cachette.demo.surface import Surface
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
    demo.renderer = Sketch(world, seed=SEED)
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
    sketch = Sketch(world, seed=SEED)
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
    drawn = frame_of(Sketch(world, seed=SEED), world, camera).reshape(HEIGHT, WIDTH)

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
    sketch = Sketch(world, seed=SEED)
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


def test_the_sketch_refuses_an_engine_that_publishes_no_height(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """The renderer reads the world through the overlays the engine names."""
    world, _ = build()
    monkeypatch.setattr(ink, "HEIGHT_OVERLAY", "no overlay carries this name")
    with pytest.raises(BoundaryGap):
        Sketch(world, seed=SEED)


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
