# ADR-0177: The row axis of a world is a latitude that the world states

## Context

Cachette simulates a hex world. The engine holds weather as water over a
lattice of cells, and each cell covers a block of tiles. The wind is carried
state that a pressure difference accelerates, the water rides the wind by an
exact integer move, and water enters the air where it is hot and falls where
the air cools.[^1] [^2] [^3] The temperature of a cell is carried state that a
sun and the sky drive.[^4]

**The project held two readings of its own map, and they disagreed by about a
factor of sixty.** The scale register fixes a tile edge and a world extent that
make the map a few hundred kilometres across, which is about three degrees of
latitude.[^5] The weather read the row axis as a latitude that runs from one
pole to the other, swung a sun along it and gave that swing most of the
temperature scale.[^4] A research report found both readings, judged both
defensible, and said the project must choose.[^6]

**The two readings demand opposite models.** Inside three degrees of latitude
the annual mean energy from the sun changes by about four percent, which
produces no climate zones at all, and the climate then comes from the ground
and the sea alone. Across a whole globe the same geometry produces poles, a
banded circulation, a belt of deserts and an equatorial rain belt.[^6]

**The project owner ruled on 6 September 2026 that the map is a planet.** A
world with poles, trade winds, subtropical deserts and an equatorial rain belt
is the world the project wants. The tile edge is therefore a game unit and not
a measurement, and the scale register must say so.

Three forces pull against each other here.

**A reading welded into the code cannot be tried the other way.** A model that
reads the row as a pole-to-pole latitude, and states that nowhere, cannot be
turned into a region model without a rewrite. The report recommends taking the
latitude from a parameter, so that the two readings differ by one constant
rather than by a model.[^6]

**A banded circulation cannot emerge from this field.** The middle cell of the
three in each hemisphere is thermally indirect and eddy driven. A single-layer
field has no vertical structure, so it has no baroclinic eddies, so it never
grows that cell. Three belts also need three pressure extremes, and one
temperature profile that falls from the equator to a pole has two. The belts
therefore never appear on their own, whatever the field does.[^6]

**A distance from where the sun stands cannot hold a polar day.** The daily
energy that the top of the atmosphere receives has a published closed form. At
a solstice the pole receives about a third more daily energy than the equator,
and the curve is nearly flat from thirty degrees to the pole and rises at the
end. Any function of the distance from the sun peaks under the sun and falls
away from it, so no such function has that shape.[^6]

## Decision

### D1. A world states a centre latitude and a latitude span, and the weather reads a latitude and never a raw row

**The latitudes of a world are configuration.** A world states the latitude of
its middle row and the span from its first row to its last. A world that states
neither is a whole planet: the middle stands at the equator and the span runs
from pole to pole.

**No weather term reads a raw row.** Every term that varies with the latitude
takes the latitude, and the row reaches it through the span alone. So the same
code answers the planet reading and the region reading, and the two differ by
one constant.

**The span is refused when it does not fit on the globe.** A span wider than
the globe, a negative span, and a span whose end passes a pole are all caller
errors, and the constructor returns a typed refusal.

The row that a term reads is a row of the world and never a row of the whole
lattice. The lattice carries a margin of cells that no reader sees, so the two
are different, and a reader that takes one for the other shifts every belt by
the margin width.[^7]

**This changes ADR-0166 D2.** That decision reads the row of a cell as a
distance from where the sun stands on the row axis, and states that nothing
states a latitude band. The row is now a latitude. Every other decision of
ADR-0166 stands, including that the temperature is carried state, that four
terms drive it, that the third of them is a cycle in the tick, and that the
wind carries it.

### D2. The field imposes the banded circulation as an offset to the pressure, and the offset never reaches the temperature

**The pressure of a cell is its temperature less an offset that its latitude
gives.** The offset is a low at the equator, a high at thirty degrees, a low at
sixty and a high at each pole, which is the published order of the belts. It is
one cosine of six times the latitude, so both hemispheres carry the same shape
without a second statement of it.

**The offset reaches the wind and nothing else.** The capacity of the air, the
rain and the cloud all read the temperature that the sun and the ground gave,
and none of them reads the offset. A pressure belt is not a temperature.

**This is what makes a desert belt.** Air leaves a high, so the subtropics
diverge and the equator converges. The terrain still perturbs all of it, so the
circulation is imposed and the weather is still emergent. That is what a
limited-area weather model does: it takes the large scale from outside and
computes the small scale itself.[^6]

**A vertical layer is rejected.** It would multiply the whole stage by the
layer count, and it buys the same belts that one table over the row gives for
one load and one add.[^6]

The amplitude of the offset is a value that no measurement chose, and a blocker
holds the question of what the wind should be worth.[^8]

### D3. The sun term is the published insolation geometry, split into the belt of a latitude and the season around it

**The term reads a table of the published daily mean insolation, built once by
integer arithmetic from the sine table that the arithmetic module holds.**[^9]
The table is a pure function of the geometry and holds no world, no span and no
pitch. A world reads the slice of it that its own span chooses. The table is
never built with floating point, because the result enters simulated
state.[^10]

**Two parts make the term.** The annual mean of the geometry falls from the
equator to a pole, and that part is the belt of a latitude. The daily value
against that annual mean is the season, and that part reverses across the
equator.

**The two parts carry separate amplitudes.** The annual mean stands in
equilibrium with what the ground and the sea hold, and the season is damped by
the same heat. One amplitude for both would make a summer pole hotter than the
equator, which is what the top of the atmosphere receives and not what a
surface holds.

**The two normalisers are properties of the globe and not of the world.** They
are read at the equator, at a pole and at the middle latitude of the same
geometry, whatever span the world states. A normaliser read from the world
itself would stretch a four percent change across the whole scale, and a narrow
span would then hold a full set of climate zones inside three degrees.

**The obliquity is published and the rest of the orbit is dropped.** The
obliquity creates the season. The eccentricity moves the energy by a few
percent and points the same way all year, and the precession terms act over
tens of thousands of years against a season period of a few simulated days.[^6]

### D4. The capacity of the air is the published saturation curve, in a rational exponent and a table of eight entries

**The capacity of the air above a cell follows the published Magnus form of
Alduchov and Eskridge**, which is the recommended set for meteorology.[^6] The
earlier curve was an empirical stand-in: it rose as a fourth power of the
temperature as a share of the heat scale, and it was chosen before the
published curve was read.

**A fixed doubling width is refused.** The published doubling width runs from
about seven kelvin at the cold end of the range to about fourteen at the warm
end, and the best fixed doubling is wrong by a third at the cold end. That is
the end where a player expects tundra and ice.[^6]

**The integer form is a rational exponent and a small table.** The base-two
logarithm of the published curve is one multiply and one divide. The whole part
of that logarithm is a shift and is exact, so the error of the form does not
grow with the temperature range. Only the fraction reads a table, and the table
has eight entries with a linear interpolation. The worst error of the form
against the published curve is stated in the commit body, with the command that
measured it.

**The field declares one map from the warmth of a cell to a temperature.**
Every published curve needs a temperature, and the warmth of a cell is an
abstract count. The map is linear, it stands in one place, and every published
form reads it. Without it no published curve can be checked against anything.

**The curve over ice is dropped.** It matters below the cold end of the liquid
curve, and the liquid curve is clamped there instead until a player can see the
difference.[^6]

## Consequences

**The engine now has a temperature scale, and the field reads cold against
it.** The four heat terms were balanced so that they span the heat scale
exactly. The sun term can no longer reach the swing it reserves, because its
two parts never peak at one place at one moment: the belt peaks at the equator,
where the season is near nothing, and the season peaks at the middle latitudes,
where the belt is near nothing. So the top of the scale is out of reach and the
world runs cold against the temperature the scale declares. A blocker holds
where the base should stand.[^11]

**The ceiling of the air plane rose by more than an order of magnitude.** The
published curve puts the capacity of the hottest cell far above the capacity of
a temperate one. Every reader that painted cloud against the ceiling of the
plane must read the share of the sky instead, because a ramp against the
ceiling paints a cold sky blank. Every mark that was a share of the ceiling is
now a share of the capacity at the freezing point.

**A polar continental interior is a desert.** The published curve makes the air
of a cold cell hold very little, so the poleward transport rains out most of
what it carries. A test that asks about polar cloud must supply water at the
pole, because a landlocked pole holds no sky whatever the model does. The cloud
that the earlier curve gave a polar interior came from a floor of two drops
rather than from water that reached it.[^12]

**The engine cannot be a region model without stating a span.** Nothing forces
a caller to state one, and a world that states none is a planet. A region world
is one call away and it costs no separate code path.

**A latitude term now costs a table lookup rather than a distance.** The lookup
is one interpolation over the latitude and one over the declination, against
the one multiply and one smooth curve the earlier term used.

**The banded pressure costs one load and one add for each cell of each wind
pass**, against no vertical structure at all.

## References

[^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^2]: ADR-0161, water rides the wind, and every transfer is an exact integer move. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^3]: ADR-0162, water enters the air where it is hot, and it falls where the air cools. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
[^4]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decisions D1 and D2. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
[^5]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^6]: Research report 30, the published atmospheric math. `docs/research/reports/30-the-published-atmospheric-math.md`
[^7]: Findings register, FND-569. `docs/FINDINGS.md`
[^8]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^9]: The arithmetic module, the sine table. `crates/cachette-core/src/sim_math.rs`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: Blockers register, BLK-151. `docs/BLOCKERS.md`
[^12]: Findings register, FND-570. `docs/FINDINGS.md`
