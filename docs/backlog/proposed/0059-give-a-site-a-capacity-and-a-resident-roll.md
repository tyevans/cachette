---
id: 0059
title: Give a site a capacity and a resident roll
status: proposed
created: 2026-08-31
implements: [ADR-0066 D1, ADR-0004 D4, ADR-0014 D1]
changes: []
creates: []
serves: [PRD-0014]
blocked-by: [0052]
---

## Why

A unit stands on a tile and belongs to nothing. Any number of units can fill a
region and no place holds anybody. Growth costs nothing, no place is worth
defending, and crowding cannot happen.

This item gives a site a capacity and gives a unit a place it lives in. Where
a unit stands and where it lives become two different facts.

## What the work does

1. A site holds a capacity, and the capacity follows from what has been built
   there rather than from the size of the ground.
2. A unit holds a residence. A unit that lives nowhere is still a unit.
3. A site holds an occupancy count, kept by the change and never recomputed by
   a sweep over the units.
4. A site that is lost updates every resident of it as one set-valued
   operation.

## Impact review

**Governed by.** ADR-0066 D1 puts the capacity on the settlement shape, which
is the shape that is fixed to a tile.[^1] ADR-0014 D1 makes a residence a
generational identity, so a lost site never hands its identity to the site
founded next in that slot.[^2] ADR-0004 D4 requires a stable key for the
assignment and for the eviction.[^3]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-005 is resolved and gives the settlement count.[^4] BLK-003 is resolved:
the population counts everybody, so everybody needs somewhere to live.[^5]

**Serves.** PRD-0014.

**Conflict surface.** `crates/cachette-core/src/site.rs`,
`crates/cachette-core/src/soldier.rs` at a new residence column, and
`crates/cachette-core/src/world.rs` at the state hash and the invariant check.
**It cannot run beside item 0063**, which reads the residence to scope an
assignment, until this item lands.

## What is missing before this is refined

**The registry row.** This work states a constraint that no reserved row
holds: **occupancy is a maintained count and a residence is a forward column,
so no query walks the population.** All three conditions of the scope rule
hold.[^6] A contributor could reasonably put a dwelling slot on every tile or
recompute occupancy by a sweep, and PRD-0014 rejects both by name. Changing it
later means rewriting every reader. The reasoning is not visible in a counter.

Whoever picks this item up **allocates the row in the registry before writing
the record**.[^7] The row is not reserved today, and this item does not choose
a number.

**The relation between a place to live and a household.** Point five of item
0050 records that PRD-0014 gives a dwelling no owner and PRD-0015 models no
inheritance, and that the household and the place to live may turn out to be
the same thing.[^8] **This plan carries that question rather than answering
it, and the carry is deliberate**: the answer costs nothing while descent does
not exist, and item 0067 is where it becomes cheap to answer. **Do not invent
a household structure in this item.** A household is the residents of one site
who share a line, and it is derived once both halves exist.

## Done when

- A site answers its capacity, and the capacity follows from what was built.
- A unit answers where it lives, and living nowhere is a representable answer.
- A site answers how many live in it, at a cost that does not grow with the
  population. A test asserts the count against a full pass over the units.
- A population larger than the capacity produces a consequence a watcher can
  read and name.
- Losing a site clears every resident in one operation, and a test asserts
  that no resident still names the lost site.
- The invariant check fails when the occupancy count and the residence column
  disagree, and a test proves that it fails.
- A property test asserts that the assignment and the eviction are identical
  at 1, 2 and 12 threads.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^6]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
[^8]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
