---
id: 0480
title: Paint three map layers and register five panels
status: proposed
created: 2026-09-05
implements: [ADR-0067 D1, ADR-0067 D2, ADR-0070 D1, ADR-0070 D2]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: []
---

## Why

**A watcher cannot see a relation, a board, a storm or a score.** The engine
holds them after passes 3 to 8, and the window shows none. This item is pass 9
of the living world game layer.[^1]

The viewer paints three new layers. Upgrades show a glyph or a tint per kind,
with the condition as the tint depth. Weather shows air water as an overlay,
and wet ground darker than dry ground. Luxuries show one mark per tile that
holds a deposit.

Five panels join the deck: weather, relations, market, economy and score. The
viewer holds the deck in one registration, and the Python `panel_names` reader
derives from it, so a new panel appears with no Python edit. A panel reads a
bounded number of addresses and starts no pass over the world.

**This pass does not touch `fn step` in `world.rs`.** It runs beside passes 3
to 8, one panel as each subsystem lands.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0067 D2 states that the engine
  holds no value that exists for the viewer, and the review must say which
  reader each panel reads and that none was added for it. ADR-0070 D1 states
  that a panel adds no pass, and the review must state the address count each
  panel reads.
- Whether item 0316, which gives the panel one standard that a new panel
  registers with, lands first. Five panels written before the standard are
  five files to move.
- Which pass each panel waits for, so the item can be split into one row per
  panel if the passes land far apart.
- The test that drives the real caller: the demonstration must draw each panel
  through the function keys, and a test must assert the panel count from
  `panel_names` against the registration.
- The "Done when" statements, in the shape of item 0472. This pass adds no
  keyed draw, so the per-field statement does not apply, and the item must say
  so rather than omit it.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 9 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
