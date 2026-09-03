---
id: 0277
title: Hold a thread back when the work will not pay for it
status: proposed
created: 2026-09-03
implements: []
changes: [ADR-0087]
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The influence solve gets slower with every thread added, on a small world.**
It is 5.9 times slower at twelve threads than at one on the demonstration
world, and it costs more there in absolute terms than it does at the target
extent, on a world with 256 times fewer tiles. The register holds the
measurement.[^1]

**One relaxation pass opens a thread scope for each faction, and a solve runs a
fixed count of passes.** A frame therefore opens the pass count multiplied by
the faction count of them, and each one spawns up to the thread count. At the
target extent every spawned thread relaxes thousands of cells and the spawn is
paid back. On a small world each gets a handful and the spawn is the whole
cost.

**The guard that exists cannot catch it, and it is right not to.** It holds a
thread back only when the cell count is at or below the thread count, on the
stated ground that the rule reads two numbers the caller already supplied and
holds no constant of its own. A world of any interesting size has more cells
than threads, so the guard never fires.

**This is the reason the demonstration runs at the rate it does.** Both front
ends ask for the smaller of the machine's parallelism and twelve. The frame is
between two and three times faster with fewer, with no engine change.

## Done when

- The solve does not lose time by being given threads it cannot use.
- The rule that decides is stated, and if it holds a constant then that
  constant is derived and named rather than read off one machine.
- The state hash is unchanged at every thread count and every extent that the
  determinism tests cover.
- A test can fail. Put the old shape back and watch a cost assertion notice.

## What makes this hard

**Any work-per-thread rule introduces a constant, and the current code was
written to avoid one.** The comment says so in as many words. A threshold read
off this development machine is exactly the kind of figure the project keeps
removing, and the target platform is the only one whose numbers bind.

**Hoisting the scope across factions is the constant-free alternative and it
contradicts a record.** The factions are independent, so one scope could serve
all of them and cut the spawn count by the faction count. But one scratch plane
serves every faction today, on a stated decision, and running factions together
needs one scratch each.[^2] That is a memory-for-time trade, and the register
holds an open decision about whether that trade is now available.[^3]

**So this is a decision before it is a change**, which is why it is filed
rather than done.

## What is already known

Measured on an x86-64 development machine, four factions, the mean of 120
frames after 30 warm-up frames. The state hash is identical between the two
thread counts at every extent, so nothing here is a determinism question.

| Extent | Tiles | Threads | The frame | `influence_solve` | State hash |
|---|---|---|---|---|---|
| 256 | 65,536 | 1 | 39.264 ms | 9.320 ms | `9d81e94936b9f445` |
| 256 | 65,536 | 12 | 191.613 ms | 131.728 ms | `9d81e94936b9f445` |
| 512 | 262,144 | 1 | 147.656 ms | 23.865 ms | `08f75d4b4a298532` |
| 512 | 262,144 | 12 | 86.018 ms | 54.371 ms | `08f75d4b4a298532` |
| 1024 | 1,048,576 | 1 | 187.753 ms | 17.974 ms | `384a1a5c89e235f9` |
| 1024 | 1,048,576 | 12 | 259.005 ms | 136.601 ms | `384a1a5c89e235f9` |

**The frame crosses over between 256 and 512. The solve does not cross over at
any extent measured.** At 512 the whole frame is faster with twelve threads
while the solve inside it is still more than twice as slow, so other stages
carry the win. At 1,048,576 tiles the solve is still 7.6 times slower with
twelve threads than with one. The extent at which threads start to pay this
stage is above a million tiles, and the guard therefore has to be about the
stage rather than about the frame.

**The absolute figures are noisy and the direction is not.** The machine ran
other builds throughout, and one row was measured at 81 ms in an earlier sweep
and 192 ms here. Every point in both sweeps has the same sign. Read the ratios,
not the milliseconds, and read nothing here as a figure about the target
platform.

## References

[^1]: Findings register, FND-294. `docs/FINDINGS.md`
[^2]: ADR-0060, an influence map is stored as a shared basis, decision D4. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^3]: Decisions register, DEC-105. `docs/DECISIONS.md`
