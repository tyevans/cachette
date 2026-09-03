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

**It pays a second time, and that is why it is worth doing before the
optimisation and not after.** A stage with a declared boundary can be run and
timed from outside. Today a frame is a sequence of private methods on the world,
so a cost belongs to the frame rather than to a stage, and a large cost cannot be
attributed without editing the engine to measure it.

The record on cost states the constraint that makes this urgent and deliberately
does not carry this claim, because a record holds one claim and mixing cost with
observability would make neither rejectable on its own.[^2]

## What is missing before this is refined

- The impact review, and whether this needs a record of its own or is a design
  document. It states a mechanism rather than a constraint, which is the test
  that decides.[^3]
- What a declaration is. A type, a const table, an attribute, or a doc comment
  that a script parses. Only the first two can fail a build.
- Whether the declaration can be derived rather than written. A second
  declaration site that nothing checks against the code is the shape this project
  meets most often.[^4]
- Whether the ordering check runs at compile time or as a test.
- What a stage is. The frame is a sequence of private methods today, and some of
  them do several things.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0009, parallel stages write disjoint outputs, the consequences. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
