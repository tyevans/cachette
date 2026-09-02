---
id: 0209
title: Tell a frontier from the edge of the claimed ground
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0006]
blocked-by: []
---

## Why

A held tile takes a border when any of its six neighbours holds differently.
Unclaimed ground counts as a difference, so a holding shows its whole outline.

**Two different facts draw the same picture.** A holding that meets unclaimed
ground is a frontier with nobody. A holding that meets another faction is a
frontier with somebody. The first says where a faction stopped. The second says
where two factions are in contact, and that is the one a watcher wants to find.

The product record asks that a watcher can see that a place belongs to
somebody.[^1] It does not ask that a watcher can see where two holdings meet,
and the drawing does not show it.

The counts say the distinction is real and not rare. Of the held tiles in three
sampled worlds, 83 to 89 in 100 sit on some boundary, and 71 to 77 in 100 meet
another faction. So between 6 and 13 tiles in every 100 held tiles border only
unclaimed ground, and those are the tiles the two cases separate.[^2]

## What the work might do

The shape is open. A second colour, a second weight, or a border on one case
and none on the other.

The questions this item must answer before it is refined:

- Whether a watcher wants the frontier marked, or the coastline. Marking both
  in different ways may say less than marking one.
- Whether the viewer may hold a second colour for this. The colour table is
  keyed on the faction and holds one colour for each, and a second table would
  be one fact in two places.[^3]
- What it costs. The neighbour read already happens, and it already reads the
  holder rather than a bit, so the faction of the neighbour is in hand. The
  test is a comparison and not a read.
- What a test asserts. A fixture needs a holding that meets unclaimed ground
  and a holding that meets another faction, in one window, and it must tell the
  two apart in the picture.

**Do not judge this from a rendered picture alone.** Two repairs to this layer
were proposed from a render and both were wrong: one keyed on the tile size and
one asked for behaviour the code already had. Count the tiles first.[^2]

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^2]: Findings register, FND-206. `docs/FINDINGS.md`
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
