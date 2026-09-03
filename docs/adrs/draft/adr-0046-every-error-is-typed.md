# ADR-0046: Every error is typed

## Context

The engine refuses things. A world whose extent is empty is not a world. A step
at zero threads is not a step. A verb given an address the world rejects cannot
run. An identity that names a dead entity does not resolve.[^1]

Each refusal has to reach the control plane as something a caller can act on.
The default path does not do that. A conversion that maps every Rust error to
one general exception type is one line, it compiles, and it is what happens if
nobody decides otherwise. A caller then cannot tell a configuration mistake
from a refused verb without reading the message text, and message text is not
an interface.

The interpreter binding also constrains how the types are built. Deriving a
Python exception class from another Python class under the stable application
binary interface needs a later interpreter than the project sets as its floor.
Building the types with the binding library's own macro does not.[^2]

## Decision

### D1. One root type holds every error the engine raises

Every exception the engine raises derives from one root type that belongs to
this project. A caller catches the root to catch everything the engine can
raise, and catches a leaf to catch one kind.

Nothing the engine raises sits outside the root.

### D2. The engine raises no general-purpose exception

A refusal never arrives as the interpreter's own general error type. There is
no catch-all conversion, because a catch-all conversion is what collapses the
hierarchy back to one type while every leaf still exists and looks used.

This is the decision that is easiest to break by accident, and it breaks
silently: the code compiles, the test that expects a failure still passes, and
only a caller that wanted to distinguish two failures notices.

### D3. A kind of refusal gets a type, not a message

Which type is raised carries the meaning. The message says what was wrong with
this call; the type says what class of thing went wrong.

A caller that has to match on message text is a caller the interface failed.

### D4. An error names the thing that was wrong

A message names the value that was refused, not only that something was
refused. An address the world rejected is named in the message that reports the
rejection.

This is checkable and a test can assert it, which is the reason it is a
decision rather than a style note.

### D5. A panic that reaches the boundary becomes an exception, and the build must permit that

A Rust panic that unwinds across the foreign function boundary is undefined
behaviour. It is caught at the boundary and converted, so the release build must
not abort on a panic. A build that aborts gives up every panic message and
breaks this decision.

The type for it is part of the hierarchy of D1, so a caller catching the root
catches it too.

## The alternatives this rejects

**One exception type for the whole engine.** The project rejects it because a
caller can then only catch everything, and every distinction lives in text.

**The interpreter's own exception types, chosen to fit each case.** A
configuration error raises a value error, a refused verb raises a runtime
error. The project rejects this because a caller catching a value error would
also catch one raised by unrelated code in its own program, and because the
engine would have no root to catch.

**Deriving the types from a Python base class.** The project rejects this
because it needs a later interpreter than the floor this project supports, and
the macro form does not.[^2]

**Returning a status value instead of raising.** The project rejects this for a
programming error, which is what these are. A refusal that is part of normal
operation, such as a command that could not apply to every member of a set, is
a different thing and is not settled by this record.

## Consequences

**A new failure mode needs a type or an existing one.** A contributor adding a
refusal chooses which leaf it is, and adding a leaf is adding it under the root
of D1. There is no correct way to add a refusal that sits outside.

**A caller can write a narrow handler and it will keep working.** The type is
the contract, so a message may be improved without breaking a caller.

**Several declared types have no raise site today.** The hierarchy names types
for a selector, for a determinism defect and for a panic that reached the
boundary, and nothing raises any of them, because no selector exists, no
determinism check reports through the boundary, and no panic hook is installed.
A reader must not take the presence of a type as evidence that the engine
produces it. This is the shape this project keeps meeting, and it is recorded
here rather than left for a reader to discover.[^3]

**D5 constrains the release profile.** Aborting on panic is not available while
this record stands, and a change to the profile that enables it breaks this
record rather than merely changing a setting.

**Nothing enforces D2.** No check looks for a general exception raised from the
bindings. A reviewer looks for it, and a contributor who adds one gets no
failure.

## References

[^1]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^2]: Report 05, the Rust and Python boundary, error mapping. `docs/research/reports/05-rust-python-boundary.md`
[^3]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
