# ADR-0150: Held ground is the ground within reach of a city its faction owns, and upgrades extend the reach to a bound

## Context

A tile carries one holder, and the holder is one faction or nobody.[^1] The
rule that gives a tile its holder is a spread. A claim on a tile raises support
from the neighbours that already hold it and from the units that stand on it.
The ground sets the support a claim must raise, and a contested tile resolves
by a stable key.[^2] Two accepted decisions state that rule.[^2]

**The spread has no centre and no stop.** A holding starts wherever a unit
stands. It grows one ring each step, and a tile inside a holding never changes
hands, because its holder draws support from all six neighbours. The holding
stops only against water, high ground or another holding. At the target
population the held ground reaches a large share of the world, so a pass whose
cost follows the holding follows the world in practice.[^3]

The project owner asked for a different rule, and a product record states the
need.[^4] A faction's ground exists only around a city the faction owns. The
upgrades finished inside that ground extend how far it reaches, up to a bound.
A controller settles new ground. Outside a road, a unit builds only inside its
own faction's ground. Ground nobody holds is improved only by a settler or by a
road.

**This record supersedes the two decisions that state the spread.** The holder
column, the block masks, the running total and the exclusivity of the storage
are unchanged, and the decisions that state them stay in force.[^1] [^5]

Three forces fix the shape.

**A city is the settlement the engine already holds.** A settlement is one of
the four fixed entity shapes, it stands on one tile, and it carries a
faction.[^6] This record adds no second kind of place.

**The holder column is read by three things that need a column.** The state
hash folds it. The tile event stamps it. The land side of a contract writes
it.[^7] A rule that derived the holder on demand would leave each of those
without a value to read.

**Every value here is a game value.** How far a city reaches, how much one
upgrade extends it, and where the extension stops are rules of the downstream
game, and one blocker holds those rules.[^8] This record names the values and
states none of them.

## Decision

**A tile is held by the faction that owns the nearest city within reach of it.
Reach is a base plus a bounded extension that counts finished upgrades. The
holder column is rewritten from the cities each step. A build outside the
builder's own ground is refused unless it is a road. A settler founds a city
through one verb that the caller and the controller both use.**

### D1. A tile is held by the faction whose city is nearest within reach, and no tile is held with no city in reach

For each tile, the rule finds every city whose reach covers the tile. The
nearest city wins. Two cities at one distance resolve by the lower settlement
slot index. The tile is held by the faction of the winning city. A tile that no
city reaches is held by nobody.

Distance is the hex distance between the tile and the city's tile. A tile whose
ground admits no unit is held by nobody, whatever reaches it, so no faction
holds water. That consequence of the old rule survives.[^2]

The rule reads no previous holder and no unit position. A faction that owns no
city therefore holds nothing after the next step, and a unit standing on a tile
gives its faction no claim on it.

A reviewer finds a violation when a tile is held and no city of its holder
reaches it, when two cities at one distance resolve by anything other than the
slot index, or when a tile's holder depends on the tile's holder in the
previous step.

### D2. The reach of a city is a base plus an extension that counts finished upgrades, capped at a bound

The reach of a city is one whole number of hex steps. It is a base, plus one
extension step for each block of finished upgrades that stand on ground the
city held at the end of the previous step, and it never passes a bound. The
base, the upgrades that earn one step, and the bound are three balance
values.[^9] This record states none of them.

The count reads the holder column as the previous step left it, so the rule has
no recursion: the reach of this step depends on the ground of the last step,
and the ground of this step depends on the reach of this step. An upgrade under
construction counts for nothing.[^10] An upgrade of any kind counts, including
a road, when it stands on ground the city holds.

The extension is a whole number, and so is the reach.[^11] Two threads that
count the upgrades of one city in any order reach one sum.

A reviewer finds a violation when a reach is computed from a fraction, when an
upgrade outside the city's ground counts, when an unfinished upgrade counts, or
when a reach passes the bound.

### D3. The holder column is rewritten from the cities at the holding stage of every step

The holder column stays a stored column, and it stays inside the state hash.
The rule of D1 and D2 rewrites it once each step, at the stage where the spread
ran, after the barrier of the frame and before the tile event stamps the
holder.[^12] The rewrite computes the holder of every tile any city reaches,
compares it with the stored holder, and writes the tiles that differ through
the same apply path that the spread and the land transfer use. That path
repairs the running total, the block masks and the held list.[^5]

**This is the form the storage already has, and the presence relation's form
is rejected for it.** The presence relation is derived at the end of the step
and stored nowhere.[^13] The holder is different in one way that decides the
matter: three readers need a value that persists across the step. The state
hash folds the column. The tile event carries the holder as the frame left it.
The land side of a contract writes the column between two rewrites, and D6
states what the next rewrite does with that write.[^7]

The rewrite visits the tiles within reach of each city, and the tiles the held
list names, and nothing else. Its cost follows the cities multiplied by the
largest reach, plus the ground held, and never the world. Each thread decides a
contiguous run of tiles and writes its own slot, and the join reads the slots
in slot order.[^14] Nothing reads which thread finished first.

A reviewer finds a violation when the holder is computed on read, when the
rewrite reads the whole world, when a change reaches the column by a path other
than the apply, or when the stage runs before the barrier of the frame.

### D4. A build order is refused outside the builder's own held ground, unless the kind is a road

The pass that collects the build intents of a step reads the holder of the tile
each builder stands on. It drops an intent when the holder is not the builder's
faction, unless the kind of the build is a road. A road may be built on ground
nobody holds. The verb that gives a build order applies the same test at the
moment of the order and refuses the set, so a caller learns at once, and the
pass applies it at every step, so a build whose ground changed hands stops.

**The road is the one exception, and it is exempt from the ground test
wherever it is built.** The project owner stated it so. A road is how a faction
reaches ground it does not yet hold, and a settler walks it.

A build that stops because its ground changed hands leaves its progress in
the entry, and the entry stays.[^10] Whether that entry then belongs to the new
holder is the question one blocker holds, and this record does not answer
it.[^15]

A reviewer finds a violation when a unit finishes a build of any kind other
than a road on ground its faction does not hold, or when the build verb and the
build pass apply two different tests.

### D5. A settler is a unit whose type row has a settle capability above zero, and one verb founds a city for the caller and the controller

The unit type table gains one column, settle. It is a whole number, and zero
means the type cannot found a city.[^16] No flag stands beside it.

One verb founds a city from a set of units. For each unit in the set whose
settle column is above zero, the verb founds a settlement on the tile the unit
stands on, for the unit's faction. It refuses a unit whose column is zero, a
unit whose tile is held by any faction, a unit whose tile already carries a
settlement, and a tile that admits nobody. The founding distance rule of the
seeding applies to a settler's city as it applies to a founding.[^17] A refused
set changes nothing.

The controller founds a city through this verb and through no other path.[^18]
Whether it settles is one of its evaluations, drawn from the keyed generator
like every other choice it makes.[^19] A settle command the verb refuses is
dropped and counted.[^18]

The settler survives the founding. Whether founding costs the unit is a game
value, and a later record may change it without touching the verb.

The verb that founds a settlement at an address, with no unit, stays for the
seeding layer. It founds the first city of a faction before any settler exists.

A reviewer finds a violation when a pass compares a type index to a constant to
find a settler, when the controller founds a city through a path a Python
caller cannot call, or when a settler founds on held ground.

### D6. Traded land keeps the new holder only where D1 gives it

A land side of a contract writes the holder of every tile in the set to the
creditor when the other side delivers in full.[^7] That write stands until the
next rewrite. The next rewrite reads no previous holder, so a traded tile is
held by the creditor after the next step only when a city of the creditor is
the nearest city within reach of it. A traded tile outside the reach of every
city of the creditor is held by nobody after the next step. A traded tile
inside the reach of a nearer city of the debtor returns to the debtor.

**This is a consequence a reviewer must weigh against the traded land
need.**[^20] That need says a watcher reads the holder of a traded tile before
the deal and after it, and the two readings differ. Under this record they
differ for one step, and then the cities decide. The two records are
consistent, because the need states no duration. A reviewer who wants traded
land to hold must write a record that gives a traded tile a claim the cities do
not, and this record does not.

A reviewer finds a violation when the rewrite reads the previous holder to keep
a traded tile, or when a land contract writes the holder through a path other
than the apply.

## The alternatives this rejects

**Grow by presence, as today.** A claim raises support from the units standing
on a tile and from the neighbours that hold it. Rejected because a holding
has no centre, cannot be lost and cannot be chosen, and because its cost follows
the world at the target population.[^3] The product record states the four
costs.[^4]

**Grow by an influence field.** Each city radiates a field over the cells, and
a tile is held by the faction whose field leads there. Rejected because the
field lives on the level 1 lattice and a holder lives on a tile, so the border
would follow a block edge, which is the granularity this project already
refused for a fight.[^21] Rejected also because a field solve costs the
lattice on every step, and this rule costs the cities.

**Hold forever once claimed.** A tile that a faction once held stays held until
another faction takes it. Rejected because the owner asked that ground exist
only around a city, and a faction with no city would then hold ground for the
rest of the run.

**Derive the holder on read, like the presence relation.** Rejected under D3,
because three readers need a column that persists across the step.

## Consequences

**The territory win path counts only city-held ground.** The reader reads the
running total, and the running total counts tiles within reach of a city. A
faction that founds more cities scores more, and a faction that loses its
cities scores nothing.[^22]

**The spreading colour of the demonstration stops.** A watcher sees a disc
around each city that grows as upgrades finish and stops at the bound. The
edge between two factions is where two reaches meet.

**A faction with no city builds only roads.** Its units gather where they
stand, and every other build is refused. The refusal is counted where the
controller ordered it.[^18]

**Terrain shapes the ground in one way only.** Water is never held. The
ordering of the old rule, in which level ground was held before high ground, is
gone. The product record that asks for terrain to influence a holding is met
only by the water exclusion, and a reviewer of that record should know it.[^23]

**Every golden file that holds a held tile moves.** The rule that writes the
holder column changed, and the column is inside the hash.

**Two gates read a different ground.** The presence gate on a trade and the
ground gate on a storm both read the holder column, and both now read ground
that a city reaches.[^13] [^24] Neither record changes, because each reads the
column and states no rule for how the column is filled.

**Three balance rows and one capability column are new.** The base reach, the
upgrades that earn one step, the bound and the settle column arrive with the
work that implements this record. Nothing here was measured, and one blocker
governs every cost figure in this project.[^25]

## References

[^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decisions D5 and D6. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^3]: Findings register, FND-285. `docs/FINDINGS.md`
[^4]: PRD-0054, a god's ground is the ground around its cities. `docs/product/shaped/prd-0054-a-gods-ground-is-the-ground-around-its-cities.md`
[^5]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decisions D3 and D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^6]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^7]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
[^8]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^9]: Balance register. `docs/reference/balance.md`
[^10]: ADR-0090, a tile upgrade is stored sparsely, decision D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^11]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^12]: Findings register, FND-029 and FND-079. `docs/FINDINGS.md`
[^13]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D1 and D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^14]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^15]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^16]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^17]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^18]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decisions D2 and D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^19]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D4. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^20]: PRD-0051, a god trades land. `docs/product/accepted/prd-0051-a-god-trades-land.md`
[^21]: ADR-0121, a meeting between two factions resolves at the tile, decision D1. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^22]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^23]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^24]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
[^25]: Blockers register, BLK-007. `docs/BLOCKERS.md`
