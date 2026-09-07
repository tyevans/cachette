# ADR-0180: A site changes hands or the taker destroys it, and what stands there follows the ground

## Context

A settlement is a stored row with a faction, a tile, a store, a housing
capacity, production rates and a build queue.[^1] Until now the founding wrote
the faction of a settlement, and nothing else ever wrote it. A settlement
therefore belonged to its founder for the whole run.

Ground already changes hands without a fight. A tile carries a lease, and the
lease follows the units that stand on the tile.[^2] A faction that walks the
same ground long enough takes the ground.[^3] A faction can therefore stand in
the middle of another faction's city and hold every tile around it, while the
city itself stays with the founder.

That state is not stable, and war has no object in it. A faction that beats
another faction in the field gains ground and gains nothing that produces. The
project owner asked for conquest.

One question held this record, and this record answers it. The project owner
held the question of whether an upgrade changes hands when the ground under it
does, and the register that held it now records the answer.[^4] The question is
reached on every captured city, because a developed city stands on ground full
of roads, terraces, lodging and walls.[^5]

Three forces fix the shape.

**A capture must not pause the city.** A city that stops producing when it
changes hands makes conquest worth less than razing. It also makes the taker
wait for a state that the engine already holds.

**A capture and a raze are two different acts.** One keeps the city and one
destroys it. A player must be able to choose, and the engine must not choose
for the player.

**Every balance value here is governed by an open question.** What a capture
costs and what a raze pays are rules of the downstream game. One blocker holds
every cost figure of this project.[^6]

## Decision

**A faction that holds a site tile against no defending unit besieges the
site, and the site falls when the siege has done the work the site resists. A
kept site keeps its identity, its store, its housing, its rates and its staff,
and every upgrade on the ground goes with the ground. A raze costs more work
than a capture. It destroys the site and moves its store to the razing
faction.**

### D1. A site's faction is writable, and a capture writes it and nothing else

The faction of a settlement is a stored value with two writers. The founding
writes the faction of a new settlement. One rule writes the faction of a
settlement that already stands.

A capture writes that value alone. The tile, the identity, the store, the
housing capacity, the production rates and the staff positions are untouched. A
taken city therefore produces on the tick after it changes hands, in the way it
produced before.

The identity of a settlement is a slot index and a generation, and a capture
advances neither.[^7] A handle that a caller stored before the capture still
resolves after it. The handle then names a site of another faction, and the
caller reads the faction to learn that.

A reviewer finds a violation when a capture clears the store, when it resets a
rate, when it allocates a new identity, or when it writes any field other than
the faction.

### D2. An upgrade changes hands with the ground

This decision answers the question the project owner held about an upgrade
whose ground changes hands.[^4] An upgrade changes hands with the ground. No
rule moves it, ends it or refunds it.

An upgrade is stored against a tile, and no owner is stored beside it.[^5] An
upgrade therefore already follows the ground. This decision makes that binding
rather than incidental, so that no later rule adds an owner to an upgrade and
calls the addition a repair.

A captured city keeps its roads, its terraces, its lodging and its walls. A
developed city is therefore a prize. A raze is therefore a real sacrifice,
because the razing faction destroys work that it could have used.

A reviewer finds a violation when an upgrade carries a faction of its own, when
a capture destroys an upgrade, or when a change of holder pays anything back to
the previous holder.

### D3. The trigger is occupation of the site tile with no defending unit standing, and the same trigger serves both acts

The trigger for both acts is the same. One faction has units on the site tile,
and the faction of the site has no unit on that tile.

**Neither act needs an order.** The ground decided that the site is taken, so
the step performs the taking wherever the trigger holds. A separate decision
says which of the two acts follows.[^12] A caller cannot refuse a taking and
cannot ask for one.

**A caller may still order a raze.** The order is not an act of its own. It
writes the intent of a siege that stands, and the siege then presses to the
raze work, which decision D11 states. It lets a caller burn a site that the step would have kept, so
a control plane keeps the choice that the step makes for a faction that has
none.

The capture runs after the pass that moves the lease of each tile, and before
the pass that spreads the ground out from the cities.[^2] The ground of a
captured city therefore follows the city on the tick the city changes hands.
This is a constraint on the order of the passes within one tick. It is not a
statement about where the code lives.

A reviewer finds a violation when a taking waits for an order, when the raze
verb accepts an order against a site that no siege of the ordering faction
stands against, or when the ground of a captured city follows on a later tick.

### D4. The occupier of a tile is stated once, and the capture reads the same statement the lease reads

Who stands on a tile is one statement. The faction with the most units on the
tile occupies it, and two factions with equal counts resolve by the lower
faction identifier.[^3] The engine builds that statement once for a tick. The
lease pass and the capture both read it.

Two factions that could take one site on one tick therefore resolve by a rule.
Neither the order in which a pass visits the units nor the order in which a
thread finishes decides the taker.[^8] A rule that took the first unit it found
would take the packing of the unit storage, and a packing is not a stable
key.[^9]

The defence is read separately from the units on the tile. The occupancy names
only the largest faction, so it cannot say whether the owner still has a unit
there. A garrison of one unit must refuse the capture. The capture therefore
asks whether any unit of the owning faction stands on the site tile.

A reviewer finds a violation when the capture builds a second occupancy of its
own, when the tie reads anything other than the faction identifier, or when a
site with a defender still changes hands.

### D5. A capture keeps the residents and clears the queue, and a raze destroys both and pays the store as plunder

A resident is a live unit whose home names the site.[^10]

**A capture moves every resident to the taker.** Each resident keeps the
character it carries, and the character moves with the unit. Conquest therefore
makes the taker larger, and the people of the city serve the faction that took
it.

**A capture clears the build queue.** A queue holds orders that the taker never
gave, so the taker does not inherit them.[^11] Work that stands on the ground
is an upgrade on a tile, and D2 leaves it untouched. Only the unbuilt orders go.

**A raze destroys the site, the upgrades on its ground and its residents.** The
store of the razed site moves to the nearest live site of the razing faction,
and a tie goes to the lower settlement slot. The plunder moves rather than
appearing, so nothing is made and nothing is lost. The account of every good
therefore holds across a raze, with no rule of its own. A razing faction with
no live site carries nothing away, and the store goes with the site.

A reviewer finds a violation when a capture leaves a resident with the previous
faction, when a character and the unit that carries it name different factions,
when a capture keeps a queue entry, or when a raze creates or destroys any
amount of a good.

### D6. A capture carries no lasting mark

A captured city is worth exactly what a founded city of the same size is worth.
No stored value records that a city changed hands, and no rule reads such a
value.

The project considered a mark, such as unrest that lowers production for a
time. The project refused it for three reasons. A mark is a subsystem of its
own, with its own state, its own decay and its own readers. Every value the
mark would need is a balance value, and one open blocker governs every cost
figure of this project.[^6] The choice between a capture and a raze is already
priced, because the plunder of the store stands against the buildings that a
raze destroys.

**The project cannot today make a captured city harder to hold than a founded
one.** A project that wants that must write a new record. This record states
the absence, so that a later reader sees a decision rather than an oversight.

A reviewer finds a violation when any stored value marks a city as captured, or
when a rule reads the previous faction of a site.

### D7. The taker keeps a site its own reach supplies, and burns one it does not

**A faction keeps a city it can hold and destroys one it cannot.** The engine
decides between the two acts, and it decides from a quantity it already
computes.

Every city reaches a distance out from its seat. That reach is what decides
which ground a faction holds, it grows with the upgrades the faction finishes
inside its own ground, and it stops at a bound.[^1] A captured site is supplied
when it stands inside the reach of a city the taker already holds. A supplied
site is kept. A site no city of the taker reaches is burned.

**The reach is derived and this decision states no distance of its own.** A
rule with a distance chosen here would be a balance value, and one blocker
holds every value of the downstream game.[^6] A rule that reads the reach binds
the choice to the ground the faction has actually built. A road between two
cities extends the reach, so the same conquest that burns today is kept once a
faction has built the ground between.

**A taker that holds no city keeps what it takes.** The rule asks which city of
the taker supplies the captured one. A faction with no city has not failed to
reach it, and the captured site is then the only city that faction has, so it
supplies itself. Without this clause the last army of a beaten faction could
never take a city, and a faction that lost every city could never return.

A reviewer finds a violation when the choice reads a distance that no pass
derives, when a site inside the reach of a city of the taker is burned, or when
a faction holding no city burns what it takes.

### D8. A site falls to work and never to a moment

**A siege is stored state, and it is work in the way a build is work.** The
site carries the faction that besieges it and the work that faction has done.
Both enter the state hash, because the step reads them.[^13]

The siege does one work for each unit of the besieging faction that stands on
the site tile, on each tick the trigger of D3 holds. One besieging unit
therefore does what one building unit does, so a siege reads against the same
work the upgrade table states and needs no scale of its own.[^16] A larger army
takes a city sooner.

**A site resists by the residents it holds.** The work a capture costs is the
resident count multiplied by a work for each resident, and it never falls below
that one work. A large city is therefore hard to take and a small one is not.
The resident count is a quantity the engine already keeps, so no second
statement of how strong a city is enters the project.[^10]

**The work for each resident is a balance value, and this record states none.**
The balance register holds the row, and one blocker governs every value of the
downstream game.[^6] [^14]

A reviewer finds a violation when a site changes hands on the tick a rival
arrives, when the siege state stays out of the state hash, when the work of a
tick reads anything other than the units of the besieging faction on the tile,
or when this record or the code states a work value of its own.

### D9. A raze costs more work than a capture

The work a raze costs is the work a capture costs, multiplied by a number above
one. A besieger that cannot supply the site therefore holds the tile for longer
than a besieger that can, and it holds it against every relief the owner sends
in that time.

**This is the answer to the act being too cheap.** A capture and a raze fired
on the same trigger and in the same tick, so razing a foreign city cost a
faction one unit and one step. The project owner ruled that the damage and the
time a raze costs must be far larger.[^17]

**The multiple is a balance value, and this record states none.** The register
holds the row.[^14]

A reviewer finds a violation when a raze costs the work of a capture or less,
and when this record or the code states a multiple of its own.

### D10. A siege stands only while its trigger holds, and its work is gone when it fails

The trigger of D3 is read again on every tick. The tick a unit of the owning
faction stands on the site tile, or the tick the besieger leaves it, the siege
ends and the work returns to zero. A besieger that comes back starts at
nothing.

**This is what makes a garrison and a relief force matter.** A besieged city
has two answers that cost it nothing new: keep a unit at home, or send one back
before the work is done. A raze that nothing could interrupt would be a delay
and not a defence.

**The work is gone rather than lowered.** A rule that lowered the work over
time would need a rate, and a rate is another balance value under the open
blocker.[^6] Ending the siege needs no value at all.

Two factions that could besiege one site resolve by the rule that decides the
occupier of a tile, which D4 states. The work of a faction that loses the tile
is gone by this decision, so the second faction starts at nothing and no result
reads an iteration order.[^3]

A reviewer finds a violation when a siege survives a defender on the site tile,
when it survives the besieger leaving, when the work carries across a change of
besieger, or when the work decays by a rate.

### D11. A caller states an intent against a siege, and it changes no cost

A caller orders a raze against a siege that stands. The order writes an intent
that the siege carries, and the siege then presses to the raze work whatever
the reach of D7 says. The order changes which act follows. It never changes
what the act costs.

**The order lasts as long as the siege.** A relief force that ends the siege
ends the order with it, by D10, and a besieger that comes back must order
again. An intent that outlived its siege would name a faction that is not
there.

**The engine reaches no order of its own.** Nothing inside the engine writes
the raze intent, so a seeded run with no caller uses the reach of D7 alone.
This record states the intent as a capability of the caller, and it does not
claim that the engine uses it.[^18]

A reviewer finds a violation when an order reaches a site that the ordering
faction does not besiege, when an order lowers the work a raze costs, or when
an intent survives the siege that carried it.

## The alternatives this rejects

**The upgrade stays with the faction that built it.** A faction would keep a
road inside another faction's ground. Rejected because an upgrade is stored
against a tile and carries no owner, so this option adds a faction column to
every upgrade.[^5] It also gives a city whose walls belong to its enemy, and no
rule of movement or combat could read that simply.

**The upgrade is destroyed when the ground changes hands.** Rejected because it
makes every capture a raze. A taker would gain a city with no roads and no
walls, and the choice of D3 would then have no content.

**The residents die on a capture.** Rejected because it removes the reason to
capture. A city with no people produces nothing and grows from nothing, so a
taker would always raze and take the store instead.

**Capture only, with no raze.** Rejected because a faction must be able to deny
a city to its rival. A faction that cannot hold a city it took needs a way to
stop the previous owner taking it back.

**A raze pays a fraction of the store rather than the whole of it.** Rejected
because a fraction is a balance value, and no measurement supports one.[^6]
Moving the whole store needs no value at all, and it conserves every good
exactly.

**An unrest mark or a disorder mark on a captured city.** Rejected under D6. It
is a subsystem of its own, every value in it is governed by an open blocker,
and the capture against the raze is already a priced choice.

**A capture stays instant and only a raze costs work.** Rejected under D8. A
city that opened its gates to one soldier standing in its square would still be
the act the project owner called too easy, and a taker would then always
capture and never besiege. A capture costs a siege, and the raze costs more.

**The walls on the ground of a site are what the site resists with.** Rejected
because nothing builds a wall. The plan solver zones a project from a column of
the upgrade table, and the wall row carries no column it reads, so no seeded run
ever finishes one. A resistance read from the walls would be a rule that no run
reaches.[^18]

**A defender lowers the siege work by a rate rather than ending the siege.**
Rejected under D10. The rate is a balance value that no measurement supports,
and ending the siege needs no value at all.

**The siege work decays when the besieger leaves.** Rejected for the same
reason. It also lets a faction take a city in short visits, which is the
opposite of holding ground.

## Consequences

**A faction can now be reduced to nothing.** No record before this one let a
faction lose its last city. A companion record states what happens to a faction
that holds no site and no unit.[^12]

**A captured site keeps its rates, so production does not pause.** A watcher
sees the colour of a city change, and sees its output continue.

**A war now takes time, and a faction must hold ground to win one.** A
conquest costs the besieger every tick it stands on the tile, and the owner may
end it with one unit. A run therefore ends later than it did, and it ends
holding more cities.

**The engine now chooses between the two acts.** A seeded run with no caller
both captures cities and burns them. Which of the two is common follows from
how far apart the cities of a world stand against the reach each one has, so a
change to either value changes how many cities a run ends with. A register
holds those values, and one blocker governs them.[^6] [^14]

**The store account stays exact.** Plunder moves between two stores, so the sum
of a good over the world does not change when a faction razes a site.

**Every golden state file that holds a site moves.** A second rule now writes
the faction of a site, the site carries a siege, and the state hash folds every
stored value that the step reads.[^13]

**A caller cannot burn a site the engine would keep without ordering it
first.** The order needs a siege to write, and a siege needs a tick. A caller
that stands on an undefended site and wants it gone therefore waits one step,
orders, and then waits for the work.

**The balance register gains no value from this record.** This record states no
capture cost, no raze cost and no fraction. Nothing here was measured, and one
blocker governs every cost figure of this project.[^6] The balance register is
where a later value belongs.[^14]

**One value is now stated in one place and read in two.** The engine builds the
occupancy of a tile once, and the lease pass and the capture both read it. A
second statement of it would be the defect shape this project already
records.[^15]

## References

[^1]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decisions D1 and D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^2]: ADR-0153, a tile's lease follows the units that stand on it, decisions D2, D5 and D7. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^3]: ADR-0153, a tile's lease follows the units that stand on it, decision D3. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^5]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: ADR-0014, entity identity is an index plus a generation, decisions D1 and D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decision D3. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^11]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decisions D1 and D3. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^12]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
[^13]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^14]: Balance register. `docs/reference/balance.md`
[^15]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^16]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^17]: Decisions register, DEC-279. `docs/DECISIONS.md`
[^18]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
