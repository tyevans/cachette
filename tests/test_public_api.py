"""Black-box tests of the installed Python package.

Every test here goes through the front door. It uses only names that the
package exports.

References
----------
Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import numpy as np
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


def test_the_event_columns_carry_the_fields_by_name(seed: int) -> None:
    # DEC-060: the bindings return one column for each field, so a reader
    # holds no byte offset, no field width and no field order.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.step(threads=1)
    columns = world.event_log_columns()

    assert set(columns) == {"tick", "tile", "value", "holder", "kind"}
    assert len(columns["tile"]) == world.event_count
    for name in columns:
        assert len(columns[name]) == world.event_count


def test_no_event_column_is_a_floating_point_array(seed: int) -> None:
    # ADR-0002 D1 bans a floating point number in simulated state. A float
    # that enters through this interface is the same defect one layer out.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.step(threads=1)
    for columns in (world.event_log_columns(), world.gather_log_columns()):
        for name, column in columns.items():
            assert np.issubdtype(column.dtype, np.integer), name
    assert world.event_log_columns()["value"].dtype == np.int32


def test_the_columns_agree_with_the_bytes(seed: int) -> None:
    # The columns and the raw bytes are two views of one log. This is the
    # check that fails if they ever stop describing the same thing. The
    # test reads the byte count from the log, and it holds no field offset.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.step(threads=1)
    columns = world.event_log_columns()
    raw = world.event_log_bytes()
    assert world.event_count > 0
    assert len(raw) % world.event_count == 0
    assert len(columns["tick"]) == world.event_count


def test_a_unit_identity_survives_the_round_trip(seed: int) -> None:
    # ADR-0085 D1 and D3: Python holds the whole identity and gives it
    # back, and the engine resolves it.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    unit = world.spawn_soldier(0, 0, 1)
    assert world.soldier_tile(unit) == 0


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
    dead = world.spawn_soldier(0, 0, 1)
    assert world.despawn_soldier(dead)
    living = world.spawn_soldier(0, 0, 1)

    assert living != dead, "the arena must mint a new identity"

    # ADR-0046: the refusal is typed. ADR-0085 D3: it never falls back to
    # the unit that now holds the slot.
    with pytest.raises(cachette.ViewError):
        world.soldier_tile(dead)
    with pytest.raises(cachette.ViewError):
        world.despawn_soldier(dead)
    assert world.soldier_tile(living) == 0


def test_python_cannot_compose_an_identity(seed: int) -> None:
    # ADR-0085 D2: the bindings expose no way to build an identity. A
    # caller that assembles one from an index it chose gets a refusal.
    world = cachette.World(width=8, height=8, seed=seed, faction_count=2)
    world.spawn_soldier(0, 0, 1)
    with pytest.raises(cachette.ViewError):
        world.soldier_tile(0)
    with pytest.raises(cachette.ViewError):
        # A number the engine never gave out. The caller has no way to know
        # which numbers are identities, which is the point.
        world.soldier_tile(2**40 + 7)


def test_a_gather_event_names_a_unit_that_resolves(seed: int) -> None:
    # ADR-0085 D1: the unit column holds the whole identity, so a reader
    # can follow the unit that took the amount.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    units = []
    for q in range(16):
        for r in range(16):
            try:
                units.append(world.spawn_soldier(q, r, 1))
            except cachette.VerbError:
                continue
    assert units, "the world must admit a unit"
    for unit in units:
        world.order_gather(unit, 0)

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
