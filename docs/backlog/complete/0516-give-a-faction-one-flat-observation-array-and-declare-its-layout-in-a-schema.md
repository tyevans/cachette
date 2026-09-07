---
id: 0516
title: Give a faction one flat observation array, and declare its layout in a schema
status: complete
created: 2026-09-06
implements: [ADR-0154 D1, ADR-0154 D2, ADR-0154 D3]
changes: []
creates: []
serves: [PRD-0056, PRD-0001]
blocked-by: [BLK-007]
---

## Why

**A learner reads one array on every decision, and the engine offered none.**
An accepted record says what that array is. The observation of one faction is
one flat array of signed integers, its length is a function of the world
parameters and never of the population, and the engine returns a schema that
names each field, where it starts, how many positions it holds, its width and
the bound of each position.[^1] [^2] No file outside the engine may state an
offset, a length or a bound.[^1]

The engine held neither half. No observation reader existed, and no schema call
existed for anything but the declared event layouts.[^3]

The item before this one gave the engine three faction-scoped readers: a tile,
a masked cell summary and a unit set.[^4] Those readers answer one question at
a time. An observation is a whole map at once, and a point query is the wrong
shape for a whole map.[^5]

## Impact review

**Governed by.** ADR-0154 D1 fixes the schema. ADR-0154 D2 fixes the array as
one flat integer array whose length follows the world parameters. ADR-0154 D3
binds the reader to the sight rule and forbids an argument that widens the
answer. ADR-0022 D2 says the fog layer and the summary level share one
lattice.[^6] ADR-0059 D2 and D4 say what a faction reads and what it cannot.[^7]
ADR-0002 D1 forbids a floating point value in the array.[^8] ADR-0004 D1 fixes
the iteration order.[^9] ADR-0084 D1 gives the reserved unit count, which is a
world parameter and not a population.[^10]

**Changes.** None. No record changed and no record was superseded.

**Creates.** None. ADR-0154 already holds every decision this work
implements, so the work states no decision the records do not hold.

**Blockers.** BLK-007 governs every cost figure of this project.[^11] The work
states no measured bound. A bound that the world parameters do not give reads
as the whole range of the position width, and the code says so.

**Precedent.** FND-568 says the standing of a faction reports the work toward a
victory claim, while the wonder reader compares the claim itself.[^12] A learner
reads the standing, so the array carries both quantities. FND-569 says a plane
over a padded lattice has two address spaces, and that a reader which confuses
them fails nowhere.[^13] The array is indexed by the cell index of the block
lattice, and that lattice carries no margin.

## What the work decided

The item asked five questions before it could be refined. Each one is answered
below.

**Which fields the array holds, and in what order.** The array holds four
groups: the frame, the standing of the faction, its relation row, the public
trade board and its own weight vector, and then the map. Every field is
contiguous, and the map fields are laid out one field at a time rather than one
cell at a time. A field that holds one position for each cell therefore needs no
stride, and the schema states a start and a length and nothing else.

**Every length and every bound comes from the world parameters, in code.** The
item expected a row in the reference tables for each one. A reference row would
be a second declaration site of a value the world already holds, and nothing
would fail when the two disagreed.[^14] The code derives each bound from the
faction count, the tile count, the reserved unit count, the board size and the
block edge instead. No literal states a bound.

**What the array says about a place the faction has never seen.** It answers
with zero, and two positions of each cell say how much of that cell the faction
holds. A caller therefore tells an unobserved cell from an empty one, which is
the property the tile reader gives for one tile.[^7] A flat array cannot refuse
one position, so a refusal was never an option, and a second mask would repeat
what the two count positions already say.

**The version integer is this item.** The engine carries it beside the schema,
and the schema returns it. A trained policy therefore has something that says
which layout produced it.

**What the reader costs.** The reader walks the cells the faction observed, and
no other cell. The measurement is in the commit body. It is a figure from one
development machine, and BLK-007 keeps it derived until the target platform
measures it.[^11]

**How a test proves the schema and the array cannot disagree.** The writer
indexes the array through the schema, and a match over the field list that the
compiler checks for coverage fills it. A test asserts that every start follows
the field before it and that the fields fill the array with no gap. The fog test
beside it compares against the truth of the same cell, so a reader that leaked
the truth fails it.

## Done when

- The engine returns one flat array of signed integers for one faction, and one
  schema that declares its layout. Done.
- No file outside the core crate states a position, a length or a bound. Done.
  The type stub names the keys of the schema and no offset.
- The length follows the world parameters and never the population. Done, with
  a test that raises the population and reads the same length.
- A test drives the engine, moves a unit, and shows the array change in the way
  the fog says it should. Done.
- A cell the faction has never seen reads as nothing, and the test can fail.
  Done. The defect was put back and the test went red.
- The array carries the quantity the wonder reader compares. Done.
- One thread count and another give one array. Done.

## Outcome

The work landed the array, the schema and the version integer. It also moved
the masking rule of one cell into one place, which the whole-lattice read and
the per-cell reader now share.

**What changed from the plan.** The item expected the bounds to become rows of
the reference tables. They became derivations from the world parameters in the
code instead, for the reason above.

**What the work found.** The saving came from the mask and not from the loop
shape. A cell the faction has never seen a tile of now costs no tile work at
all, because the block form answers for the whole block. That shortcut sits in
the shared rule, so the per-cell reader gained it too. The whole-lattice read
saves the layer lookup on top of that, and that saving is the smaller of the
two. The findings register holds this.[^15]

**Registers.** FND-572 opened. No blocker opened or closed. DEC-276 stays open,
and the work sharpened it: an observation reports a remembered cell with its
ground and no upgrade, so a learner cannot see a city it watched a rival raise.

**What the work left undone.** The action table, the legality answer, the
reward, the batch step, the environment wrapper and the per-unit mask are all
later items. The observation of a remembered place still holds no upgrade and
no holder, which is the question DEC-276 asks.

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^4]: Backlog item 0495, build the observation plane and let every reader answer for one faction. `docs/backlog/complete/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^5]: Project orientation, the design principles. `AGENTS.md`
[^6]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^7]: ADR-0059, fog storage grows with observed area, not with world area, decisions D2 and D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^8]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0084, the world reserves the unit columns at construction, decision D1. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-568. `docs/FINDINGS.md`
[^13]: Findings register, FND-569. `docs/FINDINGS.md`
[^14]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^15]: Findings register, FND-572. `docs/FINDINGS.md`
