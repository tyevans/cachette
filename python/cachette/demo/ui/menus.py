"""The menus a person moves through, built from the run they are watching.

**Every list here comes from the engine.** The overlays, the panels and the
faction count are read from the world when a menu is drawn, so a run that
gains an overlay gains a row with no edit to this file. A hand-written list
would be a second declaration of a thing the engine already publishes, and it
would name a choice that does nothing the moment the two parted.

This module holds the words of the interface and no drawing. The card that
shows a menu is elsewhere, and the routing of the keyboard is elsewhere again.
"""

from __future__ import annotations

from collections.abc import Callable
from typing import TYPE_CHECKING

from cachette import VerbError, World
from cachette.demo.clock import SPEEDS, says
from cachette.demo.player import TURN_CHOICES
from cachette.demo.ui.menu import Item, Menu

if TYPE_CHECKING:
    from cachette._core import FoundingReport
    from cachette.demo.ui.host import Host

# The words a row shows for a thing that is on and for a thing that is off.
ON = "ON"
OFF = "OFF"


def yes_or_no(state: bool) -> str:
    """Give back the word a row shows for a state."""
    return ON if state else OFF


def main(host: Host) -> Menu:
    """Give back the menu a person opens on."""

    def rows() -> list[Item]:
        return [
            Item("THE WORLD", opens=lambda: world_menu(host)),
            Item("WHAT IS DRAWN", opens=lambda: drawn_menu(host)),
            Item("PLAY A FACTION", opens=lambda: play_menu(host)),
            Item("SETTINGS", opens=lambda: settings_menu(host)),
            Item("LEAVE THE WORLD", act=lambda: _stop(host), over_rule=True),
        ]

    return Menu("CACHETTE", rows)


def world_menu(host: Host) -> Menu:
    """Give back the menu that holds the clock and the places to go."""

    def rows() -> list[Item]:
        items = [
            Item(
                "THE CLOCK",
                act=host.clock.toggle,
                reads=lambda: "STOPPED" if host.clock.paused else "RUNNING",
                live=lambda: host.seat is None,
            ),
            Item(
                "SPEED",
                act=_cycle_speed(host),
                reads=lambda: says(host.clock.speed).upper(),
            ),
            Item("STEP ONE TICK", act=host.clock.step_once),
            Item("TICK", reads=lambda: str(host.world.tick), over_rule=True),
            Item("SEED", reads=lambda: format(host.world.seed, "X")),
            Item(
                "GO TO A FACTION",
                opens=lambda: seats_menu(host),
                over_rule=True,
                live=lambda: bool(_seated(host)),
            ),
            Item("STORMS", opens=lambda: storm_menu(host)),
        ]
        return items

    return Menu("THE WORLD", rows)


def storm_menu(host: Host) -> Menu:
    """Give back the menu that raises a storm where the person is pointing.

    **A storm is placed and it does not form.** The field of the engine holds
    one layer of air, and one layer grows no low of its own, so a person
    places one and the engine carries it until it dies.

    The two rows are one storm at two settings. The large one covers several
    cells and lasts. The small one is deeper over one cell and is over
    quickly. **The small one is not a tornado.** One weather cell spans tens
    of kilometres at least, and a tornado is under one, so the small setting
    is an intensity on a cell rather than a funnel.
    """

    def rows() -> list[Item]:
        return [
            Item(
                "STORMS STANDING",
                reads=lambda: str(len(host.world.cyclones())),
                over_rule=True,
            ),
            Item(
                "RAISE A LARGE STORM",
                act=_raise_storm(host, "tropical"),
                live=lambda: host.pointer is not None,
            ),
            Item(
                "RAISE A SMALL VIOLENT STORM",
                act=_raise_storm(host, "severe"),
                live=lambda: host.pointer is not None,
            ),
        ]

    return Menu("STORMS", rows)


def _raise_storm(host: Host, kind: str) -> Callable[[], None]:
    """Give back an action that raises one storm under the pointer.

    The engine refuses a place outside the world, and it refuses a storm when
    it already carries as many as it holds. The action takes the refusal and
    leaves the world alone, because a menu row is not the place to report one.
    """

    def act() -> None:
        place = host.pointer
        if place is None:
            return
        try:
            host.world.raise_cyclone(place, kind)
        except VerbError:
            return

    return act


def seats_menu(host: Host) -> Menu:
    """Give back a menu of the place each faction was founded on."""

    def rows() -> list[Item]:
        return [
            Item(
                host.faction_name(int(report["faction"])).upper(),
                act=_go_to(host, report),
                reads=lambda report=report: f"{report['q']}, {report['r']}",  # type: ignore[misc]
            )
            for report in _seated(host)
        ]

    return Menu("GO TO A FACTION", rows)


def drawn_menu(host: Host) -> Menu:
    """Give back the menu of what the frame draws over the world."""

    def rows() -> list[Item]:
        return [
            Item(
                "RENDERER",
                act=_swap_renderer(host),
                reads=lambda: "SKETCH" if host.drawing_sketch() else "ENGINE",
            ),
            Item(
                "MINIMAP",
                act=_toggle_minimap(host),
                reads=lambda: yes_or_no(host.minimap.visible),
            ),
            Item(
                "COMPASS",
                act=_toggle_compass(host),
                reads=lambda: yes_or_no(host.compass.visible),
            ),
            Item(
                "OVERLAY",
                opens=lambda: overlay_menu(host),
                over_rule=True,
            ),
            Item("PANELS", opens=lambda: panel_menu(host)),
        ]

    return Menu("WHAT IS DRAWN", rows)


def overlay_menu(host: Host) -> Menu:
    """Give back one row for each overlay the engine publishes.

    The engine names the overlays. A row here can therefore never name one
    that no overlay carries.
    """

    def rows() -> list[Item]:
        items = [
            Item(
                "THE PLAIN MAP",
                act=lambda: host.choose_overlay(None),
                reads=lambda: yes_or_no(host.overlay is None),
            )
        ]
        items.extend(
            Item(
                name.upper(),
                act=lambda name=name: host.choose_overlay(name),  # type: ignore[misc]
                reads=lambda name=name: yes_or_no(host.overlay == name),  # type: ignore[misc]
            )
            for name in World.overlay_names()
        )
        return items

    return Menu("OVERLAY", rows)


def panel_menu(host: Host) -> Menu:
    """Give back one row for each panel of the engine deck."""

    def rows() -> list[Item]:
        return [
            Item(
                name.upper(),
                act=lambda name=name: host.toggle_panel(name),  # type: ignore[misc]
                reads=lambda name=name: yes_or_no(name in host.panels),  # type: ignore[misc]
            )
            for name in World.panel_names()
        ]

    return Menu("PANELS", rows)


def play_menu(host: Host) -> Menu:
    """Give back the menu that hands a faction to the person, and takes it back."""

    def rows() -> list[Item]:
        seat = host.seat
        if seat is None:
            return [
                Item("TAKE A FACTION", opens=lambda: take_menu(host)),
                Item(
                    "THE ENGINE PLAYS EVERY FACTION",
                    over_rule=True,
                ),
            ]
        return [
            Item("YOU HOLD", reads=lambda: host.faction_name(seat.faction).upper()),
            Item("TURN", reads=lambda: str(seat.turn)),
            Item(
                "A TURN RUNS",
                act=_cycle_turn(host),
                reads=lambda: f"{seat.turn_ticks} TICKS",
            ),
            Item(
                "END THE TURN",
                act=lambda: seat.end_turn(),
                live=lambda: seat.frozen,
                over_rule=True,
            ),
            Item("LEAVE THE SEAT", act=host.leave_seat),
        ]

    return Menu("PLAY", rows)


def take_menu(host: Host) -> Menu:
    """Give back one row for each faction a person may take."""

    def rows() -> list[Item]:
        return [
            Item(host.faction_name(faction).upper(), act=_take(host, faction))
            for faction in range(host.world.faction_count)
        ]

    return Menu("TAKE A FACTION", rows)


def _take(host: Host, faction: int) -> Callable[[], None]:
    """Give back an action that hands one faction to the person.

    **The menu closes behind it.** The turn card comes up the moment a person
    takes a seat, and a menu still open over it would hide the thing the
    person just asked for.
    """

    def act() -> None:
        host.take_seat(faction)
        host.chrome.close()

    return act


def settings_menu(host: Host) -> Menu:
    """Give back the settings menu, which the run remembers between runs."""

    def rows() -> list[Item]:
        video = host.settings.video
        return [
            Item(
                "RENDERER",
                act=_swap_renderer(host),
                reads=lambda: "SKETCH" if host.drawing_sketch() else "ENGINE",
            ),
            Item(
                "WINDOW SIZE",
                act=_next_size(host),
                reads=lambda: "{} X {}".format(*video.size),
                over_rule=True,
            ),
            Item(
                "FULLSCREEN",
                act=_toggle_fullscreen(host),
                reads=lambda: yes_or_no(video.fullscreen),
            ),
            Item(
                "VERTICAL SYNC",
                act=_toggle_vsync(host),
                reads=lambda: yes_or_no(video.vsync),
            ),
        ]

    return Menu("SETTINGS", rows)


def _toggle_minimap(host: Host) -> Callable[[], None]:
    """Give back an action that shows the minimap, or hides it."""

    def act() -> None:
        host.minimap.toggle()

    return act


def _toggle_compass(host: Host) -> Callable[[], None]:
    """Give back an action that shows the compass, or hides it."""

    def act() -> None:
        host.compass.toggle()

    return act


def _seated(host: Host) -> list[FoundingReport]:
    """Give back the founding report of each faction that got a place."""
    return [report for report in host.foundings if report.get("seated")]


def _go_to(host: Host, report: FoundingReport) -> Callable[[], None]:
    """Give back an action that points the camera at one founding place."""

    def act() -> None:
        host.open_on((int(report["q"]), int(report["r"])))

    return act


def _cycle_speed(host: Host) -> Callable[[], None]:
    """Give back an action that takes the next speed, and wraps at the top."""

    def act() -> None:
        host.clock.choose((host.clock.speed_index + 1) % len(SPEEDS))

    return act


def _cycle_turn(host: Host) -> Callable[[], None]:
    """Give back an action that takes the next turn length."""

    def act() -> None:
        seat = host.seat
        if seat is None:
            return
        order = list(TURN_CHOICES)
        at = order.index(seat.turn_ticks) if seat.turn_ticks in order else -1
        seat.choose_turn_ticks(order[(at + 1) % len(order)])

    return act


def _swap_renderer(host: Host) -> Callable[[], None]:
    """Give back an action that puts the other renderer at the frame."""

    def act() -> None:
        host.choose_renderer(not host.drawing_sketch())

    return act


def _next_size(host: Host) -> Callable[[], None]:
    """Give back an action that takes the next window size."""

    def act() -> None:
        host.settings.video.next_size()
        host.apply_video()

    return act


def _toggle_fullscreen(host: Host) -> Callable[[], None]:
    """Give back an action that turns fullscreen on or off."""

    def act() -> None:
        host.settings.video.fullscreen = not host.settings.video.fullscreen
        host.apply_video()

    return act


def _toggle_vsync(host: Host) -> Callable[[], None]:
    """Give back an action that turns vertical synchronisation on or off."""

    def act() -> None:
        host.settings.video.vsync = not host.settings.video.vsync
        host.apply_video()

    return act


def _stop(host: Host) -> None:
    """Ask the run to close the window."""
    host.stopping = True
