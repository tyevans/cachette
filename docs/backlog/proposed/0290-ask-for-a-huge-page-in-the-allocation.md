---
id: 0290
title: Ask for a huge page in the allocation
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**A frame costs 3.9 percent less on huge pages, and the engine does nothing to
get them.** That figure was measured on the target platform, at the target
scale, by changing a kernel setting for the whole machine. The register holds
the run, the machine, the commit and the setting each row was taken
under.[^1]

**A machine setting is not something the engine controls where it runs.** The
image the measurement used takes `madvise` by default, and under that setting
the engine received no huge page at all: the register reports zero bytes of
its resident set on one, twice. The saving exists and the engine does not have
it.

**Where the saving comes from is known, not guessed.** The stage table divides
it. The change merge gives 15.4 milliseconds, the holding spread 14.4 and the
barrier bridge rebuild 3.4, and every other stage moved by less than half a
millisecond.[^1] Those three passes write scattered over a large array, which
is the shape a translation cost takes.

## What is missing before this is refined

- **Which allocation.** The three stages that gained name the arrays worth
  advising. The tile arrays are one candidate, the bridge another. A blanket
  advice over every allocation is not the same work.
- **How to ask.** The choice is between advice on a mapping the engine already
  holds and an allocator that maps with the flag from the start. The first
  costs no allocation path and the second may waste less.
- **What it costs in memory.** The measurement grew the resident set by
  27,889,664 bytes, which is 2.65 percent, and it advised the whole process. A
  narrower advice should cost less, and nobody has measured that.
- **Whether the target refuses it.** A kernel set to `never` gives nothing
  whatever the engine asks. The work must report that case rather than assume
  a saving it did not get.
- **Who owns the allocation path.** Two other items reshape the same storage,
  and doing this first would make each of the three harder to attribute.[^2]
  [^3]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Target platform costs, huge pages. `docs/reference/graviton-costs.md`
[^2]: Backlog item 0266, order the unit arena by cell. `docs/backlog/refined/0266-order-the-unit-arena-by-cell.md`
[^3]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/proposed/0268-hold-the-cell-index-on-the-unit.md`
