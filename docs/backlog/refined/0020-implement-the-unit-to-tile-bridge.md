---
id: 0020
title: Implement the unit-to-tile bridge
status: refined
created: 2026-08-30
implements: [ADR-0007 D1, ADR-0004 D1, ADR-0004 D4]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

A soldier knows its tile. A tile knows nothing.

Movement needs the other direction. Admission must ask how many soldiers
already stand on a tile before it grants a move, and the viewer must ask
which soldiers to draw on a tile it is about to paint. Both questions are the
same question, and scanning every soldier to answer it costs the whole
population for one tile.

Registry row 0018 holds the claim: the bridge is three structures, and units
stay sorted by tile. Item 0029 writes it.

## Impact review

**Governed by.** ADR-0007 states that the sort takes a key vector and never a
comparison function, and that the last key field is a stable identifier so no
two items tie. ADR-0004 D1 requires an explicit stable order and D4 requires
a stable key. ADR-0017 D1 gives the tile index that the sort key uses. Row
0018 holds the bridge claim.

**Changes.** None.

**Creates.** None, if row 0018 holds. A rebuild policy that row 0018 does not
state is a decision this item must record rather than assume.

**Blockers.** BLK-007 governs every cost figure. The tile capacity is eight,
and crossing terrain raises it to sixteen; both are settled and come from the
scale constants table rather than from a constant in the code.

**What the caller can get wrong.** A caller can read the bridge after moving
a soldier and before the rebuild, which returns a stale answer that looks
correct. A caller can hold a range into the sorted array across a structural
change. Both must be impossible by construction or refused by a typed error;
a comment that says "call this first" is the shape that FND-040 warns about.

**Precedent.** FND-040 records that one fact in two places rots when nothing
fails on disagreement. The bridge is a second declaration of where a soldier
stands, and the soldier's own tile field is the first. A check must fail when
the two disagree, and the invariant check is where it goes.

## Done when

- A tile answers which soldiers stand on it, without scanning the population.
- A soldier answers which tile it stands on, as it already does.
- The sort goes through the key vector interface of item 0014, not through a
  comparison function.
- The invariant check fails when the bridge and the soldier tile column
  disagree, and a test proves that it fails.
- A caller cannot read a stale bridge: either the read rebuilds, or the type
  makes the stale read impossible, or the read returns a typed error.
- A property test asserts that the bridge holds exactly the live soldiers,
  each on the tile its own column names.
- A property test asserts that the bridge is identical at 1, 2 and 12
  threads, including when many soldiers share one tile.
- The new tests are checked against a mutation, and the mutations are named
  in the commit body.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in on completion.

## References

[^1]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/draft/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^4]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-040. `docs/FINDINGS.md`
