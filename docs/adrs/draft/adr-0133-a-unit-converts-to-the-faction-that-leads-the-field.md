# ADR-0133: A unit converts to the faction that leads the influence field at its cell

## Context

A god converts people. The project owner decided that conversion changes the
faction of a unit outright.[^1] What that record does not say is what makes a
conversion happen.

Four shapes were available. A field that spreads belief across the world. A
proximity rule between units. A verb that a god calls on a named set. A
standing policy that the engine applies on a schedule.

The engine already holds a field that carries the reach of a faction across the
world. It is a plane over the level 1 cell lattice. The control plane sets a
source term at a cell, and a solve of a fixed pass count spreads what the
sources hold.[^2] A consumer reads the cell it already reads, and it follows no
link to a faction and no link to a ruler.[^3]

Belief is that shape. A god has reach where it has put a source, the reach
falls with distance, and ground that resists influence slows it. Building a
second field for belief would be a second declaration of the same idea, and
nothing would fail when the two disagreed.[^4]

Two things still had to be settled, and neither is obvious. What stops two
factions taking one unit from each other every frame for ever. And whether a
god can convert deliberately, which the owner asked for, without the engine
holding a rule about when a god may do it.

## Decision

### D1. A unit converts to the faction that leads the influence field at its cell, and only when that faction leads strictly

**The pass reads the level 1 cell that covers the tile a unit stands on.** It
finds the faction that reaches that cell most strongly. A unit of any other
faction is a candidate, and the margin is how far the leader stands above the
faction of the unit.

A tie leads to nothing. Two factions that reach a cell equally convert nobody
there. The tie between two equal leaders breaks on the lower faction number,
which is a stable key and never a thread order.[^5]

A cell that no faction reaches converts nobody. That is the ordinary case in a
world where the control plane set no source, so a world that never asked for
conversion never gets one.

The alternative is a proximity rule between units, which asks each unit about
the units near it. It was rejected because the cost of such a rule follows the
population, and the field costs the lattice.[^6]

### D2. Strict dominance is what stops a unit flipping every frame, and the engine holds no cooldown

**A unit converts to the faction that leads. After the change that faction is
its own faction, so the margin against it is zero and it cannot convert
again.** A second conversion of the same unit needs the field itself to change,
and the field changes only when the control plane moves a source or the ground
changes.

The rule is therefore antisymmetric by construction, and it needs no state on
the unit.

The alternatives were a cooldown counter on each unit and a threshold that the
margin must clear. Both were rejected. A cooldown is a column that every unit
carries so that a rare case behaves, and it enters the state hash. A threshold
is a tuning value that decides behaviour, so it would enter the state hash as
well, and it buys nothing that strict dominance does not already give.

The engine does not police the control plane. Two gods that call the verb on
one unit on alternate frames make that unit flip on alternate frames. That is
the control plane doing it on purpose, and the engine states no rule against
it.

### D3. The count that converts is exact arithmetic on the margin, and one keyed draw for each tile names which units

**The pass takes one draw for each group on a tile, and never one for each
unit.** A group is the units of one faction on one tile. The count that
converts is the product of the margin and the headcount, divided by one
reference unit of influence, floored. One keyed draw decides whether the
remainder converts one more.

A second keyed draw rotates the ordinals of the group, and the units that fall
below the count convert. A rotation is a bijection, so exactly as many units
convert as the margin paid for.

A draw for each unit would give each unit an independent chance, and the number
that converted would then vary around the count the margin paid for. The same
argument settled the casualties of a meeting, and this follows it.[^7]

Every draw keys on the tuple of the system, the frame, the tile and the draw
index, and the conversion pass owns a system identifier that no other pass
shares.[^8] The draw index names the faction that loses the unit. It never
names the position of the group inside the tile, because a position depends on
who else stands there.

Every value is an exact integer, and every arithmetic step goes through the
arithmetic module.[^9] [^10]

### D4. The control plane converts a named set outright, and the engine holds no rule about when it may

**A god converts deliberately by calling a verb on a set of units.** The verb
is all or nothing: every identity resolves and the faction is checked before
anything changes.[^11]

The engine holds no cost, no cooldown and no eligibility rule for the verb. The
same shape already holds elsewhere: the control plane may remove an upgrade
instantly, and a rule that decides when it should lives above the engine.[^12]

A god also converts by setting an influence source, which is the write side of
the field and is already a verb of the control plane.[^3] The two routes are
deliberate in different ways. The source is slow, spatial and visible to
everybody. The verb is immediate and names people.

### D5. The pass runs after the influence solve and before the presence relation is folded

**It reads the field that this frame produced, so it runs after the solve.**[^2]

**It writes the faction of a unit, and the presence relation is folded from
that column, so it runs before the fold.**[^13] A conversion after the fold
would leave the relation answering for the factions of the frame before, and
the freshness check of the relation would pass, because the fold records the
arena revision it read. That is a wrong answer that states it is fresh, which
is worse than a refusal.

The pass changes no unit structurally. It removes nobody and it moves nobody,
so no barrier stands between it and the fold.

### D6. Conversion is not gated on territory, because the field is already the gate

**A unit converts wherever another faction leads the field, whether or not that
faction holds the ground.**

A faction reaches a cell because the control plane put a source there and the
solve spread it. The ground already governs that spread, because ground that
resists influence carries less of it.[^2] A second gate that asked who holds
the tile would be a second statement of the same restriction, and the two would
disagree the first time either changed.[^4]

The alternative is the gate that the trade verbs use, which asks whether a unit
of the speaker stands on the ground of the listener.[^14] It was rejected here
because it would make conversion useless: a god that already holds the ground
has less need to convert the people on it, and a god that does not could never
start.

## Consequences

- Conversion costs the occupied tiles multiplied by the faction count, and it
  never costs the population multiplied by anything.
- A world whose control plane sets no influence source never converts a unit.
  The pass still runs, and it still costs the walk over the occupied tiles.
- A frame that converts somebody raises the arena revision, so the derived unit
  structure rebuilds. A settled world converts nobody, so it pays this once
  while the field is moving.
- The engine cannot express a conversion that costs the converting faction
  something. A game that wants a price charges it above the engine.
- A game cannot ask the engine why a unit converted. It reads the field at the
  cell and the event that names both factions.[^15]

## References

[^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decision D1. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
[^2]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^3]: Decisions register, DEC-040 and DEC-041. `docs/DECISIONS.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^7]: ADR-0123, casualties are whole units served to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
[^8]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: ADR-0010, Python is a control plane, and it never touches an entity one at a time. `docs/adrs/REGISTRY.md`
[^12]: Blockers register, BLK-034. `docs/BLOCKERS.md`
[^13]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D2 and D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^14]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D3. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^15]: ADR-0134, a god reads conversion as an event log and as the counts it already reads, decision D1. `docs/adrs/draft/adr-0134-a-god-reads-conversion-as-an-event-log.md`
