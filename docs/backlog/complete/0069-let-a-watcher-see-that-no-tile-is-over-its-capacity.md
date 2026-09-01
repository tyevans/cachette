---
id: 0069
title: Let a watcher see that no tile is over its capacity
status: complete
created: 2026-08-31
implements: [ADR-0070 D1, ADR-0070 D2, ADR-0067 D1, ADR-0067 D2, ADR-0067 D3, ADR-0056 D4]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The product record for the first renderable example asks for two things about
the capacity of a tile. No tile holds more units than its capacity allows, and
a watcher can see that this holds. The review of that record found the
statement unmet and left the record in `shaped/`.[^1]

The panel states the mean number of units on an open tile of the region under
the crosshair. A mean of one unit a tile is consistent with one tile holding
four hundred. The panel states no maximum, no count of tiles at their
capacity, and the picture gives an over-filled tile no mark of its own.

The reviewer put four hundred units on one tile of ordinary ground and drew
the frame. The engine accepted the spawn, the viewer painted every unit on
that tile, and neither said anything.

This item is the half of the statement that nothing has recorded: **the
watcher must be able to see the answer, whichever way the other half goes.**

## The other half, and why this item does not decide it

Whether a spawn may over-fill a tile is an open choice, and the register holds
it with three options and a recommendation.[^2] The engine follows the
recommendation today: admission never raises a tile above its capacity, and a
spawn does not read the capacity at all.

This item states no position on that choice. It shows what is true. Under the
recommendation, a tile above its capacity is a caller mistake, and the picture
must show the mistake instead of hiding it. Under either other option, the
same numbers show that the refusal works.

## What the work does

1. The pass that paints the units counts, for each tile it paints, how many
   units it painted there.
2. The pass keeps two numbers from those counts: the largest count on any
   painted tile, and the number of painted tiles at or above the capacity of
   their ground.
3. The pass marks a tile it painted above its capacity, so that a watcher sees
   the tile itself and not only a number.
4. The panel states the two numbers, and it labels each as a count of what the
   frame painted.

**The count needs no new pass and no new sort.** The derived structure hands
the drawing pass the units of one block in tile order, so the units of one
tile are already one adjacent run. Counting a run costs one comparison for
each unit on a path that already visits every unit it paints.

## Impact review

**Governed by.**

- ADR-0070 D1. The panel adds no pass over the world. Every number comes from
  the engine at once, from the viewer's own state, or from a count the drawing
  pass produced while it painted. These two numbers are of the third kind, and
  a loop over the population inside the reporting code is the violation.[^3]
- ADR-0070 D2. A number the panel cannot afford is absent, never estimated.
  The maximum over the whole world is not affordable, so the panel does not
  show it, does not sample for it and does not extrapolate from the window.
  The label says that the number counts the painted window.[^3]
- ADR-0067 D1. The viewer reads the world through the public interface and
  writes nothing to it.[^4]
- ADR-0067 D2. The engine holds no value that exists for the viewer. **This
  rejects the obvious alternative**, which is a maximum field on the summary
  pyramid.[^4]
- ADR-0067 D3. Floating point begins at the viewer boundary. Both numbers are
  integer counts, so neither needs the boundary at all.[^4]
- ADR-0056 D4. The capacity is a data-driven property of the terrain, and no
  reader holds a capacity of its own. The viewer reads the capacity of a tile
  through the terrain reader and holds no capacity value.[^5]

**A summary field is rejected, and the reason is algebraic as well as
architectural.** A maximum is associative and commutative and has an identity,
but it has no inverse. An accepted record states that only a field with an
inverse may be repaired incrementally, and that a field without one is rebuilt
from the level below.[^6] A maximum on the pyramid would therefore force the
rebuild path on every cell that a unit leaves. The record that forbids a value
held for the viewer already settles the question; the algebra says the value
would also be the expensive kind.[^4]

**Changes.** No record changes.

**Creates.** No record. The judgement follows the scope rule.[^7] Condition one
fails: the alternatives are a pass over the population and a summary field, and
each contradicts an accepted decision. Condition three fails: the count sits on
the painting path, and a reviewer sees there that it costs one addition.

A world-wide maximum is a different claim. It needs a summary field, it needs
the rebuild path, and it needs a record. This item does not open that question
and does not answer it.

**Blockers.** BLK-007 governs every cost figure, so this item states none.[^8]
BLK-009 is resolved: it fixes the capacity of ordinary ground and states that
crossing terrain carries a higher one.[^9] **This item invents neither value
and quotes neither.** It reads the capacity of each tile from the terrain
reader, so a change to either value reaches the picture with no edit.
DEC-020 holds the open choice about the spawn, and this item is written to
work under any of its options.[^2] DEC-022 holds the choice behind the other
failing statement of the product record, and it does not bear on this
item.[^10]

**Precedent.**

- FND-060 records that the ground has two declaration sites for what admits a
  unit.[^11] The viewer must read the capacity, never a kind name, or it adds
  a third. Item 0071 removes the second site.[^12]
- FND-051 records that a fixture built from the demonstration world supplies
  no extreme, so the assertion never receives the case.[^13] The fixture here
  is built to hold an over-filled tile, a tile at exactly its capacity, and an
  empty tile.
- FND-048 records that the two determinism tests cannot see a broken
  invariant, and it names this invariant as the example: a tile above its
  capacity is a pure function of the intent set, so both tests pass over
  it.[^18] The panel is the watcher's version of the same gap.
- FND-061 records that a fixture proves its case over the outcome and not over
  its own inputs.[^14] The fixture asserts what the frame reported, not what
  the spawn asked for.
- FND-070 records that a restored defect must be affordable.[^15] The defect
  here is the removal of the counter, which costs one test run.

**Serves.** PRD-0002. The record states nine checkable statements, and the
review found two of them unmet.[^1] **This item closes the second half of one
of the two: the half that says a viewer can see that the capacity holds.**

**Closing it does not move the product record, and the item claims no such
thing.** Three things stand in the way. The other failing statement is about
the viewer making the engine wait, and item 0070 owns it. The first half of the
capacity statement is the open choice that DEC-020 holds. The review itself is
a separate item, which checks each statement against the code by running
it.[^16]

**Conflict surface.** `crates/cachette-view/src/paint.rs` at the unit pass, at
the canvas counters and at the mark. `crates/cachette-view/src/hud.rs` at the
panel rows. The viewer tests gain the cases. `crates/cachette-core` is read and
not changed. **It cannot run beside item 0072**, which changes the same drawing
pass and the same panel rows.

## Done when

- The panel states the largest number of units the frame painted on one tile,
  and the number of painted tiles at or above the capacity of their ground.
- Each of the two rows says that it counts what the frame painted. A reader
  who wants a count of the world learns that the panel has none.
- The picture marks a painted tile that holds more units than its capacity
  allows, so a watcher sees it without reading the panel.
- The two numbers come from the pass that paints. The viewer starts no loop
  over the units and no loop over the tiles for them.
- The viewer holds no capacity value. A whole-tree search of the viewer crate
  finds none, and the search command sits in the commit body.[^17]
- A fixture builds a world with a tile above its capacity, a tile at exactly
  its capacity, and an empty tile. A test asserts that the frame reports the
  over-filled tile and does not mark the tile at exactly its capacity.
- The fixture asserts the case over what the frame reported, never over the
  spawn that built it.[^14]
- A test moves the camera off the over-filled tile and asserts that both
  numbers fall, which proves that the label is true.
- A test compares the count for one tile fully in view against the count the
  engine gives for that tile through the public interface, and the two agree.
- The counter is removed, and the tests are watched failing, before the item is
  claimed done. The restored defect is the smallest change that violates the
  claim.[^15]
- A test asserts that the state hash of one tick is the same with the viewer
  attached and with no viewer attached.
- `just check` exits 0.

## Outcome

**The watcher can now see the answer.** The drawing pass counts the units it
paints on each tile, keeps the largest count and the number of painted tiles
at or above the capacity of their ground, and outlines a painted tile that
holds more units than its ground admits. The panel states the two numbers
under one heading, and two notes say that the numbers count the drawn tiles
and that the panel holds no count of the world.

**The count rides on the pass that paints.** The derived structure hands the
pass the units of one block in tile order, so the units of one tile arrive as
one adjacent run. The pass closes a run when the address changes and when the
block ends. No loop over the units and no loop over the tiles was added.

**The viewer holds no capacity value.** It reads the capacity of each tile
through the terrain reader, so a change to either capacity reaches the picture
with no edit here.

**No engine file changed and no golden state hash moved.** The viewer reads
the world through a shared reference, and a test asserts that the state hash
of the crowded fixture is the same after a draw.

**This does not move PRD-0002, and the item claims no such thing.** The other
failing statement of that record belongs to item 0070 and to DEC-022. The
first half of the capacity statement is the open choice that DEC-020 holds.
The review that would move the record is a separate item.[^16]

**The tests were watched failing.** The fixture builds a tile under its
capacity, a tile at exactly its capacity, a tile over it, and an empty tile.
Four defects were put back one at a time, and each was caught. The commit body
holds the list and the counts.

## References

[^1]: Backlog item 0028, the review of the first renderable example. `docs/backlog/complete/0028-close-out-the-first-renderable-example.md`
[^2]: Decisions register, DEC-020. `docs/DECISIONS.md`
[^3]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^5]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^6]: ADR-0023, an aggregate combines exactly, in any order, decision D4. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^7]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: Blockers register, BLK-009. `docs/BLOCKERS.md`
[^10]: Decisions register, DEC-022. `docs/DECISIONS.md`
[^11]: Findings register, FND-060. `docs/FINDINGS.md`
[^12]: Backlog item 0071. `docs/backlog/refined/0071-derive-tile-passability-from-tile-capacity.md`
[^13]: Findings register, FND-051. `docs/FINDINGS.md`
[^14]: Findings register, FND-061. `docs/FINDINGS.md`
[^15]: Findings register, FND-070. `docs/FINDINGS.md`
[^16]: Backlog item 0073. `docs/backlog/complete/0073-review-the-first-renderable-example-again.md`
[^17]: Commit Message Rules, after a sweep. `.claude/rules/commits.md`
[^18]: Findings register, FND-048. `docs/FINDINGS.md`
