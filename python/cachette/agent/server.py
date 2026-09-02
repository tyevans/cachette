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
with nothing that fails when the two disagree.[^2] The bindings return one
column for each field of an event, and this server reads the columns by their
names.[^4] It holds no byte offset, no field width and no field order. The
event log tool still returns the bytes and a digest of them, because a digest
answers whether two runs agree without the bytes.

A gather event names the unit that took the amount. The name is the whole
identity of the unit, and this server passes it back to the engine without
taking it apart. The engine resolves it, and it refuses the identity of a unit
that has died.[^5]

The reference implementation of the protocol is a development dependency, not
a runtime dependency of the package. The server is a tool for a contributor
to this repository.

References
----------
[^1]: Project orientation, the design principles. ``CLAUDE.md``
[^2]: Recurring Defect Shapes, shape 1. ``.claude/rules/recurring-defects.md``
[^3]: Findings register, FND-137. ``docs/FINDINGS.md``
[^4]: Decisions register, DEC-060. ``docs/DECISIONS.md``
[^5]: ADR-0085, an entity crosses to Python as one opaque identity that the
    engine resolves.
    ``docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md``
"""

from __future__ import annotations

import base64
import hashlib
from dataclasses import dataclass

from mcp.server.mcpserver import MCPServer

from cachette import version
from cachette.agent.session import SessionStore, WorldSession, WorldSettings

__all__ = [
    "EventLogReport",
    "GatherEvent",
    "GatherReport",
    "InvariantReport",
    "RemovalReport",
    "SpawnReport",
    "TileChange",
    "TileChangeReport",
    "UnitReport",
    "WorldReport",
    "build_server",
    "main",
]

INSTRUCTIONS = """
This server runs the Cachette world simulation engine.

Build a world with build_world. Every other tool takes the name it returns.
Step the world with step_world, then read the result with world_report.

The state hash is a hexadecimal string. It is the value to compare when you
check that two runs agree. The same settings and the same tick count give the
same hash at any thread count.

The engine holds the entities. No tool returns a list of them.

Read what a step changed with tile_changes. Put units in the world with
spawn_units, tell them to gather with order_gather, step, then read
gather_events.

Every verb that acts on units takes a set and answers once. There is no
per-unit verb to call in a loop, because a unit is one of a million.

A gather event names a unit by its identity. That identity is one opaque
number. Do not build one and do not take one apart. Pass it back to unit_tile
or to despawn_units. The engine refuses the identity of a unit that has died,
so a stale identity is an error and never a report about another unit.
""".strip()

# The event log of a large step does not belong in an agent's context. The
# tool returns the head of the log and says how much it left out. The digest
# always covers the whole log.
DEFAULT_MAX_BYTES = 4096

# A world of many tiles changes many of them in one step. A tool that returned
# every row would fill an agent's context with the world. The reading tools
# return the head of the log and say how many rows they left out.
DEFAULT_MAX_ROWS = 64


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
    declares. Read the fields with tile_changes, which takes one column for
    each field from the engine.

    The bytes and the digest answer a different question: whether two runs
    emitted the same log.
    """

    world: str
    tick: int
    event_count: int
    byte_count: int
    digest_sha256: str
    bytes_base64: str
    bytes_returned: int
    truncated: bool


@dataclass(frozen=True)
class TileChange:
    """One row of the tile change log."""

    tile: int
    value: int
    holder: int
    kind: int


@dataclass(frozen=True)
class TileChangeReport:
    """What a step changed, read from the engine's own columns.

    The value is the raw fixed-point integer of the engine. It is not a
    floating point number, and this server does not scale it.

    The holder names a faction, or nobody. The value for nobody sits above
    the faction ceiling, so no faction collides with it.
    """

    world: str
    tick: int
    event_count: int
    rows_returned: int
    truncated: bool
    changes: list[TileChange]


@dataclass(frozen=True)
class GatherEvent:
    """One row of the gather log.

    The unit is the whole identity of the unit that took the amount. It is
    not a slot index. Pass it back; do not take it apart.
    """

    unit: int
    tile: int
    amount: int
    kind: int


@dataclass(frozen=True)
class GatherReport:
    """What the gather resolve granted in the last step."""

    world: str
    tick: int
    event_count: int
    rows_returned: int
    truncated: bool
    grants: list[GatherEvent]


@dataclass(frozen=True)
class UnitReport:
    """One unit, named by the identity the engine gave."""

    world: str
    tick: int
    unit: int
    tile: int


@dataclass(frozen=True)
class SpawnReport:
    """The units a set-valued call names.

    The identities are opaque. Pass them back; do not take one apart.
    """

    world: str
    tick: int
    units: list[int]
    soldier_count: int


@dataclass(frozen=True)
class RemovalReport:
    """The units that the engine removed.

    There is no field for whether it removed them. The engine resolves every
    identity first and refuses a dead one, so a report that exists is a
    report of a removal.
    """

    world: str
    tick: int
    units: list[int]
    soldier_count: int


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
            "a digest of the whole log, to compare two runs. To read what "
            "changed, call tile_changes instead."
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

    @server.tool(
        title="Read what a step changed",
        description=(
            "Returns the tile changes of the last step, as rows. The engine "
            "returns one column for each field, so this server holds no copy "
            "of the event layout."
        ),
    )
    def tile_changes(world: str, limit: int = DEFAULT_MAX_ROWS) -> TileChangeReport:
        """Return the tile change log of the last step.

        The value of a row is the raw fixed-point integer of the engine.
        """
        if limit < 0:
            raise ValueError(f"limit is {limit}; it must be 0 or more")
        session = sessions.get(world)
        columns = session.world.event_log_columns()
        total = len(columns["tile"])
        head = min(limit, total)
        rows = [
            TileChange(
                tile=int(columns["tile"][row]),
                value=int(columns["value"][row]),
                holder=int(columns["holder"][row]),
                kind=int(columns["kind"][row]),
            )
            for row in range(head)
        ]
        return TileChangeReport(
            world=session.name,
            tick=session.world.tick,
            event_count=total,
            rows_returned=head,
            truncated=head < total,
            changes=rows,
        )

    @server.tool(
        title="Read the gather grants",
        description=(
            "Returns what the gather resolve granted in the last step. Each "
            "row names the unit by its identity."
        ),
    )
    def gather_events(world: str, limit: int = DEFAULT_MAX_ROWS) -> GatherReport:
        """Return the gather log of the last step.

        The unit of a row is an opaque identity. Pass it to unit_tile.
        """
        if limit < 0:
            raise ValueError(f"limit is {limit}; it must be 0 or more")
        session = sessions.get(world)
        columns = session.world.gather_log_columns()
        total = len(columns["unit"])
        head = min(limit, total)
        rows = [
            GatherEvent(
                unit=int(columns["unit"][row]),
                tile=int(columns["tile"][row]),
                amount=int(columns["amount"][row]),
                kind=int(columns["kind"][row]),
            )
            for row in range(head)
        ]
        return GatherReport(
            world=session.name,
            tick=session.world.tick,
            event_count=total,
            rows_returned=head,
            truncated=head < total,
            grants=rows,
        )

    @server.tool(
        title="Put units in the world",
        description=(
            "Adds a unit at each address and returns the identities the "
            "engine gave them. Every other unit tool takes an identity."
        ),
    )
    def spawn_units(
        world: str, addresses: list[list[int]], faction: int = 0
    ) -> SpawnReport:
        """Add one unit of a faction at each axial address.

        Give a list of two-number addresses. The set is all or nothing: an
        address the engine refuses leaves no unit behind, and the error names
        the address. The engine refuses an address outside the world, ground
        that admits no unit, and a faction the world does not hold.
        """
        pairs: list[tuple[int, int]] = []
        for address in addresses:
            if len(address) != 2:
                raise ValueError(f"the address {address} is not a q and an r")
            pairs.append((address[0], address[1]))
        session = sessions.get(world)
        units = session.world.spawn_soldiers(pairs, faction)
        return SpawnReport(
            world=session.name,
            tick=session.world.tick,
            units=[int(unit) for unit in units],
            soldier_count=session.world.soldier_count,
        )

    @server.tool(
        title="Read a unit",
        description=(
            "Returns the tile a unit stands on. The engine refuses the "
            "identity of a unit that has died."
        ),
    )
    def unit_tile(world: str, unit: int) -> UnitReport:
        """Read one unit by the identity the engine gave.

        A unit that died leaves its slot to another unit. This call refuses
        the dead identity rather than report on the new occupant.
        """
        session = sessions.get(world)
        return UnitReport(
            world=session.name,
            tick=session.world.tick,
            unit=unit,
            tile=session.world.soldier_tile(unit),
        )

    @server.tool(
        title="Order a unit to gather",
        description=(
            "Tells one unit to gather a kind of resource. The kind is the "
            "number that the gather event carries."
        ),
    )
    def order_gather(world: str, units: list[int], kind: int = 0) -> SpawnReport:
        """Tell every named unit to gather until it is told to stop.

        The set is all or nothing. One dead identity leaves the whole set
        without an order.
        """
        session = sessions.get(world)
        session.world.order_gather(units, kind)
        return SpawnReport(
            world=session.name,
            tick=session.world.tick,
            units=list(units),
            soldier_count=session.world.soldier_count,
        )

    @server.tool(
        title="Remove a unit",
        description="Removes one unit and reports whether it removed one.",
    )
    def despawn_units(world: str, units: list[int]) -> RemovalReport:
        """Remove every unit the identities name.

        Every identity resolves before anything is removed, so one dead
        identity removes nothing and is an error rather than a false answer.
        """
        session = sessions.get(world)
        session.world.despawn_soldiers(units)
        return RemovalReport(
            world=session.name,
            tick=session.world.tick,
            units=list(units),
            soldier_count=session.world.soldier_count,
        )

    return server


def main() -> None:
    """Run the server over standard input and output."""
    build_server().run(transport="stdio")
