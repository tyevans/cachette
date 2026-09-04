"""Black-box tests of the economy tuning knobs.

Every test here starts at the Python boundary. The engine held each of these
knobs before any binding called one, and their own Rust tests passed the whole
time.[^1] A test that built the mechanism again would prove the same thing
again, so each test below drives the installed package and then asserts that
the simulation moved.

Two kinds of witness appear here.

A behavioural witness steps the world and reads a value the engine wrote. It
is the stronger one, and it is used wherever the boundary publishes a read
that reaches the effect.

A hash witness compares the whole-world state hash before and after a write.
It proves two things at once: the write reached the engine, and the value is
hashed state rather than configuration outside the hash. It is used where the
boundary publishes no read of the effect.

References
----------
[^1]: Findings register, FND-481. ``docs/FINDINGS.md``

[^2]: Findings register, FND-480. ``docs/FINDINGS.md``

Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import pytest

import cachette

# One Q16.16 unit, as its raw integer. The boundary never carries a float,
# because a float in simulated state does not add associatively.
ONE = 1 << 16

# The doc comment of `set_economy_schedule` states this ceiling on the period.
DOCUMENTED_PERIOD_CEILING = 32767

# The doc comment of `influence` and of `set_influence_source` states this
# reference unit. It is not the Q16.16 scale.
DOCUMENTED_INFLUENCE_UNIT = 65535

# The doc comment of `set_recovery_rules` states one period for each resource
# kind, in the order food, wood, stone.
DOCUMENTED_RESOURCE_KINDS = 3

# The resource kinds, as `order_gather` and the recovery rules take them.
FOOD = 0
WOOD = 1
STONE = 2


def a_world(seed: int, width: int = 16, height: int = 16) -> cachette.World:
    """Build a small world with two factions."""
    return cachette.World(width=width, height=height, seed=seed, faction_count=2)


def a_site(world: cachette.World, address: tuple[int, int] = (8, 8)) -> int:
    """Found one settlement and return its identity."""
    return int(world.found_settlements([address], faction=0)[0])


def store_of(world: cachette.World, site: int) -> int:
    """Return what the site holds of commodity zero."""
    return int(world.site_economy(site)["store"])


# ---------------------------------------------------------------- production


def test_a_production_rate_set_from_python_fills_the_store(seed: int) -> None:
    """The knob to hunt: one that accepts a value and changes nothing."""
    world = a_world(seed)
    site = a_site(world)
    world.set_economy_schedule(period=1, phase=0)
    world.set_settlement_store([site], 0)
    world.set_production_rate([site], 2 * ONE)
    assert world.site_economy(site)["production"] == 2 * ONE

    before = store_of(world, site)
    world.step(threads=2)
    after = store_of(world, site)

    # The rate is what one tick earns and the period is one, so one step adds
    # exactly two units.
    assert after - before == 2 * ONE


def test_a_production_rate_of_zero_leaves_the_store_where_it_was(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    world.set_economy_schedule(period=1, phase=0)
    world.set_settlement_store([site], 5 * ONE)
    world.set_production_rate([site], 0)

    world.step(threads=2)

    assert store_of(world, site) == 5 * ONE


def test_one_command_sets_the_production_rate_of_a_set_of_sites(seed: int) -> None:
    world = a_world(seed)
    made = world.found_settlements([(q, 0) for q in range(4)], 0)
    sites = [int(site) for site in made]
    world.set_economy_schedule(period=1, phase=0)
    world.set_settlement_store(sites, 0)

    world.set_production_rate(sites, ONE)
    world.step(threads=2)

    assert [store_of(world, site) for site in sites] == [ONE] * 4


# -------------------------------------------------------------------- upkeep


def test_an_upkeep_rate_set_from_python_empties_the_store(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    world.set_economy_schedule(period=1, phase=0)
    world.set_settlement_store([site], 10 * ONE)
    world.set_upkeep_rate([site], 3 * ONE)
    assert world.site_economy(site)["upkeep"] == 3 * ONE

    world.step(threads=2)

    assert store_of(world, site) == 7 * ONE


def test_production_pays_the_upkeep_of_the_same_application(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    world.set_economy_schedule(period=1, phase=0)
    world.set_settlement_store([site], 0)
    world.set_production_rate([site], 4 * ONE)
    world.set_upkeep_rate([site], 4 * ONE)

    world.step(threads=2)

    # Production runs first, so a site that earns what it owes stays solvent.
    assert store_of(world, site) == 0


# ------------------------------------------------------------------ schedule


def test_the_economy_schedule_decides_which_ticks_move_the_store(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    world.set_settlement_store([site], 0)
    world.set_production_rate([site], ONE)

    # A long period with a far phase puts no application inside the steps
    # below. The first due tick is tick 500.
    world.set_economy_schedule(period=DOCUMENTED_PERIOD_CEILING, phase=500)
    for _ in range(3):
        world.step(threads=2)
    assert store_of(world, site) == 0

    world.set_economy_schedule(period=1, phase=0)
    world.step(threads=2)
    assert store_of(world, site) == ONE


def test_the_period_does_not_change_what_a_site_earns_over_a_span(seed: int) -> None:
    """A rate is what one tick earns, so the period cancels out."""
    earned = []
    for period in (1, 4):
        world = a_world(seed)
        site = a_site(world)
        world.set_economy_schedule(period=period, phase=0)
        world.set_settlement_store([site], 0)
        world.set_production_rate([site], ONE)
        for _ in range(8):
            world.step(threads=2)
        earned.append(store_of(world, site))

    assert earned[0] == earned[1]


# ----------------------------------------------------------------- the store


def test_a_store_written_from_python_is_what_the_site_reports(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)

    world.set_settlement_store([site], 12 * ONE)

    assert store_of(world, site) == 12 * ONE


def test_a_store_write_is_absolute_and_not_relative(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)

    world.set_settlement_store([site], 12 * ONE)
    world.set_settlement_store([site], 3 * ONE)

    assert store_of(world, site) == 3 * ONE


# --------------------------------------------------------------- the recovery


def a_depleted_world(seed: int) -> tuple[cachette.World, tuple[int, int]]:
    """Build a world where units have taken from one tile, and return it."""
    world = a_world(seed, width=32, height=32)
    address = _a_tile_with_stock(world)
    units = world.spawn_soldiers([address] * 8, faction=0)
    world.order_gather(units, FOOD)
    for _ in range(12):
        world.step(threads=2)
    assert world.tile_report(*address)["taken"][FOOD] > 0
    return world, address


def _a_tile_with_stock(world: cachette.World) -> tuple[int, int]:
    for q in range(world.width):
        for r in range(world.height):
            report = world.tile_report(q, r)
            if report["passable"] and report["generated"][FOOD] > 4:
                return (q, r)
    message = "the world holds no passable tile with stock"
    raise AssertionError(message)


def test_the_recovery_rules_written_from_python_change_what_a_deposit_holds(
    seed: int,
) -> None:
    fast, address = a_depleted_world(seed)
    slow, other = a_depleted_world(seed)
    assert address == other
    started = fast.tile_report(*address)["taken"][FOOD]
    assert started == slow.tile_report(*other)["taken"][FOOD]

    # Nothing gathers from here on, so the only thing that can move the taken
    # amount is the recovery.
    fast.despawn_soldiers(_every_unit(fast))
    slow.despawn_soldiers(_every_unit(slow))
    fast.set_recovery_rules([1, 1, 1])
    slow.set_recovery_rules([None, None, None])

    for _ in range(40):
        fast.step(threads=2)
        slow.step(threads=2)

    left = fast.tile_report(*address)["taken"][FOOD]
    assert left < slow.tile_report(*other)["taken"][FOOD]


def test_the_recovery_rules_read_back_what_was_written(seed: int) -> None:
    world = a_world(seed)

    world.set_recovery_rules([7, None, 3])

    assert world.recovery_rules() == [7, None, 3]


def test_the_recovery_rules_are_outside_the_state_hash(seed: int) -> None:
    """Assert a defect, so the test fails when the defect is repaired.

    The recovery rules govern the step and stand outside the state hash.[^2]
    """
    world = a_world(seed)
    before = world.state_hash()

    world.set_recovery_rules([1, 1, 1])

    assert world.state_hash() == before
    assert world.recovery_rules() == [1, 1, 1]


def _every_unit(world: cachette.World) -> list[int]:
    return [int(unit) for unit in world.faction_units(0)["unit"]]


# ------------------------------------------------------------ deed threshold


def test_the_deed_threshold_reads_back_what_was_written(seed: int) -> None:
    world = a_world(seed)

    world.set_deed_threshold(41)

    assert world.deed_threshold() == 41


def test_the_deed_threshold_is_hashed_state(seed: int) -> None:
    world = a_world(seed)
    before = world.state_hash()

    world.set_deed_threshold(world.deed_threshold() + 1)

    assert world.state_hash() != before


# ----------------------------------------------------------------- home site


def test_the_home_site_is_hashed_state(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    units = world.spawn_soldiers([(8, 8)], faction=0)
    before = world.state_hash()

    world.set_home_site(units, site)

    assert world.state_hash() != before


def test_a_home_site_can_be_taken_away(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    units = world.spawn_soldiers([(8, 8)], faction=0)
    world.set_home_site(units, site)
    housed = world.state_hash()

    world.set_home_site(units)

    assert world.state_hash() != housed


# ----------------------------------------------------------------- influence


def test_an_influence_source_set_from_python_spreads_on_the_next_step(
    seed: int,
) -> None:
    world = a_world(seed, width=64, height=64)
    quiet = a_world(seed, width=64, height=64)
    assert world.influence(0, 32, 32) == 0

    world.set_influence_source(0, [(32, 32)], DOCUMENTED_INFLUENCE_UNIT)
    world.step(threads=2)
    quiet.step(threads=2)

    assert world.influence(0, 32, 32) > 0
    assert quiet.influence(0, 32, 32) == 0


def test_one_command_sets_the_influence_source_of_a_set_of_places(seed: int) -> None:
    world = a_world(seed, width=64, height=64)
    places = [(8, 8), (32, 32), (56, 56)]

    world.set_influence_source(0, places, DOCUMENTED_INFLUENCE_UNIT)
    world.step(threads=2)

    assert all(world.influence(0, q, r) > 0 for q, r in places)


def test_an_influence_source_belongs_to_one_faction(seed: int) -> None:
    world = a_world(seed, width=64, height=64)

    world.set_influence_source(0, [(32, 32)], DOCUMENTED_INFLUENCE_UNIT)
    world.step(threads=2)

    assert world.influence(0, 32, 32) > 0
    assert world.influence(1, 32, 32) == 0


# ------------------------------------------------------------------ refusals


def test_a_production_rate_below_zero_raises_and_writes_nothing(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    world.set_production_rate([site], 5 * ONE)
    before = world.state_hash()

    with pytest.raises(cachette.VerbError):
        world.set_production_rate([site], -1)

    assert world.state_hash() == before
    assert world.site_economy(site)["production"] == 5 * ONE


def test_an_upkeep_rate_below_zero_raises_and_writes_nothing(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    before = world.state_hash()

    with pytest.raises(cachette.VerbError):
        world.set_upkeep_rate([site], -ONE)

    assert world.state_hash() == before


def test_a_refused_rate_leaves_every_site_of_the_set_alone(seed: int) -> None:
    """The engine refuses the value before it writes the first site.

    The boundary states no rule of its own about the rate or the commodity.
    The engine holds both rules, and neither refusal depends on the site, so
    the first site refuses and no site is written.
    """
    world = a_world(seed)
    made = world.found_settlements([(q, 0) for q in range(3)], 0)
    sites = [int(site) for site in made]
    world.set_production_rate(sites, 5 * ONE)
    before = world.state_hash()

    with pytest.raises(cachette.VerbError):
        world.set_production_rate(sites, -ONE)
    with pytest.raises(cachette.VerbError):
        world.set_production_rate(sites, 2 * ONE, 1)

    assert world.state_hash() == before
    assert all(world.site_economy(site)["production"] == 5 * ONE for site in sites)


def test_a_commodity_the_world_does_not_hold_raises(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)

    with pytest.raises(cachette.VerbError):
        world.set_production_rate([site], ONE, 1)
    with pytest.raises(cachette.VerbError):
        world.set_upkeep_rate([site], ONE, 1)
    with pytest.raises(cachette.VerbError):
        world.set_settlement_store([site], ONE, 1)


def test_a_stale_site_in_the_set_leaves_every_other_site_alone(seed: int) -> None:
    world = a_world(seed)
    good = a_site(world)
    stale = good + 1
    before = world.state_hash()

    with pytest.raises(cachette.ViewError):
        world.set_production_rate([good, stale], 9 * ONE)

    assert world.state_hash() == before
    assert world.site_economy(good)["production"] == 0


def test_a_period_of_zero_raises(seed: int) -> None:
    world = a_world(seed)
    before = world.state_hash()

    with pytest.raises(cachette.VerbError):
        world.set_economy_schedule(period=0, phase=0)

    assert world.state_hash() == before


def test_a_period_above_the_documented_ceiling_raises(seed: int) -> None:
    world = a_world(seed)

    world.set_economy_schedule(period=DOCUMENTED_PERIOD_CEILING, phase=0)
    with pytest.raises(cachette.VerbError):
        world.set_economy_schedule(period=DOCUMENTED_PERIOD_CEILING + 1, phase=0)


def test_the_recovery_rules_need_one_period_for_each_kind(seed: int) -> None:
    world = a_world(seed)
    before = world.recovery_rules()
    assert len(before) == DOCUMENTED_RESOURCE_KINDS

    with pytest.raises(ValueError):
        world.set_recovery_rules([1, 1])

    assert world.recovery_rules() == before


def test_a_recovery_period_of_zero_raises(seed: int) -> None:
    world = a_world(seed)
    before = world.recovery_rules()

    with pytest.raises(cachette.VerbError):
        world.set_recovery_rules([0, 1, 1])

    assert world.recovery_rules() == before


def test_a_stale_unit_in_a_home_set_leaves_the_world_alone(seed: int) -> None:
    world = a_world(seed)
    site = a_site(world)
    units = [int(unit) for unit in world.spawn_soldiers([(8, 8)], faction=0)]
    before = world.state_hash()

    with pytest.raises(cachette.ViewError):
        world.set_home_site([units[0], units[0] + 1], site)

    assert world.state_hash() == before


def test_a_home_site_that_names_no_settlement_raises(seed: int) -> None:
    world = a_world(seed)
    units = world.spawn_soldiers([(8, 8)], faction=0)
    before = world.state_hash()

    with pytest.raises(cachette.ViewError):
        world.set_home_site(units, 12345)

    assert world.state_hash() == before


def test_an_address_outside_the_world_refuses_the_whole_influence_set(
    seed: int,
) -> None:
    world = a_world(seed, width=64, height=64)
    before = world.state_hash()

    with pytest.raises(cachette.ViewError):
        world.set_influence_source(0, [(32, 32), (900, 900)], DOCUMENTED_INFLUENCE_UNIT)

    assert world.state_hash() == before


def test_a_faction_the_world_does_not_hold_refuses_an_influence_write(
    seed: int,
) -> None:
    world = a_world(seed, width=64, height=64)
    before = world.state_hash()

    with pytest.raises(cachette.ViewError):
        world.set_influence_source(9, [(32, 32)], DOCUMENTED_INFLUENCE_UNIT)
    with pytest.raises(cachette.ViewError):
        world.influence(9, 32, 32)

    assert world.state_hash() == before


# --------------------------------------------------------------- determinism


@pytest.mark.parametrize("threads", [1, 2, 12])
def test_a_world_tuned_from_python_gives_one_answer_at_any_thread_count(
    seed: int, threads: int
) -> None:
    world = a_world(seed, width=32, height=32)
    site = int(world.found_settlements([(16, 16)], faction=0)[0])
    units = world.spawn_soldiers([(16, 16)] * 4, faction=0)
    world.set_economy_schedule(period=2, phase=1)
    world.set_production_rate([site], 3 * ONE)
    world.set_upkeep_rate([site], ONE)
    world.set_settlement_store([site], 20 * ONE)
    world.set_deed_threshold(9)
    world.set_home_site(units, site)
    world.set_influence_source(0, [(16, 16)], DOCUMENTED_INFLUENCE_UNIT)
    world.set_recovery_rules([2, 5, None])

    for _ in range(6):
        world.step(threads=threads)

    assert world.check_invariants()
    assert (world.state_hash(), store_of(world, site)) == _expected_tuning(seed)


def _expected_tuning(seed: int) -> tuple[int, int]:
    """Return the answer of the same run at one thread.

    The comparison is between two runs and never between a run and itself, so
    the test can fail.[^1]

    References
    ----------
    [^1]: Testing rules, section 1. ``.claude/rules/testing.md``
    """
    world = a_world(seed, width=32, height=32)
    site = int(world.found_settlements([(16, 16)], faction=0)[0])
    units = world.spawn_soldiers([(16, 16)] * 4, faction=0)
    world.set_economy_schedule(period=2, phase=1)
    world.set_production_rate([site], 3 * ONE)
    world.set_upkeep_rate([site], ONE)
    world.set_settlement_store([site], 20 * ONE)
    world.set_deed_threshold(9)
    world.set_home_site(units, site)
    world.set_influence_source(0, [(16, 16)], DOCUMENTED_INFLUENCE_UNIT)
    world.set_recovery_rules([2, 5, None])
    for _ in range(6):
        world.step(threads=1)
    return (world.state_hash(), store_of(world, site))
