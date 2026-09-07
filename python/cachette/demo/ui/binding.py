"""The window adapter: it turns a key symbol into something the interface says.

**The interface names no window library.** It takes a direction, a place and a
plain word, so it runs with no display and a test drives it directly. This
module is the one place that knows what a key symbol is, and it holds nothing
else.

The caller passes the key module of the window library, because that library
ships no type information and importing it at the top of a module would make
every function it touches untyped.

The window numbers the rows of the frame from the bottom, and the frame
numbers them from the top. This module turns one into the other, so no card
ever holds a second copy of that rule.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, TypeAlias

if TYPE_CHECKING:
    from cachette.demo.surface import Surface
    from cachette.demo.ui.chrome import Chrome

# The key names of the window library.
#
# **The library ships no type information**, so nothing can describe the module
# that holds the key symbols. The type is open here, in the one module that
# touches the library, and every other module of the interface is closed.
KeyNames: TypeAlias = Any

# The button the window library reports for the left mouse button.
LEFT_BUTTON = 1


class Keys:
    """The keyboard and the mouse of the window, as the interface hears them.

    Every method gives back whether the interface took the event. The window
    library stops passing an event on when a handler says it took it, so a
    person inside a menu never moves the map by mistake.
    """

    # **The window library holds a handler weakly.** A class of slots carries
    # no weak reference unless it names one, and the library then refuses it.
    # The run keeps a strong reference of its own, because a handler the
    # library holds weakly and nobody else holds is collected at once.
    __slots__ = ("__weakref__", "_chrome", "_key", "_surface")

    def __init__(self, chrome: Chrome, key: KeyNames, surface: Surface) -> None:
        """Bind one interface to the key names of the window library."""
        self._chrome = chrome
        self._key = key
        self._surface = surface

    def _down(self, y: int) -> int:
        """Turn a row the window counts from the bottom into one from the top."""
        return self._surface.height - y

    def on_key_press(self, symbol: int, modifiers: int) -> bool:
        """Give one key press to the interface, and say whether it took it."""
        del modifiers
        key = self._key
        chrome = self._chrome
        if symbol == key.ESCAPE:
            chrome.toggle_menu()
            return True
        if symbol == key.SLASH or symbol == key.QUESTION:
            chrome.toggle_keys()
            return True
        if symbol in (key.RETURN, key.ENTER, key.NUM_ENTER):
            return chrome.enter()
        if symbol == key.UP:
            return chrome.move(-1)
        if symbol == key.DOWN:
            return chrome.move(1)
        if symbol == key.LEFT and chrome.menus.showing:
            chrome.menus.back()
            return True
        if symbol == key.RIGHT and chrome.menus.showing:
            chrome.menus.enter()
            return True
        # **A card that holds the keyboard holds this key too.** A person
        # reading a menu who pressed this would end their turn without seeing
        # the card that says what the turn did.
        if symbol == key.E and not chrome.modal:
            return chrome.end_turn()
        # **A card that holds the keyboard holds every key.** A person inside
        # a menu who pressed a number would otherwise change the overlay under
        # the card they were reading.
        return chrome.modal

    def on_mouse_motion(self, x: int, y: int, dx: int, dy: int) -> bool:
        """Follow the pointer over the open cards."""
        del dx, dy
        return self._chrome.hover(int(x), self._down(int(y)))

    def on_mouse_press(self, x: int, y: int, button: int, modifiers: int) -> bool:
        """Give one press to the open cards."""
        del modifiers
        if button != LEFT_BUTTON:
            return self._chrome.modal
        return self._chrome.click(int(x), self._down(int(y)))

    def on_mouse_release(self, x: int, y: int, button: int, modifiers: int) -> bool:
        """Keep a release from reaching the map while a card is open."""
        del x, y, button, modifiers
        return self._chrome.modal

    def on_mouse_drag(
        self, x: int, y: int, dx: int, dy: int, buttons: int, modifiers: int
    ) -> bool:
        """Keep a drag from moving the map while a card is open."""
        del x, y, dx, dy, buttons, modifiers
        return self._chrome.modal

    def on_mouse_scroll(self, x: int, y: int, scroll_x: float, scroll_y: float) -> bool:
        """Keep the wheel from zooming the map while a card is open."""
        del x, y, scroll_x, scroll_y
        return self._chrome.modal
