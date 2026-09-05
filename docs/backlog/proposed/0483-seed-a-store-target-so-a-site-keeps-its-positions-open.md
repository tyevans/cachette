---
id: 0483
title: Seed a store target so a site keeps its positions open, and count the positions that exist
status: proposed
created: 2026-09-05
implements: [ADR-0099]
changes: []
creates: []
serves: [PRD-0048, PRD-0016]
blocked-by: [BLK-050]
---

## Why

**A site closes every position as soon as its store passes the default
target, and the census cannot say so.** A site opens positions in proportion
to what it lacks. Every site starts with a target of 1.0 for every kind of
work, and the seeding layer sets no other value. A store climbs to hundreds, so
the site lacks nothing and opens nothing. Two of four sites in the
demonstration world hold no position by tick 200, and the game ends with
none.[^1]

The census row `seats_filled` counts positions that exist and have a holder.
A zero in it reads as "positions stand empty", and the true meaning is "no
position exists". A reader cannot tell the two apart, because no row counts
the positions that exist.

This item does two things. It seeds a store target that keeps positions open
while a site has people to fill them. It adds a `seats_open` row beside
`seats_filled`, so a zero has one meaning.

## What is missing before this is refined

- The decision: does a store target scale with the group that lives at the
  site, or with the housing that the site holds? The answer decides which pass
  writes the target and when it changes. It belongs to the record that governs
  the position pass, and that record is a draft.[^2]
- The impact review against ADR-0099, decision by decision, and whether the
  answer above needs a new decision in that record or a new record.
- The balance register row for the store target. The value is unset under
  BLK-050, and the item must express it as a parameter.[^3] [^4]
- Whether `seats_open` joins the subsystem census, and whether the balance
  harness reads it.
- The extreme that the fixture reaches: a site whose store passes the target
  and keeps its positions, and a site with no people, which opens none. The
  defect put back and the test red.
- The "Done when" statements: the two determinism tests at 1, 2 and 12
  threads, and the type stub edited by hand in the same commit as any new
  census row.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-483. `docs/FINDINGS.md`
[^2]: ADR-0099, a site fills its positions by one sort and one scan. `docs/adrs/draft/adr-0099-a-site-fills-its-positions-by-one-sort-and-one-scan.md`
[^3]: Balance register. `docs/reference/balance.md`
[^4]: Blockers register, BLK-050. `docs/BLOCKERS.md`
