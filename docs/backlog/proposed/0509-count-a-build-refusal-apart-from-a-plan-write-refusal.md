---
id: 0509
title: Count a build refusal apart from a plan write refusal
status: proposed
created: 2026-09-05
implements: [ADR-0152 D1]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: []
---

## Why

**One census row counts two subsystems.** The plan register holds one refusal
counter. A write the verb turned away raises it, and a build order the plan
refused raises it as well. The census prints the sum under one name, so a
reader cannot tell a plan that refuses writes from a plan that refuses
builds.[^1]

The earlier finding named this and left it open. It states that the two names
read as one subsystem and are three things.[^1] One of the three is now apart:
a write the bound drops is a drop and nothing else, and a test holds that
partition.[^2] The build refusals are still inside the refusal count.

**The rows of the census are meant to be added.** Two rows a reader adds must
be disjoint, and a row must name what it counts. A row that holds two
subsystems fails the second rule while it passes the first.

This is small and it is not urgent. Nothing reads the refusal row today except
the census print and one test, so the wrong reading costs a reader of the
demonstration deck and nothing else.

## What is missing before this is refined

A refiner answers these before this item leaves `proposed/`.

- **Which acts belong to which counter.** The plan verb refuses five ways
  before it writes, and the build rule refuses three. Say which of the eight
  the plan counts and under which name.
- **Whether the build refusals belong to the plan register at all.** The build
  rule is not the plan. A counter on the build side may be the honest place,
  and the plan register would then count writes alone.
- **What the row names are.** A name states what the row counts. Say the names
  and place them in the one table.
- **What test fails when a row stops counting.** Each row needs one, and the
  test must be proved by putting the defect back.[^3]

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-496. `docs/FINDINGS.md`
[^2]: Findings register, FND-548. `docs/FINDINGS.md`
[^3]: Testing Rules, section 2. `.agents/rules/testing.md`
