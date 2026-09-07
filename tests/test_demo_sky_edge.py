"""The sky stops at the edge of the page, and its shadow falls on ground.

**The world does not wrap.** A neighbour outside the world is absent, and the
edge of the world is an edge.[^1] The sky pass rolled its field off one side of
the page and back onto the other, so a point near one edge read the cloud of
the far side of the world. It drew a wedge of hatch on the paper beside the
map, hard-edged and shaped like ground that stands elsewhere.

**A shadow falls on something.** The cloud is drawn above the ground, so its
hatch crosses bare paper, and that is the drawing working. The shadow crossed
the paper with it and drew a grey copy of the terrain beside the terrain.

These tests drive the array renderer, which is the renderer that declares the
arithmetic. The device renderer runs the same arithmetic in a shader, and the
module that holds the two renderers together compares their frames.[^2]

References
----------
[^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision
D2.
``docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md``

[^2]: The two renderers, and the bound they agree within.
``tests/test_demo_sketch_gl.py``

Findings register, FND-627. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import numpy as np

from cachette import Camera, World
from cachette.demo import sketch as ink
from cachette.demo.sketch import Sketch
from cachette.demo.surface import Surface
from cachette.demo.view import View

# A world large enough to carry weather, and a frame that leaves bare paper
# around the map.
#
# **The camera must show the paper beside the map.** The sky reaches past the
# ground on purpose, so the fault it hid is only visible where the page has
# room beside the ground. A camera zoomed into the middle of the map shows
# none of it.
SIDE = 96
WIDTH = 700
HEIGHT = 540
SEED = 5

# The steps a world takes before it is drawn, so that it carries weather.
WARMING_STEPS = 30


def _world() -> World:
    """Give back a world that carries weather over ground."""
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=3)
    world.seed_world()
    for _ in range(WARMING_STEPS):
        world.step(1)
    return world


def _frame(world: World, camera: Camera, sky: bool) -> np.ndarray:
    """Draw one frame with the array renderer and give back the pixels."""
    surface = Surface(WIDTH, HEIGHT)
    Sketch(world, view=View(), sky=sky)(camera, WIDTH, HEIGHT, surface.pixels)
    return surface.pixels.reshape(HEIGHT, WIDTH).copy()


def _bare_and_unreachable(
    world: World, camera: Camera
) -> tuple[np.ndarray, np.ndarray]:
    """Say which pixels show no ground, and which of those the sky cannot see.

    A pixel shows no ground when the page holds none at the point it shows.
    The sky reads the cloud of a point further down the page, by the height
    the cloud layer stands at, so a point whose reading lies past the bottom
    of the page has no cloud to draw and no shadow to cast.

    **This reads the mask the build made, and not the colour of a pixel.**
    Bare paper and flat lit ground draw as the same colour, so a test that
    read the colour would call some ground bare and would then assert
    nothing.[^1] The projection and the fit below are public, because the
    device renderer reads them too.

    [^1]: Testing Rules, section 2a. ``.agents/rules/testing.md``
    """
    drawing = Sketch(world, view=View())
    ground = drawing._for(camera, WIDTH, HEIGHT)
    stood = drawing.projection(ground.window, WIDTH, HEIGHT, int(ground.stand[2]))
    take_x, take_y = drawing.fit_lists(
        stood, drawing.box_of(stood), camera, WIDTH, HEIGHT
    )
    # A value below nought names a pixel the page does not reach. That pixel
    # carries bare paper already, and no pass writes to it.
    on_page = (take_x[None, :] >= 0) & (take_y[:, None] >= 0)
    shows_ground = (
        ground.drawn[np.clip(take_y, 0, None)][:, np.clip(take_x, 0, None)] & on_page
    )
    lift = int(stood.rise * ink.CLOUD_HEIGHT)
    page_row = np.broadcast_to(take_y[:, None], (HEIGHT, WIDTH))
    unreachable = page_row + lift >= stood.rows
    return on_page & ~shows_ground, unreachable


def test_the_sky_marks_no_paper_that_it_cannot_see_the_cloud_of() -> None:
    """The sky lays nothing where it has no cloud to read.

    A pixel that shows no ground, and whose cloud stands past the bottom of
    the page, has nothing above it and nothing beside it. The sky must leave
    it as it found it.

    **Two faults reach this one place.** A pass that rolled its field read the
    far side of the world and drew a hatch there. A shadow that fell on bare
    paper drew a grey copy of the terrain there. Either one marks the pixels
    below, and the drawing marks none of them.

    The cloud itself still crosses the paper above the map, and that is the
    drawing working. A cloud stands above the ground, and a watcher reads it
    over bare paper.
    """
    world = _world()
    camera = Camera.fitting(world, WIDTH, HEIGHT)
    bare, unreachable = _bare_and_unreachable(world, camera)
    quiet = bare & unreachable
    assert int(quiet.sum()) > 0, (
        "no pixel of this frame shows bare paper with its cloud off the page, "
        "so this test asserts nothing; the camera must leave paper beside the "
        "map"
    )
    changed = _frame(world, camera, sky=True) != _frame(world, camera, sky=False)
    marked = int((changed & quiet).sum())
    assert marked == 0, (
        f"the sky marked {marked} pixels of bare paper that it has no cloud "
        f"for, so it is reading across the edge of the world or casting a "
        f"shadow where nothing stands"
    )


def test_the_sky_still_draws_the_cloud_that_stands_over_the_map() -> None:
    """The cloud crosses the paper above the ground, and that is the drawing.

    A fix that stopped the sky at the edge of the ground would pass the test
    above by drawing no sky at all. This states that the sky still reaches the
    paper where it has a cloud to reach it with.
    """
    world = _world()
    camera = Camera.fitting(world, WIDTH, HEIGHT)
    bare, unreachable = _bare_and_unreachable(world, camera)
    overhead = bare & ~unreachable
    changed = _frame(world, camera, sky=True) != _frame(world, camera, sky=False)
    assert int((changed & overhead).sum()) > 0, (
        "the sky drew nothing at all on the paper over the map, so the cloud "
        "layer no longer stands above the ground"
    )


def test_a_shift_brings_nothing_from_the_far_side_of_a_field() -> None:
    """The move that the sky reads with leaves the far side alone.

    The rule is one line of the drawing, and the frame cannot state it on its
    own: a shadow and a hatch both reach the same pixel, and the page holds
    ground that draws as bare paper. This states the rule directly.[^1]

    [^1]: Testing Rules, section 4. ``.agents/rules/testing.md``
    """
    field = np.array([[1.0, 2.0, 3.0, 4.0]], dtype=np.float32)
    # A step towards the higher index leaves nothing behind it, and it drops
    # what runs off the end.
    assert ink._slid(field, 1, 1).tolist() == [[0.0, 1.0, 2.0, 3.0]]
    assert ink._slid(field, -1, 1).tolist() == [[2.0, 3.0, 4.0, 0.0]]
    # A step as long as the axis empties the field. A roll would give it back
    # unchanged.
    assert ink._slid(field, 4, 1).tolist() == [[0.0, 0.0, 0.0, 0.0]]
    assert ink._slid(field, -4, 1).tolist() == [[0.0, 0.0, 0.0, 0.0]]
    # No step is no change.
    assert ink._slid(field, 0, 1).tolist() == field.tolist()
    # The other axis behaves alike.
    down = np.array([[1.0], [2.0], [3.0]], dtype=np.float32)
    assert ink._slid(down, 1, 0).tolist() == [[0.0], [1.0], [2.0]]
    assert ink._slid(down, -1, 0).tolist() == [[2.0], [3.0], [0.0]]
