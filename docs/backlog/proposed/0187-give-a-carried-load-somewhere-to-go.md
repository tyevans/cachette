---
id: 0187
title: Give a carried load somewhere to go
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0007, PRD-0013]
blocked-by: []
---

## Why

**The resource loop has no sink.** A unit gathers into a carry column. No verb
anywhere moves a carry load into the store of a site. A load leaves the world
when the unit dies, and the world counts it as departed so that the ledger
balances.[^1]

Gathering therefore cannot feed anybody. The store of a site rises only by the
fixed rate that the founding set from the survey, so the economy is a constant
and the ground the units stand on does not change it.[^2]

## What the work does

1. A unit that stands on the tile of its home site gives its carry load to the
   store of that site.
2. The ledger balances across the transfer. What leaves the carry columns
   equals what reaches the stores.

After this, what a settlement holds depends on what its people fetched. The
chain from the ground to the store to the ration to the death of a unit is
then closed at both ends.

## What is missing before this is refined

- The impact review.
- Where the transfer sits in the step. It changes no structure, so it needs no
  barrier of its own, and it must run after the movement that put the unit on
  the tile.
- How a resource kind becomes a commodity in a store. The two are separate
  identifiers today.
- Whether the founding keeps its fixed production rate after this, or whether
  the rate becomes a second declaration of the same quantity.[^3]
- What the transfer does to the carry ledger invariant, which the whole-world
  check asserts.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 3.5. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
