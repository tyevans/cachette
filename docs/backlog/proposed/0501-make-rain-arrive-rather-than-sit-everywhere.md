---
id: 0501
title: Make rain arrive rather than sit everywhere
status: proposed
created: 2026-09-05
implements: [ADR-0162]
changes: [ADR-0141]
creates: []
serves: [PRD-0004]
blocked-by: [0500, BLK-130]
---

## Why

A measurement over a thousand ticks of the demonstration world, with no storm
raised at all, shows every cell of the lattice above the wet mark at every
tick. The driest cell holds nearly twice the mark and the middle cell holds
more than four times it.[^1]

**So the one reader that a simulation pass takes from the weather answers the
same everywhere.** The rule that gives a gatherer more on wet ground gives
every gatherer more, so it gives nobody anything.[^2]

The ground water itself is not flat. It runs nearly fourfold from the driest
cell to the wettest. **The value that hides that range is the mark**, and the
mark is a value rather than a decision. Setting it is part of this item and it
is the cheapest half of the answer.

The other half is the source. A cell of open water lifts a fixed quantity at
odds that read only its water share, and a share of the air falls on every cell
by its height alone. Neither rule reads the model that items 0499 and 0500
build, so the world would fill itself uniformly under a wind that carries the
uniform result about. A decision record states what the source and the sink
should read instead.[^3]

## What the tree already holds, and what it does not

Read on 5 September 2026.

- `crates/cachette-core/src/weather.rs` holds the lift and the settle. The lift
  raises a fixed quantity on a keyed draw. The settle takes a share of the air
  by the mean height of the cell, and then takes a share of the ground out of
  the world.
- The same file holds the wet mark as a constant with a blocker footnote.
  **There is no balance row for it today**, so nothing outside the code states
  what it should be. This item adds the row and fills the derivation.
- `crates/cachette-core/src/pyramid.rs` gives the mean height and the open tile
  count of each cell, so the heat of a cell needs no new source.
- **The picture already exists.** The viewer holds a weather overlay and a
  weather panel. The work of this item is the field, not the picture.

## What the work does

1. Make the evaporation of a cell rise with the heat of that cell, and let it
   lift from wet ground as well as from open water.
2. Keep the two running totals correct. A lift from open water raises the total
   that counts what entered the air. A lift from the ground does not, because
   that water is already in the account.
3. Make the share of the air that falls rise with the cooling the air meets,
   rather than with the mean height alone.
4. Add the balance rows for the wet mark, the lift quantity, the lift period,
   the evaporation share and the rain share, and write a first provisional
   value for the wet mark against the measured distribution.
5. Check that a god's storm still behaves. The gate is unchanged, and the storm
   is now an injection into a system that carries it.[^4]

**This touches `fn step`.** One worker holds it at a time, and it waits for
item 0500 to merge.

## Impact review

Not done. This item is `proposed/`, and refining it is the work.

**What refining must answer.**

- Whether the wet mark should be a fixed value or a share of what the world
  holds. A fixed mark goes stale the moment the lift rate changes, and the
  measurement shows the lift rate is itself unset. The review must choose and
  say why.
- What "the cooling the air meets" reads, exactly. It needs the heat of the
  cell the air came from, which means the pass reads the wind as well as the
  heat. The review must name the one place that composes them.
- Whether the evaporation from the ground can empty a cell below zero at any
  heat, and whether the truncation leaves a residue that never lifts.
- Whether the account still balances when a drop leaves the ground into the air
  of the same cell. That path does not exist today, and the check must see it.
- Whether raising the wet mark breaks any test that assumes wet ground. A
  whole-tree search for the mark is part of refining, not part of the work.

**Blockers.** BLK-130 governs the wet mark, the lift quantity, the lift period,
the evaporation share and the rain share.

**Waits on item 0500**, because the cooling rule reads the direction the air
travelled.

## Done when

- A test steps a world with no storm and asserts that some cells are wet and
  some are dry at the same tick. **This is the statement the measurement says
  is false today**, and it fails before the change.
- A test asserts that ground on the near side of high ground holds more water
  than ground on the far side, under a steady wind.
- The account check passes at every tick of a long run, including the new path
  from the ground into the air.
- A test asserts that a storm raised at the strength ceiling still refuses a
  place whose cell holds no ground of the caller's faction.
- The wet mark has a balance row with a filled derivation column, and the code
  reads it from one place.
- The thread-count test and the golden state test pass at 1, 2 and 12 threads.
- No figure appears in the code or in a comment.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Research report 26, the scale of the weather, sections 2, 3 and 6. `docs/research/reports/26-the-scale-of-the-weather.md`
[^2]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
[^3]: ADR-0162, water enters the air where it is hot, and it falls where the air cools. `docs/adrs/draft/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
[^4]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
