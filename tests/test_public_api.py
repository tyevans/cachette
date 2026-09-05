"""Black-box tests of the installed Python package.

Every test here goes through the front door. It uses only names that the
package exports.

References
----------
Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

from collections.abc import Mapping
from typing import Any

import numpy as np
import numpy.typing as npt
import pytest

import cachette


def test_the_package_reports_a_version() -> None:
    assert isinstance(cachette.__version__, str)
    assert cachette.__version__ != ""


def test_a_new_world_starts_at_tick_zero(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    assert world.tick == 0
    assert world.tile_count == 256
    assert world.check_invariants()


def test_a_step_advances_the_tick(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    world.step(threads=2)
    assert world.tick == 1
    assert world.check_invariants()


def test_the_tile_column_is_an_integer_array(seed: int) -> None:
    # ADR-0002 D1 forbids floating point in simulated state, so the column
    # is an integer array and never a float array.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=1)
    values = world.tile_values()
    assert values.dtype == np.int32
    assert values.shape == (64,)


def test_the_thread_count_does_not_change_the_result(seed: int) -> None:
    # ADR-0001 D4: the highest-value test in the project, seen from the
    # Python side.
    hashes = []
    logs = []
    for threads in (1, 2, 12):
        world = cachette.World(width=32, height=32, seed=seed, faction_count=4)
        for _ in range(4):
            world.step(threads=threads)
        hashes.append(world.state_hash())
        logs.append(world.event_log_bytes())
    assert len(set(hashes)) == 1
    assert logs[0] == logs[1] == logs[2]


def test_a_step_at_zero_threads_raises_a_typed_error(seed: int) -> None:
    # ADR-0046: the engine never raises a bare runtime error.
    world = cachette.World(width=4, height=4, seed=seed, faction_count=1)
    with pytest.raises(cachette.StepError):
        world.step(threads=0)


def test_an_empty_extent_raises_a_typed_error(seed: int) -> None:
    # ADR-0046: the engine never raises a bare runtime error.
    with pytest.raises(cachette.ConfigError):
        cachette.World(width=0, height=4, seed=seed, faction_count=1)
    with pytest.raises(cachette.ConfigError):
        cachette.World(width=4, height=0, seed=seed, faction_count=1)


def test_the_world_reports_its_extent(seed: int) -> None:
    # ADR-0017 D1: the world is a rhombus, so the extent is a width and a
    # height and the tile count follows from them.
    world = cachette.World(width=8, height=4, seed=seed, faction_count=1)
    assert world.width == 8
    assert world.height == 4
    assert world.tile_count == 32


def test_the_repr_names_the_constructor_arguments(seed: int) -> None:
    # A repr that names a field the constructor does not take is a small lie
    # that costs a reader a failed call. This test is what makes the two
    # agree, because nothing else compares them.
    world = cachette.World(width=8, height=4, seed=seed, faction_count=1)
    text = repr(world)
    assert "width=8" in text
    assert "height=4" in text
    assert "tile_count" not in text


def test_every_error_type_sits_under_one_root() -> None:
    # ADR-0046: a user catches broadly or narrowly.
    for error in (
        cachette.ConfigError,
        cachette.StepError,
        cachette.SelectorError,
        cachette.VerbError,
        cachette.ViewError,
        cachette.DeterminismError,
        cachette.EnginePanic,
    ):
        assert issubclass(error, cachette.CachetteError)


def test_two_worlds_run_independently(seed: int) -> None:
    # ADR-0047: many worlds in one interpreter.
    first = cachette.World(width=16, height=8, seed=seed, faction_count=2)
    second = cachette.World(width=16, height=8, seed=seed, faction_count=2)
    first.step(threads=1)
    assert first.state_hash() != second.state_hash()
    second.step(threads=1)
    assert first.state_hash() == second.state_hash()


def _named(columns: Mapping[str, Any]) -> dict[str, npt.NDArray[Any]]:
    """Read the columns by name, for a test that walks every field.

    The stub types each set of columns as a mapping with known keys, which
    is what a reader wants: it names the fields and refuses a typo. A test
    that walks the fields has no literal key to give, so it widens the type
    here rather than at each call.
    """
    return dict(columns)


def test_the_event_columns_carry_the_fields_by_name(seed: int) -> None:
    # DEC-060: the bindings return one column for each field, so a reader
    # holds no byte offset, no field width and no field order.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.step(threads=1)
    columns = world.event_log_columns()

    assert set(columns) == {"tick", "tile", "value", "holder", "kind"}
    for name, column in _named(columns).items():
        assert len(column) == world.event_count, name


def test_no_event_column_is_a_floating_point_array(seed: int) -> None:
    # ADR-0002 D1 bans a floating point number in simulated state. A float
    # that enters through this interface is the same defect one layer out.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.step(threads=1)
    for columns in (
        _named(world.event_log_columns()),
        _named(world.gather_log_columns()),
        _named(world.fell_log_columns()),
    ):
        for name, column in columns.items():
            assert np.issubdtype(column.dtype, np.integer), name
    assert world.event_log_columns()["value"].dtype == np.int32


def test_a_unit_identity_survives_the_round_trip(seed: int) -> None:
    # ADR-0085 D1 and D3: Python holds the whole identity and gives it
    # back, and the engine resolves it.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    (unit,) = world.spawn_soldiers([(0, 0)], 1)
    assert world.soldier_tile(int(unit)) == 0


def test_the_identity_of_a_dead_unit_refuses(seed: int) -> None:
    # The defect this guards: a reader holds an identity, the unit dies,
    # another unit takes the slot, and the reader reports on the new unit
    # with nothing failing. Testing Rules section 2 records the engine-side
    # instance of it.
    #
    # This test cannot check that the arena reused the slot, because
    # checking it would mean taking the identity apart, and no reader here
    # may do that. The Rust test of the same fixture makes that check, in
    # crates/cachette-core/tests/identity_resolution.rs.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    (dead,) = (int(unit) for unit in world.spawn_soldiers([(0, 0)], 1))
    world.despawn_soldiers([dead])
    (living,) = (int(unit) for unit in world.spawn_soldiers([(0, 0)], 1))

    assert living != dead, "the arena must mint a new identity"

    # ADR-0046: the refusal is typed. ADR-0085 D3: it never falls back to
    # the unit that now holds the slot.
    #
    # **The write verb is what covers the resolution.** Deleting the
    # generation comparison in resolve_soldier was measured against this
    # test. The read below stayed green, because the arena compares the
    # generation a second time when it reads a tile, so the read refuses
    # whether or not resolution did. The despawn below went red. A reader
    # who takes the read line as the coverage would be wrong.[^1]
    #
    # [^1]: Findings register, FND-148. `docs/FINDINGS.md`
    with pytest.raises(cachette.ViewError):
        world.soldier_tile(dead)
    with pytest.raises(cachette.ViewError):
        world.despawn_soldiers([dead])
    assert world.soldier_tile(living) == 0


def test_python_cannot_compose_an_identity(seed: int) -> None:
    # ADR-0085 D2: the bindings expose no way to build an identity. A
    # caller that assembles one from an index it chose gets a refusal.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.spawn_soldiers([(0, 0)], 1)
    with pytest.raises(cachette.ViewError):
        world.soldier_tile(0)
    with pytest.raises(cachette.ViewError):
        # A number the engine never gave out. The caller has no way to know
        # which numbers are identities, which is the point.
        world.soldier_tile(2**40 + 7)


# The world the gather tests build, and the tiles they use.
#
# The seed is part of the fixture, not a detail. The terrain is generated
# from it, and a resource sits on the ground that carries it, so the seed
# decides whether these four tiles hold anything to gather.
#
# Measured at 16 by 16 with the engine's own deposit read. Seed 1 holds food
# at (0, 0), (1, 0) and (1, 1), and wood at all four. Seed 7 holds no food at
# any of them, only stone at two. Seed 42 admits no unit at any of them. The
# first version of this test used seed 7 and asked for food, and it failed for
# that reason.
#
# **The control plane cannot check this.** No read tells Python where a
# resource is, so the seed was chosen against the engine's read from Rust and
# recorded here. That gap is the finding, not an accident of this test.
GATHER_SEED = 1
GATHER_KIND = 0

# The count is the assertion's, not the world's. Four units prove that a
# gather event carries a resolvable identity.
#
# They cross in one call. DEC-063 made the spawn verb set-valued, so nothing
# here shows a caller repeating a verb over the mass tier.
GATHER_ADDRESSES = ((0, 0), (1, 0), (0, 1), (1, 1))


def test_a_gather_event_names_a_unit_that_resolves() -> None:
    # ADR-0085 D1: the unit column holds the whole identity, so a reader
    # can follow the unit that took the amount.
    #
    # The seed is the fixture's own, not the shared one, because the ground
    # under the four tiles is what the test needs and the shared seed does
    # not promise it.
    world = cachette.World(width=16, height=16, seed=GATHER_SEED, faction_count=2)
    units = world.spawn_soldiers(GATHER_ADDRESSES, 1)
    world.order_gather(units, GATHER_KIND)

    for _ in range(8):
        world.step(threads=1)
        if world.gather_count:
            break
    assert world.gather_count, "the fixture must produce a gather event"

    columns = world.gather_log_columns()
    assert set(columns) == {"tick", "unit", "tile", "amount", "kind"}
    for row in range(len(columns["unit"])):
        unit = int(columns["unit"][row])
        assert world.soldier_tile(unit) == int(columns["tile"][row])
        assert int(columns["amount"][row]) > 0


def test_the_bindings_expose_no_slot_index(seed: int) -> None:
    # ADR-0085 D1: no column of slot indices, and no accessor that splits
    # an identity into its parts.
    for name in dir(cachette.World):
        assert "slot" not in name, name
        assert "generation" not in name, name


def test_a_refused_spawn_set_leaves_no_soldier_behind(seed: int) -> None:
    # The set is all or nothing. A caller that got half a population and an
    # error would have to work out which half, and the engine knows.
    #
    # The count is what makes this test able to fail. Without it the test
    # would assert only that the call raised, which it would do whether or
    # not the rollback ran.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    before = world.soldier_count

    with pytest.raises(cachette.VerbError) as refused:
        world.spawn_soldiers([(0, 0), (1, 0), (99, 99)], 1)

    assert "(99, 99)" in str(refused.value), "the error names the address"
    assert world.soldier_count == before


def test_a_refused_order_set_gives_no_order(seed: int) -> None:
    # Every identity resolves before any order is given, so one dead
    # identity leaves the whole set untouched.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    units = [int(unit) for unit in world.spawn_soldiers([(0, 0), (1, 0)], 1)]
    world.despawn_soldiers([units[1]])

    with pytest.raises(cachette.ViewError):
        world.order_gather(units, 0)

    # The living unit took no order, so the world grants nothing.
    world.step(threads=1)
    assert world.gather_count == 0


# The addresses of the fixture that reads a faction as a set.
#
# The count is the assertion's, not the world's. Five units in one faction and
# two in another prove that the read answers for one faction and not for the
# world.
FACTION_ADDRESSES = ((0, 0), (1, 0), (2, 0), (3, 0), (4, 0))
OTHER_ADDRESSES = ((0, 1), (1, 1))


def test_the_units_of_a_faction_come_back_as_columns(seed: int) -> None:
    # The read takes a faction and answers with columns. It takes no identity
    # from the caller, so every entry names a live soldier and no entry needs
    # a validity mask.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=3)
    world.spawn_soldiers(FACTION_ADDRESSES, 1)
    world.spawn_soldiers(OTHER_ADDRESSES, 2)

    columns = world.faction_units(1)
    assert set(columns) == {"unit", "tile"}
    # The element types are the ones the doc comment declares. A caller that
    # read a different width would read a different number.
    assert columns["unit"].dtype == np.uint64
    assert columns["tile"].dtype == np.uint32
    assert len(columns["unit"]) == len(FACTION_ADDRESSES)
    assert len(world.faction_units(2)["unit"]) == len(OTHER_ADDRESSES)
    assert len(world.faction_units(0)["unit"]) == 0


def test_the_set_read_agrees_with_the_singular_read(seed: int) -> None:
    # The set read must answer what the loop answered, for every unit. A read
    # that disagreed with the singular one would be a second answer to one
    # question, and nothing would fail when the two disagreed.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=3)
    world.spawn_soldiers(FACTION_ADDRESSES, 1)
    world.spawn_soldiers(OTHER_ADDRESSES, 2)
    world.step(threads=1)

    columns = world.faction_units(1)
    # This loop is the thing the read exists to remove. It runs here because
    # the fixture holds five units, and it is the only way to prove that the
    # column says what the loop said.
    assert len(columns["unit"]), "the fixture must hold a unit"
    for row in range(len(columns["unit"])):
        unit = int(columns["unit"][row])
        assert world.soldier_tile(unit) == int(columns["tile"][row])


def test_the_set_read_returns_arrays_and_not_one_object_for_each_unit(
    seed: int,
) -> None:
    # **One crossing, and no Python object for any entity.** The result is two
    # NumPy arrays that hold the engine's own values. A read that built one
    # object for each unit would cross once for each of them at the target
    # scale.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=3)
    world.spawn_soldiers(FACTION_ADDRESSES, 1)

    columns = world.faction_units(1)
    assert isinstance(columns["unit"], np.ndarray)
    assert isinstance(columns["tile"], np.ndarray)
    # An array of Python objects would have this element type, and it is the
    # failure this assertion names.
    assert columns["unit"].dtype != np.dtype(object)


def test_a_dead_unit_leaves_the_set_read(seed: int) -> None:
    # The engine builds the set at the moment of the call, so a unit that died
    # is not in it. Nothing here stands for nothing.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=3)
    units = [int(unit) for unit in world.spawn_soldiers(FACTION_ADDRESSES, 1)]
    world.despawn_soldiers([units[0]])

    columns = world.faction_units(1)
    assert len(columns["unit"]) == len(FACTION_ADDRESSES) - 1
    assert units[0] not in {int(unit) for unit in columns["unit"]}


def test_a_sent_set_takes_one_call_and_leaves_the_units_alive(seed: int) -> None:
    # The control plane names a set of units and a set of tiles in one call.
    # The engine builds one field and every unit of the set climbs it.
    world = cachette.World(width=64, height=64, seed=seed, faction_count=3)
    units = world.spawn_soldiers(FACTION_ADDRESSES, 1)

    world.send_units_to(units, [(32, 32)])
    for _ in range(4):
        world.step(threads=1)

    # The read side answers where the set went, in one call.
    columns = world.faction_units(1)
    assert len(columns["unit"]) == len(FACTION_ADDRESSES)

    world.stop_sending(units)


def test_a_send_refuses_a_destination_the_world_does_not_hold(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    units = world.spawn_soldiers([(0, 0)], 1)
    with pytest.raises(cachette.VerbError):
        world.send_units_to(units, [(1, 1)], 2**16 - 1)


def test_a_send_refuses_an_address_outside_the_world(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    units = world.spawn_soldiers([(0, 0)], 1)
    with pytest.raises(cachette.VerbError):
        world.send_units_to(units, [(99, 99)])


def test_a_refused_send_set_sends_nobody(seed: int) -> None:
    # Every identity resolves before anything changes, so one dead identity
    # leaves the whole set untouched.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    units = [int(unit) for unit in world.spawn_soldiers([(0, 0), (1, 0)], 1)]
    world.despawn_soldiers([units[1]])

    with pytest.raises(cachette.ViewError):
        world.send_units_to(units, [(2, 2)])


def test_one_tank_still_kills_four_bowmen(seed: int) -> None:
    """The acceptance test the project owner set, at the Python boundary.

    A bowman cannot exceed the armour of a tank, so any number of bowmen
    deliver exactly nothing. The tank delivers four whole casualties.
    """
    world = cachette.World(width=1, height=1, seed=1, faction_count=2)
    one = 65536
    world.define_unit_type(0, one, 0, **_worker_columns(world))
    world.define_unit_type(1, 4 * one, 2 * one, **_worker_columns(world))
    bowmen = world.spawn_soldiers([(0, 0)] * 4, 0)
    tank = world.spawn_soldiers([(0, 0)], 1)
    world.set_unit_types(bowmen, 0)
    world.set_unit_types(tank, 1)
    # The contest resolves a meeting only across a pair at war.
    world.set_relation(0, 1, FAR_BELOW_EVERY_EDGE)
    assert world.relation_band(0, 1) == 0

    world.step(threads=2)

    population = world.faction_population()
    assert population[0] == 0, "the tank ends all four bowmen"
    assert population[1] == 1, "four bowmen reach the tank for exactly nothing"


def test_ten_thousand_bowmen_also_lose_to_one_tank() -> None:
    """A sum of zeroes stays zero, so the crowd changes nothing."""
    crowd = 10_000
    world = cachette.World(width=1, height=1, seed=1, faction_count=2)
    one = 65536
    world.define_unit_type(0, one, 0, **_worker_columns(world))
    world.define_unit_type(1, 4 * one, 2 * one, **_worker_columns(world))
    bowmen = world.spawn_soldiers([(0, 0)] * crowd, 0)
    tank = world.spawn_soldiers([(0, 0)], 1)
    world.set_unit_types(bowmen, 0)
    world.set_unit_types(tank, 1)
    # The contest resolves a meeting only across a pair at war.
    world.set_relation(0, 1, FAR_BELOW_EVERY_EDGE)
    assert world.relation_band(0, 1) == 0

    world.step(threads=2)

    population = world.faction_population()
    assert population[1] == 1, "no number of bowmen reaches the tank"
    assert population[0] == crowd - 4, "the tank ends what its attack pays for"


def test_a_refused_unit_type_set_gives_no_type() -> None:
    """One dead identity leaves the whole set untouched.

    This test reads the write through what it changes: a unit of the armed
    type ends a unit of the other faction, and a unit that kept the unarmed
    type ends nobody. Without that step the test would assert only that the
    call raised, which it would do whether or not the set was written.

    The module now also reads the type of one unit back, and a test beside
    this one asserts the refusal that way.[^1]

    References
    ----------
    [^1]: The unit type and log tests.
    ``tests/test_unit_types_and_logs.py``
    """
    world = cachette.World(width=1, height=1, seed=1, faction_count=2)
    # Type one reaches. Type zero, which every new soldier carries, does not.
    world.define_unit_type(1, 65536, 0, **_worker_columns(world))
    attackers = [int(unit) for unit in world.spawn_soldiers([(0, 0)] * 2, 0)]
    world.spawn_soldiers([(0, 0)], 1)
    world.despawn_soldiers([attackers[1]])

    with pytest.raises(cachette.ViewError):
        world.set_unit_types(attackers, 1)

    world.step(threads=2)
    assert world.faction_population()[1] == 1, (
        "the living attacker took no type, so it ends nobody"
    )


def test_a_unit_type_the_table_does_not_hold_is_refused(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    units = [int(unit) for unit in world.spawn_soldiers([(0, 0)], 1)]
    with pytest.raises(cachette.VerbError) as refused:
        world.set_unit_types(units, 200)
    assert "200" in str(refused.value), "the error names the number"


# The world the fallen tests build, and the units they put in it.
#
# The seed is part of the fixture. The terrain comes from it, and a unit
# stands only on ground that admits it, so the seed decides whether these two
# tiles hold anybody at all. Seed 1 admits a unit at both, and the gather
# fixture above measured the same ground.
#
# **The two factions stand on neighbouring tiles, not on one tile.** A unit
# reaches every unit of another faction on its own tile and on the six tiles
# beside it, and admission refuses a step onto a full tile without reading the
# faction, so a fixture that needed co-occupation would measure the case a
# fight is least about.
#
# The attack and the armour are content, and this fixture states them. The
# tank delivers four whole casualties and carries an armour above the attack
# of the bowman, so it ends the four bowmen and takes nothing back. The
# fixture therefore produces a fallen log of exactly four entries, of one
# faction and one type, which is what the assertions below read.
FALLEN_SEED = 1
BOWMAN = 0
TANK = 1
# A relation value below every edge the register could set. It is not a copy
# of an edge: the fixture asserts the band it lands in.
FAR_BELOW_EVERY_EDGE = -(1 << 20)


def _worker_columns(world: cachette.World) -> dict[str, int]:
    """Return the six columns beyond attack and armour, as the worker holds them.

    The worker is row zero of the table a new world is built with. The values
    are read from the engine rather than written here, so the test holds no
    second copy of them.
    """
    table = world.unit_type_table()
    return {
        name: int(table[name][0])  # type: ignore[literal-required]
        for name in table
        if name not in ("attack", "armour")
    }


BOWMAN_TILE = (0, 0)
TANK_TILE = (1, 0)
BOWMAN_FACTION = 1
TANK_FACTION = 2
BOWMAN_ADDRESSES = (BOWMAN_TILE, BOWMAN_TILE, BOWMAN_TILE, BOWMAN_TILE)
WHOLE_UNIT = 1 << 16


def _contested_world() -> tuple[cachette.World, list[int], list[int]]:
    """Build a world in which one tank meets four bowmen, and spawn both.

    Returns the world, the bowman identities and the tank identity, before
    any step. The caller steps it.
    """
    world = cachette.World(width=16, height=16, seed=FALLEN_SEED, faction_count=3)
    world.define_unit_type(BOWMAN, WHOLE_UNIT, 0, **_worker_columns(world))
    world.define_unit_type(
        TANK, 4 * WHOLE_UNIT, 2 * WHOLE_UNIT, **_worker_columns(world)
    )
    bowmen = world.spawn_soldiers(BOWMAN_ADDRESSES, BOWMAN_FACTION)
    world.set_unit_types(bowmen, BOWMAN)
    tank = world.spawn_soldiers([TANK_TILE], TANK_FACTION)
    world.set_unit_types(tank, TANK)
    # The contest resolves a meeting only across a pair at war, so the
    # fixture declares one. The value sits far below any edge, and the band
    # read proves it landed in the war band.
    world.set_relation(BOWMAN_FACTION, TANK_FACTION, FAR_BELOW_EVERY_EDGE)
    assert world.relation_band(BOWMAN_FACTION, TANK_FACTION) == 0
    return world, [int(unit) for unit in bowmen], [int(unit) for unit in tank]


def _fight(world: cachette.World, threads: int = 1) -> None:
    """Step the world until the fallen log holds an entry."""
    for _ in range(8):
        world.step(threads=threads)
        if world.fell_count:
            return
    raise AssertionError("the fixture must produce a fallen event")


def test_the_fallen_columns_carry_the_fields_by_name() -> None:
    # DEC-060: the bindings return one column for each field, so a reader
    # holds no byte offset, no field width and no field order.
    world, _bowmen, _tank = _contested_world()
    _fight(world)

    columns = world.fell_log_columns()
    assert set(columns) == {"tick", "unit", "tile", "faction", "unit_type"}
    for name, column in _named(columns).items():
        assert len(column) == world.fell_count, name

    assert columns["tick"].dtype == np.uint64
    assert columns["unit"].dtype == np.uint64
    assert columns["tile"].dtype == np.uint32
    assert columns["faction"].dtype == np.uint16
    assert columns["unit_type"].dtype == np.uint8


def test_a_fallen_event_says_who_fell_and_where() -> None:
    # The whole point of the log: a caller learns who fell, which faction it
    # belonged to, where it stood and which type it carried.
    world, bowmen, tank = _contested_world()
    _fight(world)

    columns = world.fell_log_columns()
    assert world.fell_count == len(bowmen), "the tank ends every bowman"

    fallen = {int(unit) for unit in columns["unit"]}
    assert fallen == set(bowmen), "the log names the units the fixture spawned"
    assert not fallen & set(tank), "the tank takes nothing back"

    width = world.width
    tile = BOWMAN_TILE[0] + BOWMAN_TILE[1] * width
    for row in range(world.fell_count):
        assert int(columns["tick"][row]) == world.tick
        assert int(columns["tile"][row]) == tile
        assert int(columns["faction"][row]) == BOWMAN_FACTION
        assert int(columns["unit_type"][row]) == BOWMAN


def test_an_identity_in_the_fallen_log_is_dead() -> None:
    # ADR-0085 D3: the step ended the unit the identity names, so the engine
    # refuses the identity rather than report on the next occupant of the
    # slot. The tile column is what places the death, for that reason.
    world, _bowmen, _tank = _contested_world()
    _fight(world)

    columns = world.fell_log_columns()
    for row in range(world.fell_count):
        with pytest.raises(cachette.ViewError):
            world.soldier_tile(int(columns["unit"][row]))


def test_a_step_with_no_fight_gives_an_empty_log() -> None:
    # The hazard the doc comment names: the log covers the last step alone.
    # A step that ends nobody must give empty columns and never the entries
    # of the step before it.
    world, bowmen, _tank = _contested_world()
    _fight(world)
    assert world.fell_count == len(bowmen)

    # Nobody is left to fight the tank, so the next step ends nobody.
    world.step(threads=1)
    assert world.fell_count == 0
    columns = world.fell_log_columns()
    for name, column in _named(columns).items():
        assert len(column) == 0, name


def test_a_new_world_gives_an_empty_fallen_log(seed: int) -> None:
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    assert world.fell_count == 0
    columns = world.fell_log_columns()
    assert set(columns) == {"tick", "unit", "tile", "faction", "unit_type"}
    for name, column in _named(columns).items():
        assert len(column) == 0, name


def test_the_thread_count_does_not_change_the_fallen_log() -> None:
    # ADR-0001 D4 and ADR-0004 D1: the step ends the marked units in
    # ascending slot order, so the log a caller reads is the same at every
    # thread count. A claim proved at one thread count proves nothing.
    logs = []
    for threads in (1, 2, 12):
        world, _bowmen, _tank = _contested_world()
        _fight(world, threads=threads)
        columns = world.fell_log_columns()
        logs.append([_named(columns)[name].tolist() for name in sorted(columns)])
    assert logs[0] == logs[1] == logs[2]


def _open_ground(world: cachette.World, count: int) -> list[tuple[int, int]]:
    """Find addresses that admit a unit, in row order.

    The ground is generated, so a test cannot name a tile and assume it is
    open. It searches instead.
    """
    found: list[tuple[int, int]] = []
    for row in range(world.height):
        for column in range(world.width):
            if world.tile_report(column, row)["passable"]:
                found.append((column, row))
                if len(found) == count:
                    return found
    raise AssertionError("the world holds no open ground")


def test_the_control_plane_converts_a_set_of_units() -> None:
    """The verb changes the faction of a whole set in one call."""
    world = cachette.World(width=16, height=16, seed=1, faction_count=2)
    where = _open_ground(world, 1)[0]
    units = world.spawn_soldiers([where] * 4, 0)

    world.convert_units(units, 1)

    population = world.faction_population()
    assert population[0] == 0, "the set kept its old faction"
    assert population[1] == 4, "the new faction did not gain the set"
    # The identity survives, so the same array names the same units.
    world.convert_units(units, 0)
    assert world.faction_population()[0] == 4, "the identities stopped naming the units"


def test_a_conversion_set_holding_a_dead_unit_changes_nobody() -> None:
    world = cachette.World(width=16, height=16, seed=1, faction_count=2)
    where = _open_ground(world, 1)[0]
    units = [int(unit) for unit in world.spawn_soldiers([where] * 3, 0)]
    world.despawn_soldiers([units[0]])

    with pytest.raises(cachette.ViewError):
        world.convert_units(units, 1)

    assert world.faction_population()[1] == 0, (
        "the verb changed a unit before it refused the set"
    )


def test_a_conversion_to_a_faction_the_world_does_not_hold_is_refused() -> None:
    world = cachette.World(width=16, height=16, seed=1, faction_count=2)
    where = _open_ground(world, 1)[0]
    units = world.spawn_soldiers([where], 0)
    with pytest.raises(cachette.VerbError) as refused:
        world.convert_units(units, 9)
    assert "9" in str(refused.value), "the error names the number"


def test_belief_takes_units_and_the_log_says_where_they_went() -> None:
    """A source of belief takes the units of the faction that has none."""
    world = cachette.World(width=64, height=64, seed=3, faction_count=2)
    seat = _open_ground(world, 8)
    world.spawn_soldiers(seat, 0)
    world.set_influence_source(1, [seat[0]], 65535)
    assert world.influence(1, seat[0][0], seat[0][1]) == 0, "the field starts empty"
    # A leader at peace with a faction converts none of its units, so the
    # source faction is put one band below peace toward the old faction. The
    # value sits far below the peace edge and the band read proves it is not
    # in the peace band.
    world.set_relation(1, 0, FAR_BELOW_EVERY_EDGE)
    assert world.relation_band(1, 0) < 2

    gained = 0
    entries: list[tuple[int, int]] = []
    for _ in range(12):
        world.step(threads=4)
        changed = world.converted_log_columns()
        gained += world.converted_count
        entries.extend(
            (int(one), int(two))
            for one, two in zip(
                changed["from_faction"], changed["to_faction"], strict=True
            )
        )

    assert gained > 0, "the source of belief took nobody"
    assert world.influence(1, seat[0][0], seat[0][1]) > 0, "the field never climbed"
    assert all(entry == (0, 1) for entry in entries), (
        "the log reports a change that the field did not make"
    )
    assert world.faction_population()[1] == gained, (
        "the population count disagrees with the log"
    )
