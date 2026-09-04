# ADR-0122: An attacker whose attack does not exceed the defender's armour contributes exactly zero

## Context

The project owner stated one acceptance test for combat: **one tank still
kills four bowmen**. He also asked that ten thousand bowmen lose to one tank.
A contest that satisfies the first and fails the second is not the contest he
asked for.

A contest that aggregates first and applies a threshold afterwards cannot
satisfy the second. Any number of small contributions eventually sums past any
threshold, so a large enough crowd of weak attackers always reaches a strong
defender. Holding that off needs a cap, a rate or a balance figure, and each of
those is a number that a later measurement moves.

The engine resolves a meeting at the tile, as a table over unit types rather
than as a fight for each pair of units.[^1] A unit carries a type, and the type
indexes a shared table that the world is built with.[^2] So the pair of types
is known before any aggregation begins.

**Zero is the identity of integer addition.** A sum of zeroes is zero at any
count. That property does not depend on a rate, a cap or a balance figure, so
no later measurement can weaken it.

The cost of this rule is a cliff, and the register holds the judgement that
accepted it. An attack one step below the armour does nothing and an attack one
step above it does everything. One point of armour makes a unit immune to a
whole class of attacker.[^3] This project has met the same shape before, in the
rule that feeds a unit, and removed it there with a per-unit accumulator.[^4]
**The project owner chose the cliff here**, because the stated test asks for
it.

## Decision

**The engine applies the penetration threshold for each attacker type before
it aggregates anything. An attacker type whose attack does not exceed the
defender type's armour contributes exactly zero.**

### D1. The threshold applies before the aggregation, for each pair of types

The pass takes the attack of the attacker type and the armour of the defender
type. It compares them. When the attack does not exceed the armour, the pass
adds nothing at all for that attacker type, whatever number of attackers the
tile holds.

The comparison is strict. An attack that equals the armour does not exceed it,
and does not reach.

The order is the decision. A pass that multiplied the attack by the count and
then tested the product against the armour would satisfy the tank test at four
bowmen and fail it at ten thousand. The pass tests the pair, and never the
product.

### D2. The threshold is a pair of table values, and this record holds neither

The attack and the armour are rows of the shared unit type table. They are
content that a caller supplies, and no record holds one, because a record may
hold no number that a content choice can move.[^5]

The engine refuses a negative attack and a negative armour when a caller writes
a row. A negative armour would sit below every attack, including no attack at
all, and a negative attack would heal.

### D3. The aggregation stays exact and order-free

The harm against one defender type is a sum of terms. Each term is a
fixed-point attack scaled by a whole count, in a 64-bit accumulator.[^6] The
sum is integer addition, so it is commutative, associative and exact, and a
reduction over it gives one answer at any thread count.[^7]

The threshold does not change that. It fixes the set of terms before the sum
begins, and it fixes it from the pair of types alone. Nothing about it reads a
thread, an order or a count.

## Consequences

**A unit can be immune, and immunity is total.** No number of attackers below
the armour of a defender ever reaches it. That is the requested behaviour and
it is also the complaint a player will make.

**The threshold removes attrition for every pair it applies to.** A tank that
cannot be hurt at all produces no grinding, and grinding is what makes an army
look like it is overpowering another. A game that wants the grinding gives the
attacker enough attack to exceed the armour, and then the aggregate takes over.

**A change of one step in a table value can change a whole war.** A caller who
raises an armour past an attack makes a class of attacker useless in one edit.
The engine reports nothing, because nothing is wrong.

**A softer threshold needs a new record.** A contribution that fell off steeply
but never to zero would satisfy neither the stated test nor this record. To
change this, supersede it.

**Nothing here depends on a measurement.** The tank test passes structurally.
A benchmark, a platform, or a new derivation cannot move it.

## References

[^1]: ADR-0121, a meeting between two factions resolves at the tile, decision D3. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^2]: ADR-0120, a unit carries a type, and the type is an index into a table the world is built with, decision D1. `docs/adrs/draft/adr-0120-a-unit-carries-a-type-that-indexes-a-table.md`
[^3]: Decisions register, DEC-145. `docs/DECISIONS.md`
[^4]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D4. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^5]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
