---
id: 0018
title: Implement the rhombus tile grid and the axial index
status: complete
created: 2026-08-30
implements: [ADR-0017 D1, ADR-0002 D1, ADR-0004 D1]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The core holds a flat array of tile values and a tile count. It has no
geometry. A tile has no address, no neighbour, and no position, so nothing
can move and nothing can be drawn.

This item gives the world its shape. It is the smallest change that turns a
list of values into a map.

## Impact review

**Governed by.** ADR-0017 states the index. ADR-0002 D1 forbids a floating
point coordinate, so the axial pair and every derived value stay exact.
ADR-0004 D1 requires that iteration over tiles has an explicit stable order,
which is index order. ADR-0006 D1 governs any type whose bytes are hashed, so
a coordinate that enters the state hash declares its padding.[^1] [^2] [^3]
[^4]

**Changes.** None. The world gains geometry; no accepted claim changes.

**Creates.** None, unless the work finds a decision that ADR-0017 does not
hold. A neighbour convention is part of the index claim. A wrap rule at the
world edge is not, and if the work needs one, that is a new record and a
deliverable of this item.

**Blockers.** BLK-007 governs every cost figure. State no throughput figure
in the code, in a comment or in a record. The world extent comes from the
scale constants table and is read, not invented.[^5]

**Precedent.** FND-040 requires that a citation in a source file names a
decision that exists. Item 0012 makes that a gate, and this item is the first
new code the gate covers.[^6]

**Serves.** PRD-0002.

## Done when

- A tile has an axial address, and the conversion from the address to the
  index is one multiply and one add.
- The reverse conversion exists, because the renderer needs it.
- A tile knows its six neighbours, and a neighbour outside the world is
  absent rather than wrapped.
- No floating point appears in any of it, and the float gate passes.
- A property test asserts that the address and the index round-trip for every
  tile in a generated world.
- A property test asserts that the neighbour relation is symmetric: if B is a
  neighbour of A, then A is a neighbour of B.
- A test asserts that the neighbour set of a corner tile has the right size,
  so the edge case is covered rather than assumed.
- The world builds at a configured extent, and the extent comes from the
  configuration rather than from a constant in the code.
- The state hash covers the geometry, or the test says why it does not.
- The thread-count test still passes at 1, 2 and 12 threads, and the
  perturbed build still fails it.
- `just check` runs green.

## Outcome

`crates/cachette-core/src/hex.rs` holds `Axial` and `Grid`. The world takes a
width and a height instead of a flat tile count, and reads a tile through an
address, so the geometry has a real caller rather than a test that drives it
directly.

Three things changed from the plan.

`World::new` now returns a result. The plan assumed the extent could not be
wrong, but a zero side and a tile count that overflows the index are both
reachable from the public interface, and a panic there is not a typed error.
The Python side gained `ConfigError` to match, under the same root.

The state hash covers the width and the height rather than a tile count, so
the golden files were re-recorded. The difference was read before it was
recorded: the hash input changed shape, and the scenario extents changed with
it.

The tests were checked against three mutations rather than assumed to work: a
neighbour offset that breaks the symmetric pair, a transposed index formula,
and a `contains` that never rejects an outside address. Each mutation failed
four tests. The commands are in the commit body.

No new record was needed. The neighbour convention and the wrap rule are both
part of the ADR-0017 D2 claim.

## References

[^1]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/REGISTRY.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0006, an event is plain data and applying it is pure. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^5]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^6]: Findings register, FND-040. `docs/FINDINGS.md`
