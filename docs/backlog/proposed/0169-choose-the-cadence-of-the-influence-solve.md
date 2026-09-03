---
id: 0169
title: Choose the cadence of the influence solve
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: [BLK-007]
---

## Why

The influence solve runs on every tick, for every faction the world holds. The
research that supports the field says that is the wrong cadence at the target
scale, and it gives an amortised schedule instead: a share of the factions is
solved on each tick, and the rest carry the field the last solve left
them.[^1]

Item 0104 built the field and declined to choose the cadence, because a
cadence is a value chosen against a cost and no run has priced a solve.[^2] The engine already holds two schedules of this shape, one
for the site rates and one for the choice, so the mechanism is not the hard
part.[^3]

The hard part is the value. A cadence that is too slow makes the writ of a
ruler lag the world by more ticks than the consumers accept, and the research
states the staleness each consumer tolerates.[^1]

## What is missing before this is refined

- The impact review.
- Whether the cadence belongs to the world settings or to the field.
- What a solve that is skipped does to the state hash, and whether a world
  that skips a solve still gives one answer at any thread count.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Influence maps, section 7. `docs/research/reports/09-influence-maps.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Backlog item 0104. `docs/backlog/refined/0104-carry-the-writ-of-a-ruler-in-the-influence-field.md`
