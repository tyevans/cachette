"""Type stubs for the compiled extension module.

The contributing guide requires that continuous integration checks the stubs. The
build regenerates them and the job fails when the result differs from this
file. Hand-write the parts that a generator cannot infer.
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

class CachetteError(Exception):
    """The root of every Cachette error."""

class StepError(CachetteError):
    """A step refused to run."""

class SelectorError(CachetteError):
    """A selector was not valid."""

class VerbError(CachetteError):
    """A verb refused a command."""

class ViewError(CachetteError):
    """A view was stale or out of scope."""

class DeterminismError(CachetteError):
    """The engine detected a determinism defect."""

class EnginePanic(CachetteError):
    """A Rust panic reached the boundary."""

class ConfigError(CachetteError):
    """The world settings do not describe a world."""

class World:
    """A simulated world."""

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

def version() -> str: ...
