//! The exception classes the module raises.
//!
//! One root class holds the whole hierarchy. The engine never raises a bare
//! runtime error, so a caller catches the root class and catches every
//! refusal this module can make.[^1]
//!
//! The classes sit in one module because they are one hierarchy. A reader who
//! wants to know what a call can raise reads this file and no other.
//!
//! # References
//!
//! [^1]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`

use pyo3::create_exception;
use pyo3::exceptions::PyException;

// ADR-0046: one root exception type holds the whole hierarchy. The
// engine never raises a bare runtime error. The macro builds the types,
// because subclassing a Python class under the stable ABI needs a later
// interpreter version and the macro does not.
create_exception!(
    _core,
    CachetteError,
    PyException,
    "The base class of every error this module raises.

Catch this class to catch every refusal the engine makes. It is a subclass
of the Python built-in `Exception`.

Every other error class in this module is a subclass of this one. The engine
reports every refusal of its own through one of them, and never through a
bare `RuntimeError`.

The binding still refuses an argument before the call reaches the engine, and
it refuses with a built-in class. A wrong argument type raises `TypeError`.
An integer outside the range of the parameter raises `OverflowError`. A
sequence of the wrong shape raises `ValueError`. None of the three is a
subclass of this class."
);
create_exception!(
    _core,
    StepError,
    CachetteError,
    "A step refused to run.

`World.step` raises this class when the thread count is zero. A step needs
at least one thread."
);
create_exception!(
    _core,
    FrameError,
    CachetteError,
    "A frame refused to fill.

`World.draw` raises this class when the pixel array does not match the width
and the height, when a side is zero, when the array is not one contiguous
block, or when the camera draws a tile smaller than one pixel.

An array of the wrong element type raises `TypeError` instead, because the
interpreter refuses the argument before the engine reads it. Build the array
with the `numpy.uint32` element type."
);
create_exception!(
    _core,
    ConfigError,
    CachetteError,
    "The arguments do not describe a world.

The `World` constructor raises this class. A side of zero and a faction count
above 63 are the two cases a caller meets first. The doc comment of the
`World` class names every argument the constructor takes, its default and its
bound, so a caller checks a value before the call."
);
create_exception!(
    _core,
    SelectorError,
    CachetteError,
    "A selector was not valid.

**No call in this module raises this class today.** The module declares it
for the selector interface, and that interface is not written. A finding
records the gap.[^1]

# References

[^1]: Findings register, FND-326. `docs/FINDINGS.md`"
);
create_exception!(
    _core,
    VerbError,
    CachetteError,
    "A verb refused a command.

A verb is a call that changes the world. This class covers a refusal the
engine makes of the command itself. The refusals are:

- a number that names no kind,
- an address the ground refuses,
- a target below zero,
- a radius above the ceiling.

The message names the value the engine refused. The ceiling of a window
census is a radius of 64 tiles.

A verb that takes a set refuses the whole set. It writes nothing and
creates no partial state."
);
create_exception!(
    _core,
    ViewError,
    CachetteError,
    "A view was stale or out of scope.

The class covers two cases. The first is an identity that names no live
entity. This includes an identity the engine gave for an entity that has
since died. The second is an address or a window outside the world.

An identity is stale rather than wrong. The engine compares the generation.
It refuses the dead identity. It never answers for the next occupant of the
slot.[^1]

# References

[^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`"
);
create_exception!(
    _core,
    DeterminismError,
    CachetteError,
    "The engine detected a determinism defect.

**No call in this module raises this class today.** The module declares it
for a check that is not written. Two tests in the Rust workspace hold the
determinism guarantee instead. Neither reports through this class. A finding
records the gap.[^1]

# References

[^1]: Findings register, FND-326. `docs/FINDINGS.md`"
);
create_exception!(
    _core,
    EnginePanic,
    CachetteError,
    "A Rust panic reached the boundary.

**No call in this module raises this class today, and a panic does not
produce it.** The binding library catches a panic and raises its own
`pyo3_runtime.PanicException`. That class is not a subclass of
`CachetteError`. A caller that wants to survive a panic catches
`BaseException`. A finding records the gap.[^1]

# References

[^1]: Findings register, FND-326. `docs/FINDINGS.md`"
);
