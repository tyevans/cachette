"""The deck says when an upgrade collapses, and it says what fell and to what.

A toast is one sentence a watcher reads without looking away from the map. The
demonstration reads it from an event log that the engine writes, so a line
appears only when the engine says something happened.

An upgrade collapse was the one interesting event the deck could not show. The
engine published neither a log nor a counter for it when the deck was built.
The engine now publishes a log that carries the tick, the tile, the holder, the
category, the level and the cause.

The test drives the engine and then reads the deck. It builds no log of its
own, because a test that fed a made-up log would prove that the formatting
works and not that the deck reaches the engine.[^1]

References
----------
[^1]: Testing rules, section 5. ``.agents/rules/testing.md``
"""

from __future__ import annotations

import pytest

from cachette import World
from cachette.demo.toasts import Announcer
from cachette.names import Names

# The world the test runs. It is small enough to step quickly and large enough
# that four factions each found a settlement and build a road.
EXTENT = 128
SEED = 0x1337
FACTIONS = 4

# The ticks the test gives the world to finish an upgrade.
TICKS = 400


@pytest.fixture(name="built")
def built_world() -> tuple[World, int]:
    """Give back a world that holds a finished upgrade, and the tile of one."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    world.seed_world()
    for _ in range(TICKS):
        world.step(4)
        columns = world.log("upgrade_finished")
        if len(columns["tick"]):
            return world, int(columns["tile"][0])
    pytest.fail("the world finished no upgrade, so the test measures nothing")


def test_a_collapse_names_the_nation_the_thing_the_place_and_the_cause(
    built: tuple[World, int],
) -> None:
    """A destroyed upgrade writes a line that says what was lost and to what."""
    world, tile = built
    q, r = tile % world.width, tile // world.width
    assert world.destroy_upgrades([(q, r)]) == 1
    announcer = Announcer(Names(world.seed))
    announcer.after_step(world, 0.0)
    lines = [text for text in announcer.toasts.texts() if "loses a" in text]
    assert lines, (
        f"the deck said nothing about the collapse: {announcer.toasts.texts()}"
    )
    line = lines[0]
    # The cause separates an order from the weather and from an army. A line
    # that named no cause would read the same for all three.
    assert "to an order" in line
    assert "at level" in line
    assert f"near {Names(world.seed).place(q, r)}" in line
    # The line names a nation and not an index.
    assert "Faction " not in line
