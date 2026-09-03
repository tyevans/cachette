---
id: 0298
title: Walk the tables admission searches
status: complete
created: 2026-09-03
implements: [ADR-0009]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**Admission was 17.55 percent of a frame and the bridge rebuild was 16.37
percent, and together they were more than a third of it.** Both already took a
thread count, so neither was the serial-pass problem that the passes before them
had. The question was what each does that it need not do.[^1]

## Impact review

**Governed by.** ADR-0001 binds every change to the step: one binary gives one
answer at any thread count.[^2] ADR-0004 D1 binds the iteration order to
something explicit and stable.[^3] ADR-0009 binds a parallel stage to disjoint
outputs and to a combine in an order the data fixes.[^4] ADR-0018 D2 and D4
state what the derived unit structure holds and how it answers.[^5] ADR-0056 D2
and D3 state that movement is admitted by sort-then-admit and that admission
reads the occupancy the last barrier built.[^6] ADR-0071 D2 states that the
bridge rebuild orders on one thread and accepts no thread count.[^7]

**None is contradicted, and one was already being contradicted before this
work.** Three stages that wrap the bridge rebuild declared that they take a
thread count, and ADR-0071 D2 says the rebuild accepts none. Measured at one
thread and at twelve, the stage does not improve. The declarations are now
`false`, and a finding holds the case.[^8]

**The two replacements answer the same questions the searches did.** The forward
reader over a count table and the walk through a block both rely on the caller
asking in ascending order, which each caller already does because a sort put it
in that order. Both have a test that drives them against the search they replace
and compares.

**One decision was discovered and it is not made here.** Every ordering pass
sorts its whole key set a second time to check that no identifier repeats, and
the property the engine needs is narrower and free. Narrowing the contract of a
shared sort is a determinism decision, so the scope rule says it gets a record.
The decisions register holds the options with a recommendation.[^9]

**No blocker governs a value here.** Every figure is measured on the target
platform.

## Done when

- The searches admission repeats for each segment are gone.
- The thread-count declarations agree with the records that govern them.
- The two determinism tests pass, and the golden state hash does not move.
- A measurement on the target platform gives the cost before and after.
- Each forward reader has a test that fails when the reader is wrong.

## Outcome

**Admission costs 21,274,145 nanoseconds instead of 32,973,316**, which is 35.5
percent less. A frame costs 177,862,658 instead of 187,862,216. Measured on
`c7g.4xlarge` at 16,777,216 tiles, 1,000,000 units scattered and 12 threads.
Every other stage is unchanged to within the spread of the apparatus.[^10]

**Two searches went, and both were removed by an order the caller already had.**

The grant passes read an arrival count and a departure count for each segment,
by binary search over tables that reach one entry for almost every target tile.
The segments are in ascending tile order and both tables are too, and neither
changes while the passes run, so one forward reader for each replaces every
search. Those passes were 60 percent of the stage.[^11]

Reading how many units stand on a target was two binary searches into an eight
megabyte key array, once for each segment. The walk that the holding spread
already used replaces it, and the walk now lives on the derived unit structure
rather than inside the spread, so it is written once and both callers use
it.[^12]

**The bridge rebuild is untouched, and the reason is a record.** It is now the
largest single stage in the engine at 17.65 percent. Sixty-three percent of it
is the ordering pass, which ADR-0071 D2 gives one thread on reasoning about the
histogram, the combine order and the placement offsets.[^7] That reasoning is
unaffected by the size of the stage. What the measurement changes is the
record's premise that the rebuild "does not earn" a thread, and a finding
records that without changing the record.[^8]

**What is left.** Eighty-seven percent of admission still runs on the calling
thread: the sort is 20 percent, the segment table 6 percent and the grant passes
whatever they now cost. The stage still takes one intent for each moving unit,
so it follows the population rather than the lattice.

## References

[^1]: Target platform costs, every stage of a frame after the tile value field became a dense delta. `docs/reference/graviton-costs.md`
[^2]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D2 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decisions D2 and D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^7]: ADR-0071, the bridge rebuild orders on one thread, decision D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
[^8]: Findings register, FND-301. `docs/FINDINGS.md`
[^9]: Decisions register, DEC-111. `docs/DECISIONS.md`
[^10]: Target platform costs, every stage of a frame after admission stopped searching. `docs/reference/graviton-costs.md`
[^11]: Findings register, FND-300. `docs/FINDINGS.md`
[^12]: Findings register, FND-295. `docs/FINDINGS.md`
