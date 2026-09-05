"""The trade board, and a contract that carries land or a relation.

Every test here goes through the front door. It builds a world, sends the
verbs a player would send, steps, and reads what the engine answers. Nothing
here reaches into the engine.[^1]

**The fixture is built for the case.** A land transfer is compared against a
control world that ran the same step without the contract, so the tiles the
holding rule moved on its own do not count as the contract's work. The carrier
fixture puts a laden unit of the party that owes land on the other party's
settlement, which is the input that would let a carrier pay a land debt if the
engine let it.[^2]

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

BOUND = 3
SETTLED = 4

ACT_SETTLE = 6
ACT_TRANSFER_LAND = 8
ACT_STEP_RELATION = 9

RESOURCE = 0
LAND = 1
RELATION = 2

FOOD = 0
WOOD = 1

OFFERS = 0
WANTS = 1


def a_world(seed: int = 7) -> World:
    """Build a world in which both factions hold ground and stand somewhere."""
    world = World(width=WIDTH, height=HEIGHT, seed=seed, faction_count=2)
    world.found_run_for_every_faction(24)
    for _ in range(6):
        world.step(threads=1)
    return world


def tiles_held_by(world: World, faction: int) -> list[tuple[int, int]]:
    """Return every address whose ground the faction holds, ascending."""
    found: list[tuple[int, int]] = []
    for r in range(world.height):
        for q in range(world.width):
            if world.tile_report(q, r)["holder"] == faction:
                found.append((q, r))
    return found


def holders(world: World) -> list[int | None]:
    """Return the holder of every tile, in tile order."""
    return [
        world.tile_report(q, r)["holder"]
        for r in range(world.height)
        for q in range(world.width)
    ]


def give_presence(world: World, speaker: int, listener: int) -> None:
    """Put a unit of the speaker on ground the listener holds."""
    world.spawn_soldiers([tiles_held_by(world, listener)[0]], speaker)
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
    """Spawn a unit of the faction at the address and fill its carry."""
    units = world.spawn_soldiers([place], faction)
    world.order_gather(units, FOOD)
    for _ in range(3):
        world.step(threads=1)
    unit = int(units[0])
    assert world.soldier_tile(unit) == place[1] * world.width + place[0]
    return unit


def index_of(world: World, place: tuple[int, int]) -> int:
    return place[1] * world.width + place[0]


def test_existing_positional_calls_keep_working() -> None:
    """The widened verbs take the old seven and six positional arguments."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)
    world.offer_trade(0, 1, FOOD, 10, WOOD, 4, 50)
    world.counter_trade(1, 0, FOOD, 8, WOOD, 4)
    row = world.trade_status(0, 1)
    assert row["give_tag"] == RESOURCE
    assert row["take_tag"] == RESOURCE
    assert row["give_amount"] == 8
    assert len(row["give_tiles"]) == 0


def test_land_for_a_relation_moves_exactly_the_listed_tiles() -> None:
    """The listed tiles go to the creditor. Every other tile is as the control."""
    world = a_world()
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)
    mine = tiles_held_by(world, 0)
    listed = mine[: min(5, len(mine))]
    control = World(width=WIDTH, height=HEIGHT, seed=7, faction_count=2)
    control.found_run_for_every_faction(24)
    for _ in range(6):
        control.step(threads=1)
    give_presence(control, 0, 1)
    give_presence(control, 1, 0)
    assert control.state_hash() == world.state_hash(), "the control is not the world"

    world.offer_trade(
        0, 1, 0, 0, 0, 1, 50, give_tag=LAND, give_tiles=listed, take_tag=RELATION
    )
    row = world.trade_status(0, 1)
    assert row["give_tag"] == LAND
    assert row["give_amount"] == len(listed)
    assert row["give_tiles"].tolist() == sorted(index_of(world, p) for p in listed)
    assert row["take_tag"] == RELATION
    world.accept_trade(1, 0)

    world.step(threads=1)
    control.step(threads=1)

    after = holders(world)
    expected = holders(control)
    wanted = {index_of(world, p) for p in listed}
    for index, (got, want) in enumerate(zip(after, expected, strict=True)):
        if index in wanted:
            assert got == 1, f"tile {index} did not move to the creditor"
        else:
            assert got == want, f"tile {index} moved and was not listed"
    row = world.trade_status(0, 1)
    assert row["status"] == SETTLED
    acts = world.trade_log_columns()["act"].tolist()
    assert ACT_TRANSFER_LAND in acts
    assert ACT_STEP_RELATION in acts
    assert ACT_SETTLE in acts
    assert world.check_invariants()


def test_a_land_side_names_a_cell_or_a_list() -> None:
    """A cell resolves to its tiles, and an empty land side is refused."""
    world = a_world()
    give_presence(world, 0, 1)
    cell = world.cell_tiles(0, 0)
    assert cell.dtype == np.uint32
    assert cell.tolist() == sorted(cell.tolist())
    with pytest.raises(VerbError, match="names a cell, a list of tiles, or both"):
        world.offer_trade(0, 1, 0, 0, 0, 1, 50, give_tag=LAND, take_tag=RELATION)
    # A whole cell is above the stand-in bound, so the bound refuses it first.
    with pytest.raises(VerbError, match=f"names {len(cell)} tiles, and the bound is"):
        world.offer_trade(
            0, 1, 0, 0, 0, 1, 50, give_tag=LAND, give_cell=(0, 0), take_tag=RELATION
        )
    world.set_land_list_bound(len(cell))
    with pytest.raises(VerbError, match="does not hold tile"):
        world.offer_trade(
            0, 1, 0, 0, 0, 1, 50, give_tag=LAND, give_cell=(0, 0), take_tag=RELATION
        )
    with pytest.raises(VerbError, match="names no consideration kind"):
        world.offer_trade(0, 1, 0, 1, 0, 1, 50, give_tag=5)


def test_a_carrier_pays_no_land_debt() -> None:
    """A laden unit on the creditor's settlement delivers nothing against land.

    Faction zero owes land and faction one owes food. A laden unit of faction
    zero stands on a settlement of faction one, which is the position a
    resource carrier delivers from. The land side must stay at zero, because no
    unit carries a tile.
    """
    world = a_world()
    place = stocked_tiles(world, 1)[0]
    a_laden_unit(world, place, 0)
    world.found_settlements([place], 1)
    give_presence(world, 0, 1)
    give_presence(world, 1, 0)
    mine = tiles_held_by(world, 0)[:2]
    world.offer_trade(
        0, 1, 0, 0, FOOD, 5, 50, give_tag=LAND, give_tiles=mine, take_tag=RESOURCE
    )
    world.accept_trade(1, 0)
    world.step(threads=1)
    row = world.trade_status(0, 1)
    assert row["status"] == BOUND
    assert row["given"] == 0, "a carrier paid a land debt"
    assert row["taken"] == 0
    assert world.check_invariants()


def test_an_upgrade_on_traded_ground_is_refused_and_names_the_blocker() -> None:
    """The refusal text names BLK-036, so the commit that closes it finds this."""
    world = a_world()
    mine = tiles_held_by(world, 0)
    site = mine[len(mine) // 2]
    builder = world.spawn_soldiers([site], 0)
    world.order_build(builder, 0)
    for _ in range(12):
        if world.tile_report(*site)["upgrade"] is not None:
            break
        world.step(threads=1)
    assert world.tile_report(*site)["upgrade"] is not None, (
        "the fixture raised no upgrade"
    )
    give_presence(world, 0, 1)
    with pytest.raises(VerbError, match="BLK-036"):
        world.offer_trade(
            0, 1, 0, 0, 0, 1, 50, give_tag=LAND, give_tiles=[site], take_tag=RELATION
        )


def test_the_board_replaces_whole_and_refuses_more_rows_than_the_bound() -> None:
    """A write replaces every row, and a write over the bound changes nothing."""
    world = a_world()
    assert len(world.market(0)["good"]) == 0
    world.advertise(0, [(FOOD, 10, OFFERS, WOOD, 4), (WOOD, 3, WANTS, FOOD, 9)])
    board = world.market(0)
    assert board["good"].tolist() == [FOOD, WOOD]
    assert board["quantity"].tolist() == [10, 3]
    assert board["wants"].tolist() == [OFFERS, WANTS]
    assert board["asking_good"].tolist() == [WOOD, FOOD]
    assert board["asking_quantity"].tolist() == [4, 9]

    world.advertise(0, [(WOOD, 3, WANTS, FOOD, 9)])
    assert world.market(0)["good"].tolist() == [WOOD]

    bound = world.board_rows()
    with pytest.raises(VerbError, match=f"the board holds {bound} rows"):
        world.advertise(0, [(FOOD, n + 1, OFFERS, WOOD, 1) for n in range(bound + 1)])
    assert world.market(0)["good"].tolist() == [WOOD], (
        "a refused write changed the board"
    )
    with pytest.raises(VerbError, match="names neither offers"):
        world.advertise(0, [(FOOD, 1, 2, WOOD, 1)])
    world.advertise(0, [])
    assert len(world.market(0)["good"]) == 0
    assert len(world.market(1)["good"]) == 0


def test_two_worlds_that_differ_in_one_board_row_have_different_hashes() -> None:
    posted = a_world()
    quiet = a_world()
    assert posted.state_hash() == quiet.state_hash()
    posted.advertise(1, [(FOOD, 1, WANTS, WOOD, 1)])
    assert posted.state_hash() != quiet.state_hash()
    other = a_world()
    other.advertise(1, [(FOOD, 2, WANTS, WOOD, 1)])
    assert posted.state_hash() != other.state_hash()


def test_the_land_list_bound_is_a_parameter() -> None:
    world = a_world()
    give_presence(world, 0, 1)
    mine = tiles_held_by(world, 0)[:3]
    world.set_land_list_bound(2)
    assert world.land_list_bound() == 2
    with pytest.raises(VerbError, match="names 3 tiles, and the bound is 2"):
        world.offer_trade(
            0, 1, 0, 0, 0, 1, 50, give_tag=LAND, give_tiles=mine, take_tag=RELATION
        )
    world.set_land_list_bound(3)
    world.offer_trade(
        0, 1, 0, 0, 0, 1, 50, give_tag=LAND, give_tiles=mine, take_tag=RELATION
    )
    book = world.trade_book(0)
    assert book["give_tag"].tolist() == [LAND, RESOURCE]
