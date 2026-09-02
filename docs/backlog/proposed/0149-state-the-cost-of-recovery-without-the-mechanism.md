---
id: 0149
title: State the cost of recovery without the mechanism
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: [PRD-0018]
blocked-by: []
---

## Why

A product record states a need. It never states a structure.[^1]

The product record for a deposit that comes back states a structure. Its cost
section says what the world stores, says that recovery removes a stored record,
and says that the world answers the amount when a caller asks. A decision
record already holds all three claims, and that record is under review now.[^2]
The product record therefore holds a second copy of a decision. One fact in two
places, with nothing that fails when the copies disagree, is the shape this
project keeps meeting.[^3]

The check does not see this. It fails a record that cites a decision record by
number, and this record cites none.[^4] A review found it by reading, and a
finding records the gap.[^5]

The record cannot be accepted until this changes. Everything else in it passes
the six gate questions.[^6]

**What refining this must answer.** Whether the cost section can state the cost
without naming the depleted set at all, or whether "the set of deposits units
depleted" is a property of the need rather than of the store. The review takes
the first position. A refiner that takes the second must say why the name is
not a structure.

## What the work does

1. The cost section keeps the four cost claims. The cost of recovery grows with
   the number of deposits that units depleted, and not with the tile count or
   the extent. It does not grow without bound over a long run. A world where
   nobody gathered pays nothing. Every amount is an exact whole number, so a
   total combines the same in any order and at any thread count.
2. The cost section loses every sentence that says what the world stores or
   does not store.
3. The cost section loses every sentence that says recovery removes or shrinks
   a record.
4. The cost section loses the sentence that says the world answers the amount
   when a caller asks.
5. The crop paragraph loses the two sentences that describe what the engine
   carries today. The distinction stands on the sentence that a crop is an act
   on a chosen site and recovery is not.
6. A reviewer then accepts the record, or states what still fails.

## Done when

- The cost section of the product record states cost alone.
- No sentence in the record names a store, an algorithm, or a module.
- The record still answers all six gate questions.
- The record check and the whole gate command run green.
- A reviewer moves the record to `accepted/`, or writes what still fails.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product requirement records, what does not belong here. `docs/product/README.md`
[^2]: ADR-0080, a depleted deposit recovers by ageing the stored take, decisions D1 and D2. `docs/adrs/draft/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: The product record check. `scripts/check_prds.py`
[^5]: Findings register, FND-134. `docs/FINDINGS.md`
[^6]: Reviews, the founding and deposit product records. `docs/reviews/0149-the-founding-and-deposit-product-records.md`
