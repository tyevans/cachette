---
id: 0521
title: Give each unit the mask of the factions that see it
status: refined
created: 2026-09-06
implements: [ADR-0059 D3, ADR-0059 D5]
changes: []
creates: []
serves: [PRD-0001, PRD-0056]
blocked-by: [BLK-007]
---

## Why

An accepted decision record names three derived projections of the fog of war,
and the engine holds only two.[^1] A faction mask for each cell says which
factions see that cell now. A second mask for each cell says which factions have
ever seen it. A mask for each unit says which factions see that unit now, and
nothing writes one.[^1]

The record states that the question "which factions see this tile" costs one
lookup for each faction at level 0, that the cell mask answers the cell form of
the question, and that the per-unit mask answers the form the engine actually
asks.[^1] [^2]

A reader that answers which units one faction sees exists today.[^3] It walks the
units of every faction and tests each unit's tile against the visible layer of
the reading faction. That walk costs one block lookup and one intra-block search
for each live unit. The mask turns it into one bit test for each live unit. It
also answers the opposite direction, which is "who sees this unit", in one load.

## Impact review

**Governed by.** ADR-0059 D3 requires the per-unit faction mask projection.[^1]
ADR-0059 D5 and ADR-0164 D2 require that derived projections that are pure
functions of the frame stay outside the state hash.[^1] [^4] ADR-0004 D1 governs
iteration order over the unit slots.[^5]

**Changes.** None.

**Creates.** None.

**Blockers.** BLK-007 governs every derived cost figure this item states.[^6] At
the target scale of 1,000,000 units, the projection stores one two-byte
`FactionMask` for each live unit slot, which costs 2 MB. The rebuild visits each
live unit once, reads its block mask in one load, and checks only the factions
that observe that block.

**Serves.** PRD-0001 and PRD-0056.[^7] [^8]

**Conflict surface.** `crates/cachette-core/src/observation.rs` at `Observation`
storage, rebuild and readers. `crates/cachette-core/src/faction_view.rs` at
`units_seen_by` and `sees_unit`. `crates/cachette-core/src/world/mod.rs` at the
unit mask reader.

**Precedent.** Recurring defect shape 3 warns against shipping an inert
capability without an active caller.[^9] The caller is
`FactionView::units_seen_by`, which now reads the unit mask directly, and
`World::sees_unit`, which answers whether a faction sees an entity.

## Done when

- `Observation` holds a column of `FactionMask` over the soldier arena slots.
- `Observation::rebuild` populates the mask of each live unit during the
  observation pass.
- A dead unit, an unplaced unit, or an unobserved unit has an empty mask.
- A unit standing on a tile observed by a faction has that faction's bit set in
  its mask.
- `FactionView::units_seen_by` reads the unit mask projection.
- `World::sees_unit` and `World::unit_mask` expose the unit visibility status.
- The unit masks do not enter the state hash.
- Unit tests verify mask correctness, prove failure when a mask bit is omitted,
  and verify consistency with `Observation::sees_now`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0059, fog storage grows with observed area, not with world area, decisions D3 and D5. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^2]: ADR-0059, fog storage grows with observed area, not with world area, the consequences. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^3]: The faction view of the core crate. `crates/cachette-core/src/faction_view.rs`
[^4]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^8]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^9]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
