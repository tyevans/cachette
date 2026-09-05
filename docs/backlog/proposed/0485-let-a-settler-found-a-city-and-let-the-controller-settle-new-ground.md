---
id: 0485
title: Let a settler found a city and let the controller settle new ground
status: proposed
created: 2026-09-05
implements: [ADR-0150 D5, ADR-0145 D1, ADR-0145 D2, ADR-0144 D2, ADR-0144 D4, ADR-0076 D1, ADR-0003 D1]
changes: []
creates: []
serves: [PRD-0054, PRD-0012]
blocked-by: [BLK-050]
---

## Why

**No unit founds a city, and no faction settles new ground.** A settlement is
founded by a Python verb that names an address and a faction, and by the
seeding layer once at the start of a run. Once ground exists only around a
city, a faction that cannot found a city cannot grow.[^1] The product record
asks that a settler found a city on unclaimed ground, and that a faction
playing itself found new cities during a run.[^2] A second record asks that a
world start small and grow, and a second founding is one way it grows.[^3]

This item adds one capability column, settle, to the unit type table. Zero
means the type cannot found a city.[^4] It adds one verb that takes a set of
units and founds a settlement on the tile each settler stands on, for the
settler's faction. The verb refuses a unit whose settle column is zero, a tile
any faction holds, a tile that carries a settlement, a tile that admits nobody,
and a tile inside the founding distance of a settlement that stands.[^5] A
refused set changes nothing.

The controller gains one choice, settle. Whether an evaluation settles is drawn
from the keyed generator with the other choices, and the command goes through
the new verb and through no other path.[^6] A refused settle command is dropped
and counted.

The census gains one row, `settlements_founded`, which counts the foundings the
verb made since the last reset. The existing row `settlements` already counts
the cities that stand.

## What is missing before this is refined

- The impact review, decision by decision. ADR-0150 is a draft beside this
  item, and the registry holds its status.[^7] The review must say whether
  ADR-0120 needs a change, because the type table gains a column and the row
  width is what that record fixes.
- Whether a row named `cities` is needed beside `settlements`. The owner asked
  for both names. The review must say whether they count one thing, and add
  no second row for one fact.[^8]
- Where the controller's settle choice draws its place. A settler must stand on
  unheld ground to found, so the controller must first send a settler there,
  and the movement verbs are what it has. The review must name the verb.
- What the settle weight is. The controller's weight vector has four weights,
  and a fifth changes the range that the balance register holds.[^9]
- Whether the settler survives. ADR-0150 D5 says it does and names the
  alternative as a game value. The review must state which the code does.
- The per-field tests. A unit with settle zero is refused. A settler on held
  ground is refused. A settler on unheld ground founds, and the ground around
  the city is held on the next step. The controller founds through the verb
  and the founding appears in its log.
- The extreme the fixture reaches. A settler at the exact founding distance
  from a standing city, a settler on the last unheld tile between two
  reaches, and a world where every tile is held.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, each keyed draw with a per-field test, the
  defect put back and the test red, and the type stub edited by hand in the
  same commit as the verb.[^10]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0150, held ground is the ground within reach of a city its faction owns, and upgrades extend the reach to a bound. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^2]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^4]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^5]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D4. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
[^8]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^9]: Balance register, the controller. `docs/reference/balance.md`
[^10]: Findings register, FND-320. `docs/FINDINGS.md`
