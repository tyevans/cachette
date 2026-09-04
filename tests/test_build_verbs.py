"""Black-box tests of the build verbs and the return field.

Every test here starts at the Python boundary. The core held these verbs
before any binding called them, and their own Rust tests passed the whole
time.[^1] A test that built the mechanism again would prove the same thing
again. Each test below drives the installed package.

References
----------
[^1]: Findings register, FND-360. ``docs/FINDINGS.md``

Testing policy. ``docs/TESTING.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette

# The upgrade kinds the engine holds, as `order_build` takes them.
ROAD = 0
TERRACE = 1

# A build takes several steps, and no test here states how many. Each loop
# below stops when the tile reports the build finished, and fails when this
# many steps pass without it.
STEP_CEILING = 64


def _open_address(world: cachette.World) -> tuple[int, int]:
    """Return an address of ground that admits a unit."""
    for q in range(world.width):
        for r in range(world.height):
            if world.tile_report(q, r)["passable"]:
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def _build_until_complete(
    world: cachette.World, address: tuple[int, int], threads: int = 2
) -> int:
    """Step until the tile reports a finished upgrade, and count the steps."""
    for step in range(1, STEP_CEILING + 1):
        world.step(threads=threads)
        if world.tile_report(*address)["upgrade_complete"]:
            return step
    message = f"the build at {address} did not finish in {STEP_CEILING} steps"
    raise AssertionError(message)


def test_a_soldier_told_to_build_marks_the_ground(seed: int) -> None:
    # The whole path, from the boundary: order, step, and read the tile.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    before = world.tile_report(*address)
    assert before["upgrade"] is None
    assert before["upgrade_progress"] == 0
    assert before["upgrade_complete"] is False

    world.order_build(units, ROAD)
    assert world.build_order(int(units[0])) == ROAD

    world.step(threads=2)
    part = world.tile_report(*address)
    assert part["upgrade"] == ROAD
    assert part["upgrade_progress"] > 0

    _build_until_complete(world, address)
    after = world.tile_report(*address)
    assert after["upgrade"] == ROAD
    assert after["upgrade_complete"] is True
    # A road changes what the tile does, and that is what a watcher sees.
    assert after["capacity"] > before["capacity"]


def test_a_terrace_is_a_different_upgrade_from_a_road(seed: int) -> None:
    # The kind is data and it reaches the tile. A verb that ignored the kind
    # would pass the test above.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    world.order_build(units, TERRACE)
    assert world.build_order(int(units[0])) == TERRACE
    world.step(threads=2)
    assert world.tile_report(*address)["upgrade"] == TERRACE


def test_an_upgrade_kind_the_engine_does_not_hold_is_refused(seed: int) -> None:
    # ADR-0046: the engine never raises a bare runtime error.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    with pytest.raises(cachette.VerbError):
        world.order_build(units, 2)
    assert world.build_order(int(units[0])) is None


def test_a_build_order_that_names_a_dead_identity_writes_nothing(
    seed: int,
) -> None:
    # ADR-0085 D3 and the all-or-nothing rule. One stale identity in the set
    # must leave every other unit in the set as it was.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address, address], faction=0)
    live, dead = int(units[0]), int(units[1])
    world.despawn_soldiers([dead])

    with pytest.raises(cachette.ViewError):
        world.order_build(units, ROAD)

    # The live unit took no order, and the ground is unchanged after a step.
    assert world.build_order(live) is None
    world.step(threads=2)
    assert world.tile_report(*address)["upgrade"] is None


def test_the_build_order_of_a_dead_identity_is_refused(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    world.despawn_soldiers(units)
    with pytest.raises(cachette.ViewError):
        world.build_order(int(units[0]))


def test_stopping_a_build_keeps_the_work_already_done(seed: int) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    world.order_build(units, ROAD)
    world.step(threads=2)
    banked = world.tile_report(*address)["upgrade_progress"]
    assert banked > 0

    world.stop_build(units)
    assert world.build_order(int(units[0])) is None
    for _ in range(4):
        world.step(threads=2)
    stopped = world.tile_report(*address)
    assert stopped["upgrade_progress"] == banked
    assert stopped["upgrade_complete"] is False

    # The same soldier continues rather than restarts.
    world.order_build(units, ROAD)
    world.step(threads=2)
    assert world.tile_report(*address)["upgrade_progress"] > banked


def test_stopping_a_build_that_names_a_dead_identity_writes_nothing(
    seed: int,
) -> None:
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address, address], faction=0)
    world.order_build(units, ROAD)
    live, dead = int(units[0]), int(units[1])
    world.despawn_soldiers([dead])

    with pytest.raises(cachette.ViewError):
        world.stop_build(units)
    assert world.build_order(live) == ROAD


def test_destroying_an_upgrade_returns_the_tile_to_the_generated_world(
    seed: int,
) -> None:
    # ADR-0090 D4: removing the entry is the whole of the return.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    generated = world.tile_report(*address)
    units = world.spawn_soldiers([address], faction=0)
    world.order_build(units, ROAD)
    _build_until_complete(world, address)
    assert world.tile_report(*address)["capacity"] != generated["capacity"]

    assert world.destroy_upgrades([address]) == 1
    returned = world.tile_report(*address)
    assert returned["upgrade"] is None
    assert returned["upgrade_progress"] == 0
    assert returned["upgrade_complete"] is False
    assert returned["capacity"] == generated["capacity"]

    # An address that carries no upgrade is not a refusal, and it counts none.
    assert world.destroy_upgrades([address]) == 0


def test_destroying_an_upgrade_leaves_the_build_order_standing(
    seed: int,
) -> None:
    # The removal takes the mark off the ground. It gives no order.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    world.order_build(units, ROAD)
    world.step(threads=2)
    assert world.destroy_upgrades([address]) == 1
    assert world.build_order(int(units[0])) == ROAD
    world.step(threads=2)
    assert world.tile_report(*address)["upgrade"] == ROAD


def test_destroying_at_an_address_outside_the_world_removes_nothing(
    seed: int,
) -> None:
    # The all-or-nothing rule, on the address side.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address], faction=0)
    world.order_build(units, ROAD)
    _build_until_complete(world, address)

    outside = (world.width + 4, world.height + 4)
    with pytest.raises(cachette.ViewError):
        world.destroy_upgrades([address, outside])
    assert world.tile_report(*address)["upgrade"] == ROAD


def test_a_build_gives_one_answer_at_every_thread_count(seed: int) -> None:
    # ADR-0001 D4, through the verb this work bound.
    hashes = []
    progress = []
    for threads in (1, 2, 12):
        world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
        address = _open_address(world)
        units = world.spawn_soldiers([address] * 8, faction=0)
        world.order_build(units, ROAD)
        for _ in range(4):
            world.step(threads=threads)
        hashes.append(world.state_hash())
        progress.append(world.tile_report(*address)["upgrade_progress"])
    assert len(set(hashes)) == 1
    assert len(set(progress)) == 1


def test_the_direction_offsets_are_the_six_neighbours() -> None:
    offsets = cachette.World.direction_offsets()
    assert len(offsets) == len(set(offsets))
    assert all(offset != (0, 0) for offset in offsets)
    # Every offset has its opposite, so the list covers a whole ring.
    assert all((-q, -r) in offsets for q, r in offsets)


def _walk_home(world: cachette.World, faction: int, start: tuple[int, int]) -> int:
    """Follow the return field from an address, and count the steps."""
    offsets = cachette.World.direction_offsets()
    q, r = start
    for step in range(world.width + world.height):
        direction = world.return_direction(faction, q, r)
        if direction is None:
            return step
        across, down = offsets[direction]
        q, r = q + across, r + down
    message = "the walk did not reach the site"
    raise AssertionError(message)


def test_the_return_field_leads_a_faction_to_its_own_site(seed: int) -> None:
    # ADR-0110 D1. The field answers for a block of ground, so the walk ends
    # at the block that holds the site rather than at the site itself.
    world = cachette.World(width=128, height=128, seed=seed, faction_count=2)
    world.found_settlements([(8, 8)], faction=0)
    world.step(threads=2)

    far = (100, 100)
    assert world.return_direction(0, *far) is not None
    steps = _walk_home(world, 0, far)
    assert steps > 0
    # A faction with no site of its own gets no direction anywhere.
    assert world.return_direction(1, *far) is None


def test_the_return_direction_depends_on_the_faction(seed: int) -> None:
    # Test what the value depends on, not only that it repeats. Two factions
    # with sites at opposite corners must be sent opposite ways.
    world = cachette.World(width=128, height=128, seed=seed, faction_count=2)
    world.found_settlements([(8, 8)], faction=0)
    world.found_settlements([(120, 120)], faction=1)
    world.step(threads=2)

    middle = (64, 64)
    first = world.return_direction(0, *middle)
    second = world.return_direction(1, *middle)
    assert first is not None
    assert second is not None
    assert first != second


def test_a_return_direction_outside_the_world_is_refused(seed: int) -> None:
    world = cachette.World(width=32, height=32, seed=seed, faction_count=2)
    with pytest.raises(cachette.ViewError):
        world.return_direction(0, 99, 99)
    with pytest.raises(cachette.ViewError):
        world.return_direction(7, 0, 0)


def test_the_verbs_take_the_column_the_engine_gave(seed: int) -> None:
    # ADR-0040 D1. A caller passes the identities back as one column, and
    # never loops over them.
    world = cachette.World(width=16, height=16, seed=seed, faction_count=2)
    address = _open_address(world)
    units = world.spawn_soldiers([address] * 4, faction=0)
    assert units.dtype == np.uint64
    world.order_build(units, ROAD)
    world.stop_build(units)
    assert world.build_order(int(units[3])) is None
