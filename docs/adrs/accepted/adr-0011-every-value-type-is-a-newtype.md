# ADR-0011: Every value type is a newtype

Status: Accepted

## Context

The engine passes small integers everywhere. A tile index, an entity slot, a
faction, a tick, a fixed-point value and a wide accumulator are all integers,
and several of them are the same integer type.

A function that takes three bare `u32` arguments accepts them in any order. The
compiler checks nothing, because there is nothing to check: the arguments have
one type. A caller that passes a tile index where an entity slot belongs
compiles, runs, and reads a real entity that is not the one it meant.

The failure is quiet in the way this project cannot tolerate. The wrong value
is a valid value, so no bound is exceeded and nothing panics. The result enters
the state hash, and the hash is the same on every machine and at every thread
count, because the defect is deterministic.[^1]

The cost of the alternative is what usually stops a project doing this. A
wrapper type that is a different size or a different alignment from the value
it wraps cannot be cast in bulk, cannot be written to a buffer as bytes, and
cannot sit in a column that the engine reads as a slice.

## Decision

### D1. A value with a meaning is a newtype, not a bare integer

**Every quantity that means something in the simulation has its own type.** A
tile index, an entity identity, a faction, a tick, a fixed-point value and an
accumulator are each a distinct type, and the compiler refuses one where
another belongs.

A bare integer in a signature is a count, an index into an anonymous array, or
a thread count. It is never a value the simulation gives a name to.

### D2. A newtype is transparent, so it has the size and the alignment of the
value it wraps

**Every value newtype declares `repr(transparent)`.** The type is then exactly
its inner value in memory: same size, same alignment, no padding, no
discriminant.

This is what makes the type free. A column of them is a column of the inner
type. A slice of them casts to bytes. A buffer written from one and read into
the other holds the same bytes.

A wrapper that changed the layout would put a cost on every column in the
engine, and the project would drop the wrappers rather than pay it.

### D3. The layout of a value that crosses a boundary is declared, and its
padding is declared with it

**A type that is written to a buffer, hashed, or handed to another language
declares `repr(C)` and declares its padding as a field.** Undeclared padding
is uninitialised memory, and uninitialised memory in a state hash is a
difference between two runs that nothing caused.[^2]

`repr(transparent)` answers this for a newtype over one integer. A record with
several fields does not get it, and states its layout and its padding
instead.

### D4. A conversion between two value types is written out, never inferred

**A newtype converts to another by a named function.** There is no blanket
`From` between two value types that happen to share an inner integer, because
such an implementation restores exactly the confusion D1 exists to prevent.

A conversion to and from the inner integer is permitted and named, because a
caller sometimes has a raw value and must say so.

## Consequences

**Every signature says what it takes.** A reviewer reads the types and knows
what a call means without reading the body. That is the whole return on this
record, and it is paid for at every call site by a wrapper.

**A column of a value type is a column of integers.** D2 makes this true, so
no bulk operation in the engine pays for the wrapper: not a sort, not a hash,
not a copy, not a cast to bytes.

**A bare integer in a signature is a signal.** It says the value has no
meaning the simulation names. A contributor who finds one that does has found
either a missing type or a genuine count.

**The newtypes are public, and their inner value is public with them.** A
caller outside the crate builds one and reads one. The type buys the
compiler's check on the way through, not an invariant on the value inside, and
a record that claimed the second would be claiming something the code does not
enforce.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
