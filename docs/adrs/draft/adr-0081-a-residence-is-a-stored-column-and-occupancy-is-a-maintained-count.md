# ADR-0081: A residence is a stored column and occupancy is a maintained count

## Context

A settlement is fixed to a tile and holds the pooled store of the units that
belong to it.[^1] A unit already carries the slot of the site it draws
from.[^2] Nothing states how many units a site can hold, and nothing states
how the engine answers how many it holds now.

A product record states the need. A place holds a stated number of people, a
unit lives somewhere, and a watcher reads how full a place is without walking
the population.[^3] That record states the need and states no structure, so
the structure is decided here.

**A reader who knows the tile answer will expect the opposite answer here.** An
accepted record rejects a dense per-tile occupancy count, and it rejects it for
good reasons: nothing would call it, and it would be a second declaration of
where units stand.[^4] A reader who applies that reasoning to housing concludes
that occupancy must be derived too.

The two are not the same question. Tile occupancy is a frame-local fact over
the tiles that some intent named, and the frame discards it. A residence is a
stored fact with a lifetime: a unit lives in one place across many frames, and
a watcher asks how full a place is between frames as well as inside one. This
record says why the answers differ, so that a later reader does not have to
reconcile them alone.

**No code implements this record.** The engine holds the residence column. It
holds neither a capacity nor an occupancy count. The decisions below state what
the work must satisfy.[^5]

## Decision

### D1. A site stores a housing capacity, and the ground does not set it

The capacity of a site is a stored field of the settlement shape. It follows
from what has been built at the site. It is not derived from the terrain, and
it is not derived from the number of tiles the site covers.

A capacity that the ground set would make housing a property of the map. The
recorded need is the opposite: building is the thing that limits a faction.[^3]

The capacity is an exact integer, so a sum of capacities over many sites
combines to the same total in any order.[^6]

### D2. The residence of a unit is the site column the unit already carries

The engine holds one column that names the site a unit belongs to. That column
is the residence. The engine adds no second column for it.

One fact in two places, with nothing that fails when the copies disagree, is
the failure this project meets most often.[^7] The site a unit draws from and
the place a unit lives are one fact today, because a settlement is the shape
that holds both the store and the capacity.

A unit that lives nowhere is representable, and it is still a unit.[^3] The
column already carries that answer.

**Separating the two facts is a later decision.** A unit that draws from one
site and lives in another is a world the project may want. Nothing needs it
today, and the work that needs it writes the record that splits the column.

### D3. A site holds an occupancy count, and a check that can fail guards it

The engine maintains a count of the residents of each site. The change that
assigns a residence and the change that clears one both update it.

**No pass over the units recomputes the count during a running frame.** That is
the prohibition, and it is what makes the count worth storing. Two passes are
permitted and neither is the maintenance path. A world that is restored rather
than run has no history of changes to have maintained, so it derives the count
from the residence column once, before the first frame. The check that guards
the count also walks the units, because a check that read the same stored number
it is checking would pass by construction.[^15]

This is a second declaration of a fact that the residence column already holds.
The project accepts it here for two reasons that the tile case does not
have.[^4]

**A caller asks for it, and asks often.** A watcher reads how full a place
is, and the admission of a birth reads the same number.[^3] The tile count was
rejected partly because nothing would call it.

**The fact has a lifetime, so a derived answer is not free.** The unit-to-tile
structure rebuilds each frame for a reason of its own, and tile occupancy rides
on that rebuild.[^8] A residence changes rarely and is read between frames, so
a rebuild would pay the whole population for a number that almost never moved.

The price of the second site is a check that fails when the two disagree, and
that check needs a test that proves it can fail.[^9]

### D4. The engine holds no reverse index from a site to its residents

Nothing stores the residents of a site as a list. A caller that must reach
every resident of one site passes over the units.

Such an index would be a full duplicate of the residence column rather than a
summary of it, and one rare caller wants it: the loss of a site. The count of
D3 answers the frequent question, and this record declines to store the rest of
the answer before a caller exists that pays for it.[^7] [^10]

**A later caller may need the index.** A read of who lives under one roof is
the candidate. That work states its own caller and writes its own record.

## Consequences

**Losing a site costs a pass over the population.** Clearing every residence of
a lost site reads every unit. A site is lost rarely, and the project takes that
cost rather than maintaining an index for it.

**A crowded world is representable.** A population above the capacity of the
places that hold it is a state of the world and not a fault, in the same way
that an over-full tile is.[^11] What follows from crowding is not decided here.

**A test cannot prove D3 by watching a healthy world.** The count and the
column agree in every ordinary run, so a test that reads both proves nothing
until the invariant check is made to fail. The test that proves the check works
removes the maintenance and asserts the failure.[^9]

**Assignment and eviction need a stable key.** Both are set-valued, both run in
parallel, and neither may take its order from thread completion.[^12]

**A fifth column set is not opened.** A dwelling is not a new entity shape. The
capacity sits on the settlement, and the shapes do not vary at run time.[^13]

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^3]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^4]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D3. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^5]: Findings register, FND-116. `docs/FINDINGS.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^9]: Testing Rules, a determinism test must be able to fail. `.claude/rules/testing.md`
[^10]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^11]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^12]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^13]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^15]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
