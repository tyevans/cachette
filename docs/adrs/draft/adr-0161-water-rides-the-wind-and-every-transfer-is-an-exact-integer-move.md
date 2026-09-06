# ADR-0161: Water rides the wind, and every transfer is an exact integer move

## Context

The engine holds the water in the air as one quantity for each level 1 cell,
and it moves that water with a spread pass. The pass is a gather: a cell
computes what it keeps and what each neighbour hands it, and it writes only
itself.[^1] Every neighbour receives the same share, so the transfer has no
direction.

**A measurement shows the result.** One storm raised at the strength ceiling
reaches about a third of the lattice within one tick, holds its maximum at the
cell it was raised on for two ticks, and leaves nothing a watcher can find by
the twentieth tick. The maximum flattens where it stands and never
travels.[^2]

A companion record gives each cell a wind, which is carried state that the
pressure difference accelerates.[^3] This record says what the water does with
it.

Two properties must survive the change, and they are the hard part.

**The water must still balance exactly.** A product record asks that what
leaves one place arrive at another, with no loss to rounding and no gain.[^4]
The present rule holds that property because a giver and a receiver compute one
transfer from one input plane with one truncating division, so the two ends of
an edge agree by construction.[^1] A directional rule is harder, because the
quantity a cell sends is no longer the same in every direction, and several
cells may send into one.

**The answer must not depend on the thread count or on the order.** That is the
one property this project cannot recover once it is lost.[^5]

## Decision

### D1. The transport is a gather along the wind of the giver

**A cell computes, for each neighbour, the quantity that the neighbour sends it
along that neighbour's wind, and it adds those quantities to what it kept.** It
writes only itself. No cell writes another cell, so a parallel pass writes
disjoint output and needs no atomic operation.[^6]

The quantity that a giver sends in one direction rises with the part of that
giver's wind that points that way. A cell whose wind is still sends the same
quantity every way, so the present isotropic behaviour is the zero-wind case of
this rule rather than a separate rule.

**The receiver computes the giver's share with the same input and the same
truncating division that the giver would use.** Both ends therefore reach one
integer. The remainder stays with the giver. That is how the rule conserves
when several cells send into one: each edge is one integer that two cells agree
on, and the receiver adds each of them.

The cells are visited in ascending cell index and the neighbours in the fixed
direction order. Both orders are fixed, and the pass reads a settled plane and
writes a separate one.[^7] [^6]

**This changes ADR-0141 D1**, which states that a cell hands each neighbour the
same truncated share. Every other decision of that record stands unchanged.

The alternative is a push: each cell writes its share into its neighbours. It
is rejected because two threads then write one cell, which the disjoint-output
rule forbids, and because a sum of pushes in thread order is not a fixed
order.[^6]

### D2. The quantity a cell sends over one pass is strictly less than what it holds

**Whatever the wind, the sum of the quantities a cell sends over one pass is
below the quantity that cell holds.** The field therefore never falls below
zero, for any input, and no cell can send water it does not have.

This is what the speed ceiling of the wind is for.[^3] The share a cell sends
in one direction rises with the wind, so an unbounded wind would let a cell
promise more than it holds. The ceiling bounds the share, and this statement is
the reason the ceiling is not decoration.

**A pass carries water at most one cell.** A rule that carried water further in
one pass would have to read a cell that another pass step had already changed,
and the reach of a solve is then the pass count rather than the arithmetic.

### D3. The account still balances, and the check still reports it

**The running totals of ADR-0141 D2 are unchanged, and the invariant check
reads them as before.** Every drop that entered the air is in the air, on the
ground, or counted as evaporated. A transport that scaled the quantity rather
than moving it fails that check, and nothing else would report it.[^1]

The check is what makes this record enforceable. A directional rule has more
ways to lose a drop than a symmetric one, and the account is the only reader
that sees a lost drop at all.

## Consequences

**A front can now cross the map, and a watcher can follow it.** That is the
behaviour the project owner asked for, and it is what the present rule cannot
give at any value.

**The transport share must be smaller than the present one.** The measurement
shows one storm covering a third of the lattice in one tick under the share and
pass count that stand today.[^2] A directional transport at that rate would
carry a front off a small map before a watcher saw it. The share and the pass
count are values, and the balance register holds the rows.[^8]

**The pass now reads a second field.** It reads the wind as well as the air, so
the work of one pass grows. The cost follows the lattice and a fixed pass
count, as the present pass does. It does not follow the tile count and it does
not follow the number of units. That is a shape and not a figure, and one
blocker governs every cost figure in this project.[^9]

**The rule is harder to check by reading.** A symmetric share is one number and
a reader can see that it is safe. A directional share depends on the wind, so
the safety of D2 rests on the ceiling that another record states. A change to
either record can break the other, and a reviewer must read both.

Truncation still makes a small quantity stand still. A cell holding too little
sends nothing in any direction, so a trace of water stays where it is until
something larger moves it. That is exact rather than approximate, and it is the
price of an integer transfer.

## References

[^1]: ADR-0141, a weather pass moves water and never scales it, decisions D1 and D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^2]: Research report 26, the scale of the weather, sections 4 and 7. `docs/research/reports/26-the-scale-of-the-weather.md`
[^3]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decisions D1 and D3. `docs/adrs/draft/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^4]: PRD-0004, the world has weather that a watcher can read, what good looks like. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^5]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^6]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: Balance register, the weather section. `docs/reference/balance.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
