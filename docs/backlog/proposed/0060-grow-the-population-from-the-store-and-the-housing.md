---
id: 0060
title: Grow the population from the store and the housing
status: proposed
created: 2026-08-31
implements: [ADR-0003 D1, ADR-0004 D1, ADR-0014 D3, ADR-0020]
changes: []
creates: []
serves: [PRD-0011, PRD-0014]
blocked-by: [0056, 0059]
---

## Why

The number of units is a number somebody chose. Nothing the world does changes
it. A faction that gathers well and one that gathers badly hold the same units
for ever, so success has no expression and the world has no decline.

Item 0057 lets a unit end. This item lets one begin, and ties both to what a
place has.

## What the work does

1. A site with a surplus and a spare place to live adds a unit at an interval.
2. A site with neither adds none.
3. The birth draw is keyed, and it is keyed on the site and the frame, so that
   two sites in one frame do not draw the same value.
4. A count of the population is read from a maintained figure, not from a pass
   over the units.

## Impact review

**Governed by.** ADR-0003 D1 requires every draw to be keyed on the tuple of
system, frame, entity and draw, and D2 forbids thread-local state.[^1] ADR-0014
D3 makes the new unit's identity distinct from the identity of the unit that
died in its slot.[^2] ADR-0020 batches the structural change at the
barrier.[^3] ADR-0004 D1 fixes the order in which births are applied.[^4]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-003 gives the population target and BLK-005 gives the settlement
count.[^5] [^6]

**Serves.** PRD-0011 and PRD-0014.

**Conflict surface.** `crates/cachette-core/src/cohort.rs`,
`crates/cachette-core/src/site.rs`, `crates/cachette-core/src/soldier.rs` at
the spawn path, and `crates/cachette-core/src/world.rs` at the step. **It
cannot run beside item 0057**, which writes the death path in the same step
stage, and **it cannot run beside item 0061**, which spawns the founding
group.

## What is missing before this is refined

**The composition of the two limits, and the record that holds it.** This is
point four of item 0050, and it is the reason this item is not refined.[^7]
PRD-0011 says the population responds to what a faction has. PRD-0014 says
growth slows when there is nowhere to live. Two independent limits on one
quantity give a result that depends on which one runs first, and that is
exactly the kind of order dependence this project cannot carry.

**The recommendation this item carries, for whoever refines it.** Make the
housing capacity an **admission bound** and the store a **rate**. A birth is
proposed at a rate the store sets, and it is admitted only while a place is
free. The two then compose by one operation with one answer, in the order the
schedule states, and neither limit is applied twice. That is the same
intent-then-admission shape the project already uses for movement.[^8] It is a
recommendation and not a decision, because nobody has recorded it.

**The registry row.** The composition above is a constraint, and no reserved
row holds it. The three conditions of the scope rule hold: a contributor could
reasonably apply two multipliers instead, the cost of changing it later is the
whole growth curve, and the reasoning is not visible in a minimum. **Allocate
the row in the registry before writing the record.**[^9]

**Whether a birth is a unit or a headcount.** Item 0056 gives a cohort a
headcount. A birth may add to the headcount, or it may spawn a unit, or both.
Nothing states which, and the answer decides whether this item touches the
spawn path at all. Answer it in the impact review.

## Done when

- A site with a surplus and a free place adds a unit at an interval.
- A site with no surplus adds none, and a site with no free place adds none. A
  test asserts each case separately.
- The two limits compose by one stated rule, and a test asserts that the
  result does not depend on the order the two were evaluated in.
- The birth draw is keyed. A test changes the frame and asserts the draw
  changes; a second test changes the site and asserts the draw changes.[^10]
- A dead unit's slot, reused for a birth, gives an identity that never
  resolves as the dead unit. A test asserts it.
- A count of the population is read at a cost that does not grow with the
  population, and a test asserts it against a full pass.
- A property test asserts that the same seed gives the same births, in the
  same order, at 1, 2 and 12 threads.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^2]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^3]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
[^4]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^7]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
[^8]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^9]: ADR Registry. `docs/adrs/REGISTRY.md`
[^10]: Testing Rules, section 2. `.claude/rules/testing.md`
