---
id: 0052
title: Provide the settlement column set
status: complete
created: 2026-08-31
implements: [ADR-0066 D1, ADR-0066 D2, ADR-0066 D3, ADR-0012 D3, ADR-0014 D1, ADR-0014 D3, ADR-0004 D1]
changes: []
creates: []
serves: [PRD-0014, PRD-0017]
blocked-by: []
---

## Why

The accepted storage record names four entity shapes: the soldier, the
settlement, the living character, and the tile upgrade.[^1] One of the four
exists. The core holds a soldier arena and nothing else, so the world has no
place that is fixed to a tile and holds a pooled store.

Nine of the seventeen product records want that place. Production attaches to
it, consumption draws from it, housing counts residents in it, work is
assigned inside it, and a founding creates one. None of them can start while
it is missing.

This item builds the shape and nothing that uses it. It is the pinch point of
the whole simulation plan, and it is the one item that every other item in the
plan waits on.

## Impact review

**Governed by.**

- ADR-0066 D1. A settlement is one of the four shapes. It is fixed to a tile
  and holds pooled stores. This item builds that shape, with no field that a
  later item does not need.
- ADR-0066 D2. A structural change is a move between column sets, so a
  settlement founding and a settlement loss go through the batched path, not
  through an immediate edit.
- ADR-0066 D3. The shape does not vary at run time. A column set that a
  caller assembles from a component list is a compile-time error.
- ADR-0012 D3. An entity of any shape lives in the generational arena, so the
  settlement arena is a second arena and not a table of its own design.
- ADR-0014 D1 and D3. A settlement identity is a slot and a generation, and
  the generation advances on free. A settlement that is destroyed must never
  hand its identity to the settlement founded next in the same slot.
- ADR-0004 D1. Iteration over settlements is slot order, stated once.
- ADR-0002 D1. No field is a floating point number. A stored quantity is an
  integer or a Q16.16 fixed-point value.

**Changes.** No record changes.

**Creates.** No record. Every claim this work needs is written. The field set
of the shape is content, not a constraint, and the accepted record already
names the shape and forbids a run-time archetype.

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
settlement count comes from the scale constants table and is read, not
invented.[^2]

**Precedent.** FND-040 records that one fact in two places rots when nothing
fails on disagreement. A settlement holds a tile index and the tile will later
carry a back-reference, so the invariant check must fail when the two
disagree, in the same way the unit-to-tile bridge check already does.[^3]

FND-043 records that a value type which cannot hold zero can lose a real
value. A store of zero is a real state and must be representable.[^4]

**Serves.** PRD-0014 and PRD-0017 both state that a place holds people. This
item builds the place. It gives it neither residents nor positions.

**Conflict surface.** `crates/cachette-core/src/site.rs` is new.
`crates/cachette-core/src/lib.rs`, and `crates/cachette-core/src/world.rs` at
the constructor, the state hash and the invariant check. **No other item in
this plan may merge before this one**, because every item that adds a site
field rebases on this file.

## Done when

- A settlement arena exists, with the same identity rule as the soldier arena.
- A settlement holds a tile index, a faction, and a store of one commodity
  quantity. The commodity set is one, and adding a second changes no code
  outside the store.
- A settlement founding and a settlement loss both go through the batched
  structural path, and a test proves that a lost settlement's identity never
  resolves to the settlement founded after it.
- The tile a settlement stands on is a tile the world holds, and a settlement
  outside the world is a typed error rather than a panic.
- Two settlements cannot stand on one tile, or the record of this item says
  why they may.
- The state hash covers the arena, and the golden files are re-recorded with
  the difference read before it is recorded.
- The invariant check fails when the arena and the tile back-reference
  disagree, and a test proves that it fails.
- A property test asserts that the arena is identical at 1, 2 and 12 threads.
- Every new test is checked against a mutation, and the mutations are named in
  the commit body.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

The settlement column set exists. A settlement holds a generational identity,
the tile it stands on, a faction, and a pooled store of one commodity
quantity. The world builds the arena, hashes it, and checks it.

**The identity rule holds.** The generation advances when the arena frees the
slot, so a destroyed settlement loses its identity at the moment it is lost. A
test founds a settlement, destroys it, founds another into the same slot, and
asserts that the old identity resolves to nothing and reads nothing of the new
one. The test asserts first that the fixture reused the slot, because a fixture
that opened a second slot would prove nothing.

**One tile holds one settlement.** A settlement pools the stores of its tile,
so two pools on one tile would give every later question about the tile two
answers. A second founding on a held tile is a typed refusal.

**The tile back-reference exists now, and the check compares it.** The arena
keeps a column of holders, indexed by tile, beside the tile column of the
slots. The founding reads the holders to refuse a second settlement in one
subscript. The invariant check fails when a holder names the wrong tile, when
it names a settlement that is gone, and when a live settlement is missing from
the column.

**The state hash covers the arena.** Every golden file moved, because an empty
settlement arena writes its three counters into the hash. The difference was
read before it was recorded: every scenario changed on every frame, which is
what appending constant bytes to the hash input does, and the shape of each
sequence is unchanged. A new golden scenario founds settlements, writes part of
their stores, destroys part of them and founds again into freed slots.

**Zero is a real state.** A new settlement holds a store of zero, and the store
type represents that. A test writes a quantity, writes it back to zero, and
reads zero rather than an absent store.

### What changed from the plan

**The batched structural path does not exist, so the founding and the loss do
not use it.** The record that would define that path holds a reserved registry
row and no file, and the soldier arena, which the same storage record governs,
edits its columns inside the call. The settlement arena follows the soldier
arena. The finding register records the gap, and the backlog holds the item
that closes it.[^5] [^6]

**No decision record was written.** The impact review said none was needed, and
that held. The one choice this work made that a contributor could reasonably
make differently is the dense column of holders against a scan of the slots.
That choice is private to the module and cheap to change, so it fails the
second condition of the scope test and states no constraint.[^7]

### Deliberately out of scope

The settlement has no production, no consumption, no residents, no positions,
no capacity and no terrain rule. A settlement may stand on any tile the world
holds, including water, because no record states otherwise and this item states
no new rule. Each of those is a separate item.

### Registers

The findings register gained one row. The blockers register did not move: the
one open blocker governs cost figures, and this work states none.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^3]: Findings register, FND-040. `docs/FINDINGS.md`
[^4]: Findings register, FND-043. `docs/FINDINGS.md`
[^5]: Findings register, FND-063. `docs/FINDINGS.md`
[^6]: Backlog item 0077. `docs/backlog/proposed/0077-move-the-structural-change-onto-the-batched-path.md`
[^7]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
