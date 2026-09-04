"""Contractual trade between two factions, driven from the control plane.

Every test here goes through the front door. It builds a world, sends the
verbs a player would send, steps, and reads what the engine answers. Nothing
here reaches into the engine and nothing here constructs a mechanism to drive
it directly, because a capability that only its own test invokes ships
inert.[^1]

**The fixture is built for the case and not copied from the demonstration.**
Two things the tests need are hard to arrange in a world chosen to look right.
A speech act needs a unit of one faction standing on ground the other holds,
and the holding rule takes that tile for the speaker on the next step, so every
speech act happens between two steps. A delivery needs a laden unit standing on
another faction's settlement, so the fixture puts a unit on a tile that carries
stock, lets it fill its carry, and then founds the other faction's settlement
under it.[^2]

References
----------
[^1]: Testing Rules, section 5. ``.claude/rules/testing.md``

[^2]: Testing Rules, section 2a. ``.claude/rules/testing.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import VerbError, World

WIDTH = 48
HEIGHT = 48

# The status numbers the engine answers with.
IDLE = 0
OFFERED = 1
COUNTERED = 2
BOUND = 3
SETTLED = 4
DEFAULTED = 5

# The speech acts the log reports.
ACT_OFFER = 0
ACT_COUNTER = 1
ACT_ACCEPT = 2
ACT_REFUSE = 3
ACT_CLOSE = 4
ACT_REOPEN = 5
ACT_SETTLE = 6
ACT_DEFAULT = 7

FOOD = 0


def a_world(seed: int = 7) -> World:
    """Build a world in which both factions hold ground and stand somewhere."""
    world = World(width=WIDTH, height=HEIGHT, seed=seed, faction_count=2)
    world.found_run_for_every_faction(24)
    for _ in range(6):
        world.step(threads=1)
    return world


def a_tile_held_by(world: World, faction: int) -> tuple[int, int]:
    """Return one address whose ground the faction holds."""
    for r in range(world.height):
        for q in range(world.width):
            if world.tile_report(q, r)["holder"] == faction:
                return (q, r)
    message = f"no tile of this world is held by faction {faction}"
    raise AssertionError(message)


def give_presence(world: World, speaker: int, listener: int) -> None:
    """Put a unit of the speaker on ground the listener holds.

    The spawn happens between two steps and the caller speaks before the next
    one. The holding rule reads where units stand and takes the tile for the
    speaker on the next step, so a test that stepped here would lose the
    presence it just arranged.
    """
    world.spawn_soldiers([a_tile_held_by(world, listener)], speaker)
    assert world.stands_in_territory_of(speaker, listener)


def stocked_tiles(world: World, wanted: int) -> list[tuple[int, int]]:
    """Return addresses that carry food and that nobody holds."""
    found: list[tuple[int, int]] = []
    for r in range(world.height):
        for q in range(world.width):
            here = world.tile_report(q, r)
            if here["passable"] and here["stock"][FOOD] > 4 and here["holder"] is None:
                found.append((q, r))
                if len(found) == wanted:
                    return found
    message = "this world holds too few tiles that carry food and nobody holds"
    raise AssertionError(message)


def a_laden_unit(world: World, place: tuple[int, int], faction: int) -> int:
    """Spawn a unit of the faction at the address and fill its carry.

    The unit gathers where it stands. It has no home site, so nothing pulls it
    away, and it is still on the address when the caller reads it back.
    """
    units = world.spawn_soldiers([place], faction)
    world.order_gather(units, FOOD)
    for _ in range(3):
        world.step(threads=1)
    unit = int(units[0])
    tile = world.soldier_tile(unit)
    assert tile == place[1] * world.width + place[0], "the unit walked away"
    return unit


def bind_a_contract(
    world: World,
    give_amount: int,
    take_amount: int,
    term: int,
) -> None:
    """Run one offer, one counteroffer and one acceptance, with presence.

    Every act needs a unit of the speaker on the listener's ground, so this
    arranges presence in both directions and nothing steps until the contract
    binds.
    """
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)
    world.offer_trade(0, 1, FOOD, give_amount * 2, FOOD, take_amount, term)
    world.counter_trade(1, 0, FOOD, give_amount, FOOD, take_amount)
    world.accept_trade(0, 1)


def test_an_offer_a_counteroffer_and_an_acceptance_bind_a_contract() -> None:
    """The three acts of a negotiation, and what each leaves behind."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    after_offer = world.trade_status(0, 1)
    assert after_offer["status"] == OFFERED
    assert after_offer["turn"] == 1
    assert after_offer["give_amount"] == 10
    assert after_offer["take_amount"] == 4

    # The counteroffer restates both sides. The terms stay in the orientation
    # of the pair, so the give side is still what faction 0 owes.
    world.counter_trade(1, 0, 0, 6, 1, 4)
    after_counter = world.trade_status(0, 1)
    assert after_counter["status"] == COUNTERED
    assert after_counter["turn"] == 0
    assert after_counter["give_amount"] == 6
    assert after_counter["rounds"] == 2

    world.accept_trade(0, 1)
    after_accept = world.trade_status(0, 1)
    assert after_accept["status"] == BOUND
    assert after_accept["turn"] is None
    assert after_accept["deadline"] == world.tick + 50

    acts = world.trade_log_columns()["act"].tolist()
    assert acts == [ACT_OFFER, ACT_COUNTER, ACT_ACCEPT]


def test_a_party_may_not_speak_out_of_turn() -> None:
    """The party that spoke last waits for the other one."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    with pytest.raises(VerbError):
        world.counter_trade(0, 1, 0, 9, 1, 4)
    with pytest.raises(VerbError):
        world.accept_trade(0, 1)

    world.counter_trade(1, 0, 0, 6, 1, 4)
    with pytest.raises(VerbError):
        world.accept_trade(1, 0)


def test_a_bound_contract_moves_the_goods() -> None:
    """A unit carries the quantity onto the other party's settlement.

    The assertion is that the store of the receiving settlement rose and that
    the contract records the delivery. Nothing here moves a quantity between
    two stores, because nothing in the engine does.
    """
    world = a_world()
    place = stocked_tiles(world, 1)[0]
    a_laden_unit(world, place, 0)
    site = int(world.found_settlements([place], 1)[0])
    before = world.site_economy(site)["store"]

    bind_a_contract(world, give_amount=3, take_amount=2, term=200)
    world.step(threads=1)

    row = world.trade_status(0, 1)
    assert row["given"] == 3, "the contract records no delivery"
    after = world.site_economy(site)["store"]
    assert after > before, "the store of the receiving settlement did not rise"

    # The quantity that arrived is the quantity the contract named, in the
    # fixed-point scale the store keeps. Nothing above the debt moved.
    assert after - before == 3 * 65536

    # A delivery takes a quantity out of the carries and puts it into a store,
    # so it must credit the account that links the two. Nothing else fails
    # when it does not, and the conservation check is what does.
    assert world.check_invariants()


def test_a_contract_settles_when_both_parties_deliver() -> None:
    """Both debts reach zero, and the pair reports a settlement."""
    world = a_world()
    first, second = stocked_tiles(world, 2)
    a_laden_unit(world, first, 0)
    a_laden_unit(world, second, 1)
    world.found_settlements([first], 1)
    world.found_settlements([second], 0)

    bind_a_contract(world, give_amount=2, take_amount=2, term=200)
    world.step(threads=1)

    row = world.trade_status(0, 1)
    assert row["given"] == 2
    assert row["taken"] == 2
    assert row["status"] == SETTLED
    assert ACT_SETTLE in world.trade_log_columns()["act"].tolist()


def test_a_refusal_is_not_a_closed_door() -> None:
    """A plain no leaves the pair free to open a new negotiation at once."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    world.refuse_trade(1, 0)

    after = world.trade_status(0, 1)
    assert after["status"] == IDLE
    assert after["closed_until"] == 0

    # The same pair opens again on the next call, with no wait.
    world.offer_trade(0, 1, 0, 8, 1, 4, 50)
    assert world.trade_status(0, 1)["status"] == OFFERED


def test_a_terminal_refusal_closes_the_pair_and_says_when_it_opens() -> None:
    """No and no more counteroffers, and the caller can read how long."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    world.close_trade(1, 0, 25)

    closed_at = world.tick
    row = world.trade_status(0, 1)
    assert row["status"] == IDLE
    assert row["closed_until"] == closed_at + 25

    with pytest.raises(VerbError) as refusal:
        world.offer_trade(0, 1, 0, 8, 1, 4, 50)
    assert str(closed_at + 25) in str(refusal.value)
    assert "opens again" in str(refusal.value)

    # The closure is directional. Faction 1 shut its own door and promised no
    # silence of its own, so it may still open a negotiation toward faction 0.
    assert world.trade_status(1, 0)["closed_until"] == 0
    world.offer_trade(1, 0, 1, 4, 0, 8, 50)
    assert world.trade_status(1, 0)["status"] == OFFERED


def test_only_the_party_that_closed_a_pair_opens_it_again() -> None:
    """Nothing the refused party does shortens a terminal refusal."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    world.close_trade(1, 0, 25)

    # The refused party cannot clear the closure that was written against it.
    with pytest.raises(VerbError):
        world.reopen_trade(0, 1)
    assert world.trade_status(0, 1)["closed_until"] != 0

    world.reopen_trade(1, 0)
    assert world.trade_status(0, 1)["closed_until"] == 0
    world.offer_trade(0, 1, 0, 8, 1, 4, 50)
    assert world.trade_status(0, 1)["status"] == OFFERED


def test_a_closure_ends_at_the_step_it_named() -> None:
    """The pair opens at the step the closure states, and not one step later."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    world.close_trade(1, 0, 3)
    opens_at = world.trade_status(0, 1)["closed_until"]

    while world.tick < opens_at - 1:
        world.step(threads=1)
    give_presence(world, 0, 1)
    with pytest.raises(VerbError):
        world.offer_trade(0, 1, 0, 8, 1, 4, 50)

    world.step(threads=1)
    assert world.tick == opens_at
    give_presence(world, 0, 1)
    world.offer_trade(0, 1, 0, 8, 1, 4, 50)
    assert world.trade_status(0, 1)["status"] == OFFERED


def test_a_contract_that_is_not_kept_fails_at_its_deadline() -> None:
    """A party that cannot deliver defaults, and the default costs it time."""
    world = a_world()
    bind_a_contract(world, give_amount=5, take_amount=5, term=4)
    deadline = world.trade_status(0, 1)["deadline"]

    while world.tick < deadline:
        world.step(threads=1)

    # The log covers the last step alone, so the default is read on the step
    # that resolved it and not on a later one.
    acts = world.trade_log_columns()["act"].tolist()
    assert ACT_DEFAULT in acts

    row = world.trade_status(0, 1)
    assert row["status"] == DEFAULTED
    assert row["given"] == 0
    assert row["taken"] == 0

    # Both parties owed, so both lose the direction they would ask on again,
    # for as long as the contract ran.
    assert row["closed_until"] > world.tick
    assert world.trade_status(1, 0)["closed_until"] > world.tick

    give_presence(world, 0, 1)
    with pytest.raises(VerbError):
        world.offer_trade(0, 1, 0, 5, 1, 5, 10)


def test_a_partial_delivery_stays_where_it_arrived_when_a_contract_fails() -> None:
    """A failure returns nothing, because no unit carried it back."""
    world = a_world()
    place = stocked_tiles(world, 1)[0]
    a_laden_unit(world, place, 0)
    site = int(world.found_settlements([place], 1)[0])

    # Faction 0 can deliver two and faction 1 can deliver nothing, so the
    # contract fails with one side paid.
    bind_a_contract(world, give_amount=2, take_amount=9, term=3)
    before = world.site_economy(site)["store"]
    for _ in range(5):
        world.step(threads=1)

    row = world.trade_status(0, 1)
    assert row["status"] == DEFAULTED
    assert row["given"] == 2
    assert row["taken"] == 0
    assert world.site_economy(site)["store"] - before == 2 * 65536

    # Only the party that owed loses its direction.
    assert world.trade_status(1, 0)["closed_until"] > world.tick
    assert world.trade_status(0, 1)["closed_until"] == 0


def test_a_speech_act_needs_a_unit_on_the_other_party_s_ground() -> None:
    """The presence gate governs a trade, as it governs a message."""
    world = a_world()
    assert not world.stands_in_territory_of(0, 1)

    with pytest.raises(VerbError) as refusal:
        world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    assert "no unit of the speaker" in str(refusal.value)

    give_presence(world, 0, 1)
    world.offer_trade(0, 1, 0, 10, 1, 4, 50)

    # The other party must answer from inside the first party's territory.
    with pytest.raises(VerbError):
        world.counter_trade(1, 0, 0, 6, 1, 4)
    give_presence(world, 1, 0)
    world.counter_trade(1, 0, 0, 6, 1, 4)
    assert world.trade_status(0, 1)["status"] == COUNTERED


def test_a_pair_holds_one_live_negotiation() -> None:
    """A second offer on a live pair is refused, in either direction."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)

    world.offer_trade(0, 1, 0, 10, 1, 4, 50)
    with pytest.raises(VerbError):
        world.offer_trade(0, 1, 0, 9, 1, 4, 50)
    with pytest.raises(VerbError):
        world.offer_trade(1, 0, 1, 4, 0, 9, 50)


def test_the_engine_refuses_terms_that_bind_nothing() -> None:
    """A quantity of zero, a term of zero, and a party trading with itself."""
    world = a_world()
    give_presence(world, 0, 1)

    with pytest.raises(VerbError):
        world.offer_trade(0, 1, 0, 0, 1, 4, 50)
    with pytest.raises(VerbError):
        world.offer_trade(0, 1, 0, 10, 1, 4, 0)
    with pytest.raises(VerbError):
        world.offer_trade(0, 0, 0, 10, 1, 4, 50)
    with pytest.raises(VerbError):
        world.offer_trade(0, 1, 9, 10, 1, 4, 50)
    with pytest.raises(VerbError):
        world.close_trade(0, 1, 0)


def test_the_book_answers_every_pair_one_faction_is_party_to() -> None:
    """One crossing, and no loop over pairs in the control plane."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)
    world.offer_trade(0, 1, 0, 10, 1, 4, 50)

    book = world.trade_book(0)
    assert book["proposer"].dtype == np.uint16
    assert book["give_amount"].dtype == np.uint32
    assert len(book["status"]) == len(book["proposer"])
    pairs = list(
        zip(book["proposer"].tolist(), book["responder"].tolist(), strict=True)
    )
    assert (0, 1) in pairs
    assert (1, 0) in pairs
    assert (0, 0) not in pairs
    row = pairs.index((0, 1))
    assert int(book["status"][row]) == OFFERED
    assert int(book["give_amount"][row]) == 10


def test_a_world_that_never_traded_holds_no_book() -> None:
    """The plane costs nothing until somebody speaks."""
    world = a_world()
    assert len(world.trade_book(0)["status"]) == 0
    assert world.trade_status(0, 1)["status"] == IDLE


def test_the_thread_count_does_not_change_a_trade() -> None:
    """ADR-0001 D4, seen through the trade verbs."""
    hashes = []
    books = []
    for threads in (1, 2, 12):
        world = a_world()
        place = stocked_tiles(world, 1)[0]
        a_laden_unit(world, place, 0)
        world.found_settlements([place], 1)
        bind_a_contract(world, give_amount=3, take_amount=5, term=4)
        for _ in range(6):
            world.step(threads=threads)
        hashes.append(world.state_hash())
        books.append(world.trade_book(0)["status"].tobytes())
        assert world.check_invariants()

    assert hashes[0] == hashes[1] == hashes[2]
    assert books[0] == books[1] == books[2]
