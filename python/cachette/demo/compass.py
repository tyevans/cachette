"""The compass of the demonstration: which way the ground now lies.

A page that stands at an angle to the ground can be turned and leaned, and
nothing on the frame said so. A control that nobody finds is a control that
does not exist.

This module draws a small compass in the lower left corner of the frame. It
answers two questions that a line of instructions cannot.

**It says how far the ground is turned right now.** A watcher who has turned
the view has no other way to learn which way north lies, or how to get back to
where the view opened.

**It says that the ground turns at all.** The needle moves while a person
drags, so the gesture explains itself the first time it happens by accident.

**It draws nothing of the world.** It reads the two angles and the size of the
frame, and it writes to the pixels the caller owns. It reads no tile, no unit
and no camera.[^1]

Where it stands
---------------

The lower left corner is the one corner that nothing else holds. The minimap
holds the upper right. The engine puts a card in the upper right while the
reference key is held. The panels run down the left from the top, and the
clock stands at the upper left.

References
----------
ADR-0067, the viewer reads the world and never writes to it, decision D2.
``docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md``
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING

import numpy as np

from cachette.demo.sketch import Projection
from cachette.demo.text import paint_text, text_height, text_width

if TYPE_CHECKING:
    from cachette.demo.surface import Surface
    from cachette.demo.view import View

__all__ = ["Compass"]

# How wide the compass is, in pixels, and how far it stands from the corner.
DIAMETER = 82
MARGIN = 14

# The ink of the compass and the paper it sits on, as one packed colour each.
#
# **The compass is drawn in the ink of the page.** It is a mark on the same
# paper, not a panel over it, so it takes the colours of a pencil study.
INK = 0x252320
PAPER = 0xF2ECDD

# How much of the paper reaches the disc, and how much of the ink reaches a
# line of the compass. The disc is a wash and the lines are drawn over it.
DISC_WEIGHT = 0.62
LINE_WEIGHT = 0.95
FAINT_WEIGHT = 0.46

# How long the needle is and how wide its head is, as shares of the radius.
NEEDLE_REACH = 0.78
NEEDLE_WIDE = 0.26

# How far the caption stands below the disc, in pixels.
CAPTION_GAP = 3

# The largest share of the frame the compass may take, across and down.
#
# **A compass that fills a corner of a small frame reads as furniture.** It is
# a mark in the margin of a drawing, so it stands down when the frame is too
# small to leave it a margin.
ROOM_ACROSS = 0.30
ROOM_DOWN = 0.30

# What the caption says.
#
# **It names the gesture and not the key.** The turn has no key, and a person
# who reads this is looking for what to do with the hand already on the mouse.
CAPTION = "right-drag turns"


# The window, the frame and the two places the needle asks the projection
# about. The two places are one row apart, so the step between them on the page
# is the direction north lies in. The size of the probe does not reach the
# answer, because the step is taken as a direction and then made one long.
_PROBE_WINDOW = (0, 0, 4, 4)
_PROBE_SIDE = 100
_PROBE_Q = np.array([2.0, 2.0], dtype=np.float64)
_PROBE_R = np.array([2.0, 1.0], dtype=np.float64)


def _disc(radius: int) -> np.ndarray:
    """Give back which point of a square stands inside the disc."""
    across = np.arange(-radius, radius + 1, dtype=np.float32)
    away = np.hypot(across[None, :], across[:, None])
    return np.asarray(away <= radius)


def _blend(under: np.ndarray, colour: int, weight: np.ndarray) -> np.ndarray:
    """Mix one colour into a block of packed pixels, point by point."""
    bands = []
    for shift in (16, 8, 0):
        was = ((under >> shift) & 0xFF).astype(np.float32)
        now = float((colour >> shift) & 0xFF)
        bands.append(np.clip(was + (now - was) * weight, 0.0, 255.0).astype(np.uint32))
    mixed: np.ndarray = (bands[0] << 16) | (bands[1] << 8) | bands[2]
    return mixed


def _line(
    reach: np.ndarray,
    along: np.ndarray,
    length: float,
    half: float,
) -> np.ndarray:
    """Give back the weight of a spike that runs one way from the middle.

    The spike reaches the length given and narrows to nothing at its point,
    so it reads as a drawn needle rather than as a bar.
    """
    part = np.clip(along / max(length, 1e-3), 0.0, 1.0)
    room = half * (1.0 - part)
    inside = (along >= 0.0) & (along <= length) & (np.abs(reach) <= room)
    return np.asarray(inside).astype(np.float32)


class Compass:
    """The compass of the demonstration, drawn over the frame.

    A caller builds one and paints it after the renderer filled the frame.

    **It holds the picture it drew last, against the angles it drew it at.**
    The compass changes only when the view turns or leans, and a drag is the
    only thing that turns the view, so a still frame paints the same disc
    again for nothing.
    """

    __slots__ = ("_at", "_ink", "_mask", "visible")

    def __init__(self) -> None:
        """Build the compass, shown."""
        self.visible = True
        # The angles the held picture stands at, the weight of the ink at each
        # point of it, and which points it covers at all.
        self._at: tuple[float, float] | None = None
        self._ink: np.ndarray | None = None
        self._mask: np.ndarray | None = None

    def toggle(self) -> bool:
        """Show the compass or hide it, and say which it now is."""
        self.visible = not self.visible
        return self.visible

    def _drawn(self, turn: float, lean: float) -> tuple[np.ndarray, np.ndarray]:
        """Give back the ink of the compass at these angles, building it once."""
        held, ink, mask = self._at, self._ink, self._mask
        if held == (turn, lean) and ink is not None and mask is not None:
            return ink, mask
        radius = DIAMETER // 2
        across = np.arange(-radius, radius + 1, dtype=np.float32)
        page_x = np.broadcast_to(across[None, :], (2 * radius + 1, 2 * radius + 1))
        page_y = np.broadcast_to(across[:, None], (2 * radius + 1, 2 * radius + 1))
        inside = _disc(radius)

        # **The needle stands where the page puts north, and the page says
        # where that is.** North is the row the world numbers first. A copy of
        # the turn here would be a fourth place that states the projection, so
        # this asks the projection for two neighbouring rows and takes the
        # step between them.
        stood = Projection(_PROBE_WINDOW, _PROBE_SIDE, _PROBE_SIDE, turn, lean, 0.1)
        at_column, at_row = stood.to_page(_PROBE_Q, _PROBE_R)
        aim_x = float(at_column[1] - at_column[0])
        aim_y = float(at_row[1] - at_row[0])
        size = math.hypot(aim_x, aim_y)
        weight = np.zeros_like(page_x)
        if size > 1e-6:
            aim_x, aim_y = aim_x / size, aim_y / size
            for way, length, half, full in (
                (1.0, NEEDLE_REACH, NEEDLE_WIDE, True),
                (-1.0, NEEDLE_REACH * 0.62, NEEDLE_WIDE * 0.7, False),
            ):
                along = (page_x * aim_x + page_y * aim_y) * way
                reach = page_x * -aim_y + page_y * aim_x
                spike = _line(reach, along, radius * length, radius * half)
                weight = np.maximum(weight, spike * (1.0 if full else 0.55))

        # The rim of the disc, drawn as a ring one point thick.
        away = np.hypot(page_x, page_y)
        rim = (away <= radius) & (away >= radius - 1.4)
        weight = np.maximum(weight, rim.astype(np.float32) * 0.75)
        weight = np.where(inside, weight, 0.0)

        self._at = (turn, lean)
        self._ink = weight
        self._mask = inside
        return weight, inside

    def paint(self, view: View, surface: Surface) -> bool:
        """Paint the compass over the frame, and say whether it painted.

        A hidden compass paints nothing. A frame too small to hold the
        compass paints nothing, because a compass cut by the edge of the
        frame points somewhere it does not mean.
        """
        if not self.visible:
            return False
        caption = text_width(CAPTION, 1)
        wide = max(DIAMETER, caption)
        tall = DIAMETER + CAPTION_GAP + text_height(1)
        if surface.width < wide + MARGIN * 2 or surface.height < tall + MARGIN * 2:
            return False
        if wide > surface.width * ROOM_ACROSS or tall > surface.height * ROOM_DOWN:
            return False
        weight, inside = self._drawn(view.turn, view.lean)
        rows, columns = weight.shape
        left = MARGIN
        top = surface.height - MARGIN - tall
        frame = surface.pixels.reshape(surface.height, surface.width)
        block = frame[top : top + rows, left : left + columns]
        # The paper goes down first, so the compass reads against the hatch of
        # the page rather than through it.
        laid = _blend(block, PAPER, np.where(inside, DISC_WEIGHT, 0.0))
        frame[top : top + rows, left : left + columns] = _blend(
            laid, INK, weight * LINE_WEIGHT
        )
        paint_text(
            surface,
            left + max((DIAMETER - caption) // 2, 0),
            top + DIAMETER + CAPTION_GAP,
            CAPTION,
            INK,
            weight=FAINT_WEIGHT,
        )
        return True
