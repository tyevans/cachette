---
id: 0184
title: Score the forage option against food
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0009]
blocked-by: []
---

## Why

The option set holds a row named `forage`. It scores the mean stub value of
the level 1 cell.

**The stub value is noise.** The tile value pass draws a number for every tile
on every tick and adds one, minus one, or nothing. No other system reads it,
and no other system writes it. A unit that forages therefore walks toward a
random walk.[^1]

Item 0183 puts the food of a cell into the summary. This item points the
option at it. After both, the choice pass scores a quantity that another
system writes, and a watcher can check the choice against the ground: the
explanation reports a food value, and the deposit under the unit holds that
food.[^2]

## What the work does

1. The `forage` row of the option table reads the mean food of the cell
   instead of the mean stub value.
2. The option table stays a table of values. The pass calls no content
   code.[^3]

## What is missing before this is refined

- The impact review.
- Whether the stub value keeps a reader at all after this. The viewer paints
  it, and item 0188 replaces that. A field that nothing reads is the shape
  FND-181 records.[^4]
- Whether the option order changes, because the order is the tie-break order
  and it is part of the behaviour.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: Backlog item 0183, carry the food of a cell into the level 1 summary. `docs/backlog/proposed/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
[^3]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^4]: Findings register, FND-181. `docs/FINDINGS.md`
[^5]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
