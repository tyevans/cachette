"""The interface as one object: what it draws, and who holds the keyboard.

**One thing owns the order of the layers, and one thing owns the focus.** A
frame draws the title block, then the turn card, then the menu, and a key
reaches the topmost card that wants it. If each card held its own keyboard
handler, two of them would answer one key press and a person would see both
answers.

**Nothing here names a window library.** The methods take a direction, a
place, and a plain word. The window adapter turns a key symbol into one of
them, so this module runs and is tested with no display.

The cost of a frame here follows the number of rows on the open cards. It
never follows the size of the world, and it reads no tile.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from cachette.demo.minimap import DIAMETER, MARGIN
from cachette.demo.ui import keycard, menu, menus, status, theme, turncard

if TYPE_CHECKING:
    from cachette.demo.player import Choice, Seat
    from cachette.demo.surface import Surface
    from cachette.demo.ui.host import Host


class Chrome:
    """The whole interface the control plane draws over the frame."""

    __slots__ = ("_host", "chosen_action", "keys_showing", "menus")

    def __init__(self, host: Host) -> None:
        """Build the interface for one run. Nothing is open."""
        self._host = host
        self.menus = menu.Stack()
        # Whether the card that names every key is on the frame.
        self.keys_showing = False
        # The row of the turn card the keyboard stands on. It is a place in
        # the list of legal rows, and the list changes every tick, so it is
        # held to the length of the list when the card draws.
        self.chosen_action = 0

    # -- what holds the keyboard -------------------------------------------

    @property
    def modal(self) -> bool:
        """Say whether a card holds the keyboard and the mouse."""
        return self.menus.showing or self.keys_showing

    @property
    def choosing(self) -> bool:
        """Say whether the turn card is up and waiting for a choice."""
        seat = self._host.seat
        return seat is not None and seat.frozen

    def clears_foot(self) -> int:
        """Give back how many rows at the foot of the frame the interface takes.

        The lines the deck writes over the map sit at the foot, and the title
        block sits there as well. The block states its own height, so the two
        never overlap and neither holds a copy of the other.
        """
        return status.clears(self._host)

    def grabs_arrows(self) -> bool:
        """Say whether a card wants the arrow keys.

        **The arrows scroll the map, and a card needs them too.** A frame that
        did both would move the map under a person choosing a row. The letter
        keys still scroll while a card is up, so the map is never stuck.
        """
        return self.modal or self.choosing

    # -- what a person asks for --------------------------------------------

    def toggle_menu(self) -> None:
        """Open the menu, close it, or step back out of a submenu.

        A person who opened three menus deep expects the key that opened them
        to take them back one, and to close the interface from the top.
        """
        if self.keys_showing:
            self.keys_showing = False
            return
        if not self.menus.showing:
            self.menus.open(menus.main(self._host))
            return
        self.menus.back()

    def close(self) -> None:
        """Close every card that holds the keyboard."""
        self.menus.close()
        self.keys_showing = False

    def toggle_keys(self) -> None:
        """Show the card that names every key, or hide it."""
        self.keys_showing = not self.keys_showing
        if self.keys_showing:
            self.menus.close()

    def move(self, by: int) -> bool:
        """Move the caret of the topmost card, and say whether one took it."""
        if self.menus.showing:
            self.menus.step(by)
            return True
        if self.keys_showing:
            return True
        if self.choosing:
            self.chosen_action += by
            return True
        return False

    def enter(self) -> bool:
        """Take the row the caret stands on, and say whether one took it."""
        if self.menus.showing:
            self.menus.enter()
            return True
        if self.keys_showing:
            self.keys_showing = False
            return True
        if self.choosing:
            self._take(self.chosen_action)
            return True
        return False

    def end_turn(self) -> bool:
        """End the turn of the person, and say whether there was one."""
        seat = self._host.seat
        if seat is None or not seat.frozen:
            return False
        seat.end_turn()
        self.chosen_action = 0
        return True

    # -- the mouse ---------------------------------------------------------

    def hover(self, x: int, y: int) -> bool:
        """Put the caret under the pointer, and say whether a card took it.

        The place is in frame coordinates, which count the rows down from the
        top of the frame.
        """
        if self.menus.showing:
            over_menu = self._menu_plan()
            if over_menu is None:
                return False
            at = menu.row_at(over_menu, x, y)
            if at is not None:
                self.menus.stand_on(at)
            return menu.inside(over_menu, x, y)
        if self.keys_showing:
            return True
        if self.choosing:
            over_turn = self._turn_plan()
            if over_turn is None:
                return False
            at = turncard.row_at(over_turn, x, y)
            if at is not None:
                self.chosen_action = at
            return turncard.inside(over_turn, x, y)
        return False

    def click(self, x: int, y: int) -> bool:
        """Take the row under the pointer, and say whether a card took it.

        A click outside an open menu closes it. That is what a person expects
        of a card laid on a drawing, and it saves them reaching for a key.
        """
        if self.menus.showing:
            over_menu = self._menu_plan()
            if over_menu is None or not menu.inside(over_menu, x, y):
                self.menus.close()
                return True
            at = menu.row_at(over_menu, x, y)
            if at is not None:
                self.menus.stand_on(at)
                self.menus.enter()
            return True
        if self.keys_showing:
            self.keys_showing = False
            return True
        if self.choosing:
            over_turn = self._turn_plan()
            if over_turn is None or not turncard.inside(over_turn, x, y):
                return False
            at = turncard.row_at(over_turn, x, y)
            if at is not None:
                self.chosen_action = at
                self._take(at)
            return True
        return False

    # -- the frame ---------------------------------------------------------

    def paint(self, surface: Surface) -> None:
        """Draw the interface over the frame the renderer filled.

        The title block is always there. The turn card is there while a person
        is choosing. The menu and the key card are on top of both, and they
        never appear together.
        """
        status.paint(surface, self._host)
        seat = self._host.seat
        if seat is not None and seat.frozen:
            self._hold_chosen(seat)
            turncard.paint(surface, seat, self.chosen_action, clear_top=self._clear())
        if self.menus.showing:
            open_menu = self.menus.menu
            if open_menu is not None:
                menu.paint(surface, open_menu, open_menu.rows(), self.menus.chosen)
        elif self.keys_showing:
            keycard.paint(surface)

    # -- the parts nobody outside calls ------------------------------------

    def _menu_plan(self) -> menu.Layout | None:
        """Give back where the open menu card sits, or nothing."""
        open_menu = self.menus.menu
        if open_menu is None:
            return None
        surface = self._host.surface
        return menu.lay_out(
            open_menu.rows(), open_menu.title, surface.width, surface.height
        )

    def _turn_plan(self) -> turncard.Layout | None:
        """Give back where the turn card sits, or nothing."""
        seat = self._host.seat
        if seat is None or not seat.frozen:
            return None
        self._hold_chosen(seat)
        return turncard.lay_out(seat, seat.actions(), self._host.surface, self._clear())

    def _clear(self) -> int:
        """Give back the first row of the frame a right hand card may use.

        **The minimap owns the top right corner.** A card that started at the
        top of the frame would run under the disc. The disc declares its own
        size, so this reads it rather than holding a copy of it.
        """
        if not self._host.minimap.visible:
            return theme.MARGIN
        return MARGIN + DIAMETER + theme.MARGIN

    def _hold_chosen(self, seat: Seat) -> None:
        """Hold the caret of the turn card inside the list, and wrap it.

        The list of legal rows changes as the world changes, so a caret that
        stood on the last row of a longer list must come back inside.
        """
        shown = len(_legal(seat)[: turncard.MOST_ROWS])
        self.chosen_action = self.chosen_action % shown if shown else 0

    def _take(self, at: int) -> None:
        """Send the action of one row of the turn card to the engine."""
        seat = self._host.seat
        if seat is None:
            return
        legal = _legal(seat)
        if 0 <= at < len(legal):
            seat.take(legal[at].action)


def _legal(seat: Seat) -> list[Choice]:
    """Give back the rows of the action table that are legal now."""
    return [choice for choice in seat.actions() if choice.legal]
