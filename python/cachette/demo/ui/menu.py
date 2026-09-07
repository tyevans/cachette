"""A menu of choices, and the card that shows one.

**A menu is a list of live readings, not a list of words.** A row that held
the word "ON" would state a second copy of the thing it names, and the two
would part the moment anything else changed the setting. Every row therefore
reads its value through a function when the frame draws it.

**One function lays a menu out, and both the drawing and the mouse use it.**
A card that painted rows at one set of places and answered a click at another
would put the caret on the row above the one a person pressed.

Nothing here reads the world. A caller supplies the rows.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING, NamedTuple

from cachette.demo.ui import theme

if TYPE_CHECKING:
    from cachette.demo.surface import Surface

# How wide a menu card is at the least, in pixels.
#
# A card that shrank to its widest row would change width as a value changed
# under it, and the eye reads that as the card jumping.
LEAST_WIDTH = 340

# How far the value column sits from the label column at the least.
COLUMN_GAP = 40

# The mark on the row the keyboard stands on, and the mark on a row that opens
# another menu.
CARET = "> "
NO_CARET = "  "
DEEPER = ">"


class Item:
    """One row of a menu.

    A row that does something carries an action. A row that opens another menu
    carries a menu maker. A row that only reports carries neither.
    """

    __slots__ = ("act", "label", "live", "opens", "over_rule", "reads")

    def __init__(
        self,
        label: str,
        act: Callable[[], None] | None = None,
        reads: Callable[[], str] | None = None,
        opens: Callable[[], Menu] | None = None,
        live: Callable[[], bool] | None = None,
        over_rule: bool = False,
    ) -> None:
        """Build one row."""
        self.label = label
        self.act = act
        self.reads = reads
        self.opens = opens
        self.live = live
        self.over_rule = over_rule

    def value(self) -> str:
        """Give back the value the row reports, as words."""
        if self.opens is not None:
            return DEEPER
        if self.reads is None:
            return ""
        return self.reads()

    def usable(self) -> bool:
        """Say whether a person may choose this row now."""
        if self.live is not None and not self.live():
            return False
        return self.act is not None or self.opens is not None


class Menu:
    """A titled list of rows.

    The rows come from a function, because a menu whose rows depend on the
    state of the run must not hold a list that was true when it was built. The
    play menu holds different rows before and after a person takes a faction.
    """

    __slots__ = ("build", "title")

    def __init__(self, title: str, build: Callable[[], list[Item]]) -> None:
        """Build a menu that reads its rows when it is drawn."""
        self.title = title
        self.build = build

    def rows(self) -> list[Item]:
        """Give back the rows of the menu as they stand now."""
        return self.build()


class Placed(NamedTuple):
    """Where one row of a card sits in the frame.

    The row is named ``at`` and not ``index``, because a named tuple already
    carries a method of that name.
    """

    at: int
    top: int
    height: int


class Layout(NamedTuple):
    """Where a whole card sits in the frame, and where each row sits."""

    left: int
    top: int
    width: int
    height: int
    rows: list[Placed]


def lay_out(
    items: list[Item],
    title_words: str,
    width_of_frame: int,
    height_of_frame: int,
) -> Layout:
    """Give back where a centred menu card and each of its rows sit.

    **This is the one declaration of the layout.** The drawing calls it and
    the mouse calls it, so a click always names the row it landed on.
    """
    labels = [NO_CARET + item.label for item in items]
    values = [item.value() for item in items]
    widest_label = max((theme.line_width(line) for line in labels), default=0)
    widest_value = max((theme.line_width(line) for line in values), default=0)
    body = widest_label + COLUMN_GAP + widest_value + 2 * theme.PADDING
    width = max(LEAST_WIDTH, body, theme.card_width([title_words], theme.TITLE_SCALE))
    rules = sum(1 for item in items if item.over_rule)
    head = theme.PADDING + theme.row_height(theme.TITLE_SCALE) + 8
    height = (
        head
        + len(items) * theme.row_height()
        + rules * (theme.ROW_GAP + 2)
        + theme.PADDING
    )
    left = (width_of_frame - width) // 2
    top = (height_of_frame - height) // 2
    places: list[Placed] = []
    row = top + head
    for index, item in enumerate(items):
        if item.over_rule:
            row += theme.ROW_GAP + 2
        places.append(Placed(index, row, theme.row_height()))
        row += theme.row_height()
    return Layout(left, top, width, height, places)


def paint(
    surface: Surface,
    menu: Menu,
    items: list[Item],
    chosen: int,
    weight: float = 1.0,
) -> Layout:
    """Draw the card of one menu, and give back where it landed."""
    plan = lay_out(items, menu.title, surface.width, surface.height)
    theme.card(surface, plan.left, plan.top, plan.width, plan.height, weight)
    theme.title(
        surface, plan.left, plan.top + theme.PADDING, plan.width, menu.title, weight
    )
    for place in plan.rows:
        item = items[place.at]
        if item.over_rule:
            theme.rule(
                surface,
                plan.left + theme.PADDING,
                place.top - theme.ROW_GAP,
                plan.width - 2 * theme.PADDING,
                theme.DIVIDER_WEIGHT * weight,
            )
        mark = CARET if place.at == chosen else NO_CARET
        ink = theme.LIVE_WEIGHT if item.usable() else theme.QUIET_WEIGHT
        theme.pair(
            surface,
            plan.left,
            place.top,
            plan.width,
            mark + item.label,
            item.value(),
            chosen=place.at == chosen,
            weight=ink,
            card_weight=weight,
        )
    return plan


def row_at(plan: Layout, x: int, y: int) -> int | None:
    """Give back the row a place in the frame lands on, or nothing.

    The place is in frame coordinates, which count rows down from the top.
    """
    if not (plan.left <= x < plan.left + plan.width):
        return None
    for place in plan.rows:
        if place.top - theme.ROW_GAP // 2 <= y < place.top + place.height:
            return place.at
    return None


def inside(plan: Layout, x: int, y: int) -> bool:
    """Say whether a place in the frame lands on the card at all."""
    return (
        plan.left <= x < plan.left + plan.width
        and plan.top <= y < plan.top + plan.height
    )


class Stack:
    """The menus a person has opened, and the row each one stands on.

    A person opens a menu from a row of another menu, and goes back to the row
    they came from. The stack is what makes "back" mean the row rather than
    the top of the list.
    """

    __slots__ = ("_open",)

    def __init__(self) -> None:
        """Build an empty stack. Nothing is open."""
        self._open: list[tuple[Menu, int]] = []

    def __len__(self) -> int:
        """Give back how deep the stack runs."""
        return len(self._open)

    @property
    def showing(self) -> bool:
        """Say whether any menu is open."""
        return bool(self._open)

    @property
    def menu(self) -> Menu | None:
        """Give back the menu on top of the stack."""
        return self._open[-1][0] if self._open else None

    @property
    def chosen(self) -> int:
        """Give back the row the keyboard stands on."""
        return self._open[-1][1] if self._open else 0

    def open(self, menu: Menu) -> None:
        """Put a menu on top of the stack, standing on its first live row."""
        self._open.append((menu, 0))
        self.step(0)

    def close(self) -> None:
        """Close every menu."""
        self._open.clear()

    def back(self) -> None:
        """Close the menu on top, and show the one under it."""
        if self._open:
            self._open.pop()

    def stand_on(self, index: int) -> None:
        """Put the keyboard on one row by its number."""
        if not self._open:
            return
        menu, _ = self._open[-1]
        rows = menu.rows()
        if 0 <= index < len(rows):
            self._open[-1] = (menu, index)

    def step(self, by: int) -> None:
        """Move the keyboard by a number of rows, and wrap at the ends.

        A row nobody can choose is stepped over, so the caret never rests on a
        line that does nothing. A menu of no live rows leaves the caret where
        it was rather than looping.
        """
        if not self._open:
            return
        menu, at = self._open[-1]
        rows = menu.rows()
        if not rows:
            return
        count = len(rows)
        at = max(0, min(at, count - 1))
        if by == 0 and rows[at].usable():
            self._open[-1] = (menu, at)
            return
        step = by if by else 1
        for _ in range(count):
            at = (at + step) % count
            if rows[at].usable():
                break
        self._open[-1] = (menu, at)

    def enter(self) -> None:
        """Take the row the keyboard stands on."""
        if not self._open:
            return
        menu, at = self._open[-1]
        rows = menu.rows()
        if not (0 <= at < len(rows)):
            return
        item = rows[at]
        if not item.usable():
            return
        if item.opens is not None:
            self.open(item.opens())
            return
        if item.act is not None:
            item.act()
