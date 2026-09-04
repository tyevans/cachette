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
    world.define_unit_type(0, one, 0)
    world.define_unit_type(1, 4 * one, 2 * one)
    bowmen = world.spawn_soldiers([(0, 0)] * 4, 0)
    tank = world.spawn_soldiers([(0, 0)], 1)
    world.set_unit_types(bowmen, 0)
    world.set_unit_types(tank, 1)

    world.step(threads=2)

    population = world.faction_population()
    assert population[0] == 0, "the tank ends all four bowmen"
    assert population[1] == 1, "four bowmen reach the tank for exactly nothing"


def test_ten_thousand_bowmen_also_lose_to_one_tank() -> None:
    """A sum of zeroes stays zero, so the crowd changes nothing."""
    crowd = 10_000
    world = cachette.World(width=1, height=1, seed=1, faction_count=2)
    one = 65536
    world.define_unit_type(0, one, 0)
    world.define_unit_type(1, 4 * one, 2 * one)
    bowmen = world.spawn_soldiers([(0, 0)] * crowd, 0)
    tank = world.spawn_soldiers([(0, 0)], 1)
    world.set_unit_types(bowmen, 0)
    world.set_unit_types(tank, 1)

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
    world.define_unit_type(1, 65536, 0)
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
