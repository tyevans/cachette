---
id: 0509
title: Count a build refusal apart from a plan write refusal
status: complete
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

## What was missing before this was refined

A refiner answers these before this item leaves `proposed/`.

- **Which acts belong to which counter.** The plan verb refuses five ways
  before it writes, and the build rule refuses three. The plan counts the write
  refusals (`NoSuchFaction`, `AddressOutsideWorld`, `GroundNotHeld`, and
  `NoRowFits`) under `projects_refused`. The drop at the bound (`PlanFull`) is
  counted under `projects_dropped`. The build rule counts its three permission
  refusals (`ProjectHoldsAnother`, `NoProject`, and `GroundNotHeld`) under
  `builds_refused`.
- **Whether the build refusals belong to the plan register at all.** The build
  refusals do not belong to the plan register. The build rule is not the plan.
  The build side counts build refusals in the census totals, and the plan
  register counts plan writes, finishes, drops, and write refusals alone.
- **What the row names are.** `projects_refused` counts the write refusals the
  plan turned away. `builds_refused` counts the build orders the build rule
  turned away. Both appear as `CensusBasis::Total` in the subsystem census
  table.
- **What test fails when a row stops counting.** A test asserts that a build
  order refusal increments `builds_refused` and does not change
  `projects_refused`. A second test asserts that a plan write refusal
  increments `projects_refused` and does not change `builds_refused`.

## Done when

1. `CensusTotals` holds `builds_refused: i64`, and `SUBSYSTEM_CENSUS` exposes
   the row `builds_refused` with basis `CensusBasis::Total`.
2. `order_build` increments `builds_refused` upon a permission refusal
   (`ProjectHoldsAnother`, `NoProject`, or `GroundNotHeld`), and no longer
   calls `self.plan.count_refusal()`.
3. `controller_take_projects` counts an order refusal once and not twice.
4. `PlanRegister::refused_count()` and census row `projects_refused` count plan
   write refusals alone.
5. Tests verify that `projects_refused` and `builds_refused` are disjoint and
   fail when either counter stops counting.[^3]

## Outcome

The build order refusals and plan write refusals are now counted apart in the
subsystem census.

1. `CensusTotals` holds `builds_refused: i64`, and `SUBSYSTEM_CENSUS` exposes
   the row `builds_refused` with basis `CensusBasis::Total`. `World` exposes
   the reader `builds_refused(&self) -> i64`.
2. `order_build` increments `builds_refused` upon each permission refusal
   (`ProjectHoldsAnother`, `NoProject`, or `GroundNotHeld`), and no longer
   calls `self.plan.count_refusal()`.
3. `controller_take_projects` no longer double-counts build refusals across the
   group set evaluation.
4. `PlanRegister::refused_count()` and census row `projects_refused` count plan
   write refusals alone.
5. Added test `a_build_refusal_and_a_plan_write_refusal_are_counted_apart` in
   `crates/cachette-core/tests/plan.rs`, proving that build refusals and plan
   write refusals increment disjoint counters and each act is counted once.

## References

[^1]: Findings register, FND-496. `docs/FINDINGS.md`
[^2]: Findings register, FND-548. `docs/FINDINGS.md`
[^3]: Testing Rules, section 2. `.agents/rules/testing.md`
