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
        "event_log",
        "step_world",
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
