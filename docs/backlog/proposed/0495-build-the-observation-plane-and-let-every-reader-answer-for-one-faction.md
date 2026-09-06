---
id: 0495
title: Build the observation plane, and let every reader answer for one faction
status: proposed
created: 2026-09-05
implements: [ADR-0059 D1, ADR-0059 D2, ADR-0059 D3, ADR-0059 D4, ADR-0059 D5, ADR-0059 D6, ADR-0154 D3]
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

**Nothing in the engine implements any of that.** No column, no plane and no
table holds what a faction has observed. Every reader answers the truth of the
world, and the reader that counts each subsystem counts the whole world with no
faction column.[^2] A search of the tree for the words of this subject returns
one reserved bit and one doc comment, and no storage.

A second accepted product record depends on it. A learner plays one faction
while the controllers play the rest, and the first checkable statement of that
record is that the harness never shows a learner anything a player of that
faction could not see.[^3] The harness cannot satisfy that statement while
every reader answers the truth, so this item gates the whole training loop.

An accepted record already binds the reader. A caller passes a faction and
receives what that faction sees, no argument asks for the truth, and the engine
applies the rule inside the reader.[^4] **That record decides the reader and
decides no storage.** A second record decides the storage, and this item builds
what the reader reads.[^5]

The work has four parts.

**The two layers.** Each addressable faction carries a layer of the tiles it
sees now and a layer of the tiles it has ever seen. Each layer is an array of
blocks, one block for each level 2 cell, and each block takes one of four
forms: no payload when the faction sees nothing there, a sorted array of
offsets when it sees few, a bitmap when it sees many, and no payload again when
it sees all.[^5] [^6]

**The rebuild.** A pass at the end of the step marks the blocks that a unit
entered or left, rebuilds each marked block from the units that observe it now,
and folds the result into the remembered layer. Nothing counts how many units
cover a tile. Each worker takes a disjoint set of blocks.[^5]

**The derived masks.** A faction mask for each level 1 cell, a second mask for
what the factions have ever seen, and a mask for each unit that names the
factions which see it.[^5]

**The readers.** Every reader at the boundary that a faction may act on takes a
faction and answers for that faction. A remembered place answers with the
ground the faction last saw and with no unit. A summary answers by combining
only what the same rule admits, so a cell cannot state what its tiles
hide.[^1] [^5]

**This item touches `fn step`. Only one worker may hold it at a time.** It adds
a stage at the end of the step, beside the pass that derives the presence
relation.[^7] It waits for the item that holds `fn step` before it to merge.

## What is missing before this can be refined

- **How far a unit sees, and what carries the number.** The record states that
  sight is a whole number of hex steps, rounded to a fixed set, and it states
  no value.[^5] The balance register holds no row for any of them. The work
  must add a row for the sight of each unit type, a row for the rounded set,
  and a row for the largest sight the engine admits, and every one stays
  provisional until a measurement sets it.[^8] [^9]

- **Whether the world builds the layers for every faction or only for the
  factions that need them.** The research report tiers the factions: a rendered
  faction keeps both layers at the tile, an active faction keeps a capped
  visible layer and a coarse remembered layer, and a passive faction stores no
  visible layer at all.[^6] The record decides no tier. The work must say
  whether the first version builds one form for every faction, and if it does,
  it must say what that costs at the addressable ceiling.

- **Which readers change, and what a changed reader answers about a place the
  faction has never seen.** The record binds every reader that names a faction
  and describes no per-faction form of a world-wide count.[^5] The work must
  list the readers it changes, and it must decide the shape of "nothing": a
  refusal, a zero, or a separate mask that says which entries the caller may
  read. A zero that a caller cannot tell from a real zero is the worse of the
  three, and nothing in the tree settles it.

- **What the remembered layer costs in the state hash.** The layer is state, so
  the hash walks it every frame.[^5] The hash of the whole world is already
  measured on the target platform, and this adds a walk over the blocks of
  every faction.[^9] The work must state what that walk costs and whether the
  hash reads the payload of each block or a running digest the rebuild keeps.

- **How the tests prove the rule can fail.** A determinism test that compares a
  run against itself proves nothing, and a fixture that models the typical case
  supplies no extreme.[^10] The work must say which fixture puts a faction next
  to a tile it has never seen, and it must put the defect back and watch the
  test stay green before it claims the case is covered.

- **What the field of view pass costs at the target scale.** A shadowcast for
  every unit that changed tile is the largest new cost this item adds, and
  every figure for it is derived rather than measured.[^9]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: The world module of the core crate, the subsystem census. `crates/cachette-core/src/world.rs`
[^3]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^4]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^5]: ADR-0059, fog storage grows with observed area, not with world area. `docs/adrs/draft/adr-0059-fog-storage-grows-with-observed-area.md`
[^6]: Research report 08, fog of war representation. `docs/research/reports/08-fog-of-war-representation.md`
[^7]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^8]: Balance register. `docs/reference/balance.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: Testing Rules, sections 1 and 2a. `.agents/rules/testing.md`
