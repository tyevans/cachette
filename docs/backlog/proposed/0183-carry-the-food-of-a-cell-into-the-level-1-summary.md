---
id: 0183
title: Carry the food of a cell into the level 1 summary
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0009]
blocked-by: []
---

## Why

A unit chooses by reading the level 1 cell it stands in. The cell summary holds
six fields: the tile count, the open tile count, the unit count, the held tile
count, the value total and the height total. **None of them is a resource, a
store or a need.**

The world does hold food. Every tile carries a generated stock, and the
founding survey reads that stock to choose a place and to set the production
rate of the new site. Nothing summarises it, so no unit can see it, however
well the unit chooses.[^1]

This item adds one accumulator. It changes no behaviour on its own, and item
0184 is what reads it.

## What the work does

1. The cell summary gains a food total, as a 64-bit accumulator. A one-byte
   field summed over the target tile count overflows a 32-bit
   accumulator.[^2]
2. The rebuild takes the resource field as one more argument and adds the
   stock of each tile as it walks the cell.
3. The summary gains an intensive accessor for the mean, beside the accessors
   the other totals already have.[^3]

## What is missing before this is refined

- The impact review.
- Whether the field is the food alone or one total for each resource kind.
  One kind is the smallest change. Three is the shape the rest of the engine
  uses, and it triples the width of the summary.
- Whether the rebuild reads the stock through the same generated-plus-stored
  path a tile reader uses, and what that costs when a cell holds no
  depletion.[^4]
- What the added field does to the state hash and to the golden files.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 3.2. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^3]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
