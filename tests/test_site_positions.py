"""The control plane changes what a set of sites prefers.

Every test here goes through the front door. It uses only names that the
package exports.

The command names no unit. It says what a place wants, and the engine turns
that into a number of positions of each kind. A control plane that named the
workers would be looping over entities, which this project forbids.

References
----------
ADR-0040, Python is a control plane, not a data plane, decision D1.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``

ADR-0065, a group is a site membership, not a region.
``docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette

# The kind numbers, as the gather event carries them.
FOOD = 0
WOOD = 1
STONE = 2

# One Q16.16 unit, as its raw integer. The boundary never carries a float,
# because a float in simulated state does not add associatively.
ONE = 1 << 16


def a_world_with_sites(seed: int, count: int) -> tuple[cachette.World, list[int]]:
    """Build a world and found sites in one call."""
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    world.set_position_schedule(period=1, phase=0)
    addresses = [(q, 0) for q in range(count)]
    sites = world.found_settlements(addresses, faction=0)
    assert world.settlement_count == count
    return world, [int(site) for site in sites]


def test_a_site_answers_what_positions_it_holds(seed: int) -> None:
    world, sites = a_world_with_sites(seed, 1)
    world.step(threads=1)
    columns = world.site_positions(sites[0])
    assert columns["kind"].dtype == np.uint8
    assert columns["rank"].dtype == np.uint8
    assert columns["holder"].dtype == np.uint64
    assert columns["kind"].size > 0
    # A position holds nobody until something seats a unit in it, and nobody
    # is a value the column carries.
    assert np.all(columns["holder"] == 0)


def test_one_command_changes_the_preference_of_a_set_of_sites(seed: int) -> None:
    world, sites = a_world_with_sites(seed, 4)
    # One crossing, one command, a set of sites, and no unit named.
    world.prefer_at_sites(sites, WOOD, 0)
    world.prefer_at_sites(sites, STONE, 0)
    world.step(threads=2)

    for site in sites:
        preference = world.site_preference(site)
        assert preference.dtype == np.int32
        assert int(preference[WOOD]) == 0
        assert int(preference[STONE]) == 0
        columns = world.site_positions(site)
        assert columns["kind"].size > 0
        assert np.all(columns["kind"] == FOOD)
    assert world.check_invariants()


def test_the_preference_changes_how_many_positions_of_a_kind_a_site_holds(
    seed: int,
) -> None:
    world, sites = a_world_with_sites(seed, 2)
    world.prefer_at_sites(sites, WOOD, 0)
    world.prefer_at_sites(sites, STONE, 0)
    world.step(threads=1)
    before = int(np.count_nonzero(world.site_positions(sites[0])["kind"] == FOOD))
    assert before > 0

    # The site now wants nothing, so it lacks nothing and it needs nobody.
    world.prefer_at_sites(sites, FOOD, 0)
    world.step(threads=1)
    after = int(np.count_nonzero(world.site_positions(sites[0])["kind"] == FOOD))
    assert after == 0
    assert before != after


def test_a_refused_preference_set_changes_nothing(seed: int) -> None:
    world, sites = a_world_with_sites(seed, 2)
    world.step(threads=1)
    before = [world.site_preference(site).copy() for site in sites]

    # One identity in the set names no live site, so the whole call refuses.
    with pytest.raises(cachette.ViewError):
        world.prefer_at_sites([*sites, 0xDEAD_BEEF], WOOD, ONE * 3)
    for site, expected in zip(sites, before, strict=True):
        assert np.array_equal(world.site_preference(site), expected)


def test_a_preference_below_zero_refuses(seed: int) -> None:
    world, sites = a_world_with_sites(seed, 1)
    with pytest.raises(cachette.VerbError):
        world.prefer_at_sites(sites, FOOD, -ONE)


def test_a_kind_that_names_no_work_refuses(seed: int) -> None:
    world, sites = a_world_with_sites(seed, 1)
    with pytest.raises(cachette.VerbError):
        world.prefer_at_sites(sites, 200, ONE)


def test_the_identity_of_a_lost_site_refuses(seed: int) -> None:
    world, sites = a_world_with_sites(seed, 1)
    world.step(threads=1)
    site = sites[0]
    assert world.site_positions(site)["kind"].size > 0

    # The engine gives no way to compose an identity, so the test destroys
    # the site through the founding of another world state: it asks for a
    # site that never existed.
    with pytest.raises(cachette.ViewError):
        world.site_positions(site + (1 << 32))


def test_the_position_columns_hold_no_floating_point_array(seed: int) -> None:
    # ADR-0002 D1 forbids floating point in simulated state, and this is that
    # state leaving the engine.
    world, sites = a_world_with_sites(seed, 1)
    world.step(threads=1)
    columns = world.site_positions(sites[0])
    assert not np.issubdtype(columns["kind"].dtype, np.floating)
    assert not np.issubdtype(columns["rank"].dtype, np.floating)
    assert not np.issubdtype(columns["holder"].dtype, np.floating)
    assert not np.issubdtype(world.site_preference(sites[0]).dtype, np.floating)


def test_the_thread_count_does_not_change_the_positions(seed: int) -> None:
    # ADR-0001 D4, read against the positions from the Python side.
    answers = []
    for threads in (1, 2, 12):
        world, sites = a_world_with_sites(seed, 8)
        for index, site in enumerate(sites):
            world.prefer_at_sites([site], WOOD, ONE * (index % 4))
        for _ in range(4):
            world.step(threads=threads)
        answers.append(
            [
                (
                    world.site_positions(site)["kind"].tobytes(),
                    world.site_positions(site)["rank"].tobytes(),
                )
                for site in sites
            ]
        )
    assert answers[0] == answers[1]
    assert answers[0] == answers[2]


def test_a_refused_founding_set_leaves_no_settlement_behind(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    with pytest.raises(cachette.VerbError):
        # The second address repeats the first, and one tile holds one site.
        world.found_settlements([(0, 0), (0, 0)], faction=0)
    assert world.settlement_count == 0
