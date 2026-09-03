"""Cachette, a deterministic world simulation engine.

Python is the control plane. Python is not the data plane. Python builds a
selector and sends one command. Rust resolves the selector and runs the
verb. Python must not loop over entities of the mass tier.

That last sentence is a rule, and this package does not enforce it. No
type here refuses a loop, and the tier a shape declares reaches no code
outside the core crate. The record that states the enforcement says the
same, so a reader does not conclude that the package refuses something.

This package is a stub. It re-exports the compiled module. The selector
API, the verb API and the view scope are not written yet.

References
----------
ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``

ADR-0043, a declared tier enforces the no-loop rule, and the API refuses
the loop, decision D5.
``docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md``
"""

from cachette._core import (
    CachetteError,
    Camera,
    ConfigError,
    DeterminismError,
    EnginePanic,
    FrameError,
    SelectorError,
    StepError,
    VerbError,
    ViewError,
    World,
    version,
)

__all__ = [
    "CachetteError",
    "Camera",
    "ConfigError",
    "DeterminismError",
    "EnginePanic",
    "FrameError",
    "SelectorError",
    "StepError",
    "VerbError",
    "ViewError",
    "World",
    "__version__",
    "version",
]

__version__: str = version()
