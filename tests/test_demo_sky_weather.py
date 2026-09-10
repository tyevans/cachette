"""The sky draws weather: a broken cover, a moving mass, and a deep sky.

**A cover share and an opacity are two quantities, and the engine carries
one.** A broken sky is one part of the sky at full opacity beside a part at
none. A renderer that turned the share into a weight painted thin cloud
everywhere, at every share, so no share drew a broken sky. The middle of the
range is where the field lives: over a measured window, 47 percent of cells
stood in the broken band for more than half of it.[^1]

**The sky over a place does not change much.** Over four hundred ticks, 412
cells of 9216 were ever clear and ever overcast, and one cell at the equator
held one narrow band for the whole window.[^1] The motion in the picture
therefore belongs to the renderer. These tests state that it is there.

These tests drive the array renderer, which is the renderer that declares the
arithmetic. The device renderer runs the same arithmetic in a shader, and the
module that holds the two renderers together compares their frames.[^2]

References
----------
[^1]: Findings register, FND-715. ``docs/FINDINGS.md``

[^2]: The two renderers, and the bound they agree within.
``tests/test_demo_sketch_gl.py``

Testing Rules, section 2a, a fixture supplies the input.
``.agents/rules/testing.md``
"""

from __future__ import annotations

import itertools

import numpy as np

from cachette import Camera, World
from cachette.demo import sketch as ink
from cachette.demo.sketch import Sketch
from cachette.demo.surface import Surface
from cachette.demo.view import View

# A square of the page that the cloud field is read over, in page points.
#
# **The square must be many cloud cells wide.** One cell is tens of page
# points, so a small square samples one mass and reports whatever that mass
# happens to be. This is wide enough to hold a whole sky.
PAGE_SIDE = 320

# The shares of the sky that the tests read the cloud at.
#
# **The middle of the range is the case that matters**, and a fixture built
# from the demonstration world does not supply it reliably. These are stated
# rather than drawn from a world, so the assertion always meets the input it
# is about.
CLEAR_SHARE = 0.10
BROKEN_SHARE = 0.55
CLOSED_SHARE = 1.00

# A world small enough to build quickly, and a frame that leaves bare paper
# around the map.
SIDE = 96
WIDTH = 700
HEIGHT = 540
SEED = 5
WARMING_STEPS = 30

# A wider world and a second seed, for the test that needs the deep of the
# sky.
#
# **A fixture supplies the input, and this fixture was chosen against the
# field.** The world above holds a broken sky and barely reaches the top of
# the cover range. This one holds cover above the deep mark over a real share
# of itself, which is the case that the test is about.[^1]
#
# [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
DEEP_SIDE = 128
DEEP_SEED = 0x0123_4567_89AB_CDEF


def _world(side: int = SIDE, steps: int = WARMING_STEPS, seed: int = SEED) -> World:
    """Give back a world that carries weather over ground."""
    world = World(width=side, height=side, seed=seed, faction_count=3)
    world.seed_world()
    for _ in range(steps):
        world.step(1)
    return world


def _drawing() -> Sketch:
    """Give back a renderer whose cloud field the tests read."""
    return Sketch(_world(side=32, steps=4), view=View())


def _place() -> tuple[np.ndarray, np.ndarray]:
    """Give back every point of a square of the page."""
    rows = np.arange(PAGE_SIDE, dtype=np.float32)
    down, across = np.meshgrid(rows, rows, indexing="ij")
    return across, down


def _mass(
    drawing: Sketch,
    share: float,
    clock: float = 0.0,
    heading: tuple[float, float] = (1.0, 0.0),
) -> np.ndarray:
    """Give back the cloud mass over a square of the page, at one cover.

    The heading is where the wind blows on the page. The clock is in ticks.
    """
    across, down = _place()
    along_x = np.full_like(across, heading[0])
    along_y = np.full_like(across, heading[1])
    density, _, _, _ = drawing.cloud_body(across, down, along_x, along_y, clock, 1.0)
    return ink.cloud_mass_of(density, np.full_like(density, share))


def test_a_middling_cover_leaves_part_of_the_sky_open() -> None:
    """A broken cover paints cloud over part of the sky and nothing over the rest.

    **This is the property the old treatment could not hold.** That treatment
    turned the cover into the width of a ruled line, so a cover of a half drew
    a half-weight line everywhere. Its mass was one value over the whole sky,
    and no share of the cover could put full cloud beside open sky.

    The test states both halves. The sky must be part closed, and it must be
    part open, and both parts must be large.
    """
    mass = _mass(_drawing(), BROKEN_SHARE)
    closed = float((mass > 0.95).mean())
    open_sky = float((mass < 0.05).mean())
    assert closed > 0.15, (
        f"only {closed:.3f} of a broken sky carries full cloud, so the cover "
        f"is painting a tint rather than a mass"
    )
    assert open_sky > 0.15, (
        f"only {open_sky:.3f} of a broken sky is open, so the cover is "
        f"painting a tint rather than a mass"
    )
    assert closed + open_sky > 0.55, (
        f"the sky is neither closed nor open over {1.0 - closed - open_sky:.3f} "
        f"of itself, so the edges of the masses carry more of the picture "
        f"than the masses do"
    )


def test_the_cloud_closes_as_the_cover_rises() -> None:
    """More cover closes more of the sky, and a full cover closes all of it."""
    drawing = _drawing()
    shares = (0.25, 0.40, 0.55, 0.70, 0.85, CLOSED_SHARE)
    closed = [float((_mass(drawing, share) > 0.5).mean()) for share in shares]
    for lower, higher in itertools.pairwise(closed):
        assert higher >= lower - 0.01, f"the sky opens as the cover rises: {closed}"
    assert closed[0] < 0.25, f"a thin cover closes too much of the sky: {closed[0]:.3f}"
    assert closed[-1] > 0.95, f"a full cover leaves the sky open: {closed[-1]:.3f}"


def test_a_clear_sky_draws_no_cloud() -> None:
    """A sky the engine reports as clear carries no wisp.

    The floor is the share below which the page draws nothing, and the cover
    opens from nothing above it. A field of noise reaches high values
    somewhere, so a treatment with no such ramp would paint a wisp over a
    clear sky.
    """
    mass = _mass(_drawing(), CLEAR_SHARE)
    assert float(mass.max()) == 0.0, f"a clear sky carries cloud up to {mass.max():.3f}"


def test_the_clock_moves_the_cloud() -> None:
    """The cloud field is not the same field one tick later.

    **The motion belongs to the renderer.** The sky over a place holds one
    narrow band and does not leave it, so a picture that waited for the engine
    to move the weather would stand still.
    """
    drawing = _drawing()
    first = _mass(drawing, BROKEN_SHARE, clock=0.0)
    later = _mass(drawing, BROKEN_SHARE, clock=1.0)
    moved = float((np.abs(first - later) > 0.25).mean())
    assert moved > 0.02, (
        f"one tick moved the cloud over {moved:.4f} of the sky, so the sky stands still"
    )


def test_the_heading_of_the_wind_steers_the_cloud() -> None:
    """The cloud drifts along the wind, so a second heading gives a second sky.

    The drift is a distance along the heading, and it grows with the clock. A
    clock of nothing therefore leaves the two headings alike, and this reads
    the field after the clock has run.
    """
    drawing = _drawing()
    east = _mass(drawing, BROKEN_SHARE, clock=12.0, heading=(1.0, 0.0))
    south = _mass(drawing, BROKEN_SHARE, clock=12.0, heading=(0.0, 1.0))
    apart = float((np.abs(east - south) > 0.25).mean())
    assert apart > 0.1, (
        f"two headings drew the same sky over {1.0 - apart:.3f} of itself, so "
        f"the wind does not steer the cloud"
    )


def test_the_swirl_deforms_the_cloud_rather_than_sliding_it() -> None:
    """A mass changes shape as it crosses the page.

    The cloud field drifts along the wind. A second, slower field bends the
    reading, and the two rates differ, so a mass runs through the second field
    and deforms.

    **A field with no swirl would slide and nothing else.** The reading below
    moves the place by exactly the drift of the clock and reads at a clock of
    nothing. Without the swirl the two readings would be one field, to the
    last bit. The difference is the swirl.
    """
    drawing = _drawing()
    across, down = _place()
    run = 40.0 * ink.CLOUD_DRIFT
    along_x = np.ones_like(across)
    along_y = np.zeros_like(across)
    slid, _, _, _ = drawing.cloud_body(across - run, down, along_x, along_y, 0.0, 1.0)
    drifted, _, _, _ = drawing.cloud_body(across, down, along_x, along_y, 40.0, 1.0)
    bent = float(np.abs(slid - drifted).mean())
    assert bent > 0.02, (
        f"the cloud slid without changing shape: the two fields differ by "
        f"{bent:.5f} on average"
    )


def test_a_mass_carries_a_lit_side_and_a_shaded_side() -> None:
    """The face of a mass against the light is not one value.

    The page reads the coarsest octave again a short step toward the light,
    and the difference is the face. A treatment that dropped that reading
    would give one face everywhere, and a mass would draw flat.
    """
    drawing = _drawing()
    across, down = _place()
    along_x = np.ones_like(across)
    along_y = np.zeros_like(across)
    _, coarse, at_x, at_y = drawing.cloud_body(across, down, along_x, along_y, 0.0, 1.0)
    toward_x, toward_y = ink._toward_light()
    ahead = ink._cloud_noise(
        drawing.cloud_lattice(),
        at_x + toward_x * ink.CLOUD_LIGHT_STEP,
        at_y + toward_y * ink.CLOUD_LIGHT_STEP,
    )
    face = np.clip(0.5 - (coarse - ahead) * ink.CLOUD_RELIEF, 0.0, 1.0)
    assert float((face < 0.2).mean()) > 0.1, "no mass turns its face to the light"
    assert float((face > 0.8).mean()) > 0.1, "no mass turns its face away"


def _frames(world: World, sky: bool, phases: tuple[float, ...]) -> list[np.ndarray]:
    """Draw one frame of a world at each phase, and give back the pixels."""
    camera = Camera.fitting(world, WIDTH, HEIGHT)
    surface = Surface(WIDTH, HEIGHT)
    drawing = Sketch(world, view=View(), sky=sky)
    drawn = []
    for phase in phases:
        drawing(camera, WIDTH, HEIGHT, surface.pixels, phase=phase)
        drawn.append(surface.pixels.reshape(HEIGHT, WIDTH).copy())
    return drawn


def test_the_sky_moves_while_the_world_stands() -> None:
    """The picture of the sky changes between two moments of one tick.

    **This drives the renderer that a watcher opens, and not the field
    alone.** A cloud field that moves and a renderer that never asks it for a
    second moment would pass every test above and draw a still sky.

    The world does not step, so every other layer is the same layer. The
    frames drawn with no sky give the pixels that the rest of the drawing
    agrees on, and the sky must move over those.
    """
    world = _world()
    still = _frames(world, sky=False, phases=(0.0, 0.9))
    settled = still[0] == still[1]
    assert int(settled.sum()) > 0, "the drawing agrees on no pixel of two phases"
    clouded = _frames(world, sky=True, phases=(0.0, 0.9))
    moved = int(((clouded[0] != clouded[1]) & settled).sum())
    assert moved > 500, (
        f"the sky moved {moved} pixels between two moments of one tick, so a "
        f"watcher of a slow world sees a still sky"
    )


# The band value above which a pixel counts as paper.
#
# **A panel is not paper.** The renderer keeps the pixels that a panel, a card
# or the clock painted, and puts the sketch under them. Those pixels lie over
# the page and the page never reaches them, so a test that read them would
# measure the panel. Paper runs above this value and every panel runs below
# it.
PAPER_BAND = 200


def _bare(world: World, camera: Camera) -> np.ndarray:
    """Say which pixels of the frame show no ground.

    **This reads the mask the build made, and not the colour of a pixel.**
    Bare paper and flat lit ground draw as the same colour, so a test that
    read the colour would call some ground bare and would then assert
    nothing.[^1] The projection and the fit below are public, because the
    device renderer reads them too.

    **Over bare paper the sky is the only layer that marks the page.** The
    hatch, the wash, the roads, the silhouette and the shadow of a cloud all
    stand on ground. A pixel that shows no ground therefore holds the paper
    and the sky and nothing else.

    [^1]: Testing Rules, section 2a. `.agents/rules/testing.md`
    """
    drawing = Sketch(world, view=View())
    ground = drawing._for(camera, WIDTH, HEIGHT)
    stood = drawing.projection(ground.window, WIDTH, HEIGHT, int(ground.stand[2]))
    take_x, take_y = drawing.fit_lists(
        stood, drawing.box_of(stood), camera, WIDTH, HEIGHT
    )
    on_page = (take_x[None, :] >= 0) & (take_y[:, None] >= 0)
    shows_ground = (
        ground.drawn[np.clip(take_y, 0, None)][:, np.clip(take_x, 0, None)] & on_page
    )
    return on_page & ~shows_ground


def test_the_deep_of_the_sky_reaches_the_page() -> None:
    """A sky at the top of the cover range draws darker than a fair sky can.

    **The deep of the sky is the top of the cover range.** Over a measured
    window, 522 cells of 9216 stood overcast at every tick, and the set of
    overcast cells turned over, so the deep of the sky is a real thing that
    moves.

    The test states two things. The fixture must reach the deep band, or the
    assertion below measures nothing. The drawing must then put ink on the
    page that a fair sky cannot reach.

    **The bound comes from the constants and not from a number typed here.**
    Over bare paper a fair sky lays the ink of a fair sky at the weight of a
    shaded face, and nothing else marks the pixel. That is the darkest a fair
    sky reaches there. A deep sky goes below it, because it lays a gloom over
    the whole of itself first and then lays the ink of a deep sky.
    """
    world = _world(side=DEEP_SIDE, seed=DEEP_SEED)
    cover = Sketch(world, view=View()).sky_fields()[0]
    deep = float((cover > ink.SKY_DEEP_MARK).mean())
    assert deep > 0.05, (
        f"only {deep:.4f} of this world stands in the deep of the sky, so "
        f"this test asserts nothing"
    )
    camera = Camera.fitting(world, WIDTH, HEIGHT)
    clouded = _frames(world, sky=True, phases=(0.0,))[0] & 0xFF
    plain = _frames(world, sky=False, phases=(0.0,))[0] & 0xFF
    paper = _bare(world, camera) & (plain > PAPER_BAND)
    assert int(paper.sum()) > 1000, "the camera leaves no paper beside the map"
    fair = plain * (1.0 - ink.CLOUD_FACE_DARK) + float(ink.SKY_INK[2]) * (
        ink.CLOUD_FACE_DARK
    )
    darker = int(((clouded < fair - 1.0) & paper).sum())
    assert darker > 200, (
        f"the sky drew {darker} pixels of bare paper darker than a fair sky "
        f"can reach, so the deep of the sky puts no ink on the page"
    )
