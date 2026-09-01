---
id: 0011
title: Record the terrain multiplier and reconcile the crossing time
status: proposed
created: 2026-08-30
---

A crossing time depends on the terrain multiplier that scales the step cost of
a tile. No record states that multiplier. The accepted 50-second mountain
crossing implies a value of 2 against ordinary ground, but the value is
inferred from a result rather than decided.

Two loose ends need closing. First, the multiplier needs a recorded value and
a recorded home. DEC-017 asks whether that home is content or engine code.
Second, a timing check measured 12.9 seconds for a dwell-2 baseline with a
capacity-16 crossing, and the closed-form throughput law gives 12.5 seconds
for the same parameters. The 4-tick difference is unexplained. The likely
cause is the entry and clearing ticks that the steady-state law omits, but
nobody has verified that.

ADR-0056 is allocated and unwritten. It claims that movement is tile-discrete
and admitted by sort-then-admit. This work feeds that record. The impact
review must say whether the multiplier belongs in ADR-0056 or in a separate
record.

BLK-001 is now answered. The tile edge is 80 metres, the dwell is 2, and the
crossing-terrain capacity is 16. The scale constants table holds these values,
so the movement constants no longer need to stay parametric. This item is
unblocked on that count.
