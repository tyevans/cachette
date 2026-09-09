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

The schema states the structure of the array and not only the position of each
field. A start and a width do not say how many cells a block holds, how many
channels a cell holds, or which of the two axes runs first. The tests below
read those entries and check that they account for every position of the
spatial part.[^5]

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
[^5]: ADR-0195, the observation of a faction is a fixed-width scale-free
    table, decisions D4 and D8.
    ``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``
"""

from __future__ import annotations

from pathlib import Path

import numpy as np
import pytest

from cachette import VerbError, World

# The binding that carries the schema across the boundary. The test that reads
# it asserts that it states no number of its own.
BINDING = (
    Path(__file__).resolve().parents[1]
    / "crates"
    / "cachette-py"
    / "src"
    / "world"
    / "faction_view.rs"
)

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


def rows_of_space(world: World, space: str) -> list[dict]:
    """Return every schema row the engine marks with one space."""
    return [
        row for row in world.observation_schema()["fields"] if row["space"] == space
    ]


def ring_row(world: World) -> dict:
    """Return the one row the engine marks as ring space."""
    rows = rows_of_space(world, "ring")
    assert len(rows) == 1, "the engine publishes one ring stack"
    return rows[0]


def channel_column(row: dict, channel: str, cells: int, order: str) -> np.ndarray:
    """Return the position of one channel of a row, at every place.

    The channel order says which axis runs first, so this reads the order
    rather than assuming one.
    """
    index = list(row["channels"]).index(channel)
    count = len(row["channels"])
    start = int(row["start"])
    if order == "cell_major":
        return np.arange(start + index, start + count * cells, count, dtype=np.int64)
    assert order == "channel_major", f"the schema states the order {order!r}"
    return np.arange(start + index * cells, start + (index + 1) * cells, dtype=np.int64)


def test_the_ring_geometry_accounts_for_every_position_of_the_block() -> None:
    """The published geometry adds up, cell by cell and channel by channel.

    A geometry that did not add up would let a reader gather a plausible block
    that is not the one the engine wrote.
    """
    world = a_seeded_world()
    schema = world.observation_schema()
    counts = list(schema["ring_cells"])
    assert counts, "the schema states the cells of each ring"
    assert counts[0] == 1, "the centre has no direction, so ring 0 holds one cell"
    row = ring_row(world)
    cells = sum(counts)
    channels = len(row["channels"])
    assert channels > 0, "a ring field names its channels"
    assert cells * channels == row["positions"], (
        f"{cells} cells of {channels} channels do not fill {row['positions']} positions"
    )


def test_every_token_set_is_one_field_of_a_whole_number_of_tokens() -> None:
    """One field states one shape, so each token set gets a field of its own."""
    world = a_seeded_world()
    rows = rows_of_space(world, "token")
    assert len(rows) >= 2, "one field cannot state the shape of every set"
    shapes = set()
    for row in rows:
        channels = len(row["channels"])
        assert channels > 0, f"{row['name']} names its channels"
        tokens, remainder = divmod(int(row["positions"]), channels)
        assert remainder == 0, (
            f"{row['name']} holds {row['positions']} positions over {channels} channels"
        )
        assert tokens >= 1
        shapes.add((tokens, channels))
    assert len(shapes) > 1, (
        "the sets hold different shapes, which is why one field cannot state them all"
    )


def test_a_channel_list_holds_no_repeated_name() -> None:
    """A reader names a channel, so two channels of one field cannot share one."""
    world = a_seeded_world()
    for row in world.observation_schema()["fields"]:
        names = list(row["channels"])
        assert len(names) == len(set(names)), f"{row['name']} repeats a channel name"
        if row["space"] is None:
            assert not names, f"{row['name']} lays out in no space and names channels"


def test_the_schema_names_the_channel_that_gates_a_cell() -> None:
    """An absent cell must not read as a cell that holds zero.

    The gate channel states the difference.[^1]

    References
    ----------
    [^1]: ADR-0195, the observation of a faction is a fixed-width scale-free
        table, decision D8.
        ``docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md``
    """
    world = a_seeded_world()
    schema = world.observation_schema()
    gate = schema["spatial_gate"]
    row = ring_row(world)
    assert gate in row["channels"], (
        f"the schema gates the spatial part with {gate!r} and the ring stack "
        f"names {list(row['channels'])}"
    )
    values = world.faction_observation(WATCHER)
    cells = sum(schema["ring_cells"])
    column = values[channel_column(row, gate, cells, schema["channel_order"])]
    assert column.max() > 0, "a cell of the world holds a value"
    assert column.min() == 0, (
        "the frame reaches past the edge of this world, so a cell of it lies "
        "outside the world and holds nothing"
    )


def test_the_channel_order_the_schema_states_is_the_order_the_engine_wrote() -> None:
    """Read a channel the engine never fills, and find zero at every cell.

    The ring stack holds three channels that no source in the engine fills.
    Each one reads zero in every cell of every world. A reader that took the
    wrong axis for the fastest one gathers a different set of positions, and
    those positions hold the quantities of the cells instead. The wrong order
    therefore fails here and looks plausible everywhere else.
    """
    world = a_seeded_world()
    schema = world.observation_schema()
    row = ring_row(world)
    values = world.faction_observation(WATCHER)
    cells = sum(schema["ring_cells"])
    order = schema["channel_order"]
    empty = ("memory_age", "own_strength", "rival_strength")
    for channel in empty:
        column = values[channel_column(row, channel, cells, order)]
        assert not column.any(), (
            f"the engine fills no source for {channel!r}, so every cell of it "
            f"reads zero under the order {order!r}"
        )
    gate = values[channel_column(row, schema["spatial_gate"], cells, order)]
    assert gate.any(), "the gate channel is not one of the empty ones"


def schema_binding_body() -> str:
    """Return the body of the function that publishes the observation schema."""
    text = BINDING.read_text()
    opening = "fn observation_schema<'py>"
    start = text.index(opening)
    end = text.index("\n    }\n", start)
    return text[start:end]


def test_the_published_schema_states_no_number_of_its_own() -> None:
    """The binding translates the layout. It does not restate it.

    A hand-written channel count or cell count in the binding is a second
    declaration of a number the engine owns, and nothing would fail when the
    two disagreed.[^1] The binding therefore holds no digit at all: every
    number it publishes comes from the schema the engine built.

    References
    ----------
    [^1]: Recurring Defect Shapes, shape 1.
        ``.agents/rules/recurring-defects.md``
    """
    body = schema_binding_body()
    digits = sorted({character for character in body if character.isdigit()})
    assert digits == [], (
        f"the binding of the observation schema states the digits {digits}. "
        f"Read the number from the schema instead."
    )


def test_the_channel_names_come_from_the_engine_and_not_from_the_binding() -> None:
    """No channel name of the schema appears in the binding.

    The engine names its channels, and the binding carries the list across the
    boundary. A name written into the binding would be a second declaration of
    the list.
    """
    body = schema_binding_body()
    world = a_seeded_world()
    for row in world.observation_schema()["fields"]:
        for channel in row["channels"]:
            assert channel not in body, (
                f"the binding names the channel {channel!r}, which the engine "
                f"already names"
            )


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
    shapes = ((24, 24, 2), (48, 48, 5), (96, 96, 7), (128, 64, 12))
    for width, height, factions in shapes:
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
