"""Weather, and the power of a god to inflict it on a place.

Every test here goes through the published interface. None reads a private
attribute, because a test that reads one pins the implementation rather than
the behaviour.[^1]

References
----------
[^1]: Testing rules, section 6. `.claude/rules/testing.md`
"""

from __future__ import annotations

import pytest

from cachette import World
from cachette._core import VerbError

# A world that holds open water somewhere, so the sea lifts water on its own.
COASTAL = {"width": 128, "height": 128, "seed": 0x0123456789ABCDEF, "faction_count": 2}

# A world that holds no open water at all, so only a god puts water in the air.
INLAND = {"width": 64, "height": 64, "seed": 2, "faction_count": 2}


def test_the_world_makes_weather_without_a_caller() -> None:
    world = World(**COASTAL)
    assert world.weather_totals()["raised"] == 0
    for _ in range(64):
        world.step(4)
    totals = world.weather_totals()
    assert totals["raised"] > 0
    assert totals["ground"] > 0
    assert totals["wet_cells"] > 0


def test_the_water_account_is_exact() -> None:
    world = World(**COASTAL)
    for _ in range(32):
        world.step(4)
        totals = world.weather_totals()
        held = totals["air"] + totals["ground"]
        assert held + totals["evaporated"] == totals["raised"]


def test_a_watcher_reads_the_whole_field_in_one_crossing() -> None:
    world = World(**COASTAL)
    for _ in range(32):
        world.step(4)
    plane = world.weather_ground()
    assert plane.dtype.name == "int64"
    assert len(plane) == world.cells_wide * world.cells_wide
    assert int(plane.sum()) == world.weather_totals()["ground"]


def test_a_read_outside_the_world_refuses() -> None:
    world = World(**COASTAL)
    with pytest.raises(VerbError):
        world.ground_water_at(-1, -1)
    with pytest.raises(VerbError):
        world.air_at(-1, -1)
    with pytest.raises(VerbError):
        world.ground_is_wet(-1, -1)


def a_congregation() -> tuple[World, tuple[int, int]]:
    """Build an inland world in which one faction holds ground."""
    world = World(**INLAND)
    place = (2, 2)
    world.spawn_soldiers([place], 0)
    for _ in range(16):
        world.step(1)
    return world, place


def test_a_god_wets_the_ground_it_strikes() -> None:
    world, place = a_congregation()
    assert world.ground_is_wet(*place) is False
    storm = world.inflict_weather(0, [place], world.weather_strength_ceiling)
    assert storm["cells"] == 1
    assert storm["drops"] > 0
    assert storm["ready_at"] == world.tick + world.weather_cooldown_ticks
    world.step(1)
    assert world.ground_is_wet(*place) is True
    assert world.ground_water_at(*place) >= world.weather_wet_mark


def test_a_god_waits_between_one_storm_and_the_next() -> None:
    world, place = a_congregation()
    world.inflict_weather(0, [place], 1)
    with pytest.raises(VerbError):
        world.inflict_weather(0, [place], 1)


def test_a_god_may_not_strike_ground_its_faction_does_not_hold() -> None:
    world, _ = a_congregation()
    far = (INLAND["width"] - 1, INLAND["height"] - 1)
    with pytest.raises(VerbError):
        world.inflict_weather(0, [far], 1)
    assert world.weather_totals()["raised"] == 0


def test_one_refusal_leaves_the_world_unchanged() -> None:
    world, place = a_congregation()
    far = (INLAND["width"] - 1, INLAND["height"] - 1)
    before = world.state_hash
    with pytest.raises(VerbError):
        world.inflict_weather(0, [place, far], 1)
    assert world.state_hash == before


def test_the_verb_refuses_a_strength_outside_its_range() -> None:
    world, place = a_congregation()
    with pytest.raises(VerbError):
        world.inflict_weather(0, [place], 0)
    with pytest.raises(VerbError):
        world.inflict_weather(0, [place], world.weather_strength_ceiling + 1)


def test_the_verb_refuses_more_places_than_one_call_carries() -> None:
    world, place = a_congregation()
    with pytest.raises(VerbError):
        world.inflict_weather(0, [place] * (world.weather_places_ceiling + 1), 1)
