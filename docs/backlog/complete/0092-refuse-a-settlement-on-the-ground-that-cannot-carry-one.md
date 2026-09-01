---
id: 0092
title: Refuse a settlement on the ground that cannot carry one
status: complete
created: 2026-08-31
implements: [ADR-0053 D5, ADR-0056 D4, ADR-0066 D1, ADR-0068 D4, ADR-0075 D1]
changes: []
creates: []
serves: [PRD-0012, PRD-0006]
blocked-by: [0071]
---

## Why

A settlement stands on any tile the world holds, and that includes water.

The settlement arena refuses a tile another settlement holds, and it refuses
an address outside the world. It applies no rule about the ground. The item
that built the arena states this in its outcome, and it states it
correctly.[^1] No record forbade it then, and that item stated no new rule.

The project owner states that a settlement on water is not wanted.

A unit is already refused on water. The world reads the ground and passes the
refusal into the spawn and into the movement admission. A settlement reads
nothing. The world therefore holds two answers to one question, and one of
them is silent.

## The question this item asked, and the answer

The proposed item asked whether the ground that carries a settlement is the
same ground that carries a unit. **It is, and an accepted record already says
so.**

The faction record decides that ground which admits no unit admits no
holder.[^2] A settlement is a holder of ground in the plainest sense: the
group that founds it stands on the tiles around it, and the spread rule reads
where those units stand. A settlement on water therefore sits on a tile that
no faction can ever hold, peopled by a group that the spawn refuses to place.
The refusal is not a new rule. It is the rule the engine already applies,
reaching one call site that never called it.

**This item therefore creates no record, and that is a deliberate judgement
against the scope rule.**[^3] Condition one fails: a contributor cannot
reasonably choose otherwise, because the alternative contradicts an accepted
decision. Condition three fails too, because the reasoning is a call to the
passability reader and a reviewer sees it in the code.

**The second ground property is now decided, and it is not this item.** The
project owner answers that a settlement reads a suitability rule of its own,
separate from passability.[^6] A unit crosses a mountain, and a settlement
cannot occupy one. That answer widens this item. It does not rewrite it. This
item refuses the ground that carries nobody at all, and every such ground stays
refused under the wider rule.

**A follow-up item adds the second property, and this item keeps its scope.**
Two reasons decide it this way. The wider rule gives every ground kind a second
value, which is a second declaration site, and shape 1 of the recurring defect
rule asks for a check that fails when two sites disagree.[^7] That check is
work this item does not hold. The wider rule also states a claim a contributor
could reasonably choose otherwise, so it needs a record of its own, and this
item creates none.[^3]

## What makes this more than one call site

Item 0071 records that passability has two declaration sites. The capacity
table states that a capacity of zero is the whole of passability, while the
passability reader matches on the water kind by name, and the second site is
the one every engine caller uses.[^4] The findings register holds the
instance.[^5]

**A settlement rule added on top of that adds a third caller of the wrong
site.** This item therefore does not state its own ground rule and does not
match on a kind. It calls the passability reader, and item 0071 makes that
reader derive its answer from the capacity. **The order is fixed: 0071 first,
then this item.** That is why 0071 blocks this one, and it is the answer to
the recurring defect the rules name first.[^7]

## Impact review

**Governed by.**

- ADR-0053 D5. Ground that admits no unit admits no holder. This is the
  decision the item reads, and it is why the refusal needs no record of its
  own.[^2]
- ADR-0056 D4. The capacity is a data-driven property of the terrain. The
  refusal reads that property through the passability reader and holds no
  capacity literal of its own.[^8]
- ADR-0068 D4. The engine says what a tile is and never what a tile costs. The
  refusal is a statement about what the tile is, so it belongs to the terrain
  reader and not to the settlement arena.[^9]
- ADR-0066 D1. A settlement is one of the four fixed shapes. The refusal adds
  no shape and no column. It is a condition on an existing founding path.[^10]
- ADR-0075 D1. The founding reads a bounded sample. The refusal must not widen
  that sample, and a founding that finds no acceptable place in its sample
  reports the refusal rather than drawing again.[^11]

**Changes.** No record changes. The terrain module comment that denies the
second site is repaired by item 0071, not here. The follow-up item that adds
the settlement suitability property changes what the founding reads, and it
runs after this item.[^6]

**Creates.** No record. The reasoning is above, and it is a judgement against
the scope rule rather than an omission.[^3]

**Blockers.** BLK-007 governs every cost figure, so this item states
none.[^12] No blocker governs the rule. BLK-018 is resolved: every faction
founds one group. It does not bear on this item, because the refusal holds for
one founding and for many.[^13]

**Precedent.**

- FND-060 records the two passability sites and asks for the second to be
  derived away rather than reconciled.[^5]
- FND-054 records that a test world narrower than the coarsest lattice spacing
  holds one kind of ground. A fixture for this item must hold water, and the
  extent belongs in the fixture.[^14]
- FND-061 records that a fixture assertion must be stated over the outcome and
  not over the inputs.[^15]
- FND-070 records that a restored defect must be the smallest change that
  violates the claim, or the proof cannot run.[^16]

**Serves.** PRD-0012 and PRD-0006.

PRD-0012 asks that the engine choose the founding place by reading the world,
and that a group founded in a poor place does worse than a group founded in a
good one.[^17] A place that carries nobody is not a poor place. It is not a
place, and the record's test cannot be applied to it.

PRD-0006 asks that terrain influence a holding, and its consequences state
that a holding cannot spread over ground that admits no unit.[^18] A
settlement on such ground contradicts the record that serves it.

**PRD-0014 is not served, and the proposed item named it.** No statement in
that record is about the ground under a place, so this item answers none of
them.[^19]

**Conflict surface.** `crates/cachette-core/src/site.rs` at the founding path
and at the error type. `crates/cachette-core/src/world.rs` where the world
founds a settlement and where it founds a run.
`crates/cachette-core/src/terrain.rs` is read and not changed.
`crates/cachette-core/tests/settlement_arena.rs` and
`crates/cachette-core/tests/founding.rs` gain the cases.

**It cannot run beside item 0071**, which changes the reader this item calls.
**It cannot run beside item 0095**, which changes the same founding path where
it counts who already stands on a tile.

## Done when

- A founding on ground that carries no unit is refused, and the refusal is a
  named variant of the settlement error rather than a silent failure.
- The rule reads the passability reader and states no ground rule of its own.
  A whole-tree search finds no second site that says which ground carries a
  settlement, and the search command is in the commit body.
- A test founds a settlement on water through the public interface and asserts
  the refusal by name.
- The fixture asserts that the world it built holds water, and it asserts it
  over the world rather than over the seed that made it.[^14] [^15]
- A test founds a run in a world that holds water and asserts that every
  settlement the founding placed stands on ground the rule accepts.
- The refusal is put back, and the tests are watched failing, before the item
  is claimed done. The restored defect is the smallest change that violates
  the claim.[^16]
- The founding still reads the same number of tiles in a small world and in a
  large one, so the refusal did not widen the sample.
- The golden state hash files are regenerated only where a founding moved, and
  the commit body says which files moved and why.
- `just check` exits 0.

## Outcome

**Done.** The world refuses a settlement on ground that carries no unit. The
settlement error gained one variant for it, and the world returns that variant
by name. The rule calls the reader that answers whether a tile admits a unit,
and that reader derives its answer from the capacity of the ground. The rule
holds no kind name and no capacity literal.

**The refusal sits in the world, not in the arena.** The arena holds the grid
and no terrain, so it cannot read the ground. The world holds the terrain. The
extent refusal therefore keeps its own name, and a test asserts that an address
outside the world still reports the extent and not the ground.

**No record was written.** The reasoning is above and it is unchanged.

**What each test proves.** Two tests call the founding directly. They founded a
settlement on water, asserted the refusal by name, then founded on open ground
in the same world. Both went red when the refusal was removed from the source.

A third test founds a whole run in four worlds and asserts that every
settlement stands on ground that carries a unit. **That test did not go red
when the refusal was removed.** The survey already drops a candidate whose
ground admits no unit, so the refused case never reaches the lower rule. The
test stayed green when the survey filter itself was removed as well, because
the score prefers good ground and chose a passable place anyway. The test is
therefore a guard against a later change, and it is not evidence that the new
rule works. The findings register holds this as precedent.[^20]

**The fixture holds water, and it says so over the world.** The fixture builds
a world three lattice cells wide along each axis, scans it, and keeps the first
tile of zero capacity and the first tile of greater capacity. It asserts that
it found both. It asserts nothing about the seed.

**The sample did not widen.** The refusal reads the tile the caller named and
reads no other tile. The bounded-sample test still holds: four worlds whose
tile counts span a factor of sixty-four read the same order of tiles.

**One fixture moved.** The thread-count fixture founded settlements on the
first twenty-three tiles of each scenario world, and some of those are water.
It now walks the tiles that admit a unit, in the same index order.

**No golden state hash file moved.** The golden founding pattern already
avoided water in every scenario it holds.

**Item 0102 holds the wider settlement suitability rule.** It was already in
the backlog, so this item created no follow-up.

## References

[^1]: Backlog item 0052, the outcome. `docs/backlog/complete/0052-provide-the-settlement-column-set.md`
[^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D5. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Backlog item 0071. `docs/backlog/complete/0071-derive-tile-passability-from-tile-capacity.md`
[^5]: Findings register, FND-060. `docs/FINDINGS.md`
[^6]: Decisions register, DEC-035. `docs/DECISIONS.md`
[^7]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^9]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^10]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^11]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^12]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^13]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^14]: Findings register, FND-054. `docs/FINDINGS.md`
[^15]: Findings register, FND-061. `docs/FINDINGS.md`
[^16]: Findings register, FND-070. `docs/FINDINGS.md`
[^17]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^18]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^19]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^20]: Findings register, FND-093. `docs/FINDINGS.md`
