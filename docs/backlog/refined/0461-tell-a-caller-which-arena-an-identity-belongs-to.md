---
id: 0461
title: Tell a caller which arena an identity belongs to
status: refined
created: 2026-09-03
implements: [ADR-0014 D1, ADR-0046 D1]
changes: []
creates: []
serves: [PRD-0021, PRD-0005]
blocked-by: [DEC-266]
---

## Why

Each entity arena (units, characters, settlements) numbers its own slots from
zero. As a result, the first unit, the first character, and the first
settlement of a world share identical integer identifiers. A call that queries
a unit accesses the soldier arena, while a call that queries a character
accesses the character arena. Neither rejects the other's ID.[^1]

This allows callers to inadvertently pass a character identifier to a unit query
and receive a valid, completely wrong record with no warning or error. Because
Cachette is strictly deterministic, the silent mismatch repeats reliably,
evading determinism comparison harnesses.[^2] Providing typed identities and
clear arena validation gives developers immediate feedback when referencing
world entities.[^6]

## Impact review

**Governed by.** ADR-0014 D1 establishes generational identities. ADR-0046 D1
defines the Python exception and error hierarchy. DEC-266 provides options for
arena disambiguation.

**Design choices.**
1. DEC-266 is resolved following its recommendation: encode the arena tag
   (e.g., Soldier, Character, Settlement) into the entity identity at the
   boundary, or provide strongly typed wrapper types in the Python control
   plane.[^3]
2. Passing an identity tagged with one arena to an API method expecting another
   raises a descriptive, typed exception (`ArenaMismatchError`).
3. The internal generational index inside `Entity` is preserved so simulation
   hot paths remain zero-overhead.
4. House-based historical queries are kept separate to maintain single-purpose
   scope for this item.[^4] [^5]

**Changes.** None to decision records.

**Creates.** None.

**Blockers.** DEC-266 is resolved by this refinement.

**Precedent.** FND-472 recorded that mixing character and unit identifiers
passed silently through both the engine and test suites.

**Conflict surface.** `crates/cachette-core/src/types.rs`,
`crates/cachette-core/src/errors.rs`, and `crates/cachette-python/src/`.

## Done when

- Boundary identity representations distinguish between soldier, character,
  and settlement arenas.
- Calling an engine function with an entity identity from the wrong arena
  fails immediately with a typed error.
- Python bindings expose distinct identity types or enforce tag validation on
  entry.
- A test verifies that supplying a character ID to a unit inspection method
  raises `ArenaMismatchError` with a clear diagnostic message.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-472. `docs/FINDINGS.md`
[^2]: Testing rules, section 2. `.agents/rules/testing.md`
[^3]: Decisions register, DEC-266. `docs/DECISIONS.md`
[^4]: Findings register, FND-471. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-265. `docs/DECISIONS.md`
[^6]: PRD-0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
