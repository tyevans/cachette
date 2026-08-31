---
id: 0056
title: Draw consumption from a site store by cohort
status: refined
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0002 D3, ADR-0004 D1, ADR-0004 D2, ADR-0066 D1]
changes: []
creates: [ADR-0063]
serves: [PRD-0013]
blocked-by: [0052]
---

## Why

Nothing takes a quantity back out of the world for a reason. A pile of food
and no food are the same to the unit that holds them, so a surplus means
nothing and a shortage cannot happen.

This item makes existence cost something. It is the first rule that removes a
quantity for a reason, and every later limit on growth reads its result.

## The recommendation this item takes, stated plainly

**The project takes the cohort model for the draw, and keeps the need on the
individual.** The research recommends the split in that exact form, and the
project has already made it binding: BLK-008 is resolved as individual decay,
pooled consumption, aggregate decisions.[^1] The research derives the same
split from cost and reaches the same place.[^2] [^3]

So the two halves are:

- **The need is a per-unit scalar.** A unit carries its own need value and its
  own deficit, decayed by a saturating subtract over the whole population at
  an interval. The research finds this affordable at the target and says so
  against its own earlier conclusion.[^2]
- **The draw against a store is pooled.** A cohort is one row that stands for
  the units of one kind in one place, keyed so that the array is already
  sorted by site and the reduction needs no sort. The draw is one segmented
  reduction over cohorts, then one capped transfer, and never a loop over
  units and never a lock.

**The individual form of the draw is rejected, and this is why.** PRD-0013
rejects it too, in its own words: a rule that draws one unit at a time against
a shared store makes every unit in a place a writer to one location. The cost
is the contention, not the arithmetic. That is a shape argument and it does not
depend on any figure, so BLK-007 does not weaken it.[^4]

**One loss is carried deliberately.** A pure cohort has a cliff: a place is
fine at a little above its demand and starves entirely a little below it. The
per-unit deficit accumulator removes the cliff, because a shortage degrades
before it kills. That is why the need stays on the individual and is not
folded into the cohort row.[^2]

## Impact review

**Governed by.**

- ADR-0066 D1. A settlement holds pooled stores. The cohort rows attach to the
  settlement and add no fifth shape; a cohort is a row of the settlement's
  columns, not an entity.
- ADR-0002 D1 and D3. Every quantity is an integer or Q16.16, and the
  accumulator widens. A per-unit need summed over the target population
  overflows a narrow type.
- ADR-0004 D1. The cohort key fixes the iteration order, and the reduction
  visits spans in ascending order.
- ADR-0004 D2. The reduction is order-free, so it combines the same way at any
  thread count.
- ADR-0014 D1. A cohort holds a headcount and not a list of identities, so no
  identity is stored twice.[^5] [^6] [^7]

**Changes.** No record changes.

**Creates.** ADR-0063. The registry reserves the row and states the claim: a
need is a rate with a threshold, and crossing it is a fact.[^8] The claim
passes the three-condition test. A contributor could reasonably write the
individual draw, the cost of changing it later is every call site, and the
reasoning is not visible in a subtraction.

**This item resolves point one and point three of item 0050.**[^9] Point one:
PRD-0013 owns the draw and the condition; PRD-0011 keeps the rule by which a
unit ends, and item 0057 implements it. Point three: **a unit draws from the
site store, not from what it carries.** ADR-0066 D1 already settles it, because
the settlement is the shape that holds pooled stores and no other shape does.
Say so in ADR-0063 rather than deciding it again.

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-003 is resolved: one million is the whole population and every unit
consumes.[^10] BLK-008 is resolved and is quoted above.[^1] The interval is a
schedule parameter, not a value this item invents.

**Precedent.** FND-001 records that a monoid needs exact associativity. FND-016
records that a capacity cap is not a negative rate, which bears on the clamp at
the top of a need.[^11] [^12]

**Serves.** PRD-0013.

**Conflict surface.** `crates/cachette-core/src/cohort.rs` is new.
`crates/cachette-core/src/site.rs` gains the cohort columns.
`crates/cachette-core/src/soldier.rs` gains a need column and a deficit
column. `crates/cachette-core/src/world.rs` at the step and the state hash.
**It cannot run beside item 0055**, which edits the same site reduction, and
**it cannot run beside item 0057**, which extends its own output.

## Done when

- A unit carries a need that falls at an interval by a saturating subtract,
  never by a wrapping one, and a test asserts the saturation at zero.
- A cohort holds a headcount and a site, and the sum of every headcount equals
  the number of units that live. A test asserts that equality.
- The draw is one segmented reduction over cohorts followed by one capped
  transfer, and no part of it holds a lock, an atomic or a retry.
- The store falls by exactly what the cohorts received, and a conservation
  test over the world balances to zero.
- A store that cannot serve every cohort splits what it has by a stated rule,
  and the split is exact: the parts sum to the whole with no unit lost and
  none created. A test asserts the exactness at a store that divides unevenly.
- A property test asserts that the draw is identical at 1, 2 and 12 threads.
- A test asserts that a place producing less than its people consume runs its
  store down tick by tick, and that a place producing more accumulates to a
  stated bound.
- The fixture is built to hold a site in deficit and a site in surplus, and
  the commit body says how that was checked.[^13]
- ADR-0063 is written, the registry row moves to `Draft`, and the record holds
  no interval value, no cohort count and no cost figure.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Blockers register, BLK-008. `docs/BLOCKERS.md`
[^2]: Needs, consumption and the economy. `docs/research/reports/15-needs-consumption-and-economy.md`
[^3]: Open decisions register, DEC-002. `docs/DECISIONS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: ADR Registry, row 0063. `docs/adrs/REGISTRY.md`
[^9]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
[^10]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^11]: Findings register, FND-001. `docs/FINDINGS.md`
[^12]: Findings register, FND-016. `docs/FINDINGS.md`
[^13]: Findings register, FND-051. `docs/FINDINGS.md`
