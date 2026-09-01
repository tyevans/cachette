---
id: 0084
title: Give a tile one faction column
status: refined
created: 2026-08-31
implements: [ADR-0053 D2, ADR-0006 D1, ADR-0011 D3, ADR-0001 D4, ADR-0022 D1]
changes: []
creates: []
serves: [PRD-0006]
blocked-by: []
---

## Why

A tile carries two values that name a faction. Only one of them is a holder.

The holder says who holds the ground. It changes while the world runs, and it
names nobody where nobody holds.[^1] The second value is the tile faction
column of the tile stub. The world writes it once, from the tile index and the
faction count. No rule ever writes it again. It covers open water as readily
as open ground.

The stub column is private, so it looks contained. It is not. The tile stub
event carries it. The event documents the field as the faction that owns the
tile, and the event log crosses to the control plane.[^2] A developer who
compares the faction of a unit against the owner the event reports gets a
confident wrong answer, and nothing fails.

One fact in two places is the shape this project keeps finding.[^3] Here the
two places do not hold the same kind of fact, so no reader can reconcile them.
The product record asks for one value. It asks that every interface which
reports the owner of a tile reports that value.[^4]

## What each caller reads today

This list comes from the code, not from the record.

- The tile stub emitter reads the stub column. It copies the value into the
  event that it emits for each tile it changes.
- The state hash reads the stub column. It also hashes the holder column.
- The world invariant check reads the stub column. It refuses a value at or
  above the faction count.
- The viewer holder layer reads the holder, through the tile holder reader of
  the world. It never reads the stub column.[^5]
- Level 1 reads the holder column when it rebuilds.
- The control plane reads neither column by name. It receives the event log as
  raw bytes, and those bytes hold the stub value. **No Python code decodes the
  log today, so no Python caller reads the wrong value yet.** The doc comment
  is the published meaning of the field, and it is already wrong.

Nothing outside the stub emitter, the state hash and the invariant check reads
the stub column.

## What the work does

1. The world loses the tile faction column. The field goes. The construction
   loop that fills it goes. No code derives a faction from a tile index again.
2. The tile stub event carries the holder of the tile. The field changes type
   from the faction identifier to the holder type. The doc comment says that
   the value names a faction or nobody.
3. **An unheld tile carries the holder value that names nobody.** The holder
   type already defines that value. Its raw number sits above the faction
   ceiling, so no faction collides with it.[^6] The work invents no encoding
   and adds no flag byte.
4. The event reports the tile as the frame left it. The stub changes the tile
   value at the top of the step, and the holding spreads later in the same
   step. The work therefore reads the holder after the holding spread, when
   the log is sealed. It does not read the holder inside the parallel value
   pass. A holder read before the spread reports the frame before, and a stale
   read is a confident wrong answer of the kind this item removes.[^7]
5. The state hash loses the stub column. It keeps the holder column, which it
   already covers, so the hash still reports each change to who holds what.
6. The invariant check loses the stub clause. The holding check already
   re-derives the census, the held list and the block masks from the holder
   column.
7. The golden state hash files are recorded again.

The event layout does not move. The holder type and the faction identifier are
both transparent two-byte newtypes. The event keeps its size, its alignment
and its declared padding.[^8]

## Impact review

**Governed by.**

- ADR-0053 D2. A subject carries one holder field, whose value names a faction
  or nobody, and exclusivity is a property of the storage.[^1] The stub column
  is a second faction field on one subject. This work makes the code obey the
  record.
- ADR-0006 D1. An event is plain data with a declared layout, no boolean and
  declared padding.[^8] The field changes type and keeps its width, so the
  padding count does not change and the type stays plain data.
- ADR-0011 D3. The layout of a value that crosses a boundary is declared.[^9]
  The holder crosses to the control plane, so the event declares it.
- ADR-0001 D4. The whole-world hash covers simulated state, and the golden
  file compares against it.[^10] The work drops a value that no rule writes,
  and keeps the value that rules write.
- ADR-0022 D1. Level 0 is the only truth.[^11] The holder column is level 0.
  The event reads that column and never a summary level.

**Changes.** No record changes. No accepted record states that the tile stub
holds a faction column. The source comment that describes the second column is
code, and it goes with the column.

**Creates.** **No record. This is a judgement against the three conditions of
the scope rule, and here is the reasoning.**[^12]

The candidate claim is that the tile event carries the holder, and that nobody
has a stated value. Condition one holds. A contributor could choose a sentinel
faction, a separate flag byte, or a second event type. Condition two fails.
The claim follows from an accepted record rather than from a new choice,
because ADR-0053 D2 already says a holder names a faction or nobody, and
ADR-0011 D3 already says a value that crosses the boundary declares its
layout.[^1] [^9] Condition three fails as well. The holder type states the
value for nobody in its own declaration, and a reader of the event sees the
type.

The counter-test asks whether the decision governs determinism.[^12] It does
not. The hash rule does not change, the hash already covers the holder column,
and the work drops a column that no rule writes.

**Blockers.**

- BLK-007 governs every cost figure, so this item states none.[^13] The cost
  statement here is a shape. The stamping pass runs once for each event, not
  once for each tile.
- BLK-013 is resolved. The ceiling is 63, and one value is reserved for no
  faction, so the value that names nobody cannot collide with a faction.[^6]
- No other blocker governs this work.

**Precedent.**

- FND-079 records the defect and the direction. The stub goes, and the event
  carries the holder with a stated value for nobody.[^2]
- FND-029 records that a stale read produces a confident wrong answer. This is
  why the event reads the holder after the holding spread.[^7]
- FND-051 records that a fixture chosen for realism hides the defect it should
  show.[^14] The default world is not enough here. The fixture must hold an
  unheld tile, a held tile, and a tile that changed holder on the tick under
  test.
- FND-061 records that a fixture must assert over the outcome, not over its
  own inputs.[^15] The fixture proves it holds those three cases by reading
  the holders back after the step.

**Product.** PRD-0006 asks that exactly one value names the faction that owns
a tile, and that every interface which reports an owner reports that
value.[^4]

## Conflict surface

The work touches the files below. Other work holds some of them today, so the
worker who takes this item checks before it starts.

| File | What changes |
|---|---|
| `crates/cachette-core/src/world.rs` | The column field, the construction loop, the step, the stamping pass, the state hash, the invariant check, and the unit tests of that check |
| `crates/cachette-core/src/event.rs` | The field type of the tile event, and its doc comment |
| `crates/cachette-core/src/holding.rs` | Read only, if the holder slice reader is enough |
| `crates/cachette-core/tests/event_layout.rs` | The constructor calls that pass a faction |
| `crates/cachette-core/tests/golden/` | The recorded hashes |
| `crates/cachette-view/tests/shows_who_holds_the_ground.rs` | The comment that names the stub column |
| `docs/reviews/0019-the-soldier-arena.md` | The sentence that names the stub column |

The Python package needs no change. It gives the log to the caller as bytes,
and it decodes no field.

## Done when

- The world holds no tile faction column. A whole-tree search for the field
  name returns nothing, and the commit body holds the command.
- No code derives a faction from a tile index.
- The tile event field holds the holder type. Its doc comment says the value
  names a faction or nobody.
- The event layout test still asserts the same size, the same alignment and
  zero padding bytes.
- A test steps a world. It asserts that each event reports the holder that the
  tile holder reader of the world reports for that tile after the step.
- A test asserts that an event for an unheld tile carries the value that names
  nobody. It asserts that this value is no faction the world holds.
- A test asserts that a tile whose holder changed on the tick under test
  reports the new holder, and not the holder of the frame before.
- The fixture holds an unheld tile, a held tile, and a tile that changed holder
  on the tick under test. The test proves this by reading the holders back
  after the step, and not by asserting over the settings.[^15]
- The worker puts the stamp back to a read before the holding spread, and
  watches the change test fail. The worker puts the stamp back to a faction
  from the tile index, and watches the agreement test fail. The commit body
  reports both experiments.[^14]
- The state hash no longer reads the stub column, and it still reads the holder
  column.
- The golden files are recorded again in the same commit. The commit body says
  that the hash moved because a column that no rule wrote left the hash.
- The thread-count equivalence test passes at 1, 2 and 12 threads, and the
  event logs match byte for byte.
- The invariant check no longer names the stub column. The unit tests of the
  removed clause go with it.
- Each document that names the tile faction column is repaired, and the search
  command is in the commit body.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^2]: Findings register, FND-079. `docs/FINDINGS.md`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^5]: Backlog item 0085, show a watcher who holds the ground. `docs/backlog/complete/0085-show-a-watcher-who-holds-the-ground.md`
[^6]: Blockers register, BLK-013. `docs/BLOCKERS.md`
[^7]: Findings register, FND-029. `docs/FINDINGS.md`
[^8]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^9]: ADR-0011, every value type is a newtype, decision D3. `docs/adrs/accepted/adr-0011-every-value-type-is-a-newtype.md`
[^10]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^11]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^12]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^14]: Findings register, FND-051. `docs/FINDINGS.md`
[^15]: Findings register, FND-061. `docs/FINDINGS.md`
