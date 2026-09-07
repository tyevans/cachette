---
id: 0521
title: Give each unit the mask of the factions that see it
status: proposed
created: 2026-09-06
implements: [ADR-0059 D3, ADR-0059 D5]
changes: []
creates: []
serves: [PRD-0001, PRD-0056]
blocked-by: [BLK-007]
---

## Why

**An accepted record names three derived projections, and the engine holds
two.** A faction mask for each cell says which factions see that cell. A second
mask for each cell says which factions have ever seen it. **A mask for each
unit says which factions see that unit, and nothing writes one.**[^1]

The record also states which of the three the engine actually asks for. It says
that the question "which factions see this tile" now costs one lookup for each
faction at level 0, that the cell mask answers the cell form of it, and that
**the per-unit mask answers the form the engine actually asks**.[^1] [^2]

A reader that answers which units one faction sees exists today. It walks the
units of every faction and tests each unit's tile against the visible layer of
the reading faction.[^3] That walk costs one lookup for each live unit. The
mask would turn it into one bit test, and it would answer the other direction
of the same question, which is "who can see this unit", at one load.

**Nothing calls a per-unit mask today.** The project rule is to not declare a
capability before something calls it, and a capability that nothing invokes
ships inert.[^4] So this item must name its caller before it builds the mask.

## What is missing before this can be refined

- **Who reads it.** Name the caller. The candidates are the reader that answers
  which units one faction sees, a contest or a meeting pass that must know
  whether a unit is observed, and the flat observation array of the learner
  seat.[^5] An item that names no caller must not be refined.

- **What it costs at the target scale.** The mask is one faction mask for each
  live unit, so its extent follows the unit ceiling and not the world. The
  rebuild must fill it, and the fill is one test for each unit and each faction
  that sees the block the unit stands in. Every figure for that is derived
  rather than measured on the target platform.[^6]

- **Whether it enters the state hash.** The mask is derived from the visible
  layers, and the visible layer is itself derived and deliberately outside the
  hash.[^7] The work must say whether the mask follows the visible layer out of
  the hash, and it must say what makes the two answers consistent.

- **Where the fill runs.** The observation pass gives each worker a disjoint
  set of blocks. A mask indexed by the unit is not indexed by the block, so the
  work must say what keeps two workers off one unit slot, or it must run the
  fill as a separate pass over the units.[^8]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^2]: ADR-0059, fog storage grows with observed area, not with world area, the consequences. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^3]: The faction view of the core crate. `crates/cachette-core/src/faction_view.rs`
[^4]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
[^5]: Backlog item 0516. `docs/backlog/complete/0516-give-a-faction-one-flat-observation-array-and-declare-its-layout-in-a-schema.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: ADR-0059, fog storage grows with observed area, not with world area, decision D5. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^8]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
