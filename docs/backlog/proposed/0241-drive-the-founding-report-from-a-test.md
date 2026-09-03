---
id: 0241
title: Drive the founding report from a test
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The demonstration reports whether each founded ground carries its group, and
nothing drives that report.** It is a pair of `println` calls inside the
founding routine of the binary. No test reaches them, so nothing fails if the
condition inverts, if the comparison drifts, or if the line stops printing.

The report exists because the demonstration fed every unit for as long as it
had existed and nobody noticed.[^1] It is the thing that makes a silent fixture
speak. **A guard against a silent failure that is itself unguarded is the
weaker half of the repair.**

This is the inert capability shape. The rule asks who is obliged to invoke a
thing: if the engine is, the test starts at the engine.[^2] Here the
demonstration is obliged, on every run, and no test starts there.

**The report holds one fact that is derived, and the derivation could drift.**
A site feeds as many people as the food its survey measured, because the
production rate is a sixteenth of the reached food and a ration is a sixteenth
of a full need. The report compares the reached food against the group size on
that basis. If either sixteenth changes, the comparison is silently wrong and
the demonstration goes back to asserting a split it does not have.

## What is missing before this is refined

- The impact review.
- **Where the predicate goes.** It cannot be tested where it is, because it
  sits in a binary. Moving it to the viewer library makes it reachable, and
  that is an arrangement rather than a constraint, so the item must say why the
  move serves the test rather than describing the move.
- Whether the engine should answer the question instead. "How many people can
  this site carry" is a fact about the world, and the demonstration derives it
  from two constants that belong to the engine. A public reader would remove
  the derivation from the viewer and the drift with it. That is the larger
  answer and it needs its own decision.
- What the test asserts. A world founded on ground that cannot carry its group
  must report that it cannot, and one founded on ground that can must report
  that it can. Both directions, because a report that always said one thing
  would pass a test that only checked the other.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-232. `docs/FINDINGS.md`
[^2]: Testing rules, section 5. `.claude/rules/testing.md`
