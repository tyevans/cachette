# ADR-0182: The temperature a cell is driven toward is a published energy balance, and the poleward transport is imposed

## Context

The weather field carries a temperature for each cell and moves it toward a
value that the world asks of that cell. That asked value is the subject of this
record.

Two records built the asked value as it stands. One states that four terms
drive the temperature and that the sun is one of them.[^1] The other states
that the row axis of a world is a latitude, that the belt of a latitude comes
from the annual mean of the insolation, and that the season comes from the
daily value against that mean.[^2]

**The asked value carries three written-down numbers that no published source
chose.** The belt has an amplitude. The season has an amplitude. A normaliser
maps the sum of the two onto the swing that the heat scale reserves. Each is a
count of heat units, and the reasoning for each is internal: the three were
chosen so that the sum fits the scale.

Three consequences follow, and a measurement found all three.[^3] [^4]

**The season saturates.** The season is normalised so that it reaches its whole
amplitude at the middle latitude, and it is then clamped. The geometry gives a
larger anomaly at every latitude beyond that one, so the clamp binds over the
whole of the high latitudes. Every latitude from the middle to the pole
therefore receives one season, and the published seasonal range grows with the
latitude instead.

**The belt is the profile of the radiation alone.** The belt maps the annual
mean insolation onto its amplitude with a straight line. That is the shape a
planet holds when nothing moves heat across it. **A published energy balance
model carries a term that does move it**, and the profile that model settles on
is flatter than the radiation profile at every latitude away from the
equator.[^5]

**The two are locked together.** The measurement found that reducing the season
alone puts the temperate band where the published classification puts it, and
in the same run drops the polar summer below freezing, because the oversized
season was the only thing holding the polar warmest month above it.[^3] Neither
can be judged alone, so neither can be corrected alone.

**One earlier attempt failed, and its failure shapes this record.** A diffusion
was added to the carried temperature plane, which is where a published model
carries its transport term. It changed nothing at any affordable pass count,
because the driver relaxes each cell to a local value far faster than any
affordable diffusion can move heat between cells. **A diffusion cannot compete
with an assignment.**[^4] So the transport must appear in the value that is
assigned.

**A single-layer field cannot grow the eddies that carry the heat.** The
transport on a real planet is done in large part by baroclinic eddies, and a
field with no vertical structure has no baroclinic instability and therefore no
eddies.[^6] This project has already met that argument and accepted it once:
the banded circulation is imposed on the pressure for the same reason, because
the cell that produces the mid-latitude belt is eddy driven and cannot
emerge.[^5]

## Decision

### D1. The asked temperature is stated in degrees, and every term that a published source fixes takes its value from that source

The warmth scale of a cell is an affine map to a temperature in degrees, and
the field already declares that map. **Every term of the asked value is stated
in degrees and converted through that map**, rather than in heat units chosen
to fit the scale.

The belt amplitude and the normaliser that mapped the sum of the two terms onto
a reserved swing are removed. The published constants of the energy balance
replace them, and the reference table holds those constants.[^7]

**One written-down amplitude survives, and this record does not pretend
otherwise.** The season needs the heat capacity of the surface to fix its size,
the published model this record draws on states no value this project could
verify, and a first-order estimate did not reproduce the observed amplitude and
the observed phase lag together. **So the season amplitude stays a balance
value that a blocker governs**, and it is declared as one.[^8] Its shape is
derived; only its size is not.

**The reason for the rule is that a written-down swing has no test.** A
published constant can be wrong and a reader can check it against its source. An
amplitude chosen to fit a scale is correct by construction and says nothing. The
one amplitude that remains is now labelled as unmeasured instead of reading as
if it were chosen.

### D2. The belt is the equilibrium of a published diffusive energy balance model, and the transport term of that model is imposed

The belt of a latitude is the steady state of the published one-dimensional
energy balance: the absorbed solar radiation, less a linear function of the
temperature for the outgoing radiation, plus a diffusion of heat along the
latitude.[^5] The reference table holds the four constants and their
source.[^7]

**The equation has a closed solution and the engine takes it, not the
diffusion.** The absorbed radiation is very close to a constant plus one second
Legendre polynomial of the sine of the latitude, and the diffusion damps each
Legendre mode by a factor that depends only on its order. So the settled profile
is a constant plus a damped second Legendre polynomial, and that is arithmetic
over the latitude with no solver, no iteration and no neighbour read.

**The transport is imposed and it does not emerge. Say so plainly.** No cell
gives heat to its neighbour under this decision. The engine writes down the
profile that a transporting planet settles at. A reader who expects to find heat
moving between cells will not find it, and this clause exists so that the reader
stops looking.

**This is not a licence to impose the rest of the weather.** The rule is the one
the banded circulation already follows: impose a structure only when the field
structurally cannot produce it. A single layer cannot produce baroclinic eddies.
It can produce a front, a rain shadow and a storm, and those stay emergent.

### D3. The season is the insolation anomaly at the same latitude, normalised where the geometry peaks, so that no latitude clamps

The season of a cell is the daily mean insolation at its latitude against the
annual mean insolation at that same latitude.

**The normaliser is the largest anomaly the geometry holds anywhere**, which
stands at a pole. It is not the anomaly at a chosen latitude. The amplitude that
the normalised anomaly is multiplied by is the balance value of D1.

**The clamp goes.** Nothing clamps, because nothing can exceed the largest value
the geometry holds. The seasonal range then grows with the latitude, as the
geometry makes it grow, and no latitude receives another latitude's season.

**The choice of a middle latitude was the defect and not the amplitude.** A
normaliser at the middle latitude makes every latitude beyond it clamp, which
replaces the geometry with one number over the whole of the high latitudes.

## Consequences

**The project cannot tune the mean temperature profile by changing an amplitude,
because there is no longer an amplitude to change.** A change to that profile is
a change to one of the published constants, and that is a claim about the
physical model rather than about this world. The reference table is where such a
change is argued. **The seasonal amplitude stays tunable**, and that is the one
lever a designer keeps over the temperature.

**The equator-to-pole difference is now whatever the published constants give.**
It is no longer a free choice, and it may not be the difference a designer wants.
If a world needs a different one, the lever is the latitude span that the world
states, which is already a parameter.[^2]

**The seasonal range at a high latitude is larger than at a middle latitude.**
Any content that assumed one season everywhere is wrong under this record.

**The albedo is fixed.** The published model this record takes its constants from
holds the albedo constant, so it carries no ice feedback. It is therefore known
to settle warmer at the poles than a planet with ice does. **The engine inherits
that bias**, and a later record that adds an albedo that reads the temperature
would supersede this clause rather than amend it.

**No term of the asked value reads a neighbour.** The belt, the season and the
transport are all functions of the latitude and the tick. The asked value stays
a pure function of the cell, which is what lets the driver pass run without a
stencil.

**One published quantity is still absent.** The model this record takes carries
no term for the land and the sea holding heat differently in the mean. The field
carries that separately, as a lag on how fast a cell follows its driver, and
that lag is untouched here.[^1]

## References

[^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decisions D1 and D2. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
[^2]: ADR-0177, the row axis of a world is a latitude that the world states, decisions D1, D3 and D5. `docs/adrs/draft/adr-0177-the-row-axis-of-a-world-is-a-latitude-that-the-world-states.md`
[^3]: Findings register, FND-601. `docs/FINDINGS.md`
[^4]: Findings register, FND-602. `docs/FINDINGS.md`
[^5]: Research report 30, the published atmospheric math, sections 4.4 and 5.3. `docs/research/reports/30-the-published-atmospheric-math.md`
[^6]: Research report 30, the published atmospheric math, section 5.3, on the eddy-driven cell. `docs/research/reports/30-the-published-atmospheric-math.md`
[^7]: Balance register, the energy balance constants. `docs/reference/balance.md`
[^8]: Blockers register, BLK-130. `docs/BLOCKERS.md`
