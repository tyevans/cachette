---
id: 0167
title: Index the units of one dwelling
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0014]
blocked-by: []
---

## Why

The household read passes over the unit arena to find the residents of one
dwelling.[^1] PRD-0014 rejects a residency query that walks the population,
because a watcher asks about a place often.

A reverse index from a dwelling slot to the units that name it would answer
the same question without the pass. The unit-to-tile bridge is the shape:
derived arrays that own no unit, rebuilt at the frame barrier by a sort on a
key, with a check that derives the index again and compares.[^2]

The destroy path has the same shape of cost. It reads every unit of the world
to find the residents of one lost site, and FND-116 names that as the open
part of the eviction.[^3]

## What is missing before this is refined

- **The measurement.** Nothing on the target platform has been measured, so
  nothing says the pass is too slow.[^4] Take this item when a measurement
  asks for it. An index built before then is a cost paid against a guess.
- **The staleness rule.** A derived index is stale between the write and the
  rebuild. The bridge answers that with an error rather than a wrong
  answer.[^2] Whether the household read must do the same, or must stay a
  direct read that is never stale, is a decision and it needs a record.
- **Whether one index serves both callers.** The eviction and the household
  read ask the same question. If one index serves both, the eviction stops
  passing over the population as well.

## Done when

Filled in when the item is refined.

## References

[^1]: Backlog item 0103. `docs/backlog/complete/0103-derive-a-household-from-the-dwelling-slot.md`
[^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decisions D1, D3 and D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^3]: Findings register, FND-116. `docs/FINDINGS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
