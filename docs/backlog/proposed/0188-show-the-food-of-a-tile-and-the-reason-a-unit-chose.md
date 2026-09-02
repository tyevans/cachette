---
id: 0188
title: Show the food of a tile and the reason a unit chose
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0005, PRD-0009]
blocked-by: []
---

## Why

**The viewer paints noise.** The colour of a tile comes from the stub value
field, which is a random walk that no other system reads or writes. The
resources, which the ground generates and the founding survey reads, are drawn
by nothing.[^1]

**The engine can explain a choice and nobody can ask.** A verb reports every
score, the value each option read, and the winner. No file outside the core
crate calls it. The product record asks that a watcher can ask why a unit did
what it did and get an answer from the engine. The answer exists and the
question cannot be put.[^2]

## What the work does

1. The tile colour reads the food stock of the tile.
2. The panel gains a row that reports the choice explanation for one unit.
3. The viewer keeps reading the world and never writes to it.[^3]

A watcher then sees a deposit drain as a crowd works it, sees it recover, and
can ask why a unit walked where it walked.

## What is missing before this is refined

- The impact review.
- Which unit the explanation names, when the viewer has no cursor over a unit
  today.
- Whether the stub value keeps any reader at all after this, and whether the
  pass that computes it should stay. It is a full pass over every tile on
  every tick.
- Where the row goes, given that the panel is already longer than the window
  and cuts.[^4]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: What a unit does in a tick, section 3.8. `docs/research/what-a-unit-does-in-a-tick.md`
[^3]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^4]: Backlog item 0133, the panel is longer than the window. `docs/backlog/PRIORITY.md`
