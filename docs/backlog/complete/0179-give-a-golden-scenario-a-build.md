---
id: 0179
title: Give a golden scenario a build
status: complete
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

## Outcome

**Closed as already done. The work landed under other items.** An audit read the
code on 5 September 2026.

**Two golden scenarios build, and each has its own golden file.** The scenario
table holds a wonder row and a building row. The wonder population orders a
build of the wonder category, and the building population defines a build cost
and builds.[^2]

**The wonder run is long enough to finish.** Its frame count is sized so that
the builders complete the wonder inside the run, so the upgrade map is not
empty. That is the case the item was written to reach.[^2]

## References

[^1]: Findings register, FND-174. `docs/FINDINGS.md`
[^2]: The golden state hash scenarios and their golden files. `crates/cachette-core/tests/golden_state_hash.rs`
