---
id: 0485
title: Let a settler found a city and let the controller settle new ground
status: complete
created: 2026-09-05
implements: [ADR-0150 D5, ADR-0145 D1, ADR-0145 D2, ADR-0144 D2, ADR-0144 D4, ADR-0076 D1, ADR-0003 D1]
changes: [ADR-0150 D5]
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

## What this item owns, and what item 0497 owns

**This item owns what a settler does. Item 0497 owns where a settler comes
from.** The two were written apart and neither claims the other's work.

- This item adds the settle capability column, the verb that founds a city with
  a settler, the controller's settle choice, and the census row for the
  foundings.
- Item 0497 builds the per-site production queue, and a settler is one of the
  types that comes off it.[^11]

**Neither item is enough on its own, and this one is not blocked by the other.**
A test of this item may set a unit's type through the type verb, and that path
exists today. A faction playing itself cannot settle until both land, because
nothing else gives it a settler.

The refinement of this item must not plan a way to make a settler. If it needs
one, it names item 0497.

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

- The unit type table holds a settle column, and zero in it means the type
  founds no city.
- One verb founds a city from a set of units, and every refusal it makes is a
  named outcome.
- The controller holds a settle option, and a run of the demonstration world
  founds cities that the seeding did not.
- The production queue reaches a settler with no change of its own.
- The two determinism tests pass at one, two and twelve threads.

## Outcome

### What was built

**The settle column.** The unit type row gains `settle_group`, a whole number.
Zero means the type founds no city. A value above zero is the group that a
founding by a unit of this type seats at the new city. The default table gains
the settler row at index four, and the open row moves to index five. The row
goes from eight columns to nine, and it still holds no padding.

**The verb.** `settle_set` takes a set of units and answers each one. For a
unit whose settle column is above zero it founds a settlement on the tile the
unit stands on, for the faction of the unit, and it then removes the unit. The
Python boundary calls it `order_settle`, and the call returns the number of
cities it founded.

**The refusals**, each a named variant: no such unit, not a settler, outside
the world, a faction the world does not hold, ground a faction holds, a
settlement already standing, ground that admits nobody, and a place inside the
founding distance. A refused unit changes nothing and keeps its life.

**The controller option.** The weight vector gains a fifth weight, settle. The
choice set gains one command at the draw index past the queue order. A faction
that holds a settler draws once for it, biased by that weight, and a refused
command is counted.

**The settlers.** The production queue needed no change. The controller offers
every filled row of the unit type table to its type draw, and the build cost
table holds one placeholder row for every type, so the settler row became a
buildable type the moment the table held it.

### How the founding distance is honoured

The verb derives the place of every settlement that stands and hands the list
to the survey the seeding uses.[^12] That survey holds the one comparison
against the minimum founding distance in the tree. The verb reads the
`separated` flag of the candidate for the distance refusal, so no second copy
of the rule exists.

### The record that changed

ADR-0150 D5 said that the settler survives the founding, and named the cost as
a game value. The project owner asked that the founding spend it. The record is
a draft, and D5 now states that the founding spends the settler and seats the
group the column names.[^13] A settler that survived would found a city, walk
past the founding distance, and found again without limit.

### Left undone

- **The census row `settlements_founded` was not added.** The census table is
  held by another worker in this session, and the item forbade the edit. The
  count is readable today through the settlement count and through the
  controller log.
- **No balance row was written.** The register is held by another worker. Three
  rows need an edit: the default table row, now six rows by nine columns; the
  weight vector range, now five weights; and a new row for the settle group.
- The question of a row named `cities` beside `settlements` is unanswered. The
  code adds no second count.

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
[^11]: Backlog item 0497. `docs/backlog/complete/0497-build-a-per-site-production-queue-that-spends-a-person-and-goods.md`
[^12]: The founding survey, and the places a founding keeps its distance from. `crates/cachette-core/src/founding.rs`
[^13]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
