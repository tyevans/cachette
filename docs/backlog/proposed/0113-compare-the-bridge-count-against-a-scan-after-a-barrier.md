---
id: 0113
title: Compare the bridge count against a scan after a barrier
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Admission enforces the capacity of a tile. It reads the occupancy from the
derived unit-to-tile bridge, through one function, and it trusts what that
function returns.

The bridge refuses a read when it knows it is stale, and four tests hold that
refusal. The refusal catches a bridge that is known to be out of date. It does
not catch a bridge that is considered fresh and is wrong. A rebuild that drops
a unit, mis-keys a block edge, or loses a tile at a block boundary produces a
fresh bridge with a wrong answer. Admission then enforces the capacity from
that wrong answer, and the world it produces violates the invariant while
staying perfectly repeatable. Both determinism tests pass, because both
compare two runs that genuinely agree.[^1]

A property test already compares a per-tile query against a scan of the arena.
It covers the function that returns the units of a tile. It does not cover the
function that returns the count, and the count is the one admission reads.

The fixture is the second half of the gap. It builds an arena by spawning and
despawning, and it rebuilds the bridge once. No test compares the two after a
world has stepped with movement in it, which is the only time the rebuild runs
against a moving population.[^2]

## What to do

Add a property test that steps a world with movement in it and asserts, after
the barrier, that the count the bridge gives for every occupied tile equals a
scan of the arena.

Prove the test can fail. Perturb one bridge entry behind a test-only switch
and assert that the test then fails.[^3]

Check what the fixture supplies before trusting the result. A world whose
units never contend, never cross a block edge, and never fill a tile gives the
assertion no case that could fail it. If the test passes with the perturbation
in place, the fixture is the defect and not the bridge.[^2]

## Impact review

**Which records govern this.** ADR-0018 governs the bridge and its rebuild at
the barrier. ADR-0056 D3 states that admission reads the occupancy of a target
from the derived structure and carries no per-tile array of its own, so the
bridge is the only source of the value this test checks.

**Does this contradict a record.** No. It tests a claim two accepted records
already make.

**Does this create a decision.** No.

**Is this blocked.** No.

## Done when

- A property test steps a world with movement and compares the count for every
  occupied tile against a scan of the arena after the barrier.
- A perturbation of one bridge entry makes that test fail, and the proof is a
  test rather than a claim.
- The fixture is stated to produce contention, a block edge crossing, and a
  full tile, and each is asserted rather than assumed.

## References

[^1]: Testing rules, section 2. `.claude/rules/testing.md`
[^2]: Testing rules, section 2a. `.claude/rules/testing.md`
[^3]: Testing rules, section 1. `.claude/rules/testing.md`
