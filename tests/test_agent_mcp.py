"""Tests of the agent server, driven by a real protocol client.

Every test here starts the server the way an agent starts it: as a separate
process that speaks the Model Context Protocol over standard input and
output. A test that built the server object and called its function would
prove that the function works. It would not prove that a client can reach
it.[^1]

One test prints a transcript. Run it with ``uv run pytest -s -k transcript``
to read what a client and the server say to each other.

References
----------
[^1]: Testing Rules, section 5. ``.claude/rules/testing.md``
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D4.
    ``docs/adrs/REGISTRY.md``
[^3]: Testing Rules, section 4. ``.claude/rules/testing.md``
[^4]: Testing Rules, section 2a. ``.claude/rules/testing.md``
"""

from __future__ import annotations

import itertools
import json
import sys
from collections.abc import AsyncIterator, Awaitable, Callable
from contextlib import asynccontextmanager
from typing import Any, TypeVar

import anyio
import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

T = TypeVar("T")

SERVER = StdioServerParameters(command=sys.executable, args=["-m", "cachette.agent"])

# The world the gather tests build, and the tiles they use.
#
# The seed is part of the fixture, not a detail. The terrain is generated
# from it, and a resource sits on the ground that carries it, so the seed
# decides whether these four tiles hold anything to gather.
#
# Measured at 16 by 16 with the engine's own deposit read. Seed 1 holds food
# at (0, 0), (1, 0) and (1, 1), and wood at all four. Seed 7 holds no food at
# any of them, only stone at two. Seed 42 admits no unit at any of them. The
# first version of this test used seed 7 and asked for food, and it failed for
# that reason.
#
# **The control plane cannot check this.** No read tells Python where a
# resource is, so the seed was chosen against the engine's read from Rust and
# recorded here. That gap is the finding, not an accident of this test.
GATHER_SEED = 1
GATHER_KIND = 0

# The count is the assertion's, not the world's. Four units prove that a
# gather event carries a resolvable identity.
#
# They cross in one call. DEC-063 made the spawn verb set-valued, so nothing
# here shows a client repeating a verb over the mass tier.
GATHER_ADDRESSES = ((0, 0), (1, 0), (0, 1), (1, 1))


@asynccontextmanager
async def _client() -> AsyncIterator[ClientSession]:
    """Start the server as a subprocess and give an initialised session."""
    async with stdio_client(SERVER) as (read, write):
        async with ClientSession(read, write) as session:
            await session.initialize()
            yield session


def _drive(body: Callable[[ClientSession], Awaitable[T]]) -> T:
    """Run one client conversation and return what the body returned."""

    async def run() -> T:
        async with _client() as session:
            return await body(session)

    return anyio.run(run)


async def _call(
    session: ClientSession, name: str, **arguments: object
) -> dict[str, Any]:
    """Call one tool and return its structured result."""
    result = await session.call_tool(name, dict(arguments))
    is_error = getattr(result, "is_error", False)
    content = getattr(result, "content", None)
    assert not is_error, f"{name} failed: {content}"
    structured = getattr(result, "structured_content", None)
    assert isinstance(structured, dict), f"{name} returned no structured result"
    return structured


def test_a_client_reaches_every_tool() -> None:
    async def body(session: ClientSession) -> list[str]:
        listed = await session.list_tools()
        return sorted(tool.name for tool in listed.tools)

    assert _drive(body) == [
        "build_world",
        "check_invariants",
        "despawn_units",
        "event_log",
        "found_group",
        "founding_survey",
        "gather_events",
        "order_gather",
        "region_summary",
        "site_economy",
        "spawn_units",
        "step_world",
        "tile_changes",
        "tile_report",
        "unit_choice",
        "unit_tile",
        "window_census",
        "world_report",
    ]


def test_a_client_builds_and_steps_a_world() -> None:
    async def body(session: ClientSession) -> tuple[dict[str, Any], dict[str, Any]]:
        built = await _call(session, "build_world", width=8, height=8, seed=7)
        stepped = await _call(session, "step_world", world=built["world"], ticks=3)
        return built, stepped

    built, stepped = _drive(body)
    assert built["tick"] == 0
    assert built["tile_count"] == 64
    assert stepped["tick"] == 3
    assert stepped["state_hash"] != built["state_hash"]
    assert len(stepped["state_hash"]) == 16


def test_the_invariant_check_reaches_the_engine() -> None:
    async def body(session: ClientSession) -> dict[str, Any]:
        built = await _call(session, "build_world", width=8, height=8, seed=7)
        await _call(session, "step_world", world=built["world"], ticks=2)
        return await _call(session, "check_invariants", world=built["world"])

    report = _drive(body)
    assert report["tick"] == 2
    assert report["holds"] is True


def test_the_event_log_reports_its_own_size() -> None:
    async def body(session: ClientSession) -> dict[str, Any]:
        built = await _call(session, "build_world", width=8, height=8, seed=7)
        await _call(session, "step_world", world=built["world"], ticks=1)
        return await _call(session, "event_log", world=built["world"])

    log = _drive(body)
    assert log["byte_count"] > 0
    assert log["bytes_returned"] <= log["byte_count"]
    assert len(log["digest_sha256"]) == 64


def test_an_unknown_world_is_a_tool_error() -> None:
    async def body(session: ClientSession) -> bool:
        result = await session.call_tool("world_report", {"world": "world-99"})
        return bool(getattr(result, "is_error", False))

    assert _drive(body)


def test_the_same_settings_give_the_same_hash_through_the_server() -> None:
    # ADR-0001 promises one answer for one binary. The promise must survive
    # this layer, or the layer is not usable to check determinism.[^2]
    async def body(session: ClientSession) -> tuple[str, str]:
        hashes: list[str] = []
        for _ in range(2):
            built = await _call(session, "build_world", width=16, height=16, seed=42)
            stepped = await _call(session, "step_world", world=built["world"], ticks=5)
            hashes.append(stepped["state_hash"])
        return hashes[0], hashes[1]

    first, second = _drive(body)
    assert first == second


@pytest.mark.parametrize("threads", [2, 12])
def test_the_thread_count_does_not_change_the_hash(threads: int) -> None:
    # ADR-0001 D4 asks for the same event log and the same state at any
    # thread count. The check runs through the server, not around it.[^2]
    single = _drive(lambda session: _one_run(session, threads=1))
    assert _drive(lambda session: _one_run(session, threads=threads)) == single


async def _one_run(session: ClientSession, threads: int) -> str:
    """Build one world, step it, and return the state hash."""
    built = await _call(session, "build_world", width=16, height=16, seed=42)
    stepped = await _call(
        session, "step_world", world=built["world"], ticks=5, threads=threads
    )
    hash_value = stepped["state_hash"]
    assert isinstance(hash_value, str)
    return hash_value


def test_transcript() -> None:
    """Print a whole conversation, so that a reader can see the protocol."""

    async def body(session: ClientSession) -> None:
        print(f"\nprotocol version: {session.protocol_version}")
        server_info = session.server_info
        assert server_info is not None
        print(f"server: {server_info.name} {server_info.version}")
        listed = await session.list_tools()
        for tool in listed.tools:
            print(f"tool: {tool.name} -- {tool.title}")
        calls: list[tuple[str, dict[str, Any]]] = [
            ("build_world", {"width": 16, "height": 16, "seed": 42}),
            ("step_world", {"world": "world-1", "ticks": 5, "threads": 1}),
            ("check_invariants", {"world": "world-1"}),
            ("event_log", {"world": "world-1", "max_bytes": 48}),
        ]
        for name, arguments in calls:
            print(f"\n-> call {name} {json.dumps(arguments)}")
            print(f"<- {json.dumps(await _call(session, name, **arguments))}")

    _drive(body)


def test_a_client_reads_which_tile_changed() -> None:
    # FND-137 recorded what the server could not do: an agent could prove
    # two runs emitted the same log and could not see which tile changed.
    # This is the test of the answer, driven through a real client.
    async def body(session: ClientSession) -> dict[str, Any]:
        built = await _call(session, "build_world", width=8, height=8, seed=7)
        await _call(session, "step_world", world=built["world"], ticks=1)
        return await _call(session, "tile_changes", world=built["world"], limit=4)

    report = _drive(body)
    assert report["event_count"] > 0
    assert report["rows_returned"] == min(4, report["event_count"])
    assert len(report["changes"]) == report["rows_returned"]
    for row in report["changes"]:
        assert set(row) == {"tile", "value", "holder", "kind"}
        assert isinstance(row["value"], int)
        assert 0 <= row["tile"] < 64


def test_a_client_follows_the_unit_that_took_a_resource() -> None:
    # The gather event names a unit. The identity crosses whole, and the
    # client gives it back without taking it apart.
    async def body(session: ClientSession) -> tuple[dict[str, Any], dict[str, Any]]:
        built = await _call(
            session, "build_world", width=16, height=16, seed=GATHER_SEED
        )
        name = built["world"]
        spawned = await _call(
            session,
            "spawn_units",
            world=name,
            addresses=[list(address) for address in GATHER_ADDRESSES],
        )
        units = [int(unit) for unit in spawned["units"]]
        await _call(session, "order_gather", world=name, units=units, kind=GATHER_KIND)

        grants: dict[str, Any] = {}
        for _ in range(8):
            await _call(session, "step_world", world=name, ticks=1)
            grants = await _call(session, "gather_events", world=name, limit=4)
            if grants["event_count"]:
                break
        assert grants["event_count"], "the fixture must produce a gather event"
        followed = await _call(
            session, "unit_tile", world=name, unit=grants["grants"][0]["unit"]
        )
        return grants, followed

    grants, followed = _drive(body)
    first = grants["grants"][0]
    assert first["amount"] > 0
    assert followed["tile"] == first["tile"]


def test_the_identity_of_a_removed_unit_is_a_tool_error() -> None:
    # ADR-0085 D3: the engine refuses a stale identity. It never reports on
    # the unit that now holds the slot. The refusal must reach the client
    # as an error and not as a plausible answer.
    #
    # **This test proves the refusal reaches the client. It does not cover
    # the resolution.** Deleting the generation comparison in
    # resolve_soldier leaves this test green, because the arena compares the
    # generation again when it reads a tile. The package test of the write
    # verbs is what catches that.[^1]
    #
    # [^1]: Findings register, FND-148. `docs/FINDINGS.md`
    async def body(session: ClientSession) -> tuple[bool, dict[str, Any]]:
        built = await _call(session, "build_world", width=8, height=8, seed=7)
        name = built["world"]
        first = await _call(session, "spawn_units", world=name, addresses=[[0, 0]])
        dead = int(first["units"][0])
        await _call(session, "despawn_units", world=name, units=[dead])
        second = await _call(session, "spawn_units", world=name, addresses=[[0, 0]])
        living = int(second["units"][0])
        assert living != dead
        stale = await session.call_tool("unit_tile", {"world": name, "unit": dead})
        return bool(getattr(stale, "is_error", False)), await _call(
            session, "unit_tile", world=name, unit=living
        )

    refused, alive = _drive(body)
    assert refused, "the dead identity must refuse"
    assert alive["tile"] == 0


# The world the reading tests build.
#
# The extent matters. The pyramid puts one level 1 cell over a world of this
# size, so a test can compare the cell against every tile of the world in 64
# calls. A larger world would need a second cell and the test would have to
# know which tiles the cell covers, which is engine geometry the control
# plane must not hold a copy of.
READ_WIDTH = 8
READ_HEIGHT = 8

# **The seed is the assertion's, not the world's.** Seed 7 gives a world of
# hill and mountain in which every tile admits a unit. In such a world the
# open tiles equal the tiles, so a read that returned one where the other
# belongs is invisible. The defect was put back and the suite stayed green.
#
# Seed 51 gives three tiles of water at (0, 6), (0, 7) and (1, 7), and forest
# everywhere else. The open tiles are then fewer than the tiles, and two kinds
# of ground occur, so a swap of either pair fails. Measured with the engine's
# own census over the whole extent.[^4]
READ_SEED = 51

# The window the census test reads.
#
# It is placed over the water rather than in the middle of the world. A window
# of one kind of ground measures the fixture and not the census.[^4]
CENSUS_CENTRE = (1, 6)
CENSUS_RADIUS = 2


async def _every_tile(
    session: ClientSession, world: str, first: tuple[int, int], last: tuple[int, int]
) -> list[dict[str, Any]]:
    """Read every tile of a rectangle of addresses, one call each.

    **This loop belongs to the test, not to the control plane.** It exists to
    check an aggregate the engine computed against the tiles it summed, and
    the rule for an aggregate is that it equals that sum exactly.[^3]
    """
    rows: list[dict[str, Any]] = []
    for r in range(first[1], last[1] + 1):
        for q in range(first[0], last[0] + 1):
            rows.append(await _call(session, "tile_report", world=world, q=q, r=r))
    return rows


def test_a_level_one_cell_equals_the_sum_of_its_tiles() -> None:
    """The region summary is derived, so it must equal level 0 exactly.

    This is the strongest thing any of the reading tools can claim. The
    engine folded the tiles into the cell, and this test folds them again
    from a different tool and compares. A summary that had drifted from the
    tiles it summarises would fail here.
    """

    async def body(
        session: ClientSession,
    ) -> tuple[dict[str, Any], list[dict[str, Any]]]:
        built = await _call(
            session,
            "build_world",
            width=READ_WIDTH,
            height=READ_HEIGHT,
            seed=READ_SEED,
        )
        name = built["world"]
        cell = await _call(session, "region_summary", world=name, q=0, r=0)
        tiles = await _every_tile(
            session, name, (0, 0), (READ_WIDTH - 1, READ_HEIGHT - 1)
        )
        return cell, tiles

    cell, tiles = _drive(body)
    assert cell["tiles"] == len(tiles) == READ_WIDTH * READ_HEIGHT
    assert cell["open_tiles"] == sum(1 for tile in tiles if tile["passable"])
    # The fixture must reach the case. A world in which every tile admits a
    # unit gives the same number for both, and the assertion above then holds
    # whichever one the engine returned.[^4]
    assert cell["open_tiles"] < cell["tiles"], "the fixture must hold closed ground"
    assert cell["food_total"] == sum(tile["stock"][0] for tile in tiles)
    assert cell["value_total"] == sum(tile["value"] for tile in tiles)


def test_a_census_equals_the_tiles_it_counted() -> None:
    """The census reports the window it read, so a reader can repeat it.

    The corners come back in the report. The test reads every address between
    them and adds the counts up itself.
    """

    async def body(
        session: ClientSession,
    ) -> tuple[dict[str, Any], list[dict[str, Any]]]:
        built = await _call(
            session,
            "build_world",
            width=READ_WIDTH,
            height=READ_HEIGHT,
            seed=READ_SEED,
        )
        name = built["world"]
        counted = await _call(
            session,
            "window_census",
            world=name,
            q=CENSUS_CENTRE[0],
            r=CENSUS_CENTRE[1],
            radius=CENSUS_RADIUS,
        )
        tiles = await _every_tile(
            session,
            name,
            (counted["first_q"], counted["first_r"]),
            (counted["last_q"], counted["last_r"]),
        )
        return counted, tiles

    counted, tiles = _drive(body)
    # The count is the assertion's. The report names its own corners, so a
    # census that read the wrong window would still agree with a sum over the
    # window it reported. This number is what refuses that.
    assert counted["tiles"] == len(tiles) == 16
    assert counted["open_tiles"] == sum(1 for tile in tiles if tile["passable"])
    for kind, count in enumerate(counted["by_kind"]):
        assert count == sum(1 for tile in tiles if tile["kind"] == kind)
    # The window must hold both kinds of ground and some closed ground, or it
    # measures the fixture rather than the census.[^4]
    assert counted["open_tiles"] < counted["tiles"]
    assert sum(1 for count in counted["by_kind"] if count) > 1


def test_a_census_clips_a_window_to_the_world() -> None:
    """A window at a corner reads the part of it that the world holds."""

    async def body(session: ClientSession) -> dict[str, Any]:
        built = await _call(
            session,
            "build_world",
            width=READ_WIDTH,
            height=READ_HEIGHT,
            seed=READ_SEED,
        )
        return await _call(
            session, "window_census", world=built["world"], q=0, r=0, radius=3
        )

    counted = _drive(body)
    assert (counted["first_q"], counted["first_r"]) == (0, 0)
    assert (counted["last_q"], counted["last_r"]) == (3, 3)
    assert counted["tiles"] == 16


def test_a_census_refuses_a_radius_above_the_ceiling() -> None:
    """A call that could name the whole world is a pass over the world."""

    async def body(session: ClientSession) -> bool:
        built = await _call(session, "build_world", width=8, height=8, seed=READ_SEED)
        refused = await session.call_tool(
            "window_census", {"world": built["world"], "q": 0, "r": 0, "radius": 4096}
        )
        return bool(getattr(refused, "is_error", False))

    assert _drive(body), "the ceiling must refuse"


def test_a_census_counts_the_units_a_spawn_put_in_the_window() -> None:
    """The crowding counts are of the window, and a spawn moves them.

    The step is not decoration. The unit-to-tile bridge is derived and it
    rebuilds at the barrier, so a census taken before the step reads a bridge
    that predates the spawn and refuses.
    """

    async def body(session: ClientSession) -> tuple[bool, dict[str, Any]]:
        built = await _call(
            session, "build_world", width=16, height=16, seed=GATHER_SEED
        )
        name = built["world"]
        await _call(
            session,
            "spawn_units",
            world=name,
            addresses=[list(address) for address in GATHER_ADDRESSES],
        )
        stale = await session.call_tool(
            "window_census", {"world": name, "q": 0, "r": 0, "radius": 2}
        )
        await _call(session, "step_world", world=name, ticks=1)
        return bool(getattr(stale, "is_error", False)), await _call(
            session, "window_census", world=name, q=0, r=0, radius=2
        )

    refused, counted = _drive(body)
    assert refused, "a census over a stale bridge must refuse"
    assert counted["units"] == len(GATHER_ADDRESSES)
    assert counted["crowd_worst"] == 1
    assert (counted["crowded_q"], counted["crowded_r"]) != (None, None)


def test_a_tile_reports_the_stock_as_what_was_given_less_what_was_taken() -> None:
    """The stock of a tile is derived, and the report gives all three numbers.

    This is the read the fixture comment above says the control plane did not
    have. The seed was chosen against the engine's own read from Rust and
    recorded in a comment, because nothing in Python could ask. This test
    asks.
    """

    async def body(session: ClientSession) -> list[dict[str, Any]]:
        built = await _call(
            session, "build_world", width=16, height=16, seed=GATHER_SEED
        )
        return [
            await _call(session, "tile_report", world=built["world"], q=q, r=r)
            for q, r in GATHER_ADDRESSES
        ]

    tiles = _drive(body)
    for tile in tiles:
        assert tile["passable"], "the gather fixture needs ground that admits a unit"
        for kind, stock in enumerate(tile["stock"]):
            assert stock == tile["generated"][kind] - tile["taken"][kind]
            assert tile["taken"][kind] == 0, "nothing has gathered yet"
    # The comment on GATHER_SEED records that seed 1 holds food at three of
    # these four addresses. It was measured from Rust, because nothing here
    # could read it. Now something can, so the claim is a check.
    holding_food = [tile for tile in tiles if tile["generated"][GATHER_KIND] > 0]
    assert len(holding_food) == 3


def test_a_gather_moves_the_stock_the_tile_reports() -> None:
    """What a unit took leaves the tile, and the tile says so."""

    async def body(session: ClientSession) -> tuple[dict[str, Any], dict[str, Any]]:
        built = await _call(
            session, "build_world", width=16, height=16, seed=GATHER_SEED
        )
        name = built["world"]
        spawned = await _call(
            session,
            "spawn_units",
            world=name,
            addresses=[list(address) for address in GATHER_ADDRESSES],
        )
        units = [int(unit) for unit in spawned["units"]]
        await _call(session, "order_gather", world=name, units=units, kind=GATHER_KIND)
        grants: dict[str, Any] = {}
        for _ in range(8):
            await _call(session, "step_world", world=name, ticks=1)
            grants = await _call(session, "gather_events", world=name, limit=4)
            if grants["event_count"]:
                break
        assert grants["event_count"], "the fixture must produce a gather event"
        first = grants["grants"][0]
        # The engine turns the identity into an address. The test holds no
        # rule of its own for reading a tile index, because the grid owns
        # that rule and a copy here would be a second declaration of it.
        where = await _call(session, "unit_choice", world=name, unit=first["unit"])
        assert where["tile"] == first["tile"], "the taker must stand where it took"
        tile = await _call(
            session, "tile_report", world=name, q=where["q"], r=where["r"]
        )
        return first, tile

    first, tile = _drive(body)
    assert tile["taken"][first["kind"]] > 0
    assert tile["stock"][first["kind"]] == (
        tile["generated"][first["kind"]] - tile["taken"][first["kind"]]
    )


def test_the_chosen_option_follows_the_scores_the_engine_reported() -> None:
    """The engine reports every score, and the option it names must follow.

    The scan starts at the floor and uses a strict comparison, so the lowest
    option index wins a tie and an option that only equals the floor never
    wins. A unit whose every score is below the floor holds what it was
    doing, and the engine names no option.
    """

    async def body(session: ClientSession) -> dict[str, Any]:
        built = await _call(
            session, "build_world", width=16, height=16, seed=GATHER_SEED
        )
        name = built["world"]
        spawned = await _call(session, "spawn_units", world=name, addresses=[[0, 0]])
        await _call(session, "step_world", world=name, ticks=1)
        return await _call(
            session, "unit_choice", world=name, unit=int(spawned["units"][0])
        )

    answer = _drive(body)
    assert len(answer["scores"]) == len(answer["weights"]) == len(answer["fields"])
    best = None
    for option, score in enumerate(answer["scores"]):
        if score > answer["floor"] and (best is None or score > answer["scores"][best]):
            best = option
    no_intent = 255
    if best is None:
        assert answer["best"] == no_intent
        assert answer["best_name"] is None
    else:
        assert answer["best"] == best
        assert answer["best_name"] is not None


def test_a_founding_provisions_the_site_it_seated() -> None:
    """The loop the panel shows must close through the server too.

    The survey reads the ground, the founding sets the production rate from
    what it read, and the store feeds the units of that site. A site founded
    at an address of a caller's choosing earns nothing, so a report of zero
    production here would mean the founding tool bypassed the survey.
    """

    async def body(session: ClientSession) -> tuple[dict[str, Any], dict[str, Any]]:
        built = await _call(
            session, "build_world", width=32, height=32, seed=GATHER_SEED
        )
        name = built["world"]
        made = await _call(session, "found_group", world=name, group=8, faction=0)
        return made, await _call(session, "site_economy", world=name, site=made["site"])

    made, economy = _drive(body)
    assert made["seated"] > 0
    assert made["food"] > 0, "the fixture needs a place that reaches food"
    assert (economy["q"], economy["r"]) == (made["q"], made["r"])
    assert economy["faction"] == made["faction"]
    assert economy["production"] > 0, "the founding must set the rate from the survey"
    # The founding sets the production rate and never the upkeep. A report
    # that gave one where the other belongs would pass every assertion above,
    # and the defect was put back to prove it.
    assert economy["upkeep"] == 0, "the founding writes no upkeep"
    assert economy["production"] != economy["upkeep"]
    assert economy["rationed"] is False
    assert economy["demanded"] is None


def test_a_survey_ranks_the_place_the_founding_takes_first() -> None:
    """The survey answers why, and the founding acts on the same answer.

    Two worlds, the same settings. One is surveyed and not founded. The other
    is founded. The place the founding took must be the first row of the
    survey, or the survey explains a choice that was not made.
    """

    async def body(session: ClientSession) -> tuple[dict[str, Any], dict[str, Any]]:
        looked = await _call(
            session, "build_world", width=32, height=32, seed=GATHER_SEED
        )
        survey = await _call(
            session, "founding_survey", world=looked["world"], group=8, faction=0
        )
        acted = await _call(
            session, "build_world", width=32, height=32, seed=GATHER_SEED
        )
        made = await _call(
            session, "found_group", world=acted["world"], group=8, faction=0
        )
        return survey, made

    survey, made = _drive(body)
    assert survey["considered"] > 1
    assert survey["tiles_read"] > survey["considered"]
    first = survey["candidates"][0]
    assert first["eligible"] is True
    assert (first["q"], first["r"]) == (made["q"], made["r"])
    assert first["score"] == made["score"]
    assert first["food"] == made["food"]
    # Every eligible place precedes every refused one, and the scores of the
    # eligible places never rise.
    eligible = [row for row in survey["candidates"] if row["eligible"]]
    assert survey["candidates"][: len(eligible)] == eligible
    for earlier, later in itertools.pairwise(eligible):
        assert earlier["score"] >= later["score"]


def test_a_dead_site_identity_is_a_tool_error() -> None:
    """A site that no longer stands never answers for another one."""

    async def body(session: ClientSession) -> bool:
        built = await _call(session, "build_world", width=8, height=8, seed=READ_SEED)
        refused = await session.call_tool(
            "site_economy", {"world": built["world"], "site": 1}
        )
        return bool(getattr(refused, "is_error", False))

    assert _drive(body), "an identity the engine never gave must refuse"
