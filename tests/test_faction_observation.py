"""Tests of the one flat array a faction reads.

The schema declares the layout of that array.

A learner reads one array on every decision. The engine returns that array
for one faction, and it returns one schema that says where every field of it
sits.[^1]

These tests state no offset, no length and no bound of their own. Each one
reads the schema and decodes by arithmetic over it. A test that wrote a
position would be a second declaration site of the thing under test.[^2]

Every test drives the step. The step runs the observation pass that fills the
fog layers, so a test that filled a layer itself would prove that the reader
works and not that anything reaches it.[^3]

The layout replaced a layout whose width followed the world shape and the
faction count, so a policy trained against one shape could not read
another.[^4] One test below builds four world shapes at four faction counts
and asserts that the schema does not move.

Every position of the array is a share, a signed relation or a compressed
magnitude, and each of the three lies between minus one and one. A field the
engine cannot answer is reserved: it reads zero and the schema declares its
bounds as zero and zero.

References
----------
[^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decisions D1, D2 and D3.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
[^2]: Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
[^3]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``
[^4]: Findings register, FND-670. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import numpy as np
import pytest

from cachette import VerbError, World

# The world every test below builds. It is wide enough that a faction seated
# in one part of it has never seen another part.
EXTENT = 96
SEED = 0x00C0FFEE01234567
FACTIONS = 3

# The faction that reads in every test below.
WATCHER = 0

# One unit of the fixed-point scale of this project.
ONE = 65536

# How many steps each test runs before it reads. The seeding seats a faction
# and the step then fills the fog layers, so one step is enough to give the
# watcher sight of its own ground.
STEPS = 3


def a_seeded_world(
    width: int = EXTENT, height: int = EXTENT, factions: int = FACTIONS
) -> World:
    """Build a seeded world and step it, so the fog layers hold something."""
    world = World(width=width, height=height, seed=SEED, faction_count=factions)
    reports = world.seed_world()
    assert any(report["seated"] for report in reports), (
        "the fixture must seat at least one faction"
    )
    for _ in range(STEPS):
        world.step(1)
    return world


def field(world: World, values: np.ndarray, name: str) -> np.ndarray:
    """Return the positions of one field, taken from the schema alone."""
    for row in world.observation_schema()["fields"]:
        if row["name"] == name:
            return values[row["start"] : row["start"] + row["positions"]]
    raise AssertionError(f"the schema declares no field named {name}")


def test_the_schema_covers_the_array_exactly() -> None:
    """The fields fill the array, with no gap and no overlap.

    The count includes every reserved range. A reserved block holds its
    declared positions, so every later block starts where the design puts it
    and a builder that fills one moves nothing.
    """
    world = a_seeded_world()
    schema = world.observation_schema()
    values = world.faction_observation(WATCHER)

    assert values.dtype == np.int64, "every position is a signed eight-byte integer"
    assert len(values) == schema["length"]
    assert schema["version"] >= 1
    assert schema["fields"], "the schema declares at least one field"

    start = 0
    for row in schema["fields"]:
        name = row["name"]
        assert row["start"] == start, f"the field {name} follows the one before it"
        assert row["positions"] > 0
        assert row["low"] <= row["high"]
        assert row["dtype"] == "int64"
        start += row["positions"]
    assert start == schema["length"], "the fields leave no gap"


def test_every_position_sits_inside_the_bound_the_schema_states() -> None:
    """No position of the array leaves the bounds the schema declares.

    Every bound comes from the value kind of the field, and every kind lies
    inside the unit range. A position outside it would be a raw count that
    escaped the normalisation rule.
    """
    world = a_seeded_world()
    values = world.faction_observation(WATCHER)
    for row in world.observation_schema()["fields"]:
        span = values[row["start"] : row["start"] + row["positions"]]
        name = row["name"]
        assert span.min() >= row["low"], f"the field {name} states its own floor"
        assert span.max() <= row["high"], f"the field {name} states its own ceiling"
        assert row["low"] >= -ONE, f"the field {name} declares a bounded floor"
        assert row["high"] <= ONE, f"the field {name} declares a bounded ceiling"


def test_a_reserved_field_reads_zero() -> None:
    """A reserved field is not a zero that states a real quantity of zero.

    The fixture asserts that the schema holds reserved fields, so a layout
    that reserved nothing would fail here rather than pass.
    """
    world = a_seeded_world()
    values = world.faction_observation(WATCHER)
    reserved = 0
    for row in world.observation_schema()["fields"]:
        if (row["low"], row["high"]) != (0, 0):
            continue
        reserved += int(row["positions"])
        span = values[row["start"] : row["start"] + row["positions"]]
        assert not span.any(), (
            f"the reserved field {row['name']} reads zero in every position"
        )
    assert reserved > 0, "the fixture must find a reserved field"


def test_the_width_is_one_number_for_every_world_shape() -> None:
    """The schema does not move between world shapes or faction counts.

    The fixture asserts that the worlds it built really differ, so a fixture
    that built one world four times would fail rather than pass.
    """
    declared = a_seeded_world().observation_schema()
    tile_counts = set()
    for width, height, factions in ((24, 24, 2), (48, 48, 5), (96, 96, 7), (128, 64, 12)):
        world = a_seeded_world(width, height, factions)
        tile_counts.add(width * height)
        schema = world.observation_schema()
        assert schema["length"] == declared["length"], (
            f"the {width} by {height} world at {factions} factions holds the width"
        )
        assert schema["fields"] == declared["fields"], (
            "every field starts at one position in every world"
        )
        assert len(world.faction_observation(WATCHER)) == declared["length"]
    assert len(tile_counts) > 1, "the fixture must build worlds of different sizes"


def test_no_position_of_the_array_names_a_seat() -> None:
    """A policy must not be able to learn a seat number from the array.

    A league seats one policy in one seat for one game and in another seat for
    the next, so a policy that learned a seat number reads another faction's
    quantities under the same weight.[^1]

    References
    ----------
    [^1]: Findings register, FND-647. ``docs/FINDINGS.md``
    """
    world = a_seeded_world()
    names = {str(row["name"]) for row in world.observation_schema()["fields"]}
    assert "faction" not in names
    assert "relation" not in names
    assert "power_held_tiles" in names, "the rivals arrive as order statistics"


def test_a_faction_reads_the_ground_it_holds_and_not_the_fog_of_this_frame() -> None:
    """The held tile count is the whole count, not the count of what it sees.

    A count scoped to the fog of the frame flickers with sight, and a policy
    cannot learn from a quantity that moves when nothing moved.[^1]

    References
    ----------
    [^1]: Findings register, FND-671. ``docs/FINDINGS.md``
    """
    world = a_seeded_world()
    values = world.faction_observation(WATCHER)
    assert world.standing(WATCHER)["held_tiles"] > 0, (
        "the fixture must give the watcher ground"
    )
    assert field(world, values, "held_tiles")[0] > 0, (
        "the array reports the ground the faction holds"
    )
    assert 0 < field(world, values, "held_share_world")[0] <= ONE, (
        "the held share of the world lies inside the unit range"
    )


def test_the_reader_refuses_a_number_that_names_no_faction() -> None:
    """No argument widens the answer, and an unknown faction is refused."""
    world = a_seeded_world()
    with pytest.raises(VerbError):
        world.faction_observation(FACTIONS)


def test_the_length_follows_the_world_and_not_the_population() -> None:
    """The declared length does not move when the population moves."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
    empty = world.observation_schema()["length"]
    assert len(world.faction_observation(WATCHER)) == empty

    world.seed_world()
    for _ in range(20):
        world.step(1)

    assert world.observation_schema()["length"] == empty, (
        "the declared length does not move when the population moves"
    )
    assert len(world.faction_observation(WATCHER)) == empty


def test_the_array_carries_the_progress_of_every_victory_track() -> None:
    """A faction that leads on ground and trails on renown plays differently.

    Without a gap for each track the policy cannot choose a track. The wonder
    track reaches one unit when a finished claim stands, because the work of
    the standing row is the whole requirement.[^1]

    References
    ----------
    [^1]: Findings register, FND-568. ``docs/FINDINGS.md``
    """
    world = a_seeded_world()
    values = world.faction_observation(WATCHER)
    for track in ("domination", "wonder_track", "renown", "ground"):
        for statistic in ("progress", "leader", "gap", "rank"):
            span = field(world, values, f"{track}_{statistic}")
            assert span.shape == (1,), f"{track}_{statistic} holds one position"
    assert field(world, values, "ground_progress")[0] > 0, (
        "the watcher holds ground, so it has made progress on that track"
    )
