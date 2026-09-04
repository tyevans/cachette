"""Type stubs for the compiled extension module.

This file is hand-written. **No generator produces it and no job compares it
against the module**, so a signature here can disagree with the module and
nothing fails. An earlier version of this docstring claimed the opposite, and a
finding records the correction.[^1] A backlog item holds the generator and the
check.[^2]

**This file states types. It does not state prose.** A member that the compiled
module provides carries its prose in the Rust doc comment, and the published
reference is generated from the module rather than from this file.[^3] A
declaration here that the module does not provide, such as a typed dictionary
that describes a returned mapping, has no other home and carries its own prose.

Every docstring that copied the Rust source is gone. The prose that stays
below belongs to a declaration the compiled module does not provide, and a
finding records what the copies were.[^4]

References
----------
[^1]: Findings register, FND-320.
``docs/FINDINGS.md``

[^2]: Backlog item 0307, generate the type stub from the compiled module.
``docs/backlog/proposed/0307-generate-the-type-stub-from-the-compiled-module.md``

[^3]: ADR-0107, the Python reference is generated from the compiled module,
decisions D2 and D3.
``docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md``

[^4]: Findings register, FND-321.
``docs/FINDINGS.md``
"""

from collections.abc import Sequence
from typing import TypedDict

import numpy as np
import numpy.typing as npt

# What a verb accepts where it names units.
#
# The engine hands identities back as a column, and a caller passes that
# column straight to the next verb. A caller that built its own list of
# identities passes that. Both are one crossing, so both are allowed.
Identities = Sequence[int] | npt.NDArray[np.uint64]

class TileChangedColumns(TypedDict):
    """One column for each field of the tile change event.

    The names are the field names of the event in the Rust source. A reader
    takes a field by its name, so it holds no byte offset and no field
    order.

    The value column carries the fixed-point value as its raw integer. It is
    never a floating point number.
    """

    tick: npt.NDArray[np.uint64]
    tile: npt.NDArray[np.uint32]
    value: npt.NDArray[np.int32]
    holder: npt.NDArray[np.uint16]
    kind: npt.NDArray[np.uint8]

class ResourceTakenColumns(TypedDict):
    """One column for each field of the gather event.

    The unit column holds the whole identity of the unit. It is not a slot
    index. Hand a value from it back to ``World.soldier_tile``.
    """

    tick: npt.NDArray[np.uint64]
    unit: npt.NDArray[np.uint64]
    tile: npt.NDArray[np.uint32]
    amount: npt.NDArray[np.uint32]
    kind: npt.NDArray[np.uint8]

class PositionColumns(TypedDict):
    """One column for each field of a position at a site.

    The columns hold the positions of the site and nothing else. An entry of
    the storage that is no position does not appear.

    The kind column holds the number of the kind of work. The holder column
    holds the whole identity of the unit that holds each position, and zero
    where a position holds nobody. It is not a slot index.
    """

    kind: npt.NDArray[np.uint8]
    rank: npt.NDArray[np.uint8]
    holder: npt.NDArray[np.uint64]

class FactionUnitColumns(TypedDict):
    """One column for each field of a live soldier of one faction.

    The engine builds the set at the moment of the call, so every entry names
    a live soldier and no entry stands for nothing.

    The unit column holds the whole identity of each soldier. It is not a slot
    index. The tile column holds the row-major tile index that each soldier
    stands on.
    """

    unit: npt.NDArray[np.uint64]
    tile: npt.NDArray[np.uint32]

class FoundingColumns(TypedDict):
    """What one founding chose, and what it made.

    The site is the whole identity of the settlement the founding seated. It
    is not a slot index.

    The counts are the ones the survey read at the chosen place, and they are
    whole numbers. The score is the engine's own weighted sum of them, as a
    Q16.16 value in its raw integer.

    The seated entry is how many people the founding seated. It is not a
    flag. The report of ``World.found_run_for_every_faction`` uses the same
    key for a ``bool``.
    """

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

class SurveyColumns(TypedDict):
    """What a founding survey read, one column for each candidate place.

    The rows are the candidates in the order the founding ranks them, best
    first. Row zero is the place a founding would take.

    The score column holds the engine's own weighted sum of the counts beside
    it. It is a Q16.16 value as its raw integer, and never a floating point
    number. The counts beside it are whole numbers.

    The three trailing entries are scalars. The survey counts them as it
    reads, so they measure the run rather than restate the sample size.
    """

    q: npt.NDArray[np.int32]
    r: npt.NDArray[np.int32]
    score: npt.NDArray[np.int64]
    food: npt.NDArray[np.uint32]
    wood: npt.NDArray[np.uint32]
    stone: npt.NDArray[np.uint32]
    open_ground: npt.NDArray[np.uint32]
    room: npt.NDArray[np.uint32]
    water_edge: npt.NDArray[np.uint32]
    eligible: npt.NDArray[np.uint8]
    separated: npt.NDArray[np.uint8]
    drawn: int
    considered: int
    tiles_read: int

class RegionSummary(TypedDict):
    """The level 1 summary of one cell.

    Every field is an exact integer total over the level 0 tiles of the cell,
    and none is a floating point number.

    The value total and the height total are Q16.16 values as their raw
    integers. The food total is a whole count of units of stock. A reader that
    scales all three reports a food total 65536 times too small.
    """

    tiles: int
    open_tiles: int
    units: int
    held_tiles: int
    value_total: int
    height_total: int
    food_total: int

class SiteEconomy(TypedDict):
    """What one site earns, holds and owes, for one commodity.

    The store, the production and the upkeep are Q16.16 values as their raw
    integers.

    The demanded and granted entries hold the last ration the site could not
    serve in full, and they are ``None`` when it served every cohort.
    """

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

class ChoiceReport(TypedDict):
    """Why one unit chose what it chose.

    The need, every score, every field value, every weight and the floor are
    Q16.16 values as their raw integers.

    The best entry names the option the scores select, or the no-intent value
    when every score is below the floor. The name entry is ``None`` for a
    hold.
    """

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

class TileReport(TypedDict):
    """What one tile holds.

    The stock, generated and taken entries hold one amount for each kind of
    resource, in the order of the kind numbering. The stock is the generated
    amount less what units took.

    The holder names the faction that holds the ground, and it is ``None``
    for ground that nobody holds.

    The upgrade entry names the upgrade the tile carries, finished or under
    construction, and it is ``None`` for a tile that carries none.
    """

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
    upgrade: int | None
    upgrade_progress: int
    upgrade_complete: bool

class WindowCensus(TypedDict):
    """What one window of the world holds.

    The four corner entries give the window after the engine clipped it to
    the world, so a reader can repeat the count one address at a time.

    The by_kind entry holds one count for each kind of ground, in the order
    of the kind numbering.

    The crowded entries name the address that holds the largest number of
    units, and they are ``None`` when the window holds no unit.
    """

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

class FoundingReport(TypedDict, total=False):
    """What one faction got when the run was founded."""

    faction: int
    seated: bool
    q: int
    r: int
    people: int
    considered: int
    food: int
    wood: int
    stone: int
    open_ground: int
    water_edge: int
    carries_its_group: bool
    refusal: str

class FrameReading(TypedDict):
    """What the drawing pass read while it filled a frame."""

    tick: int
    tiles_painted: int
    soldiers_painted: int
    soldiers_live: int
    sites_held: int
    seats: int
    seats_taken: int
    characters: int
    promoted_now: int
    promoted_deeds: int | None
    newest_character: tuple[int, int] | None
    panel_height: int
    units_short: int
    units_carrying: int
    carried_by_kind: list[int]
    units_housed: int
    sites_rationed: int
    # Fixed point at a scale of 65536, not a count of goods.
    rationed_short_accum: int
    tiles_at_capacity: int
    crowd_worst: int
    centre: tuple[int, int]
    extent_shown: tuple[int, int]
    step_mean_micros: float
    draw_mean_micros: float
    ticks_each_second: float

class CachetteError(Exception): ...
class StepError(CachetteError): ...
class SelectorError(CachetteError): ...
class VerbError(CachetteError): ...
class ViewError(CachetteError): ...
class DeterminismError(CachetteError): ...
class EnginePanic(CachetteError): ...
class FrameError(CachetteError): ...
class ConfigError(CachetteError): ...

class Camera:
    def __init__(self, tile_size: float | None = ...) -> None: ...
    @staticmethod
    def fitting(world: World, width: int, height: int) -> Camera: ...
    tile_width: float
    tile_height: float
    origin_x: float
    origin_y: float
    def nudge(self, across: float, down: float, width: int, height: int) -> None: ...
    def pan(self, across: float, down: float) -> None: ...
    def zoom_in(self, width: int, height: int) -> None: ...
    def zoom_out(self, width: int, height: int) -> None: ...
    def look_at(self, q: int, r: int, width: int, height: int) -> None: ...
    def clamp(self, world: World, width: int, height: int) -> None: ...
    def tile_at(self, x: float, y: float) -> tuple[int, int]: ...

class World:
    def __init__(
        self,
        width: int = ...,
        height: int = ...,
        seed: int = ...,
        faction_count: int = ...,
    ) -> None: ...
    @property
    def tick(self) -> int: ...
    @property
    def tile_count(self) -> int: ...
    @property
    def width(self) -> int: ...
    @property
    def height(self) -> int: ...
    @property
    def event_count(self) -> int: ...
    def state_hash(self) -> int: ...
    def check_invariants(self) -> bool: ...
    def step(self, threads: int) -> int: ...
    def found_run_for_every_faction(self, group: int = ...) -> list[FoundingReport]: ...
    def draw(
        self,
        camera: Camera,
        width: int,
        height: int,
        pixels: npt.NDArray[np.uint32],
        reference: bool = ...,
        panel: bool = ...,
        panels: Sequence[str] | None = ...,
        pointer: tuple[int, int] | None = ...,
    ) -> FrameReading: ...
    @staticmethod
    def panel_names() -> list[str]: ...
    def faction_population(self) -> list[int]: ...
    def presence_masks(self) -> npt.NDArray[np.uint64]: ...
    def stands_in_territory(self, guest: int, host: int) -> bool: ...
    def tile_holders(self) -> npt.NDArray[np.uint16]: ...
    @property
    def gather_count(self) -> int: ...
    @property
    def soldier_count(self) -> int: ...
    def event_log_bytes(self) -> bytes: ...
    def event_log_columns(self) -> TileChangedColumns: ...
    def gather_log_columns(self) -> ResourceTakenColumns: ...
    def tile_values(self) -> npt.NDArray[np.int32]: ...
    def spawn_soldiers(
        self, addresses: Sequence[tuple[int, int]], faction: int
    ) -> npt.NDArray[np.uint64]: ...
    def despawn_soldiers(self, units: Identities) -> None: ...
    def order_gather(self, units: Identities, kind: int) -> None: ...
    def soldier_tile(self, unit: int) -> int: ...
    def order_build(self, units: Identities, kind: int) -> None: ...
    def stop_build(self, units: Identities) -> None: ...
    def build_order(self, unit: int) -> int | None: ...
    def destroy_upgrades(self, addresses: Sequence[tuple[int, int]]) -> int: ...
    def return_direction(self, faction: int, q: int, r: int) -> int | None: ...
    @staticmethod
    def direction_offsets() -> list[tuple[int, int]]: ...
    def send_units_to(
        self,
        units: Identities,
        seeds: Sequence[tuple[int, int]],
        destination: int = ...,
    ) -> None: ...
    def stop_sending(self, units: Identities) -> None: ...
    @property
    def destination_count(self) -> int: ...
    def set_destination_count(self, count: int) -> None: ...
    def faction_units(self, faction: int) -> FactionUnitColumns: ...
    @property
    def settlement_count(self) -> int: ...
    def found_settlements(
        self, addresses: Sequence[tuple[int, int]], faction: int
    ) -> npt.NDArray[np.uint64]: ...
    def prefer_at_sites(self, sites: Identities, kind: int, target: int) -> None: ...
    def site_positions(self, site: int) -> PositionColumns: ...
    def site_preference(self, site: int) -> npt.NDArray[np.int32]: ...
    def set_position_schedule(self, period: int, phase: int) -> None: ...
    def found_group(self, group: int, faction: int) -> FoundingColumns: ...
    def founding_survey(self, group: int, faction: int) -> SurveyColumns: ...
    def region_summary(self, q: int, r: int) -> RegionSummary: ...
    def site_economy(self, site: int, commodity: int = ...) -> SiteEconomy: ...
    def explain_choice(self, unit: int) -> ChoiceReport: ...
    def tile_report(self, q: int, r: int) -> TileReport: ...
    def window_census(self, q: int, r: int, radius: int = ...) -> WindowCensus: ...

def version() -> str: ...
