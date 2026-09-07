"""The look of the interface, and the marks every card is built from.

**The interface belongs to the page the sketch draws.** The sketch draws one
ink on one paper. A card that arrived in a third colour would read as a window
from another program laid over a drawing. Every mark here is therefore the
same ink at a weight, on the same paper.

A card is a sheet laid on the drawing. It carries a shadow, so the eye reads
it as lifted; a plate rule, so it reads as drawn rather than as filled; and a
title with a rule under it, so it reads as a page of a notebook.

Nothing here reads the world. A caller passes the words in, and this paints
them. The cost of every mark follows the size of the card and never the size
of the world.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from cachette.demo.page import INK, PAPER
from cachette.demo.text import paint_block, paint_text, text_height, text_width

if TYPE_CHECKING:
    from cachette.demo.surface import Surface

# The two colours of the page. **They are not declared here.** The sketch
# declares them, and one module packs them into whole numbers for a caller
# that blends whole colours. A copy here would go stale the moment the sketch
# changed its paper, and nothing would fail.

# How much of the frame below a card the paper keeps.
#
# **A fully opaque card reads as a hole in the drawing.** A little of the page
# below it keeps the card part of the picture, in the same way the minimap
# keeps a little of the frame under its disc.
PAPER_WEIGHT = 0.955

# The shadow under a card, as a pair of offsets and the weight of each.
#
# The shadow is two steps of ink and not a blur. A blur costs a pass over the
# whole card, and two offset rectangles read as a lifted sheet at a fraction
# of the cost.
SHADOW_STEPS = ((5, 0.085), (2, 0.085))

# The weight of the outer plate rule and of the inner one, and how far the
# inner rule sits inside the outer.
RULE_WEIGHT = 0.85
INNER_RULE_WEIGHT = 0.26
INNER_RULE_INSET = 4

# The weight of the rule under a title, and of a rule between two blocks.
TITLE_RULE_WEIGHT = 0.55
DIVIDER_WEIGHT = 0.38

# The wash that marks the row the keyboard stands on, and the weight of the
# rule under it.
#
# **A selected row is marked, not filled.** A block of solid ink under a line
# would read as a highlight from another interface. A pale wash and a rule
# under the line is how a person marks a line on paper.
CHOSEN_WASH = 0.10
CHOSEN_RULE = 0.45

# How much ink a line of text takes when it is live, when it is dimmed, and
# when it names something that cannot be chosen.
LIVE_WEIGHT = 1.0
QUIET_WEIGHT = 0.62
REFUSED_WEIGHT = 0.30

# The scale of the title text and of the body text.
#
# The glyphs are five pixels wide before the scale, so a scale of two gives a
# body line that a person reads at arm's length on a large display.
TITLE_SCALE = 2
BODY_SCALE = 2

# The padding inside a card, and the gap between two rows.
PADDING = 12
ROW_GAP = 5

# The margin every card keeps from the edge of the frame.
MARGIN = 16


def row_height(scale: int = BODY_SCALE) -> int:
    """Give back the height of one row, including the gap under it."""
    return text_height(scale) + ROW_GAP


def line_width(text: str, scale: int = BODY_SCALE) -> int:
    """Give back how wide a line of text is, in pixels."""
    return text_width(text, scale)


def card_width(lines: list[str], scale: int = BODY_SCALE) -> int:
    """Give back how wide a card must be to hold these lines."""
    widest = max((text_width(line, scale) for line in lines), default=0)
    return widest + 2 * PADDING


def rule(
    surface: Surface,
    left: int,
    top: int,
    width: int,
    weight: float,
    height: int = 1,
) -> None:
    """Draw one horizontal rule of ink."""
    paint_block(surface, left, top, width, height, INK, weight)


def upright(
    surface: Surface,
    left: int,
    top: int,
    height: int,
    weight: float,
    width: int = 1,
) -> None:
    """Draw one upright rule of ink."""
    paint_block(surface, left, top, width, height, INK, weight)


def card(
    surface: Surface,
    left: int,
    top: int,
    width: int,
    height: int,
    weight: float = 1.0,
) -> None:
    """Lay a sheet of paper on the frame, with a shadow and a plate rule.

    The weight fades the whole card, so a caller animates a card in and out by
    lowering it. A weight of zero paints nothing.
    """
    if weight <= 0.0 or width <= 0 or height <= 0:
        return
    for step, shade in SHADOW_STEPS:
        paint_block(
            surface, left + step, top + step, width, height, INK, shade * weight
        )
    paint_block(surface, left, top, width, height, PAPER, PAPER_WEIGHT * weight)
    edge = RULE_WEIGHT * weight
    rule(surface, left, top, width, edge)
    rule(surface, left, top + height - 1, width, edge)
    upright(surface, left, top, height, edge)
    upright(surface, left + width - 1, top, height, edge)
    inner = INNER_RULE_WEIGHT * weight
    inset = INNER_RULE_INSET
    if width > 2 * inset and height > 2 * inset:
        span = width - 2 * inset
        tall = height - 2 * inset
        rule(surface, left + inset, top + inset, span, inner)
        rule(surface, left + inset, top + height - inset - 1, span, inner)
        upright(surface, left + inset, top + inset, tall, inner)
        upright(surface, left + width - inset - 1, top + inset, tall, inner)


def title(
    surface: Surface,
    left: int,
    top: int,
    width: int,
    words: str,
    weight: float = 1.0,
) -> int:
    """Write the title of a card, rule it, and give back the row under it.

    The width is the width of the card, so the rule spans it inside the
    padding whatever the title says.
    """
    paint_text(surface, left + PADDING, top, words, INK, TITLE_SCALE, weight)
    under = top + text_height(TITLE_SCALE) + 4
    rule(
        surface, left + PADDING, under, width - 2 * PADDING, TITLE_RULE_WEIGHT * weight
    )
    return under + 7


def row(
    surface: Surface,
    left: int,
    top: int,
    width: int,
    words: str,
    chosen: bool = False,
    weight: float = LIVE_WEIGHT,
    card_weight: float = 1.0,
) -> int:
    """Write one row of a card, and give back the row under it.

    A chosen row takes a pale wash across the card and a rule under the line.
    The caret belongs to the caller, because a caller that draws two columns
    puts it in the first of them.
    """
    tall = text_height(BODY_SCALE)
    if chosen:
        paint_block(
            surface,
            left + INNER_RULE_INSET + 1,
            top - 2,
            width - 2 * (INNER_RULE_INSET + 1),
            tall + 4,
            INK,
            CHOSEN_WASH * card_weight,
        )
        rule(
            surface,
            left + PADDING,
            top + tall + 1,
            width - 2 * PADDING,
            CHOSEN_RULE * card_weight,
        )
    paint_text(
        surface, left + PADDING, top, words, INK, BODY_SCALE, weight * card_weight
    )
    return top + row_height()


def pair(
    surface: Surface,
    left: int,
    top: int,
    width: int,
    label: str,
    value: str,
    chosen: bool = False,
    weight: float = LIVE_WEIGHT,
    card_weight: float = 1.0,
) -> int:
    """Write a label on the left and a value on the right of one row.

    A settings row reads as two columns, and a value that hangs off the right
    edge lines up down the card whatever the labels say.
    """
    row(surface, left, top, width, label, chosen, weight, card_weight)
    paint_text(
        surface,
        left + width - PADDING - text_width(value, BODY_SCALE),
        top,
        value,
        INK,
        BODY_SCALE,
        weight * card_weight,
    )
    return top + row_height()


def divider(surface: Surface, left: int, top: int, width: int, weight: float) -> int:
    """Draw a rule between two blocks, and give back the row under it."""
    rule(surface, left + PADDING, top, width - 2 * PADDING, DIVIDER_WEIGHT * weight)
    return top + ROW_GAP + 2
