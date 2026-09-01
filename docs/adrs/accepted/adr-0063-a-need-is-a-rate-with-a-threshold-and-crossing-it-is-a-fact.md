# ADR-0063: A need is a rate with a threshold, and crossing it is a fact

## Context

A unit exists and costs nothing. A pile of food and no food are the same to the
unit that stands beside them, so a surplus means nothing and a shortage cannot
happen. Nothing in the engine takes a quantity back out of the world for a
reason.

A site holds a pooled store, and rates fill it and empty it.[^1] [^2] The store
now has a source and a drain that belong to the place. It has no drain that
belongs to the people. This record adds that drain.

**The first force is the target scale.** The project simulates one million
units, and every one of them consumes.[^3] A rule that charges each unit
against a shared store makes every unit in a place a writer to one location.
The cost is the contention, not the arithmetic. That is a shape argument, and
it holds whatever the cost figures turn out to be.[^4]

**The second force is that a whole-group model has a cliff.** A group that
draws as one is fine a little above its demand and starves entirely a little
below it. The research report states the cliff and states the remedy: keep a
per-unit accumulator, so a shortage degrades before it kills.[^5]

**The third force is that the project has already settled where consumption
sits.** The split is individual decay, pooled consumption, aggregate
decisions.[^6] The research report derives the same split from cost and reaches
the same place.[^5] This record must honour that split rather than reopen it.

**The fourth force is exactness.** An aggregate must combine in any order and
give one answer.[^7] Every quantity here is an integer or a fixed-point value,
and the arithmetic saturates rather than wraps.[^8] A wrapping subtract in this
kernel turns a small shortfall into a full satisfaction, which is the classic
underflow defect in this exact shape.[^5]

**The fifth force is that a shortage must be visible.** A quantity that leaves
a store without a record breaks the conservation equality, and nothing fails
when it does. The engine must therefore say what a store could not give, in an
order that no thread count decides.[^9]

**A unit already carries a load.** It takes an amount from a tile and carries
what it took. A reader could reasonably decide that a unit eats what it
carries. The settlement is the shape that holds pooled stores, and no other
shape does, so the draw goes against the store of a site.[^1] This record
states that rather than deciding it again.

## Decision

**A need is a per-unit rate with a threshold. A unit draws against the store of
its site as one cohort, and crossing the threshold is a fact that
accumulates.**

### D1. A need belongs to the unit, and it falls by a saturating subtract

Each unit carries a need. The need falls at an interval by a subtract that
saturates at zero. The engine never wraps it, and it never lets it fall below
zero.

The need stays on the unit even though the draw is pooled. That is what removes
the cliff of a whole-group model: a unit that is fed part of its ration holds
part of its need, and the difference between a place that is short and a place
that is empty is then visible.[^5]

A need is not a conserved quantity. Nothing flows into it and nothing flows out
of it, so the clamp at the top of a need is safe. The commodity that satisfies
the need is conserved, and it is conserved at the store. A cap is not a
negative rate, and the same finding holds here.[^10]

The rate and the threshold are content. The engine holds them as parameters and
never as a constant of a kernel. The interval is a parameter of the frame
schedule, in the same way as every other rate of the economy.[^2]

### D2. A unit draws from the store of its site, and it draws as a cohort

A unit never draws against a store on its own. A **cohort** is one row that
stands for the units of one faction that belong to one site. The row holds a
headcount and never a list of identities, because an identity lives in the
arena that minted it.[^11]

The draw is one segmented reduction over the cohort rows of a site, then one
capped transfer out of the store of that site. It never loops over units, it
holds no lock, no atomic and no retry.

The cohort array is indexed by the slot of the site, so it is sorted by site by
construction and the reduction needs no sort. A segment is therefore contiguous,
and the partition gives a whole segment to one thread.[^12]

**A cohort is keyed on the faction as well as the site.** Two factions in one
place never pool their draw. A pooled draw would feed a rival out of a store it
does not hold, and a faction is the mask that says who holds what.[^13]

**The cohort table is derived from the unit columns, and the engine derives it
again rather than carrying it.** A table that a caller maintained beside the
columns would be one fact in two places, with nothing that fails when the
copies disagree.

### D3. A store that cannot serve every cohort splits what it has, and the
split is exact

Each cohort takes the truncated proportion of what it asked for. The remainder
goes one unit at a time to the cohorts in ascending row order.

The parts sum to the whole. No unit is lost and none is created. The store
falls by what the cohorts received, and never by what the transfer meant to
give: the two are the same number while the split is exact, and a split that
lost a unit would otherwise take that unit out of the world in silence.

The engine records what a store could not give. A shortage is a normal state
and not a failure to run, so nothing may treat it as an error.

### D4. Crossing the threshold accumulates, and this record stops at the
accumulator

A unit whose need is below the threshold adds the shortfall to a deficit
accumulator. A unit at or above the threshold takes a recovery rate off it.
Both moves saturate, and the accumulator never falls below zero.

The accumulator is the input that a later rule reads to end a unit. **This
record states the accumulator and states no consequence of it.** A rule that
ends a unit is a separate claim about how a unit ends, and it belongs to the
record that holds that claim. An accumulator with no reader is a capability
that nothing invokes, so the rule that reads it follows this work rather than
waiting on it.

### D5. The pass runs after every stage that moves a quantity, and it is not a
barrier

The pass runs on the schedule of the economy and after the rates of the frame.
The rates are what filled the store that the draw spends, so a draw before them
would spend the store of the frame before. The pass runs before the derived
levels rebuild, because a derived level must read the store that the frame
settled on.[^14]

The pass reads no derived structure and changes no structure. It is therefore
not a barrier and it does not need one.

### D6. The reduction is order-free, and the reported log is not

Each thread owns a contiguous span of the site slots and writes only inside
it.[^12] The ledger totals are integer accumulators, so a reduction over them is
exactly associative and exactly commutative, and it needs no declared
order.[^7]

The log of sites that could not serve is not order-free, because a
concatenation depends on the order it reads. It takes the slot order, which
does not depend on the thread count.[^15]

## Consequences

The engine cannot charge a unit against a store one unit at a time. A mechanic
that needs a per-unit transaction must express it as a cohort, or it must argue
against this record.

A unit cannot eat what it carries. What a unit carries is a load it took from a
tile, and the draw reads the store of a site. A design that wants a unit to live
off its own load must say how that load reaches a store.

Two factions in one place hold two cohorts, so a site holds one row for each
faction of the mask whether or not a faction stands there. That is the price of
an index that needs no sort.

A unit that belongs to no site draws from nothing. Its need therefore falls to
zero and its deficit accumulator rises. Any rule that gives a unit a site must
say what happens to a unit that has none.

The need of a unit is state that a later frame reads, so it reaches the state
hash. A change to the need rule changes the hash of every world, and the golden
files move with it.

A store cannot go below zero. A design that needs debt must model the debt as
its own quantity.

Every cost figure that governs this decision is derived rather than measured,
because no measurement exists on the target platform.[^4]

## Alternatives rejected

**Charge each unit against the store as it eats.** This is the model that reads
as honest, and it is what a contributor will reach for. Rejected because every
unit in a place then writes to one location. The project owner already closed
this question, and the three-tier split is what replaced it.[^6]

**Fold the need into the cohort row and drop the per-unit need.** Fewer values,
one row for a whole population. Rejected because a whole-group need has the
cliff: a place is fine a little above its demand and starves entirely a little
below it. The per-unit accumulator removes the cliff at no extra row.[^5]

**Key a cohort on the site alone.** One row for each site, and the smallest
possible table. Rejected because two factions in one place would pool their
draw, and because a site with one cohort has nothing to split, which would make
the exactness rule of D3 a capability that nothing invokes.

**Give the remainder of a split to nobody.** Simpler by one loop. Rejected
because the parts would then sum to less than the whole, and the difference
would sit in the store while the engine reported that the store had given it.

**Let a store go below zero and carry the shortage as debt.** Rejected because
a store below zero is a second mechanic wearing the clothes of the first. Debt
has its own rules about who owes whom, and none of them belong in a store.

**Wrapping arithmetic.** Cheaper by one branch. Rejected because a wrapping
subtract turns a small shortfall into full satisfaction, and nothing fails when
it does.[^5]

**End a unit in this record.** Rejected. The accumulator is the input to that
rule, and how a unit ends is a separate claim.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0062, production and upkeep are rates attached to a site, decision D4. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^3]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Research report 15, needs, consumption and the input-output economy, sections 5.4, 6.3 and 6.4. `docs/research/reports/15-needs-consumption-and-economy.md`
[^6]: Blockers register, BLK-008. `docs/BLOCKERS.md`
[^7]: ADR-0023, an aggregate combines exactly, in any order, decision D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^8]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: Findings register, FND-016. `docs/FINDINGS.md`
[^11]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^12]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^13]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^14]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^15]: ADR-0004, iteration order is explicit, decisions D2 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
