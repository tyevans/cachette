# ADR-0153: A tile's lease follows the units that stand on it, and a lease at the claim threshold outranks the reach of a city

## Context

A tile carries one holder, and the holder is one faction or nobody.[^1] One
record decides who that holder is. A tile is held by the faction of the nearest
city whose reach covers it, and a tile no city reaches is held by nobody.[^2]
The reach of a city grows only with the upgrades finished inside its own
ground, and it stops at a bound.[^3]

**That rule gives a faction no way to gain ground by using it.** A faction
whose units cross the same ground every tick for a whole run holds none of it,
unless a city of that faction already reaches it. The border between two
factions is the line where two discs meet, and nothing a faction does outside a
city moves that line.

The project owner asked for a second way to hold ground. Ownership should
follow where a faction's units are. A faction goes to the same places outside
its territory, and repeated use should erode the border, so that the frequency
of use becomes ownership. He also asked what fills the space in between, when
the ground of one faction almost surrounds a piece of ground that faction does
not hold.

Four forces fix the shape.

**Use is not presence.** The rule this project superseded held ground wherever
a unit stood, and a holding then had no centre and could not be lost.[^4] A
rule that reads the units of one tick repeats that failure. The engine must
record use over many ticks, so that ground a faction stops visiting returns to
nobody.

**No field of the world is indexed by the faction.** The holder is one dense
column over the tiles, and exclusivity is a property of that storage: one tile
holds one value, so no tile names two factions.[^5] [^6] A count for each
faction on each tile would be the tile count multiplied by the faction ceiling,
which the storage rule refuses.[^7]

**Use accumulates, so the engine cannot derive it.** The presence relation is
derived at the end of every step and stored nowhere, because every part of its
answer is in the world already.[^8] A record of use over many ticks is not in
the world. It is a fact the step must carry forward.

**Every value here is a game value.** How fast use claims ground, how fast an
unused claim falls away, and how far a hole is closed are rules of the
downstream game, and one blocker holds those rules.[^9] This record names the
values and states none of them.

## Decision

**Each tile carries one lease. A lease is one faction and one count, and the
count follows the units that stand on the tile. A tile is held by its lease
faction when the count is at or above the claim threshold, then by the nearest
city in reach, then by nobody. A fixed number of closure passes then gives an
unheld tile to the one faction that rings it.**

### D1. A lease is one faction and one signed count for each tile, and no field is indexed by the faction

Each tile carries one lease. A lease holds one faction identifier, which names
one faction or nobody, and one count. The count is a whole number.[^10] It is
never below zero after the rule ends, and it never passes a bound.

The lease is two dense columns over the tiles, in the form the holder column
already has. **No field of the lease is indexed by the faction.**[^7] One tile
names one faction, so the storage refuses a second one and no rule keeps two
factions apart.

The count is a signed type, so that a subtraction below zero is representable
and the overflow gate does not fire on the ordinary case.[^11] The rule of D2
then reads the sign and writes a value at or above zero.

A reviewer finds a violation when a column of the lease is indexed by the
faction, when a count is a fraction, when a tile carries two leases, or when a
count passes the bound.

### D2. The lease of a tile that carries a unit moves one step each tick, and it changes hands when the count reaches zero

At the holding stage of every step, the rule reads every tile that carries at
least one unit. D3 names the faction present at the tile.

- When the lease names the faction present, the rule adds the raise step.
- When the lease names another faction, the rule subtracts the lower step.
  When the result is at or below zero, the lease names the faction present, and
  the count becomes the amount by which the result passed zero.
- When the lease names nobody, the lease names the faction present, and the
  count becomes the raise step.

The raise step, the lower step and the bound are balance values.[^12] This
record states none of them.

The rule reads the lease as the previous step left it, and it writes the lease
of this step. It reads no holder, so the holder of a tile never decides the
lease of that tile.

A reviewer finds a violation when a tile that carries no unit gains the raise
step, when a change of hands drops the amount that passed zero, when the rule
reads the holder column, or when a step is a fraction.

### D3. A tile that carries units of two factions gives the tick to the faction with more units, and a tie goes to the lowest faction identifier

The rule counts the units of each faction that stand on the tile. The faction
with the most units takes the tick. Two factions with equal counts resolve by
the lower faction identifier.

The tally holds only the factions that stand on one tile, and the capacity of
the ground bounds how many units that is.[^13] The tally is therefore local to
one tile, and D1 still holds: no stored field is indexed by the faction.

**The tie must not read the order in which the rule visits the units.** The
units of one tile reach the rule in the order the unit index packs them, and
that order changes when a unit dies, moves or is promoted. A rule that took the
first faction it found would take the packing, and a packing is not a stable
key.[^14] Both determinism tests would still pass, because each compares one
run against another run of the same binary, and a defect that is itself
deterministic survives both.[^15]

A reviewer finds a violation when the resolution reads the first unit on the
tile, when it stores a count for each faction over the tiles, or when the tie
reads anything other than the faction identifier.

### D4. The count falls toward zero on a fixed schedule, and a lease at zero names nobody

On a fixed period and phase, the rule subtracts the decay step from the count
of every tile whose count is above zero. A count that reaches zero stops there,
and the lease then names nobody. The decay step, the period and the phase are
balance values.[^12]

The schedule is fixed. No convergence test, no time budget and no wall clock
ends the decay, because a run must give one answer at any thread count and on
any machine.[^15]

**The decay is what makes a path that a faction abandons return.** The bound of
D1 is what stops a lease from becoming unshiftable. A faction that used one
tile for a whole run holds a count no larger than the bound, so another faction
takes the tile in a stated number of ticks.

The cost of the pass follows the tiles that carry a unit and the tiles whose
count is above zero. It never follows the world.

A reviewer finds a violation when the decay reads a clock, when a tile that no
unit visits keeps its count for the rest of the run, or when a lease at zero
still names a faction.

### D5. A tile is held by its lease faction at the claim threshold, then by the nearest city in reach, then by nobody

The rewrite decides the holder of each tile by three tests, in this order.

1. **The lease.** When the lease of the tile names a faction, and the count is
   at or above the claim threshold, that faction holds the tile.
2. **The city.** Otherwise the faction of the nearest city whose reach covers
   the tile holds it. Two cities at one distance resolve by the lower
   settlement slot index. That rule is unchanged.[^2]
3. **Nobody.** Otherwise no faction holds the tile.

The claim threshold is a balance value.[^12] It is at or below the bound of D1,
because a threshold above the bound would make the lease reach nothing.

**This changes the order that gives a tile its holder.**[^2] It changes no
other decision of that record. The reach of a city, the stage of the rewrite,
the refusal of a build outside the builder's own ground and the settler verb
all stand.[^3]

A ground that admits no unit carries no unit, so no lease ever forms on it by
D2. Water is therefore never held by this test, and only the closure of D6
gives a faction ground that admits nobody.

A reviewer finds a violation when the rewrite tests the city before the lease,
when a tile whose count is below the claim threshold is held by its lease
faction, or when a lease at the claim threshold loses to a nearer city.

### D6. A fixed number of closure passes gives an unheld tile to the one faction that rings it

After every tile has a holder from D5, the rewrite runs a fixed number of
closure passes.

In one pass, the rule reads each tile that no faction holds. It counts the
neighbours of that tile that one faction holds. When no neighbour names another
faction, and the count reaches the neighbour threshold, that faction holds the
tile. A neighbour outside the world counts as no neighbour, so a tile at the
edge of the world reaches a lower count than a tile inside it. A tile whose
neighbours name two factions stays unheld.

Each pass reads the column the previous pass wrote, and writes a new column.
Two threads therefore write disjoint outputs, and no result depends on the
order in which a pass visits the tiles.[^16]

**The pass count is fixed, and no rule ends the passes early.** A rule that
repeated the pass until nothing changed would be a convergence test, and this
project refuses one wherever a result reaches the state.[^15] The pass count
and the neighbour threshold are balance values.[^12]

**Closure takes unheld ground only. It never takes ground another faction
holds.** A faction must not lose the ring around its own city because a
neighbour surrounded it, and a border that moves with no act would make both
the lease of D2 and the reach of a city pointless. Contested ground is what the
lease is for, and a faction that wants ground another faction holds walks on
it.

A reviewer finds a violation when a closure pass takes a tile that a faction
holds, when the pass count depends on how much changed, when a pass reads the
column it is writing, or when a tile outside the world counts as a neighbour.

### D7. The rewrite runs at the holding stage in one order, and the lease enters the state hash

The stage order is fixed. The rewrite computes the city reach for every tile,
then the lease of D5 overrides it, then the closure passes of D6 run over the
result. The whole rewrite runs at the holding stage of the step, after the
barrier of the frame and before the tile event stamps the holder.[^17]

**The lease is simulated state, so the state hash folds both of its columns.**
The step reads the lease on every tick and writes it on every tick. A value the
step reads and that the hash does not fold lets two worlds that differ in it
hash the same and then diverge, and this project has already recorded one
instance of that shape.[^18]

Each thread of the lease pass takes a contiguous run of tiles and writes only
the lease entries of that run. The join reads the runs in tile order. Nothing
reads which thread finished first.[^16]

A reviewer finds a violation when the lease stands outside the state hash, when
two threads write one lease entry, when the closure runs before the lease, or
when the stage runs before the barrier of the frame.

## The alternatives this rejects

**A count for each faction on each tile.** Every faction would gain ground on
one tile at once, and a tile would name the faction that used it most over the
whole run. Rejected because the storage would be the tile count multiplied by
the faction ceiling, and the storage rule refuses a field indexed by the
faction.[^7] One lease with one count holds the same border in two values.

**Hold ground by presence at one tick.** Rejected because that is the rule the
project superseded. A holding then has no centre, cannot be lost and cannot be
chosen, and its cost follows the world at the target population.[^4]

**Grow the reach of the city instead of the ground.** A faction that used
ground far away would gain reach at its city. Rejected because a reach is a
disc and a path is not. A faction that walked one road would gain a disc it
never visited.

**Derive the lease at the end of the step, as the presence relation is
derived.**[^8] Rejected because use accumulates over ticks. One step holds no
record of the step before it, so nothing exists to derive the lease from.

**Give the lease no decay.** Rejected because a path walked once would then
stay held for the rest of the run. The owner asked that repeated use move the
border, and one use is not repeated use.

**Close a hole by repeating a pass until nothing changes.** Rejected because a
fixpoint loop is a convergence test, and the run must give one answer at any
thread count.[^15] A fixed pass count also bounds the cost of the stage.

**Let closure take ground another faction holds.** Rejected under D6. A faction
would lose the ring around its own city with no act by anybody.

## Consequences

**A faction may lose ground without losing a fight.** Units of another faction
stand on a tile for enough ticks, and the lease turns. No contest fires and no
casualty occurs. A watcher sees the holder change and reads no cause.

**A single unit that walks one path long enough claims it.** The claim
threshold, the raise step and the bound decide how long, and the decay decides
how quickly the path returns. Those values are the whole of the bound on this,
and one blocker governs each of them.[^9]

**A faction may hold ground that no city of it reaches.** That is the point of
this record. It changes what the territory reader counts: the running total now
counts leased ground and closed ground as well as city-held ground.[^19]

**The border between two factions may be a path rather than a line between two
discs.** A watcher sees a line of colour along a road, and a disc around each
city.

**A faction erodes a border only where the two factions are at peace.** The
movement rule refuses a guest onto ground the holder is below a stated edge
toward.[^20] A faction at war with the holder cannot reach that ground, so it
cannot raise a lease on it. Ground that nobody holds is open to anybody.

**A lake or a mountain range that one faction rings becomes that faction's.**
Closure reads only the holder of a neighbour, and it asks nothing of the
ground. **This changes the water rule of the record it amends**, which says
that no faction holds water.[^2] A faction now holds the water it has closed,
and it holds no water it has not. The pass count bounds how far a hole is
swallowed, so a wide sea stays unheld in the middle.

**Traded land keeps its new holder for longer than one step in one case.** A
land contract writes the holder of a set of tiles to the creditor.[^21] The
next rewrite reads no previous holder, so the tile keeps the creditor only
where a test of D5 or D6 gives it. A creditor whose units then stand on the
tile raises a lease, and the lease holds the tile with no city.

**Every golden file that holds a held tile moves.** The rule that writes the
holder column changed, and the lease enters the hash.

**The balance register gains the values of this record.** The raise step, the
lower step, the decay step, the decay period, the bound, the claim threshold,
the closure pass count and the neighbour threshold arrive with the work that
implements this record. Nothing here was measured, and one blocker governs
every cost figure in this project.[^22]

**One open question is reached more often.** Nobody has said whether an upgrade
changes hands when the ground under it does.[^23] Ground now changes hands with
no trade and no fight, so an upgrade meets that question on more tiles.

## References

[^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D1. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D2, D3, D4 and D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^4]: Findings register, FND-285. `docs/FINDINGS.md`
[^5]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^6]: The holding module of the core crate. `crates/cachette-core/src/holding.rs`
[^7]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^8]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D1 and D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^9]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: ADR-0083, the gate build checks every integer overflow. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
[^12]: Balance register. `docs/reference/balance.md`
[^13]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^14]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^15]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^16]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^17]: Findings register, FND-029 and FND-079. `docs/FINDINGS.md`
[^18]: Findings register, FND-480. `docs/FINDINGS.md`
[^19]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^20]: ADR-0146, a faction relation is one signed integer per ordered pair, decision D4. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^21]: ADR-0147, a contract consideration is a tagged kind, decision D3. `docs/adrs/accepted/adr-0147-a-contract-consideration-is-a-tagged-kind.md`
[^22]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^23]: Blockers register, BLK-036. `docs/BLOCKERS.md`
