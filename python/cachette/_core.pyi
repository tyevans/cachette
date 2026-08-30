"""Type stubs for the compiled extension module.

The contributing guide requires that continuous integration checks the stubs. The
build regenerates them and the job fails when the result differs from this
file. Hand-write the parts that a generator cannot infer.
"""

import numpy as np
import numpy.typing as npt

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

class World:
    """A simulated world."""

    def __init__(
        self,
        tile_count: int = ...,
        seed: int = ...,
        faction_count: int = ...,
    ) -> None: ...
    @property
    def tick(self) -> int: ...
    @property
    def tile_count(self) -> int: ...
    @property
    def event_count(self) -> int: ...
    def state_hash(self) -> int: ...
    def check_invariants(self) -> bool: ...
    def step(self, threads: int) -> int: ...
    def event_log_bytes(self) -> bytes: ...
    def tile_values(self) -> npt.NDArray[np.int32]: ...

def version() -> str: ...
