"""The graded relation between two factions, from the control plane.

A god reads what one faction feels toward another, sets it outright for a
scenario, and moves it through a speaker unit whose type has command reach.
A crossing of the war edge lands in one log the demonstration reads.[^1]

The band numbers count the edges at or below the value, so zero is the war
band. The tests read the edges through the band and hold no copy of them.[^2]

References
----------
[^1]: ADR-0146, a faction relation is one signed integer per ordered pair, and
a pass reads a threshold.
``docs/adrs/draft/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md``

[^2]: Balance register, the relation. ``docs/reference/balance.md``
"""

from __future__ import annotations

import numpy as np
import pytest

import cachette
from cachette import VerbError, World

EXTENT = 16
SEED = 0x0CAC_4E77_0474
A = 0
B = 1

# A value below every edge the register could set. The tests assert the band
# it lands in rather than copying an edge.
FAR_BELOW_EVERY_EDGE = -(1 << 20)

# The rows of the default table: the worker has no command reach and the
# leader has some. The test reads the column to prove it.
WORKER = 0
LEADER = 3


def _world() -> World:
    return World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=2)


def _open_address(world: World) -> tuple[int, int]:
    for q in range(world.width):
        for r in range(world.height):
            if bool(world.tile_report(q, r)["passable"]):
                return (q, r)
    message = "the world admits a unit nowhere"
    raise AssertionError(message)


def test_a_new_world_holds_every_pair_at_peace() -> None:
    world = _world()
    assert world.relation(A, B) == world.relation(B, A)
    assert world.relation_band(A, B) == 2, "two is the peace band"
    assert world.relation_crossed_count == 0


def test_set_relation_writes_one_direction_and_logs_a_declaration() -> None:
    world = _world()
    before = world.relation(B, A)
    world.set_relation(A, B, FAR_BELOW_EVERY_EDGE)
    assert world.relation(A, B) == FAR_BELOW_EVERY_EDGE
    assert world.relation_band(A, B) == 0
    assert world.relation(B, A) == before, "the other direction is a separate entry"
    columns = world.relation_log_columns()
    assert set(columns) == {
        "tick",
        "from_faction",
        "to_faction",
        "band_before",
        "band_after",
    }
    assert columns["tick"].dtype == np.uint64
    assert columns["from_faction"].dtype == np.uint16
    assert columns["to_faction"].dtype == np.uint16
    assert columns["band_before"].dtype == np.uint8
    assert columns["band_after"].dtype == np.uint8
    assert world.relation_crossed_count == 1
    assert (int(columns["from_faction"][0]), int(columns["to_faction"][0])) == (A, B)
    assert int(columns["band_after"][0]) < int(columns["band_before"][0])


def test_the_readers_refuse_a_faction_the_world_does_not_hold() -> None:
    world = _world()
    with pytest.raises(VerbError):
        world.relation(A, 7)
    with pytest.raises(VerbError):
        world.relation_band(7, A)
    with pytest.raises(VerbError):
        world.set_relation(A, A, 1)
    with pytest.raises(VerbError):
        world.relation(A, A)


def test_move_relation_reads_the_command_reach_of_the_speaker() -> None:
    world = _world()
    table = world.unit_type_table()
    assert int(table["command_reach"][WORKER]) == 0
    assert int(table["command_reach"][LEADER]) > 0
    place = _open_address(world)
    worker, leader = (int(unit) for unit in world.spawn_soldiers([place, place], A))
    world.set_unit_types([worker], WORKER)
    world.set_unit_types([leader], LEADER)
    start = world.relation(A, B)

    with pytest.raises(VerbError):
        world.move_relation(worker, B, -1)
    assert world.relation(A, B) == start, "a refused move changes nothing"
    with pytest.raises(VerbError):
        world.move_relation(leader, A, -1)
    with pytest.raises(VerbError):
        world.move_relation(leader, 9, -1)
    with pytest.raises(VerbError):
        world.move_relation(leader, B, 1 << 20)

    assert world.move_relation(leader, B, -1) == start - 1
    assert world.relation(A, B) == start - 1
    assert world.move_relation(leader, B, 1) == start
    with pytest.raises(cachette.ViewError):
        world.move_relation(1 << 40, B, -1)


def test_the_log_covers_the_last_step_alone() -> None:
    world = _world()
    world.set_relation(A, B, FAR_BELOW_EVERY_EDGE)
    assert world.relation_crossed_count == 1
    world.step(1)
    assert world.relation_crossed_count == 0
    assert len(world.relation_log_columns()["tick"]) == 0
