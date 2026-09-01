---
id: 0106
title: Show a watcher what is moving and where it goes
status: proposed
created: 2026-08-31
implements: [ADR-0067 D1]
changes: []
creates: []
serves: [PRD-0010, PRD-0005]
blocked-by: [0105]
---

## Why

A flow that nothing displays cannot be judged. PRD-0010 asks that a watcher
see what is moving and where it is going, and that a watcher see what changes
when a route is cut.[^1] A solver whose only witness is a conservation test
proves that it loses nothing. It does not show that it moves the right thing
to the right place.

## What the work does

1. The viewer shows the quantity in motion on each link of the network.
2. The viewer shows the value of a good at each place, if a value exists.
3. A watcher blocks a place and sees what arrives change.

## Impact review

**Governed by.** ADR-0067 D1 requires the viewer to read the world and never
write to it.[^2]

**Changes.** None.

**Creates.** None expected. A display is not a constraint.

**Blockers.** BLK-007 governs every cost figure, so this item states none.[^3]

**Precedent.** FND-049 records the dominant cost term.[^4] A display that
walks every link each frame pays the network for each frame.

**Serves.** PRD-0010 for the statement that a watcher sees what is moving, and
PRD-0005 for the statement that a watcher can tell what is happening and
why.[^1] [^5]

**Conflict surface.** `crates/cachette-view/`. Unknown beyond that until item
0105 fixes the network representation.

## What is missing before this is refined

**Item 0105 must land first.** There is nothing to display until a flow
exists, and the shape of the display follows from what a node and a link turn
out to be.

## Done when

- A watcher sees the quantity on a link, and the display changes when the flow
  changes.
- A watcher blocks a place and sees what arrives change.
- The viewer writes nothing to the world, and a test asserts it.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0010, a good moves to where it is wanted. `docs/product/accepted/prd-0010-a-good-moves-to-where-it-is-wanted.md`
[^2]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-049. `docs/FINDINGS.md`
[^5]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
