# ADR-0195: The observation of a faction is a fixed-width scale-free table in an egocentric frame

## Context

The engine gives one faction one flat array of signed integers on every
decision. It declares where each field of that array starts in one schema, and
that schema is the only declaration of the layout.[^1] A learner trains a
function of the layout, so the layout is the interface of every stored policy.

An accepted decision bounds every length of that array. It also permits a
length that follows the world parameters, and it asks the reader to start no
pass over the tiles.[^2] Both clauses were reasonable when nobody had built the
array. Both are now wrong, and a measurement says so.

**The width follows the world shape and the faction count, so one policy fits
one world and no other.** A field with one position for each faction multiplies
by the faction count. A field with one position for each cell of the block
lattice multiplies by the cell count, and the cell count follows the tile
count. The finding holds the measured widths.[^3] The same rule is blind at the
small end and unaffordable at the large end. On the small world the project
trains on, the lattice holds one cell, so every spatial field covers the whole
world and carries no direction. At the target tile count the same rule gives a
lattice that no policy of this project reads.

**A faction reads its own ground through the fog of the current frame, so its
own borders flicker.** The held tile count is the whole count. The spatial
field is scoped to the tiles the faction sees this frame, so the two report one
quantity at two scopes. A second finding holds the measurement.[^4] No rule of
the game hides the holdings of a faction from itself. The picture moves when
the units move, and the ground did not move.

**A published value is a raw count with no stated bound.** A held tile count
means one thing on the world the project trains on, and another thing on the
target world. A running total may be declared with the whole integer range as
its bound. The trainer takes a fixed-size normalised step, so an input whose
range spans many orders of magnitude cannot be trained.[^5]

A research report works this problem out and states a complete design.[^6] It
gives the value kinds, the spatial frame, the rival encoding, and the pass that
builds the array. This record takes the constraints of that design and holds
none of its figures.

**A draft record answers one part of this problem and is replaced here.** That
record addresses a field with one position for each faction by the distance
from the reader, rather than by a seat number.[^7] The rule is sound against
the defect it names. This record removes the field it governs, so the rule has
nothing left to govern.

**This record runs ahead of the code, and every decision below is a constraint
on work that nobody has written.** No builder produces the array this record
describes. The array the engine publishes today holds the three defects above.

## Decision

**The observation of a faction is one table whose width is a constant, whose
every value is scale free, and whose spatial part reaches the policy in a frame
centred on the faction.**

**This record changes the accepted decision that bounds the array, and it
replaces that decision.**[^2] It narrows the length rule from a function of the
world parameters to a constant. It also withdraws the clause that forbids a
pass over the tiles, and it puts a cost bound in its place under D5. Every
other decision of that record stands, and D1 of it governs this layout as it
governs the current one.[^1]

**This record supersedes the record that addresses a faction-indexed field by
the distance from the reader.**[^7] D4 below removes every faction-indexed
field, so the rotation rule of that record governs nothing. Its version clause
returns as part of D9.

### D1. The width of the observation is constant in the world shape and in the faction count

The array holds the same number of positions on every world and at every
seated faction count. Every field starts at the same position on every world.

No length follows the tile count, the world width, the world height, the block
lattice cell count, the seated faction count, or the population. A scale
constant bounds the seated faction count, and the width does not depend on that
bound either.[^8]

The width is a function of the layout parameters alone. Those parameters are
register rows, and a decision register holds the open choice of what each one
should be.[^9] [^10]

A reviewer finds a violation when any field length reads a world parameter or
the faction count. A test builds the array on a small world at the lowest
seated count and on a large world at the highest seated count, then asserts one
width and one start position for each field.

### D2. Every published value is a share, a signed relation, or a compressed magnitude

The array publishes no raw count and no total whose bound is the integer range.
Every position holds one of three kinds.

A **share** lies between zero and one. It divides an extensive quantity by a
denominator the schema names. The denominator is a structural property of the
world, or a total the engine already maintains. This is the rule an accepted
record already states for an intensive summary field: store the extensive
parts, divide at read time, and name the extent the field is defined over.[^11]
[^12]

A **signed relation** lies between minus one and one. It divides the difference
of two magnitudes by the sum of their absolute values, so it needs no chosen
denominator.

A **compressed magnitude** lies between zero and one and carries the sign of
the quantity. It maps the quantity through an integer base-two logarithm
against a structural cap. The cap is a property of the widest quantity the
engine can hold, and not a budget.

Each kind is an exact integer function of integer inputs, so no position holds a
floating point value and the array holds none.[^13] Every division truncates
toward zero, and the rounding rule is the same for every position. The learner
scales the integers on its own side of the boundary, where floating point is
allowed.[^14]

**A share hides the absolute scale, and the array gives that scale back through
its own positions.** Where the correct action depends on the absolute quantity,
the array publishes the compressed magnitude beside the share. It also
publishes the scale of the world and the seated faction count as positions of
their own. A policy that reads a share and a scale can recover the quantity.

A reviewer finds a violation when a position holds a count, when a share has no
named denominator, or when a denominator depends on the episode or on the data
seen so far.

### D3. Space reaches the policy in an egocentric frame whose origin follows the faction and whose axes do not turn

The spatial part of the array is a stack of rings around one centre. The centre
is the integer centroid of the ground the faction holds. Resolution falls with
the distance from the centre, so a near ring covers few tiles and a far ring
covers many. The ring count and the sector count are layout parameters.[^9]

**The origin follows the faction. The axes do not.** A sector is anchored to
the axial directions of the world, and the world is a rhombus whose tile index
is a raw axial pair.[^15] The engine selects a sector by comparing the signs
and the magnitudes of the cube coordinates of the offset, so the frame needs no
division and no trigonometry.

A rotating frame is refused for the reason a state-dependent order is
refused.[^7] A rotation would come from the state of the game, so the meaning
of a sector would move from one decision to the next under one weight. A policy
cannot learn a function of an address that moves under it.

The frame also gives the width rule of D1 its mechanism. A ring that lies
outside a small world reads the position that reports how much of the ring lies
inside the world, and that position gates the rest of the ring.

A reviewer finds a violation when a spatial length follows the world extent,
when the sector frame reads a game quantity, or when a cell of the stack has no
position that reports how much of it lies inside the world.

### D4. No position of the array names a seat

No field holds one position for each faction. No position holds a seat number,
and no position holds the identity of the reader.

A faction reaches its rivals in two ways. The first is an order statistic over
the rivals, whose width follows the statistic count and never the faction
count. The second is a set of tokens over a fixed number of rivals, where the
position of a token carries no identity.

**The engine sorts a token set only to fix the published bytes.** The sort key
ends in the seat index, so two states that tie still publish one order.[^16]
The order carries no meaning, and a reader must not learn a function of a token
position. The consequences state what that costs a reader.

This decision agrees with the accepted rule that no field of the world is
indexed by the faction.[^17] It also removes the field that the superseded
record addresses.[^7]

A reviewer finds a violation when a length follows the seated faction count,
when a position names a seat, or when a token set has no validity position. A
test relabels the seats of every rival by a permutation and asserts that the
order statistics are identical byte for byte, and that the token set is
identical as a set.

### D5. The build cost follows what the faction has observed, and never the world area

The pass that builds the array reads a fixed inner region at tile level, and it
reads the summary cells the faction has observed for everything beyond that
region. It never walks the world.

This replaces the clause of the accepted record that forbids a pass over the
tiles.[^2] A bounded pass is admissible, and an unbounded one is not. The
bound is the observed area plus a fixed region, and the fog record already
holds the same bound for the storage.[^18] The cost of a decision therefore
follows the lattice the faction has seen, and never the population.[^19]

The engine masks every value it reads from the shared summary level with what
the faction has observed. It never publishes an unmasked summary value, because
that would give a faction knowledge that no rule of the game gives it.[^20]

A reviewer finds a violation when the build reads a structure whose size
follows the world area, when it iterates the units of a rival, or when it reads
a summary value without the mask. A test instruments the build to count what it
touches, then asserts that two worlds of different size and equal observed area
touch the same count. Every cost figure of this pass stays derived until the
target platform measures it.[^21]

### D6. The build is one integer pass whose order the engine states, and the two levels it reads must agree

The pass performs integer arithmetic alone. It accumulates into wide integer
accumulators, so no accumulator depends on a margin below its own ceiling.[^13]

The pass may shard. A shard accumulates into its own array, and the engine
combines the shards in ascending shard index. Integer addition is associative,
so the combination is exact whatever the shard count. The pass never combines
in thread completion order and never in work-stealing order.[^22] [^23] One
binary therefore gives one array at any thread count.[^24]

The pass reads the truth at level zero for the inner region and the derived
summary level beyond it. The summary level is a projection of level zero, so
the two must agree.[^25] A cell of the ring stack that the faction has observed
in full holds the same value whichever level built it, and a test asserts that
equality.[^26]

A reviewer finds a violation when a build order comes from the completion of a
thread, when an accumulator is narrower than the widest sum it can take, or
when the two levels disagree for a fully observed cell.

### D7. The fog of the current frame never masks the state of the reader

The array reports the state of the faction that reads it at full scope. The
ground the faction holds, the units it owns, the settlements it owns and the
stores it holds are its own state, and no sight rule hides them from it.

The fog scope applies to what the faction reads about the world and about a
rival. Where a spatial cell reports the ground of the reader and the ground of
a rival, the two are separate positions under separate rules.

**A position that reports what a faction sees now and a position that reports
what a faction saw once are different positions, and each says which it
counted.**[^27] [^28] A remembered value carries the age of the sighting, so a
policy can tell a current fact from an old one.

A reviewer finds a violation when a position that reports the state of the
reader is scoped to the sight of the current frame, or when one position mixes
what a faction sees now with what it saw once.

### D8. An estimate carries how much of it the faction observed, and an absent row says that it is absent

Every estimate about a rival carries a position that reports how much of the
estimate the faction observed. Without it a policy reads an inferred quantity
and a seen quantity as one fact, and it cannot learn to scout.

A row of a token set that has no subject holds a validity position that reads
false. A reader therefore separates an absent row from a row whose quantities
are small. The array never reports an unobserved value as zero without a
position that says the value is absent.

A reviewer finds a violation when an estimate has no confidence position, when
a token set has no validity position, or when an absent row is indistinguishable
from a present row of small quantities.

### D9. The layout carries a revision, and a reserve absorbs a new signal without changing the width

The engine publishes one revision integer beside the schema. A stored policy
states the revision it trained against, and a load against another revision
stops with an error that names both.

**The revision is now the only guard.** While the width followed the world, a
length mismatch caught a policy loaded against the wrong world. Under D1 two
layouts of different meaning have one width, so nothing but the revision
separates them. The revision rises when a field is added, removed, moved,
rebounded or rescaled, and when the addressing of a field changes.[^7]

The layout holds a reserve of positions. Every reserved position reads zero
until a revision claims it, and the engine asserts that. A signal added inside
the reserve therefore leaves the width and every earlier start position
unchanged. The size of the reserve is a layout parameter and a judgement, and
the decision register holds it.[^10]

A reviewer finds a violation when a reserved position holds a value under the
current revision, when a change to a field leaves the revision where it was, or
when a policy loads against a revision it did not train against.

## The alternatives this rejects

**A raster of the whole world at tile resolution.** Rejected because the width
follows the tile count and the build cost follows the world area. It fails D1
and D5.

**A raster of the whole world at a fixed resolution.** The width is fixed,
which is correct, and one leading environment publishes such a view.[^6] Two
faults remain. The resolution for each tile falls as the world grows, so one
settlement covers a whole position on a small world and no position on a large
one. The build cost also follows the world area. Rejected, and adopted in part:
the outer rings of D3 do the same work at a cost bounded by observed area.

**A fixed local crop and nothing else.** The width is fixed and the cost is
constant. The policy then reads nothing beyond the crop, so it cannot take a
decision at the scale of the world. Rejected as the whole answer, and adopted
as the inner rings of D3.

**The derived summary level published as it stands.** The channel count is
fixed and the cell count follows the world, so it fails D1 for the same reason
as the raster. Rejected as the published form, and adopted as the source of the
outer rings.

**Square egocentric crops at falling resolution.** This is the closest
rejected alternative, and a later reader should reconsider it first. It needs no
sector arithmetic and it is simpler to build. Two reasons decide against it. A sector axis wraps, so a reader that shares weight across
directions shares it correctly, and the edge of a square does not wrap. A ring
frame also holds one resolution rule for every distance, and a stack of crops
holds one rule for each crop.

**A block for each seat.** Rejected under D1 and D4. The width follows the
faction count, and a policy learns a seat number, so it fails on another
seating.

**Rivals sorted into fixed positions, read by their position.** The width is
fixed, which is correct. A policy then learns behaviour conditioned on a
position, and its output jumps when two rivals change rank. Rejected under D4
in favour of order statistics and a token set whose position carries no
meaning.

**A field with one position for each faction, addressed by the distance from
the reader.** This is the superseded record.[^7] It removes the seat that a
policy would otherwise learn, and it leaves the width following the faction
count. Rejected because D1 needs a constant width and D4 removes the field.

**A learned running normaliser over the inputs.** It gives the same range
control, and the established methods for it are known.[^6] It holds floating
point state, which the physics forbids, and it makes the mapping depend on the
data seen so far. One world state then gives two different arrays in two runs,
and the determinism test fails. Rejected under D2 and D6.

**A minimum and maximum taken over the episode.** The denominator then depends
on the episode, so the same defect follows. Rejected.

**The scale of the world published as the whole mechanism.** Publishing the
scale helps, and D2 publishes it. It does not by itself make the width
constant. Adopted as a signal, rejected as the mechanism.

**Hand-written strategic features with no spatial input.** Cheap to build and
quick to train. It caps the policy at the quality of the features, and the
policy cannot learn a spatial idea that no author had. Rejected.

**A token for every unit of a rival.** Rejected because the token count is
unbounded and the cost follows the population.[^19] The summary level already
holds the strength of a block, so a cluster token reads it instead.

**The shared summary level read without the mask.** Cheaper, and it removes the
need for any per-faction remembered value. It gives a faction knowledge it has
not observed, which the product record and the fog record both refuse.[^20]
[^18] Rejected under D5.

## Consequences

**Every stored policy of the current layout is retired.** The width changes,
the fields change and the value kinds change, so no permutation of weights
recovers a trained policy. Retrain rather than convert.

**A measurement taken on a small world becomes comparable with one taken on a
large world.** One policy reads both. That is the point of D1, and it is the
thing the current layout cannot do.

**A reader cannot address a named rival.** A weight cannot say "the faction in
that seat", and it cannot say "the second token". D4 gives the token set no
meaningful order, so a reader must reduce the set in a way that ignores the
order. A reader that reads a token position learns an order the engine does not
promise.

**A policy must hold two inputs to recover an absolute quantity.** D2 publishes
a share and a scale, and the product of the two is the quantity. A linear
reader cannot form that product, so a policy that needs an absolute quantity
needs at least one hidden layer.

**The compressed magnitude is a poor reward term.** Its slope falls as the
quantity grows, so an early gain outweighs a late gain of the same size. A
reward built from this array should read the shares. This record decides no
reward, and the reward is a separate claim.

**The cost bound of D5 obliges a per-faction remembered value for every
fog-dependent channel.** The shared summary level holds the truth, so a masked
read cannot answer what a faction saw about a rival at a cell it no longer
watches. What a remembered place answers is an open choice the register already
holds, and this record sharpens it rather than settling it.[^29] The storage
itself is governed by the fog record, and its shape must follow the observed
area and never the world area.[^18]

**The engine performs a division the policy would otherwise learn.** A coverage
in ticks and a share of a total are integer divisions the engine already has
the inputs for. The learner is a control plane, so it sends one integer and
reads one array, and it never loops over entities to derive one.[^30]

**This record holds no figure.** Every ring count, sector count, channel count,
token count, cap and reserve size is a register row, and the report holds the
derivation of each.[^6] [^9] Every cost figure of the build stays derived until
the target platform measures it.[^21]

**The superseded record still describes the code until this layout lands.** Its
rule is implemented, and no record governs that rule once the registry marks it
superseded.[^7] The rule leaves the tree with the field it addresses, and D4
removes that field. Until then a reader of the code should read this record for
the direction and the superseded one for the shipped behaviour.

**A test that reads one world proves nothing about D1.** The width rule fails
only across two world shapes and two faction counts, and the seat rule fails
only across a permutation of the seats. Each test of this record needs a paired
negative test that proves it can fail.[^31]

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: Findings register, FND-670. `docs/FINDINGS.md`
[^4]: Findings register, FND-671. `docs/FINDINGS.md`
[^5]: Salimans, Ho, Chen, Sidor, Sutskever, "Evolution Strategies as a Scalable Alternative to Reinforcement Learning", 2017. https://arxiv.org/abs/1703.03864
[^6]: Research report 42, what a policy should be able to see. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
[^7]: ADR-0193, a faction's observation names another faction by a position relative to the reader. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
[^8]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^9]: Reinforcement learning parameters register. `docs/reference/rl-costs.md`
[^10]: Decisions register, DEC-282. `docs/DECISIONS.md`
[^11]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^12]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^13]: ADR-0002, state holds no floating point number, decisions D1 and D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^14]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^15]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^16]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^17]: ADR-0053, a faction is a bit in a mask and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^18]: ADR-0059, fog storage grows with observed area, not with world area, decision D2. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^19]: ADR-0096, cost follows the lattice, not the population, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^20]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^21]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^22]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^23]: ADR-0023, an aggregate combines exactly in any order, decisions D1 and D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^24]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^25]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^26]: ADR-0023, an aggregate combines exactly in any order, decision D5. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^27]: ADR-0059, fog storage grows with observed area, not with world area, decision D3. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^28]: ADR-0059, fog storage grows with observed area, not with world area, decision D6. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^29]: Decisions register, DEC-276. `docs/DECISIONS.md`
[^30]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^31]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
