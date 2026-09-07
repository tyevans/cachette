"""Where the watcher stands. The one source of truth for the view.

The engine camera holds where the view sits over the flat map: the size of a
tile in pixels, and the pixel offset of the tile at the origin. It holds no
angle, because the flat map has none.

A drawn page may stand at an angle to the ground. This module holds the two
angles beside the camera, so that one object answers where the view is. A
renderer reads this object. It never keeps a second copy of an angle.[^1]

**Nothing in this module draws.** The input layer writes here and the drawing
layer reads here, so a change to either one leaves the other alone.

What the two angles mean
------------------------

The turn is the angle the page turns the ground through, about the up
direction. Zero looks along the axes of the grid. An eighth of a circle is the
angle of an isometric drawing, and it is where the view opens.

The lean is how far the page tips away from the watcher. One is a plan seen
from straight above. A value near zero is a view along the ground. The lean is
therefore the sine of the angle the view stands above the ground.

The bounds of the tile size and the step of one zoom press come from the
engine and not from this module. A second copy of a bound here would be read
back correctly and would change nothing, which is a failure nobody sees.[^2]

References
----------
ADR-0094, the caller owns the camera and the pixels, decision D1.
``docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md``

Recurring Defect Shapes, redundant declaration sites.
``.agents/rules/recurring-defects.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING

from cachette import Camera

if TYPE_CHECKING:
    from cachette import World


def _engine_tile_bounds() -> tuple[float, float]:
    """Ask the engine for the smallest and the largest tile it accepts.

    The engine holds the bounds and its camera holds any size it is given to
    that range. A size below the range therefore comes back as the smallest,
    and a size above it comes back as the largest.

    **This asks rather than states.** A number written here would be a second
    declaration of a value the engine already owns, and the two would part
    company without anything failing.
    """
    return Camera(tile_size=0.0).tile_width, Camera(tile_size=1.0e9).tile_width


def _engine_zoom_step() -> float:
    """Ask the engine how far one zoom press changes the size of a tile.

    The wheel and the zoom keys must agree, so both take the step from here.
    """
    probe = Camera(tile_size=1.0)
    smallest, largest = _engine_tile_bounds()
    start = (smallest + largest) * 0.5
    probe.tile_width = start
    probe.tile_height = start
    probe.zoom_in(1, 1)
    return probe.tile_width / start


# The smallest and the largest tile the engine draws, in pixels, and the
# factor one press of a zoom key applies to the size of a tile.
MIN_TILE, MAX_TILE = _engine_tile_bounds()
ZOOM_STEP = _engine_zoom_step()

# How many zoom presses one notch of the wheel is worth.
#
# **The wheel and the zoom keys use one step.** One press of a key is a small
# change, because a person holds a key down. One notch of a wheel is a single
# act, so a notch is worth several presses. Three notches then about double
# the size of a tile, which is the range a person expects from a map.
WHEEL_PRESSES = 3

# Where the view opens, as the angle the page turns the ground through.
#
# An eighth of a circle is the angle of an isometric drawing.
OPENING_TURN = math.pi / 4.0

# How far the page leans away from the watcher when the view opens.
#
# One half is the two to one of an isometric drawing.
OPENING_LEAN = 0.5

# How far the view may lean, at each end.
#
# **The upper bound is a plan seen from straight above.** A lean over one
# would put the watcher past the top and the picture would turn over.
#
# **The lower bound keeps the watcher off the ground.** At a lean of nothing
# the view lies in the ground plane, the page has no height, and the ground
# is a line. The bound holds the view about nine degrees above the ground.
MIN_LEAN = 0.15
MAX_LEAN = 1.0

# The smallest number this module divides by, so that a camera with no size
# cannot raise.
TINY = 1.0e-3


def _held(value: float, least: float, most: float) -> float:
    """Give back the value, held between the two bounds."""
    return max(least, min(most, value))


class View:
    """Where the view stands: the camera, the turn and the lean.

    A caller reads the camera to draw the flat map, and it reads the two
    angles to draw a page that stands at an angle to the ground.

    **The pan bound needs the world, so it is a separate call.** The size of a
    tile is a property of the view alone and this class holds it. How far the
    view may travel depends on how large the world is, and this class does not
    hold a world.
    """

    __slots__ = ("camera", "lean", "turn")

    def __init__(
        self,
        camera: Camera | None = None,
        *,
        turn: float = OPENING_TURN,
        lean: float = OPENING_LEAN,
    ) -> None:
        """Build the view, at the opening angles unless the caller names one."""
        self.camera = camera if camera is not None else Camera()
        self.turn = turn
        self.lean = _held(lean, MIN_LEAN, MAX_LEAN)

    def pan_by(self, across: float, down: float) -> None:
        """Move the view by a count of pixels of the frame.

        The offsets are measured across the frame and down it, in the
        direction the frame numbers its rows.

        **The ground stays under the hand.** A positive offset moves the
        ground to the right and down the frame by that many pixels, so the
        point the hand took hold of is the point it still holds.
        """
        self.camera.pan(-across, -down)

    def zoom_at(self, notches: float, x: float, y: float) -> None:
        """Make each tile larger or smaller, about a point of the frame.

        The notches are a count of wheel notches. A positive count makes each
        tile larger. The point is measured across the frame and down it.

        **The point under the cursor stays under the cursor.** A zoom about
        the middle of the frame throws away what the person was pointing at,
        which is the one thing they were telling the view about.

        A zoom that would pass a bound of the tile size changes nothing at
        all, so a wheel turned at the end of the range cannot drift the view.
        """
        camera = self.camera
        was = camera.tile_width
        size = _held(was * ZOOM_STEP ** (notches * WHEEL_PRESSES), MIN_TILE, MAX_TILE)
        if size == was:
            return
        # Read the address under the point before the scale changes, then put
        # that address back under the point at the new scale.
        row = (y - camera.origin_y) / max(camera.tile_height, TINY)
        column = (x - camera.origin_x) / max(camera.tile_width, TINY) - row / 2.0
        camera.tile_width = size
        camera.tile_height = size
        camera.origin_x = x - (column + row / 2.0) * size
        camera.origin_y = y - row * size

    def orbit_by(self, radians: float) -> None:
        """Turn the view about the up direction.

        The turn wraps, because a view that has gone all the way round stands
        where it started.
        """
        self.turn = (self.turn + radians) % (2.0 * math.pi)

    def tilt_by(self, amount: float) -> None:
        """Lean the view further towards a plan, or further towards the ground.

        A positive amount tips the view towards a plan seen from above.

        **The lean is held at both ends.** The view cannot pass through the
        ground, and it cannot pass over the top and turn the picture over.
        """
        self.lean = _held(self.lean + amount, MIN_LEAN, MAX_LEAN)

    def clamp(self, world: World, width: int, height: int) -> None:
        """Hold the view so that the world cannot leave the frame.

        The width and the height are the size of the frame in pixels. The
        engine owns the bound, so this call states none of its own.
        """
        self.camera.clamp(world, width, height)
