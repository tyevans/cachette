"""The minimap of the demonstration.

The minimap is a round wide view in the top right corner of the window. It
reads the level 1 summary of the engine, which holds one entry for a square
block of tiles, and it paints a disc over the frame the engine drew.

**These tests drive the frame, not the disc.** The demonstration is obliged
to paint the minimap, so a test that built a minimap and painted it would
prove that the disc works and prove nothing about whether anything reaches
it.[^1] Each test below steps the demonstration and then reads the pixels.

References
----------
Testing Rules, section 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import numpy as np
import pytest

from cachette import World, faction_colours
from cachette.demo.app import Demo
from cachette.demo.minimap import (
    DIAMETER,
    MARGIN,
    OUTSIDE_COLOUR,
    Minimap,
    _fade,
    _grid,
    read_summary,
)
from cachette.demo.surface import Surface
from cachette.names import Names

if TYPE_CHECKING:
    # The shape of the dictionary the engine returns. It lives in the stub
    # beside the compiled module, so importing it at run time would fail.
    from cachette._core import RegionSummary

# The world these tests build. It is small, so a test runs quickly, and it is
# wider than one block, so the summary holds more than one entry.
WIDTH = 128
HEIGHT = 128
SEED = 0x0123_4567_89AB_CDEF
FACTIONS = 3

# The window these tests draw into.
WINDOW_WIDTH = 480
WINDOW_HEIGHT = 360


def build() -> Demo:
    """Build a demonstration on a seeded world, with a camera in the middle."""
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    demo = Demo(world, Names(SEED), WINDOW_WIDTH, WINDOW_HEIGHT, threads=1)
    demo.seed()
    demo.open_on((WIDTH // 2, HEIGHT // 2))
    # **The clock reads the wall clock, so an unpaused world steps between two
    # drawings.** These tests compare two frames, and a world that moved
    # between them would make the comparison measure the world rather than
    # the disc.
    demo.clock.toggle()
    return demo


def corner(demo: Demo) -> np.ndarray:
    """Give back the block of the frame that the disc covers."""
    frame = demo.surface.pixels.reshape(demo.surface.height, demo.surface.width)
    left = demo.surface.width - MARGIN - DIAMETER
    return frame[MARGIN : MARGIN + DIAMETER, left : left + DIAMETER].copy()


def test_the_minimap_is_on_when_the_demonstration_opens() -> None:
    """The watcher gets the minimap without pressing anything.

    The top right corner holds nothing in an ordinary frame, so the disc
    takes no space from the window. A wide view that starts hidden is a wide
    view that nobody sees.
    """
    demo = build()
    assert demo.minimap.visible


def test_the_toggle_shows_and_hides_the_disc() -> None:
    """The toggle must reach the pixels, not only the flag.

    The frame is drawn three times: with the disc, without it, and with it
    again. The world does not step between the drawings, so the disc is the
    only thing that can differ.
    """
    demo = build()
    demo.advance()
    shown = corner(demo)
    demo.minimap.toggle()
    demo.advance()
    hidden = corner(demo)
    demo.minimap.toggle()
    demo.advance()
    again = corner(demo)
    assert not np.array_equal(shown, hidden), "the toggle did not reach the pixels"
    assert np.array_equal(shown, again), "the disc did not come back the same"


def test_the_hidden_minimap_leaves_the_frame_to_the_engine() -> None:
    """A hidden minimap paints nothing at all.

    The frame is drawn, and the hidden minimap is then asked to paint over
    it. It must refuse, and it must leave every pixel as the engine left it.
    """
    demo = build()
    demo.minimap.visible = False
    demo.advance()
    frame = bytes(demo.surface.to_bytes())
    assert not demo.minimap.paint(demo.world, demo.camera, demo.surface)
    assert bytes(demo.surface.to_bytes()) == frame


def test_the_disc_follows_the_camera() -> None:
    """The middle of the disc is the tile the watcher is looking at."""
    demo = build()
    demo.open_on((32, 32))
    near = demo.minimap.looks_at(
        demo.world, demo.camera, demo.surface.width, demo.surface.height
    )
    demo.open_on((96, 96))
    far = demo.minimap.looks_at(
        demo.world, demo.camera, demo.surface.width, demo.surface.height
    )
    assert near != far, "the disc did not move with the camera"
    assert far[0] > near[0]
    assert far[1] > near[1]


def test_the_disc_has_no_hard_edge() -> None:
    """The disc must fade to nothing, not stop at a circle.

    **A test that the rim differs from the frame passes for a hard cut too.**
    The property that separates the two is how far the rim moved. A fade
    moves the rim a little and the middle a lot. A cut moves both by the same
    amount, because both take the whole colour.

    This measures how far each channel moved from the frame the engine drew,
    in a ring at the rim and in a ring inside it, and asserts that the rim
    moved much less.
    """
    demo = build()
    demo.minimap.visible = False
    demo.advance()
    bare = corner(demo)
    demo.minimap.visible = True
    demo.advance()
    disc = corner(demo)
    rows, columns = np.mgrid[0:DIAMETER, 0:DIAMETER]
    middle = (DIAMETER - 1) / 2.0
    away = np.hypot(rows - middle, columns - middle) / (DIAMETER / 2.0)
    assert not np.asarray(bare != disc)[away > 1.0].any(), "the disc left its circle"
    moved = _moved(bare, disc)
    inner = float(moved[(away > 0.55) & (away < 0.75)].mean())
    rim = float(moved[(away > 0.95) & (away < 1.0)].mean())
    assert inner > 4.0, "the disc painted almost nothing"
    assert rim < inner * 0.35, (
        f"the rim moved {rim:.1f} against {inner:.1f} inside it, so the edge is a cut"
    )


def test_the_summary_holds_one_entry_for_each_block() -> None:
    """The minimap reads the summary level and not the tiles.

    The engine states how many blocks lie across the world. The lattice the
    minimap reads must hold exactly that many, and far fewer entries than the
    world holds tiles.
    """
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    world.seed_world()
    summary = read_summary(world)
    assert summary.across == world.cells_wide
    assert summary.water.size == summary.across * summary.down
    assert summary.water.size * summary.edge * summary.edge >= world.tile_count
    assert summary.water.size < world.tile_count


def test_the_disc_marks_where_the_camera_looks() -> None:
    """A wide view with no mark of the current view is half a tool.

    **A test that the disc changed when the camera zoomed proves nothing**,
    because the disc moves by a tile when the zoom moves the middle. This
    counts the outline itself.

    No layer of the summary can paint a pixel that is bright in all three
    channels. The land runs to a pale sand whose blue is low, and every
    faction colour is low in one channel. Only the outline is bright in all
    three, so a count of those pixels is a count of the outline. A camera
    that zooms in covers less of the disc, so that count must fall.
    """
    demo = build()
    demo.advance()
    wide = _outline_pixels(corner(demo))
    for _ in range(4):
        demo.steer(0.0, 0.0, 1)
    demo.advance()
    close = _outline_pixels(corner(demo))
    assert wide > 0, "the disc drew no outline at all"
    assert close > 0, "the outline went away when the camera zoomed in"
    assert close < wide, f"the outline held {close} pixels against {wide} zoomed out"


def test_the_disc_marks_who_holds_the_ground() -> None:
    """The faction pips must reach the pixels.

    **The summary level counts the held tiles of a block and does not name
    the faction that holds them.** The minimap therefore reads the tile
    holder raster for the name and the summary for the strength. A pip is
    the only thing in the disc that carries a faction colour, so this counts
    the pixels close to one.

    The world is stepped, because nobody holds anything at the founding, and
    a fixture with no held ground would let a broken pip pass.[^1]
    """
    demo = build()
    for _ in range(120):
        demo.world.step(1)
    demo.minimap.forget()
    demo.advance()
    assert _faction_pixels(corner(demo)) > 0, "the disc marked no holder"


def _faction_pixels(block: np.ndarray) -> int:
    """Count the pixels that carry a faction colour.

    A pip takes the whole faction colour, and the disc mixes a fixed share of
    it into the frame. No other layer of the disc paints a colour this
    saturated, because the ground colours are a green, a sand and a blue that
    all sit near the middle of the range.
    """
    channels = [((block >> shift) & 0xFF).astype(np.int64) for shift in (16, 8, 0)]
    spread = np.maximum.reduce(channels) - np.minimum.reduce(channels)
    return int((spread > 120).sum())


def test_the_disc_marks_the_edge_of_the_world() -> None:
    """The disc is centred on the camera and is not held inside the world.

    A camera near an edge therefore looks at ground that does not exist. The
    disc paints that ground in a colour of its own, so a watcher reads the
    edge rather than a block repeated outward.

    **A test that counted dark pixels would pass for a repeated block**, since
    deep water is dark as well. This works out the exact colour an outside
    pixel must hold: the frame the engine drew, mixed with the outside colour
    by the weight the middle of the disc uses. Only an outside pixel matches
    it.
    """
    world = World(width=256, height=256, seed=SEED, faction_count=FACTIONS)
    demo = Demo(world, Names(SEED), WINDOW_WIDTH, WINDOW_HEIGHT, threads=1)
    demo.seed()
    demo.clock.toggle()
    counted = []
    for place in ((128, 128), (2, 2)):
        demo.open_on(place)
        demo.minimap.visible = False
        demo.advance()
        bare = corner(demo)
        demo.minimap.visible = True
        demo.advance()
        counted.append(_outside_pixels(bare, corner(demo)))
    middle, edge = counted
    assert middle == 0, f"the middle of the world gave {middle} pixels of outside"
    assert edge > DIAMETER * DIAMETER // 20, (
        f"the corner of the world gave only {edge} pixels of outside"
    )


def _outside_pixels(bare: np.ndarray, disc: np.ndarray) -> int:
    """Count the pixels the disc painted with the colour of the void."""
    matched = np.ones(bare.shape, dtype=np.bool_)
    for shift in (16, 8, 0):
        was = ((bare >> shift) & 0xFF).astype(np.float64)
        wants = float((OUTSIDE_COLOUR >> shift) & 0xFF)
        weight = float(np.max(_fade(*_grid())))
        want = np.floor(was * (1.0 - weight) + wants * weight)
        matched &= np.abs(((disc >> shift) & 0xFF).astype(np.float64) - want) <= 1.0
    return int(matched.sum())


def test_a_window_too_small_for_the_disc_paints_nothing() -> None:
    """The minimap refuses a window it does not fit in, and says so."""
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    world.seed_world()
    demo = Demo(world, Names(SEED), DIAMETER, DIAMETER, threads=1)
    demo.open_on((WIDTH // 2, HEIGHT // 2))
    assert not demo.minimap.paint(world, demo.camera, demo.surface)


def test_the_summary_refuses_a_block_edge_it_cannot_derive() -> None:
    """The derived block edge is checked against the engine.

    The engine owns the block edge and does not state it. The minimap derives
    it and then checks the derivation, so the two cannot drift apart in
    silence. This gives the check a world whose block count disagrees.
    """
    world = World(width=WIDTH, height=HEIGHT, seed=SEED, faction_count=FACTIONS)
    world.seed_world()

    class Lying:
        """A world that reports a block count the tile count denies."""

        cells_wide = 999
        width = world.width
        height = world.height

        @staticmethod
        def region_summary(q: int, r: int) -> RegionSummary:
            """Give back the summary the real world gives."""
            return world.region_summary(q, r)

    with pytest.raises(ValueError, match="cannot derive the block edge"):
        read_summary(Lying())  # type: ignore[arg-type]


def test_the_minimap_stands_down_for_the_reference_key() -> None:
    """The engine puts a card in the top right while the key is held.

    Two things in one corner means a watcher reads neither, so the disc goes
    away while the key is down and comes back when it is released.

    **A frame with the key held differs from a frame without it whatever the
    disc does**, because the engine draws the card in one of them. The test
    therefore holds the key in both frames and changes only whether the
    watcher wants the disc.

    The card the engine draws there reports a cost it measures, so a few of
    its pixels differ between two frames of one world. The test therefore
    counts how many pixels of the corner moved rather than demanding that
    none did. A card that reports a new cost moves a few pixels. A disc moves
    thousands.
    """
    demo = build()
    demo.reference = True
    demo.minimap.visible = False
    demo.advance()
    quiet = corner(demo)
    demo.advance()
    card = _differing(quiet, corner(demo))
    demo.minimap.visible = True
    demo.advance()
    held = _differing(quiet, corner(demo))
    demo.reference = False
    demo.advance()
    shown = _differing(quiet, corner(demo))
    assert shown > DIAMETER * DIAMETER // 4, "the disc did not come back"
    assert held < max(card * 4, 200), (
        f"{held} pixels moved while the key was held, against {card} for the "
        f"card alone, so the disc covered the key"
    )


def test_the_camera_holds_a_minimap_of_its_own() -> None:
    """A minimap built by hand paints the same disc as the one the demo holds.

    The demonstration owns one minimap. This proves the disc is a function of
    the world and the camera and holds no other state, so two runs of the same
    frame give the same pixels.
    """
    demo = build()
    demo.advance()
    first = corner(demo)
    fresh = Minimap()
    demo.world.draw(
        demo.camera, demo.surface.width, demo.surface.height, demo.surface.pixels
    )
    fresh.paint(demo.world, demo.camera, demo.surface)
    assert np.array_equal(first, corner(demo))


def test_the_disc_is_stable_when_nothing_moves() -> None:
    """Two frames of one world at one camera give one disc.

    The demonstration compares frames elsewhere, and a disc that flickered
    would break that comparison without saying why.
    """
    demo = build()
    demo.advance()
    first = corner(demo)
    demo.advance()
    assert np.array_equal(first, corner(demo))


def test_the_disc_takes_its_colours_from_the_engine() -> None:
    """The faction pips carry the colours the engine states.

    **A second table of faction colours in the control plane would drift from
    the one the engine draws with, and nothing would fail.** This paints the
    disc, takes the most saturated pixel it holds, and asks which colour of
    the engine it lies nearest. The answer must be a faction colour and not
    the ground.
    """
    demo = build()
    for _ in range(120):
        demo.world.step(1)
    demo.minimap.forget()
    demo.advance()
    block = corner(demo)
    channels = [((block >> shift) & 0xFF).astype(np.int64) for shift in (16, 8, 0)]
    spread = np.maximum.reduce(channels) - np.minimum.reduce(channels)
    at = np.unravel_index(int(np.argmax(spread)), spread.shape)
    seen = np.array([int(channel[at]) for channel in channels], dtype=np.float64)
    wanted = np.array(
        [
            [(colour >> shift) & 0xFF for shift in (16, 8, 0)]
            for colour in faction_colours()
        ],
        dtype=np.float64,
    )
    away = np.abs(wanted - seen).sum(axis=1).min()
    assert away < 120.0, f"the strongest pixel {seen} matches no faction colour"


def _differing(one: np.ndarray, other: np.ndarray) -> int:
    """Count how many pixels of two blocks of the frame differ."""
    return int(np.count_nonzero(one != other))


def _moved(bare: np.ndarray, disc: np.ndarray) -> np.ndarray:
    """Give back how far each pixel moved, as the sum over the three channels."""
    total = np.zeros(bare.shape, dtype=np.float64)
    for shift in (16, 8, 0):
        was = ((bare >> shift) & 0xFF).astype(np.float64)
        now = ((disc >> shift) & 0xFF).astype(np.float64)
        total += np.abs(now - was)
    return total


def _outline_pixels(block: np.ndarray) -> int:
    """Count the pixels that are bright in all three channels."""
    least = np.full(block.shape, 255, dtype=np.int64)
    for shift in (16, 8, 0):
        least = np.minimum(least, ((block >> shift) & 0xFF).astype(np.int64))
    return int((least >= 205).sum())


def _blank() -> Surface:
    """Give back a surface the size of the test window."""
    return Surface(WINDOW_WIDTH, WINDOW_HEIGHT)
