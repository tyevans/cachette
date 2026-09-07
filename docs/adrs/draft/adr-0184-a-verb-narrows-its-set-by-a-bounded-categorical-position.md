# ADR-0184: A verb narrows the set it acts on by a bounded categorical position, and never by a list of identities

## Context

A faction acts through a bounded table of actions, and one action is one
integer over that table.[^1] A verb declares its own argument positions, and
the action integer is a mixed radix over them.[^2] The verb set of the table
is the choice enumeration of the built-in controller, plus one no-op row.[^3]

**Every verb that reaches units reaches all of them.** A gather order takes
the whole live unit set of the faction. So does a build order. A founding
takes every settler that stands on ground a city may take. A project order
takes every idle unit. No verb of the table names a part of the set.

A faction therefore does one thing at a time. It cannot send half its workers
to gather food and the other half to build a road. An audit of the learner
surface ranked this above every missing observation field. A missing field
bounds what a policy can perceive. This bounds what any policy can
express.[^4]

The built-in controller has the same limit, so this is not why the controller
wins. It is the ceiling on both.

### What makes the choice hard

The obvious answer is to let a caller name the units. That answer is closed.
The control plane builds a selector and sends one command, and it never loops
over entities.[^5] A list of identities inside an action would make the action
unbounded. An action is one integer over a table whose length is a function of
the world parameters.[^1]

The second answer is to let a verb name a place. That answer is open, and it
is where the bound is at risk. A place index is bounded by the world, so a
table that held one would grow with the world extent. The record that fixes
the table bounds it against the population and says nothing about the
extent.[^1] A reviewer therefore has no written constraint with which to
refuse such a position.

The third pressure is the cheaper algorithm. A set-valued command exists for
one reason. The engine may then choose an algorithm over the whole set, rather
than run a loop for each unit.[^5] A narrowing that forced a per-unit search
would give that reason away. It would buy expressiveness at the price of the
algorithm.

## Decision

### D1. A narrowing position is a bounded categorical selector

A verb may declare an argument position whose value narrows the set the verb
acts on. Such a position is a **narrowing position**.

The candidate list of a narrowing position is a fixed enumeration. Its bound
is a compile-time constant, and the schema states it in the way it states
every other bound.[^2]

A narrowing position is never a list of identities, never a count, and never a
fraction. A count is a quantity, and a quantity is a bucket position that only
a verb taking a quantity declares.[^6] A fraction names no particular unit.
The engine would have to choose which units it meant, and that choice would be
a rule that nothing states.

A reviewer finds a violation in three cases. A narrowing position carries an
identity. Its bound is not a constant. The engine resolves a narrowing by a
draw.

### D2. Index zero of every narrowing enumeration is the whole set

Every narrowing enumeration reserves its first value. That value names the set
the verb reaches when it declares no narrowing position.

This keeps three things true at once. The whole-set command stays one row of
the table, so the algorithm over the whole set stays reachable.[^5] The choice
enumeration of the controller maps onto the table unchanged. The verb set
therefore does not move, and the record that fixes it stands.[^3] A policy
that learns nothing about narrowing behaves as a policy did before the
position existed.

A reviewer finds a violation when an enumeration has no whole-set value. A
reviewer also finds one when a controller choice maps onto a narrowed row.

### D3. A narrowing position never names a place

No narrowing position carries a cell of the map lattice, a tile or a block. No
narrowing position carries any other index into the extent of the world.

**The length of the action table does not follow the world extent.** The
earlier record does not state this constraint. It bounds the table against the
population and leaves the extent open.[^1]

Two costs make the extent unacceptable. The legality answer returns one byte
for each row of the table, and it asks a question for each row. A table
indexed by the lattice would therefore make one decision cost a walk over the
world.[^7] A weight file is a function of the table length. A table that
followed the extent would part every policy from every world of another
size.[^8]

A faction that must act on a place acts through a verb whose content the
engine resolves. A campaign and a crossing already work that way.[^9]

A reviewer finds a violation when any bound of the action schema follows the
world extent. The width, the height, the tile count and the cell count are all
the extent.

### D4. The admissible narrowing kinds are the unit type and the assignment state

Two kinds of narrowing are admissible, and a third kind requires a new record.

**The unit type.** A unit type is an index into a table the world is built
with. A type parameterises a verb, and it does not multiply the verbs.[^10]
[^11] A narrowing by type is therefore the narrowing this project already has
a vocabulary for.

**The assignment state.** Whether a unit carries an order. The project verb
already reads this state to find the units it sends. The engine therefore
states the question, and this decision exposes it.[^9]

The engine reads both from one column of the unit arena. A narrowed verb
therefore filters inside the one pass it already makes. Neither adds a search.

A third kind narrows by ground. It names the ground a faction holds, or the
ground on which it observes a rival. This record does not admit it. Such a
narrowing is sound against D3, because it names no index. It is sound against
the fog rule, because a faction narrows by what it observes.

It is left out because it reads a second structure for each unit. Nobody has
measured what that costs at the target scale.[^12]

A reviewer finds a violation when a verb declares a narrowing position of a
kind this decision does not name.

### D5. A narrowing is not an observation

A narrowing position states what a faction wants. It never tells the faction
anything.

The legality answer already reports whether a verb could act, for each row of
the table.[^7] A narrowed row that no unit satisfies reads as not legal. That
tells a faction it holds no unit of that kind. That is a fact about its own
units, and a faction knows its own units.

A narrowing that named anything of another faction would leak, so no narrowing
kind names one. The two kinds of D4 read the faction's own units alone.

A reviewer finds a violation when the legality of a narrowed row depends on
anything the faction does not observe.

## The alternatives this rejects

**Let an action carry a list of unit identities.** Rejected because the action
is one integer over a bounded table, and a list has no bound.[^1] It would
also put the control plane in the data plane, which the boundary record
refuses.[^5]

**Add a separate selector verb that a later verb consumes.** Rejected because
an action would then carry state between two decisions. The log holds one row
for one action, with every field of it inside the integer.[^13] A selector
held between two actions is a field of an action that lives outside the
action.

**Give the narrowing a place index, bounded by the cell count.** Rejected
under D3. The bound is a world parameter rather than a population, so the
earlier record permits it. The costs are the legality walk and the parting of
every stored policy from every world of another size.

**Give the narrowing a fraction or a count.** Rejected because neither names
which units it means. The engine would choose. A choice made inside a verb is
a rule of the game that no record states. A choice made by a draw would also
put a draw where a caller expects a command.

**Add a new verb for each narrowed form.** Rejected because a type
parameterises a verb and does not multiply the verbs.[^11] It would also move
the verb set away from the controller's enumeration, which an accepted record
fixes.[^3]

**Leave the limit in place and widen the observation instead.** Rejected
because no observation field lets a faction do two things at once. The audit
that found this ranked it above every field for that reason.[^4]

## Consequences

**The table grows by a product and never by a sum.** A verb that declares two
narrowing positions holds the product of their bounds, multiplied by the
bounds it already had. Every factor is a constant. The table length therefore
stays independent of the population and of the extent.

**Every stored policy is parted from the table.** A narrowing position moves
every row above the verb it joins, so the action version moves with it.[^2] A
reader refuses a weight file whose stated fit is not the world it plays. The
move is therefore loud rather than silent.[^8]

**The legality answer costs more.** It asks one question for each row, and the
rows multiply. The cost stays bounded by constants, and no part of it follows
the world.

**A faction still cannot name a place.** The verbs whose content the engine
resolves are the only way to act on a place. This record does not change
them.[^9] A player who wants a road on a named tile has no verb for it.

**A narrowing by ground stays unavailable.** D4 leaves it out for a measured
reason that nobody has measured. A record that admits it must come with the
cost of reading the second structure for each unit.[^12]

**This record holds no figure.** Every bound is a fixed enumeration count that
the engine declares. The reference tables hold every value that a measurement
can change.[^12]

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decisions D2 and D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decisions D1 and D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^3]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decision D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^4]: Report 33, what a learner can see, say and be scored on, sections 4.9 and 7. `docs/research/reports/33-what-a-learner-can-see-and-say.md`
[^5]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^6]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decision D3. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^7]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^8]: Findings register, FND-635. `docs/FINDINGS.md`
[^9]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decision D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^10]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^11]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^12]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^13]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
