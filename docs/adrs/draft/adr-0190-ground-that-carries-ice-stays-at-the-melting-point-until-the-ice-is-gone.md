# ADR-0190: Ground that carries ice stays at the melting point until the ice is gone

## Context

The weather field carries a temperature for each cell and moves it, on each
pass, a fixed share of the way toward a value that the world asks of that
cell.[^1] An earlier record replaced the written-down terms of that asked value
with a published energy balance, and it stated the term it knew was still
missing.[^2]

**The polar summer of this world is far too warm, and the defect is
structural.** A measurement read the warmest month of the polar band at three
lattice resolutions. The reading did not move with the resolution, and it rose
toward the pole in both hemispheres alike, against a published reading that
falls. A defect that survives a resolution change and is symmetric about the
equator is a missing term and not a sampling artefact.[^3]

**The cause is named and it is a phase change.** The insolation anomaly peaks
at a pole, so the seasonal amplitude of this model grows poleward while the
annual mean flattens. Nothing stops the polar summer. On Earth the surface of a
polar cell stays near the melting point through its summer, because the energy
that arrives melts the ice that stands there rather than raising the
temperature of the ground under it. The latent heat of fusion of water is
large, and a surface pays it before it may warm.[^4]

**The field carries no ice, so it has nothing to melt.** The module already
reads an ice share from the temperature a cell carries, and the albedo term
uses that share.[^5] The reader holds no memory: it answers from the
temperature of the moment, so the ice of a cell appears and goes at no cost. **A
term that must remember what the winter built cannot come from the temperature
of the summer.**

**A diffusion of heat is not an alternative here, and the project has already
paid to learn it.** An earlier attempt moved heat between cells on the carried
plane. It changed nothing at any affordable pass count, because the driver
relaxes each cell to a local value far faster than a diffusion moves heat
between cells.[^6] A remedy for the polar summer must act on the pass that the
driver acts on.

## Decision

### D1. A cell carries a freeze bank, and the bank is state

**Each cell of the weather lattice carries one count**, which is the freezing
that this cell has taken and that its ice has not yet paid back. The count is
simulated state. It enters the state hash, because a world that loads a saved
bank and a world that recomputes one are different worlds.

**The bank could not be derived.** The alternative was to read the ice of a
cell from the temperature it carries, in the way the albedo term does. That
reader answers from the moment and holds no history, so a cell that spent a
winter frozen and a cell that reached the same temperature on this pass read
alike. The clamp is a memory of the winter, so it needs a memory.

### D2. The deposit is the depth below the melting point, and it costs the cell nothing

On each solve, a cell whose carried temperature stands below the melting point
of water adds to its bank the distance it stands below that point. **Nothing
holds the temperature back while the bank fills.**

**The freezing of a real surface does stall it, and this decision leaves that
out.** A water surface stays near its freezing point while it freezes over, and
the ice then radiates from its own top and cools freely. This field carries one
temperature for a cell, and that temperature is the surface. The stall is short
and the free cooling that follows is the whole of a polar winter, so the
decision takes the second and drops the first. **Say so plainly**, because a
reader who expects a plateau at each end will find only one.

**The deposit counts freezing over time. It is not a depth of ice.** The
published growth of ice goes as the square root of the freezing a surface has
taken, so the bank is the argument of that growth and not the growth itself.
This decision holds the relation straight rather than square, and the
consequences name what that costs.

### D3. The withdrawal reads the settled temperature, after every pass that moves it

**The clamp is its own pass and it runs last.** A cell that ends the solve above
the melting point gives back as much of that rise as its bank pays for, and each
unit given back takes, from the bank, the freezing that grew the ice which that
unit would have melted. **A cell stands no higher than the melting point while
any bank stands**, and it stands where it likes once the bank is empty.

**The clamp does not live inside one of the passes that move the temperature.**
Two passes move it: the driver relaxes a cell toward what the world asks, and
the wind then carries heat onto it from its neighbours. A clamp inside the first
one lets the second lift a cell over the melting point with its ice still
standing, and nothing pays for that rise. A test found the hole and the register
holds it.[^10]

**The pass runs once and it does not loop.** There is no melt loop, no
convergence test, and no trip count that depends on the values. One pass makes
one comparison and spends one withdrawal, and the count of passes that a solve
runs is fixed before the frame.[^7]

### D4. What one unit of melt costs is a balance value, and a blocker governs it

The deposit counts freezing over time and the withdrawal spends temperature, so
one constant converts between the two. **That constant is the latent heat of
fusion divided by the heat capacity of the surface, in the units of this
field**, and this project can verify neither of the two quantities.

**So the constant is declared a balance value under the blocker that already
governs the weather**, in the same way and for the same reason as the seasonal
amplitude.[^8] [^9] The balance register holds the value and how it was
reached. **It is not a published constant and this record does not dress it as
one.**

### D5. The bank is bounded, and the bound is a property of the declared scale

The bank stops at what one whole season period deposits when a cell stands the
whole of the heat scale below the melting point for the whole of that period.
That is arithmetic over two constants the module already declares, so this
decision states no new figure.

**The bound is what makes an ice cap possible, and it is not a tuning value.** A
cell whose winter deposits more than its summer withdraws carries a residue
into the next year. The residue grows, the summer of that cell is then held for
ever, and the bound is what stops the count from running away.

### D6. The zero of the term is the melting point of water

The record that fixed the asked value requires a term for a quantity that the
published constants already average over to be an anomaly about that average,
and it asks of every new term what its zero is and why.[^5]

**The zero of this term is the melting point of water.** That is a physical
constant and not a fitted mean. A cell that never reaches it never touches the
term, and the term takes no watts out of the balance. It changes when the
energy reaches the temperature of a cell, and not how much energy arrives.

**The term is one-sided in time, and that answers the second half of the
question.** The deposit costs nothing and the withdrawal costs, so a cell that
freezes each year holds a lower annual mean than the same cell without the
term. The published constants were fitted to a planet where this melt happens,
so that lowering is not a second copy of an effect the constants hold. It is
the seasonal path those constants averaged over, put back.

## Consequences

**The warmest month of a cell that carries a bank is the melting point.** Any
content that read a polar summer temperature above that point is wrong under
this record, and the classification of such a cell moves with it.

**The clamp binds over the whole high-latitude belt of this world, and not
only at the pole.** The two bands are nearly the same cell in this model: they
stand at nearly one annual mean, one coldest month and one warmest month. The
bank is a function of how far below the melting point a cell stands and for how
long, so two bands that are cold in the same way bank in the same way. **Whether
the balance value of D4 can separate them is not settled**, and the register
says what measurement settles it. **The cause is the flatness of the
belt, which the record this one extends already names**, and the register holds
the measurement.[^11] [^12] A worker who tunes the balance value against the
band below the pole is tuning this term against a defect in another one.

**A polar reading needs a spin-up of one whole season period.** The bank is
carried state that a winter fills, so a world read before its first full winter
holds a pole that never banked. A probe that settled long enough for the field
without this term may not settle long enough for the field with it.

**The spring of a middle-latitude cell is flattened.** A cell that freezes each
winter spends part of its spring at the melting point before it warms. The
effect is a real one. Its length here is whatever the balance value of D4
makes it, so a designer who wants a shorter spring changes that row.

**The coldest month of a cell does not move.** The deposit costs nothing, so
nothing holds a winter up. The boundary between the temperate class and the
continental class reads the coldest month, and this record leaves that boundary
alone.

**A cell that is cold and dry carries a bank anyway.** The field holds no
account of the water a cell has to freeze, and the albedo term already reads
ice from the temperature alone. So a cold desert pays for a melt it never
earned. A later record that gave the ice a mass in water would supersede this
clause.

**The bank goes as the freezing and not as its square root.** The published
growth of ice is the square root of the freezing a surface takes, so this
record over-rewards a long mild winter against a short sharp one. The relation
is monotone and the order between two cells is right. The ratio between them is
not.

**A reader who wants to know when the clamp runs must read the solve.** The
record states that it runs after every pass that moves the temperature, and it
does not state where the code puts it. A later solve that gains a third writer
of the temperature must run the clamp after that one too.

**One more plane enters the state hash.** The golden state hash changes, and
every reader that compares a saved world against a computed one must be built
again against this record.

**The temperature of a cell is no longer a function of the last pass and the
world alone.** It is a function of the bank as well, so two cells at one
temperature under one sky may move differently. A reader who debugs the driver
must read both planes.

## References

[^1]: ADR-0166, the temperature of a cell is carried state that a season and the sky drive, decisions D1 and D2. `docs/adrs/draft/adr-0166-the-temperature-of-a-cell-is-carried-state-that-a-season-and-the-sky-drive.md`
[^2]: ADR-0182, the temperature a cell is driven toward is a published energy balance. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
[^3]: Findings register, FND-619. `docs/FINDINGS.md`
[^4]: Research report 30, the published atmospheric math. `docs/research/reports/30-the-published-atmospheric-math.md`
[^5]: ADR-0182, the temperature a cell is driven toward is a published energy balance, decision D5. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
[^6]: Findings register, FND-602. `docs/FINDINGS.md`
[^7]: ADR-0005, a solver runs a fixed iteration count, never a convergence test, decisions D1 and D2. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^8]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^9]: Balance register, the weather values. `docs/reference/balance.md`
[^10]: Findings register, FND-630. `docs/FINDINGS.md`
[^11]: Findings register, FND-632. `docs/FINDINGS.md`
[^12]: ADR-0182, the temperature a cell is driven toward is a published energy balance, the consequences. `docs/adrs/draft/adr-0182-the-temperature-a-cell-is-driven-toward-is-a-published-energy-balance.md`
