---
id: 0237
title: Declare what each stage reads and what it writes
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**An accepted record says in its own consequences that a reviewer cannot check
it.** The rule is that a parallel stage writes only to memory no other thread
writes, and that the outputs combine in an order the data fixes. Its consequences
say the rule is cheap to check and impossible to check for: a reviewer can see an
atomic and a lock, and cannot see a thread reading a location another is
writing.[^1]

A stage that declared what it reads and what it writes would make that
mechanical. Two stages whose write sets intersect cannot run together. A stage
whose read set intersects another's write set has an ordering constraint that a
check can state rather than a reviewer remember.

**It paid a second time, and that half is now done under its own number.** A
stage with a declared boundary can be timed from outside. Item 0289 named every
pass of a frame and made the step record what each one costs, so a cost now
belongs to a stage.[^5] **That item declares no read set and no write set**, and
it says so: an instrument that a feature switches off binds nothing, and the
declaration this item is about is a constraint.

**What is left here is the checking half alone.** Two stages whose write sets
intersect cannot run together, and today only a reviewer can see that.

The record on cost states the constraint that makes this urgent and deliberately
does not carry this claim, because a record holds one claim and mixing cost with
observability would make neither rejectable on its own.[^2]

## What is missing before this is refined

- The impact review, and whether this needs a record of its own or is a design
  document. It states a mechanism rather than a constraint, which is the test
  that decides.[^3] The answer may differ now that the timing half is out: a
  declaration that a check enforces is closer to a constraint than an
  instrument is.[^5]
- What a declaration is. A type, a const table, an attribute, or a doc comment
  that a script parses. Only the first two can fail a build.
- Whether the declaration can be derived rather than written. A second
  declaration site that nothing checks against the code is the shape this project
  meets most often.[^4]
- Whether the ordering check runs at compile time or as a test.
- What a stage is. **Item 0289 answered this in one direction**: the frame is
  now a named list of twenty-one passes, derived from one macro list, and a test
  compares that list against the step.[^5] Whether the same list is the right
  unit for a read set and a write set is open, because three of its entries are
  the same function called at three positions.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0009, parallel stages write disjoint outputs, the consequences. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Backlog item 0289, price every stage of a frame by name. `docs/backlog/complete/0289-price-every-stage-of-a-frame-by-name.md`
