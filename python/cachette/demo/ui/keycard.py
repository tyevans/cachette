"""The card that names every key.

**A key nobody can find does not exist.** The run prints its keys to the
terminal once, and a person who opened the window from a launcher never sees
that. This card puts the same list on the frame.

The panel keys and the overlay keys are not here. The engine publishes those
lists and the menu builds a row for each of them, so a copy here would go
stale the moment the engine gained one.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from cachette.demo.ui import theme

if TYPE_CHECKING:
    from cachette.demo.surface import Surface

# The keys the card names, as a key and what it does.
#
# A blank key draws a rule, which groups the list without a second heading.
LINES: tuple[tuple[str, str], ...] = (
    ("ESC", "OPEN THE MENU"),
    ("/", "THIS CARD"),
    ("", ""),
    ("DRAG LEFT", "MOVE THE MAP"),
    ("DRAG RIGHT", "TURN AND LEAN"),
    ("CTRL DRAG", "TURN AND LEAN"),
    ("WHEEL", "ZOOM AT THE CURSOR"),
    ("WASD", "SCROLL"),
    ("- AND =", "ZOOM"),
    ("CLICK", "POINT AT A TILE"),
    ("", ""),
    ("SPACE", "STOP AND START"),
    (".", "ONE TICK"),
    ("[ AND ]", "SLOWER AND FASTER"),
    ("", ""),
    ("M", "THE MINIMAP"),
    ("C", "THE COMPASS"),
    ("TAB", "NAME THE COLOURS"),
    ("0 TO 9", "THE OVERLAYS"),
    ("F1 UP", "THE PANELS"),
    ("", ""),
    ("ENTER", "TAKE THE CHOSEN ACTION"),
    ("E", "END THE TURN"),
)

# How wide the card is, in pixels.
WIDTH = 470


def paint(surface: Surface, weight: float = 1.0) -> None:
    """Draw the card of the keys, centred in the frame."""
    head = theme.PADDING + theme.row_height(theme.TITLE_SCALE) + 8
    rows = sum(
        theme.row_height() if key or words else theme.ROW_GAP + 2
        for key, words in LINES
    )
    height = head + rows + theme.PADDING
    left = (surface.width - WIDTH) // 2
    top = max(theme.MARGIN, (surface.height - height) // 2)
    theme.card(surface, left, top, WIDTH, height, weight)
    row = theme.title(surface, left, top + theme.PADDING, WIDTH, "THE KEYS", weight)
    for key, words in LINES:
        if not key and not words:
            row = theme.divider(surface, left, row, WIDTH, weight)
            continue
        row = theme.pair(surface, left, row, WIDTH, key, words, card_weight=weight)
