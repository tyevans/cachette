"""The bulk overlay reader, and the per-tile answers it must agree with.

An overlay is one quantity of the world, painted over every tile as a
strength of one colour. The engine owns the palette, and it owns the ramp
from a value to a strength. ``World.overlay_paint`` copies both over the
whole world in one crossing, in the tile order that ``tile_holders`` uses.

Each test here drives a per-tile column that the engine already publishes,
and holds the bulk reader against it. A reader that agrees with nothing can
drift away from the engine, and nothing fails when it does.

Every test goes through the published interface.

References
----------
Testing rules, sections 2, 5 and 6. ``.agents/rules/testing.md``
Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
Findings register, FND-569. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette
from cachette import Camera, World, faction_colours
from cachette.demo.sketch import Sketch
from cachette.demo.surface import Surface

EXTENT = 48
"""The side of the world that these tests build."""

SEED = 0x0123_4567_89AB_CDEF
"""A seed that gives land, water and a range of heights."""

WIDTH = 240
HEIGHT = 180
"""The frame that the renderer test fills."""


def build() -> World:
    """Give back a world with factions, units and weather on it."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)
    world.seed_world()
    for _ in range(8):
        world.step(1)
    return world


def test_every_overlay_answers_two_columns_of_the_size_of_the_world() -> None:
    """Each column covers the world once, and neither is a float array."""
    world = build()
    holders = world.tile_holders()
    for name in World.overlay_names():
        paint = world.overlay_paint(name)
        assert sorted(paint) == ["colour", "strength"], name
        assert paint["colour"].shape == holders.shape, name
        assert paint["strength"].shape == holders.shape, name
        assert paint["colour"].dtype.name == "uint32", name
        assert paint["strength"].dtype.name == "uint8", name


def test_a_name_that_no_overlay_carries_is_refused() -> None:
    """A caller learns at once, and the message names the overlays."""
    world = build()
    with pytest.raises(cachette.ViewError) as raised:
        world.overlay_paint("no such quantity")
    for name in World.overlay_names():
        assert name in str(raised.value)


def test_the_holder_overlay_paints_the_colour_of_the_faction_that_holds() -> None:
    """The colour column and the holder column name the same faction.

    **This pins the order of the columns as well as the palette.** The two
    columns index alike, so a reader that walked the world in another order,
    or that took the tile beside the one it meant, gives a colour that names
    another faction on nearly every held tile.
    """
    world = build()
    holders = world.tile_holders()
    paint = world.overlay_paint("holder")
    palette = np.array(faction_colours(), dtype=np.uint32)
    held = holders < world.faction_count
    assert held.any(), "the fixture must give at least one held tile"
    assert not held.all(), "the fixture must leave at least one tile unheld"

    wanted = palette[holders[held]]
    assert np.array_equal(paint["colour"][held], wanted)
    assert (paint["strength"][held] > 0).all(), "a held tile paints"
    assert (paint["strength"][~held] == 0).all(), "an unheld tile paints nothing"
    unheld_colour = np.unique(paint["colour"][~held])
    assert unheld_colour.size == 1, "an unheld tile takes one colour"
    assert unheld_colour[0] not in set(faction_colours()), (
        "the colour of ground that nobody holds must name no faction"
    )


def test_the_height_overlay_ramps_with_the_height_column() -> None:
    """The strength follows the height of the tile, and nothing else.

    **This asks for no ramp of its own.** The engine owns the ramp, so the
    test states the property a ramp has: two tiles of one height paint alike,
    and a taller tile never paints weaker than a shorter one.

    A reader that indexed the wrong tile, that took the wrong overlay, or that
    ramped against a span the engine does not use, breaks the order.
    """
    world = build()
    heights = world.tile_heights().astype(np.int64)
    strength = world.overlay_paint("height")["strength"].astype(np.int64)
    assert heights.min() < heights.max(), "the fixture must give a range of heights"

    order = np.argsort(heights, kind="stable")
    by_height = heights[order]
    by_strength = strength[order]
    assert (np.diff(by_strength) >= 0).all(), "a taller tile never paints weaker"
    same = np.diff(by_height) == 0
    assert (np.diff(by_strength)[same] == 0).all(), (
        "two tiles of one height paint alike"
    )
    assert np.unique(strength).size > 1, "the ramp must move over this world"


def test_the_height_overlay_paints_one_colour_over_the_whole_world() -> None:
    """A quantity paints one hue at a strength, and names no second hue."""
    world = build()
    colour = world.overlay_paint("height")["colour"]
    assert np.unique(colour).size == 1


def test_the_cloud_overlay_stands_on_the_cloud_column() -> None:
    """The strength runs between the ends that the cloud column holds.

    **The cloud stands on the level 1 cell, and the drawing reads it as a
    field.** A cell is many tiles a side, so the engine interpolates between
    the four nearest cell centres and the strength is not flat over a cell. A
    field built that way reaches its ends at a cell centre, so the strongest
    tile is a tile of the fullest cell and the weakest is a tile of the
    emptiest one.

    The engine owns the map from a tile to its weather cell, and the lattice
    carries a margin around the world.[^1] A reader that indexed a weather
    plane by a world address reads another cell, and both ends then move.

    [^1]: Findings register, FND-569. ``docs/FINDINGS.md``
    """
    world = build()
    shares = world.cloud_shares().astype(np.int64)
    strength = world.overlay_paint("cloud")["strength"].astype(np.int64)
    assert shares.min() < shares.max(), "the fixture must give a range of cloud"
    assert np.unique(strength).size > 1, "the ramp must move over this world"

    assert (shares[strength == strength.max()] == shares.max()).any()
    assert (shares[strength == strength.min()] == shares.min()).any()


def test_the_wind_overlay_paints_one_colour_for_one_wind() -> None:
    """The colour follows the wind vector of the tile, and nothing else."""
    world = build()
    winds = world.tile_winds()
    colour = world.overlay_paint("wind")["colour"]
    keys = winds["q"].astype(np.int64) * (1 << 20) + winds["r"].astype(np.int64)
    order = np.argsort(keys, kind="stable")
    same = np.diff(keys[order]) == 0
    assert same.any(), "the fixture must give two tiles of one wind"
    assert (np.diff(colour[order].astype(np.int64))[same] == 0).all()
    assert np.unique(colour).size > 1, "the palette must move over this world"


def test_the_reader_moves_no_state_of_the_world() -> None:
    """Reading the paint of every overlay leaves the world where it was."""
    world = build()
    before = world.state_hash
    tick = world.tick
    for name in World.overlay_names():
        world.overlay_paint(name)
    assert world.state_hash == before
    assert world.tick == tick


def test_the_sketch_reads_its_wash_through_the_bulk_reader() -> None:
    """The renderer takes the engine's paint, and it invents none of its own.

    **This drives the real caller.** The sketch fills a frame with an overlay
    on. The pigment it holds for that overlay must be the engine's own two
    columns, in the tile order of the world.
    """
    world = build()
    camera = Camera.fitting(world, WIDTH, HEIGHT)
    sketch = Sketch(world)
    surface = Surface(WIDTH, HEIGHT)
    sketch(camera, WIDTH, HEIGHT, surface.pixels, overlay="height")

    flow, hue = sketch.overlay_paint("height")
    paint = world.overlay_paint("height")
    rows, columns = world.height, world.width
    assert flow.shape == (rows, columns)
    assert hue.shape == (rows, columns, 3)
    wanted = paint["strength"].reshape(rows, columns).astype(np.float32) / 255.0
    assert np.array_equal(flow, wanted)
    packed = paint["colour"].reshape(rows, columns).astype(np.uint32)
    for band, shift in enumerate((16, 8, 0)):
        assert np.array_equal(
            hue[..., band], ((packed >> shift) & 0xFF).astype(np.float32)
        )


def test_the_wash_of_the_sketch_does_not_follow_the_camera() -> None:
    """One tile takes one wash, at every camera and at every frame size.

    **A wash belongs to a tile and not to a pixel.** A reading taken at the
    pixel the engine drew a tile on is a function of the camera and of the
    frame size. Two cameras that cover the same whole tiles then gave one
    tile the colour of a neighbour, and the colour walked across the ink as a
    zoom moved the camera.[^1]

    [^1]: Findings register, FND-610. ``docs/FINDINGS.md``
    """
    world = build()
    sketch = Sketch(world)
    first, _ = sketch.overlay_paint("height")
    for factor in (1.0, 1.06, 1.13, 1.9, 3.1):
        fitted = Camera.fitting(world, WIDTH, HEIGHT)
        camera = Camera(tile_size=fitted.tile_width * factor)
        camera.origin_x = fitted.origin_x
        camera.origin_y = fitted.origin_y
        camera.clamp(world, WIDTH, HEIGHT)
        surface = Surface(WIDTH, HEIGHT)
        sketch(camera, WIDTH, HEIGHT, surface.pixels, overlay="height")
        after, _ = sketch.overlay_paint("height")
        assert np.array_equal(first, after), f"the wash moved at zoom {factor}"
