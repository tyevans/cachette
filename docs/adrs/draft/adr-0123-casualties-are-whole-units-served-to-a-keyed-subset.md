# ADR-0123: Casualties are whole units served to a keyed subset, never a fraction of everybody

## Context

The resolution of a meeting between two factions computes a quantity of harm
against each group of defenders on a tile.[^1] That quantity is a fixed-point
number, and the units it ends are whole. Something must turn one into the
other.

**A draw for each unit is the obvious mechanism and it is wrong here.** It
gives each unit an independent chance, so the number that falls varies around
the number the harm paid for. This project measured that failure in another
subsystem: a group whose share covered one ration served two units on one
application and none on another.[^2] It also costs one draw for each unit, and
the target population is one million units.

The project already holds the rule that replaces it. A group serves whole
portions to as many of its members as its share covers, and the members it
serves are named by a rotation of their ordinals, keyed on the group and the
frame.[^3] The arithmetic module already floors a share and leaves the
remainder to the caller.[^4]

**A wrongly keyed draw is invisible to both determinism tests.** A draw keyed
on the tile but not on the frame ends the same units for ever. A draw keyed on
a position inside a tile changes when an unrelated unit arrives. Both are
deterministic, so the thread-count test and the golden hash both pass while
they are wrong. This project has had exactly that defect once, in the movement
system, and the testing rule was written from it.[^5]

This decision governs determinism, so it needs a record even though it looks
obvious. A later contributor who wants a smoother fight will reach for a draw
for each unit, and only a written constraint refuses it.[^6]

## Decision

**Casualties are whole units, named by a keyed rotation of the defenders. One
draw serves a whole group, and no draw is ever taken for one unit.**

### D1. The whole part of the harm is certain, and the remainder takes one draw

The harm is a fixed-point quantity of whole units. Its whole part is the number
of casualties the harm certainly produces. Its fractional part covers no whole
casualty.

One draw decides whether the fractional part produces one more casualty. The
draw is uniform below the fixed-point scale, and the pass compares it against
the fractional part, so the number of casualties has the harm as its expected
value exactly. No rounding rule holds that up.

The number of casualties never exceeds the number of defenders present. A tile
cannot lose more units than it holds.

### D2. The subset is the ordinals of the group, rotated by a keyed offset

Each defender holds an ordinal: its place among the defenders of its own
faction and its own type on that tile, in ascending identity order. The pass
already walks those units, so the ordinal costs one counter and no pass of its
own.

The pass draws one offset. A unit falls when its ordinal, advanced by that
offset and wrapped at the group size, is below the number of casualties.

**A rotation is a bijection, and that is why it is a rotation.** Exactly as
many units fall as the harm paid for. A draw taken for each unit does not have
that property.[^2]

The offset is drawn again on every frame, so the block of ordinals that falls
slides and no unit is always first. A fixed offset would end the same members
of a group every time.

### D3. Every draw is keyed on the system, the frame, the tile and the group

The engine keys every draw on the tuple of the system, the frame, the entity
and the draw index.[^7] For this pass the entity is the tile, and the draw
index names the faction and the type of the defender group.

**The contest owns a system identifier of its own.** Two passes that share one
identifier draw the same value from the same frame, entity and index, so the
units that fell on a tile would follow the step the units of that tile took.

**The draw index never names a position inside the tile.** A position depends
on who else stands there, so an index taken from it would change the draw when
an unrelated unit arrived. The faction and the type are properties of the group
itself, and the index packs those two.

Every field of the key must be tested. A test changes one field and asserts
that the draw changes. A golden file is not that test: it notices that
something changed, and it cannot say which input the output stopped depending
on.[^5]

### D4. The order of the fallen never depends on a thread

The pass marks. It ends nothing. The caller applies the marks in one ascending
scan of the unit slots, after the parallel walk has finished, so the deaths
apply in an order that no thread decides.[^8]

The marks join by a bitwise union, which is commutative and associative, so the
joined set is the same at any thread count.[^9]

### D5. Every value is an integer or a fixed-point value

The harm is a 64-bit accumulator of a fixed-point attack scaled by a whole
count. The floor is a shift and the remainder is the low bits of the same
value, so the two cannot disagree. The draw returns an unsigned integer. No
part of this rounds and no part of it is a floating point value.[^10]

## Consequences

**The number of draws in a frame follows the contested tiles and the group
count, never the population.** A world at the target unit population takes no
more draws than a small world with the same number of fights.

**A unit's fortune is not a property it carries.** The ordinal is a position
among the units of its group on its tile, so it changes when a neighbour dies
or a new unit arrives. Two units next to each other in identity order share
their fortune within one frame, and that washes out because the offset moves.

**A fractional casualty is never shown.** A watcher sees whole units fall, and
a player can be shown that.

**A smoother fight cannot be bought with a per-unit draw.** To change this,
supersede this record.

**The engine holds no per-tile memory of a fight.** The remainder is resolved
by a draw in the frame it arose in, and nothing carries between frames.

## References

[^1]: ADR-0121, a meeting between two factions resolves at the tile, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^2]: Findings register, FND-318. `docs/FINDINGS.md`
[^3]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^4]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^5]: Testing Rules, section 2. `.claude/rules/testing.md`
[^6]: Decision Record Scope, the counter-test. `.claude/rules/adr-scope.md`
[^7]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
