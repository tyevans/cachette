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
"""

from __future__ import annotations

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
        "gather_events",
        "order_gather",
        "spawn_units",
        "step_world",
        "tile_changes",
        "unit_tile",
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
