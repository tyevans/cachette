---
id: 0126
title: Repair the admits-nobody claim in ADR-0074
status: complete
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The record that permits an over-fill states that a tile above its capacity
offers no room and admits nobody, while the units standing on it may still
depart.[^1] The code does not do that.

A frame runs several admission passes. A departure releases room at the end of
a pass. A tile that loses enough units inside one frame falls below its
capacity, and a later pass of the same frame admits against the lower count.
The tile then takes units in.

A test measured it. A tile held eleven units on ground that admits eight. Nine
units left in the first pass, and two arrived in a later pass. The findings
register holds the correction.[^2]

The monotone guarantee is untouched. No tile gains a unit beyond its capacity,
and an over-fill still relaxes toward the capacity and never away from it. Only
the sentence about the mechanism is wrong, and it is wrong in a record that a
reader will use to reject a change.

## What the work does

State the refusal against the occupancy after the departures of the same tick.
An over-full tile offers no room while it stays above its capacity.

The record was accepted on 31 August 2026 and it has no dependent record. A
reviewer decides whether the repair fits inside the retcon window or needs a
superseding record.[^3]

## Outcome

**Done.** The reviewer who accepted the record amended it in place on
1 September 2026, and the record carries the amendment note itself rather than
leaving the audit trail in a commit message alone.

D2 now states the refusal as a condition. A tile offers no room for as long as
it stands at or above its capacity, admission reads the occupancy after the
departures of the pass, and a frame runs several passes. An over-full tile
therefore takes units in once its own have left. What it never does is rise, and
that is the guarantee the decision exists to state.

**The retcon window was judged rather than assumed.** The window is written for
a record nothing depends on, and three source files cited this one when the
amendment was made. All three were written by the work that found the error and
all three assert the corrected behaviour, so the amendment made the record agree
with its dependents rather than breaking them. No other record cites it. A
superseding record would have said only that one sentence named a mechanism
wrongly, which the scope rule calls a record nobody needed. The judgement and
its reasoning are in the record.

## Done when

- No record says that an over-full tile admits nobody without stating the
  condition.
- The finding is cited from the repaired text.
- The test suite that found this still passes.[^4]

## References

[^1]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^2]: Findings register, FND-110. `docs/FINDINGS.md`
[^3]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`
[^4]: The over-filled tile suite. `crates/cachette-core/tests/over_filled_tile.rs`
