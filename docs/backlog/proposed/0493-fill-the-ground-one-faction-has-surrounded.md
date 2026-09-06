---
id: 0493
title: Fill the ground one faction has surrounded
status: proposed
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
a draft record states the rule.[^2] [^3]

This item adds the closure passes of that rule. After the city reach and the
lease have given every tile a holder, a fixed number of passes run. In one
pass, an unheld tile whose neighbours name exactly one faction, at or above the
neighbour threshold, becomes that faction's. Each pass reads the column the
previous pass wrote and writes a new one, so two threads write disjoint
outputs. A neighbour outside the world counts as no neighbour.

**Closure takes unheld ground only.** It never takes ground another faction
holds, because a faction must not lose the ring around its own city with no act
by anybody.

**This item is a sibling of item 0492 and depends on it.** The lease is a
column and a pass over the units. The closure is a pass over the tiles, and it
reads only the holder column that 0492 leaves. They are two mechanisms with two
sets of tests and two open questions, so one item that held both would be one
worker holding `fn step` for two things at once.[^4] The order is fixed by the
record: city reach, then lease, then closure.

**This item touches `fn step`. Only one worker may hold it at a time.** It
waits for item 0492 to merge.

## What is missing before this can be refined

- **Whether closure runs before or after a land trade settles.** A land
  contract writes the holder of a set of tiles to the creditor when the other
  side delivers in full, and that write goes through the same apply path the
  rewrite uses.[^5] A closure that runs before the write leaves a hole around
  the traded ground for one step. A closure that runs after it may swallow the
  tiles the debtor kept. The record fixes the order of the three tests inside
  the rewrite and says nothing about the trade, so the work must place it.
- **What the census reports, so that the mechanism is visible rather than
  silent.** A tile that changes hands because six neighbours changed is a
  change no watcher can trace to an act. The engine counts what its passes do
  behind one feature switch, and the holding already reports the tiles that
  moved.[^6] The work must say whether the closure counts its own tiles
  separately from the tiles the lease and the city reach moved.
- **Whether a held tile that admits no unit breaks a reader.** Closure reads
  only the holder of a neighbour, so a lake one faction rings becomes that
  faction's. The rule that no faction holds water is therefore gone.[^1]
  Several readers take the holder column: the state hash, the tile event, the
  presence relation, the storm gate and the territory score.[^7] The work must
  read each one and say which of them assumed that held ground admits a unit.
- **How many passes, and how many neighbours.** Both are balance values under
  one open blocker, and the register holds a row for each.[^8] [^9] The work
  must write a provisional value into each row with its derivation, and must
  not invent a rule that hides the value.
- **What the closure costs at the target scale.** Each pass reads the unheld
  tiles, and the record states no bound on how many of those exist. The work
  must state the cost shape, and the figure stays derived until a benchmark
  runs on the target platform.[^10]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^2]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^3]: ADR-0153, a tile's lease follows the units that stand on it, decisions D6 and D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^4]: Backlog item 0492, let repeated use move the border, and let disuse return it. `docs/backlog/proposed/0492-let-repeated-use-move-the-border-and-let-disuse-return-it.md`
[^5]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
[^6]: The holding module of the core crate. `crates/cachette-core/src/holding.rs`
[^7]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^8]: Balance register. `docs/reference/balance.md`
[^9]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
