---
id: 0291
title: Stop the holding spread walking the population
status: complete
created: 2026-09-03
implements: [ADR-0009]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The holding spread is 61 percent of a frame, and no item named it.** It
costs 514.3 milliseconds of an 836 millisecond frame at the target scale, on
the target platform, at 12 threads. It is the largest single cost in the
engine by a factor of four over the next stage.[^1]

Nobody knew, and the reason is in the register. The earlier split priced a
pass by running a frame without it, and this pass has no switch, so the
largest thing in a frame was invisible to the method that existed to find
it.[^2]

**The pass builds its candidate list on the calling thread, and that list is
49 percent of the whole frame.** It costs 400.8 milliseconds, measured. It
walks every held tile and pushes the tile and its six neighbours, then walks
every live unit and pushes the tile it stands on, then sorts the whole vector
and removes the duplicates. At one million units that is a serial sort of
several million indices in every frame.

**The half that decides costs 71.1 milliseconds and takes a thread count. The
half that chooses what to decide about costs five and a half times as much and
takes none.** The join and the write are 32.7 milliseconds. Those three are
the whole of the spread to within 3.3 milliseconds.

**The record on cost forbids exactly this.** It says the cost of a pass
follows the lattice and never the population, and that a pass which must end
by touching units applies its results in the order the lattice produced
them.[^3] The decide half of this pass already takes a thread count. The half
that decides what to decide about does not.

## Impact review

**Governed by.** ADR-0001 binds every change to the step: one binary gives one
answer at any thread count.[^5] ADR-0004 D1 binds the iteration order to
something explicit and stable.[^6] ADR-0009 binds a parallel stage to disjoint
outputs, to a combine in an order the data fixes, and to a partition derived
from the data.[^7] ADR-0053 D4 states what the pass is for.[^8]

**None is contradicted.** The pass now builds the candidate set in a bit plane
over the tiles and reads the plane back in ascending word order, so the result
is the same set in the same order at every thread count. The plane is divided
into one chunk for each thread, the chunks are joined in slot order, and the
division is the held list divided by the thread count. No thread reads a word
another thread writes, and nothing reads a completion order.

**No decision needed a record.** The three-condition test asks whether a
contributor could reasonably choose otherwise, whether choosing otherwise costs
more than changing it later, and whether the reasoning is invisible in the
artefact.[^9] The first is true: pushing indices and sorting them is the
ordinary way to build this list. The second is false. The set representation is
private to one function, it reaches no interface and no file, and replacing it
again costs the same as replacing it did. The reasoning is in the function.

**No blocker governs a value here.** Every figure in this item is measured on
the target platform.

## What the item asked, and what the answers were

- **Which part of the candidate list costs.** The sort was 68 percent of the
  pass, the walk over the held tiles 31 percent, and the walk over the units
  1.4 percent. A probe divided them, inside the stage apparatus that item 0289
  built.[^4]
- **Whether the unit walk is needed at all.** It is, and it was never the
  problem. It is 6.7 percent of the raw list, not the three quarters the plan
  assumed. A unit takes ground nobody holds, so its tile is a real candidate.
  The figure that made it look large came from the demonstration world, and a
  finding holds the correction.[^10]
- **Whether the list can be derived from the lattice.** Not attempted. The pass
  is now 3.5 percent of a frame, so the question is no longer worth its cost.
- **Whether the sort can go.** It can, and it did. The answer is a set, so the
  pass builds a set: one bit for each tile of the world, read back in ascending
  word order. Two sources that reach one tile set one bit twice.
- **What the two determinism tests say.** They pass unchanged, and the golden
  state hash did not move. The pass emits the same tiles in the same order as
  before, so the tests could not have moved.

## Done when

- The candidate pass takes a thread count and the stage declares that it does.
- The two determinism tests pass, and the golden state hash does not move.
- A measurement on the target platform gives the cost before and after.

## Outcome

**The pass costs 16.7 milliseconds instead of 400.9.** That is 24.0 times
less, and it is now 3.6 percent of a frame instead of 49.1 percent. The
holding spread costs 125.0 milliseconds instead of 514.3. A frame costs 463.4
milliseconds instead of 825.4, which is 4.6 times the budget instead of
8.3.[^11] Measured on `c7g.4xlarge` at 16,777,216 tiles, 1,000,000 units
scattered and 12 threads, before and after, by the same script.

**Two changes did it, and the sort was the larger.** The pass built a list of
14.9 million tile indices and ordered it with a comparison sort. It now sets
one bit for each candidate in a plane over the tiles and reads the plane back
in ascending order, which is linear in the world and needs no comparison. The
walk over the held tiles then took a thread count, which the pass never had.

**The output did not change, and that is why the golden hash could not move.**
The pass emits the same set of tiles in the same ascending order as before. A
change that had moved the golden file would have been a defect here, and the
shape of the change is what rules it out rather than the test passing.

**Three things are left, and none is worth an item on its own.**

The pass still walks every live unit, which is about 2 milliseconds of the
16.7. The record on cost asks that a pass follow the lattice rather than the
population, and this one still does not.[^13] The walk is a seventh of a pass
that is a twenty-eighth of a frame.

The pass allocates its plane on every frame, and 12 milliseconds of the frame
moved into the part that no stage measures. Holding the plane across frames is
the change the evidence points at, and it asks what a copy of a holding means,
which is a design question and not a repair. A finding holds the case and
records that the number of mappings was ruled out as the cause.[^12]

**The frame is still 4.6 times its budget.** The three largest stages are now
the holding spread at 27 percent, the change merge at 26 percent and the level
1 rebuild at 17 percent. The change merge is the largest serial pass in the
engine, and item 0292 holds it.

## References

[^1]: Target platform costs, every stage of a frame by name. `docs/reference/graviton-costs.md`
[^2]: Findings register, FND-277. `docs/FINDINGS.md`
[^3]: ADR-0096, cost follows the lattice, not the population, decisions D1 and D3. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^4]: Backlog item 0289, price every stage of a frame by name. `docs/backlog/complete/0289-price-every-stage-of-a-frame-by-name.md`
[^5]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^8]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^9]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^10]: Findings register, FND-285. `docs/FINDINGS.md`
[^11]: Target platform costs, every stage of a frame after the candidate pass became a bit plane. `docs/reference/graviton-costs.md`
[^12]: Findings register, FND-286. `docs/FINDINGS.md`
[^13]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
