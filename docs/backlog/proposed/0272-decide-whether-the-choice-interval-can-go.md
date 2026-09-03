---
id: 0272
title: Decide whether the choice interval can go
status: proposed
created: 2026-09-02
implements: []
changes: [ADR-0064]
creates: []
serves: []
blocked-by: [BLK-007]
---

## Why

**A unit acts on a reading as old as the interval, and nobody chose that
behaviour.** The choice runs at an interval, and a stagger spreads the
population across the ticks of that interval. The interval exists because
scoring every unit on every frame was expensive.[^1]

The deciding work no longer follows the population. The choice is decided for
each cell and each bucket of need, and a unit reads the answer.[^2] A cell
scores at most the bucket count, whatever the population, so the reason for the
interval has weakened.

The record on cost states this and declines to settle it, because removing the
interval has a frame consequence that nobody has measured under the new
shape.[^3] **The prize is behavioural, not a cost saving.** A unit that reacts
late is the behaviour this would remove.

## What is missing before this is refined

- **A measurement on the target platform.** Every cost figure in this project is
  derived, and no measurement exists on the target platform.[^4] An interval of
  zero makes every cell score on every frame, and the deciding work then rises
  by the interval. Whether the frame absorbs it is not knowable from here.
- **Whether the apply cost is the binding term.** The deciding work follows the
  lattice; applying an answer to a unit is per-unit by necessity. At an interval
  of zero the apply touches the whole population on every frame, and that term
  is the one the new shape does not remove.
- **What the mover count does.** The score floor is a frame-budget parameter
  because it decides how much of the population moves.[^5] A unit that re-reads
  the world on every frame may cross the floor more often, and the movement
  stage is sized against that count.
- **Whether stickiness survives.** The stored choice is sticky because a unit
  that re-decides on every tick swaps between two options of nearly equal score
  and arrives nowhere.[^6] Removing the interval removes the mechanism that
  makes it sticky, so stickiness would need its own rule.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: ADR-0098, the choice is decided for each cell and each bucket of need, decisions D1 and D3. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^3]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, the consequences. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^6]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
