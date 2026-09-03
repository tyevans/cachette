---
id: 0302
title: Fail when a stage declaration disagrees with a measurement
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**Every stage of a frame declares whether it takes a thread count, and nothing
compares the declaration with what the stage does.** The cost register says in
its own words that the column is a declaration in the source rather than a
measurement, and that a stage declared one way which behaves the other way means
the declaration is wrong. It also says the table is where the two can be
compared. Nothing performs that comparison.[^1]

**Three declarations were wrong for as long as the table has existed.** The
three stages that wrap the bridge rebuild all declared that they take a thread
count, and an accepted record says the rebuild accepts none.[^2] Measured at one
thread and at twelve, the barrier stage costs 43,040,085 and 57,165,452
nanoseconds: it does not improve, and it may get worse. Every cost table written
in one night printed `yes` for all three.[^3]

**The failure is silent in both directions.** A stage declared to take a thread
count that does not is read by a planner as parallel work that has already been
done, so nobody looks at it. A stage declared not to take one that does is read
as a serial bottleneck that is not there.

**The first direction is the harder one, and this project has an instance of
it.** The bridge rebuild not only fails to improve with more threads, it
measured worse at twelve than at one. A check that only looks for improvement
where `false` is declared would not have found it. Nothing here rests on a
report of a second instance elsewhere; refining this item should look for one
rather than assume it.

**This is one fact in two places with nothing that fails when they disagree**,
which is the defect shape this project records most often, and it is
simultaneously a record that the code contradicts.[^4] [^5]

## What is missing before this is refined

- **What counts as agreement.** A stage that improves by two percent at twelve
  threads is not parallel in any useful sense, and one that improves by ten
  times plainly is. The item must state a threshold and say where it comes
  from, and it must not invent one.
- **Where the measurement comes from.** The stage-cost mode already runs a
  configuration at a stated thread count. The check needs two runs of the same
  configuration at two thread counts and a comparison of the rows, which the
  mode does not do today.
- **Whether it gates a merge.** A timing assertion is flaky on a loaded machine
  and the testing rule forbids one in a test.[^6] This is a benchmark
  comparison rather than a test, so it belongs where a benchmark belongs, and
  the item must say what a failure does.
- **Whether the small stages are exempt.** Several stages cost under a
  microsecond a frame, where the ratio between two thread counts is noise. A
  threshold on the absolute cost probably has to come first.
- **Whether a record can be cited instead.** For the three bridge stages the
  answer was already written down: an accepted record says the rebuild accepts
  no thread count. A check that read the record would have caught them without
  any measurement at all.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, every stage of a frame by name. `docs/reference/graviton-costs.md`
[^2]: ADR-0071, the bridge rebuild orders on one thread, decision D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
[^3]: Findings register, FND-301. `docs/FINDINGS.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Recurring defect shapes, shape 5. `.claude/rules/recurring-defects.md`
[^6]: Testing Rules, section 3. `.claude/rules/testing.md`
