"""The card a person chooses their turn from.

**The card shows the mask the engine gave, and nothing else.** The legality of
each row comes from the engine reader that a learning policy also calls, and
that reader is fog scoped. A row that names ground the faction has never seen
is refused for the person in the same way it is refused for the policy, and
this module never widens it.

The card lists the rows that are legal now. The table holds rows that no
faction can take at most moments, and a list of thirty lines of which seven
can be chosen makes a person read the twenty-three that cannot. The foot of
the card states how many rows the table holds, so the person knows the list is
a selection.

The layout comes from one function, so a click always names the row the caret
was on.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, NamedTuple

from cachette.demo.ui import theme
from cachette.demo.ui.menu import CARET, NO_CARET

if TYPE_CHECKING:
    from cachette.demo.player import Choice, Seat
    from cachette.demo.surface import Surface

# How far the card sits from the right edge and from the foot of the frame.
RIGHT_MARGIN = 16
FOOT = 16

# How wide the card is, in pixels.
#
# The width is fixed, so a turn in which one long row falls out of the list
# does not change the shape of the card.
WIDTH = 360

# How many rows the card shows at most.
#
# A short frame cannot hold every legal row. The card shows what fits and the
# foot says how many it left out, rather than running off the bottom.
MOST_ROWS = 18

# The words on the last line of the card.
KEY_LINE = "ENTER TAKES   E ENDS TURN"


class Layout(NamedTuple):
    """Where the card sits, and where each row of it sits."""

    left: int
    top: int
    width: int
    height: int
    rows: list[int]
    shown: list[Choice]


def lay_out(
    seat: Seat, choices: list[Choice], surface: Surface, clear_top: int = 0
) -> Layout:
    """Give back where the card and its rows land in the frame.

    **This is the one declaration of the layout.** The drawing calls it and
    the mouse calls it.
    """
    shown = [choice for choice in choices if choice.legal][:MOST_ROWS]
    taken = seat.taken[-3:]
    head = theme.PADDING + theme.row_height(theme.TITLE_SCALE) + 8
    body = max(len(shown), 1) * theme.row_height()
    tail = (theme.ROW_GAP + 2) + (3 + len(taken)) * theme.row_height() + theme.PADDING
    height = head + body + tail
    left = surface.width - RIGHT_MARGIN - WIDTH
    # **The card grows upward from the foot.** The minimap sits in the top
    # right corner, and a card centred down the right edge runs under it. A
    # card anchored to the foot clears the disc whatever it lists, and it
    # keeps the horizon of the drawing open.
    top = max(clear_top, surface.height - FOOT - height)
    places: list[int] = []
    row = top + head
    for _ in shown:
        places.append(row)
        row += theme.row_height()
    return Layout(left, top, WIDTH, height, places, shown)


def paint(
    surface: Surface,
    seat: Seat,
    chosen: int,
    weight: float = 1.0,
    clear_top: int = 0,
) -> Layout:
    """Draw the card of the turn, and give back where it landed."""
    choices = seat.actions()
    plan = lay_out(seat, choices, surface, clear_top)
    theme.card(surface, plan.left, plan.top, plan.width, plan.height, weight)
    theme.title(
        surface,
        plan.left,
        plan.top + theme.PADDING,
        plan.width,
        f"TURN {seat.turn}",
        weight,
    )
    if not plan.shown:
        theme.row(
            surface,
            plan.left,
            plan.top + theme.PADDING + theme.row_height(theme.TITLE_SCALE) + 8,
            plan.width,
            "NOTHING IS LEGAL NOW",
            weight=theme.QUIET_WEIGHT,
            card_weight=weight,
        )
    for at, top in enumerate(plan.rows):
        theme.row(
            surface,
            plan.left,
            top,
            plan.width,
            (CARET if at == chosen else NO_CARET) + plan.shown[at].says(),
            chosen=at == chosen,
            card_weight=weight,
        )
    row = (
        (plan.rows[-1] + theme.row_height())
        if plan.rows
        else (
            plan.top
            + theme.PADDING
            + theme.row_height(theme.TITLE_SCALE)
            + 8
            + theme.row_height()
        )
    )
    row = theme.divider(surface, plan.left, row, plan.width, weight)
    legal = sum(1 for choice in choices if choice.legal)
    theme.pair(
        surface,
        plan.left,
        row,
        plan.width,
        f"{legal} OF {len(choices)} LEGAL",
        f"{len(seat.taken)} TAKEN",
        weight=theme.QUIET_WEIGHT,
        card_weight=weight,
    )
    for line in seat.taken[-3:]:
        row += theme.row_height()
        theme.row(
            surface,
            plan.left,
            row,
            plan.width,
            "  " + line,
            weight=theme.REFUSED_WEIGHT,
            card_weight=weight,
        )
    theme.row(
        surface,
        plan.left,
        row + theme.row_height(),
        plan.width,
        KEY_LINE,
        weight=theme.QUIET_WEIGHT,
        card_weight=weight,
    )
    return plan


def row_at(plan: Layout, x: int, y: int) -> int | None:
    """Give back the row a place in the frame lands on, or nothing."""
    if not (plan.left <= x < plan.left + plan.width):
        return None
    for at, top in enumerate(plan.rows):
        if top - theme.ROW_GAP // 2 <= y < top + theme.row_height():
            return at
    return None


def inside(plan: Layout, x: int, y: int) -> bool:
    """Say whether a place in the frame lands on the card at all."""
    return (
        plan.left <= x < plan.left + plan.width
        and plan.top <= y < plan.top + plan.height
    )
