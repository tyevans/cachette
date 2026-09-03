"""The frame command: one call fills the caller's pixels.

These tests drive the boundary the control plane uses. They build a world,
hand the engine a buffer, and check what came back. Nothing here loops over a
tile or an entity, because that is the thing the boundary exists to
prevent.[^1]

References
----------
ADR-0094, the caller owns the camera and the pixels.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import Camera, FrameError, World

WIDTH = 96
HEIGHT = 64


def a_world() -> World:
    """Build a small world with a founded run, ready to draw.

    The step is not decoration. Founding puts units in the world, and the
    structure that answers "which units stand on this tile" rebuilds at the
    step barrier. A frame asked for before the first step is refused, because
    the drawing will not show a world without its units and call that a
    success.
    """
    world = World(width=48, height=48, seed=7, faction_count=3)
    world.found_run_for_every_faction(24)
    world.step(1)
    return world


def a_surface(width: int = WIDTH, height: int = HEIGHT) -> np.ndarray:
    """Build the pixels a caller lends the engine."""
    return np.zeros(width * height, dtype=np.uint32)


def test_a_frame_fills_the_caller_s_pixels() -> None:
    """The engine writes into memory the caller owns."""
    world = a_world()
    pixels = a_surface()
    camera = Camera()
    camera.look_at(24, 24, WIDTH, HEIGHT)

    world.draw(camera, WIDTH, HEIGHT, pixels)

    # The caller still owns the array it passed, and the engine wrote into
    # that array rather than returning one of its own.
    assert pixels.shape == (WIDTH * HEIGHT,)
    assert int(pixels.max()) > 0, "the frame left every pixel clear"


def test_a_frame_is_a_pure_function_of_a_world_and_a_camera() -> None:
    """Two calls with the same world and camera give the same picture.

    This is the property that makes a reproducible screenshot and a scripted
    flight possible, and it is why the camera is the caller's.
    """
    world = a_world()
    camera = Camera()
    camera.look_at(24, 24, WIDTH, HEIGHT)

    first = a_surface()
    second = a_surface()
    world.draw(camera, WIDTH, HEIGHT, first)
    world.draw(camera, WIDTH, HEIGHT, second)

    assert np.array_equal(first, second)


def test_a_different_camera_gives_a_different_picture() -> None:
    """The camera is in the answer, so moving it changes the frame.

    A frame that repeated whatever the camera said would pass the purity test
    above and be wrong, so this test states what the picture depends on.
    """
    world = a_world()
    near = a_surface()
    far = a_surface()

    close = Camera(tile_size=16.0)
    close.look_at(24, 24, WIDTH, HEIGHT)
    world.draw(close, WIDTH, HEIGHT, near)

    wide = Camera(tile_size=4.0)
    wide.look_at(24, 24, WIDTH, HEIGHT)
    world.draw(wide, WIDTH, HEIGHT, far)

    assert not np.array_equal(near, far)


def test_a_frame_refuses_a_buffer_of_the_wrong_size() -> None:
    """A caller that supplies the wrong memory is refused, not trusted."""
    world = a_world()
    pixels = a_surface(WIDTH, HEIGHT - 1)

    with pytest.raises(FrameError) as refusal:
        world.draw(Camera(), WIDTH, HEIGHT, pixels)

    # The refusal names the size it needed, so a caller can repair the call.
    assert str(WIDTH * HEIGHT) in str(refusal.value)


def test_a_frame_refuses_a_side_of_zero() -> None:
    """A frame of no pixels is refused rather than drawn."""
    world = a_world()
    with pytest.raises(FrameError):
        world.draw(Camera(), 0, HEIGHT, a_surface(0, HEIGHT))


def test_a_frame_refuses_a_tile_smaller_than_a_pixel() -> None:
    """Below one pixel for each tile the work is unbounded and invisible.

    A caller that asked for a hundredth of a pixel for each tile would buy a
    picture of a few pixels that swept every tile the world has. The verb
    refuses and names the bound rather than holding the scale quietly, because
    a silent hold returns a picture that does not match the camera the caller
    asked for.
    """
    world = a_world()
    camera = Camera()
    # The scroll and zoom verbs hold the scale above the bound, because a
    # person should not be able to press a key into a refusal. The setter
    # holds nothing, because the caller owns the camera and the verb is what
    # refuses.
    camera_below = Camera()
    camera_below.tile_width = 0.25
    camera_below.tile_height = 0.25

    with pytest.raises(FrameError) as refusal:
        world.draw(camera_below, WIDTH, HEIGHT, a_surface())

    assert "bound" in str(refusal.value)
    # The same call at a legal scale is accepted, so the refusal is about the
    # scale and not about the rest of the call.
    world.draw(camera, WIDTH, HEIGHT, a_surface())


def test_the_camera_verbs_hold_the_scale_above_the_bound() -> None:
    """Zooming out repeatedly never reaches a tile below one pixel."""
    camera = Camera()
    for _ in range(200):
        camera.zoom_out(WIDTH, HEIGHT)
    assert camera.tile_width >= 1.0
    assert camera.tile_height >= 1.0


def test_a_frame_reports_what_the_drawing_pass_read() -> None:
    """The caller gets the numbers the picture was made from."""
    world = a_world()
    camera = Camera()
    camera.look_at(24, 24, WIDTH, HEIGHT)

    report = world.draw(camera, WIDTH, HEIGHT, a_surface())

    assert report["tick"] == world.tick
    assert report["tiles_painted"] > 0
    # The reading comes from the pass that just ran, so the count of tiles it
    # painted is bounded by the window and not by the world.
    assert report["tiles_painted"] <= WIDTH * HEIGHT


def test_the_frame_cost_follows_the_window_and_not_the_world() -> None:
    """A larger world does not make one frame read more tiles.

    The camera names the tiles a picture covers. A drawing that swept the
    world would paint the same picture, so the count is what proves it did
    not.
    """
    camera = Camera()
    camera.look_at(20, 20, WIDTH, HEIGHT)

    small = World(width=48, height=48, seed=7, faction_count=3)
    large = World(width=192, height=192, seed=7, faction_count=3)

    painted_small = small.draw(camera, WIDTH, HEIGHT, a_surface())["tiles_painted"]
    painted_large = large.draw(camera, WIDTH, HEIGHT, a_surface())["tiles_painted"]

    assert painted_small == painted_large


def test_the_camera_turns_a_pixel_into_a_tile() -> None:
    """A click reaches one tile without a loop over the tiles."""
    camera = Camera(tile_size=16.0)
    camera.look_at(10, 10, WIDTH, HEIGHT)

    middle = camera.tile_at(WIDTH / 2.0, HEIGHT / 2.0)

    assert middle == (10, 10)


def test_a_frame_refuses_a_world_founded_but_not_yet_stepped() -> None:
    """The drawing refuses a world whose structure does not describe it.

    Founding puts units in the world and the spatial structure rebuilds at the
    step barrier. Between the two the engine cannot say which units stand
    where, so it refuses rather than drawing a picture with its people
    missing.
    """
    world = World(width=48, height=48, seed=7, faction_count=3)
    world.found_run_for_every_faction(24)

    with pytest.raises(FrameError):
        world.draw(Camera(), WIDTH, HEIGHT, a_surface())

    # One step rebuilds the structure, and the same call is then accepted.
    world.step(1)
    world.draw(Camera(), WIDTH, HEIGHT, a_surface())


def test_founding_reports_every_faction() -> None:
    """One call seats every faction and says what each one got."""
    world = World(width=48, height=48, seed=7, faction_count=3)

    foundings = world.found_run_for_every_faction(24)

    assert len(foundings) == 3
    assert {founding["faction"] for founding in foundings} == {0, 1, 2}
    for founding in foundings:
        if founding["seated"]:
            assert "q" in founding and "r" in founding
        else:
            assert "refusal" in founding
