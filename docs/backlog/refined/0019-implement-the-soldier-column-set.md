---
id: 0019
title: Implement the soldier column set
status: refined
created: 2026-08-30
implements: [ADR-0066 D1, ADR-0066 D3]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The world has tiles and no entities. PRD-0002 needs entities that appear on
the world and move between tiles, so something must hold them.

ADR-0066 fixes four shapes and gives each its own column set. The soldier is
the mobile one, so it is the shape this sprint needs. The other three wait:
the record fixes four shapes, and it does not require that all four exist at
once.

## Impact review

**Governed by.** ADR-0066 D1 gives each shape its own columns, and D3 makes a
shape that is not one of the four a compile-time error rather than a run-time
table. Row 0012 holds the arena claim and row 0014 holds the identity claim;
item 0029 writes both before this item starts. ADR-0002 D1 keeps every
soldier field exact. ADR-0004 D1 requires an explicit stable order over the
column. ADR-0006 D1 governs any field whose bytes reach the state hash.

**Changes.** None. The world gains a column set, and no accepted claim
changes.

**Creates.** None, if items 0029's records hold. A decision this work finds
that no record holds is a deliverable of this item, not a byproduct.

**Blockers.** BLK-007 governs every cost figure. State none, in the code or
in a comment. The faction ceiling is 63 and it is settled, so a soldier's
faction is a bit index and not a parametric value.

**What the caller can get wrong.** The sprint 1 retrospective asks this at
refinement, because two of five items found the answer late. A caller can ask
for more entities than the arena holds, ask for an entity on a tile outside
the world, hold a handle to a despawned entity, and ask for a faction above
the ceiling. Each needs a typed refusal rather than a panic, and each needs a
test.

**Precedent.** FND-041 records that the project stated capabilities nobody
checked. A column set that no system reads is inert, so a test drives the
step and then inspects the arena.

## Done when

- A soldier carries a generational handle, a tile address, and a faction.
- A handle names at most one live soldier. A stale handle reads as absent
  rather than as a different soldier.
- The arena refuses, with a typed error, each of the four caller mistakes
  named above.
- A soldier is placed on a tile that the grid contains, and placing one
  outside the world is refused.
- The state hash covers the soldier columns, and the golden files are
  re-recorded after reading the difference.
- A property test asserts that a handle round-trips through the arena.
- A property test asserts that spawning and despawning in any order leaves
  the arena holding exactly the live set.
- The thread-count test covers a world that holds soldiers, at 1, 2 and 12
  threads, and the perturbed build fails it.
- The new tests are checked against a mutation, as sprint 1 did, and the
  mutations are named in the commit body.
- `just check` runs green.

## Outcome

Filled in on completion.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0006, an event is plain data and applying it is pure. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-041. `docs/FINDINGS.md`
