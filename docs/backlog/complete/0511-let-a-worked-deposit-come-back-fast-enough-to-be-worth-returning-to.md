---
id: 0511
title: Let a worked deposit come back fast enough to be worth returning to
status: complete
created: 2026-09-06
implements: []
changes: []
creates: []
serves: [PRD-0018, PRD-0024]
blocked-by: [BLK-050]
---

## Why

**A deposit comes back six hundred times more slowly than a unit empties it.**
One unit takes 4 stock from a tile in one tick, and a food tile gives back one
unit every 600 ticks. A wood tile gives back one unit every 2400 ticks. Stone
never comes back. Over a run of 20000 ticks a food tile regains at most 31
units and a wood tile at most 7.[^1]

The core states the intent in its own comment: the gather rate is high against
the stock of a tile, so a full tile of gatherers always empties a deposit. The
recovery was written to answer that, and the two rates were never compared.

**The consequence is the one the product record already named.** A worked tile
is spent and does not recover, so the ground near a settlement is bare within a
few hundred ticks, and after that a faction gathers nothing. The record that
asks for a run to stay eventful names this as one of the two behaviours that
make a run go quiet.[^2] The recovery mechanism it asks for exists and is too
weak to be observable.

**This is the cheapest change on the list, because a caller already sets it.**
The recovery periods are one array of three values and a public verb writes
them. No engine change is needed to measure a different value. The core
constant is a second declaration site of the same number, and the register now
holds the row.[^3]

## What the work does

1. Measure the run at recovery periods a gatherer can meet, and report the
   distribution over the seed set rather than a mean.
2. Set the core constants from what that measures, and record the derivation in
   the register row.
3. Give the recovery a test that empties a tile, waits, and asserts that the
   stock came back. The test must fail when the period is put back.

## What is missing before this is refined

- **What the ground is for.** A deposit that refills quickly makes the ground
  a flow and removes the reason a unit moves. A deposit that never refills
  makes it a frontier. The two are different games and the record does not
  choose.
- **Whether stone should come back at all.** A kind that never recovers is a
  deliberate statement if something else replaces it, and an oversight if not.
- **Whether the gather rate is the value to move instead.** Lowering the gather
  rate reaches the same ratio and changes what a single unit is worth.

## Impact review

Not done. This item is in `proposed/` and refining it is the work.

## Outcome

Complete. A worked deposit now comes back at a rate that the ground and the
improvement decide, rather than at one rate for the whole kind.

**What was built.** The recovery rate answers to three things inside one
declaration: the kind, which states a base period in ticks; the moisture, which
indexes a table of seven bands that scales the period; and the improvement,
which divides the period by a column of the upgrade row, scaled by the condition
of the site so that a neglected terrace falls back toward the unimproved rate.

**The declaration moved from units for each day to ticks.** The rate the owner
asked for is below one unit a day, and a rate stated in units for each day
cannot reach that without a period of zero.

**The band edges come from a measurement.** A probe walked 2000 ticks of a 192
by 192 world and sorted the ground plane every 50 ticks. Of 1440 cell samples,
most held under 32 drops and the tail reached 1395, so even steps would have put
almost every cell in one band.

**The record.** ADR-0170 states the shape and changes ADR-0080 D5 in two
clauses. ADR-0080 D5's real constraint, one declaration site for the rule,
survives.

**One rule, one function.** `World::recovery_period_at` exposes the same rule to
a caller, so a display and the pass cannot disagree.

## References

[^1]: Findings register, FND-549. `docs/FINDINGS.md`
[^2]: PRD-0024, a run stays eventful for as long as it is watched.
`docs/product/idea/prd-0024-a-run-stays-eventful-for-as-long-as-it-is-watched.md`
[^3]: Balance register, the ground and what it gives.
`docs/reference/balance.md`
