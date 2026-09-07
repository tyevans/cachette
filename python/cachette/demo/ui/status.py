"""The title block, which says where the watcher is and how to reach the rest.

**A control nobody can find does not exist.** The interface therefore keeps
one small card on the frame at all times. It names the tick, the speed and the
mode, and it names the two keys that open everything else.

A drawing carries its title block in a corner, so this card sits in the lower
left, beside the compass. It never moves and it never changes width, because a
card that resized as the tick grew would read as a flicker.

The card holds no state. The caller passes the run, and this reads three
numbers from it.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from cachette.demo.clock import says
from cachette.demo.compass import CAPTION as COMPASS_CAPTION
from cachette.demo.compass import DIAMETER as COMPASS_DIAMETER
from cachette.demo.compass import MARGIN as COMPASS_MARGIN
from cachette.demo.ui import theme

if TYPE_CHECKING:
    from cachette.demo.surface import Surface
    from cachette.demo.ui.host import Host

# How far the card sits from the left edge and from the foot of the frame.
#
# **The compass sits in the same corner, and it declares its own size.** The
# card clears the disc and the caption under it, so neither covers the other.
# The caption is wider than the disc, so the room it needs is read from the
# words rather than guessed.
LEFT = (
    COMPASS_MARGIN
    + max(COMPASS_DIAMETER, theme.line_width(COMPASS_CAPTION.upper(), 1))
    + theme.MARGIN
)
FOOT = 16

# How wide the card is, in pixels.
#
# **The width is fixed and not read from the words.** A tick of four digits
# and a tick of five would give two widths, and the card would appear to jump
# every ten thousand ticks.
WIDTH = 300

# The words the card shows for the two modes.
WATCHING = "WATCHING"

# The line that names the keys that open the rest of the interface.
WAY_IN = "ESC MENU"
WAY_TO_KEYS = "/ KEYS"


def height_of(host: Host) -> int:
    """Give back how tall the title block is now, in pixels.

    The block grows by one row while a person holds a faction. **One function
    states the height**, so the drawing and everything that must clear the
    block read one number.
    """
    rows = 3 if host.seat is not None else 2
    return (
        theme.PADDING
        + theme.row_height(theme.TITLE_SCALE)
        + 8
        + rows * theme.row_height()
        + theme.PADDING
    )


def clears(host: Host) -> int:
    """Give back how many rows at the foot of the frame the block takes."""
    return FOOT + height_of(host) + theme.ROW_GAP


def paint(surface: Surface, host: Host) -> None:
    """Draw the title block into the lower left of the frame."""
    seat = host.seat
    height = height_of(host)
    top = surface.height - FOOT - height
    if top < 0 or LEFT + WIDTH > surface.width:
        return
    theme.card(surface, LEFT, top, WIDTH, height)
    heading = host.faction_name(seat.faction).upper() if seat is not None else WATCHING
    row = theme.title(surface, LEFT, top + theme.PADDING, WIDTH, heading)
    speed = "STOPPED" if host.clock.paused else says(host.clock.speed).upper()
    if seat is not None:
        state = "CHOOSING" if seat.frozen else f"RUNNING {seat.left}"
        row = theme.pair(surface, LEFT, row, WIDTH, f"TURN {seat.turn}", state)
    row = theme.pair(surface, LEFT, row, WIDTH, f"TICK {host.world.tick}", speed)
    theme.pair(
        surface,
        LEFT,
        row,
        WIDTH,
        WAY_IN,
        WAY_TO_KEYS,
        weight=theme.QUIET_WEIGHT,
    )
