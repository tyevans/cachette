---
id: 0500
title: Carry the air water along the wind
status: proposed
created: 2026-09-05
implements: [ADR-0161]
changes: [ADR-0141]
creates: []
serves: [PRD-0004]
blocked-by: [0499, BLK-130, BLK-007]
---

## Why

The project owner asked on 5 September 2026 for weather that a watcher can
follow across the map. A measurement shows that nothing in the weather field
moves in a direction: one storm raised at the strength ceiling covers about a
third of the lattice in one tick, holds its maximum where it was raised for two
ticks, and leaves nothing by the twentieth tick.[^1]

Item 0499 gives every cell a wind. **A wind that nothing reads is a capability
nobody invokes**, which is the defect shape this project names first.[^2] This
item is the reader: it makes the water ride the wind.

A decision record states the rule, and it changes the transfer rule the field
uses today.[^3] [^4]

## What the tree already holds, and what it does not

Read on 5 September 2026.

- `crates/cachette-core/src/weather.rs` holds the spread pass. It is already a
  gather over a settled plane into a scratch plane, and it already writes
  disjoint output. **The share it hands each neighbour is one constant**, and
  that constant is the whole reason the field has no direction.
- The same file holds the two running totals and the account check. That check
  is the only reader that sees a lost drop, and it must keep working.
- **The picture already exists.** The viewer holds a weather overlay and a
  weather panel. The work of this item is the field, not the picture.
- Nothing today carries a direction between two cells anywhere in the weather
  module.

## What the work does

1. Make the share a cell sends in one direction rise with the part of that
   cell's wind that points that way.
2. Keep the pass a gather. The receiver computes the giver's share from the
   same settled plane with the same truncating division that the giver would
   use, so both ends of an edge reach one integer.
3. Prove the bound: for every wind at or below the speed ceiling, the sum of
   the quantities a cell sends over one pass stays below what that cell holds.
4. Add the balance rows for the transport share and revisit the pass count row,
   unset and behind their blockers.
5. Lower the transport share and the pass count, so that a front does not cross
   a small map in a few ticks. The measurement gives the present rate.[^1]

**This touches `fn step`.** One worker holds it at a time, and it waits for
item 0499 to merge.

## Impact review

Not done. This item is `proposed/`, and refining it is the work.

**What refining must answer.**

- How the share for one direction is computed from a wind vector without a
  trigonometric function and without a floating point number. This is the whole
  arithmetic of the item, and it must go through the arithmetic module.
- Whether the receiver can compute the giver's share with fewer reads than the
  giver would need. A gather reads every neighbour's wind, so the pass reads
  more than it did.
- Whether the isotropic case survives exactly. A cell with no wind must behave
  as it does today, and a test should show it.
- What the pass does at the edge of the lattice, where a cell has fewer
  neighbours and its wind points off the map. Water that leaves the world must
  either be refused or accounted, and the review must choose one and say so.
- Whether the change to ADR-0141 D1 is stated in the record before the code
  lands. It is, and the review must confirm that no other decision of that
  record is contradicted.

**Blockers.** BLK-130 governs the transport share. BLK-007 governs the pass
count and every cost figure the item states.

**Waits on item 0499**, which stores the wind this item reads.

## Done when

- A property test asserts that the air total of the world is unchanged by a
  transport pass, for a set of random winds and random air planes.
- A property test asserts that no cell falls below zero, for a wind at the
  ceiling in every direction.
- A test injects water at one cell, steps the world, and asserts that the
  maximum of the air plane is at a different cell than it was, in the direction
  the wind points. **Put the isotropic share back and watch this test fail**,
  and record that in the commit body.
- The account check passes at every tick of a long run.
- The thread-count test and the golden state test pass at 1, 2 and 12 threads.
- No figure appears in the code or in a comment. Every value is a balance row.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Research report 26, the scale of the weather, sections 4, 6 and 7. `docs/research/reports/26-the-scale-of-the-weather.md`
[^2]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^3]: ADR-0161, water rides the wind, and every transfer is an exact integer move. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^4]: ADR-0141, a weather pass moves water and never scales it, decision D1. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
