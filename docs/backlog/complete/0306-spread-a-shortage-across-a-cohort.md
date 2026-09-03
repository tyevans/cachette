---
id: 0306
title: Spread a shortage across a cohort
status: complete
created: 2026-09-03
implements: [ADR-0002 D1, ADR-0003 D1, ADR-0004 D1, ADR-0009 D1, ADR-0023 D1, ADR-0063 D2, ADR-0063 D4]
changes: []
creates: [ADR-0106]
serves: [PRD-0007]
blocked-by: []
---

## Why

**A whole faction left the world on one tick.** The pass that feeds a unit
divided the share of its cohort by the headcount, so every unit gained the same
amount. Every other input to a need is shared too, so the units of a cohort held
one deficit value between them and crossed the death bound together.

An accepted record says a per-unit accumulator removes that cliff.[^1] It does
not. It delays it. The findings register holds the measurement.[^2]

A shortage should take part of a cohort and leave a population the ground can
carry. Supply is fixed and demand follows the headcount, so that population
exists in the arithmetic. The engine could not reach it.

## What the work did

1. A cohort serves whole rations to as many of its units as its share covers,
   and serves the rest nothing.
2. The served set is the ordinals of the cohort, rotated by an offset keyed on
   the cohort and the frame.

## Impact review

**Governed by.**

- **ADR-0063 D2.** A unit draws from the store of its site as a cohort. The
  draw is unchanged. This work changes only how what the cohort received
  reaches its members.
- **ADR-0063 D4.** Crossing the threshold accumulates and the accumulator ends
  a unit. Both hold. What changes is that two units of one cohort now hold two
  values.
- **ADR-0003 D1.** The offset is keyed on the system, the frame, the cohort row
  and the draw index. The consumption pass took no draw before, so it took a
  system identifier of its own.
- **ADR-0009 D1.** Each unit writes its own need and its own deficit. No two
  threads write one value.
- **ADR-0004 D1.** The ordinal comes from a walk in slot order and the offset
  from a key that holds no thread.
- **ADR-0023 D1.** The parts sum to the whole. A rotation is a bijection, so
  exactly as many units eat as the share covered.
- **ADR-0002 D1.** Every value is an integer or a Q16.16 value.

**Changes.** No record changes. **ADR-0063 keeps every decision it holds.** Its
reasoning about the cliff is corrected by the finding rather than by an edit,
because an accepted record does not change except in status.[^3]

**Creates. ADR-0106.** The registry row was added before the record was
written.

**Blockers.** None.

## Done when

- Two units of one short cohort hold two needs. A test asserts it.
- A cohort that can feed everybody gives one answer to everybody.
- A short cohort loses part of itself and keeps the rest.
- A cohort serves exactly as many rations as its share covered, at several
  shares and over many applications.
- The served set changes from one application to the next.
- The result is the same at 1, 2 and 12 threads.
- Every account balances while a cohort is short.
- Each test was watched to fail with a defect put back. Three defects were
  restored in turn: the equal split, the frame taken out of the key, and the
  per-unit draw.

## Outcome

Seven tests pass. All three probes were caught: the equal split fails four of
them, the frame taken out of the key fails one, and the per-unit draw fails two.

**The first implementation created food.** A keyed draw for each unit gives each
unit an independent chance, so the number that eats is binomial rather than the
count the store paid for. A fixture whose share covered one ration served two
units on one application and none on another. Neither conservation check saw
it, because the account they balance is the commodity and the commodity had
correctly left the store. The exactness test is what caught it, and the record
was rewritten to state a rotation rather than a draw.

**The demonstration world now behaves as asked.** The same 4000-tick run loses
units a few at a time instead of a whole faction at once, and settles at 53, 46,
62 and 64 of 64. The second faction lands on exactly the 46 its supply carries,
with no deficit and a store that grows.

**A unit's fortune is not a property it carries.** An ordinal is a position
among the live members of a cohort, so a death shifts it. Neighbours in slot
order also eat together within one application. Both are stated in the record.

## References

[^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D1. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^2]: Findings register, FND-318. `docs/FINDINGS.md`
[^3]: ADR Registry, the status rule. `docs/adrs/REGISTRY.md`
