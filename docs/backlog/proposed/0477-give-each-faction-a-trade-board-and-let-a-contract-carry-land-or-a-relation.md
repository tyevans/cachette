---
id: 0477
title: Give each faction a trade board and let a contract carry land or a relation
status: proposed
created: 2026-09-05
implements: [ADR-0147, ADR-0128 D1, ADR-0128 D2, ADR-0128 D4, ADR-0040 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0050, PRD-0051]
blocked-by: [BLK-007, BLK-036]
---

## Why

**A faction cannot say what it wants, and a contract moves only a resource.**
No god can trade land, and no treaty exists. This item is pass 6 of the living
world game layer.[^1]

Each faction holds one small fixed-size table of advertisements. A row holds
(good, quantity, offers-or-wants, asking good, asking quantity). The row count
is a row in the balance register.[^2] The controller writes its rows from its
site economies on a schedule. Python writes rows through `advertise(faction,
row)`, and `market(faction)` returns the board of any faction.

Each side of a contract becomes a tagged consideration of one of three kinds:
a resource, a bounded set of tiles that the offerer holds, or a step on the
relation pair. A land set is one level 1 cell or a bounded list of tiles. The
holder changes on full delivery of the other side, and no carrier moves land.
A treaty is a trade whose consideration is a relation move. The status machine
is unchanged.

A trade route is a contract plus carriers. Units with carry capacity above
zero get a home at one site and are sent to the other, and the delivery pass
does the rest. No new movement machinery exists.

**This pass does not touch `fn step` in `world.rs`.** It may run beside passes
7 and 8 once pass 5 merges.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0147 is being written beside
  this item, and the registry holds its status.[^3] ADR-0128 D1 states that
  there is no transfer between two stores, and a land delivery moves no store.
  The review must say why a holder change is inside that record.
- BLK-036 governs upgrades on traded ground, and it is open.[^4] Until it
  closes, the engine refuses a land offer whose tiles carry an upgrade. The
  review must state that refusal, and the commit that later removes it must
  search the tree for the blocker number.
- The controller pricing rule reads a surplus mark, counters at the integer
  midpoint, and never draws. The review must say which test proves that the
  controller never draws for a price.
- The extreme that the fixture reaches: a board that is full, a land set of
  one tile the offerer does not hold, and a relation step at the bound.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, the defect put back and the test red, and the
  type stub edited by hand in the same commit as `advertise` and `market`.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 2.3, 4 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^5]: Findings register, FND-320. `docs/FINDINGS.md`
