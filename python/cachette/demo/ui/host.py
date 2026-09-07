"""What the interface may ask of the run that owns it.

**The interface names what it needs, and nothing more.** The demonstration
holds a world, a camera, a clock and a renderer. If every card reached into
that object, the interface and the loop would become one thing, and neither
could be read or tested without the other.

This states the surface as a protocol. The demonstration satisfies it because
it carries these names, and no import runs from the demonstration to the
interface at run time.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Protocol

if TYPE_CHECKING:
    from cachette import World
    from cachette._core import FoundingReport
    from cachette.demo.clock import Clock
    from cachette.demo.compass import Compass
    from cachette.demo.minimap import Minimap
    from cachette.demo.pilot import Pilot
    from cachette.demo.player import Seat
    from cachette.demo.settings import Settings
    from cachette.demo.surface import Surface
    from cachette.demo.ui.chrome import Chrome
    from cachette.demo.view import View
    from cachette.names import Names


class Host(Protocol):
    """The run the interface draws over and writes to."""

    world: World
    view: View
    clock: Clock
    settings: Settings
    minimap: Minimap
    compass: Compass
    surface: Surface
    names: Names
    panels: list[str]
    overlay: str | None
    pointer: tuple[int, int] | None
    foundings: list[FoundingReport]
    seat: Seat | None
    pilots: list[Pilot]
    stopping: bool
    chrome: Chrome

    def toggle_panel(self, name: str) -> None:
        """Add a panel of the engine deck to the frame, or take it off."""
        ...

    def choose_overlay(self, name: str | None) -> None:
        """Show one overlay on the map, or show the plain map."""
        ...

    def open_on(self, place: tuple[int, int]) -> None:
        """Point the camera at a place and hold it inside the world."""
        ...

    def drawing_sketch(self) -> bool:
        """Say whether the sketch is the renderer that fills the frame."""
        ...

    def choose_renderer(self, sketch: bool) -> str:
        """Put the sketch or the engine at the frame, and say what happened.

        The answer is empty when the change succeeded, and it names the reason
        when it did not. A machine that cannot draw the sketch must say so on
        the card that asked for it, rather than leaving the watcher with a
        frame that did not change.
        """
        ...

    def take_seat(self, faction: int) -> None:
        """Put one faction in the hands of the person at the keyboard."""
        ...

    def leave_seat(self) -> None:
        """Give the faction back to the engine controller."""
        ...

    def faction_name(self, faction: int) -> str:
        """Give back the name of one faction."""
        ...

    def apply_video(self) -> None:
        """Put the video settings on the open window, and remember them.

        A run that opens no window does nothing here. The interface asks for
        the change and never holds a window of its own.
        """
        ...
