# ADR-0055: A site derives its effective rate from the world at each application

## Context

A settlement in this engine holds a pooled store. A production rate fills that
store and an upkeep rate empties it. Both rates belong to the site, so the cost
of the pass follows the number of sites and never the number of units that live
there. An accepted record states that shape, and this record does not change
it.[^1]

That record also states plainly what it left undone. It says that the rate a
site carries is the base rate and is also the effective rate, because nothing
modifies it. It says that no ordered modifier pipeline exists, that the
registry reserves a number for one, and that nobody should write it until a
second source modifies a rate. It names the condition under which the pipeline
record is written, and it says that its own claim survives that record.[^2]

**A second source now modifies a rate, and this record is the one that record
asked for.**

Two measurements made the choice unavoidable, and each named one half of it.

**A site sets its production rate exactly once.** The founding path reads the
food that the survey measured and writes the rate. Nothing reads the ground
again. A site founded on good land therefore produces the same amount on the
last tick of a run as on the first, whatever happened to the land, the weather,
the upgrades or the people. The gatherers of that site strip the ground around
it and the store does not notice.

**The upkeep table is inert.** The table exists and the rate pass spends from
it, and nothing outside a test ever writes a rate into it. Every site owes
zero. The store therefore has one sink, which is the ration that the
consumption pass draws, and that sink is bounded by the residents, which are
bounded by the housing. Production is bounded by nothing. A constant source
stands above a bounded sink, so every store rises in a straight line and stops
at the saturation point of its type. A register holds the curve.[^3]

Three forces pull against each other.

**A rate that the world does not reach is not a simulation.** The whole point
of a world model is that what happens to the ground changes what the ground
gives. A rate fixed at founding makes the ground scenery.

**A rate that the world writes into storage loses its own history.** A pass
that reads the world and overwrites the stored rate destroys what the caller
set. A caller that sets a rate must still find it there.

**A modifier is cheap to add and expensive to compose.** One source that scales
a rate is a multiplication. Four sources that each scale a rate are four
multiplications, and in fixed-point arithmetic each of them truncates. The
order then matters, and the reader cannot hold the result in mind.

## Decision

### D1. The effective rate is derived at each application, and the stored rate stays the base

**The rate pass reads the stored rate, derives an effective rate from it and
from the world, and spends the effective rate. It never writes back.** The
stored rate is the base rate, it stays on the site, and a caller that sets it
finds it unchanged. The founding rule that writes a rate at founding is
unchanged, and the accepted record that puts the rate on the site keeps its
claim.[^1]

Nothing stores the effective rate as simulated state. It is scratch, and it is
computed again on every application.

A reviewer finds a violation when a pass writes a derived rate into the stored
rate, or when a reader of the stored rate answers with a derived value.

### D2. Every term is a share of one, the terms add, and the pipeline multiplies once

**The pipeline builds one scale from a base of one, adds each term that gives
and takes away each term that takes, clamps the sum, and multiplies the base
rate by that scale once.** The terms are shares. They are not factors.

Addition in the project fixed-point scale is exact and saturating, so terms
that add carry no truncation at all. Factors that multiply truncate once each,
and the truncations compound: four factors that each move by a quarter reach
more than twice their span together, which no reader can predict from the four
values. The pipeline therefore multiplies exactly once, at the end, and both
operands of that multiply are at or above zero, which is the direction where
the truncation bias is benign.[^4]

**This is the composition rule that the reserving record asked for.** A source
that wants to change a rate writes a term. It does not write a second
multiply.

### D3. An upkeep term is a rate above zero that subtracts, and never a negative production rate

The upkeep terms compose in the same way as the production terms, and the
result is an upkeep. **No term may express itself as a production rate below
zero.** The accepted record gives two reasons and both still hold: the scaling
multiply truncates toward negative infinity, so a rate below zero carries a
permanent downward bias that a rate above zero does not, and a single net rate
cannot say which half a store could not pay, so a shortfall would have nowhere
to come from.[^5]

The derived upkeep adds to the stored base upkeep and never takes any away. A
caller that set an upkeep still pays at least that much.

### D4. The derived rate stays out of the state hash, and every input to it is in the hash

**The effective rate is derived, so it adds no line to the state hash.** Every
input it reads is stored and is already hashed: the ground, the weather, the
standing upgrades, the residents and the stored rates. The weights are
compile-time constants. Nothing stores them and no verb writes them, so they
add no line either.[^6]

This is the rule the hash record states, read in the direction it is usually
read backwards. A value the step reads must be in the hash when it is stored.
A value the step derives again from hashed inputs must not be, because a
derived line in the hash is one fact in two places and nothing fails when the
two disagree.[^7]

### D5. The pipeline visits the world in a fixed order, composes its terms in a fixed order, and draws nothing

**The pipeline walks the ground of a site in the fixed disc order, and it
composes its terms in the order this record states.**[^8] It makes no random
draw, so it needs no key.[^9] It runs to a fixed shape and tests no condition
to decide when to stop.[^10] No value in it is a floating point number and
every operation goes through the arithmetic module.[^11]

Two sites are independent, so the pass keeps the parallel shape the accepted
record already gives it, and the reduction over the sites stays order-free.[^12]

### D6. This record states the shape of a term, and never the worth of one

**No weight appears here.** Which terms exist is this decision. What each one
is worth is a rule of the downstream game, and the balance register holds one
row for each with its derivation.[^13] A blocker governs every one of those
values, and none of them is measured.[^14]

A term that a measurement has not reached is not written. The production
pipeline holds no temperature term for that reason: the field exists and its
range is not taken, so a weight written against it today would be an invented
value.[^15]

## Consequences

**The project cannot answer what a site produces without reading the world.** A
reader that wants the effective rate asks the engine for it. A caller that
reads the stored rate gets the base, and the two answer differently. Both
readers are public, and a document that names one must say which.

**A store now settles instead of climbing.** The holding term makes what a site
holds cost a share of itself each tick, so a store settles where its net
production and that cost balance. A site that earns more settles higher, and a
site whose ground is drawn down falls back. The register holds the share and
the relaxation it implies.[^13]

**Every test that sets a production rate, steps, and asserts a store quantity
now reads a different number.** A non-zero upkeep exists where none did. That
is fallout of switching on a sink, not a defect of the sink.

**A caller may still take the whole store in one application.** A period above
the relaxation time takes more than the store holds. The pass stops at zero and
logs a shortfall, so the outcome stays defined, and the accepted record already
requires every consumer of the store to handle a shortfall.[^16]

**A second commodity needs a decision that nobody has made.** Both upkeep terms
apply to every commodity, and the engine holds one commodity, so this record
states no rule about which good a person needs.

**The reserving record keeps its claim and loses one sentence.** The rate
belongs to the site, and that is unchanged. The clause saying that no ordered
modifier pipeline exists states the condition of the registry rather than the
reasoning of that record, and the registry has moved.[^17]

## References

[^1]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^2]: ADR-0062, production and upkeep are rates attached to a site, decision D7. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^3]: Findings register, FND-543. `docs/FINDINGS.md`
[^4]: Findings register, FND-012. `docs/FINDINGS.md`
[^5]: ADR-0062, production and upkeep are rates attached to a site, decision D2. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^6]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^7]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^10]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^11]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^12]: ADR-0062, production and upkeep are rates attached to a site, decision D6. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^13]: Balance register, the effective rate. `docs/reference/balance.md`
[^14]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^15]: Backlog item 0512, add the temperature term to the production pipeline. `docs/backlog/proposed/0512-add-the-temperature-term-to-the-production-pipeline.md`
[^16]: ADR-0062, production and upkeep are rates attached to a site, decision D3. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^17]: ADR Registry, repairing a fact a record imported from a register. `docs/adrs/REGISTRY.md`
