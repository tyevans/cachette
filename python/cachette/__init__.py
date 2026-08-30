"""Cachette, a deterministic world simulation engine.

Python is the control plane. Python is not the data plane. Python builds a
selector and sends one command. Rust resolves the selector and runs the
verb. Python never loops over entities of the mass tier.

This package is a stub. It re-exports the compiled module. The selector
API, the verb API and the view scope are not written yet.

References
----------
ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/REGISTRY.md``
"""

from cachette._core import (
    CachetteError,
    DeterminismError,
    EnginePanic,
    SelectorError,
    StepError,
    VerbError,
    ViewError,
    World,
    version,
)

__all__ = [
    "CachetteError",
    "DeterminismError",
    "EnginePanic",
    "SelectorError",
    "StepError",
    "VerbError",
    "ViewError",
    "World",
    "__version__",
    "version",
]

__version__: str = version()
