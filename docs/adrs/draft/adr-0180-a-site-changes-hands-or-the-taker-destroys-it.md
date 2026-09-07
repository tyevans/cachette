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

**A faction that occupies a site tile with no defending unit on it takes the
site. The site keeps its identity, its store, its housing, its rates and its
staff. Every upgrade on the ground goes with the ground. A caller may order a
raze instead, which destroys the site and moves its store to the razing
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

### D3. The trigger is occupation of the site tile with no defending unit standing, and a raze is an order while a capture is not

The trigger for both acts is the same. One faction has units on the site tile,
and the faction of the site has no unit on that tile.

**A capture needs no order.** The ground decided it, so the step performs the
capture wherever the trigger holds. A caller cannot refuse a capture and cannot
ask for one.

**A raze needs an order.** A raze destroys what a capture keeps, so a caller
must ask for it. The raze verb reads the same trigger, and it refuses when the
trigger does not hold. **Nothing inside the engine orders a raze.** The engine
razes no site on its own, and a caller is the only source of a raze order.

The capture runs after the pass that moves the lease of each tile, and before
the pass that spreads the ground out from the cities.[^2] The ground of a
captured city therefore follows the city on the tick the city changes hands.
This is a constraint on the order of the passes within one tick. It is not a
statement about where the code lives.

A reviewer finds a violation when a capture waits for an order, when a raze
fires with no order, when the raze verb accepts an order on a defended site, or
when the ground of a captured city follows on a later tick.

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

## Consequences

**A faction can now be reduced to nothing.** No record before this one let a
faction lose its last city. A companion record states what happens to a faction
that holds no site and no unit.[^12]

**A captured site keeps its rates, so production does not pause.** A watcher
sees the colour of a city change, and sees its output continue.

**The engine razes nothing on its own.** A seeded run with no caller therefore
captures cities and never razes one. A test of the raze path must issue the
order, because no engine pass issues it.

**The store account stays exact.** Plunder moves between two stores, so the sum
of a good over the world does not change when a faction razes a site.

**Every golden state file that holds a site moves.** A second rule now writes
the faction of a site, and the state hash folds every stored value that the
step reads.[^13]

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
