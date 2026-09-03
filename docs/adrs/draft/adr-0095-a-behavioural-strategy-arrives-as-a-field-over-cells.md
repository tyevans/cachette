# ADR-0095: A behavioural strategy arrives as a field over cells, never as a search from a unit

## Context

A unit in this engine chooses one option from a small fixed set. It scores
each option against the level 1 cell it stands in, and the engine writes the
winner into a column.[^1] A level 1 cell summarises one block of tiles.[^2]

**Every option the engine has is a gradient.** An option names one value that
the cell already summarises, and a unit prefers the cell that holds more of it.
There are four such options and no other kind.

That shape cannot express a destination. "Go back to the place you came from"
names one cell out of the whole lattice, and no summary field distinguishes it
from any other. A unit standing three cells away reads a summary that says
nothing about where its site is. The question a strategy asks is not "which
neighbour holds more" but "which way is that place", and nothing in the engine
answers it.

The next behaviour anybody asks for is of that second kind. Returning to a
site, retreating from danger and carrying a load to a store all name a place
rather than a quantity.

### The two shapes, and why the choice is not obvious

**The first shape gives each unit a search.** A unit knows its destination,
reads the cells between, and steps toward it. Every unit does this again. The
cost follows the population, and the work repeats for every unit that shares a
destination.

**The second shape gives each cell a direction.** The engine seeds the field at
the destination, spreads it over the lattice once, and a unit reads one entry.
The cost follows the cell count and does not change when the population grows.

A record already settles this for one step of movement, and its reasoning is
general even though its subject is not: the ranked quantity belongs to the
cell, so a per-unit search computes one order again for every unit that stands
in that cell.[^3] Its subject is one step to one neighbouring tile, for an
option the engine already computes. It does not govern a strategy that names a
place, and it says so by staying narrow.

### What does not exist

**Nothing in this record is implemented, and neither is the record it builds
on.** No field over cells exists in the simulation crate. Movement draws a
uniform direction from the counter-based generator and discards the option the
unit chose, which is the state a finding records.[^4] The record that would
change that is a draft, and the item that builds it is refined and not
started.[^3]

This record therefore describes no code. It exists because the shape of the
first strategy decides the shape of every strategy after it, and because the
search is what a contributor writes when nothing forbids it.

The registry retired one number for reaching further than this.[^5] That number
specified a portal graph and a flow tile cache before any path search existed
and before any record asked for a long path. This record states which of two
shapes a strategy takes and refuses to specify the shape it chooses.

## Decision

### D1. A strategy takes its direction from a field over cells, and never from a search that starts at a unit

A unit that holds a strategy reads one entry and steps. It does not read a
neighbouring cell, does not score a cell, and does not walk toward anything.

**The test is where the work is indexed.** A quantity indexed by the cell is a
field. A quantity computed from a unit's own position toward a unit's own
destination is a search, whatever it is called and however it is bounded.

This binds every strategy the engine gains, and not only the movement step that
a separate record governs.[^3]

### D2. The cost of a strategy follows the cell count and the strategy count, never the population

The engine derives one entry for each cell and each strategy. Adding units does
not add work to the derivation, and a unit's own cost is one read.

This is the checkable form of D1. A reviewer counts what the derivation is
indexed by. If the population appears in that count, the strategy is a search
wearing a field's name.

**No figure appears here.** No measurement exists on the target platform, and
every cost figure in this project is derived rather than measured.[^6] The
argument is about which term the cost follows, not what the term is worth.

### D3. A strategy that names a place is seeded at that place, and the field carries the direction outward

The destination is an input to the derivation, not a target a unit aims at. The
engine writes the seed into the cells that are the destination, spreads the
field over the lattice, and every unit of that strategy reads the result.

**Several destinations are one field, not one field for each.** A strategy that
sends every unit of a faction to its own nearest site seeds every site of that
faction at once. The field then carries the direction to the nearest seed
because the spread reached each cell from the nearest one, and no unit chose
which site it was aiming at.

This is what makes the shape affordable. One derivation serves a set of units
with a set of destinations, where a search would serve one unit and one
destination.

### D4. Whether a strategy field carries between frames is not settled here

A field derived from nothing at each rebuild reaches as far as its passes. A
field that carries from the last frame reaches further, and it states a value
that appears nowhere at level 0.

**That question is open, it is already open for another plane, and this record
does not answer it.** The register holds it, it blocks the acceptance of the
record that governs the influence solve, and a strategy field would be the
second plane to raise it.[^7] A separate row holds the choice as it applies
here.[^8]

A record that invented the answer would state a value an open question
governs.[^9] This record states the property instead: **a strategy field is
derived by a rule that a reader can state, at a moment a reader can name, and
it is the same at any thread count.**[^10]

## The alternatives this rejects

**Give each unit a search toward its destination.** This is what a contributor
writes, because the movement pass already reads one unit at a time and a
destination is a natural thing for a unit to carry. It is rejected on the cost
term in D2, and because the repetition is invisible at the call site: a reader
sees one unit reading a few cells and cannot see the unit beside it reading the
same ones.

**Give each unit a stored route.** The engine computes a path once and the unit
follows it. It is rejected because a route is state that the world invalidates:
ground changes, a site is destroyed, and a unit walks a path to a place that is
no longer there. A field derived again holds no stale route, because it holds
no route at all.

**Let a strategy name a target entity rather than a place.** A unit would carry
the identity of what it seeks. It is rejected because the direction then
depends on a lookup for each unit, which is D1's search under another name, and
because an identity that dies leaves the unit with a target that resolves to
nothing.

**Extend the option set instead, and let a gradient stand in for a
destination.** A summary field could be made to rise near a site, and a unit
would climb it. It is rejected because it is this record's field with the
derivation hidden inside the pyramid: a summary combines by addition and a
direction does not, which a record already states.[^11] It would also make
every strategy pay a summary field, whether any unit holds that strategy or
not.

## Consequences

**Every unit of one cell that holds one strategy goes one way.** The engine
cannot give two units of a cell two directions, because the mechanism that
would do it is the search D1 forbids. Whether that reads as a column or as a
crowd is a question only a run settles, and it is the same consequence the
movement record already carries.[^3]

**A strategy must name what its field is seeded from.** A strategy that names
no seed and no gradient cannot steer anything, and it is not a strategy under
this record.

**Adding a strategy costs an array.** The derivation is indexed by cell and by
strategy, so each strategy is a plane over the lattice. A project that wanted
many strategies would pay for each of them whether any unit held it or not, and
this record does not solve that.

**This record does not say what happens when a unit arrives.** Arrival is not a
concept the engine has: a unit holds its intent until it chooses again, and a
cell that no neighbour beats holds no direction.[^3] What a unit does at its
destination is the business of the verb it runs there, and no decision is
waiting to be made about it.

**This record does not say what starts a strategy.** A unit already takes a
strategy by scoring the fixed option set and taking the highest, and the record
that governs that scoring is accepted.[^1] A strategy is an option that steers,
and the trigger is the option winning.

**Nothing enforces this record.** No check looks for a search. A reviewer
applies D2 by reading what the derivation is indexed by, and a contributor who
writes a search gets no failure. A reader must treat this as unenforced, which
is the state the project keeps having to say out loud.[^12]

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: Findings register, FND-180. `docs/FINDINGS.md`
[^5]: ADR Registry, the retired numbers. `docs/adrs/REGISTRY.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^8]: Decisions register, DEC-095. `docs/DECISIONS.md`
[^9]: Decision Record Scope, section 4.5. `.claude/rules/adr-scope.md`
[^10]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^12]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
