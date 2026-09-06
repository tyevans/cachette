"""Black-box tests of the housing of a site and the growth it bounds.

Every test here starts at the Python boundary. The reader answers the housing,
the residents and the free places of one site, and no test walks a population
to get them.[^1]

References
----------
[^1]: ADR-0157, a site's free places are its built housing less the residents
the engine counts.
``docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md``

[^2]: Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import pytest

import cachette

# The raw integer of one whole unit in the project fixed-point scale.
ONE = 65536

# The people the fixture homes at its site.
GROUP = 2


def _a_site_with_housing(housing: int, stock: int) -> tuple[cachette.World, int]:
    """Build a world, found one site, give it housing, and home a group at it."""
    # The world is wider than the coarsest lattice of the terrain generator, so
    # it holds ground as well as water.
    world = cachette.World(width=96, height=96, seed=0x0060, faction_count=2)
    world.set_growth_schedule(1, 0)
    world.set_housing_per_person(1)
    sites = world.found_settlements([(2, 2)], faction=0)
    site = int(sites[0])
    world.set_site_housing([site], housing)
    world.set_settlement_store([site], stock)
    units = world.spawn_soldiers([(2, 2)] * GROUP, faction=0)
    world.set_home_site(units, site)
    return world, site


def test_a_site_reads_its_housing_its_residents_and_its_free_places() -> None:
    world, site = _a_site_with_housing(housing=8, stock=0)
    # The derived resident count settles at a barrier, so the world steps once
    # before the reader answers about the group the fixture homed.
    world.set_economy_schedule(1, 0)
    world.step()
    report = world.site_housing(site)
    assert report["housing"] == 8
    assert report["residents"] == GROUP
    assert report["free_places"] == 8 - GROUP


def test_a_site_above_its_housing_reads_no_free_place() -> None:
    world, site = _a_site_with_housing(housing=1, stock=0)
    world.set_economy_schedule(1, 0)
    world.step()
    report = world.site_housing(site)
    assert report["residents"] == GROUP
    assert report["free_places"] == 0, "a site above its housing has no free place"


def test_the_housing_verb_writes_and_the_reader_follows() -> None:
    world, site = _a_site_with_housing(housing=4, stock=0)
    world.set_site_housing([site], 40)
    assert world.site_housing(site)["housing"] == 40


def test_a_dead_identity_refuses_the_reader() -> None:
    world, site = _a_site_with_housing(housing=4, stock=0)
    with pytest.raises(cachette.ViewError):
        world.site_housing(site + 1)


def test_the_births_of_the_tick_read_zero_when_a_site_cannot_grow() -> None:
    # The store holds nothing, so the site has room and no food.
    world, _ = _a_site_with_housing(housing=8, stock=0)
    world.step()
    assert world.births == 0, "a site with no food grew somebody"


def test_the_births_of_the_tick_read_above_zero_when_a_site_can_grow() -> None:
    world, site = _a_site_with_housing(housing=8, stock=20 * ONE)
    grew = 0
    for _ in range(5):
        world.step()
        grew += world.births
    assert grew > 0, "a site with food and room grew nobody"
    assert world.site_housing(site)["residents"] > GROUP
