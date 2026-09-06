# ADR-0158: A site builds a typed unit from a bounded queue its store pays for

## Context

A faction gets people in one way today: the seeding founds a group once, and
from that tick the population only falls. Every founded unit carries the worker
row, and the worker row holds an attack of zero, so no faction can raise a
cohort that kills and no faction can send a settler to found a second city. The
findings register holds that reading and its evidence.[^1]

The project owner has stated that domination and territory are the primary win
conditions of a run. Both need a faction to make units it chose. A settler
founds a new city, and a verb for that founding is already specified.[^2]
Nothing produces the settler. A soldier raises a cohort and a contest between
soldiers kills. Nothing produces the soldier either.

**A queue is the shape that fits the control plane.** Queueing a unit at a site
is a command over sites, not a loop over entities, so a caller states it once
and the engine resolves it.[^3] A controller reaches it through the same
verb.[^4] A learner reaches it through the action table, because the action
table derives its verb rows from the controller's choice enumeration.[^5]

**A separate record holds organic growth.** People arrive at a site from what it
holds and what it houses, at a rate, and the housing admits them.[^6] [^7] This
record holds the other half: a faction spends a person and goods, and gets one
unit of a type it chose. The two are different claims. A reviewer can accept a
queue and reject organic growth, or the reverse.

**The loop this record serves, end to end.** A worker gathers into the store of
a site. Food and housing grow the population of the site, and the housing bounds
it. The queue spends one person and goods to make a typed unit. A settler founds
a city, and a city adds reach, housing and a store. Soldiers make a cohort large
enough to raise, a campaign kills, and domination becomes reachable.

**People are the scarce middle of that loop, and not its output.** Every unit a
faction fields is a person it did not keep. Every measure of surplus spent on a
wonder or on wealth is a settler or a soldier not made. That competition is the
game, and wealth and wonder are late paths because of it. A separate work
stream raises those two bars against a long horizon, and this record states the
tension rather than the values.

**No code implements this record.** No site holds a queue, and no stage of the
step creates a unit. The decisions below state what the work must satisfy.

## Decision

### D1. A site holds a bounded, ordered queue of plain-data entries

A site holds a queue of entries. Each entry names a unit type and holds the work
done toward that type. The queue holds at most a stated number of entries, and
the bound is a property of the world parameters rather than of the population.

An entry is plain data with a declared layout and declared padding. It holds no
identity of a unit, because no unit exists until the entry finishes.

**The order is the order the entries were queued, and nothing reorders them.**
The front entry is the one that advances. An order that came from the collection
that happened to hold the entries would be stable, would pass both determinism
tests, and would still be unrecorded, so the next contributor who changed that
collection would move every identity downstream of a build.[^8]

The whole queue of every site enters the state hash. A queue that the hash did
not cover would let two different worlds hash the same and then diverge on the
next tick.

A reviewer finds a violation when the queue has no bound, when a bound follows
the population, when an entry holds undeclared padding, or when a pass sorts the
entries by anything but their position.

**This record states no bound value.** The balance register holds the row.[^9]

### D2. One verb queues an entry, and the engine holds no policy about what to queue

A caller queues an entry through one verb. The built-in controller queues
through that same verb and through no other path. No verb exists for the
controller alone.[^4]

**What a faction builds is the faction's decision, and the engine states none of
it.** The engine holds the mechanism, the bound and the refusals. It holds no
rule that a site at war builds soldiers, and no rule that a faction with room
builds settlers. This is the treatment the project already gives to a faction's
option weights: the built-in controller behaves sensibly, and a learner may
decide otherwise, and neither one is physics.[^10]

The built-in controller must therefore behave sensibly on its own account, and
that behaviour is policy that a later record or a balance row may change without
touching this one.

A reviewer finds a violation when a verb tests who is calling it, when the
engine queues an entry that no caller and no controller asked for, or when a
rule inside the engine names a unit type.

### D3. A queue is never free, and the store pays as the entry advances

The front entry advances by an amount each time the advance applies. The
advance charges the store of the site. A site whose store cannot pay makes no
progress, and its entry stays where it is.

**The charge is taken as the entry advances, not when it is queued.** A charge
at the moment of queueing would let a faction buy a place in a queue it never
means to finish, and it would make a cancelled entry a refund rule. A charge
that is taken per tick makes a stalled entry cost nothing more, and it makes a
site that is starving stop building without a rule that says so.

The work that finishes an entry is a value of the table that the world is built
with, indexed by the unit type. It is never a constant of the kernel, and no
pass compares a type index against a constant.[^11]

The accumulator is a whole number and it is clamped at the work the entry needs.
An unclamped accumulator would let a long build carry credit into the next
entry, and nothing would fail.

**This record states no work value, no charge and no schedule.** The balance
register holds those rows.[^9]

### D4. A finished entry consumes one person of the site and the goods it costs

An entry that reaches the work it needs is removed from the front of the queue.
It takes one resident of the site and the goods the entry costs. One unit of the
entry's type is then created at the site, through the same spawn path that every
other creation uses, and its residence is the site that built it.

**A queued unit is a person the site already held, given a type and a cost.** It
is not a new person. Organic growth is the only source of people, and the queue
is the only consumer of them.

The person the entry takes is a resident of the site, chosen by a stable key
over the residents and never by the order a collection held them.[^8] The
identity of the unit that leaves and the identity of the unit that arrives are
distinct, so no reader can confuse the two.

**A finished entry is refused when the site holds no spare person or cannot pay
the goods.** It is refused and not discarded: the entry stays at the front of the
queue. It costs nothing further, because D3 charges only an advance and the
entry no longer advances. The refusal is counted, and the two reasons are
counted apart, so a watcher reading a queue that never finishes can tell a site
with no people from a site with no goods.

**What this leaves for the population.** The queue never adds a person, so it
never passes the housing. Growth adds, the queue removes, and the housing bounds
the number of people that a site holds at once.[^7] There is one source, one
consumer and one bound, so no reader must ask which limit bit.

This is the one place where this record and the organic growth record meet.
Organic growth answers how many people a site has. This record answers what
those people become. Neither is the other.

### D5. The advance runs at one fixed stage, and its cost follows the sites

The advance opens one stage of the step. It runs on every tick, and a schedule
decides which ticks it acts on. The interval is a parameter of that schedule and
never a constant of the kernel.[^12]

**The stage visits the sites and the entries. It never searches, and it never
walks the units.** The cost of the stage therefore follows the number of sites
times the queue bound of D1, and it does not follow the population.[^13]

Two sites never write one another's state, so the disjointness is a property of
the partition rather than a rule a reviewer must check by reading.[^14] The
created units apply in one ordered scan, in site order and then in queue
position order, so the slot a new unit takes follows from the world and not from
a thread.[^8]

A reviewer finds a violation by asking one question of the stage. Does any loop
in it run once for each unit? If one does, the stage is wrong.

### D6. A refusal is stated at the verb and counted, and nothing is dropped in silence

The verb refuses an entry that names a type the world does not hold, an entry
for a site that does not stand, an entry for a faction that does not own the
site, and an entry that would pass the bound of D1. A refused entry changes
nothing.

The verb refuses at the moment of queueing. D4 refuses at the moment of
finishing, for a site that has no spare person or cannot pay. The two refusals
are counted apart, because they mean different things to a watcher and to a
learner.

A refused command is dropped and counted in the subsystem census, in the way
every refused controller command is.[^15] A watcher who reads a queue that never
moves must be able to see whether the verb refused the entries or the store
could not pay for them, and those are two different counts.

**A zero that a watcher cannot explain is the failure this project has recorded
twice.** A count of the fallen read zero because no unit could fight, and a
count of filled seats read zero because no seat existed. In both the number was
correct and the reason was invisible.[^1] The census therefore gains a row for
what the queue produced, so a zero there is visible beside the refusals that
explain it.

### D7. A settler is not redefined here

A settler is a unit whose type row holds a settle capability above zero. That
claim belongs to the record that holds the ground rule, and this record does not
restate it or change it.[^2]

The queue produces a settler in the way it produces any other type: an entry
names the type, the store pays, and one unit of that type arrives. What a
settler then does, and the verb that founds a city with one, are held by that
other record and by the backlog item that implements it.[^16]

## Consequences

**A faction can reach every win condition the owner named, once this and the
housing exist.** A settler comes off the queue and founds a second city. A
soldier comes off the queue, a cohort is raised, and a contest kills.

**The whole loop must run inside one game.** The project owner has set a game
horizon, and the balance register holds it as the tick limit row.[^9] A faction
founds with a very small group, and it must gather, grow, queue a settler, found
a second city, queue soldiers, raise a cohort and reach a decision, all inside
that horizon. **A work value or a charge that is correct in principle and too
slow for the horizon makes the game unreachable**, and nothing in this record
would find it. Every value of this record is therefore derived against the
horizon, and this record states none of them.

**Expansion and war now compete with gathering and with each other.** Every
person a site spends on a settler is a person that does not gather, and a person
spent on a soldier is not a settler. This is the intended tension and not a
defect to tune away.

**Wealth and wonder become late paths.** Every measure of surplus a faction puts
into a great work is a settler or a soldier it did not make. That is why those
two paths sit late, and a separate work stream raises their bars against a long
horizon.

**A faction that queues nothing accumulates people and does nothing.** The
engine states no policy, so a controller that never queues leaves every site
with an empty queue and a rising population for the whole run. The census row of
D6 is what makes that visible, and it is not a defect of this record.

**A faction that queues everything starves its gathering.** Every finished entry
takes a person, and a site whose people all became soldiers gathers nothing into
the store that feeds them. Nothing in the engine prevents it, and nothing
should: a controller that does it is playing badly, and a learner must be able
to learn that it is bad.

**Housing becomes a thing worth building, and the project has no housing
upgrade.** A site whose housing is full stops growing, so a faction that wants
more people must build more housing. The upgrade table holds categories for a
road, a terrace, a wonder, a store and a wall, and it holds none for housing.
**This record does not invent one.** A housing category is work, it belongs with
the record that says what an upgrade category is, and a backlog item names
it.[^18]

**The built-in controller becomes the place where sensible behaviour lives.**
Anything a reader would call strategy sits in the controller, where a learner
replaces it. Nothing about it may migrate into the verb.

**A test cannot prove D3 by watching a rich world.** A site whose store always
pays advances every tick, so the refusal never reaches the assertion. The
fixture needs a site with an empty store, a site with exactly the charge of one
advance, a full queue, a site whose store cannot pay the goods of a finished
entry, and a site that holds no spare person when an entry finishes. Put each
refusal back and watch the test stay green.[^17]

**Cancelling an entry is not decided here.** Nothing removes an entry from the
queue except finishing it. A caller that wants to cancel one asks for a verb,
and the work that adds it decides what happens to the work already charged.

## References

[^1]: Findings register, FND-486. `docs/FINDINGS.md`
[^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^3]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^4]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: ADR-0082, the store sets the rate of a birth and the housing admits it. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^7]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decisions D1 and D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^8]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: Balance register, the production queue. `docs/reference/balance.md`
[^10]: ADR-0156, a faction's option weights are policy, set through one verb, decisions D3 and D4. `docs/adrs/draft/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^11]: ADR-0120, a unit carries a type that indexes a table, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^12]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^13]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^14]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^15]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^16]: Backlog item 0485. `docs/backlog/proposed/0485-let-a-settler-found-a-city-and-let-the-controller-settle-new-ground.md`
[^17]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^18]: Backlog item 0498. `docs/backlog/proposed/0498-give-the-upgrade-table-a-housing-category.md`
