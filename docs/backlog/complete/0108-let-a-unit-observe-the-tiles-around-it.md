---
id: 0108
title: Let a unit observe the tiles around it
status: complete
created: 2026-08-31
implements: [ADR-0059 D1, ADR-0059 D2, ADR-0059 D3, ADR-0059 D5, ADR-0164 D1, ADR-0164 D2, ADR-0164 D3, ADR-0164 D4]
changes: []
creates: []
serves: [PRD-0001, PRD-0056]
blocked-by: [BLK-050]
---

## Why

A faction sees a tile when one of its own units observes that tile.[^1] A
unit therefore needs a rule that says which tiles it observes.

The engine had no such rule. A unit held a position and a faction, and nothing
more. Every reader answered the truth of the world, and no column, no plane and
no table held what a faction had observed.

This item builds the observation pass and the two layers it writes. Each unit
marks the tiles it observes, the marks combine into what the faction sees this
tick, and the result is identical at every thread count.

## Impact review

**Governed by.** An accepted record decides the storage.[^2] Its decision D1
gives sight to a live unit and to nothing else, and it makes a sight radius a
whole number of hex steps. Its decision D2 makes each layer an array of blocks
over the lattice the summary level aggregates over, with four forms. Its
decision D3 splits the two layers: what a faction sees now is derived, and what
a faction saw once is remembered. Its decision D5 puts the remembered layer in
the state hash, forbids a float in either layer, and asks that every parallel
write be disjoint.

A second record governs the hash.[^3] Every stored value the step reads enters
it, a derived projection stays out, and each value gets a test that names it.

**Not governed by this item.** Decision D4 of the storage record binds every
reader that names a faction, and decision D6 binds the boundary. Those two are
the whole of the observation plane, and a separate item builds them.[^4] This
item builds what that item will read.

**Changes.** No record changes. The implementation honours the accepted records
as written.

**Creates.** No record. The storage decision is already accepted, and this item
adds no constraint that record does not hold.

**Blockers.** The sight radius, the rounding step and the ground that blocks
sight are balance values, and no measurement sets one.[^5] Each is a parameter
of the world with a provisional default, and a caller sets it through one verb.

**Precedent.** A fixture that models the typical case supplies no extreme, and
the testing rule says to put the defect back and watch the test stay
green.[^6] This item did that for every claim it makes, and one of the fixtures
found a real defect.[^7]

## Done when

- The step runs a pass that computes what each faction sees, from the live
  units of that faction and from nothing else.
- The world stores what each faction has ever seen, and the step never removes
  a tile from it.
- Neither layer is a dense bitmap over the world, and a faction that has
  observed nothing allocates nothing.
- The remembered layer and the sight rules enter the state hash, and a test
  changes each through the public interface and asserts that the hash differs.
- The pass gives one answer at one thread, at two threads and at twelve.
- No value in either layer is a floating point number, and every count over a
  layer is widened.
- A test drives the world step for every claim above.

## Outcome

**What the engine stores, and what it derives.** The world holds two layers for
each faction. The layer of what a faction sees now is derived, and the pass
rebuilds it from the live units on every tick. The layer of what a faction has
ever seen is state, and the pass folds the visible layer into it and never
removes a tile. Two masks over the level 1 cells are derived beside them, and
they name the factions that see each cell now and the factions that have ever
seen it.

**What it costs for each faction.** Every figure below is derived, not
measured.[^5] A layer holds one entry for each block of the lattice, and the
target extent divides into 16,384 blocks of 1,024 tiles. An entry is a form tag
with a payload pointer and a population, which is 36 bytes, so the two layers of
one faction cost 1.18 MB of entries once that faction observes anything. A
payload costs four bytes for each tile in the sparse form and 128 bytes for a
whole block in the dense form, and the pass takes the cheaper of the two at a
threshold the code derives from the block edge rather than states. A faction
that observes 20,000 tiles in scattered blocks therefore pays about 80 KB of
payload, and one that observes them in 40 clustered blocks pays about 5 KB. The
dense alternative that the record refuses costs 4.2 MB for each faction whether
the faction observes the whole world or nothing.

**What fixes the parallel order.** The pass builds one stamp for each block
that each observer of each faction may reach, and it sorts the stamps by block,
then by faction, then by observer tile. It then gives each worker a contiguous
run of whole blocks. Two workers never write one block, so the update needs no
atomic operation. Each worker writes its own slot, and the join reads the slots
in slot order. Nothing reads which worker finished first, and the partition
comes from the block count and the thread count. The pass draws no random
number.

**What was left undone.** The per-unit mask that names the factions which see a
unit is not built, and the observation plane item needs it. The rebuild visits
every block an observer reaches rather than only the blocks a unit entered or
left, so the cost follows the observers and not the movement. The sight of each
unit type is one value for every type, because the unit type table was held by
another worker. The balance register holds no row for the sight radius, the
rounding step or the ground that blocks sight, for the same reason.

## References

[^1]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: ADR-0059, fog storage grows with observed area, not with world area. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^3]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^4]: Backlog item 0495. `docs/backlog/proposed/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^5]: Blockers register, BLK-050 and BLK-007. `docs/BLOCKERS.md`
[^6]: Testing rules, sections 2 and 2a. `.agents/rules/testing.md`
[^7]: Findings register, FND-560. `docs/FINDINGS.md`
