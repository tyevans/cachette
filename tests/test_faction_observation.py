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

References
----------
[^1]: ADR-0154, the observation and the action of a faction are
    schema-declared bounded tables the engine owns, decisions D1, D2 and D3.
    ``docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md``
[^2]: Recurring Defect Shapes, shape 1. ``.agents/rules/recurring-defects.md``
[^3]: Testing Rules, drive the real caller. ``.agents/rules/testing.md``
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

# How many steps each test runs before it reads. The seeding seats a faction
# and the step then fills the fog layers, so one step is enough to give the
# watcher sight of its own ground.
STEPS = 3


def a_seeded_world() -> World:
    """Build a seeded world and step it, so the fog layers hold something."""
    world = World(width=EXTENT, height=EXTENT, seed=SEED, faction_count=FACTIONS)
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
    """The fields fill the array, with no gap and no overlap."""
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
    """No position of the array leaves the bounds the schema declares."""
    world = a_seeded_world()
    values = world.faction_observation(WATCHER)
    for row in world.observation_schema()["fields"]:
        span = values[row["start"] : row["start"] + row["positions"]]
        name = row["name"]
        assert span.min() >= row["low"], f"the field {name} states its own floor"
        assert span.max() <= row["high"], f"the field {name} states its own ceiling"


def test_a_cell_the_faction_has_never_seen_reads_as_nothing() -> None:
    """A cell outside everything the faction walked states nothing.

    The fixture asserts that the unseen cell covers tiles, and that a cell the
    watcher does see reports height. The map fields are therefore able to hold
    a value here, so a reader that leaked the truth would write one into the
    unseen cell and fail every assertion below.
    """
    world = a_seeded_world()
    values = world.faction_observation(WATCHER)
    seen_ever = field(world, values, "cell_seen_ever")
    unseen = int(np.argmin(seen_ever))
    assert seen_ever[unseen] == 0, "the fixture must leave one cell unseen"
    assert field(world, values, "cell_tiles")[unseen] > 0, (
        "the fixture must leave a cell that covers real tiles unseen"
    )
    assert field(world, values, "cell_height_total").max() > 0, (
        "the fixture must give the watcher a cell that reports height"
    )

    for name in (
        "cell_seen_now",
        "cell_seen_ever",
        "cell_open_tiles",
        "cell_own_units",
        "cell_other_units",
        "cell_own_held_tiles",
        "cell_other_held_tiles",
        "cell_value_total",
        "cell_height_total",
        "cell_food_total",
    ):
        assert field(world, values, name)[unseen] == 0, (
            f"the field {name} states nothing about a cell the faction never saw"
        )

    assert field(world, values, "cell_tiles")[unseen] > 0, (
        "every cell states how many tiles of the world it covers"
    )


def test_a_faction_never_forgets_a_place_it_saw() -> None:
    """The array changes over a run in the way the fog says it should."""
    world = a_seeded_world()
    before = field(world, world.faction_observation(WATCHER), "cell_seen_ever")
    assert before.max() > 0, "the fixture must give the watcher sight of its own ground"

    for _ in range(20):
        world.step(1)
    after = field(world, world.faction_observation(WATCHER), "cell_seen_ever")

    assert np.all(after >= before), "a faction never forgets a place it saw"


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


def test_the_reader_refuses_a_number_that_names_no_faction() -> None:
    """No argument widens the answer, and an unknown faction is refused."""
    world = a_seeded_world()
    with pytest.raises(VerbError):
        world.faction_observation(FACTIONS)


def test_the_array_carries_the_claim_the_wonder_reader_compares() -> None:
    """The standing reports the work, and the wonder reader compares the claim.

    A learner reads this array, so the array carries both quantities.[^1]

    References
    ----------
    [^1]: Findings register, FND-568. ``docs/FINDINGS.md``
    """
    world = a_seeded_world()
    names = [row["name"] for row in world.observation_schema()["fields"]]
    assert "wonder_claim" in names, "the array carries the claim the reader compares"
    assert "wonder_progress" in names, "the array carries the work toward a claim"
