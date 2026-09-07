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

import math
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
    relief: bool = True,
) -> tuple[np.ndarray, np.ndarray]:
    """Draw one frame each way, and give back the two frames.

    Both renderers get their own view at the same angles, so neither can read
    a state the other left behind.

    A caller that asks for no relief gets a page whose ground is flat. The two
    renderers then cover the same points of the paper, so every mark, tone and
    wash must agree.
    """
    world, camera = build()
    frames = []
    for make in (Sketch, GlSketch):
        view = View()
        if turn is not None:
            view.turn = turn
        if lean is not None:
            view.lean = lean
        drawing = make(world, view=view, sky=sky)
        if not relief:
            drawing._raised = np.zeros_like(drawing._raised)
        surface = Surface(WIDTH, HEIGHT)
        drawing(camera, WIDTH, HEIGHT, surface.pixels, overlay=overlay)
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
#
# **The two renderers now build the paper by different means.** The array
# renderer scatters each point of a turned page and resolves the occlusion
# with a running maximum down every column. The device renderer draws the
# terrain as a mesh and lets the depth test resolve it. The two rules disagree
# where a nearer surface stands in front of a further landing, which is every
# ridge and every face the lift exposes.
#
# The comparison is therefore split. The first half takes the relief out and
# holds the two to the old bound, which says that no mark, tone or wash moved.
# The second half puts the relief back, bounds how far the two part company,
# and says where: at the edges the relief makes.


# The angles the strict comparison runs at.
#
# **A turn of nought is here because it puts the ground against the edge of
# the page.** The page rolls a field at its edges, and a page whose ground
# stops short of the edge never reads across the roll. The opening angle
# leaves that gap, so a fault in the roll draws a correct picture at the
# opening angle and a wrong one as soon as anybody turns the view. This was
# not reasoned out: the fault was in the tree, and only this angle found it.
EDGE_ANGLES = [(0.0, 0.5), (0.785, 0.5), (1.1, 0.28)]

# How far apart, in points of the paper, a difference may stand from an edge
# that the relief makes and still count as an edge difference.
#
# The paper is scaled into the frame, so one point of the paper covers rather
# more than one pixel. Two points is the reach at which the measurement below
# was taken.
EDGE_REACH = 2

# The share of the frame that may differ by more than the tolerance once the
# ground has relief, and the share of those pixels that must stand at an edge
# the relief makes.
#
# **These are measured, not chosen.** On the fixture above, at the four pairs
# of angles the tests use, the loud share runs from 2.5 to 5.9 percent of the
# frame, and 81 to 91 percent of those pixels stand within two points of a
# face or a silhouette. The bounds below leave room for the hardware and no
# more.
#
# **Do not raise these to make a change pass.** They exist to report a look
# that moved. The first half of the comparison holds the marks themselves, so
# a change that moved a mark fails there rather than here.
RELIEF_LOUD_SHARE = 0.08
RELIEF_EDGE_SHARE = 0.70


def grown(mask: np.ndarray, reach: int) -> np.ndarray:
    """Grow a mask by a short reach in each direction."""
    wide = mask.copy()
    for step in range(1, reach + 1):
        wide[:, step:] |= mask[:, :-step]
        wide[:, :-step] |= mask[:, step:]
        wide[step:, :] |= mask[:-step, :]
        wide[:-step, :] |= mask[step:, :]
    return wide


def edges_in_the_frame(
    drawing: Sketch, camera: Camera, width: int, height: int
) -> np.ndarray:
    """Say which pixel of the frame shows an edge that the relief makes.

    An edge is a face the lift exposed, or the silhouette of the ground.

    **The renderer says where each pixel of the frame reads the page.** A
    mapping worked out again here would be a second copy of one the renderer
    owns, and the mask would then sit where the page used to be drawn rather
    than where it is drawn now.
    """
    ground = drawing._ground
    assert ground is not None
    drawn = ground.drawn
    edge = ground.cliff & drawn
    edge |= drawn ^ np.roll(drawn, 1, axis=0)
    edge |= drawn ^ np.roll(drawn, 1, axis=1)
    wide = grown(edge, EDGE_REACH)
    stood = drawing.projection(ground.window, width, height, int(ground.stand[2]))
    take_x, take_y = drawing.fit_lists(stood, ground.box, camera, width, height)
    inside = (take_y >= 0)[:, None] & (take_x >= 0)[None, :]
    read = wide[np.clip(take_y, 0, None)][:, np.clip(take_x, 0, None)]
    return np.asarray(read & inside)


@pytest.mark.parametrize(("turn", "lean"), EDGE_ANGLES)
def test_the_device_draws_every_mark_of_a_flat_page_as_the_processor_does(
    turn: float, lean: float
) -> None:
    """With no relief and no cloud, the two renderers agree everywhere.

    **This is the test that holds the look.** Every mark of the page is
    written twice, once as an array operation and once per pixel: the tone of
    the light, five sets of hatching, the silhouette, the grain of the paper,
    the wash of a faction and the fit of the paper into the frame. Nothing but
    this comparison says that the two say the same thing.

    The relief is taken out because it is the one thing the two now build by
    different means. A flat page puts every tile on one plane, so the mesh and
    the scatter cover the same points and every remaining difference would be
    a difference in a mark.
    """
    device_or_skip()
    on_processor, on_device = drawn_both_ways(
        sky=False, turn=turn, lean=lean, relief=False
    )
    assert_agree(on_processor, on_device)


@pytest.mark.parametrize(
    ("overlay", "sky"),
    [(None, True), (None, False), ("height", True), ("holder", True)],
)
def test_the_device_draws_every_wash_of_a_flat_page_as_the_processor_does(
    overlay: str | None, sky: bool
) -> None:
    """The cloud and a named overlay reach the page the same way in both."""
    device_or_skip()
    assert_agree(*drawn_both_ways(overlay=overlay, sky=sky, relief=False))


@pytest.mark.parametrize(("turn", "lean"), [*EDGE_ANGLES, (2.4, 0.85)])
def test_the_relief_is_the_only_thing_the_two_renderers_draw_apart(
    turn: float, lean: float
) -> None:
    """With relief, the two part company, and only at the edges it makes.

    **This test states a difference rather than hiding one.** The array
    renderer keeps the landing nearest above a point. The device renderer
    keeps the surface nearest the watcher. The two answers differ where a near
    surface stands in front of a far landing, and that is a ridge or a face.

    A change that moved a mark, a tone or a wash would show as a difference
    away from those edges, and the share below would fall.
    """
    device_or_skip()
    world, camera = build()
    view = View()
    view.turn, view.lean = turn, lean
    slow = Sketch(world, view=view, sky=False)
    surface = Surface(WIDTH, HEIGHT)
    slow(camera, WIDTH, HEIGHT, surface.pixels)
    on_processor = surface.pixels.copy()

    other = View()
    other.turn, other.lean = turn, lean
    GlSketch(world, view=other, sky=False)(camera, WIDTH, HEIGHT, surface.pixels)

    gap = np.abs(bands(on_processor) - bands(surface.pixels)).max(axis=-1)
    loud = gap > TOLERANCE
    share = float(loud.mean())
    assert share <= RELIEF_LOUD_SHARE, (
        f"{share:.4%} of the frame differs by more than {TOLERANCE} band "
        f"values, and {RELIEF_LOUD_SHARE:.2%} is the share the relief explains"
    )
    at_edge = float(loud[edges_in_the_frame(slow, camera, WIDTH, HEIGHT)].sum()) / max(
        int(loud.sum()), 1
    )
    assert at_edge >= RELIEF_EDGE_SHARE, (
        f"only {at_edge:.2%} of the pixels that differ stand at an edge the "
        f"relief makes, and {RELIEF_EDGE_SHARE:.0%} must; a difference away "
        f"from an edge is a mark that moved"
    )


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
# The terrain, and what the depth test does with it


class Ground:
    """A world whose ground this module chooses.

    The renderer reads the terrain through the readers the engine publishes,
    so a world that answers those readers differently gives the renderer a
    different ground. Everything else is the world it wraps.
    """

    def __init__(self, world: World, heights: np.ndarray, kinds: np.ndarray) -> None:
        """Hold the world, the height of every tile, and the kind of every tile."""
        self._world = world
        self._heights = heights
        self._kinds = kinds

    def __getattr__(self, name: str) -> object:
        """Answer every other question the way the world does."""
        return getattr(self._world, name)

    def tile_heights(self) -> np.ndarray:
        """Give back the height of every tile, as the engine reports it."""
        return self._heights

    def tile_kinds(self) -> np.ndarray:
        """Give back the kind of every tile."""
        return self._kinds


# The world the ridge fixture builds, and where the ridge stands in it.
#
# **This world is built for the case, not copied from the demonstration.** A
# world chosen to look right supplies no ridge that hides anything, so a test
# on that world would measure the fixture.
RIDGE_SIDE = 64
RIDGE_AT = (20, 24)
BEHIND_AT = (33, 44)

# Which columns the fixture changes.
#
# **The rows of a hex grid interlock, so the world draws as a slanted band.**
# A ridge that runs across every column still leaves the two sheared corners
# open, and ground behind the ridge shows through there. The change therefore
# stays away from both ends.
BEHIND_ACROSS = (20, 48)

# How high the ridge stands, and how high the ground behind it stands in the
# second of the two runs. Both are shares of the full range of the ground.
#
# The ground behind must stay low enough that the ridge covers it. A share
# that put it above the top of the ridge would show over the ridge, and the
# test would then be about the fixture rather than about the depth test.
RIDGE_HEIGHT = 1.0
BEHIND_HEIGHT = 0.08

# Where the watcher stands for the ridge fixture.
#
# **The view looks along the axes from the far end on purpose.** The mesh
# holds the tiles in the order the world numbers them, so from the near end
# the tiles already arrive back to front and the order alone would hide the
# ground. From this end they arrive front to back, and nothing but the depth
# test can hide anything.
RIDGE_TURN = math.pi


def ridge_world(behind: float) -> tuple[object, Camera]:
    """Give back a world with a ridge across it, and a camera that fits it.

    The ground behind the ridge stands at the height the caller names. The
    ridge stands across the whole world, and the rest of the ground is flat.
    """
    world = World(width=RIDGE_SIDE, height=RIDGE_SIDE, seed=SEED, faction_count=4)
    world.seed_world()
    heights = np.zeros((RIDGE_SIDE, RIDGE_SIDE), dtype=np.int32)
    heights[RIDGE_AT[0] : RIDGE_AT[1], :] = int(RIDGE_HEIGHT * ink.FIXED_POINT)
    heights[BEHIND_AT[0] : BEHIND_AT[1], BEHIND_ACROSS[0] : BEHIND_ACROSS[1]] = int(
        behind * ink.FIXED_POINT
    )
    # No tile is water, so the water level is nought and the whole height of
    # the ground is relief.
    kinds = np.full((RIDGE_SIDE, RIDGE_SIDE), ink.WATER_KIND + 1, dtype=np.uint8)
    ground = Ground(world, heights.ravel(), kinds.ravel())
    return ground, Camera.fitting(world, WIDTH, HEIGHT)


def drawn_over_a_ridge(behind: float) -> np.ndarray:
    """Draw the ridge world with the device renderer, and give back the frame."""
    ground, camera = ridge_world(behind)
    view = View()
    view.turn, view.lean = RIDGE_TURN, 0.5
    surface = Surface(WIDTH, HEIGHT)
    drawing = GlSketch(ground, view=view, sky=False)
    drawing(camera, WIDTH, HEIGHT, surface.pixels)
    drawing.close()
    return surface.pixels.copy()


def test_the_depth_test_hides_the_ground_behind_a_ridge() -> None:
    """Ground that stands behind a ridge does not reach the paper.

    **The test changes what it must not see.** The ground behind the ridge is
    drawn twice, at two heights, and the frame must not move. A depth test
    that let the far ground through would draw a band across the ridge, and
    the two frames would differ over that band.

    The view looks along the axes of the grid from the far end, so a tile of
    an earlier row stands nearer the watcher. The ridge therefore stands in
    front of the ground the fixture changes, and the mesh reaches the device
    front to back, which is the order that only a depth test can resolve.
    """
    device_or_skip()
    flat = drawn_over_a_ridge(0.0)
    raised = drawn_over_a_ridge(BEHIND_HEIGHT)
    moved = int((flat != raised).sum())
    assert moved == 0, (
        f"{moved} pixels moved when the ground behind the ridge rose, and the "
        f"ridge stands in front of all of it"
    )


def test_the_ridge_fixture_can_show_the_ground_it_hides() -> None:
    """The fixture reaches the case, so the test above is not measuring it.

    A ridge that hid nothing would pass the test above whatever the depth test
    did. This raises the same ground above the ridge and holds that the frame
    then changes, which proves that the ground the fixture moves is ground the
    page would otherwise draw.
    """
    device_or_skip()
    flat = drawn_over_a_ridge(0.0)
    over = drawn_over_a_ridge(RIDGE_HEIGHT)
    assert not np.array_equal(flat, over), (
        "ground raised to the height of the ridge changed no pixel, so the "
        "fixture draws nothing where it moves the ground"
    )


def test_a_turn_of_the_view_does_not_send_the_terrain_again() -> None:
    """The terrain crosses to the device once, however far the view turns.

    **This is the whole point of drawing the terrain as a mesh.** The ground
    never moves, so a turn is a transform and not a rebuild. A renderer that
    sent the vertices again for each angle would draw the same picture and
    would pay the cost this change removes, and nothing else would report it.
    """
    device_or_skip()
    world, camera = build()
    view = View()
    sketch = GlSketch(world, view=view, sky=False)
    surface = Surface(WIDTH, HEIGHT)
    for turn in (0.0, 0.4, 1.3, 2.2, 3.0):
        view.turn = turn
        sketch(camera, WIDTH, HEIGHT, surface.pixels)
    built = sketch.device.meshes_built
    sketch.close()
    assert built == 1, f"the terrain crossed to the device {built} times"


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


def test_the_window_keeps_its_own_frame_buffer_after_a_page() -> None:
    """A borrowed context must be handed back the way it was found.

    Every pass points the frame buffer at a texture the device owns. The
    device borrows the application's context when the application has a
    window, so a binding left in place sends the window's own presentation
    into that texture and the screen stays black.

    No test in this file saw it, because every one of them reads the target
    back rather than presenting a window. This drives the same calls and
    then asks the graphics library what is bound.[^1]

    [^1]: Findings register, FND-596. `docs/FINDINGS.md`
    """
    import ctypes

    import pyglet
    from pyglet import gl

    from cachette.demo.glpage import Device

    window = pyglet.window.Window(width=64, height=48, visible=False)
    try:
        window.switch_to()
        device = Device()
        assert device._borrowed, "the device must take the window's context"

        device._target(32, 32, "rgba8")
        device.read(32, 32)

        bound = gl.GLint(0)
        gl.glGetIntegerv(gl.GL_FRAMEBUFFER_BINDING, ctypes.byref(bound))
        assert bound.value == 0, (
            f"the window frame buffer must be bound and it is {bound.value}"
        )
    finally:
        window.close()
