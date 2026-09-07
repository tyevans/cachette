"""The player seat, the turn that holds time, and the interface over the frame.

**These tests drive the frame, not the parts.** The demonstration is obliged
to hold the clock while a person chooses, so a test that called the seat
directly would prove that the seat works and prove nothing about whether the
loop reaches it.[^1]

The watching mode and the playing mode share one loop. Each test below states
which of the two it drives, and the two are compared where the point of the
test is that one did not disturb the other.

References
----------
[^1]: Testing Rules, section 5. ``.agents/rules/testing.md``

[^2]: ADR-0176, the action table is flat and bounded.
``docs/adrs/draft/adr-0176-the-action-table-is-flat-and-bounded.md``
"""

from __future__ import annotations

import numpy as np

from cachette import World
from cachette.demo.app import Demo
from cachette.demo.clock import SPEEDS
from cachette.demo.player import read_table
from cachette.demo.ui import menu, menus
from cachette.names import Names

# A world small enough to step many times in a test, and large enough for the
# founding survey to seat every faction.
SIDE = 64
SEED = 0x0CAC_4E77_0472
FACTIONS = 3
WIDTH = 420
HEIGHT = 520

# How many frames a test draws while it waits for the world to move.
#
# The clock reads the wall clock, and a test must not assert on it. Every test
# below therefore drives a fixed number of frames and asserts on the tick,
# never on how long the frames took.
FRAMES = 12


def build(ticks: int = 20) -> Demo:
    """Give back a seeded, stepped demonstration on the engine renderer."""
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=FACTIONS)
    demo = Demo(world, Names(SEED), width=WIDTH, height=HEIGHT, threads=1)
    demo.foundings = demo.seed()
    for _ in range(ticks):
        world.step(1)
    demo.open_on((SIDE // 2, SIDE // 2))
    # The fastest speed runs several ticks for each frame, so a test reaches
    # the end of a turn in a few frames.
    demo.clock.choose(len(SPEEDS) - 1)
    return demo


def test_watching_runs_the_world_when_nobody_holds_a_seat() -> None:
    """The mode a run opens in is untouched: the frames step the world.

    **This is the control for the freezing test below.** A test that only
    showed a frozen world would pass if the frames never stepped at all.
    """
    demo = build()
    was = demo.world.tick
    for _ in range(FRAMES):
        demo.advance()
    assert demo.world.tick > was, "the watching mode stopped stepping the world"
    assert demo.seat is None
    for faction in range(FACTIONS):
        assert not demo.world.is_externally_controlled(faction)


def test_the_turn_freezes_time_while_the_person_chooses() -> None:
    """A person inside a turn stops the clock, and ending the turn starts it.

    The frames still draw. The world does not move.
    """
    demo = build()
    demo.take_seat(1)
    seat = demo.seat
    assert seat is not None
    assert seat.frozen, "a seat opens with the person choosing"

    held = demo.world.tick
    hashed = demo.world.state_hash()
    for _ in range(FRAMES):
        demo.advance()
    assert demo.world.tick == held, "the world moved while the person was choosing"
    assert demo.world.state_hash() == hashed, "the state moved while time was frozen"

    # Ending the turn lets exactly one turn of ticks run, and then it freezes
    # again. The clock is at its fastest, so several ticks land on one frame
    # and the turn must still stop on the right one.
    seat.end_turn()
    assert not seat.frozen
    for _ in range(FRAMES):
        demo.advance()
    assert demo.world.tick == held + seat.turn_ticks, (
        f"the turn ran to tick {demo.world.tick}, and a turn of "
        f"{seat.turn_ticks} ticks from {held} ends at {held + seat.turn_ticks}"
    )
    assert seat.frozen, "the turn did not stop at its end"
    assert seat.turn == 2


def test_the_length_of_a_turn_is_what_the_person_chose() -> None:
    """The number of ticks in a turn is a setting, not a constant of the loop.

    A turn of another length must run that many ticks. A test at one length
    cannot tell a turn that counts from a turn that stops on a fixed number.
    """
    demo = build()
    demo.turn_ticks = 7
    demo.take_seat(0)
    seat = demo.seat
    assert seat is not None
    held = demo.world.tick
    seat.end_turn()
    for _ in range(FRAMES):
        demo.advance()
    assert demo.world.tick == held + 7
    assert seat.frozen


def test_the_person_holds_the_faction_and_gives_it_back() -> None:
    """The engine controller leaves the seat alone, and takes it back after."""
    demo = build()
    demo.take_seat(2)
    assert demo.world.is_externally_controlled(2)
    assert not demo.world.is_externally_controlled(0)
    demo.leave_seat()
    assert demo.seat is None
    assert not demo.world.is_externally_controlled(2)
    # The world runs again the moment the seat is empty.
    was = demo.world.tick
    for _ in range(FRAMES):
        demo.advance()
    assert demo.world.tick > was


def test_a_person_and_a_policy_read_one_mask() -> None:
    """The rows a person may take are the rows the engine says are legal.

    **The card widens nothing.** The mask the engine gives is fog scoped, so a
    row that names ground the faction has never seen is refused. A card that
    marked a row legal on its own would show a person something a policy in
    the same seat could not choose.
    """
    demo = build(ticks=40)
    demo.take_seat(1)
    seat = demo.seat
    assert seat is not None
    mask = np.asarray(demo.world.legal_actions(1))
    rows = seat.actions()
    assert len(rows) == len(mask), "the card lists a different table"
    for choice in rows:
        assert choice.legal == bool(mask[choice.action]), (
            f"row {choice.action} reads {choice.legal} and the engine says "
            f"{bool(mask[choice.action])}"
        )
    # Row zero is the verb that does nothing, and the engine always allows it,
    # so the card is never empty.
    assert rows[0].legal


def test_the_table_comes_from_the_schema_and_not_from_a_list() -> None:
    """Every row of the card is a row the engine published.

    The engine states the length of the table. A card built from a list of its
    own would name a verb that no longer exists, and nothing would fail.
    """
    world = World(width=SIDE, height=SIDE, seed=SEED, faction_count=FACTIONS)
    world.seed_world()
    schema = world.action_schema()
    rows = read_table(world)
    assert len(rows) == int(schema["length"])
    assert [choice.action for choice in rows] == list(range(len(rows)))
    names = {str(verb["name"]) for verb in schema["verbs"]}
    assert {choice.verb for choice in rows} == names


def test_an_action_reaches_the_engine_through_the_verb_a_policy_uses() -> None:
    """Taking a row sends it to the engine and records what came back."""
    demo = build(ticks=40)
    demo.take_seat(1)
    seat = demo.seat
    assert seat is not None
    legal = [choice for choice in seat.actions() if choice.legal]
    assert legal, "the seat has no legal row to take"
    took = seat.take(legal[-1].action)
    assert isinstance(took, bool)
    assert len(seat.taken) == 1
    assert legal[-1].says() in seat.taken[0]


def test_the_interface_changes_nothing_the_simulation_computes() -> None:
    """A frame with the interface open computes the same tick and the same state.

    The interface reads three numbers and paints over the frame the renderer
    filled. **It must not reach the world.** Two runs of one world are driven
    here: one with every card open, and one with none, and the tick and the
    state hash must agree at every step.
    """
    quiet = build()
    busy = build()
    busy.chrome.toggle_menu()
    busy.chrome.menus.enter()
    for _ in range(FRAMES):
        quiet.advance()
        busy.advance()
        assert busy.world.tick == quiet.world.tick
        assert busy.world.state_hash() == quiet.world.state_hash()
    assert busy.chrome.menus.showing, "the test never opened a card"


def test_the_menu_answers_a_click_on_the_row_it_drew() -> None:
    """The card and the mouse read one layout.

    A card that painted rows at one set of places and answered a click at
    another would take the row above the one a person pressed.
    """
    demo = build()
    demo.chrome.toggle_menu()
    open_menu = demo.chrome.menus.menu
    assert open_menu is not None
    rows = open_menu.rows()
    plan = menu.lay_out(rows, open_menu.title, WIDTH, HEIGHT)
    for place in plan.rows:
        middle = place.top + place.height // 3
        assert menu.row_at(plan, plan.left + plan.width // 2, middle) == place.at
    assert menu.row_at(plan, plan.left - 5, plan.top + 5) is None


def test_the_menu_lists_the_overlays_the_engine_published() -> None:
    """A row per overlay, and no row for one the engine does not carry."""
    demo = build()
    rows = menus.overlay_menu(demo).rows()
    named = [item.label.lower() for item in rows[1:]]
    assert named == [name.lower() for name in World.overlay_names()]
    # Choosing one reaches the frame state the renderer reads.
    rows[1].act()  # type: ignore[misc]
    assert demo.overlay == World.overlay_names()[0]
    rows[0].act()  # type: ignore[misc]
    assert demo.overlay is None


def test_the_turn_card_holds_the_arrow_keys_and_the_menu_holds_them_too() -> None:
    """The map must not scroll under a person moving a caret."""
    demo = build()
    assert not demo.chrome.grabs_arrows()
    demo.chrome.toggle_menu()
    assert demo.chrome.grabs_arrows()
    demo.chrome.close()
    assert not demo.chrome.grabs_arrows()
    demo.take_seat(0)
    assert demo.chrome.grabs_arrows(), "the turn card must hold the arrows"


def test_the_seat_survives_a_faction_that_can_do_nothing_but_wait() -> None:
    """A seat with one legal row still draws and still ends its turn.

    A card that assumed a list of many rows would fail on the first turn of a
    faction that has nothing to do.
    """
    demo = build(ticks=0)
    demo.take_seat(0)
    seat = demo.seat
    assert seat is not None
    demo.advance()
    seat.end_turn()
    for _ in range(FRAMES):
        demo.advance()
    assert seat.frozen
