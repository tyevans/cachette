---
id: 0034
title: Measure the generated terrain against a stored one
status: proposed
created: 2026-08-30
implements: [ADR-0068 D1]
changes: []
creates: []
serves: [PRD-0003]
blocked-by: [BLK-007]
---

The terrain record chooses generation over storage. The reasoning is that the
saving is arithmetic and the cost of the alternative is the largest
allocation in the engine. Nobody has measured either side.

The work benchmarks a whole-world read of the generated ground against a
whole-world read of a stored column, on the target platform, at the target
tile count. It then states whether a cache is worth writing, and what shape
it would take.

The blocker governs this item. No measurement exists on the target platform,
and a measurement taken on a development machine misleads, because the cache
line size differs. The item cannot close until a benchmark harness and a
machine exist.

The record already says that a cache is a separate decision, and that the
generated ground stays the source of truth. This item feeds that decision. It
does not reopen the record.
