"""Black-box tests of the per-site production queue.

Every test here starts at the Python boundary and drives the step. A test that
advanced a queue by hand would prove the arithmetic and not that the step
reaches it.[^1]

Each fixture writes its own cost row. The row a new world holds carries
placeholders that the balance harness will change, and a test that read them
would measure the register.[^2]

References
----------
[^1]: Testing policy. ``docs/TESTING.md``

[^2]: Balance register, the production queue. ``docs/reference/balance.md``

[^3]: ADR-0158, a site builds a typed unit from a bounded queue its store pays
for.
``docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md``
"""

from __future__ import annotations

import numpy as np
import numpy.typing as npt
import pytest

import cachette

# The raw integer of one whole unit in the project fixed-point scale.
ONE = 65536

# The row of the type table that the worker takes, and the row the soldier
# takes. Neither number means anything to the engine. The rows the world is
# built with are what give them meaning.
WORKER = 0
SOLDIER = 1

# The advances one fixture entry takes, and the residents it spends.
WORK = 2
PEOPLE = 1

# The people the fixture homes at its site. One above what one entry spends,
# so the site keeps a group after it builds.
GROUP = 3


def _site_with_a_queue(
    stock: int,
) -> tuple[cachette.World, int, npt.NDArray[np.uint64]]:
    """Build a world, found one site, home a group at it, and stock its store."""
    world = cachette.World(width=16, height=16, seed=0x0497, faction_count=2)
    world.set_queue_schedule(1, 0)
    world.set_queue_charge(0, ONE)
    world.define_build_cost(SOLDIER, work=WORK, people=PEOPLE, goods=[3 * ONE])
    sites = world.found_settlements([(0, 0)], faction=0)
    site = int(sites[0])
    world.set_settlement_store([site], stock)
    units = world.spawn_soldiers([(0, 0)] * GROUP, faction=0)
    world.set_home_site(units, site)
    return world, site, units


def test_a_queued_soldier_arrives_and_can_fight() -> None:
    world, site, _ = _site_with_a_queue(stock=40 * ONE)
    table = world.unit_type_table()
    assert int(table["attack"][SOLDIER]) > 0, "the soldier row must be able to fight"

    world.queue_unit(faction=0, site=site, unit_type=SOLDIER)
    queue = world.site_queue(site)
    assert list(queue["unit_type"]) == [SOLDIER]
    assert list(queue["work"]) == [0]

    for _ in range(WORK):
        world.step(threads=1)

    census = world.subsystem_census()
    assert census["queue_produced"] == 1
    assert census["queue_refused_without_a_person"] == 0
    assert census["queue_refused_without_goods"] == 0
    assert len(world.site_queue(site)["unit_type"]) == 0

    # The row is a total for the run. It still says what the queue made after
    # the tick that made it, so a zero in it means that no site ever built.
    for _ in range(3):
        world.step(threads=1)
    assert world.subsystem_census()["queue_produced"] == 1

    units = world.faction_units(faction=0)
    types = [world.unit_type(int(unit)) for unit in units["unit"]]
    assert types.count(SOLDIER) == 1, "one unit of the queued type must stand"
    assert types.count(WORKER) == GROUP - PEOPLE, "the site spent its people"


def test_a_queue_refuses_a_finished_entry_the_site_cannot_pay_for() -> None:
    # The extreme: a store that pays every advance and nothing more. The
    # charge is one for each of two advances, so the store holds two.
    world, site, _ = _site_with_a_queue(stock=2 * ONE)
    world.queue_unit(faction=0, site=site, unit_type=SOLDIER)
    for _ in range(WORK):
        world.step(threads=1)

    census = world.subsystem_census()
    assert census["queue_produced"] == 0
    assert census["queue_refused_without_goods"] == 1
    assert census["queue_refused_without_a_person"] == 0, "the two are counted apart"
    assert list(world.site_queue(site)["work"]) == [WORK], "the entry stays"


def test_the_verb_refuses_a_site_of_another_faction() -> None:
    world, site, _ = _site_with_a_queue(stock=40 * ONE)
    with pytest.raises(cachette.VerbError):
        world.queue_unit(faction=1, site=site, unit_type=SOLDIER)
    assert len(world.site_queue(site)["unit_type"]) == 0
    assert world.subsystem_census()["queue_refused_at_the_verb"] == 1


def test_a_cleared_entry_leaves_the_order_of_the_entries_behind_it() -> None:
    world, site, _ = _site_with_a_queue(stock=0)
    for unit_type in (SOLDIER, WORKER, SOLDIER):
        world.queue_unit(faction=0, site=site, unit_type=unit_type)
    world.clear_queue_entry(faction=0, site=site, position=0)
    assert list(world.site_queue(site)["unit_type"]) == [WORKER, SOLDIER]


def test_a_bound_of_zero_turns_the_queue_off() -> None:
    world, site, _ = _site_with_a_queue(stock=40 * ONE)
    assert world.queue_bound() > 0
    world.set_queue_bound(0)
    with pytest.raises(cachette.VerbError):
        world.queue_unit(faction=0, site=site, unit_type=SOLDIER)
    for _ in range(WORK + 1):
        world.step(threads=1)
    assert world.subsystem_census()["queue_produced"] == 0
