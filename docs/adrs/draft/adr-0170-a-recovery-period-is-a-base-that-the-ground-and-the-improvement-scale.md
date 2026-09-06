# ADR-0170: A recovery period is a base for the kind that the ground and the improvement scale

## Context

A tile in this engine carries a generated stock of a resource kind. The stock
itself is never stored. Only what a unit took from the tile is stored, as an
entry in a ledger, and the tile answers with the generated stock less the
stored take.[^1]

Recovery therefore ages the stored take rather than growing an amount. An
accepted record states that shape and it is unchanged by this record: recovery
never raises the stock of a tile, because no stock is stored to raise; the cost
of recovery follows the depleted set and never the tile count; the arithmetic
is exact and whole in the order the ledger holds; and recovery runs before the
gather resolve of the same frame.[^2]

That record also decided how fast a deposit comes back. It says that each kind
states one period, or states that it does not recover, and that a period is
stated in simulated time and converted to ticks in one place. It argues that a
period stated in ticks alone would state something false the moment the span of
a tick moved.[^3]

**Two measurements made both halves of that clause untenable.**

**The recovery was far weaker than the gather.** A single unit took several
units of stock from a tile in one tick, and a food tile gave one unit back
every several hundred ticks. Over a long run a food tile regained a small
fraction of what one tile of gatherers removed in a few ticks. The ground near
a settlement was bare within a few hundred ticks and stayed bare, so a faction
gathered nothing after that and a run went quiet. A product record names that
behaviour as one of the two reasons a run stops being eventful.[^4]

**The owner then reported the opposite defect on unimproved ground.** Food came
back fast enough that foraging was a primary activity, where it should be a
last resort. The two readings are not in conflict. They say that one number for
a whole kind cannot be right, because the same period must be slow on raw
ground and fast on ground that somebody worked.

Three forces pull against each other.

**A period for each kind is one declaration and it is easy to reason about.**
That was the whole strength of the earlier clause, and a second declaration
site with no check between the copies is the defect shape this project meets
most often.[^5]

**A period that the world does not reach makes the ground scenery.** If wet
ground and dry ground give back at one rate, and worked ground and raw ground
give back at one rate, then nothing a faction does to the land changes what the
land returns.

**A rate below one unit a day cannot be stated in units for each day.** The
earlier form expressed a rate as a count of units in a simulated day. The rate
the owner asked for is below one unit a day, and the only value of that count
that reaches it is zero.

## Decision

### D1. A kind states a base period in ticks, and the ground and the improvement scale it

**The declaration for a kind holds one base period, and the period that the
engine applies to a tile is that base scaled by two readings of the world.**
The rule stays inside one declaration. A caller replaces the whole rule set
rather than one value, so no second site holds a period and nothing can
disagree with the first.[^3] That constraint of the earlier record is unchanged
and it is the part worth keeping.

The two readings are the moisture of the ground under the tile and the
improvement standing on it. Neither is stored as part of the rule.

**A kind may still state no period, and then it never recovers.** The absent
case is a real case and the rule set carries it.

### D2. The base period is stated in ticks, and not in units for each simulated day

**The declaration holds a period in ticks.** This reverses the clause of the
accepted record that requires a period in simulated time.[^3]

The reason is arithmetic and not preference. A rate expressed as a count of
units in a simulated day is a whole number, so the slowest rate it can express
above zero is one unit a day. The rate this project needs is slower than that,
and the only remaining value is zero, which means never. A form that cannot
express the value the project chose is the wrong form.

**The argument the earlier record gave is answered rather than ignored.** It
said that a period in ticks states something false the moment the span of a
tick moves. That is true, and it is now a cost the project accepts, because the
alternative cannot express the rate at all. A change to the span of a tick is a
change to every period, and the periods live in one declaration, so it is one
edit and not a sweep.

### D3. The moisture scaling is a table of bands, and the bands come from a measurement of the field

**The moisture reading indexes a table of bands, and each band holds a scale on
the period.** The table is indexed by the water on the ground of the weather
cell that covers the tile, through the reader that the gather resolve already
takes, so the project holds one moisture reader and not two.[^6] [^5]

**The scale is not monotone, and it must not be.** A kind may peak in a middle
band and suffer at both ends, because ground can be too dry and too wet for the
same thing to grow on it. A rule that only rewarded water could not express
that, and it is the shape the owner asked for.

**The band edges come from a measurement of the field and never from even
steps.** The distribution of ground water is heavily weighted toward the dry
end, so bands of equal width put almost every cell in one band and the table
would then do nothing. The register holds the reading and the band table.[^7]

A kind that states no period reads no entry of its row, so an absent kind
carries no band table that anything could disagree with.

### D4. An improvement divides the period, and the condition of the improvement scales the speedup

**A standing improvement divides the period, by a value the upgrade row
holds.** The column is a column of the upgrade table, so a category that should
speed recovery states it in the same place that every other effect of that
category is stated, and no pass branches on a category.[^8]

**The speedup is scaled by the condition of the entry, and it reaches nothing
exactly at no condition.** A neglected improvement therefore falls back toward
the unimproved rate continuously rather than at a threshold. This is what ties
the recovery to the upgrade condition, and a separate record states that
condition.[^9]

### D5. The rule answers one tile, and one function answers for the pass and for a reader

**The engine holds one function that answers the recovery period of one tile,
and both the ageing pass and a public reader call it.** A display that computed
the period a second way would be that rule in two places, and nothing would
fail when the two disagreed.[^5]

The recovery keeps every determinism property of the earlier record. The
arithmetic is exact and whole, the walk is in the order the ledger holds, and
the pass makes no random draw.[^10] [^11] The reading of the weather is a read
of the field that the previous solve left, which is a fixed relation and not a
race.

### D6. This record states no period, no band edge and no divisor

**Every value is a rule of the downstream game, and the balance register holds
one row for each with its derivation.**[^7] A blocker governs them.[^12] The
owner may change any of them without superseding this record, because this
record states the shape that holds whatever the values are.

## Consequences

**The project cannot answer how fast a deposit recovers without reading the
world.** The period of one tile depends on the weather over it and on what
stands on it, so it changes from tick to tick and from tile to tile. A test that
asserts a period must state the moisture and the improvement it means.

**The recovery pass now reads the weather field and the upgrade map.** It read
neither before. The pass therefore holds the ledger out of itself while it
runs, because the reader borrows the field and the map beside it.

**A record that said no other pass reads the weather field is no longer
accurate.** Two passes read it now: this one and the wear that takes condition
from an upgrade.[^6]

**Foraging becomes a fallback rather than a living.** Raw ground gives back
slowly enough that a faction which lives on it starves. Worked ground gives
back fast enough to be worth returning to. That is the behaviour the owner
asked for, and it is a consequence of the scaling and not of any one value.

**The cost of the pass is unchanged in shape.** It follows the depleted set and
never the tile count, and each entry now costs two reads of the world rather
than none. No measurement of it exists on the target platform.[^12]

## References

[^1]: ADR-0072, a tile stock is generated, and only what was taken is stored, decisions D1 and D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^2]: ADR-0080, a depleted deposit recovers by ageing the stored take, decisions D1 to D4. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^3]: ADR-0080, a depleted deposit recovers by ageing the stored take, decision D5. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^4]: PRD-0024, a run stays eventful for as long as it is watched. `docs/product/idea/prd-0024-a-run-stays-eventful-for-as-long-as-it-is-watched.md`
[^5]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^6]: ADR-0143, wet ground yields more to a gatherer, decisions D1 and D3. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
[^7]: Balance register, the ground and what it gives. `docs/reference/balance.md`
[^8]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^9]: ADR-0169, an upgrade holds a condition that wear takes and work mends, decision D1. `docs/adrs/draft/adr-0169-an-upgrade-holds-a-condition-that-wear-takes-and-work-mends.md`
[^10]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^12]: Blockers register, BLK-050. `docs/BLOCKERS.md`
