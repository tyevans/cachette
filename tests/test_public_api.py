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
