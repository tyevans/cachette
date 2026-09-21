---
id: 0493
title: Fill the ground one faction has surrounded
status: refined
created: 2026-09-05
implements: [ADR-0153 D6, ADR-0153 D7]
changes: [ADR-0150 D1]
creates: []
serves: [PRD-0054]
blocked-by: [BLK-050, BLK-007]
---

## Why

**A faction that surrounds a piece of ground does not hold it.** A tile is held
by the faction of the nearest city whose reach covers it, and a tile that no
city reaches is held by nobody.[^1] A lake inside one faction's ground, a
mountain that no city reaches, and a gap between two discs of the same faction
all belong to nobody, however far inside that faction they sit.

The project owner asked how the space in between fills when one faction's
ground mostly swallows a piece of land. The product record states the need, and
an accepted decision record states the rule.[^2] [^3]

This item adds the closure passes of that rule. After the city reach and the
lease have given every candidate tile a holder, a fixed number of passes run. In
one pass, an unheld tile whose neighbours name exactly one faction, at or above
the neighbour threshold, becomes that faction's. Each pass reads the column the
previous pass wrote and writes a new one, so two threads write disjoint
outputs. A neighbour outside the world counts as no neighbour.

**Closure takes unheld ground only.** It never takes ground another faction
holds, because a faction must not lose the ring around its own city with no act
by anybody.

**This item is a sibling of item 0492 and depends on it.** The lease is a
column and a pass over the units. The closure is a pass over the tiles, and it
reads only the holder column that 0492 leaves. The order is fixed by the
record: city reach, then lease, then closure.[^3]

## Impact review

**Governed by.** ADR-0153 D6 and ADR-0153 D7 govern this change. D6 defines the
closure rule: a fixed number of passes, unheld tiles only, double-buffered
columns, and neighbour thresholding. D7 defines the stage order in `fn step`.

**Changes.** ADR-0150 D1. That decision ruled that ground that admits no unit is
held by nobody whatever reaches it. ADR-0153 D6 amends that rule: closure reads
only the holder of a neighbour and asks nothing of the ground, so an enclosed
lake or mountain range becomes the faction's ground.

**Creates.** None.

**Blockers.** BLK-050 governs the pass count and the neighbour threshold.
BLK-007 governs the cost figure at the target scale.

**Order with trade settlement.** The closure passes run inside `rewrite` at the
holding spread stage (`Stage::HoldingSpread`). Trade settlement runs later in the
frame at `Stage::TradeSettle`. Traded land from tick N settles at tick N's trade
settlement and is read by tick N+1's rewrite.

**Census reporting.** The census records the closed tile count behind the
`census-holding` feature switch. The public territory readers count closed tiles
as held ground.

**Ground passability.** Five readers take the holder column: the state hash, the
tile event, the presence relation, the storm gate, and the territory score. None
of them assumes that held ground admits a unit. `Holding::check_invariants` is
updated to permit enclosed impassable ground to be held.

**Pass count and neighbour threshold.** Provisional defaults are written to the
balance register under BLK-050:
- `Closure pass count`: 2 passes. Two passes close holes up to 2 hex steps deep
  while keeping execution time bounded.
- `Closure neighbour threshold`: 4 neighbours. On a hex grid of 6 neighbours, a
  straight border has at most 3 neighbours of the held faction (so straight
  edges do not creep outward), while a concave pocket or lake has 4, 5, or 6
  neighbours of one faction and zero neighbours of any other faction.

**Target scale cost.** Candidate tiles for closure are bounded by the candidate
tiles of the rewrite (cities multiplied by reach squared, plus held and leased
tiles and their neighbours up to `pass_count` steps). The pass never scans the
16.7 million tiles of the world. Cost is
O((held + leased + cities * reach^2) * pass_count).

## Done when

- `ClosureRules` struct is defined with `pass_count` and `neighbour_threshold`.
- `Holding` stores `closure_rules`, provides accessors, and folds the rules into
  `hash_into`.
- `Holding::rewrite` runs `closure_rules.pass_count()` closure passes after city
  reach and lease candidate decisions.
- Each pass reads the column from the previous pass and writes a new column.
- An unheld tile whose valid hex neighbours name exactly one faction with count
  at or above `neighbour_threshold` takes that faction.
- Ground that another faction holds is never taken by closure.
- Boundary hex neighbours outside the world count as no neighbour.
- `Holding::check_invariants` allows enclosed impassable tiles to be held.
- Unit tests verify 1-tile hole fill, 2-tile gap fill, multi-faction refusal,
  held ground preservation, straight border stability, and boundary behavior.
- At least one test is proven able to fail when closure logic is corrupted.
- The balance register holds the provisional defaults and derivations.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^2]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^3]: ADR-0153, a tile's lease follows the units that stand on it, decisions D6 and D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
