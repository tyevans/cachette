"""The mouse controls of the demonstration.

The window of the demonstration cannot open in the gate, because the gate has
no display. Every test here therefore feeds the mouse handler the arguments
the window library delivers to it, and then reads the view the next frame
would be drawn with.[^1]

**A test that called a pan function would prove that the function works.** It
would not prove that a drag reaches it. The tests below call the methods by
the names the library calls, with the numbering of rows the library uses, so a
handler that was never wired shows up here.

The properties, and not the examples
------------------------------------

Three properties carry the feel of a mouse control, and each has a test.

The ground under the cursor is the same ground after a drag. That is
grab-and-move, and it is the whole difference between taking hold of a map and
pushing a camera.

The point under the cursor is the same point after a wheel zoom. A zoom about
the middle of the frame throws away the one thing the person was pointing at.

The lean stops at both ends, so the view cannot pass through the ground and
cannot turn over.

References
----------
The pyglet window, the mouse events it delivers and their arguments.
https://pyglet.readthedocs.io/en/latest/modules/window.html

ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``
"""

from __future__ import annotations

import math

import pytest

from cachette import Camera, World
from cachette.demo.app import Demo
from cachette.demo.mouse import (
    LEAN_EACH_PIXEL,
    LEFT_BUTTON,
    MIDDLE_BUTTON,
    RIGHT_BUTTON,
    TURN_EACH_PIXEL,
    Controls,
)
from cachette.demo.view import (
    MAX_LEAN,
    MAX_TILE,
    MIN_LEAN,
    MIN_TILE,
    WHEEL_PRESSES,
    ZOOM_STEP,
    View,
)
from cachette.names import Names

# A world large enough that the bound on how far the view may travel never
# bites in the middle of it, and a frame small enough to keep a test quick.
#
# **The bound would hide the property.** A view held at the edge of the world
# moves less than the hand asked for, and the ground would then not stay under
# the cursor. Every test below works in the middle of a world that is far
# wider than the frame.
SIDE = 160
WIDTH = 640
HEIGHT = 480
SEED = 0x0123_4567_89AB_CDEF

# Where the tests take hold of the map. It is away from the edges of the
# frame, so a drag in any direction stays inside the window.
GRAB_X = 400
GRAB_Y = 300


def build() -> Demo:
    """Give back the demonstration state, pointed at the middle of a world."""
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=2)
    world.seed_world()
    demo = Demo(world, Names(SEED), WIDTH, HEIGHT)
    demo.camera = Camera(tile_size=16.0)
    demo.open_on((SIDE // 2, SIDE // 2))
    return demo


def frame_y(y: int) -> float:
    """Turn a row the window counts from the bottom into one the frame counts.

    The window numbers its rows from the bottom of the screen and the frame
    numbers them from the top, so a test that reads the frame states the
    conversion once, here.
    """
    return float(HEIGHT - y)


def ground_under(demo: Demo, x: int, y: int) -> tuple[int, int]:
    """Give back the tile the frame shows at a place in the window."""
    return demo.camera.tile_at(float(x), frame_y(y))


def drag(
    controls: Controls,
    button: int,
    start: tuple[int, int],
    steps: list[tuple[int, int]],
) -> tuple[int, int]:
    """Press a button, move the pointer, and let go.

    The steps are the offsets the window reports for each move. The window
    reports where the pointer now is and how far it moved, so this keeps both
    and gives back where the pointer ended.
    """
    x, y = start
    controls.on_mouse_press(x, y, button, 0)
    for dx, dy in steps:
        x, y = x + dx, y + dy
        controls.on_mouse_drag(x, y, dx, dy, button, 0)
    controls.on_mouse_release(x, y, button, 0)
    return x, y


def test_the_handler_names_are_the_names_the_window_library_calls() -> None:
    """The window dispatches by name, so a wrong name reaches nothing.

    A handler that the library never calls is a control that does not exist.
    The library states the names it dispatches, so the test reads them from
    the library rather than repeating them.
    """
    pyglet = pytest.importorskip("pyglet")
    known = set(pyglet.window.Window.event_types)
    wired = {
        name
        for name in dir(Controls)
        if name.startswith("on_") and callable(getattr(Controls, name))
    }
    assert wired, "the mouse layer wires no handler at all"
    assert wired <= known, f"the library calls none of {sorted(wired - known)}"
    assert {"on_mouse_press", "on_mouse_drag", "on_mouse_scroll"} <= wired


def test_the_ground_stays_under_the_cursor_while_the_hand_drags_it() -> None:
    """A left drag moves the map, and the map keeps hold of the hand.

    **This is grab-and-move.** A control that moved the camera instead would
    send the ground the other way, and the tile the hand took hold of would
    walk out from under it.
    """
    demo = build()
    controls = Controls(demo)
    took = ground_under(demo, GRAB_X, GRAB_Y)
    at_x, at_y = drag(
        controls,
        LEFT_BUTTON,
        (GRAB_X, GRAB_Y),
        [(-13, 7), (-20, 11), (-9, 4), (-31, -18)],
    )
    assert (at_x, at_y) != (GRAB_X, GRAB_Y)
    assert ground_under(demo, at_x, at_y) == took


def test_a_drag_moves_the_map_the_way_the_hand_went() -> None:
    """The map follows the hand, and it does not run the other way.

    The property above holds for a control that moved the ground the wrong way
    by exactly the wrong amount only if the tile under the cursor happened to
    match, so this states the direction on its own.
    """
    demo = build()
    was = demo.camera.origin_x, demo.camera.origin_y
    controls = Controls(demo)
    drag(controls, LEFT_BUTTON, (GRAB_X, GRAB_Y), [(40, 25)])
    # The window counts rows up the screen and the frame counts them down, so
    # a drag up the screen moves the ground up the frame.
    assert demo.camera.origin_x == pytest.approx(was[0] + 40.0)
    assert demo.camera.origin_y == pytest.approx(was[1] - 25.0)


def test_the_point_under_the_cursor_stays_under_the_cursor_through_a_zoom() -> None:
    """The wheel zooms about the cursor and not about the middle of the frame.

    **This is the defect a reader cannot see and a person feels at once.** A
    zoom anchored at the middle of the frame passes every test that only
    checks the size of a tile.
    """
    demo = build()
    controls = Controls(demo)
    was = ground_under(demo, GRAB_X, GRAB_Y)
    for notches in (1.0, 1.0, -1.0, 2.0, -3.0):
        controls.on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, notches)
        assert ground_under(demo, GRAB_X, GRAB_Y) == was


def test_a_zoom_about_the_middle_of_the_frame_loses_the_ground_the_cursor_held() -> (
    None
):
    """The two anchors differ, so the test above can tell them apart.

    A test that asserts a property proves nothing unless the property can
    fail. This builds the defect the test above forbids and shows that it
    reaches a different view.
    """
    demo = build()
    Controls(demo).on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, 2.0)
    anchored = demo.camera.origin_x, demo.camera.origin_y

    middle = build()
    # The defect: the engine zoom verb holds the middle of the frame, not the
    # cursor. It is the call a wiring that forgot the cursor would reach for.
    for _ in range(2 * WHEEL_PRESSES):
        middle.camera.zoom_in(WIDTH, HEIGHT)

    assert middle.camera.tile_width == pytest.approx(demo.camera.tile_width)
    assert (middle.camera.origin_x, middle.camera.origin_y) != anchored
    assert ground_under(middle, GRAB_X, GRAB_Y) != ground_under(demo, GRAB_X, GRAB_Y)


def test_the_wheel_makes_a_tile_larger_and_smaller_by_the_step_of_a_key() -> None:
    """One notch is a whole number of key presses, so the two agree.

    The engine owns the step. A second copy here would part company with it
    and nothing would fail, so the test reads the step from the view module,
    which reads it from the engine.
    """
    demo = build()
    controls = Controls(demo)
    was = demo.camera.tile_width
    controls.on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, 1.0)
    assert demo.camera.tile_width == pytest.approx(was * ZOOM_STEP**WHEEL_PRESSES)
    controls.on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, -1.0)
    assert demo.camera.tile_width == pytest.approx(was)


def test_the_zoom_stops_at_both_ends_and_does_not_drift_there() -> None:
    """A wheel turned past a bound changes nothing at all.

    A view that kept moving its offset at the bound would drift sideways under
    a gesture that did not change the size of a tile.
    """
    demo = build()
    controls = Controls(demo)
    for _ in range(40):
        controls.on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, 1.0)
    assert demo.camera.tile_width == pytest.approx(MAX_TILE)
    stopped = demo.camera.origin_x, demo.camera.origin_y
    controls.on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, 5.0)
    assert (demo.camera.origin_x, demo.camera.origin_y) == stopped

    for _ in range(80):
        controls.on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, -1.0)
    assert demo.camera.tile_width == pytest.approx(MIN_TILE)


def test_the_sideways_wheel_of_a_trackpad_moves_nothing() -> None:
    """A trackpad reports a sideways wheel while a person scrolls up and down."""
    demo = build()
    was = repr(demo.camera)
    Controls(demo).on_mouse_scroll(GRAB_X, GRAB_Y, 6.0, 0.0)
    assert repr(demo.camera) == was


@pytest.mark.parametrize("button", [RIGHT_BUTTON, MIDDLE_BUTTON])
def test_the_lean_stops_at_both_ends(button: int) -> None:
    """The view cannot pass through the ground and cannot turn over.

    A lean over one puts the watcher past the top of the world, and the
    picture then turns over. A lean of nothing puts the watcher in the ground,
    and the page then has no height at all.
    """
    demo = build()
    controls = Controls(demo)
    far = int(2.0 / LEAN_EACH_PIXEL)
    drag(controls, button, (GRAB_X, GRAB_Y), [(0, -far)])
    assert demo.view.lean == pytest.approx(MAX_LEAN)
    drag(controls, button, (GRAB_X, GRAB_Y), [(0, far)])
    assert demo.view.lean == pytest.approx(MIN_LEAN)


@pytest.mark.parametrize("button", [RIGHT_BUTTON, MIDDLE_BUTTON])
def test_a_drag_down_leans_the_view_towards_a_plan(button: int) -> None:
    """The hand pulls the near edge of the ground towards itself.

    This is the same sense as the left drag. The hand takes hold of the ground
    and the ground follows it, so a pull towards the watcher lays the ground
    flat.
    """
    demo = build()
    was = demo.view.lean
    drag(Controls(demo), button, (GRAB_X, GRAB_Y), [(0, -20)])
    assert demo.view.lean == pytest.approx(was + 20 * LEAN_EACH_PIXEL)


@pytest.mark.parametrize("button", [RIGHT_BUTTON, MIDDLE_BUTTON])
def test_a_drag_across_turns_the_view_about_the_up_direction(button: int) -> None:
    """A sideways drag turns the ground, and a whole turn comes back round."""
    demo = build()
    was = demo.view.turn
    drag(Controls(demo), button, (GRAB_X, GRAB_Y), [(30, 0)])
    assert demo.view.turn == pytest.approx((was - 30 * TURN_EACH_PIXEL) % (2 * math.pi))

    view = View(turn=0.1)
    view.orbit_by(2.0 * math.pi)
    assert view.turn == pytest.approx(0.1)


def test_a_turn_and_a_lean_leave_the_camera_where_it_stood() -> None:
    """The two angles are their own axis. A turn is not a pan.

    A control that moved the offset while it turned would walk the view off
    the place the person was looking at.
    """
    demo = build()
    was = repr(demo.camera)
    drag(Controls(demo), RIGHT_BUTTON, (GRAB_X, GRAB_Y), [(30, -20), (10, 5)])
    assert repr(demo.camera) == was


def test_a_left_drag_leaves_the_two_angles_where_they_stood() -> None:
    """A pan is not a turn, in the same way that a turn is not a pan."""
    demo = build()
    stood = demo.view.turn, demo.view.lean
    drag(Controls(demo), LEFT_BUTTON, (GRAB_X, GRAB_Y), [(30, -20)])
    assert (demo.view.turn, demo.view.lean) == stood


def test_a_click_names_a_tile_and_a_drag_names_none() -> None:
    """A person who drags the map is moving the map, not choosing a tile."""
    demo = build()
    said: list[str] = []
    controls = Controls(demo, said.append)

    controls.on_mouse_press(GRAB_X, GRAB_Y, LEFT_BUTTON, 0)
    controls.on_mouse_release(GRAB_X, GRAB_Y, LEFT_BUTTON, 0)
    assert demo.pointer == ground_under(demo, GRAB_X, GRAB_Y)
    assert len(said) == 1

    drag(controls, LEFT_BUTTON, (GRAB_X, GRAB_Y), [(-25, 12)])
    assert len(said) == 1


def test_the_view_holds_the_camera_the_frame_is_drawn_with() -> None:
    """One object says where the watcher stands, and the camera is part of it.

    A second place that held the position would be read back correctly and
    would change nothing, which is a failure nobody sees.
    """
    demo = build()
    assert demo.camera is demo.view.camera
    fresh = Camera(tile_size=8.0)
    demo.camera = fresh
    assert demo.view.camera is fresh
    Controls(demo).on_mouse_scroll(GRAB_X, GRAB_Y, 0.0, 1.0)
    assert demo.view.camera.tile_width > 8.0


def test_the_view_opens_at_the_angles_of_an_isometric_drawing() -> None:
    """The page opens where it always opened.

    A run that touches no mouse therefore draws what it drew before.
    """
    view = View()
    assert view.turn == pytest.approx(math.pi / 4.0)
    assert view.lean == pytest.approx(0.5)
    assert MIN_LEAN < view.lean < MAX_LEAN
    assert MIN_TILE < view.camera.tile_width < MAX_TILE


def test_the_button_numbers_match_the_ones_the_window_library_uses() -> None:
    """The mouse layer names three buttons, and the library numbers them.

    **This is the check that a second copy of a value needs.** The layer holds
    its own three numbers, so that a caller can name a button without the
    library installed. Nothing would fail if the two parted company, so this
    fails instead.
    """
    pyglet = pytest.importorskip("pyglet")
    assert LEFT_BUTTON == pyglet.window.mouse.LEFT
    assert MIDDLE_BUTTON == pyglet.window.mouse.MIDDLE
    assert RIGHT_BUTTON == pyglet.window.mouse.RIGHT


def test_a_window_can_hold_the_controls_by_weak_reference() -> None:
    """The library takes the handler weakly, so the push must succeed.

    The earlier test named the handler methods against the library's event
    list, which is the strongest check a machine with no display can make
    from names alone. It passed while the push itself raised, because a
    class with no dictionary cannot be referred to weakly unless it names
    the slot. This drives the real call instead of the names.[^1]

    [^1]: Findings register, FND-595. `docs/FINDINGS.md`
    """
    from pyglet.event import EventDispatcher

    class Surface(EventDispatcher):
        pass

    for name in (
        "on_mouse_press",
        "on_mouse_drag",
        "on_mouse_release",
        "on_mouse_scroll",
    ):
        Surface.register_event_type(name)

    demo = build()
    controls = Controls(demo)
    surface = Surface()

    surface.push_handlers(controls)

    surface.dispatch_event("on_mouse_scroll", 10, 10, 0, 1)
