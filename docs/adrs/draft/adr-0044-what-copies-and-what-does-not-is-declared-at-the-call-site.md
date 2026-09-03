# ADR-0044: What copies and what does not is declared at the call site

## Context

The control plane reads values out of the engine. It reads a tile column, a
log of what happened in a frame, and the identities a verb produced.[^1]

Two shapes are available for such a read. The engine can copy the values into
memory the array library owns, or it can hand out an array that points at
engine memory and copies nothing. The second is faster and it is the one a
reader assumes when nobody says otherwise, because "the engine returns a NumPy
array" sounds like one thing.

They are not one thing, and the difference is not visible at the call site. An
array that points at engine memory stays valid only while the engine leaves
that memory alone. The engine moves it: a column grows, a dead entity's slot
takes a living one, a frame clears a log. The array then reads memory that
holds something else, and it reads it without failing. Silent wrong data is the
worst outcome available here, because nothing reports it and a reader trusts
the number.

The storage shape decides what is even possible. A whole column of one field is
one contiguous run, so a borrow of it is at least expressible. A selected
subset is not contiguous, so no borrow of it exists at any price and the values
must be gathered somewhere.[^2]

## Decision

### D1. Every read that crosses states whether it copies, in its own documentation

A method that returns values says, in its own first lines, whether the caller
received a copy or a borrow. It is not stated once in a guide and assumed
everywhere else.

The statement is part of the method. A reader who never opens the guide still
gets it, and a method that gains a second shape gains a second statement.

### D2. A read that copies is the default, and it is not called zero-copy

Where the engine copies, the documentation says it copies. The project does not
describe a gather or a conversion as zero-copy because the result is a NumPy
array.

This is the decision that costs something and it is the one worth keeping. A
project that overstates one read teaches its readers to disbelieve the
statement on every read.

### D3. A borrow is a separate, differently named method, never a faster version of the same one

A method never changes from copying to borrowing. The borrowing form is its own
method, with its own name, and its documentation states the lifetime the caller
must respect.

A caller therefore opts into the hazard by naming it. A caller who did not name
it cannot be surprised by it, and an engine change cannot turn a safe call into
an unsafe one.

### D4. A subset is gathered, and the gather is a copy

A selected subset is not contiguous, so the engine gathers the selected values
into a buffer and returns that.[^2] The gather is one copy and the record calls
it one.

The alternative is to return the whole column and an index array and to make
the caller gather. That is the same copy, moved to the side of the boundary
that is slower at it, and it is a loop over the population.[^1]

## The alternatives this rejects

**Borrow everything, and document the rules.** The engine would hand out arrays
that point at its memory, with a rule about when they expire. The project
rejects this because the failure is silent: the rule is invisible at the read,
and breaking it returns a number rather than an error.

**Copy everything, and offer no borrow at all.** This is safe and it is close to
where the project stands today. The project does not close the door, because a
whole tile column is the one case where a borrow is genuinely free and the
research names it as the honest example to lead with.[^3] D3 keeps that door
open with a lock on it.

**One method with a flag that selects copy or borrow.** The project rejects this
because the hazard would then depend on an argument, so a reader of a call site
would have to read the argument to know whether the result outlives the next
step.

**State it once in the package documentation.** The project rejects this for the
reason D1 gives. A reader arrives at a method, not at a guide.

## Consequences

**Every read the engine returns today is a copy, and every one of them says
so.** That is the current state and it is what the documentation claims. A
reader can take the statement at face value, which is the property this record
is buying.

**The project pays a copy on every subset read.** D4 makes it explicit rather
than avoidable. At the target scale that copy is real work, and no measurement
of it exists on the target platform.[^4]

**A borrow, if one is added, is a new name and a new record.** D3 means the
borrowing form arrives as its own method with its own safety statement, and the
lifetime rules that go with it are not in this record.

**A method that gains a copy or loses one is a documentation change too.** The
statement and the behaviour are two sites for one fact, and nothing fails when
they disagree.[^5] A reviewer checks the statement against the body, and this
record is what makes that check part of the review.

## References

[^1]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^2]: ADR-0012, tiles are dense columns and units are a generational arena, decisions D2 and D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^3]: Report 05, the Rust and Python boundary, zero-copy data exchange. `docs/research/reports/05-rust-python-boundary.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
