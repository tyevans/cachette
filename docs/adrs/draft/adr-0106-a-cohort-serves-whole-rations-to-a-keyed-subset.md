# ADR-0106: A cohort serves whole rations to a keyed subset, never an equal share to everybody

## Context

A unit carries a need. The need falls at an interval and a pooled draw against
the store of the unit's site refills it. A cohort is the units of one faction
at one site, and it draws as one row.[^1]

The pass that feeds the units divides what the cohort received by its
headcount, so every unit of a cohort gains the same amount. Every unit of a
cohort also decays by the same rate and founds with the same need. **The units
of a cohort are therefore numerically identical, and they stay identical for
ever.**

A record already answers the cliff that a whole-group model has. It says that a
per-unit accumulator removes it, because a shortage then degrades before it
kills.[^2] That was measured and it is not what happens.

**The accumulator delays the cliff. It does not spread it.** Every unit of a
cohort holds the same accumulator value, because every input to it is the same,
so every unit crosses the death bound on one tick. A run of the demonstration
world for 4000 ticks holds one distinct deficit value per faction on every
tick, and two of its four factions lose all 64 of their units on a single tick
each.[^3]

The equilibrium is what makes it certain rather than unlucky. The decay
saturates at zero, so a unit's need settles at exactly the ration it is
granted. That number sits above the threshold or below it, and nothing in the
engine moves it afterwards. A cohort below the threshold accrues deficit at a
constant rate until every one of its units crosses the bound together.

**The population is the one variable that would fix it and it never changes.**
Supply is fixed and demand scales with the headcount, so a cohort that fed 46
of its 64 units would feed all 46 to full if the other 18 were gone. The engine
cannot reach that state, because it never removes 18. It removes 64.

The project already refuses a draw in two places, for one reason: a draw makes
the answer depend on a key that carries no meaning.[^4] [^5] Both refusals are
about a **tie between two options that the ground should decide**. This record
is about **which of several identical units eats**, and the ground has nothing
to say about that. There is no fact to defer to, so a draw is not hiding one.

## Decision

**A cohort serves whole rations to as many of its units as its share covers,
and the members it serves are named by a keyed rotation of the cohort, taken
again on each application.**

### D1. A cohort feeds a subset whole, and never everybody partly

The share a cohort received covers a whole number of full rations. The pass
serves that many units their whole ration and serves the rest nothing.

An equal split is what makes every unit identical, and identical units die
together. Serving a subset is what puts two different needs, and therefore two
different deficits, on two units of one cohort.

The parts still sum to the whole. What the store gave is what the units
received, and the remainder that covers no whole ration goes to one unit, in
the order D2 fixes.

### D2. The subset is the ordinals of a cohort, rotated by a keyed offset

Each unit holds an **ordinal**: its place inside its own cohort, in slot order.
The pass that rebuilds the cohort table already walks every unit and already
counts them, so the ordinal costs one write and no pass of its own. It is
derived again at every rebuild and carries nothing between frames.

A cohort draws one offset from the counter-based generator, keyed on the tuple
of the system, the frame, the cohort row and the draw index.[^6] A unit eats
when its ordinal, advanced by that offset and wrapped at the headcount, falls
below the served count.

**A rotation is a bijection on the ordinals of a cohort, and that is why it is
a rotation.** Exactly as many units fall below the served count as the share
covered, so what the store gave is what the units received. **A draw taken for
each unit on its own does not have this property**, and it was built and
measured before this: each unit then has an independent chance, the number that
eats varies around the count the store paid for, and a cohort whose share
covered one ration served two units on one application and none on another. The
store had paid for one. The findings register holds it.[^3]

**The offset is drawn again on each application.** The block of ordinals that
eats slides from one application to the next, so no unit is always first. A
fixed offset would feed the same units every time, and a cohort would hold a
caste rather than a shortage.

**An ordinal is not an identity, and this record does not make it one.** It is
the position of a unit among the live members of its cohort, so a death shifts
the ordinal of every unit after it. Two consequences follow. A unit's fortune
is not a property it carries, and neighbours in slot order share their fortune
within one application. Both are accepted: the first is what makes the ordinal
free to compute, and the second washes out across applications because the
offset moves.

### D3. The order of the served set never depends on a thread

The pass is a map over the unit slots. Each unit reads its own ordinal and the
offset of its own cohort, and writes its own need and its own deficit, so no
two threads write one value and the result is the same at any thread
count.[^8]

Nothing sorts, and nothing needs to. The ordinal comes from a walk in slot
order, which no thread count decides, and the offset comes from a key that
holds no thread.

The remainder that covers no whole ration is not order-free, because one unit
takes it. It goes to the one unit whose rotated place equals the served count,
which is a statement about the rotation and not about a walk.

### D4. Every value is an integer or a fixed-point value

The served count is a division of the share by the ration, both of which are
integers in the accumulator scale. The draw returns an unsigned integer. No
part of this rounds and no part of it is a floating point value.[^9]

## Consequences

**A shortage now kills part of a cohort and leaves the rest.** A cohort whose
supply covers a fraction of its demand loses units until its headcount falls to
what the supply carries, and the survivors are then fed to full. That is a new
behaviour of the engine and not a tuning of an old one.

**Which units die is a rotation, and no property of a unit decides it.** A unit
is not weaker, older or further from home. A design that wants any of those to
matter must state the key it wants and supersede D2.

**Neighbours in slot order eat together within one application.** A rotation
serves a contiguous block of ordinals. A design that wants the served set
scattered needs a bijection with more structure than a rotation, and it must
say what that costs for every unit of the population.

**The deficit of a unit is no longer a function of its site alone.** A reader
that predicted a unit's death from the ration of its cohort now predicts a
distribution. Any test that asserted an exact deficit for a unit of a short
cohort asserts against a draw.

**ADR-0063 D4 stands and its reasoning is amended.** The accumulator is still
the input that ends a unit, and it still degrades before it kills. What it does
not do on its own is spread the deaths, and this record supplies what does. The
findings register holds the correction.[^3]

**The state hash of every world moves.** The need and the deficit of every unit
reach the hash, and both change on the first application. The golden files move
with it.

**A cohort of one is unchanged.** One unit either eats or does not, and a
division by one and a draw below one give the same answer.

## Alternatives rejected

**Keep the equal split and accept the cliff.** It is what the engine does. It
is rejected because a whole faction leaving the world on one tick is not a
shortage, and because the record that governs consumption says the cliff is
already removed, so the engine and the record disagree.[^2] [^3]

**Give each unit its own draw, rather than rotating an ordinal.** It needs no
ordinal and no column, and it is what this record said before it was measured.
It is rejected because it does not conserve: each unit gets an independent
chance, so the number that eats varies around the count the store paid for, and
a cohort can serve more rations than its share covered. A model that creates
food when a shortage bites is a worse answer than the cliff it replaced.[^3]

**Give each unit a rank drawn once, at spawn.** It spreads the deaths and it is
cheaper by one draw for each application. It is rejected because the same units
starve in every shortage: the cohort holds a permanent caste, and a unit's
whole future is decided by one draw it took before the world ran.

**Jitter the equal share by a keyed offset.** It is the smallest change and it
keeps the shape of the pass. It is rejected because it does not reach the
outcome. Every unit of a cohort below the threshold stays below the threshold,
so every unit still accrues a deficit and still dies. It spreads the deaths
across a few ticks and removes the whole cohort all the same.

**Sort the cohort and serve from one end.** It gives an exact served set with
no draw. It is rejected on cost: it is a sort of the population on each
application, and the consumption pass runs for every unit of one million.

**Let the need fall below zero and carry the shortage there.** It removes the
need for a second accumulator. It is rejected because a need is not a conserved
quantity and a need below zero is a second mechanic wearing the clothes of the
first, which is the same reason a store may not go below zero.[^10]

## References

[^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^2]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D1. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^3]: Findings register, FND-318. `docs/FINDINGS.md`
[^4]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^5]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^6]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^8]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, the consequences. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
