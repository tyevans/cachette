# ADR-0163: An event declares its layout once and the binding derives every column

## Context

An event type in this engine is plain data. It has an explicit layout, it
declares every padding byte, and it holds no boolean field.[^1] The engine
keeps one append-only array for each event type.[^2]

The control plane must read an event. Python is a control plane and not a data
plane, so it never walks the records itself.[^3] The binding layer therefore
hands Python one array for each field, and the keys are the field names. A
reader in Python holds no byte offset, no field width and no field order.

That answered the first half of the problem and left the second half open. The
binding named the fields itself. Each log had a function that listed the field
of the event, one line for each column, and the type stub of the module listed
them again. One fact, the field set of an event, was written in three places.

**Nothing failed when the copies disagreed.** A contributor who added a field
to an event compiled a green tree. The new field reached nobody, the reader saw
the old shape, and no test said so. That is the defect shape this project has
already met, and the rule that names it names this exact pair as a place it
would recur.[^4]

The reader must also never receive a floating point number. A fixed-point field
crosses as its raw integer, because a float at the interface is the same defect
as a float in the state, one layer further out.[^5]

## Decision

### D1. An event names its own fields, once, and nothing else names them

**Each event type carries one declaration of its fields, in order, and that
declaration is the only place in this project that names them.** The
declaration gives the name of each field and the name of the column it crosses
in. A padding field is declared and crosses nowhere.

The declaration states no offset and no width. The compiler reads both from the
type. A width therefore cannot disagree with the field it describes, because no
one writes the width.

The alternative is what the project had: a hand-written field list in the
binding for each log. It is rejected because it is a second declaration site
with no check, and because the failure it produces is silent.

### D2. Every reader of an event derives its view from that declaration

**The binding builds a column for a field because the declaration names it, and
never because a function named it.** No function in the binding layer, in the
control plane package or in the agent-facing server names a field of an event.

A tool that reports rows to a reader builds a row from the keys the engine
gave. A type stub that describes a column mapping is written from the
declaration by a script, and a test runs that script in check mode. The
published reference already comes from the compiled module, so the prose and
the types now have one origin.[^6]

A column that the event does not hold is allowed beside the derived ones, and
it must be a value the engine computes rather than a field it stores. The
address of a tile is the example: the event holds an index, and the grid owns
the rule that turns an index into an address.

### D3. A test fails when the declaration does not cover the type

**The declared fields must start at the first byte of the type, follow one
another with no gap, and end at its last byte.** A test asserts this for every
declared event.

This is the check that the compiler cannot make. A field added to an event and
left out of the declaration compiles, and it leaves the declaration short of the
last byte. The test then names the event and says how many bytes the
declaration covers.

The alternative is a compile-time assertion. It is rejected because the failure
must be readable. A contributor who adds a field needs a message that says what
to do, not a type error inside a macro.

### D4. No column crosses as a floating point type

**Every column element type is an integer.** A fixed-point field crosses as its
raw integer, and the reader scales it if it wants a quantity.[^5]

The declaration cannot express a floating point column, because the field types
that a column may have are the integer types and the newtypes over them.

## Consequences

A field added to an event reaches the control plane without an edit to the
binding. A field renamed there renames the column, and a reader that used the
old name fails at the point of the read.

A rename is therefore a breaking change to the control plane, and it must be
treated as one. Before this record a rename in the engine changed nothing
outside it, because the binding held its own name for the field. That
insulation is gone, and it was hiding the disagreement rather than preventing
it.

The declaration is one more thing to write when an event type is added. An
event with no declaration crosses to nobody, so the cost lands on the author of
the event and not on a later reader.

The engine can now report its own field set to the control plane. A reader that
builds a type from that report holds no copy of the layout. A reader that
writes one anyway is a defect that the review must catch, because no check can
see a copy that never disagrees.

The type stub of the compiled module is no longer wholly hand-written. Only the
column classes generate. The rest of the stub keeps the property that a
signature there can disagree with the module and nothing fails, and a finding
records that.[^7]

## References

[^1]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^2]: ADR-0031, events live in type-segregated arenas of plain data. `docs/adrs/REGISTRY.md`
[^3]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^4]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: ADR-0107, the Python reference is generated from the compiled module, decisions D2 and D3. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^7]: Findings register, FND-320. `docs/FINDINGS.md`
