"""The mouse controls of the demonstration.

This module turns the events the window library delivers into changes to the
view. **It draws nothing and it reads no tile.** The view is the one place the
position of the watcher is held, and this module is the one place a mouse
reaches it.[^1]

What each gesture does
----------------------

**The left button drags the ground.** The ground stays under the cursor while
the button is down, so the hand takes hold of the map and moves it. A control
that pushed the camera instead would move the ground the other way, and a
person who has used a map expects to grab it.

**The wheel zooms about the cursor.** The point under the cursor stays under
the cursor while the size of a tile changes. A zoom about the middle of the
frame throws away the one thing the person was pointing at.

**The right button and the middle button turn and lean the view.** A drag
across turns the ground about the up direction. A drag down leans the view
towards a plan seen from above, in the same sense as the left drag: the hand
pulls the near edge of the ground towards itself and the ground flattens.

The right button and the middle button both do this. A trackpad has no middle
button and a mouse with a wheel has no comfortable right drag, so a person
reaches for whichever their hardware gives them.

Which way is up
---------------

**The window numbers the rows of the screen from the bottom.** The engine
numbers them from the top. This module is the boundary between the two, so
every method here takes the numbering of the window and every call it makes
uses the numbering of the engine.

References
----------
ADR-0067, the viewer reads the world and never writes to it, decision D2.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``

Recurring Defect Shapes, redundant declaration sites.
``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from collections.abc import Callable

    from cachette import World
    from cachette.demo.surface import Surface
    from cachette.demo.view import View

# The numbers the window library gives the three buttons.
#
# **This is a second copy of a value the library owns.** The library states
# them, and a module that imported them would need the library to be installed
# before a caller could name a button. A test therefore reads the three from
# the library and fails when a copy here disagrees with it.[^2]
LEFT_BUTTON = 1
MIDDLE_BUTTON = 2
RIGHT_BUTTON = 4

# How far one pixel of a sideways drag turns the view, in radians.
#
# A drag across a window of about nine hundred pixels turns the world once.
# That is the rate a person meets in a modelling tool, where a short drag
# turns the subject far enough to read the other side of it.
TURN_EACH_PIXEL = 2.0 * math.pi / 900.0

# How far one pixel of a drag down leans the view.
#
# The lean runs over a range of less than one, and a drag of about four
# hundred pixels covers the whole of it. A shorter drag would make the lean
# hard to place, and a longer one would make a person drag twice.
LEAN_EACH_PIXEL = 1.0 / 400.0


class Watched(Protocol):
    """What this module needs from the state the control plane holds.

    A mouse moves the view and it names the tile under the cursor. It reads
    the world for the bound on how far the view may travel, and it reads the
    size of the frame for the same bound.
    """

    view: View
    world: World
    surface: Surface
    pointer: tuple[int, int] | None

    def drag_ground(self, across: float, down: float) -> None:
        """Move the ground under the hand by a step across the frame."""


class Controls:
    """The mouse of the demonstration, as the window library delivers it.

    A caller pushes one of these onto its window. The method names are the
    names the library calls, so the library reaches them and a test reaches
    the same methods with the same arguments.

    **The gesture writes to the view and to nothing else.** The renderer reads
    the view on the next frame, so a change to what draws leaves this alone.
    """

    # The window library holds a handler by weak reference, so a class with
    # no dictionary must name the weak reference slot. Without it the push
    # raises, and the failure is invisible to a test that has no window.[^1]
    #
    # [^1]: Findings register, FND-595. `docs/FINDINGS.md`
    __slots__ = ("__weakref__", "_demo", "_dragged", "_say")

    def __init__(self, demo: Watched, say: Callable[[str], None] | None = None) -> None:
        """Hold the state the gestures move.

        The second argument takes a line of text. A click that never moved
        names the tile it landed on, and this is where that line goes. A
        caller that wants no line passes nothing.
        """
        self._demo = demo
        self._say = say
        # How far the pointer has travelled since the button went down, in
        # pixels. A press that never moves names a tile. A press that moves
        # is a drag, and it names nothing.
        self._dragged = 0.0

    def _size(self) -> tuple[int, int]:
        """Give back the size of the frame in pixels."""
        return self._demo.surface.width, self._demo.surface.height

    def _hold(self) -> None:
        """Hold the view inside the world after a gesture moved it."""
        width, height = self._size()
        self._demo.view.clamp(self._demo.world, width, height)

    def on_mouse_press(self, x: int, y: int, button: int, modifiers: int) -> None:
        """Take note that a button went down, and name the tile under it.

        The arguments are the ones the window library delivers: the place of
        the cursor, the button, and the keys that were held.
        """
        del modifiers
        self._dragged = 0.0
        if button == LEFT_BUTTON:
            self._demo.pointer = self._demo.view.camera.tile_at(
                float(x), float(self._demo.surface.height - y)
            )

    def on_mouse_drag(
        self, x: int, y: int, dx: int, dy: int, buttons: int, modifiers: int
    ) -> None:
        """Move the view while a button is down.

        The arguments are the ones the window library delivers: the place of
        the cursor, how far it moved, the buttons that are down as one number
        of bits, and the keys that were held.
        """
        del x, y, modifiers
        self._dragged += abs(dx) + abs(dy)
        if buttons & LEFT_BUTTON:
            # The window counts the rows up and the frame counts them down,
            # so the drag down the screen is the drag up the frame.
            #
            # **The mouse asks for a step across the frame and no more.** A
            # page that stands at an angle turns the ground, so the step the
            # camera must take is not the step the hand made. The renderer
            # that draws the angles is the one that inverts them, and this
            # layer holds no copy of either.
            self._demo.drag_ground(float(dx), float(-dy))
            self._hold()
            return
        if buttons & (RIGHT_BUTTON | MIDDLE_BUTTON):
            self._demo.view.orbit_by(-float(dx) * TURN_EACH_PIXEL)
            self._demo.view.tilt_by(-float(dy) * LEAN_EACH_PIXEL)

    def on_mouse_release(self, x: int, y: int, button: int, modifiers: int) -> None:
        """Take note that the button came up, and name the tile of a click.

        **A click names a tile and a drag names nothing.** A person who drags
        the map across the window is moving the map, and a line that named the
        tile they let go over would say something they did not ask for.
        """
        del x, y, modifiers
        clicked = self._dragged == 0.0
        self._dragged = 0.0
        if not clicked or button != LEFT_BUTTON:
            return
        at = self._demo.pointer
        if at is not None and self._say is not None:
            self._say(f"pointing at tile ({at[0]}, {at[1]})")

    def on_mouse_scroll(self, x: int, y: int, scroll_x: float, scroll_y: float) -> None:
        """Zoom about the cursor by a count of wheel notches.

        The arguments are the ones the window library delivers: the place of
        the cursor, and how far the wheel turned in each direction.

        **The sideways wheel does nothing.** A trackpad reports it while a
        person scrolls up and down, and a view that panned on it would drift
        sideways under a gesture that meant to zoom.
        """
        del scroll_x
        if not scroll_y:
            return
        self._demo.view.zoom_at(
            float(scroll_y), float(x), float(self._demo.surface.height - y)
        )
        self._hold()

    def dragged(self) -> float:
        """Give back how far the pointer moved since the button went down.

        A caller reads this to tell a click from a drag.
        """
        return self._dragged
