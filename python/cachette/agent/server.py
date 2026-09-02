"""A Model Context Protocol server over the engine's control plane.

An agent that works on this repository needs to run the engine. Without this
server the agent reads the source and guesses, or it writes a throwaway test.
This server lets the agent build a world, step it, and read the result
through tool calls.

The server speaks the Model Context Protocol over standard input and output.
Start it with ``python -m cachette.agent``.

The server adds no simulation logic. Every tool calls one method of the
compiled module, and the engine answers. Python is the control plane, and
Python is not the data plane, so no tool loops over entities.[^1]

The server does not decode the event log. The layout of an event lives in the
Rust source, and a decoder here would be a second declaration site for it,
with nothing that fails when the two disagree.[^2] The event log tool returns
the bytes and a digest of them instead.[^3]

The reference implementation of the protocol is a development dependency, not
a runtime dependency of the package. The server is a tool for a contributor
to this repository.

References
----------
[^1]: Project orientation, the design principles. ``CLAUDE.md``
[^2]: Recurring Defect Shapes, shape 1. ``.claude/rules/recurring-defects.md``
[^3]: Findings register, FND-137. ``docs/FINDINGS.md``
"""

from __future__ import annotations

import base64
import hashlib
from dataclasses import dataclass

from mcp.server.mcpserver import MCPServer

from cachette._core import version
from cachette.agent.session import SessionStore, WorldSession, WorldSettings

__all__ = ["EventLogReport", "InvariantReport", "WorldReport", "build_server", "main"]

INSTRUCTIONS = """
This server runs the Cachette world simulation engine.

Build a world with build_world. Every other tool takes the name it returns.
Step the world with step_world, then read the result with world_report.

The state hash is a hexadecimal string. It is the value to compare when you
check that two runs agree. The same settings and the same tick count give the
same hash at any thread count.

The engine holds the entities. No tool returns a list of them.
""".strip()

# The event log of a large step does not belong in an agent's context. The
# tool returns the head of the log and says how much it left out. The digest
# always covers the whole log.
DEFAULT_MAX_BYTES = 4096


@dataclass(frozen=True)
class WorldReport:
    """What the engine reports about one world."""

    world: str
    engine_version: str
    width: int
    height: int
    seed: int
    faction_count: int
    tick: int
    tile_count: int
    state_hash: str
    event_count: int


@dataclass(frozen=True)
class InvariantReport:
    """The result of the engine's invariant check."""

    world: str
    tick: int
    holds: bool


@dataclass(frozen=True)
class EventLogReport:
    """The event log of the last step.

    The bytes are the engine's own layout. This server does not decode them,
    because a decoder here would repeat a layout that the Rust source already
    declares.
    """

    world: str
    tick: int
    event_count: int
    byte_count: int
    digest_sha256: str
    bytes_base64: str
    bytes_returned: int
    truncated: bool


def _report(session: WorldSession) -> WorldReport:
    """Read the state of one world."""
    world = session.world
    return WorldReport(
        world=session.name,
        engine_version=version(),
        width=world.width,
        height=world.height,
        seed=session.settings.seed,
        faction_count=session.settings.faction_count,
        tick=world.tick,
        tile_count=world.tile_count,
        state_hash=f"{world.state_hash():016x}",
        event_count=world.event_count,
    )


def build_server(store: SessionStore | None = None) -> MCPServer:
    """Build the server and register every tool.

    Pass a store to inspect the worlds a test drove. The server builds its
    own store when the argument is absent.
    """
    sessions = store if store is not None else SessionStore()
    server = MCPServer(
        name="cachette",
        title="Cachette world simulation engine",
        instructions=INSTRUCTIONS,
        version=version(),
    )

    @server.tool(
        title="Build a world",
        description=(
            "Builds a world and returns its name and its state. Every other "
            "tool takes the name."
        ),
    )
    def build_world(
        width: int = 64,
        height: int = 64,
        seed: int = 1,
        faction_count: int = 4,
    ) -> WorldReport:
        """Build a world of the given extent, seed and faction count.

        The world is a rhombus, so the extent is a width and a height. The
        engine refuses an extent that does not describe a world. The server
        sets no upper bound, so a large extent costs the memory it costs.
        """
        settings = WorldSettings(
            width=width, height=height, seed=seed, faction_count=faction_count
        )
        return _report(sessions.create(settings))

    @server.tool(
        title="Step a world",
        description=(
            "Runs the world forward a number of ticks at a thread count, and "
            "returns the state after the last tick."
        ),
    )
    def step_world(world: str, ticks: int = 1, threads: int = 1) -> WorldReport:
        """Run the named world forward.

        The thread count does not change the result. The engine gives one
        answer at any thread count, and the state hash proves it.

        The reported event count is the count of the last tick only. The
        engine keeps one step of the log.
        """
        if ticks < 1:
            raise ValueError(f"ticks is {ticks}; it must be 1 or more")
        if threads < 1:
            raise ValueError(f"threads is {threads}; it must be 1 or more")
        session = sessions.get(world)
        for _ in range(ticks):
            session.world.step(threads=threads)
        return _report(session)

    @server.tool(
        title="Report a world",
        description="Returns the tick, the state hash, and the counts.",
    )
    def world_report(world: str) -> WorldReport:
        """Read the named world without changing it."""
        return _report(sessions.get(world))

    @server.tool(
        title="Check the invariants",
        description="Runs the engine's invariant check and reports the result.",
    )
    def check_invariants(world: str) -> InvariantReport:
        """Ask the engine whether the world holds its invariants."""
        session = sessions.get(world)
        return InvariantReport(
            world=session.name,
            tick=session.world.tick,
            holds=session.world.check_invariants(),
        )

    @server.tool(
        title="Read the event log",
        description=(
            "Returns the event log of the last step as bytes in base64, with "
            "a digest of the whole log. This server does not decode the bytes."
        ),
    )
    def event_log(world: str, max_bytes: int = DEFAULT_MAX_BYTES) -> EventLogReport:
        """Return the raw event log of the last step.

        The digest covers the whole log. Two runs that agree have the same
        digest, so the digest answers the question without the bytes.
        """
        if max_bytes < 0:
            raise ValueError(f"max_bytes is {max_bytes}; it must be 0 or more")
        session = sessions.get(world)
        raw = session.world.event_log_bytes()
        head = raw[:max_bytes]
        return EventLogReport(
            world=session.name,
            tick=session.world.tick,
            event_count=session.world.event_count,
            byte_count=len(raw),
            digest_sha256=hashlib.sha256(raw).hexdigest(),
            bytes_base64=base64.b64encode(head).decode("ascii"),
            bytes_returned=len(head),
            truncated=len(head) < len(raw),
        )

    return server


def main() -> None:
    """Run the server over standard input and output."""
    build_server().run(transport="stdio")
