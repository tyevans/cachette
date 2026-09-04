"""A god asks whose ground its people stand on.

The tests go through the installed package. They drive the step, because the
step is what must derive the relation.

Every test reads the ground under the guest before it reads the relation. A
fixture that failed to put the guest on foreign ground would leave an
assertion that passes for the wrong reason.

References
----------
Testing rules, section 2a. ``.claude/rules/testing.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette

EXTENT = 96
"""The side of the world every test here builds."""

HOST = 0
"""The faction that holds the ground."""

GUEST = 1
"""The faction whose unit visits."""

NOBODY = 65535
"""The holder value of a tile that no faction holds."""

CORNER = (12, 43)
"""The corner of the garrison. The ground around it admits units."""

HOST_UNITS_ON_THE_VISITED_TILE = 3
"""How many host units stand with the guest, so that the tile does not turn."""


def _garrison(world: cachette.World) -> None:
    """Fill a square with host units and run the world until it holds."""
    addresses = [
        (CORNER[0] + column, CORNER[1] + row) for row in range(8) for column in range(8)
    ]
    world.spawn_soldiers(addresses, HOST)
    for _ in range(6):
        world.step(threads=1)


def _a_tile_the_host_holds(world: cachette.World) -> tuple[int, int]:
    """Return an address the host holds, read from the holder column."""
    holders = world.tile_holders()
    held = np.flatnonzero(holders == HOST)
    assert held.size > 0, "the fixture must leave the host holding ground"
    tile = int(held[held.size // 2])
    return tile % EXTENT, tile // EXTENT


def _a_world_with_a_guest() -> cachette.World:
    """Return a world in which a guest stands on the host's ground."""
    world = cachette.World(width=EXTENT, height=EXTENT, seed=7, faction_count=3)
    _garrison(world)
    address = _a_tile_the_host_holds(world)
    world.spawn_soldiers([address] * HOST_UNITS_ON_THE_VISITED_TILE, HOST)
    world.spawn_soldiers([address], GUEST)
    world.step(threads=1)
    return world


def test_the_holder_column_covers_the_world_and_names_nobody() -> None:
    world = cachette.World(width=8, height=8, seed=7, faction_count=2)
    holders = world.tile_holders()
    assert holders.dtype == np.uint16
    assert holders.shape == (64,)
    assert np.all(holders == NOBODY), "a new world holds no ground"


def test_the_holder_column_names_the_faction_that_holds_a_tile() -> None:
    world = cachette.World(width=EXTENT, height=EXTENT, seed=7, faction_count=3)
    _garrison(world)
    holders = world.tile_holders()
    assert np.any(holders == HOST), "the host must hold ground after the run"
    assert np.any(holders == NOBODY), "the host must not hold the whole world"

    # The column and the single-address report are two statements of one
    # fact, so they must agree.
    q, r = _a_tile_the_host_holds(world)
    assert world.tile_report(q, r)["holder"] == HOST
    assert holders[r * EXTENT + q] == HOST


def test_the_relation_reports_a_guest_on_foreign_ground() -> None:
    world = _a_world_with_a_guest()
    masks = world.presence_masks()
    assert masks.dtype == np.uint64
    assert masks.shape == (3,)
    assert bool(masks[HOST] & np.uint64(1 << GUEST)), "the guest is present"
    assert world.stands_in_territory(GUEST, HOST) is True


def test_the_relation_is_directed_and_holds_no_diagonal() -> None:
    world = _a_world_with_a_guest()
    assert world.stands_in_territory(HOST, GUEST) is False
    assert world.stands_in_territory(HOST, HOST) is False
    assert world.stands_in_territory(GUEST, GUEST) is False


def test_a_unit_on_its_own_ground_sets_no_bit() -> None:
    world = cachette.World(width=EXTENT, height=EXTENT, seed=7, faction_count=3)
    _garrison(world)
    assert world.soldier_count > 0, "the fixture must hold units"
    assert np.any(world.tile_holders() == HOST), "the host must hold ground"
    assert not np.any(world.presence_masks()), "every unit stands at home"


def test_the_answer_does_not_change_with_the_thread_count() -> None:
    answers = []
    for threads in (1, 2, 12):
        world = cachette.World(width=EXTENT, height=EXTENT, seed=7, faction_count=3)
        addresses = [
            (CORNER[0] + column, CORNER[1] + row)
            for row in range(8)
            for column in range(8)
        ]
        world.spawn_soldiers(addresses, HOST)
        for _ in range(6):
            world.step(threads=threads)
        address = _a_tile_the_host_holds(world)
        world.spawn_soldiers([address] * HOST_UNITS_ON_THE_VISITED_TILE, HOST)
        world.spawn_soldiers([address], GUEST)
        world.step(threads=threads)
        masks = world.presence_masks()
        assert masks[HOST] != 0, "the fixture must set a bit, or this proves nothing"
        answers.append(masks.tobytes())
    assert answers[0] == answers[1] == answers[2]


def test_a_read_after_a_spawn_is_refused() -> None:
    world = _a_world_with_a_guest()
    address = _a_tile_the_host_holds(world)
    world.spawn_soldiers([address], GUEST)
    with pytest.raises(cachette.ViewError):
        world.presence_masks()
    with pytest.raises(cachette.ViewError):
        world.stands_in_territory(GUEST, HOST)
    world.step(threads=1)
    assert world.presence_masks().shape == (3,)


def test_a_faction_the_world_does_not_hold_raises_and_is_named() -> None:
    world = cachette.World(width=8, height=8, seed=7, faction_count=2)
    with pytest.raises(cachette.ViewError) as refused:
        world.stands_in_territory(9, 0)
    assert "9" in str(refused.value), "the error names the faction"
    with pytest.raises(cachette.ViewError) as refused:
        world.stands_in_territory(0, 9)
    assert "9" in str(refused.value), "the error names the faction"
