---
id: 0499
title: Give every level 1 cell a wind that carries its momentum
status: proposed
created: 2026-09-05
implements: [ADR-0160]
changes: []
creates: []
serves: [PRD-0004]
blocked-by: [BLK-130, BLK-007]
---

## Why

The project owner read the demonstration on 5 September 2026 and said that the
weather is "kinda foggy in areas". The owner wants to follow a pattern across
the map over many ticks, and wants one part of the map to hold a storm while
another part holds none.

A measurement shows why neither is possible today. The weather spread hands
every neighbour the same share, so it carries no direction at all. One storm
raised at the strength ceiling holds its maximum at the cell it was raised on
for two ticks, halves each tick, and is indistinguishable from a calm world by
the twentieth tick. The maximum flattens where it stands. It never travels.[^1]

**No balance value repairs this.** The share a cell hands a neighbour is a
scalar, and no scalar makes a symmetric kernel carry a maximum one way rather
than another. The engine needs a quantity it does not hold: a direction and a
speed for each level 1 cell.

A decision record states the shape.[^2] This item builds it.

## What the tree already holds, and what it does not

Read on 5 September 2026.

- `crates/cachette-core/src/weather.rs` holds the field. It holds an air plane,
  a ground plane, a scratch plane, two running totals and a readiness column
  for each faction. **It holds no direction of any kind.**
- The spread pass in the same file is already a gather over a settled plane
  into a scratch plane, and it already writes disjoint output. That is the
  shape a wind pass needs.
- `crates/cachette-core/src/pyramid.rs` holds the level 1 summary. It carries
  the tile count, the open tile count and the height total of each cell, as
  exact integers. **The two inputs of the heat are already there**, and the
  step rebuilds them before the weather solve runs.
- **The picture already exists.** The viewer holds a weather overlay and a
  weather panel. The work of this item is the field, not the picture.

## What the work does

1. Add the wind of each cell to the weather field, as a bounded integer vector
   in the two-axis basis of the lattice.
2. Fold the wind into the state hash, in ascending cell index order, beside the
   two water planes.
3. Compute the heat of a cell from the level 1 summary, into a scratch buffer
   that no tick outlives.
4. Add a wind pass that reads the settled wind and writes a separate plane, at
   a fixed pass count.
5. Make the pressure difference across a cell accelerate the wind by a bounded
   step, take a drag share off the speed each pass, and clamp the speed at the
   ceiling.
6. Add the balance rows for the heat weights, the pressure scale, the
   acceleration step, the drag share, the speed ceiling and the pass count,
   unset and behind their blockers.
7. Expose the wind through the Python boundary, so that the overlay and the
   panel can read it.

**This touches `fn step`.** One worker holds it at a time.

## Impact review

Not done. This item is `proposed/`, and refining it is the work.

**What refining must answer.**

- Whether the wind is one plane of vectors or two planes of components, and
  what that costs on the target cache line. The engine targets a 64-byte line
  and the development machines do not, so a local measurement misleads.
- What the wind of a cell at the edge of the lattice reads for a neighbour that
  does not exist. The spread pass answers this today by skipping, and the wind
  pass must answer it the same way or say why it differs.
- Whether the drag share and the acceleration step can produce a wind above the
  ceiling before the clamp sees it, at any input. A ceiling reached by a clamp
  is not the same as a ceiling the arithmetic cannot pass, and the review must
  say which one this is.
- Whether the wind pass takes a thread count. The stage table declares that for
  each stage, and a wrong declaration misdirects every cost table made from it.
- What the golden state hash becomes. Adding a field to the hash changes it, so
  the item must state that the golden file moves and why.

**Blockers.** BLK-130 governs the heat weights, the pressure scale, the
acceleration step, the drag share and the speed ceiling. BLK-007 governs the
pass count and every cost figure the item states.

**Waits on nothing.** Item 0500 waits on this one, because the transport it
adds reads the wind this item stores.

## Done when

- The field holds a wind for every cell, and the state hash covers it.
- A test asserts that the speed of every cell stays at or below the ceiling,
  over a long run and from a perturbed start.
- A test asserts that the wind lags the pressure: a cell whose pressure
  difference falls to nothing keeps a non-zero wind on the next tick.
- A test asserts that the drag brings a wind under no pressure difference to
  rest, and names the number of ticks it took in the output rather than in the
  code.
- The heat of a cell is derived and stored nowhere. A whole-tree search shows
  one place that computes it, and the commit body carries the search.
- The thread-count test and the golden state test pass at 1, 2 and 12 threads.
- No figure appears in the code or in a comment. Every value is a balance row.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Research report 26, the scale of the weather, sections 4 and 6. `docs/research/reports/26-the-scale-of-the-weather.md`
[^2]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
