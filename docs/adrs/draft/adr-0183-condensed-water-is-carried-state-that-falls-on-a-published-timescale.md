# ADR-0183: Condensed water is carried state that falls on a published timescale, so a storm forms in one cell and rains in another

## Context

The weather field holds the water the air carries and the water standing on the
ground. An accepted record states that the air sheds what it cannot hold, and
that what it sheds arrives on the ground.[^1]

**It arrives on the ground of the cell that shed it, in the same solve.** So no
condensed drop ever moves. Where rain falls is decided entirely by where the air
first meets colder or higher ground, and the water has no opportunity to travel
between the two events.

Two research reports predicted this before it was measured. One states that a
cloud that condenses in one cell and rains in the same tick cannot travel, and
that a travelling storm is the thing the project asked to see.[^2] The other
gives the published mechanism: a linear theory of orographic precipitation
carries two time constants, one for the conversion of vapour into condensed
water and one for the fall of that water, and it distributes the rain downwind
of its source over the wind speed multiplied by that time.[^3]

**A measurement found the consequence.** Over land, the equatorial band of a
demonstration world holds about twenty-three times the water that the
subtropical band holds, where the published ratio is nearer four to ten. The
vapour the two bands carry differs by only about two to one, so the
amplification happens after the transport rather than in it.[^4] A blocker holds
the row.[^5]

**The engine has a plane for vapour and none for cloud.** A reader that paints
cloud derives it from the vapour held against the capacity of the cell, which is
a humidity and not a quantity of water. Nothing in the field stores condensed
water, so nothing can carry it.

## Decision

### D1. Condensed water is a plane, and it is neither vapour nor rain

The field holds a third water plane. Vapour is what the air holds invisibly and
it does not fall. **Cloud is what condensed out of the vapour**, and it falls
while the wind carries it. Ground water is what has fallen.

The water account has three stored terms and one sink. Every reader that checks
the account against the total raised must add the cloud to the air and the
ground.

**This record adds the plane and does not yet move the two readers onto it.**
The share of the sky a watcher sees, and the shading that cools a cell, are both
still derived from the vapour held against the capacity. That was the only
available reading when no plane existed. **It is now a second declaration site
for one quantity, which is the defect shape this project records most
often.**[^6]

The reason for leaving it is scope and not doubt. Moving the shading term onto
the plane changes the temperature of every cell, and this record is about where
water goes. **A record that resolved it here would be two claims.** The
consequences name it as open work rather than letting a reader believe the
plane is the only declaration.

### D2. The cloud rides the wind, on the same transport as the vapour

The cloud plane is carried by the same donor-cell transport that carries the
vapour, with the same shares and the same compile-time bound on the total a cell
sends.[^7] **That is the whole purpose of the plane.** A storm forms where the
air cools, travels while it falls, and wets ground that never cooled the air
that fed it.

The transport is exact. What one cell sends, the neighbour receives, and the
account balances at every moment.[^8]

### D3. The cloud falls at a fixed share, and the share comes from a published time constant

A fixed share of the cloud of a cell reaches the ground in each solve. **The
share is derived from the published fall time and the simulated time that one
tick carries**, and it is not a figure chosen to make a picture look right.[^3]

The share is fixed and the pass runs a fixed number of times. **Nothing tests
whether the field has settled**, because a convergence test would make the answer
depend on the arithmetic of one machine.[^9]

**The share sets a distance, and the distance is the thing that matters.** Water
travels the wind for as long as it takes to fall, so the published time constant
and the wind together give the length over which one source wets the ground. A
record that stated the share without stating that it is a length would invite a
later contributor to tune it against a picture.

## Consequences

**The world must be larger than a storm.** The published fall distance is the
wind multiplied by the time constant, and it is tens of kilometres. A world
smaller than that cannot show the behaviour this record buys, because every
source wets the whole of it. **A demonstration world at the tile pitch is such a
world**, and a measurement on one says nothing about this decision either
way.[^4]

**The rain of a cell is no longer a function of that cell.** A reader that
explained a wet cell by its own temperature and its own ground can no longer do
so, and a test that asserted the old relation will fail. That is the decision
working.

**A third plane costs memory and a transport pass.** The field carries one more
plane of the lattice and the transport runs over two planes rather than one.

**The account gains a term.** Any check of the form "what was raised equals what
the air and the ground hold plus what evaporated" is false under this record
until the cloud is added to it.

**Nothing here changes where water enters the world.** Evaporation is unchanged,
and the record that governs it stands except for the clause this one names.[^1]

**One quantity now has two declaration sites, and this record leaves it that
way.** The cloud plane holds condensed water, and the two readers that paint the
sky and cool the ground still derive a cloud from the vapour and the capacity.
The two will disagree, because the plane empties as it rains and the humidity
does not. **A later record must move both readers onto the plane**, and until it
does, a reader of this code must know that the sky a watcher sees is not the
water the field carries.[^6]

## References

[^1]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decision D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
[^2]: Research report 29, the field pattern worked through the weather, section 3.3. `docs/research/reports/29-the-field-pattern-worked-through-the-weather.md`
[^3]: Research report 30, the published atmospheric math, section 3. `docs/research/reports/30-the-published-atmospheric-math.md`
[^4]: Findings register, FND-604. `docs/FINDINGS.md`
[^5]: Blockers register, BLK-155. `docs/BLOCKERS.md`
[^6]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^7]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^8]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^9]: ADR-0001, one binary gives one answer at any thread count, decision D3. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
