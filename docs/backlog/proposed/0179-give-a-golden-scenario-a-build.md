---
id: 0179
title: Give a golden scenario a build
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

No golden state hash scenario builds anything, so the upgrade map is empty in
every one of them. A change that broke the number of ticks a build takes, or
the storage that carries it between ticks, moves no golden file and makes no
two thread counts disagree. The finding records the experiment that proved
it.[^1]

The upgrade test file covers the behaviour. The two determinism tests do not,
and they are the tests the project relies on to notice a changed simulation.

The change is one scenario: spawn a few units, tell them to build, step, and
record the hash sequence. The golden file is shared, so this waits for a
session that is not regenerating it for another reason.

## References

[^1]: Findings register, FND-174. `docs/FINDINGS.md`
