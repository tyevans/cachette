"""Black-box tests of the ground a city holds, and of the build refusal.

A faction holds the ground its cities reach. A tile no city reaches is held
by nobody. A unit builds only on ground its own faction holds, and a road
anywhere.[^1]

Every test here starts at the Python boundary and drives the installed
package.[^2]

References
----------
[^1]: ADR-0150, held ground is the ground within reach of a city its faction
      owns, decisions D1, D2 and D4.
      ``docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md``

[^2]: Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import pytest

import cachette

# The upgrade kinds the engine holds, as `order_build` takes them.
ROAD = 0
TERRACE = 1


def _open_address(world: cachette.World) -> tuple[int, int]:
    """Return an address of ground that admits a unit."""
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["passable"]:
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def _world(seed: int) -> tuple[cachette.World, tuple[int, int], int]:
    """Return a world, an address of open ground, and a city standing there."""
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    address = _open_address(world)
    sites = world.found_settlements([address], faction=0)
    world.step(threads=2)
    return world, address, int(sites[0])


def test_a_city_gives_its_faction_the_ground_it_reaches(seed: int) -> None:
    world, address, site = _world(seed)
    assert world.holds(0, *address) is True
    assert world.holds(1, *address) is False

    # The reach is a whole number of steps, and the ground stops there.
    reach = world.city_reach(site)
    assert reach >= 1
    far = (address[0], address[1] + reach + 1)
    if 0 <= far[1] < world.height:
        assert world.holds(0, *far) is False


def test_a_tile_outside_the_world_and_a_faction_the_world_lacks_are_refused(
    seed: int,
) -> None:
    world, _address, _site = _world(seed)
    with pytest.raises(cachette.ViewError):
        world.holds(0, -1, 0)
    with pytest.raises(cachette.ViewError):
        world.holds(7, 0, 0)
    with pytest.raises(cachette.ViewError):
        world.city_reach(0)


def test_a_build_off_the_builders_own_ground_is_refused_and_a_road_is_not(
    seed: int,
) -> None:
    world, address, _site = _world(seed)
    guest = world.spawn_soldiers([address], faction=1)
    # The ground belongs to the first faction, so a terrace is refused.
    with pytest.raises(cachette.VerbError):
        world.order_build(guest, TERRACE)
    assert world.build_order(int(guest[0])) is None

    # A road is the one exception, and it is permitted anywhere.
    world.order_build(guest, ROAD)
    assert world.build_order(int(guest[0])) == ROAD

    # The faction that holds the ground may build anything on it.
    owner = world.spawn_soldiers([address], faction=0)
    world.order_build(owner, TERRACE)
    assert world.build_order(int(owner[0])) == TERRACE
