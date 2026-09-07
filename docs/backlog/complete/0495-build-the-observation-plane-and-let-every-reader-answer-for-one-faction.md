---
id: 0495
title: Build the observation plane, and let every reader answer for one faction
status: complete
created: 2026-09-05
implements: [ADR-0059 D1, ADR-0059 D2, ADR-0059 D3, ADR-0059 D4, ADR-0059 D5, ADR-0059 D6, ADR-0154 D3, ADR-0004 D1, ADR-0022 D2, ADR-0024 D2]
changes: []
creates: []
serves: [PRD-0001, PRD-0056]
blocked-by: [BLK-007]
---

## Why

**A faction sees everything.** An accepted product record says it must not. A
faction sees a tile when one of its own units observes that tile, a faction
that never observed a tile reads nothing about it, and a faction that moved
away reads what it last saw and knows the reading is a memory.[^1]

A second accepted product record depends on it. A learner plays one faction
while the controllers play the rest, and the first checkable statement of that
record is that the harness never shows a learner anything a player of that
faction could not see.[^3] The harness cannot satisfy that statement while
every reader answers the truth, so this item gates the whole training loop.

An accepted record binds the reader. A caller passes a faction and receives
what that faction sees, no argument asks for the truth, and the engine applies
the rule inside the reader.[^4] **That record decides the reader and decides no
storage.** A second record decides the storage.[^5]

The work had four parts. **Three of them were built before this item was
taken**, and the item stated that all four were missing.[^11]

**The two layers.** Each faction that has observed anything carries a layer of
the tiles it sees now and a layer of the tiles it has ever seen. Each layer is
an array of blocks over the summary lattice, and each block takes one of four
forms.[^5] [^6] Built.

**The rebuild.** A pass at the end of the step rebuilds each layer from the
units that observe it now, and folds the result into the remembered
layer.[^5] Built, and the step calls it on every tick.

**The derived masks.** A faction mask for each cell, and a second mask for what
the factions have ever seen.[^5] Both built. **The third projection that the
record names, a mask for each unit, is not built.** A separate item holds
it.[^12]

**The readers.** This item is the fourth part. Every reader at the boundary
that a faction may act on takes a faction and answers for that faction. A
remembered place answers with the ground the faction last saw and with no unit.
A summary answers by combining only what the same rule admits, so a cell cannot
state what its tiles hide.[^1] [^5]

## The impact review

**The records that govern the work.** ADR-0059 D1 to D6 decide the storage, the
split between the derived layer and the remembered layer, what a remembered
place answers, and that every reader at the boundary answers for one
faction.[^5] ADR-0154 D3 decides that no argument asks for the truth.[^4]
ADR-0022 D2 gives the fog layer and the summary level one lattice, which is
what lets a masked summary walk the tiles of one cell.[^7] ADR-0024 D2 fixes
the fields a summary carries.[^8] ADR-0004 D1 fixes the iteration order of
every walk this item adds.[^9]

**The work contradicts no record.** ADR-0059 D6 says that a world-wide reader
is not a per-faction reader and that nobody may read it as one. It names the
subsystem census and keeps it as a developer reader. The work therefore adds
faction-scoped readers beside the world-wide ones and deletes none of
them.[^10]

**The work creates no record.** Every claim it implements is already stated.

**A decision it opened.** ADR-0059 D4 says a remembered place answers with the
ground of that place as the faction last saw it. The remembered layer stores
membership alone. The generated ground is recoverable from the seed and the
address, and an upgrade is not, so a faction whose rival built on ground it
left has no stored answer. The decisions register holds the options and the
cost of each.[^13]

**A blocker governs its cost figures.** Every cost figure for this subject is
derived rather than measured on the target platform.[^14]

## Done when

- A caller passes a faction and an address and receives one of three answers:
  the present frame for a place the faction sees now, the ground alone for a
  place it saw once, and nothing for a place it has never seen.
- A caller tells the third answer from a place that holds nothing.
- A remembered place reports no unit, no holder and no upgrade.
- A summary reader takes a faction, combines only the tiles the rule admits,
  and states how many tiles it withheld.
- A summary of a cell the faction sees whole equals the cell the summary level
  holds, field for field.
- A caller reads the units one faction sees, and a unit that walked out of
  sight leaves the answer.
- No reader that names a faction takes an argument that widens its answer.
- Every test drives the world step, and each one fails when the reader answers
  the truth.
- The cost of the masked summary rule is measured rather than asserted.

## Outcome

**The four rules hold and six tests drive the step to prove it.** Each test was
run again with the fog check removed from the reader, and each one failed. The
commit body holds the figures and the search commands.

**The masked summary walks the tiles of the cell.** The rebuilt cell counts
tiles the faction has not seen, so the reader cannot use it. A benchmark
measures the walk, the bit test and the combine separately. The mask itself is
cheap. The ground of an admitted tile is not, because the kind, the height and
the stock a tile started with are generated rather than stored, and the reader
generates them for each admitted tile.[^15] The benchmark is in the tree and
its figures are in the commit body. **The figures are from a development
machine and not from the target platform**, so no cost register took them.[^14]

**A point query is the wrong shape for a whole map.** The measurement says that
one masked cell read costs a measurable part of one step of the same world. A
learner that reads every cell of a world each tick needs a pass that derives a
per-faction level once, in the way the summary level is derived. That pass is
not this item, and the item that gives a faction one flat observation array is
where the question belongs.[^16]

**Two parts of the item were already built when it was taken.** The item said
that nothing in the engine implemented any of the sight rule. Item 0108 had
built the two layers, the rebuild and two of the three derived masks. The
findings register holds the correction.[^11]

**The per-unit mask is left undone**, and a separate item holds it.[^12] The
reader that answers which units a faction sees is built, and it derives the
answer from the visible layer rather than from a stored mask, so nothing inert
was added.

## References

[^1]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^3]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^4]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^5]: ADR-0059, fog storage grows with observed area, not with world area. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^6]: Research report 08, fog of war representation. `docs/research/reports/08-fog-of-war-representation.md`
[^7]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^8]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: Research report 31, the state of the learner surface, section 3.3. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^11]: Findings register, FND-568. `docs/FINDINGS.md`
[^12]: Backlog item 0521. `docs/backlog/proposed/0521-give-each-unit-the-mask-of-the-factions-that-see-it.md`
[^13]: Decisions register, DEC-276. `docs/DECISIONS.md`
[^14]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^15]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^16]: Backlog item 0516. `docs/backlog/complete/0516-give-a-faction-one-flat-observation-array-and-declare-its-layout-in-a-schema.md`
