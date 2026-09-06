# ADR-0157: A site's free places are its built housing less the residents the engine already counts

## Context

Nothing in this engine creates a unit while a world runs. The step opens a
stage for each pass it makes, and no stage spawns anybody. The seeding founds
a group once, and from that tick the population only falls.

The project owner has stated that domination and territory are the primary win
conditions of a run. Domination needs a faction to lose its units, and it needs
another faction to hold enough people to raise a cohort. A faction founds with
a small group, and a campaign raise asks for more people than a faction founds
with. Growth is the first link of that chain, and it is missing.

Two product records ask for it directly. One asks that the population respond
to what a faction holds.[^1] The other asks that a place hold a stated number
of people, that the number follow from what was built there, and that a watcher
read how full a place is without walking the population.[^2] A third asks that
a run start small and that the population at the end follow from the run.[^3]
A fourth makes a unit consume, so a new mouth is a cost.[^4]

**A rejected record already tried to answer this, and its premise was false.**
That record stated that nothing in the engine answers how many units a site
holds, and it asked the project to store a per-site occupancy count. The engine
answers the question today. Two reviews found it, and an open register row
holds the choice that follows, with a recommendation.[^5] [^6] This record does
not restate the false premise. It takes the answer the engine already gives and
states what growth may do with it.

**A record for the growth rule itself already exists.** It holds the rate, the
admission, the ordering and the draw key.[^7] This record holds what that one
rests on: where the free places come from, where a grown unit comes from, and
what the stage may cost. A section below names the boundary between the two.

**The loop this record serves, end to end.** A worker gathers into the store of
a site. Food and housing grow the population of the site. A per-site queue
spends one person and goods to make a typed unit.[^8] A settler founds a city,
and a city adds reach, housing and a store. Soldiers make a cohort large enough
to raise, a campaign kills, and domination becomes reachable. People are the
scarce middle of that loop, so growth is the first link and every later link
spends what it produces.

**No code implements this record.** No site holds a housing capacity, and no
stage of the step creates a unit. The decisions below state what the work must
satisfy.

## Decision

### D1. A site holds a stored housing capacity, and the ground never sets it

The housing capacity of a site is a stored field of the settlement shape. It
follows from what has been built at the site. It is not derived from the
terrain, and it is not derived from the number of tiles the site covers.

A capacity that the ground set would make housing a property of the map. The
recorded need is the opposite: what a faction builds is what limits it.[^2]

The capacity is a whole number, so a sum of capacities over many sites combines
to the same total in any order.[^9]

The settlement arena already holds a member named for a capacity, and it means
the ceiling on the slots the arena opens. Give the housing field another name.
One word with two meanings inside one shape is a defect that only a reader
catches, and nothing fails when a reader takes the wrong one.[^10]

**This record states no capacity value.** The balance register holds the
row.[^11]

### D2. Growth reads the resident count the engine already derives, and stores no second one

The number of units that live at a site is the sum of the cohort rows of that
site. The cohort table derives that count from the home column of the unit
arena, it rebuilds after a structural change, and both a row reader and a total
are public. A public check derives the table again and compares it against the
column.

**No pass stores a second resident count, and no pass maintains one by the
change.** The free places of a site are the capacity of D1 less this derived
count. A growth pass that kept its own count would put one fact in three
places, and a check between two copies does not guard three.[^10]

What the read needs is a reader that sums the rows of one site, because the
table splits the count by faction. That reader adds no store and no check.

This decision takes the first option of the open register row, which is the
option that row recommends.[^5]

**A derived count is not a free count.** The sum over the rows of one site
costs the faction ceiling, which is a structural constant of the project and
not a population. D4 bounds what the whole stage may cost.

**The housing is a hard bound, and it is the only bound on the population of a
site.** A site grows only while it houses fewer people than its housing allows.
A site at its housing bound grows nobody, however much food it holds. There is
no third behaviour between the two, so no reader must ask which limit bit.[^21]

The bound is not a preference and it is not a slowdown. A contributor who made
the housing scale the rate rather than stop it would give a site a behaviour
between growing and not growing, and the easy case is where a test looks.

### D3. The residence of a unit is the home column it already carries

One column names the site a unit belongs to. That column is the residence. The
engine adds no second column for it, and it stores no list of the residents of
a site.

The site a unit draws its ration from and the place a unit lives are one fact
today, because a settlement is the shape that holds both the store and the
housing. A unit that lives nowhere carries the value that means no home, and it
is still a unit that the world steps.[^2]

A caller that must reach every resident of one site passes over the units. The
household reader is that pass. It writes nothing and it holds no array of its
own. A reverse index would be a full duplicate of the column rather than a
summary of it, and no caller pays for one today.[^10]

**Separating the two facts is a later decision.** A unit that draws from one
site and lives in another is a world the project may want. Nothing needs it
today, and the work that needs it writes the record that splits the column.

### D4. Growth runs at one fixed stage, and its cost follows the sites

The growth pass opens one stage of the step. It runs on every tick, and a
schedule decides which ticks it acts on. The interval is a parameter of that
schedule and never a constant of the kernel.[^12]

**The stage visits the sites. It never searches, and it never walks the
units.** It reads the store and the housing of each site, and it reads the
resident count of D2 from the derived table. No part of the stage sorts the
population, tests a pair of units, or looks over the world for a place to put a
new unit.

The cost of the stage therefore follows the settlements and the faction
ceiling, and it does not follow the population. This is the rule the project
already applies: cost follows the lattice and the structure, never the number
of people.[^13]

A reviewer finds a violation by asking one question of the stage. Does any loop
in it run once for each unit? If one does, the stage is wrong.

**This record states no rate, no period and no phase.** The balance register
holds those rows.[^11]

### D5. Growth is the only source of people, and the queue is the only consumer

A birth creates a unit through the same call that every other creation uses.
The growth kernel writes no column of the unit arena directly, and it holds no
second creation path.

**Growth adds a person. The per-site production queue is the only thing that
takes one.**[^8] A queued entry that finishes consumes one resident of the site
and gives the world a unit of the type the entry named. It adds nobody, so it
never passes the housing bound of D1 and D2.

There is therefore one source of people, one consumer of them, and one bound
over how many a site holds at once. A reviewer who meets a second source, or a
second consumer, should refuse it, because either would need a second argument
about which bound applies.

**A grown unit carries the type the spawn path gives, and that is the worker
row.** The worker row holds an attack of zero, and a type row that holds zero in
a column means the type cannot do what the column names.[^14] The findings
register holds that reading and its evidence.[^15] This is correct rather than a
gap: a grown person is a worker, and the queue is how a faction turns a worker
into anything else.

A growth kernel that chose a type would hold a rule for a type, and a pass that
holds a rule for a type is what the type table exists to prevent.[^16] So the
growth kernel names no type at all, and the queue names one for each entry.

### D6. Every growth write is disjoint by site, and everything growth adds enters the state hash

Two sites never write one another's state in the growth stage. The stage
partitions its work by site, so the disjointness is a property of the partition
and not a rule that a reviewer must check by reading.[^17]

The housing capacity of D1 and every unit the stage creates enter the state
hash. A value that the step reads on every tick and the hash does not cover
lets two different worlds hash the same and then diverge on the next tick.

The derived resident count of D2 does not enter the hash on its own, because
the column it derives from is in the hash already. A summary hashed beside its
source would state one fact twice.

## What this record does not decide

The growth rule itself is held elsewhere, and this record adds nothing to
it.[^7] That record holds four things.

- The store sets a rate, and the rate proposes a birth.
- The housing admits the proposals as a bound, and it never scales the rate.
- A birth spawns a unit, and nothing adds to a headcount.
- Growth runs after the pass that ends a starved unit, and every draw is keyed
  on the system, the tick, the site and the index of the proposal within the
  site.

The bound on growth is therefore the housing, and it is a bound and not a
factor. A site with no free place grows not at all. A site with free places
grows at the rate its store sets. **Nothing in this record permits an unbounded
rate**, because D2 gives the free places their only source, and D1 makes that
source finite.

The default need rule, and what a grown population can survive under it, are
content. An open register row holds that choice.[^18]

## Consequences

**A watcher can read how full a place is before any growth exists.** D2 asks
for a reader over rows that already exist, so the product statement about
reading a place without walking the population becomes a small change.[^2]

**A crowded world is representable.** A population above the housing that holds
it is a state of the world and not a fault, in the same way that an over-full
tile is. What follows from crowding is not decided here.

**Losing a site costs a pass over the population.** Clearing the residence of
every unit that named a lost site reads every unit. A site is lost rarely, and
the project takes that cost rather than maintaining an index for it. That cost
sits outside the growth stage, so D4 is unaffected.

**The whole loop must run inside one game, and growth is its first link.** The
project owner has set a game horizon, and the balance register holds it as the
tick limit row.[^11] A faction founds with a very small group, and it must
gather, grow, queue a settler, found a second city, queue soldiers, raise a
cohort and reach a decision, all inside that horizon. **A birth rate that is
correct in principle and too slow for the horizon makes the game unreachable**,
and no test of this record would find it. The rate, the housing and the food a
birth costs are therefore derived against the horizon and not against taste.
This record states none of the four values.

**Growth alone reaches no win condition.** It makes workers, and a world of
workers cannot fight and cannot found a second city. Growth is the first link of
the loop and not the whole of it. The queue is what turns the people growth
makes into settlers and soldiers, and domination stays unreachable until both
exist.[^8]

**A faction that never raises its housing stops growing.** It stops for a reason
a watcher can read, and not by silently doing nothing. The census row that
counts the births reads zero, and the free places of every site read zero beside
it, so the two numbers together say why. A faction that wants more people must
build more housing or found another city.

**Housing therefore becomes a thing worth building, and the upgrade table holds
no housing category today.** It holds one that raises what a site can store, and
housing is the same shape. This record does not invent the category. A backlog
item names it, and it names the column it adds.[^22]

**A site at its bound that keeps gathering piles up food it cannot use, and this
record adds no mechanism for that surplus.** The surplus is ordinary stock. It
is still tradeable, it is still spendable on a production queue entry, and it is
still spendable on a great work.[^8] Nothing spoils it and nothing caps it
beyond the store the site holds.

That is the pressure the design intends. A faction with food and no room must
build housing, settle a new city, or trade the surplus away, and each of those
is a different path through the game. This record states the pressure as a
consequence. It states no rule that creates it.

**A test cannot prove D2 by watching a healthy world.** The derived count and
the column agree in every ordinary run. The test that proves the check works
makes the two disagree and asserts the refusal.[^19]

**A fixture built from the demonstration world measures the fixture.** Growth
lives at three extremes: a site at its housing, a site with an empty store, and
a site with one free place and more than one proposal in one frame. Build the
world that produces those, put each refusal back, and watch the test stay
green.[^19]

## References

[^1]: PRD-0011, a unit is born, holds a job and dies. `docs/product/accepted/prd-0011-a-unit-is-born-holds-a-job-and-dies.md`
[^2]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^3]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^4]: PRD-0013, a unit consumes to continue. `docs/product/accepted/prd-0013-a-unit-consumes-to-continue.md`
[^5]: Decisions register, DEC-057. `docs/DECISIONS.md`
[^6]: Review 0199, the influence, tile field, upgrade and housing records, section 4. `docs/reviews/0199-the-influence-tile-field-upgrade-and-housing-records.md`
[^7]: ADR-0082, the store sets the rate of a birth and the housing admits it. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^8]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for. `docs/adrs/draft/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^9]: ADR-0023, an aggregate combines exactly in any order, decision D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^10]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^11]: Balance register, the population. `docs/reference/balance.md`
[^12]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^13]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^14]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^15]: Findings register, FND-486. `docs/FINDINGS.md`
[^16]: ADR-0120, a unit carries a type that indexes a table, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^17]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^18]: Decisions register, DEC-044. `docs/DECISIONS.md`
[^19]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^21]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^22]: Backlog item 0498. `docs/backlog/proposed/0498-give-the-upgrade-table-a-housing-category.md`
