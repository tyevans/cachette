---
id: 0062
title: Hold ranked positions at a site
status: complete
created: 2026-08-31
implements: [ADR-0066 D1, ADR-0014 D1, ADR-0004 D1, ADR-0056 D4]
changes: []
creates: [ADR-0065]
serves: [PRD-0017]
blocked-by: [0052]
---

## Why

A unit holds a job that nothing can change. A place that runs out of food
holds the same farmers it held before, so a place cannot respond to its own
situation and a shortage ends in a number that nothing reads.

Item 0063 does the assigning. **This item builds the thing that is assigned
to**, and separating them is deliberate: a position is a structure, and an
assignment is a rule that runs over it. Building both in one change would make
one pull request that rewrites the site pass and the unit pass together.

## What the work does

1. A site holds a small fixed number of ranked positions. Each position names
   a kind of work and a rank.
2. A position holds one unit or nobody. A position that holds nobody is a
   representable state, not an absence.
3. The number of positions of each kind responds to what the site has and
   lacks, at an interval.
4. One command from the control plane changes what a set of sites prefers. The
   command names no unit.

## Impact review

**Governed by.**

- ADR-0066 D1. The positions are columns of the settlement shape. A position
  is not a fifth entity shape and it is not a named entity; making it one
  would spend the character tier on shopkeepers.[^1]
- ADR-0014 D1. A position names a unit by its generational identity, so a
  position never holds the identity of a unit that died and was replaced in
  the same slot.
- ADR-0004 D1. The positions of a site are visited in index order.
- ADR-0056 D4. A tile holds a bounded number of units, and that bound is a
  data-driven property of the terrain. **The position count of a site on one
  tile must not exceed it**, because a site cannot usefully hold more workers
  than can stand in it. Two limits on one quantity is the first recurring
  defect shape, so the two must agree by construction rather than by
  comment.[^2] [^3] [^4]

**Changes.** No record changes.

**Creates.** ADR-0065. The registry reserves the row and states the claim: a
group is a site membership, not a region.[^5] This item writes that record.
The claim passes the three-condition test, and the evidence is already in the
register: **FND-010 records that a region is not stable under movement**, so a
membership defined by a region changes its own recipient set between
frames.[^6] A contributor could reasonably define the workforce of a place as
the units standing in it, and that is precisely the shape the finding rejects.

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-005 gives the settlement count and BLK-009 gives the tile capacity that
bounds the position count.[^7] [^8] BLK-010 is resolved and its resolution is
the same claim in the military case: membership is an ownership column plus a
reverse index, and it is not a spatial region.[^9] **ADR-0065 must state the
civilian and the military case as one claim, or the project has two
declarations of one rule.**

**Precedent.** FND-010 is the evidence for the record.[^6] Shape 1 of the
recurring defect rule governs the position count against the tile capacity.[^2]

**Serves.** PRD-0017.

**Conflict surface.** `crates/cachette-core/src/site.rs` and
`crates/cachette-core/src/world.rs` at the state hash and the invariant check.
`crates/cachette-py` gains the set-valued preference command. It shares
`site.rs` with items 0055, 0056 and 0059, so it merges after whichever of them
is in flight.

## Done when

- A site answers what positions it holds and what kind each one is.
- A position holds one unit or nobody, and nobody is a representable answer.
- The position count of a site never exceeds the capacity of the tile it
  stands on, and the two bounds come from one place. A test asserts that
  raising one raises the other.
- The number of positions of each kind changes at an interval in response to
  what the site holds, and the interval is a schedule parameter.
- One command from the control plane changes the preference of a set of sites,
  and the command names no unit. A test drives the command from Python.
- A property test asserts that the position tables are identical at 1, 2 and
  12 threads.
- The invariant check fails when a position names a unit that no longer
  exists, and a test proves that it fails.
- ADR-0065 is written, the registry row moves to `Draft`, the record states
  both the civilian and the military case as one claim, and it holds no count
  and no cost figure.
- `just check` runs green.

## Outcome

A site now holds a fixed-width row of ranked positions. Each position names a
kind of work and a rank, and it holds one unit or nobody. The engine releases a
dead holder on every frame, and it resizes the row on an interval that the
world carries as a parameter. One command from the control plane changes what a
set of sites wants, and it names no unit.

**The row width and the tile capacity are one number.** The width folds the
terrain capacity table, so raising a capacity raises the width. A test asserts
the equality against the table rather than against a literal.

**The record was written and the registry row moved to `Draft`.** ADR-0065
states the civilian and the military case as one claim, because the register
already held the military half.[^9] The author did not review it.

### Three defects were put back, one at a time

**A bare slot index in place of the identity.** Four tests failed, including
the one that reuses the slot of a dead holder. The thread-count test stayed
green, and so did every test of the settlement arena and of identity
resolution: the defect gives the same wrong answer at every thread count, which
is the shape the register already names.[^10]

The first attempt at this experiment reported a false green. The fixture put
the unit under test in slot zero, and a holder field of zero means nobody, so
the wrong implementation answered correctly. A filler unit now takes slot zero.
The finding holds the detail.[^11]

**The release of a dead holder moved behind the rebalance interval.** Every
test stayed green, because the fixture rebalanced on every tick and could not
tell the two cadences apart. A test with a long interval now covers it.[^12]

**The pass sized the row from the width instead of the tile capacity.** Every
test that drives a world stayed green, and none of them can fail on it. The
founding refuses ground that admits nobody, and every other ground carries the
same capacity as the width, so the two numbers are equal wherever a site can
stand. One test drives the pass directly over ground that admits nobody, and it
is the only thing that fails.[^13]

### What was left undone

No blocker was opened and none was closed. The work found no unanswered
question that stops anything.

The map from a kind of work to the commodity it fills is a placeholder, because
the commodity set holds one entry. The register holds the open choice.[^14] A
follow-up item asks for the real map when the economy holds more than one
commodity.

The rebalance is not driven by anything the simulation produces beyond the
store. What a site wants comes from the control plane or from the value a
founded site starts with.

The golden state hash moved. The hash now covers the position table and the
rebalance schedule, so it moves for every world including one that has never
stepped.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^5]: ADR Registry, row 0065. `docs/adrs/REGISTRY.md`
[^6]: Findings register, FND-010. `docs/FINDINGS.md`
[^7]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^8]: Blockers register, BLK-009. `docs/BLOCKERS.md`
[^9]: Blockers register, BLK-010. `docs/BLOCKERS.md`
[^10]: Findings register, FND-160. `docs/FINDINGS.md`
[^11]: Findings register, FND-178. `docs/FINDINGS.md`
[^12]: Findings register, FND-179. `docs/FINDINGS.md`
[^13]: Findings register, FND-177. `docs/FINDINGS.md`
[^14]: Decisions register, DEC-073. `docs/DECISIONS.md`
