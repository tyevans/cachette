---
id: 0212
title: Count a window once and let the viewer read it
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Two places in this project count what a window of the world holds. Nothing
fails when the two disagree.

The viewer counts as it draws. It reports the largest number of units it drew
on one tile, the tiles it drew that hold at least as many units as they admit,
and one count for each kind of ground.[^1]

The engine now holds a census of a window. It counts the same three things over
a rectangle of addresses, and the agent protocol server calls it.[^2]

The two answer different questions today, and that is the reason both exist.
The viewer counts what a person saw, and it skips a block the camera did not
reach. The census counts the addresses a caller named. **The rule for which
tile counts as full is the same in both, and it is written twice.** An empty
tile is never full, whatever it admits. Neither copy fails when the other
changes.

This is the shape the project meets most often: one rule stored in two places,
with nothing that fails when the copies drift.[^3]

## What the work might do

Give the viewer the census, and let it name the window it drew. The viewer then
holds no rule of its own for what a full tile is.

The census is a free function over a shared reference to the world. It writes
nothing, so the viewer can call it under the rule that the viewer never writes
to the world.[^4] Nothing in it is specific to the agent server.

This may not be possible without losing something. The viewer has each tile in
hand already. A second walk over the same addresses is work the picture does
not need. Whether the saving is worth a second declaration site is the question
this item has to answer.

**Two workers changed this ground in one round.** One rebuilt the viewer, and
one added the census. Read both as they stand before planning the work, because
neither is what this item was written against.

## What is not yet worked out

- Whether the viewer can name its window as a rectangle of addresses. The
  camera gives a row range, and a column range for each row.
- Whether the cost of a second walk matters at the sizes a person watches.
- Whether the answer is one census, or one shared rule for a full tile and two
  counters.
- Whether the census should report a count for each faction. The viewer holds
  one and the census does not, because nobody asked the server for it.

## Done when

- One place states what a full tile is, and the viewer reads it.
- A change to that rule reaches both counts with no second edit.

## References

[^1]: The viewer. `crates/cachette-view/src/`
[^2]: The census of a window. `crates/cachette-core/src/census.rs`
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: ADR-0067, the viewer reads the world and never writes to it, decision D1. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
