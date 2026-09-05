---
id: 0484
title: Hold ground only within reach of an owned city, and refuse a build outside it
status: proposed
created: 2026-09-05
implements: [ADR-0150 D1, ADR-0150 D2, ADR-0150 D3, ADR-0150 D4, ADR-0053 D2, ADR-0053 D4, ADR-0004 D1, ADR-0001 D4]
changes: [ADR-0053 D5, ADR-0053 D6]
creates: []
serves: [PRD-0054]
blocked-by: [BLK-007, BLK-050]
---

## Why

**A holding spreads from wherever a unit stands and grows until something
stops it.** A faction cannot lose ground by losing a city, and cannot gain
ground by founding one. The project owner asked that a faction's ground exist
only around a city it owns, extended by the upgrades finished inside it up to a
bound, and that a unit build outside that ground only a road. The product
record states the need, and a draft record states the rule.[^1] [^2]

This item replaces the spread pass at the holding stage with a rewrite from the
cities. For each settlement, the pass computes a reach from three balance
values and the finished upgrades on the ground the settlement held last step.
It decides every tile within reach of any settlement by nearest city, then
lowest slot index, and it clears every held tile that no city reaches. The
changes go through the apply path that the spread and the land transfer use, so
the running total, the block masks and the held list repair as they do
today.[^3]

The build intent pass and the build verb both gain one test: the holder of the
builder's tile is the builder's faction, or the kind is a road. A refused order
is dropped and, where the controller gave it, counted.[^4]

**Two readers change without changing their code.** The territory score reader
reads the running total, so a faction's score becomes the ground its cities
reach. The controller's build orders meet the new refusal, so a faction with no
city sees every build but a road refused, and the refusal count rises.

**This item touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for the pass that holds `fn step` before it to merge.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0150 is a draft beside this
  item, and the registry holds its status.[^5] ADR-0053 D5 and D6 are accepted
  and this item stops the code they describe. The review must say that the
  supersession is by ADR-0150 and that ADR-0053 D2, D3, D4 and D7 stand.
- Three balance rows: the base reach, the finished upgrades that earn one step
  of reach, and the bound. Each is unset under BLK-050 and the item must add
  the rows with an empty derivation, or a provisional value with the
  derivation filled.[^6]
- Whether the holder column stays in the state hash once the cities decide it.
  ADR-0150 D3 keeps it there and says why. The review must confirm that the
  reasons hold, or supersede D3.
- Which golden files move. Every file that holds a held tile does, and the
  review must list the fixtures to regenerate in the commit body.
- The per-field tests. A tile at reach plus one is held by nobody. A tile at
  equal distance from two cities goes to the lower slot index. A faction whose
  last settlement is removed holds nothing after one step. A finished upgrade
  inside the ground raises the reach by the balance step, and an unfinished one
  does not. A road is built on unheld ground, and a terrace is refused there.
- The extreme the fixture reaches. Two cities of two factions at one distance
  from a tile, a city at the bound of its reach, and a world with no city.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, each keyed draw with a per-field test, the
  defect put back and the test red, and the demonstration run to the tick
  limit with the territory score read.[^7]
- Whether item 0370 closes with this one. It asks for the same refusal on
  ground another faction holds, and this item refuses a wider set.[^8]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, and upgrades extend the reach to a bound. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^5]: ADR Registry. `docs/adrs/REGISTRY.md`
[^6]: Balance register. `docs/reference/balance.md`
[^7]: Findings register, FND-320. `docs/FINDINGS.md`
[^8]: Backlog item 0370, refuse a build on ground another faction holds. `docs/backlog/proposed/0370-refuse-a-build-on-ground-another-faction-holds.md`
