---
id: 0180
title: Let a unit choose to build
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0008]
blocked-by: [0058]
---

## Why

A unit builds only when the control plane tells it to. The choice pass scores
a small fixed option set, and no option in that set builds anything.[^1] A
world left to run therefore builds nothing, whatever the ground offers.

A unit that stands on a finished upgrade goes on adding work to it, and the
clamp absorbs every contribution. Nothing tells the unit to stop, so a builder
that finished its work wastes every later tick. The record names this and
declines to decide it.[^2]

The item also needs the faction rule, because an option that builds must know
whether a unit may build on ground another faction holds.[^3]

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^3]: Blockers register, BLK-034. `docs/BLOCKERS.md`
