---
id: 0291
title: Stop the holding spread walking the population
status: proposed
created: 2026-09-03
implements: []
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

## What is missing before this is refined

- **Which part of the candidate list costs.** The walk over the held tiles,
  the walk over the units, and the sort with the deduplication are one row
  today. The stage table takes a nested row, so dividing them further is
  cheap.[^4]
- **Whether the unit walk is needed at all.** A unit contributes a candidate
  only where it can change who holds a tile. A unit standing well inside its
  own holding changes nothing, and the held tile walk already covers the edge
  of a holding.
- **Whether the list can be derived from the lattice.** A level 1 cell knows
  which factions hold ground in it. A cell in which one faction holds
  everything has no candidate, and the register says the units occupy 14,970
  cells of 16,384 at the target density.[^1]
- **Whether the sort can go.** The list is sorted to make the order
  independent of the arena order. A list built by walking tiles in ascending
  index needs no sort.
- **What the two determinism tests say.** Any change here changes what order
  the candidates are decided in, and the record binds that order.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, every stage of a frame by name. `docs/reference/graviton-costs.md`
[^2]: Findings register, FND-277. `docs/FINDINGS.md`
[^3]: ADR-0096, cost follows the lattice, not the population, decisions D1 and D3. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^4]: Backlog item 0289, price every stage of a frame by name. `docs/backlog/complete/0289-price-every-stage-of-a-frame-by-name.md`
[^5]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
