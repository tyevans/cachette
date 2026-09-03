"""Cachette, a deterministic world simulation engine.

Python is the control plane. Python is not the data plane. Python builds a
selector and sends one command. Rust resolves the selector and runs the
verb. Python must not loop over entities of the mass tier.[^1]

That last sentence is a rule, and this package does not enforce it. No
type here refuses a loop, and the tier a shape declares reaches no code
outside the core crate. The record that states the enforcement says the
same, so a reader does not conclude that the package refuses
something.[^2]

This package re-exports the compiled module. Every name in ``__all__``
comes from there, and the published reference states what each one does.

This docstring does not list what the package holds, and it does not say
which part of the interface is written. A list here would be a second
place that states the interface, and nothing would fail when the two
disagreed. That has already happened: this docstring said the verb
interface was not written while a verb ran.[^3] The reference is
generated from the compiled module itself, so it reports what the module
holds rather than what a writer remembered.[^4]

References
----------
[^1]: ADR-0040, Python is a control plane, not a data plane.
``docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md``

[^2]: ADR-0043, a declared tier enforces the no-loop rule, and the API
refuses the loop, decision D5.
``docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md``

[^3]: Findings register, FND-319.
``docs/FINDINGS.md``

[^4]: ADR-0107, the Python reference is generated from the compiled
module, decision D1.
``docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md``
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
