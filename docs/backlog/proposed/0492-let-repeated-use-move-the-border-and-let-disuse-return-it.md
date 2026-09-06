---
id: 0492
title: Let repeated use move the border, and let disuse return it
status: proposed
created: 2026-09-05
implements: [ADR-0153 D1, ADR-0153 D2, ADR-0153 D3, ADR-0153 D4, ADR-0153 D5, ADR-0153 D7]
changes: [ADR-0150 D1]
creates: []
serves: [PRD-0054]
blocked-by: [BLK-050, BLK-007]
---

## Why

**A faction gains no ground by using it.** A tile is held by the faction of the
nearest city whose reach covers it, and the reach grows only with the upgrades
finished inside the city's own ground.[^1] A faction whose people cross the
same ground every tick for a whole run holds none of that ground.

The project owner asked that ownership follow where a faction's units are. A
faction goes to the same places outside its territory, and repeated use should
erode the border toward the natural conclusion, so that the frequency of use
becomes ownership. The product record states the need, and a draft record
states the rule.[^2] [^3]

This item gives each tile one lease. A lease is one faction and one count. At
the holding stage, every tile that carries a unit moves its lease by one step
toward the faction present, or away from another faction, and the lease changes
hands when the count reaches zero. The count falls on a fixed schedule, so
ground a faction stops visiting returns to nobody. The rewrite then holds a
tile by its lease at the claim threshold, and by the nearest city in reach
otherwise.

**This item depends on item 0484 and lands on top of it.** That item replaces
the spread with the rewrite from the cities, and this one adds a test in front
of that rewrite. It must not start before 0484 merges, and it must not repeat
the work 0484 does.

**This item touches `fn step`. Only one worker may hold it at a time.** It
waits for the pass that holds `fn step` before it to merge.

**The closure of a hole is item 0493, not this item.** That item implements the
closure passes of the same record, and it depends on this one.[^4]

## What is missing before this can be refined

- **Whether a unit inside its own held ground raises the lease at all.** A rule
  that raises everywhere writes a lease on every tile a faction lives on, which
  is work the city reach already does. A rule that raises only outside the
  faction's own ground writes far fewer tiles, and it changes what happens when
  a faction loses a city: the ground it lived on would then hold no lease to
  fall back on. Neither shape is in the record, and the record must gain the
  answer before the work starts.
- **Whether a unit that is merely passing counts as much as a unit that is
  working.** A unit that crosses a tile in one tick and a unit that gathers on
  it for a hundred ticks both stand on it. The owner asked that use become
  ownership, and gathering, building and marching are three different uses. A
  rule that reads what the unit is doing needs the state of that unit at the
  holding stage, and a rule that ignores it makes a road as strong a claim as a
  mine.
- **What the census reports, so that the mechanism is visible rather than
  silent.** A border that moves with no fight and no event is a change a
  watcher cannot explain.[^3] The engine counts what its passes do behind one
  feature switch, and the holding already reports the tiles that moved.[^5] The
  work must say which counts the lease adds, and whether a change of hands
  writes an event that a caller reads at the frame barrier.
- **Whether a lease transfers with traded land.** A land contract writes the
  holder of a set of tiles to the creditor when the other side delivers in
  full.[^6] The lease of those tiles still names the debtor, so the next
  rewrite may take the ground back at once. One open question already asks
  whether an upgrade changes hands when the ground does, and this is the same
  question about a different value.[^7] The work must express the answer as a
  parameter rather than invent it.
- **What the lease costs in memory at the target scale.** The lease adds two
  values for each tile, over 16.7 million tiles. The work must state the cost
  and say what width each value takes, and the figure stays derived until a
  benchmark runs on the target platform.[^8]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^2]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^3]: ADR-0153, a tile's lease follows the units that stand on it. `docs/adrs/draft/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^4]: Backlog item 0493, fill the ground one faction has surrounded. `docs/backlog/proposed/0493-fill-the-ground-one-faction-has-surrounded.md`
[^5]: The holding module of the core crate. `crates/cachette-core/src/holding.rs`
[^6]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
[^7]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
