# ADR-0053: A faction is a bit in a mask, and a relation is a plane

## Context

A faction is a side in the simulated world. A unit belongs to one. A tile is
held by one, or by nobody. The engine must answer three kinds of question
about a faction, and the three pull in different directions.

The first question is about one subject. Who holds this tile? Which faction
does this unit belong to? An answer needs one field on the subject.

The second question is about a set. Does any faction hold ground in this
region? Does any faction that I am at war with see me? An answer needs a set
of factions, and it needs the set to be cheap to test.

The third question is about one faction over the whole world. How much ground
does this faction hold, and where? An answer must not read the world, because
the world is large and the holding of one faction is usually small.

The obvious storage for the third question is one plane for each faction: an
array over the tiles, for each faction, that says whether that faction holds
the tile. It answers every question directly. It also multiplies the size of
the world by the number of factions, and it makes exclusivity a rule that the
engine must maintain rather than a property of the storage. Two planes can
both say yes about one tile, and nothing in the representation stops them.

The faction dimension is not local to one subsystem. Ground held, ground seen,
ground explored, military presence and diplomacy all carry it. A shape chosen
for one of them is the shape every later one inherits, because a value type
that two subsystems share cannot be widened in one of them.

The width is also not retrofittable. A width change moves the layout of every
event that carries a faction set, and every event type is plain data whose
bytes a determinism test compares. It therefore invalidates every golden state
hash and every recorded replay.[^1] The research report reaches the same
conclusion from the fog of war side and asks for the decision before the first
golden file exists.[^2]

The cost shape is settled and it is not the tile count. The measured shape is
that the term which grows with the number of things dominates the term that
grows with the number of tiles.[^3] Every figure in this project is derived
rather than measured, because no measurement exists on the target
platform.[^4]

## Decision

### D1. A faction identifier is a bit index into a 64-bit mask

**A faction is one bit of a 64-bit word.** The addressable set is the faction
ceiling in the scale constants table, and the top bit is reserved.[^5] The top
bit names every faction outside the addressable set at once.

The reserved bit converts a hard limit into a soft one. A question that asks
whether anybody in a set holds, sees or threatens the subject keeps working
when a faction outside the addressable set arrives. A question that needs the
identity of that faction must fall through to a side table, and the interface
must say so.[^2]

The width is 64 because the target holds a 64-bit word in one register, and
because the population count of one word is one instruction there. A wider
mask costs two of each and buys nothing that the reserved bit does not.

### D2. A subject carries one holder, and exclusivity is a property of the
storage

**A tile carries one holder field, whose value names a faction or nobody.** No
tile field is indexed by the faction.

Exclusivity then needs no rule. One field holds one value, so a tile cannot
name two factions, and no code path can make it. A representation that can
express the violation needs a rule to forbid it, and a rule needs a check, and
a check that nobody runs is how the violation arrives.

This binds every later subject that a faction can hold. A structure, a
deposit and a route each carry one holder field for the same reason.

### D3. A set of factions is one mask, and no field of the world is indexed by
the faction

**Where a set of factions must be stored, the engine stores one mask.** It
never stores one array over the world for each faction.

A mask costs one word whatever the number of factions in it. A summary over a
block of tiles therefore carries a mask, and a query that asks where a faction
holds reads the masks, passes over every block whose mask does not name the
faction, and walks only the blocks that do.

The union of two masks is the bitwise or. It is associative, commutative and
exact, so a fold over a group of masks gives one answer whatever the grouping
and whatever the order.[^6]

This is the decision that a future contributor could reasonably reverse. A
plane for each faction answers every query without a fall-through, and it is
simpler to write. It is rejected because its size is the size of the world
multiplied by the number of factions, and because it makes D2 unenforceable.

### D4. What a faction holds is a running total, and the rule that changes a
holder maintains it

**The engine holds one count for each faction, and the rule that changes a
holder adjusts the count of the faction that gained and the count of the
faction that lost.** A caller that asks what a faction holds reads the count.
It does not read the world.

The count is a 64-bit signed accumulator, because a one-tile contribution
summed over the tile count of the target world overflows a 32-bit
accumulator.[^6]

A summary level carries the held ground as an extensive field, which is the
count of tiles whose holder is a faction rather than nobody.[^7] That field
says how much ground is held, and the mask of D3 says by whom. Neither field
is indexed by the faction, so neither grows with the faction count.

The running total is a second declaration of a fact that the holder column
already states. The invariant check therefore derives the total from the
column and compares.[^8]

### D5. The ground decides what a claim on a tile must raise

**The rule that spreads a holding reads the terrain, and the terrain sets the
support that a claim must raise.** Ground that admits no unit admits no
holder, so no faction ever holds open water. Ground that rises asks for more
support than level ground.

A rule that ignored the ground would spread a holding as a disc, and a
boundary would then say nothing about the world it sits in. The purpose of a
holding is that a place belongs to somebody, and a place that is
indistinguishable from every other place is not a place.

The support values are a property of the rule and not a measurement. They are
ordered, and the order is the part that binds: level ground is easier to hold
than high ground, and water cannot be held at all.

### D6. A contested tile resolves by a stable key

**Where two factions claim one tile, the winner is fixed by a sort key and
never by which thread decided first.** The key is the support of the claim in
descending order, then the faction identifier in ascending order.

The rule decides every tile against the holders of the previous tick. It
therefore does not depend on the order in which the tiles were visited, and it
gives one answer at any thread count.[^9]

### D7. A relation between two factions is one mask row for each faction, and
never a field of the world

**When this project stores a relation between factions, it stores one mask for
each faction.** Alliance, war and shared vision are each one such plane. A
relation is never stored as a field over the tiles.

A relation plane is the faction count squared in bits, which stays in the data
cache of one core for a whole tick. A relation expressed over the world is the
world multiplied by the faction count squared, which is not storable.

The plane expresses a relation that is neither symmetric nor transitive. One
faction may grant vision to a second without receiving it, and the second may
not pass it to a third. That is the property a shared structure cannot
express, and it is why sharing is a derived quantity here rather than a stored
one.[^2]

No code stores a relation yet, because no need for one exists. This decision
states the shape that the first one must take.

## Consequences

The project cannot hold more factions than the mask is wide. A world that
needs more must promote and demote factions between the addressable set and
the reserved bit, and every query that needs an identity must fall through to
a side table.

The project cannot widen the mask after the first golden state hash file
exists without invalidating that file and every recorded replay.

A query that asks which tiles one faction holds cannot read an array. It reads
the masks, then the tiles of the blocks the masks named. The cost of that
query grows with the holding rather than with the world, which is the property
this record buys.

The engine cannot express a tile held jointly by two factions. A joint holding
must be modelled as a faction of its own, or the holder field must change,
which is a new decision that supersedes this one.

A holding cannot spread over ground that admits no unit. An island therefore
stays unheld until a rule for crossing water exists.

The count for each faction is a second statement of the holder column. It can
go wrong, and only a check that derives it again can see that. That check is a
cost this record accepts in exchange for an answer that reads no tile.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: Research report 08, fog of war representation, sections 6.2 to 6.4. `docs/research/reports/08-fog-of-war-representation.md`
[^3]: Findings register, FND-049. `docs/FINDINGS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^6]: ADR-0023, an aggregate combines exactly, in any order, decisions D2 and D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^7]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^8]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
