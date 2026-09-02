# ADR-0031: Events live in type-segregated arenas of plain data

## Context

The engine records what happened in a frame. A watcher reads that record, and
the two determinism tests compare it byte for byte between two runs at
different thread counts.[^1]

An event is already plain data. The type declares a C layout, declares every
padding byte, and holds no boolean, so the bytes of an event are the bytes the
program wrote.[^2] That record fixes the shape of one event. It does not say
what holds the events.

The usual answer holds them together. A log is one growable sequence of a
polymorphic handle, or one wide enumeration with a variant for each kind. Both
put every kind of event in one place, and both are what a contributor reaches
for, because both let one loop walk the whole log.

The scale is what makes the usual answer wrong here. The world holds far more
tiles and units than a single sequence of handles can carry, and the scale
constants table holds the figures.[^3] A handle is a pointer, so reading an
event follows it to an address the loop did not choose, and applying a mixed
sequence dispatches on each element. A wide enumeration removes the pointer and
keeps the mixing: every element then costs the size of the largest variant, and
the discriminant is a branch the predictor cannot predict.

No measurement of any of this exists on the target platform.[^4] The argument
below is about the shape of the storage, not about its price.

## Decision

### D1. One event type owns one append-only array

Each event type has its own array. A frame appends to the array of the type it
emits. No array holds two types.

There is no polymorphic container, no boxed event, and no enumeration that
carries a variant for each kind. An event is stored as itself.

### D2. Applying a log is one loop for each type, and never a dispatch

A pass over the log runs one loop for each array. The element type of that
loop is known where the loop is written, so the loop makes no indirect call
and reads no discriminant.

A reader that wants every event of a frame in one order sorts by a stable key
across the arrays.[^5] It does not get that order by mixing the events into one
container.

### D3. An event type is named where the engine is built

Every event type is declared in the engine. Nothing introduces an event type
while the engine runs.

This is the price of D1. An array whose element type is chosen at run time is
the polymorphic container that D1 refuses, under another name.

### D4. The array is the thing that crosses to Python

An event array is already a column of plain data of one type, so it crosses to
the control plane as one column for each field of the type, in one
crossing.[^6] A record that segregates by type therefore also gives the shape
the boundary wants, and neither side converts.

## The alternatives this rejects

**One sequence of boxed events.** This is classic event sourcing, and it is the
first thing a contributor writes. Each event is one allocation, the apply loop
follows a pointer for each element, and the call it makes is indirect. The
project rejects it because every one of those costs scales with the number of
events, and the number of events scales with the population.

The rejection is stated here rather than in a record of its own. It is the
alternative that D1 refuses, and the alternative belongs with the claim that
refuses it.[^7]

**One wide enumeration.** This removes the allocation and keeps the rest. Every
element costs the size of the widest variant, and every apply reads a
discriminant. It also makes the log one array, so adding a large event type
makes every small event larger.

**One array of bytes with a type tag.** This is the enumeration with the type
safety removed. The bytes of the array then have no single type, so the array
cannot cross to Python as a column and cannot be compared as one.

**Splitting every event type into one array for each field.** The project does
not do this by default. A field split serves a reader that wants a subset of
the fields, and an event type is small enough that the split usually costs more
than it saves. Split a type when a reader asks for it.

## Consequences

**A new event type is a new array.** Adding one means adding storage, adding a
clearing step, and adding a column export. That is more work than adding a
variant to an enumeration, and it is the work this record chooses.

**The engine cannot walk the whole log in one loop.** Any question about every
event of a frame, in order, needs the sort of D2. A reader that wants the
frame in one sequence pays for that sequence.

**A plugin cannot add an event type while the engine runs.** D3 forecloses it.
A new event type is a change to the engine.

**The arrays grow rather than being reserved.** The engine appends to a
growable array and clears it, so a frame may still allocate when an array
grows. Reserving each array at construction is an addition this record permits
and does not make. Nothing reserves one today, and no record should say it
does.[^8]

**A log comparison is a comparison of bytes.** Two runs produce the same
arrays, and each array is plain data, so the thread-count test compares them
directly.[^1] It does not have to walk a structure to compare it.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0044, what copies and what does not is declared at the call site, decision D1. `docs/adrs/draft/adr-0044-what-copies-and-what-does-not-is-declared-at-the-call-site.md`
[^7]: Decision Record Scope, section 5. `.claude/rules/adr-scope.md`
[^8]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
