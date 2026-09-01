# ADR-0062: Production and upkeep are rates attached to a site

## Context

A site is a settlement. It stands on one tile, it belongs to one faction, and
it holds one pooled store of commodities.[^1] Nothing fills that store today. A
unit takes an amount from a tile and carries what it took, and the amount stops
in the hands of the unit.[^2] The world has no way to turn work into a quantity
that a place keeps.

The engine must give the store a source and a drain. The choice is where to
attach them.

**The obvious place is the unit.** A soldier eats. A worker produces. Charging
each unit on each tick reads as the honest model, because it names the thing
that does the work. The target scale refuses it. The project simulates one
million units and a far smaller number of settlements, so a per-unit charge
makes the economy cost grow with the population. The project owner settled this
question. Consumption is pooled at a place, and it is not charged to each unit
on each tick.[^3]

**The second force is exactness.** An aggregate must combine in any order and
give one answer.[^4] A rate applied to a store must therefore be integer or
fixed-point arithmetic, because floating point addition is not
associative.[^5] It must also saturate rather than wrap: a wrap turns a large
holding into a large debt, and nothing fails when it does.

**The third force is that a store of zero is a real state.** The project has
already lost a real value to a type that could not represent it, and the
finding records what that cost.[^6] A site that holds nothing is not a site
that holds no store. A drain that takes more than a store holds must therefore
stop at zero and say what it could not take, rather than going below zero or
wrapping.

**The fourth force is decay bias.** The obvious way to scale a rate is a
fixed-point multiply, and that multiply truncates towards negative infinity. A
value above zero rounds towards zero. A value below zero rounds away from it
and sticks there for ever. The project has met this exact bias before.[^7] A
capacity cap is not a negative rate, and the same finding holds for a
drain.[^8]

**The fifth force is cadence.** The research report on the economy gives a
cadence for every part of the subsystem, and production and upkeep do not run
on every tick.[^9] The frame schedule is static and known before the frame
runs, and every system declares a period and a phase offset.[^10] An interval
that a kernel holds as a constant cannot take part in that schedule.

**One modifier source exists.** A later record reserves a number for an ordered
modifier pipeline that turns a base value into an effective one.[^11] That
record has no file, and nothing in the engine modifies a rate today. A record
that assumed the pipeline would state an intent as a fact, which is the failure
that section 4.6 of the record scope rule names.[^12] A backlog item that cites
a reserved row as if it were a decision has already cost this project once.[^13]

## Decision

**Production and upkeep are rates attached to a site. They are never charged to
a unit.**

### D1. A rate belongs to a site, and never to a unit

Each site carries a production rate and an upkeep rate for each commodity. A
pass over the sites applies both to the pooled store of the site.

The cost of the pass therefore follows the number of sites. It does not follow
the number of units that live at a site, and it does not follow the size of the
world. A site with ten thousand inhabitants costs the same as a site with ten.

The engine gives no way to charge upkeep to a unit on each tick. That path is
closed, not merely discouraged.

### D2. A rate is a non-negative fixed-point value, and upkeep is not a
negative production rate

Every rate is a Q16.16 fixed-point value at or above zero. The engine refuses a
rate below zero.

Production and upkeep stay two separate rates. The engine never merges them
into one signed net rate. Two reasons hold them apart, and each one is
sufficient.

The first is arithmetic. The multiply that scales a rate truncates towards
negative infinity. A rate above zero loses at most the fraction that the scale
cannot hold. A rate below zero gains a permanent downward bias instead.[^7]
Refusing a rate below zero puts that bias out of reach rather than documenting
it.

The second is reporting. A net rate cannot say which half a store failed to
pay. A site that earns four and owes six is not the same as a site that earns
nothing and owes two, and the mechanic that reads the store needs to tell them
apart.

### D3. Production runs before upkeep, the store saturates, and neither refusal
is silent

One application produces into the store, then spends from it. A site therefore
pays this bill from these earnings. The reverse order would make a site that
earns exactly what it owes insolvent on every application.

The store saturates at both ends. It never wraps.

Production that the store cannot hold is a **spill**. Upkeep that the store
cannot pay is a **shortfall**, and the store stops at zero rather than going
below it. The engine records both. Neither is dropped in silence, because a
quantity that disappears without a record breaks the conservation equality and
nothing fails when it does.

The equality that follows is exact: what a site held, plus what production put
in, minus what upkeep took, is what the site holds. A spill never went in and a
shortfall never came out, so neither enters that equality.

### D4. The interval is a parameter of the schedule, not a constant of a kernel

A rate applies on a period with a phase offset. Both are parameters that a
caller sets. No kernel holds either one as a constant.[^10]

The stored rate is what one tick earns. The engine multiplies it by the period
to get the amount of one application. Raising the period therefore changes how
often a store moves. It does not change how much the store moves over a span of
ticks.

A period of zero is refused. It names a rate that never applies, and a rate of
zero already says that.

### D5. The pass runs after every stage that moves a quantity, and it is not a
barrier

The pass runs once each frame, after every stage that moves a quantity in that
frame, and before the derived levels rebuild. A derived level must read the
store that the frame settled on.[^14]

The pass reads no derived structure and changes no structure. It is therefore
not a barrier and it does not need one. The position is stated against the
barrier of the frame on purpose: a stage whose order is stated only against the
stage beside it moves the next time somebody adds a stage.

### D6. The reduction over the sites is order-free, and the reported log is not

Each thread owns a contiguous span of the site slots and writes only inside
it.[^15] The ledger totals are 64-bit integers, so a reduction over them is
exactly associative and exactly commutative, and it needs no declared
order.[^4]

The log of sites that fell short is not order-free, because a concatenation
depends on the order it reads. It takes the slot order, which does not depend
on the thread count.[^16]

### D7. This record states a base rate, and no modifier pipeline exists

The rate that a site carries is the base rate, and it is also the effective
rate, because nothing modifies it.

**No ordered modifier pipeline exists, and this record does not create one.**
The registry reserves a number for that pipeline, and no file holds it.[^11]
Do not write it until a second source modifies a rate. With one source there is
no decision to record, so the first condition of the record scope test
fails.[^12] A pipeline that nothing invokes would also be a declared capability
with no caller, which is a defect shape this project already tracks.[^17]

When a second source appears, the pipeline record states how the sources
compose, and this record keeps its claim: the base rate belongs to the site.

## Consequences

The engine cannot charge upkeep to a unit on each tick. A mechanic that needs
per-unit consumption must express it as a rate at the site the unit belongs to,
or it must argue against this record.

The engine cannot express upkeep as a negative production rate. A caller that
tries gets a typed refusal.

A store cannot go below zero and cannot rise above its ceiling. A design that
needs debt must model the debt as its own quantity. A design that needs an
unbounded store must widen the store type, which is a change to the settlement
shape and not to this record.

Every consumer of the store must handle a shortfall. A site that cannot pay is
a normal state and not an error, so nothing may treat it as a failure to run.

The cost of the economy grows with the number of sites. Raising the site count
raises this cost directly, and the reference tables carry the derivation.[^18]
No measurement exists on the target platform, so every cost figure that governs
this decision is derived rather than measured.[^19]

The period is a parameter, so two worlds on different periods reach the same
total over a whole number of periods and differ part way through one. A test
that reads a store part way through a period must state which period it means.

## Alternatives rejected

**Charge each unit on each tick.** This is the model that reads as honest, and
it is what a contributor will reach for. It makes the economy cost grow with
the population, which the target scale of one million units cannot pay. The
project owner closed this question, and the three-tier split is what replaced
it: individual decay, pooled consumption, aggregate decisions.[^3]

**One signed net rate for each commodity.** Fewer columns, one addition instead
of two. Rejected for the two reasons in D2. The scaling multiply biases a value
below zero downward for ever, and a net rate cannot report which half a store
failed to pay.

**A store that may go below zero, as debt.** This would make a shortfall need
no report, because the store would carry it. Rejected because a store below
zero is a second mechanic wearing the clothes of the first. Debt has its own
rules about who owes whom and what happens when it is not paid, and none of
them belong in a store.

**Wrapping arithmetic.** Cheaper by one branch. Rejected because a wrap turns a
large holding into a large debt and nothing fails when it does. The research
report gives the same warning for the neighbouring kernel: a wrapping subtract
turns a small shortfall into full satisfaction, and that is the classic
underflow defect in this exact shape.[^9]

**Apply the rates on every tick.** Simpler, and it needs no schedule. Rejected
because the research report gives production and upkeep a period, and because a
kernel that holds its own interval cannot take part in the frame
schedule.[^9] [^10]

**Store the amount of one application rather than a per-tick rate.** This would
remove the scaling multiply. Rejected because it makes the period and the rate
one value in two places: changing the period would silently change what a site
earns over time, and nothing would fail.[^17]

**Write the modifier pipeline now.** Rejected. See D7.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D1. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^3]: Blockers register, BLK-008. `docs/BLOCKERS.md`
[^4]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: Findings register, FND-043. `docs/FINDINGS.md`
[^7]: Findings register, FND-012. `docs/FINDINGS.md`
[^8]: Findings register, FND-016. `docs/FINDINGS.md`
[^9]: Research report 15, needs, consumption and the input-output economy, sections 6.3 and 12.1. `docs/research/reports/15-needs-consumption-and-economy.md`
[^10]: ADR-0050, the frame schedule is static and known before the frame runs. `docs/adrs/REGISTRY.md`
[^11]: ADR-0055, an effective stat comes from an ordered modifier pipeline. `docs/adrs/REGISTRY.md`
[^12]: Decision Record Scope, sections 1 and 4.6. `.claude/rules/adr-scope.md`
[^13]: Findings register, FND-063. `docs/FINDINGS.md`
[^14]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^15]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^16]: ADR-0004, iteration order is explicit, decisions D2 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^17]: Recurring defect shapes, shapes 1 and 3. `.claude/rules/recurring-defects.md`
[^18]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^19]: Blockers register, BLK-007. `docs/BLOCKERS.md`
