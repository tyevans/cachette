# ADR-0204: Every win path holds a bar of its own, and the tick limit ends a game with no winner

## Context

The engine ends a game when a reader fires. It holds four readers: domination,
territory, wonder and renown. A record states that the readers run in a fixed
order and that the first one to name a faction decides the game.[^1]

A product record asks that each of the four be a plan. A plan is a way to win
that a player selects at the start and steers toward for a whole game. The
record states what good looks like as a list of checkable statements, and the
engine fails two of them today.[^2]

**Three readers compare a faction against a bar. One ranks the factions.** The
territory reader answers nothing below the tick limit. At the limit it names the
faction with the most held ground. It holds no bar, so it asks for no act. A
faction that never acts keeps the ground its first city holds, and that ground
can be the most.

A code audit and a measurement over two fields of players agree on what follows
from that.[^3] [^4] Every seed that a long horizon ends on renown is a seed that
a short horizon ends on territory at the limit. **A territory win is the
truncation of another path and not a separate outcome.** The findings register
holds the correction.[^5]

The product record names the two failures directly.[^2] No way to win may be the
outcome of making no act. The distance a faction reads toward a way to win must
reach the winning value on the tick that the win fires. Territory fails both.
Its published reading measures held ground against the passable ground of the
world, and no reader compares that ratio.

A second failure is quieter. The domination reader holds a clause that the
observation cannot publish. That clause names a faction whose every rival lost
its last unit. A faction may not read the units of a rival it never saw.[^6] A
player cannot steer toward a clause it cannot read.

The value each reader compares is already a value the world holds, and a caller
sets each one.[^7] This record decides the shape of the bars. It states no
value. Open blockers govern every share, every target and the tick limit, and
the balance register holds each provisional value.[^8] [^9] [^10]

## Decision

**A game ends when one faction crosses a bar of its own. A game that reaches the
tick limit ends with no winner.**

### D1. A game ends only when one faction crosses a bar of its own

Every game end reader compares one quantity of one faction against a bar. No
reader compares the quantity of one faction against the quantity of another. No
reader names a faction because that faction leads.

Every reader runs on every tick, under one schedule. No reader stays silent
below a stated tick and answers at it.

A reader may read how many factions are still in the game. That count states the
size of the field and not the standing of a rival.[^11]

A reviewer finds a violation when a reader compares one faction against another,
or when a reader answers on one tick and refuses to answer on another.

### D2. Territory is a threshold path, and its bar is a share of the ground the world supplies

The territory reader names the first faction whose held ground reaches a stated
share of the passable ground of the world.[^12] The share is a parameter. This
record states no value for it. The blocker that holds the rules of the
downstream game owns the value.[^8] The world holds the parameter and a caller
sets it.[^7]

Both sides of the comparison are legible to the faction that reads them. The
passable ground of the world is a public quantity of the world. The held ground
of a faction is that faction's own state, and no sight rule hides it from the
faction.[^13]

A reviewer finds a violation when the territory reader ranks the factions, when
it compares a tile count rather than a share, or when this record states the
value of the share.

### D3. A game that reaches the tick limit ends with no winner, and the record says so

The tick limit ends the game. It names no faction and it fires no reader.

The end at the limit is therefore not a reader, and the switch that turns the
readers off does not reach it.[^7] A run with the readers off and a run with the
readers on that never fires both end at the limit with no winner.

The end record then states that the game ended at the limit with no winner. That
outcome differs from a game still running, and it differs from a game a faction
won. A reader of the record tells the three apart without reading the tick.[^14]

A game with no winner is a correct outcome and not a defect. It states that no
faction carried a plan far enough to finish one.

A reviewer finds a violation when a game that reaches the tick limit names a
winner, when the record of such a game reads the same as the record of a running
game, or when a reader fires because the tick reached the limit.

### D4. A reading reaches its top value on the tick its reader fires, and never before

Each win path publishes one reading for the faction that reads it. The numerator
is what the faction has done toward the requirement of that path. The
denominator is the requirement. The reading is a bounded share, and it holds no
floating point number.[^15]

**The requirement and the value a reader compares are not always one quantity.**
The wonder reader compares a standing claim, and the requirement of that path is
the work that finishes the claim.[^16] The rule below binds the reading to the
requirement, so it reaches a path whose reader compares a state rather than a
distance.

Two properties follow, and both are the decision. The reading reaches its top
value on the tick that the reader of its path fires. The reading reaches its top
value on no earlier tick.

**A clause that the reading cannot carry is not a clause of the reader.** A
reader with a second clause either publishes a second reading for that clause,
or gives the clause up. A clause that the fog rule forbids the observation to
publish is a clause the reader gives up.[^6]

A reviewer finds a violation when a reader fires while the reading of its path
sits below its top value, when a reading reaches its top value on a tick that no
reader fires, or when a reader holds a clause that no published reading carries.

### D5. Every bar is a share of a supply the world states

For each win path the world states one supply. A supply is a quantity that the
world computes from the world it built, and the world publishes it. The bar of a
path is the parameter of that path applied to its supply.

The supply of the territory path is the passable ground of the world.[^12] The
supply of the domination path is the seats of the factions still in the game, so
that bar is already a share and takes no parameter.[^11]

The supply of the wonder path and the supply of the renown path are open. One
blocker holds the rules of the downstream game.[^8] One blocker holds what
raises and lowers renown, so nobody can yet say what a whole game supplies
toward that path.[^9] This record makes each path declare a supply, and it names
neither of those two.

A reviewer finds a violation when a reader compares a quantity against a value
that does not move when the extent of the world, the faction count or the tick
limit moves. A bar that is already a share of a live quantity satisfies this
rule by construction.

## The alternatives this rejects

**Keep territory as a ranking, and name it a tiebreak rather than a win.**
Rejected. A tiebreak at the limit is still the outcome of making no act, and the
product record refuses that outcome.[^2] The choice is defensible on two
grounds, and both fail. It keeps every game decided, which reads as tidy and
which hides a faction that never acted behind a rank. It also costs nothing to
build, because the ranking already exists. The measurement says what the rank is
worth: the games that reach the limit are the games another path had not
finished, so the rank reports the horizon and not the plan.[^4]

**Remove territory from the game, so that three paths remain.** Rejected. The
ground share is a viable base for a real bar. It is continuous, it stays
contested until late in a game, and the leader reading of a measured set spans a
wide range with room for a bar inside it.[^4] Removing the path takes a live
plan out of the game to avoid writing a bar.

**Give territory a bar on the ground the faction holds against the ground its
strongest rival holds.** Rejected on two grounds. It compares one faction
against another, and D1 forbids that. It also reads a quantity of a rival that
the observation cannot publish under the fog rule, and D4 forbids that.[^6]

**Publish a second reading for the domination clause on rival units.** Rejected.
The reading states the unit count of a rival that the faction never observed,
and the fog record refuses that read.[^6] The clause goes instead.

**Break a tie at the tick limit by the reading that each faction climbed
highest.** Rejected. It is a ranking in the clothes of a bar, and D1 forbids a
ranking. It also rewards the faction that came nearest to a plan it failed to
finish.

**State the share of ground that territory asks for in this record.** Rejected.
A blocker owns every rule of the downstream game, and a record that invents one
states a false thing as soon as the blocker closes.[^8] The balance register
holds the provisional value with the reading that produced it.[^10]

**Fix each bar as a count rather than as a share.** Rejected on scale. A count
that a small world cannot reach in a whole game, and that a large world reaches
early, gives a dead path at one size and a trivial path at the other. The
product record makes the requirement of each path scale with what the world
supplies.[^2]

## Consequences

**Territory now asks for an act, and the game gains an outcome with no winner.**
A faction that founds one city and stops never reaches the bar. A set of
factions that make no act ends every game at the limit with no winner. That is
the outcome the product record asks a reader to be able to see.[^2]

**The share of games that end with no winner becomes a measurement, and it can
be large.** A bar set too high empties the game of endings. The bar is therefore
the lever that trades a decided game against an earned one, and a run that
reports its win share reports that trade.[^17]

**The reading of the territory path becomes correct without a change to what it
measures.** It already divides held ground by the passable ground of the world.
Once the bar is a share of that same quantity, the reading reaches its top value
exactly at the win.

**Domination loses its clause on rival units, and annihilation still ends a
game.** A faction that holds no site and no unit leaves the game.[^11] A faction
that outlives every rival therefore holds every seat still in the game, and its
reading reaches its top value. A rival that keeps a city and loses every unit
stays in the game, and the game continues. That is a change of behaviour, and
this record intends it.

**The fixed order of the readers now decides only a true tie.** Two paths that
cross on one tick still resolve by that order.[^1] No reader pre-empts another
by firing at the limit, because no reader fires at the limit.

**A record that states the shape of a bar states no value, so the balance
register carries the whole tuning surface.** Every parameter this record names
is unset under a blocker. A reader who wants a number reads the register and not
this record.[^10]

**Each bar is state that the step reads, so each enters the state hash.**[^18]
Each supply the world publishes is derived from the world it built, so two
worlds built alike publish one supply.

**The scale at which the paths must stay live is not known.** One blocker holds
the world size, the unit count and the faction count of the downstream
game.[^19] D5 makes each bar move with those quantities, so the rule holds at a
scale that nobody has yet stated.

## References

[^1]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^2]: PRD-0057, more than one way to win decides a game. `docs/product/shaped/prd-0057-more-than-one-way-to-win-decides-a-game.md`
[^3]: Research, what the win conditions are and what can reach them. `docs/research/what-the-win-conditions-are-and-what-can-reach-them.md`
[^4]: Research, what a game attains on every win quantity. `docs/research/what-a-game-attains-on-every-win-quantity.md`
[^5]: Findings register, FND-753. `docs/FINDINGS.md`
[^6]: ADR-0059, fog storage grows with observed area, decision D4. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^7]: ADR-0175, a win threshold decides when a reader fires, decisions D1 and D3. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
[^8]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^9]: Blockers register, BLK-150. `docs/BLOCKERS.md`
[^10]: Balance register, the win path rows. `docs/reference/balance.md`
[^11]: ADR-0181, a faction that holds no site and no unit leaves the game, decision D1. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
[^12]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^13]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D7. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^14]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^15]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^16]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^17]: ADR-0202, a run selects on the win share, decision D1. `docs/adrs/draft/adr-0202-a-run-selects-on-the-win-share-and-every-published-figure-names-its-seed-set.md`
[^18]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^19]: Blockers register, BLK-051. `docs/BLOCKERS.md`
