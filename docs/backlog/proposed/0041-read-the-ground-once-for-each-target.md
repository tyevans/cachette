---
id: 0041
title: Read the ground once for each target of a frame
status: proposed
created: 2026-08-31
---

Admission reads the capacity of each target from the ground. The intent half
has already read the ground of that same target, to test whether it admits a
unit at all. The frame therefore generates the ground of a target twice.

The ground is a pure function of the seed and the address, and generating one
tile sums four octaves of two fields. It is the largest single cost in
admission: about half of it before the segment table was built once, measured
on a development machine.

Two ways to remove the second read.

**The intent carries the capacity.** ADR-0056 D2 says an intent names the unit
and the tile it wants. A third field is a value derived from the second, and
the record says a second copy of a fact is the shape to avoid. The impact
review must say whether a value derived inside one frame from an unchanging
function counts as that shape. The argument that it does not is that the
ground cannot change during a frame, so the copies cannot disagree.

**The intent half writes a table of the targets it touched.** Admission then
reads the capacity from that table. This keeps the intent as the record
describes it and moves the cost into the parallel half.

Either way the work is a cost change and not a behaviour change, so the golden
state hash must not move. That is the check.

Do not refine this before a measurement exists on the target platform.[^1] The
figures above are from a development machine, and the cache line differs.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
