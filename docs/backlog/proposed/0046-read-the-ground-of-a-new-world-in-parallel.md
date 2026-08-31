---
id: 0046
title: Read the ground of a new world in parallel
status: proposed
created: 2026-08-31
---

Building a world reads the ground of every tile once, to fix the part of each
level 1 cell that the ground decides. It runs on one thread, because no caller
states a thread count when it builds a world.

Measured on a development machine, in a release build:

| world | time to build |
|---|---|
| 128 x 128, 16 384 tiles | 20 ms |
| 640 x 440, 281 600 tiles | 240 ms |

The cost is linear in the tile count, so the target world of 16.7 million
tiles extrapolates to about 14 seconds. That figure is an extrapolation from a
development machine and not a measurement, and the blocker governs every cost
figure in this project.[^1]

The read is a pure map over the blocks, and each block writes its own cell, so
it parallelises without any ordering question. An earlier version did exactly
that and was removed, because `World::new` passed a thread count of one and
the branch was one nothing took.

The work is to give a caller a way to state the thread count when it builds a
world, and then to take it. The impact review must say whether that belongs on
`WorldConfig`, which is content, or on a second constructor, and whether a
world that takes seconds to build wants a progress report rather than a faster
sweep.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
