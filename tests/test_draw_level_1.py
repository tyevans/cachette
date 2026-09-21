"""Tests for drawing macroscopic level 1 frames.

Level 0 holds individual tiles and units. Level 1 summarises blocks of tiles
at city scale. When a camera draws tiles smaller than one pixel, Level 0
refuses and Level 1 draws the macroscopic summary.[^1]

References
----------
[^1]: ADR-0022, level 0 is the only truth and every level above it is derived,
    decision D4.
    ``docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md``
[^2]: ADR-0094, the caller owns the camera and the pixels, decision D6.
    ``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import Camera, FrameError, World

WIDTH = 256
HEIGHT = 256


def a_world() -> World:
    """Build a world with settled factions, ready to draw."""
    world = World(width=64, height=64, seed=42, faction_count=2)
    world.found_run_for_every_faction(24)
    world.step(1)
    return world


def a_surface(width: int = WIDTH, height: int = HEIGHT) -> np.ndarray:
    """Build the pixel buffer a caller lends the engine."""
    return np.zeros(width * height, dtype=np.uint32)


def test_level_0_refuses_subpixel_and_level_1_succeeds() -> None:
    """Level 0 refuses sub-pixel zoom and Level 1 draws macroscopic summary."""
    world = a_world()
    pixels = a_surface()
    camera = Camera()
    camera.tile_width = 0.25
    camera.tile_height = 0.25
    camera.look_at(32, 32, WIDTH, HEIGHT)

    # Level 0 refuses tile_width < 1.0 (Lattice bound).
    with pytest.raises(FrameError, match="below the bound of 1 pixel"):
        world.draw(camera, WIDTH, HEIGHT, pixels, render_level=0)

    # Level 1 explicitly permits sub-pixel tile camera scales.
    report = world.draw_level1(camera, WIDTH, HEIGHT, pixels)
    assert report["render_level"] == 1
    assert report["tiles_painted"] > 0
    assert int(pixels.max()) > 0


def test_report_identifies_render_level() -> None:
    """Returned frame reading explicitly names the pyramid level drawn."""
    world = a_world()
    pixels = a_surface()
    camera = Camera(tile_size=4.0)
    camera.look_at(32, 32, WIDTH, HEIGHT)

    report_l0 = world.draw(camera, WIDTH, HEIGHT, pixels, render_level=0)
    assert report_l0["render_level"] == 0

    report_l1 = world.draw(camera, WIDTH, HEIGHT, pixels, render_level=1)
    assert report_l1["render_level"] == 1

    report_method = world.draw_level1(camera, WIDTH, HEIGHT, pixels)
    assert report_method["render_level"] == 1


def test_draw_refuses_unknown_render_level() -> None:
    """Requesting an unbuilt pyramid level raises FrameError."""
    world = a_world()
    pixels = a_surface()
    camera = Camera(tile_size=4.0)

    with pytest.raises(FrameError, match="pyramid level 2 does not exist"):
        world.draw(camera, WIDTH, HEIGHT, pixels, render_level=2)
