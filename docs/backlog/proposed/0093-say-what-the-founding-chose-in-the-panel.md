---
id: 0093
title: Say what the founding chose in the panel
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0012, PRD-0005]
blocked-by: []
---

## Why

A run now begins with a small group in a place the engine chose. The panel
still describes the world it was written for: a full world of soldiers spread
everywhere.

The panel says how many units are alive, how many the window draws, how many
of each faction the window draws, and what ground the window shows. Every one
of those rows is correct, and none of them says the thing a watcher of a
founded run wants to know: where the group is, and why the engine put it
there.

The founding already computes the answer and returns it. The engine holds the
counts that made the place the choice, and nothing shows them.[^1] A watcher
must read the terminal to learn why, and the terminal prints the line once and
then scrolls away.

## What the work does

1. The panel names the founded place and says whether the window shows it.
2. The panel gives the counts that made the place the choice: the food, the
   wood, the stone, the open ground, and the open water beside it.
3. The panel says how many places the founding compared, so a watcher can tell
   a choice from a default.
4. A watcher can compare the chosen place against a place that was not chosen.

## What is missing before this is refined

- **The impact review.** The viewer record governs this, and the panel record
  governs what a row may say.[^2] [^3] Neither has been read against this
  work.
- **Whether the panel may hold a value the drawing pass did not read.** The
  panel record says a row states what the drawing pass counted, or what the
  world already exposes.[^3] The founding report is neither: it is a value the
  world computed once, before the first frame, and holds. That may be a third
  category, and the record must say so before a row states it.
- **Whether a founding report belongs to the world at all.** The founding is
  returned to the caller today. The world does not keep it, so the viewer
  cannot ask the world about it, and the demonstration binary holds the only
  copy. Either the world keeps it or the viewer does, and one of the two is a
  second declaration site for one fact.[^4]

## Done when

- The panel says where the group was founded and why.
- A picture test covers the new rows, in the same way the existing rows are
  covered.
- No row states a value that nothing computed.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0075, the founding choice reads a bounded sample of the world, decision D5, a draft record. `docs/adrs/draft/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^2]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^3]: ADR-0070, the head-up display reports what the drawing pass read. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
