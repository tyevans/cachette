---
id: 0187
title: Give a carried load somewhere to go
status: refined
created: 2026-09-02
implements: [ADR-0002 D1, ADR-0004 D1, ADR-0004 D3, ADR-0004 D4, ADR-0022 D2, ADR-0023 D1, ADR-0023 D2, ADR-0062 D3, ADR-0062 D5, ADR-0063 D5, ADR-0072 D5, ADR-0073 D2, ADR-0073 D4]
changes: []
creates: []
serves: [PRD-0007, PRD-0013]
blocked-by: []
---

## Why

**The resource loop has no sink.** A unit gathers into a carry column. No verb
anywhere moves a carry load into the store of a site. A load leaves the world
when the unit dies, and the world counts it as departed so that the ledger
balances.[^1]

Gathering therefore cannot feed anybody. The store of a site rises only by the
fixed rate that the founding set from the survey, so the economy is a constant
and the ground the units stand on does not change it.[^2]

**Take item 0186 first.** No unit gathers today, so nothing carries a load and
this item would ship a verb with nothing to move.[^3]

## What the work does

1. A unit that stands on the tile of its home site gives its carry load to the
   store of that site.
2. The ledger balances across the transfer. What leaves the carry columns
   equals what reaches the stores.

After this, what a settlement holds depends on what its people fetched. The
chain from the ground, to the store, to the ration, to the death of a unit is
then closed at both ends.

## The answers this item takes, stated plainly

**The transfer runs after the gather resolve and before the rate pass.** It
reads where each unit stands, so it runs after the barrier that the movement
of this frame passed.[^4] It moves a quantity, and two records state that the
rate pass and the consumption pass run after every stage that moves a
quantity.[^5] [^6] It runs before the rebuild of level 1, because level 1 is
derived and it is last.[^7] It changes no structure, so it is not a barrier
and it needs none.

**A delivery is admitted by sort, then by transfer.** Two units of one site
deliver into one store, and the store saturates at the top.[^8] A saturating
add is not order-free, so the transfer sorts the deliveries by the site and
then by the identity of the unit, and it transfers in that order. This is the
shape the gather resolve already uses against a deposit.[^9]

**A load the store cannot hold stays in the carry.** The store saturates, and
a quantity that vanishes without a record breaks the conservation
equality.[^8] The unit therefore keeps what did not fit, and it delivers the
remainder on a later tick. Nothing is created and nothing is lost.

**Food reaches the one commodity that a store holds.** The map from a resource
kind to a commodity is declared in one place. Wood and stone answer no
commodity today, so a unit keeps them. The engine already writes the number of
that commodity at each site that needs one, and this item must not add a
third.[^10] The item that gives a kind of work its commodity absorbs the
map.[^11]

**The founding keeps its fixed production rate, and this item does not change
it.** The survey reads the ground of the disc, and units gather from the same
ground, so the same food can reach the store twice. The register holds that
choice open with a recommendation, and the recommendation is to keep the rate
and to restate it as the yield the site works without anybody walking to
it.[^12] A change to the rate would change the survival of every site against
no evidence.

**The carry ledger gains one term.** The conservation check compares what left
the tiles, plus what recovery returned, against what the units hold plus what
departed with the dead.[^13] A delivery moves a quantity out of that account
and into the store account, which a second check compares against the sum of
the stores. The delivered total is the term that links them, and both checks
stay exact.

## Impact review

**Governed by.**

- **ADR-0072 D5.** Conservation is a world invariant, checked for each kind.
  This item adds the delivered term to that invariant, in the same change as
  the transfer.
- **ADR-0073 D2 and D4.** The gather intents sort by the deposit and then by
  the identity of the unit, and what a unit carries is a column of the unit.
  The delivery reuses the sort shape and reads that column.
- **ADR-0062 D3 and D5.** The store saturates and no refusal is silent, and
  the rate pass runs after every stage that moves a quantity. This item runs
  before the rate pass and records what did not fit by leaving it in the
  carry.
- **ADR-0063 D5.** The consumption pass runs after every stage that moves a
  quantity. The delivery is such a stage, so it runs before consumption.
- **ADR-0023 D1 and D2.** An aggregate combines exactly, in any order. The sum
  of one site's deliveries is exact integer arithmetic and it is order-free
  until it reaches the ceiling of the store.
- **ADR-0004 D1, D3 and D4.** Iteration order is explicit, a reduction that is
  not order-free writes into a slot indexed by a stable key, and a sort uses a
  stable key. The saturating add at the store is that reduction, and the sort
  by the site and the identity is that key.
- **ADR-0022 D2.** Every level above level 0 is a pure function of level 0.
  The transfer writes level 0 only, and level 1 rebuilds after it.
- **ADR-0002 D1.** No floating point. The carry is an exact whole number and a
  store quantity is a fixed-point value, and the conversion between them is
  exact or it does not happen.

**Changes.** No record changes. This item contradicts no accepted record.

**Creates.** No record. The determinism question is real, and an accepted
record already answers it: a reduction that is not order-free writes into a
slot indexed by a stable key.[^14] There is one workable option once that rule
is applied, so there is no decision left to record.[^15] The double count
between the founding rate and the delivery is a decision, and it sits in the
register with options and a recommendation.[^12]

**Blockers.** None. BLK-007 governs any cost figure, and this item states
none.[^16]

**Precedent.** FND-191 records that the engine writes the number of the food
commodity wherever it needs one, and that the map a backlog item holds is read
by nothing.[^10] FND-043 records that zero is a real state for a store, so a
site that holds nothing of a commodity is not a site that holds no store.[^17]

## Done when

- A unit that stands on the tile of its home site and holds a load delivers
  that load to the store of the site, on the path the step already runs.
- The test drives the step. It does not call the transfer directly.[^18]
- The conservation check balances for each kind across the transfer, and the
  store check balances against the sum of the stores. A test drives several
  ticks of gathering and delivery and asserts both.
- A unit that stands on a tile that is not its home site delivers nothing.
- A unit with no home site delivers nothing.
- A store that cannot hold the whole load takes what it can, and the unit
  keeps the remainder. A test builds a full store and asserts that the carry
  falls by exactly what the store rose by.
- Two units of one site delivering on one tick produce one answer, and the
  answer does not depend on which unit the arena holds first. A test asserts
  it at 1, 2 and 12 threads.
- A stored quantity that the fixed-point scale cannot hold exactly is not
  transferred. No conversion rounds a quantity into or out of existence.
- **A test changes the carry of one unit and asserts that the store of its
  site changes.** The register recommends this test for every value that the
  work writes into state.[^19]
- The fixture is built for this test and is not copied from the demonstration
  world. It holds a site with a nearly full store, a site with an empty store,
  a unit at home and a unit away from home. The commit body says how that was
  checked: the transfer was removed, and each delivery test was watched to
  fail.[^18]
- Neither the founding rate nor the survey changes in this item.
- The two determinism tests pass, and `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: What a unit does in a tick, section 3.5. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^3]: Backlog item 0186, let the engine order a gather. `docs/backlog/refined/0186-let-the-engine-order-a-gather.md`
[^4]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^5]: ADR-0062, production and upkeep are rates attached to a site, decision D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^6]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D5. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^7]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^8]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^9]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D2. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^10]: Findings register, FND-191. `docs/FINDINGS.md`
[^11]: Backlog item 0181, give a kind of work the commodity it fills. `docs/backlog/proposed/0181-give-a-kind-of-work-the-commodity-it-fills.md`
[^12]: Decisions register, DEC-080. `docs/DECISIONS.md`
[^13]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^14]: ADR-0004, iteration order is explicit, decision D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^15]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^16]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^17]: Findings register, FND-043. `docs/FINDINGS.md`
[^18]: Testing Rules, sections 2a and 5. `.claude/rules/testing.md`
[^19]: Decisions register, DEC-074. `docs/DECISIONS.md`
