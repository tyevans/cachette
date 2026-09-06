"""Black-box tests of the plan verbs and the plan readers.

A plan is a bounded list of projects. A project is one tile and one category.
A unit builds a category that asks for no held ground, such as a road, only
inside a project.[^1]

Every test here starts at the Python boundary and drives the installed
package.[^2]

References
----------
[^1]: ADR-0152, a faction plans its roads and zones with one solver. ``docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md``

Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import pytest

import cachette

# The upgrade categories, as the verbs take them.
ROAD = 0
TERRACE = 1
# One past the last category of the table. The number names nothing.
NO_SUCH_CATEGORY = 9


def _open_address(world: cachette.World) -> tuple[int, int]:
    """Return an address of ground that admits a unit."""
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["passable"]:
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def _open_addresses(world: cachette.World, count: int) -> list[tuple[int, int]]:
    """Return several addresses of ground that admits a unit."""
    found: list[tuple[int, int]] = []
    for r in range(world.height):
        for q in range(world.width):
            if world.tile_report(q, r)["passable"]:
                found.append((q, r))
                if len(found) == count:
                    return found
    message = f"the world holds fewer than {count} open addresses"
    raise AssertionError(message)


def test_a_caller_zones_a_project_and_reads_it_back(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    assert world.plan(0) == []
    assert world.project_at(0, *address) is None

    world.zone_projects(0, [address], ROAD)

    assert world.project_at(0, *address) == ROAD
    assert world.plan(0) == [(address[0], address[1], ROAD)]
    # The plan belongs to one faction. Another faction reads nothing.
    assert world.plan(1) == []
    assert world.project_at(1, *address) is None


def test_a_plan_comes_back_in_tile_order(seed: int) -> None:
    # The writes go in reverse, so the order of the list is the plan's own
    # and never the order the caller wrote in.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    places = _open_addresses(world, 5)
    for address in reversed(places):
        world.zone_projects(0, [address], ROAD)
    read = [(q, r) for q, r, _ in world.plan(0)]
    assert read == sorted(read, key=lambda place: (place[1], place[0]))
    assert len(read) == len(places)


def test_a_caller_clears_a_project(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    places = _open_addresses(world, 3)
    world.zone_projects(0, places, ROAD)
    assert len(world.plan(0)) == 3

    assert world.clear_projects(0, places[:2]) == 2
    assert len(world.plan(0)) == 1
    # A tile the plan does not name is cleared without an error, and counts
    # for nothing.
    assert world.clear_projects(0, places[:2]) == 0


def test_a_road_outside_a_project_is_refused_and_a_zoned_road_is_taken(
    seed: int,
) -> None:
    # A road asks for no held ground, so the plan is the bound on it.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    with pytest.raises(cachette.VerbError):
        world.order_build(units, ROAD)
    assert world.build_order(int(units[0])) is None

    world.zone_projects(0, [address], ROAD)
    world.order_build(units, ROAD)
    assert world.build_order(int(units[0])) == ROAD


def test_a_category_the_verb_does_not_hold_is_refused(seed: int) -> None:
    # ADR-0046: the engine never raises a bare runtime error.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    with pytest.raises(cachette.VerbError):
        world.zone_projects(0, [address], NO_SUCH_CATEGORY)
    assert world.plan(0) == []


def test_an_address_outside_the_world_is_refused(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    with pytest.raises(cachette.ViewError):
        world.zone_projects(0, [(-1, -1)], ROAD)
    with pytest.raises(cachette.ViewError):
        world.clear_projects(0, [(-1, -1)])
    with pytest.raises(cachette.ViewError):
        world.project_at(0, -1, -1)
    assert world.plan(0) == []


def test_a_project_on_ground_the_faction_does_not_hold_is_refused_for_a_terrace(
    seed: int,
) -> None:
    # A terrace asks for held ground, and no city of this faction reaches the
    # tile, so the plan refuses to zone it. A road on the same tile is taken,
    # because a road is how a faction reaches ground it does not hold.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    with pytest.raises(cachette.VerbError):
        world.zone_projects(0, [address], TERRACE)
    world.zone_projects(0, [address], ROAD)
    assert world.project_at(0, *address) == ROAD


def test_the_solver_writes_a_plan_that_a_caller_can_read(seed: int) -> None:
    # The engine writes into the same list the caller writes into, so a
    # caller reads what the solver planned.
    world = cachette.World(width=48, height=48, seed=seed, faction_count=2)
    world.found_group(4, 0)
    for _ in range(4):
        world.step(threads=2)
    zoned = [row for faction in range(2) for row in world.plan(faction)]
    assert zoned, "the solver planned nothing in four ticks"
    for q, r, category in zoned:
        assert world.tile_report(q, r)["passable"]
        # The solver plans a way between two places, and it plans a raise of
        # the yield when the stores fall short. Both are categories the table
        # holds, and neither is a number the table does not.
        assert category in (ROAD, TERRACE)
