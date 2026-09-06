"""The controller writes a board, prices a contract and sends carriers.

Every test here drives the engine and reads what it left. Python sets a
balance value, steps, and reads the census, the board and the carrier list. It
loops over no entity.[^1]

The four balance values are rows of the balance register, and none of them is
measured.[^2] The tests pin the verbs that read them and never the numbers.

References
----------
[^1]: ADR-0144, a faction controller runs inside the step and acts only through
the caller's verbs.
``docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md``

[^2]: Balance register. ``docs/reference/balance.md``

[^3]: ADR-0149, a faction's trade board is simulated state that any faction may
read.
``docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md``
"""

from __future__ import annotations

import pytest

from cachette import VerbError, World

# A world small enough to step many times, and large enough to seat a faction.
EXTENT = 48
SEED = 0x0CAC_4E77_0482
FACTIONS = 2


def seeded_world() -> World:
    """Build and seed a small world."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    reports = world.seed_world()
    assert any(report["seated"] for report in reports)
    return world


def test_the_four_balance_values_read_back_as_they_were_written() -> None:
    """Each knob is one value in one place, and the world answers it."""
    world = seeded_world()
    world.set_advertisement_schedule(7, 3)
    assert world.advertisement_schedule == (7, 3)
    world.set_surplus_mark(11)
    assert world.surplus_mark == 11
    world.set_carriers_per_contract(5)
    assert world.carriers_per_contract == 5
    world.set_contract_term(321)
    assert world.contract_term == 321


def test_a_schedule_of_period_zero_is_refused() -> None:
    """A period of zero names a board that is never written, and zero says so."""
    world = seeded_world()
    with pytest.raises(VerbError):
        world.set_advertisement_schedule(0, 0)
    assert world.advertisement_schedule[0] > 0


def test_the_controller_writes_a_board_on_its_schedule_tick() -> None:
    """The census counts a board write on the schedule tick and on no other.

    The write replaces the whole board of a faction, and it passes the verb a
    caller calls.[^3]
    """
    world = seeded_world()
    world.set_advertisement_schedule(4, 1)
    # The census row is a total for the run, so the test reads what one tick
    # added to it and not the row itself.
    before = world.subsystem_census()["boards_written"]
    for _ in range(13):
        world.step(threads=1)
        after = world.subsystem_census()["boards_written"]
        written = after - before
        before = after
        if world.tick % 4 == 1:
            assert written > 0, f"tick {world.tick} wrote no board"
        else:
            assert written == 0, f"tick {world.tick} wrote {written} boards"


def test_a_board_the_controller_wrote_reads_back_as_rows() -> None:
    """Any faction reads any board, and the rows say what the sites hold."""
    world = seeded_world()
    world.set_advertisement_schedule(1, 0)
    world.set_surplus_mark(1)
    for _ in range(3):
        world.step(threads=1)
    columns = world.market(0)
    assert len(columns["good"]) > 0, "the controller wrote no row"
    for index in range(len(columns["good"])):
        assert int(columns["wants"][index]) in (0, 1)
        assert int(columns["asking_good"][index]) != int(columns["good"][index])
        assert int(columns["quantity"][index]) > 0


def test_the_carrier_list_answers_three_columns() -> None:
    """The list is state, and it is empty until a contract binds."""
    world = seeded_world()
    columns = world.carrier_columns()
    assert set(columns) == {"unit", "row", "faction"}
    assert len(columns["unit"]) == len(columns["row"])
    assert len(columns["unit"]) == len(columns["faction"])


def test_a_carrier_count_of_zero_assigns_none() -> None:
    """A world whose balance row is zero holds no carrier, whatever it trades."""
    world = seeded_world()
    world.set_carriers_per_contract(0)
    world.set_advertisement_schedule(1, 0)
    world.set_surplus_mark(1)
    for _ in range(40):
        world.step(threads=1)
        assert len(world.carrier_columns()["unit"]) == 0
        assert world.subsystem_census()["carriers_assigned"] == 0
