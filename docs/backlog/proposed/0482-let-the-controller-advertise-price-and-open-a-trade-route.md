---
id: 0482
title: Let the controller advertise, price and open a trade route
status: proposed
created: 2026-09-05
implements: [ADR-0144, ADR-0147]
changes: []
creates: []
serves: [PRD-0050, PRD-0051]
blocked-by: [BLK-007, BLK-036]
---

## Why

**A faction that holds a board never writes to it.** Item 0477 gives each
faction a board and lets a contract carry land or a relation, and it stops at
the engine. Nothing inside the step reads a board or posts to one. This item is
the controller half of pass 6 of the living world game layer.[^1]

The controller writes its rows from its site economies on a schedule. It
offers only where its own surplus meets a posted want on another board.
Surplus is the site store above a mark in the balance register. It counters at
the integer midpoint between the two asks, accepts when the counter meets its
own ask, and never draws for a price. A trade route is a contract plus
carriers: units with carry capacity above zero get a home at one site and are
sent to the other, and the delivery pass does the rest.[^1]

**This item was split from item 0477 on 5 September 2026.** It waits for the
controller stage of item 0472 and the relation of item 0474, because the
controller reads the band before it trades and the midpoint rule reads the
weight vector.

## What is missing before this is refined

- The impact review, decision by decision, against ADR-0144 and ADR-0147.
- The test that proves the controller never draws for a price.
- The advertisement schedule and the surplus mark are rows in the balance
  register, and both are unset.[^2]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, section 4. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
