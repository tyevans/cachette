---
id: 0084
title: Give a tile one faction column
status: proposed
created: 2026-08-31
---

A tile now carries two values that name a faction.

The first is the holder. It says who holds the ground, it changes while the
world runs, and it names nobody where nobody holds.[^1]

The second is the tile faction column of the stub system. It is written when
the world is built, from the tile index and the faction count. It never
changes, it covers water as well as open ground, and its only reader is the
event that the tile stub emits.

One fact with two declaration sites is the shape this project keeps
finding.[^2] Here the two sites do not even mean the same thing, which is
worse: a reader who takes the stub column for a holder gets a confident wrong
answer, and nothing fails.

The work is to remove the stub column and to let the event carry the holder.
The impact review must answer three things. The event field is a faction
identifier, and the holder may name nobody, so the review must say what the
event carries for an unheld tile. The state hash reads the stub column, so
every golden file moves. The viewer may read the field, so the sweep must
cover it.

## References

[^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
