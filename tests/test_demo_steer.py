"""The keys that scroll the map, and the page they scroll it across.

**A press names a direction across the frame.** The sketch renderer stands the
page at an angle, so the ground under a press moves in another direction. The
drag took the angle out and the keys did not, and the keys therefore scrolled
along the axes of the flat map while the picture stood turned.[^1]

These tests drive the state of a run rather than a window, because the gate has
no display. They call the same method the frame loop calls.

References
----------
Findings register, FND-630. ``docs/FINDINGS.md``

Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``

ADR-0067, the viewer reads the world and never writes to it, decision D2.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

import math

import pytest

from cachette import Camera, World
from cachette.demo.app import Demo
from cachette.demo.sketch import ROW_PITCH, Sketch
from cachette.names import Names

SIDE = 160
WIDTH = 640
HEIGHT = 480
SEED = 0x0123_4567_89AB_CDEF

# A quarter turn. At this angle the across axis of the frame lies along the
# down axis of the map, so a press that only moved along the map axes shows up
# as a step with no cross term at all.
QUARTER = math.pi / 2.0


def build(sketch: bool) -> Demo:
    """Give back a run pointed at the middle of a world.

    The sketch renderer here is the plain one and not the device one. The
    device one needs a frame buffer, and the gate has no display.
    """
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=2)
    world.seed_world()
    demo = Demo(world, Names(SEED), WIDTH, HEIGHT)
    demo.camera = Camera(tile_size=16.0)
    demo.open_on((SIDE // 2, SIDE // 2))
    if sketch:
        demo.renderer = Sketch(world, view=demo.view)
    return demo


def where(demo: Demo) -> tuple[float, float]:
    """Give back where the camera stands."""
    return demo.camera.origin_x, demo.camera.origin_y


def test_a_press_scrolls_along_the_frame_and_not_along_the_flat_map() -> None:
    """A quarter turn sends a sideways press down the map instead of across.

    **This is the defect a reader cannot see and a person feels at once.** The
    picture is turned, the person presses the key for right, and the map goes
    somewhere that is not right.
    """
    demo = build(sketch=True)
    demo.view.turn = QUARTER
    was = where(demo)
    demo.steer(1.0, 0.0, 0)
    moved_x = demo.camera.origin_x - was[0]
    moved_y = demo.camera.origin_y - was[1]
    assert moved_x != pytest.approx(0.0) or moved_y != pytest.approx(0.0)
    # At a quarter turn the across axis of the frame has swung onto the down
    # axis of the map, so the step the map takes is the one it did not take
    # before.
    assert abs(moved_y) > abs(moved_x)


def test_a_press_scrolls_across_the_map_when_the_page_is_not_turned() -> None:
    """The turn is the only thing that changes the press.

    A test of the turned page proves nothing about the flat one. This states
    that a press with no turn reaches the scroll it always did.
    """
    demo = build(sketch=True)
    demo.view.turn = 0.0
    was = where(demo)
    demo.steer(1.0, 0.0, 0)
    assert demo.camera.origin_x - was[0] != pytest.approx(0.0)
    assert demo.camera.origin_y - was[1] == pytest.approx(0.0)


def test_the_engine_renderer_scrolls_by_the_press_it_was_given() -> None:
    """A renderer that draws square to the ground turns nothing.

    The engine renderer holds no angle, so it publishes no step of its own and
    the press must reach the camera as it was given.
    """
    demo = build(sketch=False)
    demo.view.turn = QUARTER
    was = where(demo)
    demo.steer(1.0, 0.0, 0)
    assert demo.camera.origin_x - was[0] != pytest.approx(0.0)
    assert demo.camera.origin_y - was[1] == pytest.approx(0.0)


class Watched:
    """A renderer that records every step it was asked to turn.

    **The renderer cannot be watched by putting a method on one.** Both sketch
    renderers declare their fields, so nothing can be added to an instance of
    either. A stub is the better watcher in any case: it proves that the run
    asks the renderer at all, and not only that one particular renderer
    answers.
    """

    def __init__(self) -> None:
        """Start with no step recorded."""
        self.asked: list[tuple[float, float]] = []

    def ground_step(
        self, camera: Camera, across: float, down: float
    ) -> tuple[float, float]:
        """Record the step, and give it back turned by a quarter."""
        del camera
        self.asked.append((across, down))
        return -down, across

    def __call__(self, *arguments: object, **named: object) -> None:
        """Refuse to draw. A renderer draws a frame, and nothing here does."""
        raise AssertionError("this test draws no frame")


def test_the_keys_and_the_hand_ask_the_page_the_same_question() -> None:
    """One method answers both, so the two cannot part company.

    The drag took the angle out and the press did not. A copy of the rule
    beside each of them could drift again with nothing failing, so this states
    that both reach the one answer, and that both reach it through the
    renderer.
    """
    demo = build(sketch=True)
    watcher = Watched()
    demo.renderer = watcher  # type: ignore[assignment]
    demo.steer(1.0, -1.0, 0)
    demo.drag_ground(3.0, 5.0)
    assert watcher.asked == [(1.0, -1.0), (3.0, 5.0)]


def test_a_turned_press_moves_the_picture_the_way_the_person_asked() -> None:
    """The scroll is the inverse of the page, at every angle.

    A step across the ground draws at some place on the page. This asks the
    page where the step went, and the answer must lie along the direction the
    press named. A rotation the wrong way round passes the test above, because
    that test only reads which axis carries the larger part.

    **The page is not a pure rotation.** The flat map spaces its rows by the
    height of a tile and the plan of a hex grid spaces them closer, so the
    inverse below divides the row part by that pitch before it turns. A test
    that left the pitch out reports a defect that is not there.
    """
    demo = build(sketch=True)
    demo.view.lean = 1.0
    camera = demo.camera
    pitch = float(camera.tile_width) / (float(camera.tile_height) * ROW_PITCH)
    for turn in (0.0, 0.7, QUARTER, 2.5, 4.0):
        demo.view.turn = turn
        across, down = demo.frame_step_to_ground(1.0, 0.0)
        # Put the ground step back through the page. The pitch comes out
        # first, and then the turn the page applies, which is the turn the
        # step took out.
        plan_x, plan_y = across, down / pitch
        page_x = plan_x * math.cos(turn) - plan_y * math.sin(turn)
        page_y = plan_x * math.sin(turn) + plan_y * math.cos(turn)
        assert page_x > 0.0, f"a press for right went left at turn {turn}"
        assert page_y == pytest.approx(0.0, abs=1e-9), (
            f"a press for right drifted up or down at turn {turn}"
        )
