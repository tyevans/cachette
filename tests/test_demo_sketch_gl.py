"""The sketch renderer, composited on the graphics device.

The demonstration fills one frame through one call. These tests drive that
call with the renderer that composites on the graphics device, and they hold
that renderer against the one that composites on the processor.

**The comparison is the test that matters.** The array renderer is the
reference, and it draws the look the owner accepted. A shader that drew
something else would be a redesign of the look, and every other test here
would still pass. This one would not.

Every test skips when the machine gives no graphics context. A machine with no
context cannot run them, and a test that passed by drawing nothing would say
that the shader works when nothing ran it.
"""

from __future__ import annotations

import os

import numpy as np
import pytest

from cachette import Camera, World
from cachette.demo import sketch as ink
from cachette.demo import sketch_gl
from cachette.demo.app import Demo, main
from cachette.demo.glpage import SCRATCH_UNIT, Device, DeviceGap
from cachette.demo.sketch import Sketch
from cachette.demo.sketch_gl import GlSketch
from cachette.demo.surface import Surface
from cachette.demo.view import View
from cachette.names import Names

# A world and a frame large enough to hold every case the page draws: water,
# cliffs, deep shade and ground that more than one faction holds.
#
# **A small frame hides a difference.** The page is fitted to the frame, so a
# frame of a few hundred pixels samples the page coarsely and a shader that
# drew the hatch wrongly could still land on the same pixels. This is the size
# a watcher opens.
SIDE = 128
WIDTH = 640
HEIGHT = 460
SEED = 0x0123_4567_89AB_CDEF

# How far the two renderers may differ, as a band value of nought to 255.
#
# **The two do not agree bit for bit, and they cannot.** They evaluate the
# same expressions on different hardware. The device may contract a multiply
# and an add into one operation that rounds once instead of twice, and it may
# hold a value at a different width along the way. Either changes the last bit
# of a real number, and a value that lands on a whole number from one side
# rather than the other then packs to the neighbouring byte.
#
# One band value is therefore the expected disagreement. This allows two, so
# that a value which crosses two boundaries in one chain of stages does not
# fail the test. It does not allow a visible difference: two band values in
# 255 is under one percent of the range, and no watcher sees it.
TOLERANCE = 2

# The share of pixels that must agree exactly. The rest may differ by up to
# the tolerance above. This is measured, not chosen: the two renderers agree
# exactly on about 99.8 percent of a frame.
EXACT_SHARE = 0.99

# The share of pixels that may differ by more than the tolerance.
#
# **The page holds one place where the drawing itself is a step.** The line
# function divides the room left inside a line by the weight of that line. At
# the edge of a cloud the weight falls to nearly nothing, and the function
# then turns a whole line on or off across a change of one part in ten
# million. The two renderers hold the direction of the wind at different
# widths, so the phase of the cloud hatch differs in its last bit, and at that
# edge the last bit decides a whole line.
#
# This was measured rather than assumed. Every pixel that differs by more than
# the tolerance carries a cloud line weight under 0.02, with a median of
# nought, against a median of 0.167 where the two agree. A frame drawn with
# the cloud layer off agrees everywhere within one band value.
#
# The share is therefore allowed, and it is kept small enough that a real
# fault cannot hide under it. A defect in any stage of the page reaches far
# more than this share of the frame.
LOUD_SHARE = 0.0015


def build() -> tuple[World, Camera]:
    """Give back a seeded world and a camera that fits it."""
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=4)
    world.seed_world()
    for _ in range(12):
        world.step(1)
    return world, Camera.fitting(world, WIDTH, HEIGHT)


def device_or_skip() -> None:
    """Skip the test when the machine gives no graphics context."""
    try:
        Device().close()
    except (DeviceGap, ImportError) as gap:
        pytest.skip(f"this machine gives no graphics context: {gap}")


def bands(pixels: np.ndarray) -> np.ndarray:
    """Split one frame into red, green and blue, as whole numbers."""
    frame = pixels.reshape(HEIGHT, WIDTH)
    return np.stack(
        [(frame >> 16) & 0xFF, (frame >> 8) & 0xFF, frame & 0xFF], axis=-1
    ).astype(np.int32)


def drawn_both_ways(
    overlay: str | None = None,
    sky: bool = True,
    turn: float | None = None,
    lean: float | None = None,
) -> tuple[np.ndarray, np.ndarray]:
    """Draw one frame each way, and give back the two frames.

    Both renderers get their own view at the same angles, so neither can read
    a state the other left behind.
    """
    world, camera = build()
    frames = []
    for make in (Sketch, GlSketch):
        view = View()
        if turn is not None:
            view.turn = turn
        if lean is not None:
            view.lean = lean
        surface = Surface(WIDTH, HEIGHT)
        make(world, view=view, sky=sky)(
            camera, WIDTH, HEIGHT, surface.pixels, overlay=overlay
        )
        frames.append(surface.pixels.copy())
    return frames[0], frames[1]


def assert_agree(on_processor: np.ndarray, on_device: np.ndarray) -> None:
    """Hold the two frames within the stated tolerance."""
    gap = np.abs(bands(on_processor) - bands(on_device))
    worst = gap.max(axis=-1)
    exact = float((worst == 0).mean())
    loud = float((worst > TOLERANCE).mean())
    assert exact >= EXACT_SHARE, (
        f"only {exact:.4%} of pixels agree exactly, and {EXACT_SHARE:.2%} must"
    )
    assert loud <= LOUD_SHARE, (
        f"{loud:.4%} of pixels differ by more than {TOLERANCE} band values, "
        f"and {LOUD_SHARE:.4%} is the share the cloud hatch explains; the "
        f"worst difference is {worst.max()}"
    )


# ----------------------------------------------------------------------
# The comparison. This is the safety net for not redesigning the look.


# The angles the strict comparison runs at.
#
# **A turn of nought is here because it puts the ground against the edge of
# the page.** The page rolls a field at its edges, and a page whose ground
# stops short of the edge never reads across the roll. The opening angle
# leaves that gap, so a fault in the roll draws a correct picture at the
# opening angle and a wrong one as soon as anybody turns the view. This was
# not reasoned out: the fault was in the tree, and only this angle found it.
EDGE_ANGLES = [(0.0, 0.5), (0.785, 0.5), (1.1, 0.28)]


@pytest.mark.parametrize(("turn", "lean"), EDGE_ANGLES)
def test_the_device_draws_every_pixel_of_the_ground_as_the_processor_does(
    turn: float, lean: float
) -> None:
    """With no cloud layer, the two renderers agree everywhere.

    **This is the strict half of the comparison.** The cloud hatch is a step
    at the edge of a cloud, and it is the one part of the page where the two
    renderers may part company. With the cloud off, nothing may.
    """
    device_or_skip()
    on_processor, on_device = drawn_both_ways(sky=False, turn=turn, lean=lean)
    worst = np.abs(bands(on_processor) - bands(on_device)).max(axis=-1)
    assert worst.max() <= TOLERANCE, (
        f"the ground differs by {worst.max()} band values with no cloud layer, "
        f"at a turn of {turn} and a lean of {lean}"
    )


@pytest.mark.parametrize(
    ("overlay", "sky"),
    [(None, True), (None, False), ("height", True), ("holder", True)],
)
def test_the_device_draws_what_the_processor_draws(
    overlay: str | None, sky: bool
) -> None:
    """The two renderers draw one page, and the device does not redesign it.

    **This is the test that holds the look.** Every stage of the page is
    written twice, once as an array operation and once per pixel. Nothing but
    this comparison says that the two say the same thing.
    """
    device_or_skip()
    assert_agree(*drawn_both_ways(overlay=overlay, sky=sky))


@pytest.mark.parametrize(("turn", "lean"), [(0.0, 0.5), (1.1, 0.28), (2.4, 0.85)])
def test_the_device_follows_the_view_the_watcher_stands_at(
    turn: float, lean: float
) -> None:
    """A mouse turns and leans the page, and the shader honours both angles.

    A shader written against the opening angles alone would pass the test
    above and would draw the wrong page the moment somebody dragged.
    """
    device_or_skip()
    assert_agree(*drawn_both_ways(turn=turn, lean=lean))


def test_a_turn_of_the_view_changes_the_frame_the_device_draws() -> None:
    """The angles reach the page. A frame that ignored them would not move.

    The test above compares two renderers. Two renderers that both ignored the
    view would agree with each other and draw the same page at every angle.
    """
    device_or_skip()
    world, camera = build()
    view = View()
    sketch = GlSketch(world, view=view)
    surface = Surface(WIDTH, HEIGHT)
    sketch(camera, WIDTH, HEIGHT, surface.pixels)
    opening = surface.pixels.copy()
    view.turn = view.turn + 0.6
    sketch(camera, WIDTH, HEIGHT, surface.pixels)
    assert not np.array_equal(opening, surface.pixels), "a turn drew one page"


# ----------------------------------------------------------------------
# The renderer at the seam


def test_the_device_renderer_answers_the_frame_at_the_one_call() -> None:
    """The renderer takes the place of the engine at the one drawing call.

    This drives the demonstration rather than the renderer, because the seam
    is what a watcher meets and a renderer that nothing reaches ships inert.
    """
    device_or_skip()
    world, camera = build()
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.camera = camera
    demo.clock.pause()
    demo.renderer = GlSketch(world, view=demo.view)
    reading = demo.advance()
    surface = Surface(WIDTH, HEIGHT)
    world.draw(camera, WIDTH, HEIGHT, surface.pixels)
    assert not np.array_equal(demo.surface.pixels, surface.pixels)
    # The reading is the engine's own, so the panels read the same numbers
    # under either renderer.
    assert reading["tick"] == world.tick


def test_the_device_renderer_writes_no_value_of_the_world() -> None:
    """The renderer reads the world. It never writes to it."""
    device_or_skip()
    world, camera = build()
    before = world.state_hash
    tick = world.tick
    surface = Surface(WIDTH, HEIGHT)
    GlSketch(world)(camera, WIDTH, HEIGHT, surface.pixels)
    assert world.state_hash == before
    assert world.tick == tick


def test_the_flag_puts_the_device_renderer_at_the_frame(tmp_path: object) -> None:
    """The command line reaches the device path, and it writes a picture.

    **This runs with no window.** The picture flag opens none, so the renderer
    must make a context of its own. A path that needed a window on a screen
    would fail here and would take the picture flag and every test with it.
    """
    device_or_skip()
    drawn = f"{tmp_path}/sketch.png"
    on_processor = f"{tmp_path}/slow.png"
    shared = [
        "--picture",
        "",
        "--ticks",
        "2",
        "--extent",
        "64",
        "--width",
        "320",
        "--height",
        "240",
        "--seed",
        hex(SEED),
        "--sketch",
    ]
    shared[1] = drawn
    assert main(list(shared)) == 0
    shared[1] = on_processor
    assert main([*shared, "--sketch-on-processor"]) == 0
    assert os.path.getsize(drawn) > 0


# ----------------------------------------------------------------------
# The device itself


def test_the_renderer_opens_no_context_before_its_first_frame() -> None:
    """The demonstration builds the renderer before it opens its window.

    A context opened in the constructor would be a second context, and the
    window would then draw in one while every texture sat in the other.
    """
    world, _ = build()
    sketch = GlSketch(world)
    assert sketch._device is None


def test_a_shader_cannot_read_the_unit_the_device_builds_textures_on() -> None:
    """Making or filling a texture binds it, and it binds on one kept unit.

    A call that made a texture on a unit a shader reads would take that
    texture away from the shader. Nothing would report it: the shader would
    read whatever was built last and would draw a picture that looks nearly
    right. The device refuses that unit rather than allowing it.
    """
    device_or_skip()
    device = Device()
    try:
        device.ensure("a_field", "r32f")
        with pytest.raises(DeviceGap):
            device.bind(object(), "a_field", SCRATCH_UNIT)
    finally:
        device.close()


def test_the_shader_takes_every_constant_from_the_module_that_declares_it() -> None:
    """One value is declared once, and the shader is given it.

    **The shader is a second site for every constant of the page.** A number
    typed into the shader would be read back correctly and would draw a
    different page, and only the comparison above would notice. This checks
    the mechanism that stops it: the definitions the shader is built with come
    from the module the array renderer reads.
    """
    written = sketch_gl._defines()
    for name in ("CONTOUR_STEP", "HATCH_SPACING", "CLOUD_FLOOR", "WASH_GAIN"):
        assert f"{name} = {float(getattr(ink, name))!r};" in written, (
            f"the shader does not take {name} from the sketch module"
        )
    # No number of the page is typed into the shader sources themselves.
    for source in (sketch_gl.source.HATCH, sketch_gl.source.COMPOSITE):
        assert "CONTOUR_STEP =" not in source
        assert "HATCH_SPACING =" not in source
