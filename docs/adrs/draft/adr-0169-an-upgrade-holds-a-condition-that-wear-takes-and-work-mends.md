# ADR-0169: An upgrade holds a condition that wear takes and work mends

## Context

A tile in this engine may carry one upgrade. An upgrade is a category with a
ground fit and a level, the category is a row of a shared table, and a build
order names the category while the engine resolves the row.[^1] The entry on a
tile holds the category, the level that stands there and the work done toward
the next level.

**Until this record, an upgrade had a level and a progress and no life.**
Nothing in the engine ever damaged or removed one. Every road, terrace, store
and wall that a faction ever built stood for the rest of the run, whatever the
weather did to it and whatever army walked over it. A faction that built early
kept every advantage of it for ever, and a faction that arrived later could
take the ground and never take the works on it.

The accepted record already anticipated a condition. It says that repair comes
before a raise, that the wear work gives an entry a condition, and that the
build order on a worn entry raises the condition first.[^2] It cites a backlog
item for the wear, because at the time no record held one. **A record that
points at a backlog item for a claim is pointing at something that closes.**

Four forces pull against each other.

**A world with no decay has no history.** A watcher who returns to a place
after a war should be able to read what happened there. An upgrade that never
falls carries no such reading.

**A repair must not become the only thing a worker ever does.** A worker whose
whole tick goes into maintenance builds nothing. If the price of a repair is
not tied to the size of the gap, light wear stops a build as surely as heavy
wear does, and a level under any wear at all can never rise again.

**A condition is stored, so it costs storage and it enters the state hash.**
Storage of an upgrade is sparse and it must not grow with the level.[^3] A new
field on the entry is a real cost at the target scale.

**Wear is a rate and a rate is not a chance.** A wear expressed as a draw needs
a key, and a key is one more thing to get wrong in the one property this
project cannot recover.[^4]

## Decision

### D1. An entry holds a condition on one scale that every category shares

**The engine stores one condition on each upgrade entry, beside the level and
the work done.** It is a whole number. A level that has just been finished
stands at the full condition.

**One scale serves every category.** The condition of a road and the condition
of a wonder are read against the same full value, and no row of the upgrade
table states a scale of its own. A per-category scale would be one fact in
several places, with nothing that fails when two of them disagree.[^5]

**The scale is large, and the size of it is a consequence of the repair price
and not a preference.** A repair buys a share of the full condition in
proportion to the work spent against the work that the level cost. A costly
category buys a small share for one unit of work, and on a small scale that
share truncates to nothing, so that category could never be mended at all. The
scale is therefore chosen so that the cheapest share the table can produce is
still above zero. The balance register holds the value.[^6]

The condition enters the state hash beside the level and the work done.[^7] The
invariant bounds it above nothing and at or below the full condition, so a
stored zero is a defect that the invariant catches.

### D2. Two causes take condition, both are rates, and neither reads the category

**The weather takes condition from an entry on wet ground, at a rate for each
tick. A hostile unit standing on the tile takes condition, at a rate for each
unit.** A unit is hostile when the faction holding the tile is at war with the
faction of the unit. An upgrade on ground that nobody holds wears from the
weather alone.

**Neither cause reads the category.** The upgrade row holds no column that
resists wear, so a wall and a road wear alike. A resistance column arrives with
the pass that reads it, and nothing reads one today.[^8]

**Both causes are rates, and neither is a chance.** The pass makes no random
draw, so it needs no key, and the whole question of keying a draw on the right
field does not arise.[^4] The pass is serial and it walks the sites in
ascending tile order.[^9]

The weather is read through the same reader that the gather resolve takes, so
the project holds one statement of whether a cell is wet.[^10] [^5] The wear
pass runs before the weather solve of the same step, so it reads the field that
the previous step left. That order is fixed and every tick reads the tick
before it.[^11]

The two rates are values of the downstream game and this record states
neither.[^6]

### D3. A repair is priced at the gap it closes, and a gap worth less than one unit of work is free

**A repair costs the work that the gap is worth, and never more.** The price
function and the gain function are inverses of each other and they truncate in
the same direction, so the engine holds one statement of what a repair costs
and one of what it buys. One function answers whether a repair is due, and
three callers ask it: the order verb, the build pass and the movement hold.
They therefore cannot disagree, and none of them can charge for a repair that
the pass will not do.

**A worker spends the price and puts the work above it into the level.** A
repair does not consume the whole tick.

**A gap worth less than one unit of work is priced at zero.** It is not mended
and it is not charged for. The gap stays open, it grows with the wear, and the
repair takes a unit of work as soon as it is worth one.

This clause changes the accepted record, which says that the work reaches the
next level only from full condition.[^2] **A level now rises below full
condition, and it must.** Under the other reading a site under any wear at all
spends every tick on maintenance, the wear takes the same amount back in the
same tick, and the level never rises again. The steady state is a frozen
progress and, over a long run, a collapse. That is not a balance defect that a
value can fix, because it holds at every wear rate above zero.

**Heavy wear still stops a build, which is the point of the mechanic. Light
wear no longer does.**

### D4. A damaged site resolves to the row that stands there, and a site at nothing is removed

**A build order on a damaged entry resolves to the row standing on the tile,
and not to the row above it.** Without this a worker on a damaged entry at the
top level is refused, because there is no higher row, and nothing could ever
mend one.

**An entry that reaches nothing is removed rather than stored at nothing.** The
tile returns to the ground the generator made, which is the same removal that
a destruction performs.[^12] Nothing else stores a property of an improved
tile, so dropping the entry is the whole of a collapse.

### D5. The build advance runs before the wear, and the collapse count is not state

**The order of the two passes is fixed: the build advance runs first, then the
wear.** The other order lets a site collapse in the tick a worker filled it,
and the worker would have bought nothing for its tick.

**A count of the collapses of one tick is a reading and never state.** It
describes one tick, it enters no state hash, and a caller that misses a tick
misses the count. It is a reading in the same way that the advance visit count
is.[^7]

## Consequences

**The project cannot treat a standing upgrade as a fact.** A pass that reads
what stands on a tile must accept that the entry may be gone next tick. A
design that needs a permanent work must express the permanence, and no column
does that today.

**A worker is now a maintainer as well as a builder.** A faction that builds
more than it can keep loses works it paid for. That is a real strategic cost,
and it is the cost the mechanic exists to create.

**The wear pass costs one walk over the sites on every tick.** The cost follows
the number of upgraded tiles and never the tile count of the world, because the
entries are stored sparsely.[^3] No measurement of it exists on the target
platform.[^13]

**Every golden state hash changed.** The condition is a new stored field, so
every world hashes differently from a world built before it.

**A record that pointed at a backlog item now points at a record.** The
accepted record cited an item for the wear because no record held it. The item
has landed, and the citation is repaired to this record rather than left
pointing at work that closed.

**Nothing here states a resistance.** A category that should wear more slowly
than another needs a column and a pass that reads it, and neither exists. A
value cannot express that difference, because neither rate reads the category
at all.

## References

[^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^2]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D3. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decisions D1 and D2. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^4]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^5]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^6]: Balance register, the upgrade rows. `docs/reference/balance.md`
[^7]: ADR-0164, every stored value the step reads enters the state hash, decisions D1 and D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^8]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: ADR-0143, wet ground yields more to a gatherer, decision D1. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
[^11]: ADR-0140, weather is a field over the weather cell lattice, decision D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^12]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
