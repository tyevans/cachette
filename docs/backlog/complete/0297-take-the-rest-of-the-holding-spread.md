---
id: 0297
title: Take the rest of the holding spread
status: complete
created: 2026-09-03
implements: [ADR-0009]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The holding spread was 40.5 percent of a frame, and item 0291 left two thirds
of it.** That item replaced the pass that chooses which tiles to decide, which
fell from 400.9 milliseconds to 16.8. What it did not touch was the pass that
decides them, at 69.7 milliseconds, and the pass that writes the decision, at
34.0. Together those are 34.6 percent of a 300.0 millisecond frame on the target
platform.[^1]

The pass that writes ran on the calling thread. A serial pass at 11 percent
bounds every thread count above it.

## Impact review

**Governed by.** ADR-0001 binds every change to the step: one binary gives one
answer at any thread count.[^2] ADR-0004 D1 binds the iteration order to
something explicit and stable.[^3] ADR-0009 binds a parallel stage to disjoint
outputs, to a combine in an order the data fixes, and to a partition derived
from the data.[^4] ADR-0018 D2 and D4 state what the derived unit structure
holds and how it answers.[^5] ADR-0053 D4 states what the spread is for.[^6]
ADR-0068 D1 states that the ground is generated from the seed and never stored
as a map.[^7]

**None is contradicted.** The deciding pass reads the ground later than it did
and reads it no differently, so nothing here stores a map of it. The writing
pass cuts the tile space at values taken from its own list, so a tile falls in
exactly one band, a band writes its own buffer, and the join reads the buffers
in band order. The walk through the derived unit structure returns the same
units in the same stored order as the search it replaces.

**One decision was discovered and it needed no record.** The walk through the
derived unit structure is correct only while the caller visits tiles in
ascending order. That is a precondition on a private function, it is stated on
the function, and a test drives it against the search it replaces. A future
contributor could reasonably choose otherwise, but changing it back costs what
changing it cost, so the second condition of the scope test fails.[^8]

**No blocker governs a value here.** Every figure is measured on the target
platform.

## Done when

- The pass that writes takes a thread count and the stage declares that it does.
- The two determinism tests pass, and the golden state hash does not move.
- A measurement on the target platform gives the cost before and after.
- The walk through the derived unit structure has a test that fails when the
  walk is wrong.

## Outcome

**Three changes, and the ablation chose all three.**

**The ground is read last, because it can only refuse.** The rule read the
ground of a candidate before it gathered any support. The ground says how much
support a tile asks for, and it never turns a losing challenger into a winning
one, so a tile whose best challenger does not beat the holder keeps its holder
whatever the ground says. The read is a draw from the seed rather than a load,
and the pass was making 4.8 million of them in each frame.[^9]

**The derived unit structure is walked rather than searched.** Asking which
units stand on a tile was two binary searches into an eight megabyte key array,
once for each candidate. The low part of a bridge key is the row-major offset of
the tile inside its block, so a caller that visits tiles in ascending order asks
for ascending keys inside every block. One cursor for each block replaces the
search.[^10]

**Both repairs after a write take a thread count.** The write itself is 7
percent of the writing pass. Rebuilding the held list is 43 percent and
rebuilding the mask of every dirty block is 50 percent, and neither follows the
number of tiles that changed.[^11]

**What it bought, on `c7g.4xlarge` at 16,777,216 tiles, 1,000,000 units
scattered and 12 threads.** The figures are filled in from the run named in the
cost register.[^12]

**What is left.** Rebuilding the held list ends in one serial copy of the whole
list, because a band does not know its own length until it has merged. Both
repairs still follow the holding rather than the change, and a holding that
reaches the whole world will bring them back.[^11]

## References

[^1]: Target platform costs, every stage of a frame after the candidate pass became a bit plane. `docs/reference/graviton-costs.md`
[^2]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D2 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^7]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: Findings register, FND-299. `docs/FINDINGS.md`
[^10]: Findings register, FND-295. `docs/FINDINGS.md`
[^11]: Findings register, FND-294. `docs/FINDINGS.md`
[^12]: Target platform costs, every stage of a frame after the ground read moved last. `docs/reference/graviton-costs.md`
