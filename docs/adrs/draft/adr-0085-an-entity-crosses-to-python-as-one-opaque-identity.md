# ADR-0085: An entity crosses to Python as one opaque identity that the engine resolves

## Context

The engine simulates a world of tiles and units. A unit lives in a slot of an
arena, and the arena reuses a slot after the unit in it dies. An identity
therefore packs the slot index and a generation counter into one value, and the
generation advances when the engine frees the slot.[^1]

The entity storage is the only thing that builds an identity. Every other
caller receives one and reads its parts through accessors. A caller that
assembles an identity from an index it chose has defeated the generation,
because that index came from somewhere that could not know whether the slot
still holds the same unit.[^1] The Rust code enforces this: the constructor of
the identity type is private to the core crate.

Python is the control plane of this engine.[^2] The control plane now needs to
read the event log. An event of the gather resolve names the unit that took an
amount, so an identity must cross the boundary.[^3] The project owner chose the
column form for the crossing: the bindings return one array for each field of
an event.[^4]

**A column holds a number. It cannot hold a private Rust type.** The compiler
guard that protects the identity inside the core crate does not reach across
the boundary, and no Python type system replaces it. Whatever crosses is a
number that a Python caller can read, store, alter and hand back.

This is the point at which a contributor reasonably chooses the slot index. The
index is half the width, it indexes a column directly, and it reads as the
obvious name of a unit. The cost of that choice is silent. A watcher holds an
index, the unit dies, a new unit takes the slot, and the watcher reports on the
new unit while every test stays green. The project has already met this shape
once inside the engine, where a movement system keyed a random draw on the slot
index rather than on the identity.[^5]

The reasoning is not visible in the artefact. A binding whose parameter reads
`unit: int` says nothing about which integer is meant.

## Decision

### D1. An identity crosses as its whole value, never as a slot index

A column of units holds the whole 64 bits of the identity, as the engine gave
it out. The bindings expose no column of slot indices and no accessor that
splits an identity into its parts.

The width follows from the decision and is not a budget. An identity is one
value of the width the engine declares, and half of it is not an identity.

### D2. The bindings expose no way to build an identity

No function of the bindings takes an index and a generation and returns an
identity. Python receives an identity from the engine, holds it, and gives it
back. It does not compose one.

This keeps the rule the core crate already holds, at the one boundary where the
compiler cannot hold it.[^1]

### D3. The engine resolves an identity that Python hands back, and refuses a stale one

Every binding that takes an identity resolves it against the arena before it
does anything. Resolution compares the generation in the identity against the
generation the arena holds for that slot.[^6]

A resolution that fails raises the typed error for a stale or out-of-scope
view.[^7] It never falls back to the slot, and it never returns the unit that
now occupies it. The caller either handles the absent unit or stops.[^6]

### D4. The rule holds for every future binding that takes an entity

A later binding that takes a unit, a settlement, a character or a site follows
D1 to D3. The rule is about the identity, not about the soldier arena that
happens to be the first caller.

## Consequences

Python cannot name a unit that the engine did not name first. A control plane
that wants to act on a set of units builds a selector and lets the engine
resolve it, which is what the design already asks for.[^2]

A stale identity is a visible failure at the boundary rather than a wrong
answer inside it. An agent that watches a unit across a death learns that the
unit died, and it learns it from an exception rather than from a report about
somebody else.

Every binding that takes an entity costs one resolution. The resolution is a
comparison against one array element.

**The alternatives, and why the project rejected them.**

The bindings could return the slot index. It is narrower, and it indexes the
unit columns without a lookup. It is not an identity, and the failure it
creates is silent, so the project rejects it.

The bindings could return two columns, an index and a generation. That stores
the two parts where a caller can set them apart, which the identity record
forbids for the reason above.[^1]

The bindings could wrap the identity in a Python class that Python cannot
construct. That does not remove the need for D3, because a wrapper holding a
stale value is still stale, and a column of objects is not a column. The
project takes the check instead of the wrapper.

**What this record does not decide.** It does not decide which fields of an
event cross, or in what form the other fields cross. The decisions register
holds that choice.[^4] It does not decide whether the control plane may put one
unit in the world at a time.[^8]

## References

[^1]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: The event types. `crates/cachette-core/src/event.rs`
[^4]: Decisions register, DEC-060. `docs/DECISIONS.md`
[^5]: Testing Rules, section 2. `.claude/rules/testing.md`
[^6]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^7]: ADR-0046, every error is typed. `docs/adrs/REGISTRY.md`
[^8]: Decisions register, DEC-063. `docs/DECISIONS.md`
