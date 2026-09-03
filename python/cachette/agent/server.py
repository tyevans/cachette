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

**This tool set grows one tool at a time, against a need somebody stated.** A
tool exists here because a reader could not answer a question without it, and
never because the engine happens to hold the value. A gap that no engine call
closes is recorded as a finding and answered by an engine verb, never by a
computation in this file.[^6] The need that the set serves names its
audience.[^7]

A tool that reads a bounded window names the window and hands it to the
engine. The engine walks it. This file walks nothing.

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
[^6]: ADR-0092, the agent tool surface grows one tool at a time, against a
    stated need, decisions D1 and D2.
    ``docs/adrs/draft/adr-0092-the-agent-tool-surface-grows-against-a-stated-need.md``
[^7]: PRD-0019, an agent can ask the running engine what it holds.
    ``docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md``
"""

from __future__ import annotations

import base64
import hashlib
from dataclasses import dataclass

from mcp.server.mcpserver import MCPServer

from cachette import version
from cachette.agent.session import SessionStore, WorldSession, WorldSettings

__all__ = [
    "CensusReport",
    "ChoiceReport",
    "EventLogReport",
    "FoundingReport",
    "GatherEvent",
    "GatherReport",
    "InvariantReport",
    "RegionReport",
    "RemovalReport",
    "SiteEconomyReport",
    "SpawnReport",
    "SurveyCandidate",
    "SurveyReport",
    "TileChange",
    "TileChangeReport",
    "TileReport",
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

Read the ground of one address with tile_report. It says what the generator
put there, what units took, and who holds it. Count a square window of
addresses with window_census: the engine walks the window and answers once.

Read the region a tile sits in with region_summary. Ask why a unit chose what
it chose with unit_choice. Ask what a founding would compare with
founding_survey, which founds nothing. Found a settlement with found_sites,
then read what it earns, holds and owes with site_economy.

Every fixed-point value crosses as its raw integer. This server does not
scale one, because a scaled value would be a floating point number and the
engine holds none.

Every verb that acts on units takes a set and answers once. There is no
per-unit verb to call in a loop, because a unit is one of a million.

A gather event names a unit by its identity, and found_sites names a
settlement the same way. That identity is one opaque number. Do not build one
and do not take one apart. Pass it back. The engine refuses the identity of a
unit or a site that is gone, so a stale identity is an error and never a
report about another one.

This tool set grows against a need somebody stated, and never ahead of one.
If you need something the engine holds and no tool reports, that gap is worth
recording rather than working around.
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


@dataclass(frozen=True)
class FoundingReport:
    """What one founding chose, and what it made.

    Every quantity is the one the survey read at the chosen place. The server
    recomputes no score, so no number here can disagree with the choice the
    engine made.

    The site is an opaque identity. Pass it to site_economy; do not take it
    apart.
    """

    world: str
    tick: int
    site: int
    q: int
    r: int
    faction: int
    seated: int
    score: int
    food: int
    wood: int
    stone: int
    open_ground: int
    room: int
    water_edge: int
    drawn: int
    considered: int
    tiles_read: int
    site_count: int
    soldier_count: int


@dataclass(frozen=True)
class SurveyCandidate:
    """One place a founding survey read, and what it read there."""

    q: int
    r: int
    score: int
    food: int
    wood: int
    stone: int
    open_ground: int
    room: int
    water_edge: int
    eligible: bool
    separated: bool


@dataclass(frozen=True)
class SurveyReport:
    """What a founding would read if a group looked for a place now.

    The candidates are in the order the founding ranks them, best first, so
    the first row is the place a founding would take. A row whose eligible
    field is false is a place the founding refuses.

    The score is the engine's own weighted sum of the counts beside it. This
    server recomputes nothing, so no number here can disagree with the choice
    the engine would make.
    """

    world: str
    tick: int
    group: int
    faction: int
    drawn: int
    considered: int
    tiles_read: int
    rows_returned: int
    truncated: bool
    candidates: list[SurveyCandidate]


@dataclass(frozen=True)
class RegionReport:
    """The level 1 summary of the cell that covers one tile.

    Every field is an exact integer total over the level 0 tiles of the cell.
    A reader that adds the tiles of the cell gets these numbers back, because
    level 0 is the only source of truth and this level is derived from it.

    The three totals are raw accumulator integers. They are not scaled and
    they are never floating point numbers.
    """

    world: str
    tick: int
    q: int
    r: int
    tiles: int
    open_tiles: int
    units: int
    held_tiles: int
    value_total: int
    height_total: int
    food_total: int


@dataclass(frozen=True)
class SiteEconomyReport:
    """What one site earns, holds and owes.

    The store, the production and the upkeep are Q16.16 values as their raw
    integers. This server does not scale them.

    The demanded and granted fields hold the last draw the site could not
    serve in full. They are absent when the site served every cohort, because
    the engine keeps that log for one tick only.
    """

    world: str
    tick: int
    site: int
    q: int
    r: int
    faction: int
    commodity: int
    store: int
    production: int
    upkeep: int
    rationed: bool
    demanded: int | None
    granted: int | None


@dataclass(frozen=True)
class ChoiceReport:
    """Why one unit chose what it chose.

    The engine recomputes the answer from the world as it stands. It stores
    no score.

    Every score, field value, weight and floor is a Q16.16 value as its raw
    integer. An option whose score is below the floor cannot win.

    The best field names the option the scores select. The best_name field is
    absent when every score is below the floor, and the unit then holds what
    it was doing.

    The q and r fields name the tile the unit stands on. Give them to
    region_summary to read the cell the unit scored.
    """

    world: str
    tick: int
    unit: int
    tile: int
    q: int
    r: int
    cell: int
    need: int
    scores: list[int]
    fields: list[int]
    weights: list[int]
    floor: int
    best: int
    best_name: str | None
    intent: int
    chooses_next_frame: bool


@dataclass(frozen=True)
class TileReport:
    """What one tile holds.

    The stock, generated and taken fields hold one amount for each kind of
    resource, in the order of the kind numbering. The generator put the
    generated amount there, units took the taken amount, and the stock is
    what the engine computes from the two.

    The holder names the faction that holds the ground. It is absent for
    ground that nobody holds.

    **This report names no unit.** A count of the units on a tile comes from
    the derived bridge, which answers after a step. Ask window_census for it.
    """

    world: str
    tick: int
    q: int
    r: int
    kind: int
    passable: bool
    capacity: int
    stock: list[int]
    generated: list[int]
    taken: list[int]
    value: int
    holder: int | None


@dataclass(frozen=True)
class CensusReport:
    """What one window of the world holds.

    The engine walks the window and answers once. This server names the
    window and reads no address itself.

    The four corner fields give the window after the engine clipped it to the
    world, so a reader can repeat the count with tile_report one address at a
    time and compare.

    The by_kind field holds one count for each kind of ground, in the order
    of the kind numbering.

    The crowded fields name the address holding the largest number of units.
    They are absent when the window holds no unit.
    """

    world: str
    tick: int
    q: int
    r: int
    radius: int
    first_q: int
    first_r: int
    last_q: int
    last_r: int
    tiles: int
    by_kind: list[int]
    open_tiles: int
    units: int
    crowd_worst: int
    tiles_at_capacity: int
    crowded_q: int | None
    crowded_r: int | None


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


def _optional(value: int | None) -> int | None:
    """Return an engine field that may say nothing, as a number or nothing.

    The engine returns nothing where a value would stand for nothing: a
    ration that never happened, ground that nobody holds, a crowded address
    in a window with no unit. A zero in those places would read as a real
    answer.
    """
    return None if value is None else int(value)


def _addresses(given: list[list[int]]) -> list[tuple[int, int]]:
    """Turn a list of two-number addresses into what the engine takes."""
    pairs: list[tuple[int, int]] = []
    for address in given:
        if len(address) != 2:
            raise ValueError(f"the address {address} is not a q and an r")
        pairs.append((address[0], address[1]))
    return pairs


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
        session = sessions.get(world)
        units = session.world.spawn_soldiers(_addresses(addresses), faction)
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

    @server.tool(
        title="Found a group",
        description=(
            "Runs the engine's own founding for one faction: it surveys, "
            "takes the best place, seats the group and sets the site's "
            "production rate. Returns the site identity and what it chose."
        ),
    )
    def found_group(world: str, group: int = 8, faction: int = 0) -> FoundingReport:
        """Found one group the way the engine founds one.

        This is the whole loop in one call. The survey reads the ground, the
        founding takes the best place the sample offered, it seats the group
        around that place, and it sets what the site earns from the food the
        survey read.

        A site founded any other way earns nothing, because the rate comes
        from the survey. Give the identity this returns to site_economy.
        """
        session = sessions.get(world)
        made = session.world.found_group(group, faction)
        return FoundingReport(
            world=session.name,
            tick=session.world.tick,
            site=int(made["site"]),
            q=int(made["q"]),
            r=int(made["r"]),
            faction=int(made["faction"]),
            seated=int(made["seated"]),
            score=int(made["score"]),
            food=int(made["food"]),
            wood=int(made["wood"]),
            stone=int(made["stone"]),
            open_ground=int(made["open_ground"]),
            room=int(made["room"]),
            water_edge=int(made["water_edge"]),
            drawn=int(made["drawn"]),
            considered=int(made["considered"]),
            tiles_read=int(made["tiles_read"]),
            site_count=session.world.settlement_count,
            soldier_count=session.world.soldier_count,
        )

    @server.tool(
        title="Read a founding survey",
        description=(
            "Returns the places a founding would compare for a group of the "
            "given size, best first, with the counts that made each score."
        ),
    )
    def founding_survey(
        world: str, group: int = 8, faction: int = 0, limit: int = DEFAULT_MAX_ROWS
    ) -> SurveyReport:
        """Read what a founding would see, without founding anything.

        The survey draws a fixed number of places and reads a fixed number of
        tiles around each one. Neither number follows the size of the world.

        The call writes nothing. It is the answer to why a place is good.
        """
        if limit < 0:
            raise ValueError(f"limit is {limit}; it must be 0 or more")
        session = sessions.get(world)
        columns = session.world.founding_survey(group, faction)
        total = len(columns["q"])
        head = min(limit, total)
        rows = [
            SurveyCandidate(
                q=int(columns["q"][row]),
                r=int(columns["r"][row]),
                score=int(columns["score"][row]),
                food=int(columns["food"][row]),
                wood=int(columns["wood"][row]),
                stone=int(columns["stone"][row]),
                open_ground=int(columns["open_ground"][row]),
                room=int(columns["room"][row]),
                water_edge=int(columns["water_edge"][row]),
                eligible=bool(columns["eligible"][row]),
                separated=bool(columns["separated"][row]),
            )
            for row in range(head)
        ]
        return SurveyReport(
            world=session.name,
            tick=session.world.tick,
            group=group,
            faction=faction,
            drawn=int(columns["drawn"]),
            considered=int(columns["considered"]),
            tiles_read=int(columns["tiles_read"]),
            rows_returned=head,
            truncated=head < total,
            candidates=rows,
        )

    @server.tool(
        title="Read the region a tile sits in",
        description=(
            "Returns the level 1 summary of the cell covering one tile. "
            "Every field is an exact integer total over the tiles of the cell."
        ),
    )
    def region_summary(world: str, q: int, r: int) -> RegionReport:
        """Read the level 1 cell that covers one address.

        Level 0 is the only truth. This level is derived from it, and the
        totals combine exactly, so adding the tiles of the cell gives these
        numbers back.
        """
        session = sessions.get(world)
        summary = session.world.region_summary(q, r)
        return RegionReport(
            world=session.name,
            tick=session.world.tick,
            q=q,
            r=r,
            tiles=int(summary["tiles"]),
            open_tiles=int(summary["open_tiles"]),
            units=int(summary["units"]),
            held_tiles=int(summary["held_tiles"]),
            value_total=int(summary["value_total"]),
            height_total=int(summary["height_total"]),
            food_total=int(summary["food_total"]),
        )

    @server.tool(
        title="Read what a site holds",
        description=(
            "Returns the store, the production rate, the upkeep rate and the "
            "last ration of one site. The site is an identity from found_sites."
        ),
    )
    def site_economy(world: str, site: int, commodity: int = 0) -> SiteEconomyReport:
        """Read the economy of one site.

        The store fills at the production rate and empties at the upkeep
        rate. A site whose store cannot serve its cohorts rations them, and
        the last shortfall is in this report.
        """
        session = sessions.get(world)
        report = session.world.site_economy(site, commodity)
        return SiteEconomyReport(
            world=session.name,
            tick=session.world.tick,
            site=site,
            q=int(report["q"]),
            r=int(report["r"]),
            faction=int(report["faction"]),
            commodity=int(report["commodity"]),
            store=int(report["store"]),
            production=int(report["production"]),
            upkeep=int(report["upkeep"]),
            rationed=bool(report["rationed"]),
            demanded=_optional(report["demanded"]),
            granted=_optional(report["granted"]),
        )

    @server.tool(
        title="Read why a unit chose",
        description=(
            "Returns every score the engine gave one unit, the value each "
            "option read, the weight it carried and the floor it had to clear."
        ),
    )
    def unit_choice(world: str, unit: int) -> ChoiceReport:
        """Ask the engine why one unit chose what it chose.

        The engine recomputes the answer from the world as it stands. It
        stores no score, so this is not a record of an earlier decision. It
        is what the unit would decide now.
        """
        session = sessions.get(world)
        answer = session.world.explain_choice(unit)
        return ChoiceReport(
            world=session.name,
            tick=session.world.tick,
            unit=unit,
            tile=int(answer["tile"]),
            q=int(answer["q"]),
            r=int(answer["r"]),
            cell=int(answer["cell"]),
            need=int(answer["need"]),
            scores=[int(value) for value in answer["scores"]],
            fields=[int(value) for value in answer["fields"]],
            weights=[int(value) for value in answer["weights"]],
            floor=int(answer["floor"]),
            best=int(answer["best"]),
            best_name=answer["best_name"],
            intent=int(answer["intent"]),
            chooses_next_frame=bool(answer["chooses_next_frame"]),
        )

    @server.tool(
        title="Read a tile",
        description=(
            "Returns the ground, the capacity, the holder and the resource "
            "stock of one tile. It names no unit; ask window_census for those."
        ),
    )
    def tile_report(world: str, q: int, r: int) -> TileReport:
        """Read what one address holds.

        The stock of a tile is what the generator put there less what units
        took from it. The report gives all three numbers, so a reader can
        tell an untouched tile from a worked one.
        """
        session = sessions.get(world)
        report = session.world.tile_report(q, r)
        return TileReport(
            world=session.name,
            tick=session.world.tick,
            q=int(report["q"]),
            r=int(report["r"]),
            kind=int(report["kind"]),
            passable=bool(report["passable"]),
            capacity=int(report["capacity"]),
            stock=[int(value) for value in report["stock"]],
            generated=[int(value) for value in report["generated"]],
            taken=[int(value) for value in report["taken"]],
            value=int(report["value"]),
            holder=_optional(report["holder"]),
        )

    @server.tool(
        title="Count what a window holds",
        description=(
            "Returns the ground census and the crowding counts of a square "
            "window of the world. The engine walks the window and answers once."
        ),
    )
    def window_census(world: str, q: int, r: int, radius: int = 8) -> CensusReport:
        """Count the ground and the units of one window.

        The window is the square of the radius around the address, clipped to
        the world. The engine refuses a radius above its ceiling, because a
        call that could name the whole world would be a pass over the world.

        The unit counts come from the bridge that rebuilds at the frame
        barrier. Step the world after you change the population, or the call
        refuses rather than answer from a stale bridge.
        """
        if radius < 0:
            raise ValueError(f"radius is {radius}; it must be 0 or more")
        session = sessions.get(world)
        counted = session.world.window_census(q, r, radius)
        return CensusReport(
            world=session.name,
            tick=session.world.tick,
            q=q,
            r=r,
            radius=radius,
            first_q=int(counted["first_q"]),
            first_r=int(counted["first_r"]),
            last_q=int(counted["last_q"]),
            last_r=int(counted["last_r"]),
            tiles=int(counted["tiles"]),
            by_kind=[int(value) for value in counted["by_kind"]],
            open_tiles=int(counted["open_tiles"]),
            units=int(counted["units"]),
            crowd_worst=int(counted["crowd_worst"]),
            tiles_at_capacity=int(counted["tiles_at_capacity"]),
            crowded_q=_optional(counted["crowded_q"]),
            crowded_r=_optional(counted["crowded_r"]),
        )

    return server


def main() -> None:
    """Run the server over standard input and output."""
    build_server().run(transport="stdio")
