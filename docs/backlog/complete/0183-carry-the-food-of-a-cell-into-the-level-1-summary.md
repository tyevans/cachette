---
id: 0183
title: Carry the food of a cell into the level 1 summary
status: complete
created: 2026-09-02
implements: [ADR-0022 D2, ADR-0023 D3, ADR-0024 D1, ADR-0024 D3, ADR-0072 D4]
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
2. The build reads the original food of each tile into the ground part of the
   cell, once, beside the height total. The original stock is a pure function
   of the seed and the address, so it does not change for the life of a
   world.[^4] [^11]
3. The rebuild subtracts what the depletion ledger holds for the tiles of the
   cell. The summary therefore reports what the tiles still hold, which is
   what a tile reader reports.[^5]
4. The summary gains an intensive accessor for the mean food, beside the
   accessors the other totals already have.[^3]

## Impact review

**Governed by.** Nine decisions govern this work.

ADR-0022 D2 requires that a level 1 cell equals the exact combination of the
level 0 tiles it covers.[^6] The food total therefore holds the remaining
stock, tile by tile, and not the original stock, because the remaining stock is
what a tile holds.

ADR-0022 D3 forbids a simulation system to write a summary. The gather resolve
writes the ledger at level 0. The rebuild reads it.[^7]

ADR-0023 D1 and D2 require an exactly associative and commutative combine
operation with an identity. The new field combines by 64-bit integer addition,
and its identity is zero.[^8] [^9]

ADR-0023 D3 requires exact integer or fixed-point arithmetic. A stock is a
whole number, so the accumulator holds whole numbers.[^10]

ADR-0024 D1 requires each field to declare itself extensive or intensive. The
food total is extensive and says so. ADR-0024 D3 requires an intensive reading
to be a division of two extensive fields at read time, so the mean food is not
stored.[^12] ADR-0024 D4 requires the denominator to be the extent the field is
defined over. Every tile has a food stock, and water has a stock of zero, so
the denominator is the tile count and not the open tile count.[^13] ADR-0024 D5
requires the read to return no value over an empty cell.[^14]

ADR-0072 D1 states that the original stock is generated. ADR-0072 D4 states
that the engine stores what was taken and nothing else. The split between the
build and the rebuild follows that split exactly.[^4] [^5]

ADR-0068 D1 states that the ground is generated and never stored as a map, and
its consequences call a sweep of the whole world every frame a design
mistake.[^11] The original food is read once, at build time, for the same
reason the height total is.

ADR-0004 D1 requires an explicit iteration order. The rebuild visits the tiles
of a cell in index order and the ledger in key order. Both are fixed.[^15]

ADR-0001 D4 hashes the whole world each frame.[^16] **Level 1 does not enter
that hash.** The level is derived, and the tiles it reads are hashed already,
so a hash over it would say the same thing twice. This item therefore moves no
golden file on its own. Item 0184 moves two, because it changes what a unit
chooses.

**Changes.** No record changes.

**Creates.** No record. The work adds a field to a shape that ADR-0023 and
ADR-0024 already govern. It states no constraint that those records do not
already hold, so it fails the test for whether a decision needs a record.[^17]

**Blockers.** BLK-007 governs every cost figure in this item. No measurement
exists on the target platform, so each figure below is a shape and not a
measurement.[^18]

**Precedent.** FND-181 records that the rules against inert work look for an
absent caller and do not find inert data. This item writes a value that nothing
reads until item 0184 lands. The two items are one piece of work for that
reason, and the falsification test belongs to 0184.[^19]

## Done when

- The cell summary holds a food total, declared extensive.
- The summary reports a mean food, and reports no value for a cell that covers
  no tile.
- A test recomputes the food total of every cell from level 0, through the
  public interface, and finds it equal.
- A test gathers from a tile, rebuilds, and finds the food total of that cell
  lower by what was taken.
- The fixture holds a cell whose food total is zero and a cell whose food total
  is the highest in the world, and it asserts that spread.
- No golden state hash file changes, because level 1 is derived and the world
  hash does not read it.
- The whole check command runs green.

## Outcome

The cell summary carries a food total, declared extensive, beside the five
fields it already held. The mean food is intensive and divides by the tile
count.

**The plan changed in one place.** The item said the rebuild takes the resource
field as one more argument. It does not. The stock the ground generated joins
the part of a cell that the build computes once, beside the height total,
because it is a pure function of the seed and the address. The rebuild takes
the depletion ledger instead, and subtracts the stored take. FND-182 records
the correction and the reason.

**The state hash did not change.** Level 1 is derived and the world hash does
not read it, so this item moved no golden file. The item said the opposite
before the work, and the impact review above is repaired.

**The build costs more arithmetic and no more passes.** The sweep that reads
the ground into the level already visits every tile once. Each visit now also
generates the food stock of that tile, which is two draws from the counter-based
generator. The pass count did not change, so the item that asks for a build
without a pass over every tile is unaffected. No figure here is measured, and
BLK-007 says why.

**A found defect is placed, not fixed.** The pyramid folds level 1 into a state
hash and nothing calls that fold. Item 0190 holds the question.

**Registers.** FND-182 was added. DEC-075 was closed: the summary carries food
alone, and not one total for each resource kind. No blocker opened or closed.

**Evidence.** Four defects were put back, one at a time, and the source was
restored after each. A rebuild that ignores the ledger fails the gather test. A
ground read that takes wood as if it were food fails three recomputation tests.
A run sum that ignores the resource kind fails the wood test. A mean that
divides by the open ground and not by the tile count passed every test that
existed, so a test for the denominator was added, and the defect then failed
it.

## References

[^1]: What a unit does in a tick, section 3.2. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^3]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^4]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^5]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^6]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^7]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^8]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^9]: ADR-0023, an aggregate combines exactly, in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^12]: ADR-0024, every summary field is declared extensive or intensive, decision D1. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^13]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^14]: ADR-0024, every summary field is declared extensive or intensive, decision D5. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^15]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^16]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^17]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^18]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^19]: Findings register, FND-181. `docs/FINDINGS.md`
