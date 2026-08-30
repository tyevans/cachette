# Individual Agency and Occupations

**Report 16 of the background research for the foundational architecture
decision record.**

**Assigned number ranges: D110 to D129, and OQ70 to OQ79.**

---

## Scope

This report covers what one individual does and why. It answers four
questions.

1. How does an individual choose an action?
2. What job does an individual hold, and how does a job get filled?
3. What is an occupation, and how does skill enter the arithmetic?
4. What does an individual carry, and how does that total aggregate?

Three neighbouring reports own the adjacent ground. This report cites them
and does not redesign them.

| Owned by | Subject |
|---|---|
| Report 12 | Production rates, the modifier pipeline, the effective-stat table |
| Report 14 | Named characters, offices, succession, formations |
| Report 15 | Needs decay, consumption, wages, exchange, institutional solvency |

## Context that this report assumes

The reader needs eleven facts. Each is settled elsewhere and each is
repeated here so that this document stands alone.

**The world.** The engine simulates a hex world of 16.7 million tiles at
three levels of detail.[^1] Level 0 holds individual tiles. Level 1 holds
65,536 cells in a 256 by 256 grid, and one level 1 cell summarises 256
tiles. Level 2 holds 256 cells. Level 0 is the only source of truth.

**The population.** The target is one million units. The project owner has
decided that a unit is an individual person, not a formation.[^2] The owner
wants individual experiences. A larger group is a coarser view of the same
individuals, not a separate entity type.

**Position is tile-discrete.** A unit has no sub-tile coordinate. The tile
index is the position and it is also the spatial sort key. The unit array
stays sorted by tile index at all times.[^3]

**A tile holds at most 8 units.** The capacity is a data-driven parameter
stored as a `u8` per-tile count. The same array serves as occupancy, as the
capacity check, and as the density field.[^2]

**Movement is a floor-field cellular automaton.** A unit scores its 6
neighbours by a cached flow cost plus a density penalty, picks the lowest,
and the engine resolves conflicts by sort-then-admit.[^3] About 300,000 of
the one million units move in a typical tick.

**No floating point enters simulated state.** Float addition is not
associative, so a parallel sum is not reproducible. The fixed-point scale is
Q16.16 throughout.[^1]

**Every random draw is keyed.** A draw uses a counter-based generator keyed
on the tuple of system, frame, entity and draw index. Thread-local random
state is forbidden.[^1]

**Every parallel result is ordered by a stable key.** Thread completion
order and work-stealing order are forbidden as ordering.[^1]

**Fields live at level 1.** A field operator at level 0 costs 4.72
core-milliseconds for one diffusion pass, which is unaffordable. At level 1
the same pass costs 18.4 microseconds. The whole nine-field layer costs 0.32
to 0.71 core-milliseconds per tick.[^4]

**Effective stats come from a shared table.** The modifier pipeline runs
once per configuration, not once per entity. A per-entity input may enter
that pipeline only as a post-stage multiplier drawn from a small fixed
table, and the count of those multipliers is capped at 4.[^5]

**Population needs run on cohorts.** Report 15 recommends 40,000 cohort rows
across 5,000 settlements for needs decay, consumption and exchange. It
costs a decision at 400 nanoseconds and concludes that one million deciders
would cost 400 core-milliseconds, which is over the whole tick budget.[^6]

---

## 1. Terms used in this report

**Individual.** One simulated person. This report uses "individual" and
"unit" for the same row, because the owner has decided that a unit is a
person.

**Intent.** A small integer that names what an individual is currently
trying to satisfy. It is one `u8` column.

**Potential field.** One `u8` value for each level 1 cell. A high value
means the cell is attractive for one purpose.

**Gather.** A read from an array at a computed index.

**Argmax.** The index of the largest value in a short list.

**Occupation.** A `u16` index into a static table of jobs.

**Slot.** One ranked work position in one building.

**Skill tier.** An individual skill value quantised into one of 8 bands.

**Stagger.** A rule that splits a periodic pass across several ticks, so
that the cost is flat rather than a spike.

**Assignment problem.** The problem of matching N workers to N jobs to
maximise total benefit, where each worker takes exactly one job.

---

## 2. Executive summary

**The lead's framing holds on all four points, with two corrections and one
budget conflict that this report resolves.**

**1. Field-gradient action selection is affordable at one million, and it
is affordable by a factor of about 98.** The neighbouring report costs a
decision at 400 nanoseconds and concludes that one million deciders cost 400
core-milliseconds.[^6] That figure assumes 4 to 8 **random** gathers, which
are cache misses. The lead's formulation makes the gathers **sequential**,
because the units are already sorted by tile index and the fields are level
1 planes of 65,536 bytes. Eight `u8` planes total 512 KB and stay resident in
the level 2 cache for the whole pass. A full pass over one million
individuals with 8 fields costs **4.1 core-milliseconds**, which is 4.1
nanoseconds for each individual. Staggered over 32 ticks it costs **0.21
core-milliseconds for each tick**.

**Correction to report 15.** Its 400 nanosecond decision cost is correct for
a decision that gathers from a large scattered structure. It is wrong by
about two orders of magnitude for a decision that gathers from level 1
planes in tile-sorted order. Applying the same correction to its own cohort
decision line reduces that line from 16.00 core-milliseconds to under 0.05,
and removes what it names as 92 percent of its subsystem cost. **The
neighbouring report should adopt this formulation.**

**2. The stagger key must be the level 1 cell, not the entity identifier.**
This report found that staggering by entity identifier destroys the very
locality that makes the pass cheap. The active one thirty-second of the
population is then scattered through a 16 MB array, and the pass costs about
0.5 to 0.7 core-milliseconds rather than 0.21. Stagger by an integer mix of
the level 1 cell index. Every individual in one cell then decides on the same
tick, in one contiguous run, and the 8 field bytes for that cell are loaded
once and used about 15 times.

**3. No exact assignment method is within ten orders of magnitude.** The
question is not which exact algorithm to pick. A dense instance of one
million people against 100,000 slots holds 10^11 arcs. Any exact method must
read the arcs. Enumerating them costs 400 GB of traffic. The Hungarian method
dies at about 340 rows for a 4 core-millisecond budget. **The problem cannot
be posed at this scale, so it must be decomposed.**

**4. The lead's approximation is better than "approximate" suggests.** Global
routing by a vacancy field, then a local sorted greedy match, has a property
worth stating precisely. If the benefit of putting person `i` in slot `j`
factorises as `skill(i) * value(j)`, then sorting both sides and matching in
order is **exactly optimal**, not approximate. This is the rearrangement
inequality.[^7] **The local match loses nothing under the factorising
condition. The whole loss lives in the global routing**, and there it takes
two specific forms: an individual cannot distinguish two workplaces inside
the same level 1 cell, and migration lags the vacancy pattern by thousands of
ticks. This report declines to quote an optimality gap percentage, because
the gap depends entirely on content, and gives a measurement procedure
instead.

**5. Unemployment needs no special case, but a flat field does.** An
unemployed individual simply keeps an unmet work need and continues to
descend the vacancy gradient. That confirms the lead. **The failure mode is
different and it is serious.** When every field value is near zero the
gradient is flat, the argmax is decided by the tie-break, and the whole
population performs a correlated walk. That makes every unit a mover, which
breaks the 300,000-mover assumption that report 10 costs its movement
system against.[^3] **Add a floor: below a threshold score, hold the current
intent and do not move.**

**6. Skill fits the 4-slot budget, but only after two merges.** Six
candidates compete for 4 post-stage multiplier slots: health, veterancy,
terrain, skill, fatigue and morale. The resolution is not to raise the cap.
**Veterancy and skill are the same axis** and merge into one proficiency
tier. **Health and fatigue are the same axis** and merge into one condition
tier. **Morale does not belong in the stat pipeline at all**; it gates
behaviour rather than scaling a rate, so it belongs in the threshold-crossing
machinery. That leaves condition, proficiency, terrain, and one spare slot.
**The cap of 4 holds and does not need to move.**

**7. The inventory is 8 fixed slots and 18 MB.** Money is a separate `i32`
column, because a `u16` cap of 65,535 is too small for savings and is
absurdly generous for rations. Six slots are universal and two are
parameterised by the occupation, which gives a merchant grain and a smith
iron without storing a commodity identifier for each individual. A
per-individual commodity identifier would turn the aggregation from a map
into a general scatter, which is much worse.

**8. The whole subsystem costs about 0.5 core-milliseconds for each tick.**
That is 0.04 to 0.06 wall-milliseconds at 12 cores, against a tick of 12 to
46 wall-milliseconds.

**9. What the gradient cannot express is a real list, and it is short.**
Multi-step plans, deferred gain, precondition acquisition, negotiation, and
joint commitment. Section 4.2 gives the owner a three-question test.
Section 4.3 gives a cheap device that recovers most of the lost behaviour
without a planner: a fixed short script attached to the occupation.

---

## 3. Action selection as a field gradient

### 3.1 The claim, stated exactly

The lead proposes that "going to the store" is not a plan. It is descent
down a potential field.

Each need generates one potential field at level 1. An individual reads the
field values at its own cell, weights each by its own unmet need, and takes
the largest. The winning field names the intent. Movement then follows the
existing floor-field rule toward the neighbour cell with the higher value of
that one field.

The appearance of shopping emerges from a population descending a scarcity
gradient toward supply. No individual holds a plan and no individual runs a
search.

### 3.2 This is utility-based action selection

The method is not new. It is the standard utility architecture of game
artificial intelligence, computed as a field lookup rather than as a
per-agent option scan.

The architecture has three parts in every published description. A set of
**considerations** turns raw world state into a normalised score. A
**combination rule** folds the considerations into one utility for each
option. A **selection rule** picks an option from the utilities.

Dave Mark's book on the subject is the standard treatment, and the Infinite
Axis Utility System is its shipped form.[^8][^9] The *Game AI Pro* series
carries chapter-length treatments of needs-based and utility-based
selection.[^10]

The engine's version fixes each of the three parts to the cheapest possible
form.

| Part | General form | This engine |
|---|---|---|
| Consideration | A response curve over an input | A `u8` field value, precomputed by the field layer |
| Combination | A product or a weighted sum of curves | One multiply: unmet need times field value |
| Selection | Weighted random over the top `k` | Argmax with a fixed tie-break |

**Two of the three parts cost nothing at decision time.** The considerations
are already computed, because the field layer runs anyway for influence,
supply, price and morale.[^4] The combination is one multiply. Only the
selection is per-individual work, and it is a compare chain over 4 to 8
values.

**This is the whole reason the method is affordable.** A conventional utility
agent evaluates a response curve for each consideration for each option at
decision time. This engine amortises the curve evaluation across the whole
population by computing it once for each of 65,536 cells.

### 3.3 What is lost against a real planner

State this honestly, because the loss is the reason the character tier
exists.

**Goal-Oriented Action Planning** is the contrast case. It is a regressive
search over operators with preconditions and effects, derived from STRIPS,
and its shipped form in *F.E.A.R.* is the canonical game reference.[^11] A
planner builds a **sequence**. Given a goal, it chains backward through
operators until every precondition is satisfied by the current world state.

**A gradient has no sequence.** It has no memory between ticks, no notion of
a precondition, and no way to accept a worse position now for a better one
later.

A behaviour tree sits between the two. It encodes sequence explicitly, by
construction, and it is cheap to run, but the sequence is authored rather
than derived. A behaviour tree would express the daily round of a worker.
It would not express tool acquisition, because the tree cannot ask what it
lacks.

| Property | Field gradient | Behaviour tree | GOAP planner |
|---|---|---|---|
| Cost per agent per decision | 4 nanoseconds | 100 to 1,000 nanoseconds | 10 to 1,000 microseconds |
| Expresses a sequence | No | Yes, authored | Yes, derived |
| Expresses a precondition | No | Only as a test | Yes, as a search goal |
| Reacts to a changed world | Immediately | At the next tick | Needs a replan |
| Affordable at 1,000,000 | Yes | Marginal | No |
| Affordable at 10,000 | Yes | Yes | Yes |

**The design consequence.** The mass tier gets the gradient. The character
tier, which report 14 caps at 262,144 entities and exposes to Python, can
afford a real planner.[^12] The two tiers are not a compromise. They are the
correct answer, because a soldier does not need a plan and a duke does.

### 3.4 Affordability, computed

The pass has this shape. Every step is in the kernel vocabulary.

```
// map over the unit array, in tile-sorted order
for each individual u:                       // sequential scan
    cell = l1_cell_of(u.tile)                // 2 integer ops
    w    = weight_profile[u.occupation]      // gather, L1-resident table
    best = -1; best_score = FLOOR
    for i in 0 .. N:                         // N gathers
        f = field[i][cell]                   // u8 load, L2-resident plane
        s = (u.need[i] * w[i] >> 16) * f     // i64 multiply
        if s > best_score:                   // strict, so lowest i wins ties
            best_score = s; best = i
    u.intent = best                          // scatter into a u8 column
```

**Why the gathers are not misses.** The units are sorted by tile index and
the level 1 cell index is a fixed shift of the tile index under the same
block-tiled curve. So `cell` is monotone non-decreasing across the scan.
One million units over 16.7 million tiles gives 6 percent occupancy, so one
level 1 cell of 256 tiles holds about 15 units on average. Each field byte
is loaded once and used about 15 times.

**The whole field working set.** Each `u8` plane is 65,536 bytes. Eight
planes are 512 KB. That fits in the 1 MB private level 2 cache of a Neoverse
V1 core.[^13] The planes are touched in near-sequential order, so the
hardware prefetcher works.

**Instruction count.** The loop body costs about `12 + 3N` operations,
counting the address arithmetic, the loads, the multiplies, the compares and
the store.

| `N` | Ops for each individual | Full pass, 1,000,000 | Nanoseconds for each individual |
|---|---|---|---|
| 4 | 24 | **2.7 core-ms** | 2.7 |
| 6 | 30 | **3.4 core-ms** | 3.4 |
| 8 | 36 | **4.1 core-ms** | 4.1 |

The figures assume 3.5 GHz and an instructions-per-cycle rate of 2.5, which
is conservative for a dependence-free integer loop on a Neoverse core.[^13]

**Bandwidth check.** The pass reads a 4-need vector at 16 bytes, an
occupation identifier at 2 bytes, and a tile index at 4 bytes. It writes one
intent byte. That is 22 bytes read and 1 byte written for each individual,
plus 512 KB of field planes for the whole pass.

```
traffic = 1,000,000 * 23 bytes + 512 KB = 23.5 MB
floor   = 23.5 MB / 40 GB/s = 0.59 milliseconds
```

**The pass is compute-bound, not bandwidth-bound**, at 2.7 to 4.1 core-ms
against a 0.59 ms floor. That means adding more cores helps, which the
staggered form does not even need.

**Against the neighbouring report's figure.** Report 15 costs the same
decision at 400 nanoseconds and one million deciders at 400
core-milliseconds.[^6] The difference is 4.1 nanoseconds against 400, a
factor of 98. The whole factor is the memory behaviour of the gather.

| Formulation | Gather target | Cost for each gather | Decision cost |
|---|---|---|---|
| Report 15 | A scattered structure, DRAM | 80 to 100 ns | 400 ns |
| This report | A level 1 plane, L1 or L2 cache | under 1 ns | 4.1 ns |

**Both figures are correct for what they measure.** Report 15's figure is
right for a decision that samples prices at distant markets by identifier.
This report's figure is right for a decision that samples a level 1 field at
the individual's own cell. The design difference is that the field layer has
already moved the distant information to the individual's cell.

**This is the substantive finding of the report.** It is also the answer to
report 15's own open question 69, which asks what the cohort decision is
exactly.[^6]

### 3.5 The re-evaluation cadence, and a correction to the stagger key

The lead expects intent to be sticky, and expects re-evaluation every 30 to
60 ticks staggered across the population. **The period is right. The stagger
key is wrong, and the difference costs a factor of 3.**

**Why intent must be sticky.** An individual that re-decides every tick
oscillates. Two fields of nearly equal value swap the argmax on small field
changes, and the individual reverses direction. Movement then averages to
zero and the individual never arrives anywhere. Stickiness is not only an
optimisation. It is what makes the behaviour legible.

**The obvious stagger is wrong.** The natural key is the entity identifier:
re-evaluate individual `e` on ticks where `e mod 32 == frame mod 32`. That
is a pure function of the identifier and of the frame, so it is
deterministic. It is also 3 times slower than the alternative.

The reason is that the unit array is sorted by **tile**, not by identifier.
One thirty-second of the entities, selected by identifier, are scattered
uniformly through a 16 MB need array. Each active individual touches its own
cache line and shares it with nobody.

| Item | Cell stagger | Entity stagger |
|---|---|---|
| Active individuals for each tick | 31,250 | 31,250 |
| Contiguous runs | 2,048 | about 31,250 |
| Useful bytes for each 64-byte line | 48 to 64 | 16 to 22 |
| Need-array traffic | 0.75 MB | 2.0 MB |
| Field-plane traffic | 0.5 MB | 0.5 MB, no reuse |
| Cost for each tick, `N` of 8 | **0.21 core-ms** | 0.5 to 0.7 core-ms |

**The recommended key is an integer mix of the level 1 cell index.**

```rust
// A fixed multiply-xorshift. It is a compile-time constant function.
const fn stagger_phase(cell: u32, period_log2: u32) -> u32 {
    let mut x = cell.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x & ((1 << period_log2) - 1)
}
// individual re-evaluates when stagger_phase(cell, 5) == (frame & 31)
```

The mix is necessary. A bare `cell & 31` re-evaluates a regular spatial
stripe of the map on each tick, which correlates the decision phase with the
geography and produces visible banding.

**Three properties of this key.**

1. **It is a pure function of state.** It reads the cell index, which is a
   pure function of the position. It reads no counter and no accumulator.
   It is therefore deterministic under any thread count.
2. **Everyone in a cell decides together.** That is the property that gives
   the contiguous runs and the 15-fold field reuse.
3. **A moving individual may decide twice or skip once.** When an individual
   crosses a cell boundary into a cell with a different phase, its interval
   is not exactly 32 ticks. It is between 1 and 63 ticks. **Record this as
   accepted behaviour, not a defect.** Re-deciding on arrival in a new
   region is the correct behaviour anyway, and skipping one interval is
   invisible.

**Recommended period: 32 ticks.** Use a power of two so the phase test is a
mask rather than a division. That sits inside the lead's 30 to 60 range at
its lower end, which is the right end, because the pass is cheap and a
shorter period makes the population more responsive. **Do not go to 64
unless the budget breaks**, and it does not.

**One exception. Force an immediate re-evaluation on a discontinuity.** Three
events must not wait up to 32 ticks: the individual's employment ends, the
individual's workplace is destroyed, and a need crosses a critical
threshold. Report 12's threshold-crossing machinery already emits exactly
these as sparse events.[^5] Consume that sparse list in the same pass and set
a `re_evaluate` flag bit. The list is under 1 percent of the population, so
it adds nothing measurable.

### 3.6 The utility score in fixed point

The score is a transient comparison quantity. **It is never stored, so it
never enters simulated state.** Determinism still requires that it be exactly
reproducible, and integer multiplication is.

The inputs are three.

| Input | Type | Scale | Range |
|---|---|---|---|
| Unmet need | `i32` | Q16.16 | 0 to 65,536, meaning 0 to 1 |
| Occupation weight | `i32` | Q16.16 | 0 to 262,144, meaning 0 to 4 |
| Field value | `u8` | Q8.0 | 0 to 255 |

Compute in `i64` and shift once.

```
s = (((need as i64) * (weight as i64)) >> 16) * (field as i64)
```

The worst case is `65,536 * 262,144 = 2^34`, shifted to `2^18`, then
multiplied by 255 to give under `2^26`. **The score fits in an `i32` with 5
bits of headroom.** Use an `i64` accumulator anyway, because a widening
64-bit integer multiply runs at full rate on the target and the headroom
argument then needs no maintenance.[^13]

**A weighted sum with integer weights is exactly reproducible.** Integer
addition and integer multiplication are exactly associative and exactly
commutative. The sum is identical for any evaluation order and for any
thread count. This is the property that a floating-point utility score would
not have, and it is why the whole method survives the project's ban on
floating point without any loss.

**The tie-break.** Scan the options in ascending option index and use a
strict greater-than comparison. **The lowest option index wins a tie.** This
is free, because it is the natural form of the compare chain, and it is
total, because the option indices are distinct. Do not break ties by a
random draw. A keyed draw would be deterministic but it would add a
generator call to the hot loop for a case that carries no design value.

### 3.7 The floor, and why a flat field is the failure mode

Set a floor score. When the largest score is below the floor, **hold the
current intent and do not move**.

Without the floor, a flat field produces a correlated population-wide walk.
Every individual sees near-equal values, the tie-break selects option 0 for
everyone, and the population drifts along one gradient direction that is
mostly rounding noise.

**The cost of that failure is measurable and large.** Report 10 costs the
movement subsystem at 1.9 to 3.8 wall-milliseconds against an assumed
300,000 movers out of one million.[^3] A flat field turns all one million
into movers. The movement cost then rises by a factor of about 3.3, to 6 to
13 wall-milliseconds, and the mover-subset sort that report 10 relies on
degrades from 2 to 4 core-milliseconds back to the full 4 to 8.

```
FLOOR = 1 << 14        // Q16.16, meaning 0.25 of one unit of weighted need
```

**Set the floor at bake time and record it.** It is not a tuning constant
that a designer may change freely, because changing it changes the mover
count and therefore the frame budget.

### 3.8 Which fields, and how many

The lead asks for `N` between 4 and 8. **Recommend `N` of 6.** The list
below fits the needs that report 15 already declares and adds the two that
this report owns.

| `i` | Field | Level | Cadence | Owner | Already exists |
|---|---|---|---|---|---|
| 0 | Food availability | 1 | 4 | Report 15 | No. Add. |
| 1 | Drink availability | 1 | 4 | Report 15 | No. Add. |
| 2 | Shelter and rest | 1 | 16 | Report 15 | No. Add. |
| 3 | Safety, as the complement of threat | 1 | amortised | Report 09 | **Yes.** Reuse the military presence field. |
| 4 | Work vacancy pressure | 1 | 64 | **This report** | No. Add. Section 5. |
| 5 | Social and market draw | 1 | 8 | Report 13 | **Yes.** Reuse the price potential field. |

**Two of the six already exist.** Safety is the complement of the military
presence field that the influence subsystem computes. Social draw is the
price potential field that the trade subsystem computes. Reusing them costs
nothing and it also means the individual's behaviour responds to war and to
markets without any new machinery.

**The four new planes cost 4 by 64 KB, which is 256 KB.** Their update cost,
using report 13's published operator figures at level 1, is:

| Field | Operators | Cost for one update | Cadence | Core-us for each tick |
|---|---|---|---|---|
| Food availability | source, 3 diffuse, decay | 58 us | 4 | 14.5 |
| Drink availability | source, 3 diffuse, decay | 58 us | 4 | 14.5 |
| Shelter and rest | source, 4 diffuse, decay | 78 us | 16 | 4.9 |
| Work vacancy | separable recursion, 6 passes | 12 us | 64 | 0.2 |
| **Total** | | | | **34 core-us** |

**The four new fields cost 0.034 core-milliseconds for each tick.** Report 13
budgets the whole field layer at 0.32 to 0.71 core-milliseconds and
recommends that the record's existing influence line be renamed to cover
it.[^4] These four fit inside that line without changing it.

**Do not exceed 8 fields.** Each field adds 64 KB to the resident working
set. At 16 fields the working set is 1 MB, which no longer fits beside the
need array in a 1 MB level 2 cache, and the pass loses the property that
makes it cheap.

---

## 4. What the gradient cannot express

This section is the one the project owner asked for. It states the limits
plainly and gives a test.

### 4.1 The seven things a gradient cannot do

**1. A multi-step plan with a required order.** "Fetch the axe, then walk to
the tree, then chop, then carry the log home." The gradient holds no state
between ticks, so it cannot know which step it is on. It will descend toward
whichever of the four is most attractive right now, and that is not a
sequence.

**2. Deferred gain.** "Walk two days through empty country to reach a better
market." The gradient descends the strongest local signal. It cannot accept a
worse position now for a better one later. A field with a long decay length
partly hides this, because the distant market's potential reaches further,
but the individual is still following a local slope and it will still stop at
any local maximum.

**3. Precondition acquisition.** "I need a hammer before I can smith." This
needs backward chaining from a goal to its unmet preconditions. That is
exactly what a planner does and exactly what a gradient does not.

**4. Saving.** "Accumulate money now, buy a house later." Saving is deferred
gain plus a threshold, and the gradient has neither.

**5. Negotiation and promises.** Two individuals agreeing a price or a
contract requires a shared state that neither field holds. Fields carry
aggregates. A promise is between two named parties.

**6. Joint commitment.** "Two people carry the beam." The gradient produces
coincidence, not coordination. Both may arrive; neither commits.

**7. Anything whose target does not exist yet.** "Build a road so I can
trade." The field can only advertise what is already there.

### 4.2 The test for the project owner

Ask three questions about any behaviour you want.

> **1. Can you name a single number that is highest exactly where the
> behaviour should be strongest?**
>
> **2. Does the behaviour still make sense if the individual forgets
> everything between one tick and the next?**
>
> **3. If two individuals both do it, do they interfere only through
> crowding and scarcity?**

**Three yes answers means the field expresses it.** Add a plane, add a
weight column to the occupation table, and it is done for the whole
population at 4 nanoseconds each.

**Any no answer means it needs a sequence or a memory.** It belongs in the
character tier, where the population is a few thousand to a few hundred
thousand and a real planner is affordable.[^12]

Worked examples.

| Desired behaviour | Q1 | Q2 | Q3 | Verdict |
|---|---|---|---|---|
| Go to the store for food | Yes: food availability | Yes | Yes: the store runs out | **Field.** |
| Flee a battle | Yes: threat | Yes | Yes | **Field.** |
| Migrate to a city with work | Yes: vacancy pressure | Yes | Yes: slots fill up | **Field.** |
| Go home at night | Yes: home potential, gated by a clock | Yes | Yes | **Field**, with a time gate. |
| Sharpen the axe before chopping | **No** | **No** | Yes | Needs a sequence. |
| Save for a house | No | **No** | Yes | Needs a memory and a threshold. |
| Two people carry a beam | No | No | **No** | Needs commitment. Character tier. |
| Marry into a rival house | No | No | No | Character tier.[^12] |

### 4.3 The cheap device that recovers most of the loss

There is a middle option between a gradient and a planner, and it is much
closer to free than to expensive. **Attach a fixed short script to the
occupation.**

A script is a small cyclic state machine of at most 8 states. The occupation
table names the script. The individual stores one `u8` script state. Each
state names one field to descend and one condition that advances the state.

```
occupation = BAKER
script     = [ GoToWorkplace, Work, GoToMarket, GoHome ]
```

This is not a planner. There is no search and no goal. The sequence is
authored at bake time and it is the same for every individual with that
occupation. It costs **one `u8` column, which is 1 MB at one million
individuals**, and one extra compare in the intent pass.

**What it buys.** The daily round, the commute, the shift, and the return
home. Those are the behaviours a player actually watches, and they are the
whole reason the walker models in city builders exist. It also expresses
"fetch the tool first", because that is a state in the script rather than a
derived precondition.

**What it does not buy.** Anything the author did not write down. It cannot
adapt the sequence to a changed world. If the market has burned down, a
scripted individual still walks to where the market was, until the state's
advance condition fires or the field routes it elsewhere.

**Recommend building this in version 1**, alongside the gradient, because it
is the difference between a population that flows and a population that
behaves, and it costs 1 MB.

### 4.4 The three-tier picture

| Tier | Population | Mechanism | Cost for each decision |
|---|---|---|---|
| Mass | 1,000,000 | Field gradient, plus an occupation script | 4 ns |
| Character | up to 262,144 | Per-entity logic, Python-accessible[^12] | microseconds |
| Faction | tens | Player or the control plane | unbounded |

**The tiers are not a compromise forced by the budget.** They are also the
correct simulation. A soldier does not choose their rations. A duke chooses
an heir. The mechanism should differ because the subject differs.

---

## 5. Job assignment

### 5.1 The problem, stated at scale

The assignment problem matches N workers to M jobs to maximise total
benefit, with each worker taking at most one job and each job taking at most
one worker. It is a special case of the transportation problem, which is
itself a special case of minimum cost flow.

At this project's scale, `N` is one million and `M` is on the order of
100,000 slots.

### 5.2 The exact methods, and where each dies

Survey them honestly. The complexities are published and stable.

| Method | Complexity | Reference |
|---|---|---|
| Hungarian, Kuhn and Munkres | O(n^3) | Kuhn 1955, Munkres 1957[^14][^15] |
| Jonker-Volgenant shortest augmenting path | O(n^3) worst case, far better in practice | Jonker and Volgenant 1987[^16] |
| Bertsekas auction with epsilon scaling | O(N A log(N C)) | Bertsekas 1988[^17] |
| Min-cost flow as assignment | O(V E log V) class | Standard[^18] |
| Gale-Shapley stable matching | O(N M) | Gale and Shapley 1962[^19] |

**The budget.** Allow job assignment 4 core-milliseconds for each tick, which
is about 2 percent of the mean tick, matching the allowance that report 15
sets for its decision tier.[^6] At an optimistic 10^10 integer operations per
second, that budget is `4 * 10^7` operations.

**Where each method dies.**

| Method | Largest `n` inside 4 core-ms | Largest `n` inside 1 second |
|---|---|---|
| Hungarian, `n^3` | **about 340** | about 2,150 |
| Jonker-Volgenant, about `n^2.5` in practice | about 1,100 | about 10,000 |
| Auction, sparse with `k` of 8 arcs each | about 10,000, but see below | about 10^6, unbounded iterations |
| Gale-Shapley, `n * m` | about 6,300 by 6,300 | about 100,000 by 100,000 |

**The Hungarian method dies at about 340 rows.** That is the honest number
and it is worth writing down, because 340 is far smaller than most readers
expect from an `O(n^3)` method. At 100,000 rows it needs `10^15` operations,
which is about 28 hours. At one million it needs `10^18`, which is about
3.2 years at the same rate.

**The arc count is the real bound, and it defeats every exact method.** A
dense instance of one million people against 100,000 slots has `10^11` arcs.
Any exact method must at least read each arc once. At 4 bytes for each arc
that is 400 GB of traffic, which is 10 seconds at 40 GB/s **before the
algorithm starts**. The instance cannot even be written down inside a frame.

**Sparsifying does not save it.** Restrict each person to the `k` of 8
nearest slots and the arc count falls to `8 * 10^6`. That is tractable to
store. The auction algorithm then runs, but report 11 already records the
disqualifying property: **epsilon scaling gives a data-dependent iteration
count, and a data-dependent iteration count breaks a fixed frame
budget.**[^20] That is a determinism and scheduling objection, not a speed
objection, and it does not go away with more cores.

**Conclusion.** The question is not which exact algorithm to choose. **The
problem cannot be posed at this scale, so it must be decomposed into a
spatial part and a local part.** That is exactly what the lead proposes.

### 5.3 The recommended decomposition

**Global routing by field. Local matching by sort.**

**Part 1: vacancy pressure is a field.** Each workplace with an unfilled slot
emits a source proportional to the slot's value. The field diffuses at level
1. Individuals whose intent is work descend the gradient. This is the
`source`, `diffuse`, `decay` triple that the field layer already provides.[^4]

**How many planes?** One plane for each occupation would be 64 or more
planes. At 64 KB each that is 4 MB of memory, which is affordable, but the
update cost is not: 64 separable recursions at 12 microseconds each is 768
microseconds for one update, which exceeds the entire field-layer budget.

**Recommend 8 job families, not 64 occupations.** Eight `u8` planes are
512 KB. The update is 8 separable recursions at 12 microseconds, which is
96 microseconds, at a cadence of 64 ticks. **That is 1.5 core-microseconds
for each tick, which is free.** A family groups occupations that a worker can
plausibly move between: farming, crafting, extraction, construction, trade,
service, military, and administration.

Eight also matches the 8 strata of report 15 and the 8 tiers used elsewhere,
which lets one `u8` carry a family index with the flag bits.

**Part 2: the local match is a sort and a scan.** At a workplace, sort the
applicants and fill the ranked slots in order.

```
// per settlement, in ascending settlement identifier order
sort applicants by (family_match desc, skill_tier desc, entity_id asc)
sort slots      by (rank asc, slot_value desc, slot_id asc)
scan both lists in parallel; assign the k-th applicant to the k-th slot
```

Both sorts are the engine's stable radix sort. The scan is a `local join`.
There is no atomic and no contention, because the settlements are disjoint.

### 5.4 The quality loss, stated precisely

This is where most treatments hand-wave. This report does not.

**Finding 1. The local match loses nothing, under a stated condition.**

Let the benefit of putting person `i` in slot `j` be `b(i,j)`. If the benefit
factorises as a product of a person term and a slot term,

```
b(i,j) = skill(i) * value(j)
```

then the matrix has rank 1 and is non-negative. **For such a matrix, sorting
both sequences in the same order and matching them in order maximises the
total exactly.** This is the rearrangement inequality.[^7] Sorted greedy is
not an approximation here. It is the optimal solution.

**So design the benefit function to factorise, and the local match becomes
exact.** That is a content rule, and it is a cheap one: score a slot by a
single value number and score a person by a single skill number.

The condition generalises. Greedy is also exact when the benefit matrix has
the Monge property, which is the standard condition under which greedy solves
a transportation problem.[^21] The rank-1 product form is the simplest case
of it.

**Where the condition breaks.** It breaks when a slot needs a *specific*
skill rather than more skill. A blacksmith slot filled by a highly skilled
baker is worth nothing, not a lot. The fix is the family filter, which is the
first sort key: a person outside the family is not a candidate at all.
Inside a family, more skill is genuinely better, so the factorising condition
holds inside a family and the greedy match is exact inside a family.

**Finding 2. The whole loss lives in the global routing, and it has two
components.**

**Component A: level 1 indistinguishability.** The vacancy field is computed
at level 1, so an individual cannot tell two workplaces apart inside the
same 256-tile cell. Inside one cell the assignment is arbitrary.

> **The bound: the loss is at most the spread of slot values inside one
> level 1 cell.**

That is a useful bound, because it is small in practice. Two workplaces 200
tiles apart in the same city are close substitutes.

**Component B: migration lag.** The field routes an individual at movement
speed. A cell is 256 tiles wide, so crossing one cell takes at least 256
ticks. Equalising vacancy pressure across a 256-cell map by diffusion takes
on the order of 10^4 ticks.

> **The labour market responds over game-months, not over ticks.**

**State this as a feature, not a defect.** A labour market that reallocates
instantly is both more expensive and worse simulation. Real workers do not
teleport to the highest-paying vacancy.

**Finding 3. This report declines to quote an optimality gap percentage.**

A percentage would depend entirely on the benefit matrix that the content
supplies, and the content does not exist yet. A fabricated figure would be
worse than no figure. **Measure it instead**, with this procedure:

1. Freeze one region at a scale of 2,000 people and 2,000 slots.
2. Run the Jonker-Volgenant algorithm offline on the full benefit matrix to
   get the optimum.[^16] At `n` of 2,000 this takes seconds, not hours.
3. Run the field-and-greedy method to steady state on the same instance.
4. Report the ratio of achieved total benefit to optimal total benefit.

**Make this a regression test with a recorded threshold**, not a one-off
study. The ratio will change when content changes, and a silent degradation
would otherwise be invisible.

### 5.5 Unemployment

**Confirmed. Unemployment needs no special case.**

An individual with no slot keeps a high unmet work need. Its weighted score
on the vacancy field therefore stays high, so its intent stays "seek work",
and it continues to descend the vacancy gradient. When it reaches a
settlement with a free slot, the local match assigns it. When it does not, it
keeps walking. Nothing in the code tests for unemployment.

**The consequences arrive through the needs system, not through the job
system.** No slot means no wage. No wage means no money. No money means the
individual loses the exchange pass in report 15 and its food need goes
unmet.[^6] The starvation path already exists and this report adds nothing
to it.

**One rule is needed, and it is the floor of section 3.7.** When vacancy
pressure is zero everywhere, an unemployed population must settle rather
than wander. The floor gives that: below the threshold the individual holds
its intent and stops moving. **Without the floor, mass unemployment becomes
mass migration, and mass migration triples the movement cost.** That is a
budget failure caused by a content condition, which is the kind of failure
that is hardest to find later.

### 5.6 Cost of the job subsystem

| Work | Scale | Period | Core-ms for one pass | Core-ms for each tick |
|---|---|---|---|---|
| Vacancy fields, 8 families | 8 planes at 65,536 cells | 64 | 0.096 | **0.0015** |
| Applicant gather, sparse | about 20,000 job seekers | 100 | 0.30 | 0.003 |
| Applicant sort, radix | 20,000 keys | 100 | 0.20 | 0.002 |
| Slot scan and assign | 100,000 slots | 100 | 0.50 | 0.005 |
| Reverse index rebuild | 1,000,000 workplace column | 100 | 5.00 | 0.050 |
| **Total** | | | | **0.06 core-ms** |

The reverse index rebuild dominates and it reuses report 14's counting sort
into a compressed sparse row structure without change.[^12]

**The job subsystem costs 0.06 core-milliseconds for each tick.** The
matching itself is a rounding error. This is the expected result once the
exact methods are ruled out: what remains is a sort, and a sort of 20,000
keys is nothing.

---

## 6. Occupations, careers and skill

### 6.1 An occupation is a `u16` index

**Confirmed. An occupation is data, not code.** This follows the record's
existing rule that unit types and upgrades are indices into shared tables and
that types parameterise the verbs rather than multiply them.[^1]

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OccupationDef {
    family:        u8,       // 0..7. Selects the vacancy field plane.
    script:        u8,       // Index into the script table. Section 4.3.
    inv_slot_6:    u8,       // Which commodity fills personal slot 6.
    inv_slot_7:    u8,       // Which commodity fills personal slot 7.
    need_weight:   [i32; 6], // Q16.16. The utility weights of section 3.6.
    base_stat_set: u16,      // Row in the effective-stat table.
    wage_band:     u8,
    flags:         u8,
}
```

The row is 32 bytes and it needs no padding. At 256 occupations the whole
table is **8 KB**, which sits in the level 1 cache for the whole intent pass.

**Store the occupation as one `u16` column on the individual.** That is 2 MB
at one million individuals.

**Do not cap the occupation count below 65,536.** There is no reason to. The
`u16` is the cap and the table is tiny. The count that must be capped is the
**family** count at 8, because a family costs a 64 KB field plane and a field
update.

### 6.2 A career path is a directed acyclic graph over occupation identifiers

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CareerEdge {
    from:          u16,  // OccupationId
    to:            u16,  // OccupationId
    min_skill:     u8,   // Skill tier required, 0..7
    min_age:       u8,   // In game years
    req_rank:      u8,   // Workplace rank required in the source occupation
    req_building:  u8,   // Building kind that must exist at the destination
}
```

Store the edges as a compressed sparse row structure indexed by the source
occupation. The row is 8 bytes. A few thousand edges cost tens of kilobytes.

**Check acyclicity at bake time, not at run time.** Run one topological sort
over the edge set when the content loads and reject a cycle. A cycle at run
time would let an individual loop between two occupations indefinitely, and
detecting that at run time costs a visited set for each individual.

**A career progression check is a filter, then a gather, then a scan.** It is
cheap because it is rare.

```
filter: individuals whose skill tier increased since the last check
gather: the outgoing career edges for each one's occupation
scan:   take the first edge whose requirements are all met
        break ties by ascending destination occupation identifier
```

**Cadence: every 512 ticks, staggered by an integer mix of the entity
identifier.** Career change is a rare life event. At period 512 the active
set is about 2,000 individuals for each tick and the pass costs under 0.01
core-milliseconds.

**Here the stagger key is the entity identifier, not the cell.** That is the
opposite of section 3.5 and the reason is that this pass is not
bandwidth-bound. It touches 2,000 individuals, which is 128 KB of scattered
lines, and it does 20 to 50 operations on each. There is no locality to
preserve, so use the simpler key.

### 6.3 Skill, and its quantisation

**Skill is an individual value. The stat pipeline sees only a tier.**

| Column | Type | Bytes at 1M | Purpose |
|---|---|---|---|
| `skill` | `u16` | 2 MB | Fine-grained experience, 0 to 65,535. Progresses smoothly. |
| `skill_tier` | 3 bits, packed | 0 | Derived. 0 to 7. Enters the stat pipeline. |

**Store the fine value and derive the tier.** Two reasons. A `u8` skill would
give only 256 experience steps over a whole career, which makes progression
visibly steppy. And the tier must be recomputed anyway when the bands change,
which is a content edit.

**Pack the tier into an existing flag byte.** Do not give it its own column.
The tier changes only at the career cadence of 512 ticks, so writing it into
a shared byte causes no false sharing in the hot path.

**8 tiers. Verified against the neighbouring report.** Report 12 quantises
health into 8 bands and states that the quantisation is a visible design
choice rather than an implementation detail.[^5] Use 8 for skill for the same
reasons: 8 fits a 3-bit field, an 8-entry `i32` lookup table is 32 bytes and
stays in the level 1 cache, and 8 bands is the resolution that strategy games
already use. **Two individuals at tier 5 work identically. Record that.**

**Skill accumulation is a per-individual saturating add.**

```
skill = saturating_add(skill, gain[occupation] * worked_this_tick)
```

**A saturating add is not associative at the cap**, so it must never appear
inside a cross-entity reduce. It does not: each individual accumulates only
its own skill, and there is no reduction. **State this in the decision**, so
that nobody later "optimises" skill gain into a segmented reduce and silently
breaks reproducibility at the cap.

Skill accumulation rides the intent pass at period 32, so it costs nothing
extra. Multiply the gain by 32 at bake time.

### 6.4 The 4-slot budget conflict, and its resolution

**This is a real conflict and it must be settled before either report is
merged.**

Report 12 caps the per-entity post-stage multipliers at 4 and its own worked
example already spends 3 of them.[^5] Adding skill makes 4. Adding the two
that the design also wants makes 6.

| # | Candidate | Claimed by | Bands |
|---|---|---|---|
| 1 | Health tier | Report 12 | 8 |
| 2 | Veterancy | Report 12 | small enumeration |
| 3 | Terrain | Report 12 | 32 terrains by field |
| 4 | Skill tier | **This report** | 8 |
| 5 | Fatigue | Implied by report 15's rest need | 8 |
| 6 | Morale | Implied by the crowd and economy work | 8 |

**Six candidates. Four slots. The budget does not fit.**

**Do not raise the cap.** Each slot is one multiply and one shift for each
entity for each evaluated field. Report 12 measures 3 slots at under 3
nanoseconds for each entity for each field, so 6 slots is about 6
nanoseconds.[^5] At one million entities and 3 evaluated fields that is 18
core-milliseconds against 12 at the cap of 4. Six core-milliseconds is a real
cost for no structural gain.

**Resolve it by merging, and it merges cleanly into 3 slots with 1 spare.**

**Merge 1: veterancy and skill are the same axis.** Both are accumulated
experience in the individual's current occupation. A soldier's occupation is
soldier, and its skill *is* its veterancy. There is no case where an
individual needs both a separate combat experience and a separate trade
experience, because an individual holds one occupation at a time.

> **Recommendation: one `proficiency_tier` slot, fed by the single `skill`
> column of section 6.3.** This also removes a column and a bake-time
> decision about how veterancy and skill interact.

**Merge 2: health and fatigue are the same axis.** Both answer "how fit is
this body right now". Combine them with a small lookup table rather than a
minimum, so that content can tune the interaction.

```
condition_tier = condition_lut[health_tier][fatigue_tier]   // 8 x 8 -> 0..7
```

The table is 64 bytes and stays in the level 1 cache. The cost is one extra
byte load, not a multiply.

> **Recommendation: one `condition_tier` slot, fed by an 8 by 8 table.**

**Merge 3: morale does not belong in the stat pipeline.** Morale does not
scale a rate. It gates a behaviour: whether the individual routs, whether it
obeys an order, whether it revolts. That is a **threshold crossing**, and
report 12 already owns a two-phase threshold-crossing pass with a stated
ordering rule.[^5]

> **Recommendation: move morale out of the multiplier budget entirely and
> into the threshold predicate pass.** This costs nothing and it is also the
> more correct model.

**The result.**

| Slot | Content | Table size |
|---|---|---|
| 1 | `condition_tier`, from health and fatigue | 8 entries |
| 2 | `proficiency_tier`, from skill and veterancy | 8 entries |
| 3 | `terrain`, unchanged from report 12 | 32 by field count |
| 4 | **Spare** | — |

**The cap of 4 holds, skill fits, and one slot remains free.** No change to
report 12's schema rule is needed. Report 12's own worked example should be
updated to name `condition_tier` and `proficiency_tier` rather than health
and veterancy.

### 6.5 Workplace hierarchy

The lead asks about a blacksmith with apprentices and a tavern with a keeper.
Report 14 owns named-character hierarchies. This report owns the anonymous
case.

**A workplace carries a fixed small number of ranked slots.**

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct WorkSlot {
    holder:     u32,  // Entity identifier, or SENTINEL_EMPTY
    occupation: u16,
    rank:       u8,   // 0 is the senior slot
    flags:      u8,
}
```

**Recommend 4 slots by default and cap the count at 8.**

Why 4 as the default: it expresses master, two journeymen and an apprentice,
which is the workshop shape that the design asks for. Why 8 as the cap: **a
tile holds at most 8 units**, so a workplace on one tile cannot usefully hold
more than 8 workers standing in it.[^2] The two limits should agree, and
making them agree removes a class of content bug where a building declares
more slots than can physically be occupied.

**Storage.** At 50,000 workplaces and a mean of 4 slots that is 200,000 slot
rows at 8 bytes, which is **1.6 MB**. At the cap of 8 for every workplace it
is 3.2 MB. Negligible either way.

**Slot-holders are anonymous rows, not named entities.** This is the decisive
recommendation and there are three reasons.

1. **The character budget cannot absorb them.** Report 14 declares a ceiling
   of 262,144 characters.[^12] Two hundred thousand shopkeepers would consume
   76 percent of that ceiling and leave almost nothing for the nobility,
   which is the population the character tier exists to serve.
2. **A promotion path already exists.** Report 14 promotes a unit into the
   character tier by a filter, a sort and a budget.[^12] A master craftsman
   who accumulates deeds is exactly the case that mechanism is for. **A
   workplace slot does not need to be named. It needs to be promotable.**
3. **The forward-column pattern is already chosen.** Report 14 rejects a
   per-formation list of member identifiers in favour of a forward column on
   the unit plus a rebuilt reverse index, because a despawn invalidates every
   list that holds it.[^12] The same argument applies without change.

**Recommendation: add a `workplace: u32` column to the individual and pack
`rank` into an existing flag byte.** The forward column is 4 MB at one
million individuals. Build the reverse index by counting sort into a
compressed sparse row structure at the job-matching cadence, reusing report
14's kernel without writing a second one.

**A rank is not an office.** Report 14's office machinery, with its anchored
forest and its succession policies, is for titles. A workshop rank is not a
title. It has no succession law, no claim, and no policy column. When the
master dies, the local match of section 5.3 refills the slot at the next
matching tick from whoever is present. That is the whole rule.

---

## 7. Individual inventory

### 7.1 The lead's estimate, verified

The lead estimates 4 to 8 personal commodities at `u16`, for roughly 8 to 16
MB.

```
1,000,000 individuals * 8 slots * 2 bytes = 16.0 MB
1,000,000 individuals * 4 slots * 2 bytes =  8.0 MB
```

**The arithmetic is correct.** The recommendation adjusts it in one place.

### 7.2 Fixed slots, not slot pairs

Two designs are possible and the difference is large.

**Design A, fixed slots.** Slot `i` always holds the same commodity, decided
at bake time. Store 8 `u16` values, with no identifier.

**Design B, slot pairs.** Store a `(commodity_id: u8, quantity: u16)` pair, so
that any individual can carry any commodity.

| Property | Design A, fixed | Design B, pairs |
|---|---|---|
| Bytes for each individual | 16 | 24 |
| Total at 1,000,000 | 16 MB | 24 MB |
| Aggregation kernel | **`map` then segmented `reduce`** | **general `scatter`** |
| Vectorises on NEON | Yes, the column is a plain array | No, the index is data |
| Deterministic without care | Yes, the reduce has fixed spans | Needs an ordered scatter |

**Recommend Design A.** The memory difference is 8 MB, which is not the
argument. The argument is the aggregation kernel. Design A aggregates each
slot as an independent segmented reduce over a contiguous column, which is
the exact kernel that report 12 already specifies for pooled quantities and
which needs no atomic.[^5] Design B needs a scatter keyed on a per-individual
identifier, which is both slower and harder to make deterministic.

### 7.3 The recommended set

**8 slots. Six are universal. Two are parameterised by the occupation.**

| Slot | Contents | Type | Rationale |
|---|---|---|---|
| 0 | **Money** | `i32`, separate column | See section 7.4 |
| 1 | Food | `u16` | The primary need of report 15 |
| 2 | Drink | `u16` | The second need |
| 3 | Fuel | `u16` | Heating and cooking |
| 4 | Cloth | `u16` | The clothing need, and a trade staple |
| 5 | Tools | `u16` | Gates the occupation script of section 4.3 |
| 6 | Occupation good A | `u16` | Named by `OccupationDef.inv_slot_6` |
| 7 | Occupation good B | `u16` | Named by `OccupationDef.inv_slot_7` |

**Slots 6 and 7 are the important design move.** They give a merchant grain
and a smith iron **without storing a commodity identifier for each
individual**. The identifier lives in the occupation table, which has at most
a few hundred rows. Aggregation keys on `(occupation, slot)`, which is a
small fixed set of at most 512 pairs, so it stays a map plus a segmented
reduce rather than becoming a general scatter.

**This recovers most of Design B's flexibility at Design A's cost.** State it
as the reason the split exists, because it is not obvious.

### 7.4 Money is an `i32` column, not a `u16` slot

**Correct the lead's estimate here.** A `u16` caps a quantity at 65,535.

- For **food**, 65,535 is absurd. If one unit is one day of rations, an
  individual would carry 179 years of food. A `u16` is far more than enough.
- For **money**, 65,535 is far too small. Report 15 gives a *cohort* a
  `wealth: i64` field, and a cohort stands for hundreds or thousands of
  people.[^6] An individual whose savings cap at 65,535 cannot accumulate.

**Recommend `money: i32` as a separate column.** The range is about plus or
minus 2.1 billion, which is generous, and it is signed so that debt is
representable without a second column.

```
1,000,000 * 4 bytes (money i32)      =  4.0 MB
1,000,000 * 7 * 2 bytes (slots 1-7)  = 14.0 MB
                                       -------
Total individual inventory             18.0 MB
```

**18 MB.** For comparison, fog of war costs 21.0 MB for each faction.[^1] The
whole individual inventory is smaller than one faction's fog.

**Align the commodity index with report 15.** That report makes money
commodity slot 0 and excludes it from the transport solve.[^6] Keep money at
index 0 here so that the two subsystems share one commodity numbering.

### 7.5 Aggregation to the pyramid and to settlement pools

The owner wants individual banking with aggregation. This section gives the
mechanism.

**Two aggregations, one pass.** The individual stores aggregate along two
different keys. Run one scan and emit both, because the scan is the cost.

| Aggregate | Key | Rows | Accumulator |
|---|---|---|---|
| Level 1 plane, for the field layer | Level 1 cell index | 65,536 | `i64` |
| Settlement pool, for the economy | `pool` column | 5,000 | `i64` |

**The accumulator must be `i64`, and this is a hard invariant, not a
preference.** A `u16` field summed over one million individuals reaches
`6.5 * 10^10`, which overflows a `u32` by a factor of 15. The record already
states the rule: widen pyramid accumulators at level 1.[^1]

**The kernel.** The unit array is sorted by tile index, and the level 1 cell
index is a fixed shift of the tile index, so the array is **already grouped by
level 1 cell**. The aggregation is therefore a segmented reduce with no sort.

```
map:     read slot column, widen to i64
reduce:  segmented, boundaries at level 1 cell changes
scatter: one i64 write for each cell, in ascending cell order
```

Fix the span boundaries in ascending task order, exactly as report 12
specifies for its pooled reduce, so that no atomic is needed and the result
does not depend on the thread count.[^5]

**The settlement pool aggregation is not already grouped**, because a
settlement is not a contiguous run of tiles. Sort by the `pool` column, or
better, exploit that the pool identifier is monotone within a region and use
a small per-region gather. **Recommend the sort**, because 1,000,000 keys
radix-sort in 4 to 8 core-milliseconds and the aggregation runs at period 10,
so the amortised cost is 0.4 to 0.8 core-milliseconds. If that proves too
expensive, cache the sorted order and repair it incrementally, exactly as
report 10 repairs the tile sort.[^3]

**Cost.**

| Work | Traffic | Period | Core-ms for one pass | Core-ms for each tick |
|---|---|---|---|---|
| Read 7 `u16` slots plus money | 18 MB | 10 | 1.5 | 0.15 |
| Segmented reduce to 65,536 cells | 3.7 MB write | 10 | 0.3 | 0.03 |
| Pool reduce to 5,000 pools | 0.3 MB write | 10 | 0.2 | 0.02 |
| **Total** | | | | **0.20 core-ms** |

**The level 1 plane is 65,536 cells by 8 slots by 8 bytes, which is 4.2 MB.**
That plane is what the food-availability and drink-availability fields of
section 3.8 are built from, so this aggregation is not an extra cost added
for banking. **It is the same pass that feeds action selection.**

### 7.6 The conservation test

**Individual banking gives one exact integer invariant. Test it.**

> The sum of all individual money, plus all institution money, plus all
> pooled money, is constant under any sequence of `transfer` operations.

The sum is over `i64` accumulators and every operation is an integer
transfer, so the equality is exact and not a tolerance. Report 15 defines
three similar conservation tests for its own subsystem and this is a fourth
of the same kind.[^6]

**Run it every tick in a debug build and every 1,000 ticks in a release
build.** A drift in this sum means an unpaired write, and an unpaired write
is the failure mode that a flux-pair discipline exists to prevent.[^4]

---

## 8. Determinism

The lead names three dangers. This section confirms two, adds three more,
and gives the exact rule for each.

### 8.1 Field gathers are safe

**Confirmed.** A gather is a read. The intent pass reads the field planes,
the need columns and the occupation table, and it writes only the intent
column and the skill column, both of which are indexed by the individual's
own row. There is no shared write, so there is no atomic and no order
dependence.

**One rule protects this.** The field planes must not be written during the
intent pass. Place the field updates in the field-layer phase, which runs
before the execute phase, and place the intent pass in the execute phase.
The read and write phase split that the record already mandates gives this
for free.[^1]

### 8.2 The stagger must be a pure function of state

**Confirmed as a danger.** A counter that increments once for each visited
individual would produce a different assignment under a different thread
count, because the visit order across spans is not the completion order.

**The rule.**

> The stagger phase is `mix(cell_index) & (period - 1)`, where `mix` is a
> compile-time constant integer function. It reads no counter, no
> accumulator and no thread identifier.

The cell index is derived from the tile index, which is the individual's
position, which is simulated state. So the phase is a pure function of
simulated state, and simulated state is byte-identical across thread counts
by the record's own hash test.[^1]

### 8.3 The argmax tie-break

**Specified.** Scan the options in ascending option index with a strict
greater-than comparison. The lowest option index wins a tie.

```
best = 0; best_score = FLOOR;
for i in 0..N { if score[i] > best_score { best_score = score[i]; best = i; } }
```

The option indices are distinct by construction, so the order is total and
no second key is needed. **Do not use a keyed random draw to break the tie.**
It would be deterministic, but it adds a generator call to a loop that runs
one million times and it buys nothing that the field values do not already
express.

### 8.4 The greedy match sort key

**Specified.** The key is a triple and its last component makes the order
total.

> `(family_match descending, skill_tier descending, entity_id ascending)`

The `entity_id` is globally unique, so no two applicants compare equal. Use
the engine's stable radix sort. The slot side sorts by
`(rank ascending, slot_value descending, slot_id ascending)`, which is also
total.

**Settlements process in ascending settlement identifier order.** The
settlements are disjoint, so the passes may run in parallel, but the
**assignment events they emit must be concatenated in ascending settlement
order**, not in completion order. This is the record's existing rule for
ordering parallel results.[^1]

### 8.5 Three further dangers this report found

**Danger 1: skill accumulation is a saturating add, and a saturating add is
not associative at the cap.**

```
// clamp_add adds and clamps the result to the u16 maximum of 65535.
clamp_add(clamp_add(60000,  8000), -4000) = 61535
clamp_add(60000,  clamp_add(8000, -4000)) = 64000   // the two disagree
```

**The rule.** Skill accumulates only into the individual's own row and never
through a reduce. There is therefore no order question. **Write this into the
decision explicitly**, because a later optimisation that batches skill gain
into a segmented reduce would break replay only for individuals near the cap,
which is the hardest class of defect to find.

**Danger 2: a career transition can vacate a slot that another transition
then fills, within one tick.**

Two individuals in the same workplace both qualify for promotion. If the
first is applied and frees a slot, the second sees a different world than it
would if the order were reversed.

**The rule.** Collect candidate transitions in the execute phase as events.
Apply them in phase 6, the structural phase, in one ordered pass sorted by
`(workplace_id ascending, entity_id ascending)`. This is the same shape as
report 14's office cascade, which sorts each wave by entity identifier before
it expands.[^12]

**Danger 3: the occupation-parameterised inventory slots make the
aggregation key depend on a column.**

Slots 6 and 7 aggregate under the key `(occupation, slot)`. If two individuals
in the same level 1 cell hold different occupations, their slot 6 quantities
belong to different commodities and must not be summed.

**The rule.** Aggregate slots 6 and 7 into a `(occupation, slot)` keyed side
table of at most 512 rows, then fold that side table into the commodity
totals in ascending `(occupation, slot)` order. The fold is 512 additions
and it is serial. **Do not sum slots 6 and 7 into the cell plane directly.**

### 8.6 The determinism checklist for this subsystem

| Item | Hazard | Rule |
|---|---|---|
| Field gather | None. It is a read. | Fields are written in an earlier phase. |
| Stagger phase | A counter would break it. | `mix(cell) & (period-1)`. Pure function of state. |
| Argmax tie | Undefined without a rule. | Lowest option index, strict `>`. |
| Greedy match order | Not total without a third key. | `entity_id` ascending is the last key. |
| Settlement parallelism | Completion order. | Concatenate in ascending settlement order. |
| Skill accumulation | Saturating add is not associative. | Per-individual only. Never in a reduce. |
| Career transition | Order-dependent slot vacancy. | Phase 6, sorted by workplace then entity. |
| Inventory slots 6 and 7 | The key depends on a column. | A 512-row side table, folded serially. |
| Blocked-unit lateral step | Uses randomness. | Keyed generator, already specified.[^3] |

---

## 9. Cadence and the per-tick budget

### 9.1 The cadence table

The record requires a period and a phase offset for every system.[^1]

| Work | Period | Stagger key | Phase | Why this period |
|---|---|---|---|---|
| Intent re-evaluation | 32 | `mix(l1_cell)` | execute | Intent is sticky. Section 3.5. |
| Forced re-evaluation on a discontinuity | 1 | none, event-driven | execute | Under 1% of the population. |
| Skill accumulation | 32 | rides the intent pass | execute | Free. Multiply the gain by 32. |
| Food and drink availability fields | 4 | none | field layer | Stocks change on the economy tick. |
| Shelter field | 16 | none | field layer | Buildings change slowly. |
| Vacancy pressure fields, 8 families | 64 | none | field layer | Vacancies change on the job cadence. |
| Inventory aggregation to L1 and pools | 10 | none | economy | Shares the existing economy tick.[^6] |
| Job matching, sort and scan | 100 | `mix(settlement_id)` | execute | Matches the observed cadence in a shipped colony game.[^20] |
| Workplace reverse-index rebuild | 100 | none | barrier | Feeds the job match. |
| Career progression check | 512 | `mix(entity_id)` | execute, apply in phase 6 | A rare life event. |
| Skill-tier requantisation | 512 | rides career progression | execute | Only 8 bands, so it changes rarely. |

**Two stagger keys, and the choice is not arbitrary.** Use the level 1 cell
for a bandwidth-bound pass over the whole population, because it preserves
the sorted-array locality. Use the entity identifier for a sparse pass over a
small subset, because there is no locality to preserve and the key is
simpler.

### 9.2 The per-tick budget

| Work | Scale | Period | Core-ms for one pass | **Core-ms for each tick** |
|---|---|---|---|---|
| Intent re-evaluation, `N` of 6 | 1,000,000 | 32 | 3.4 | **0.17** |
| Forced re-evaluation | about 5,000 | 1 | 0.02 | **0.02** |
| Skill accumulation | rides the intent pass | 32 | 0.6 | **0.02** |
| Four new field planes | 4 by 65,536 cells | 4 to 64 | — | **0.034** |
| Vacancy fields, 8 families | 8 by 65,536 cells | 64 | 0.096 | **0.002** |
| Job matching | 20,000 applicants | 100 | 1.0 | **0.010** |
| Workplace reverse index | 1,000,000 column | 100 | 5.0 | **0.050** |
| Career progression | about 2,000 active | 512 | 0.05 | **0.001** |
| Inventory aggregation | 18 MB read | 10 | 2.0 | **0.200** |
| **Total** | | | | **0.51 core-ms** |
| **Wall-ms at 12 cores** | | | | **0.04 to 0.06** |

**The whole individual-agency subsystem costs about 0.5
core-milliseconds for each tick.** Against a tick of 12 to 46
wall-milliseconds it is under half of one percent.

**The two largest lines are not the decisions.** They are the inventory
aggregation at 0.20 and the intent pass at 0.17. Both are bandwidth over the
one million rows, not algorithmic work. **That is the correct shape for a
subsystem at this scale: the cost is the size of the population, not the
cleverness of the choice.**

**The line to add to the running budget table.**

| Subsystem | Cost | Source |
|---|---|---|
| Individual agency and occupations | 0.51 core-ms, 0.04 to 0.06 wall-ms | Report 16 |

### 9.3 Storage

| Item | Bytes for each individual | Total at 1,000,000 |
|---|---|---|
| Money, `i32` | 4 | 4.0 MB |
| Inventory slots 1 to 7, `u16` | 14 | 14.0 MB |
| Occupation, `u16` | 2 | 2.0 MB |
| Workplace, `u32` | 4 | 4.0 MB |
| Skill, `u16` | 2 | 2.0 MB |
| Intent, `u8` | 1 | 1.0 MB |
| Script state, `u8` | 1 | 1.0 MB |
| Rank and skill tier, packed into an existing flag byte | 0 | 0 |
| **Per-individual total** | **28** | **28.0 MB** |

| Shared structure | Size |
|---|---|
| Occupation table, 256 rows at 32 bytes | 8 KB |
| Career edge list, a few thousand at 8 bytes | tens of KB |
| Work slot rows, 200,000 at 8 bytes | 1.6 MB |
| Workplace reverse index, compressed sparse row | 8.0 MB |
| Vacancy field planes, 8 by 64 KB | 0.5 MB |
| Four new potential planes, 4 by 64 KB | 0.25 MB |
| Level 1 inventory aggregate, 65,536 by 8 by `i64` | 4.2 MB |
| **Shared total** | **about 14.6 MB** |

**Total: about 43 MB.** That is twice one faction's fog of war and it is
dominated by the 18 MB inventory and the 8 MB reverse index.[^1]

---

## 10. Prior art

The project's citation checks have repeatedly found that game implementation
claims are documented only on community wikis, with no developer
source.[^22] This section marks each claim by what supports it.

### 10.1 Utility-based action selection

**Verified as a design tradition.** Dave Mark's book is the standard
treatment of utility-based decision making for game agents, and it develops
the response-curve and weighted-combination structure that section 3.2
uses.[^8] The Infinite Axis Utility System is the shipped architecture built
on that structure, presented at the Game Developers Conference artificial
intelligence summit.[^9] The *Game AI Pro* series carries chapter-length
treatments of the same family.[^10]

**What this engine takes.** The three-part structure: considerations,
combination, selection. **What this engine changes.** It moves the
consideration evaluation out of the agent and into a field that is computed
once for 65,536 cells instead of once for each agent for each option. That is
the whole cost argument of section 3.4.

**What this engine gives up.** A published utility system evaluates a
non-linear response curve for each consideration. This engine evaluates a
linear product. Non-linearity is recoverable by baking the curve into the
field source rather than into the agent, but the curve is then shared by
every individual in the cell.

### 10.2 Planning, as the contrast case

**Verified.** Goal-Oriented Action Planning derives from STRIPS-style
regressive planning and its shipped game form is documented by its author in
a Game Developers Conference paper.[^11] It is the correct reference for what
a gradient cannot do, and section 4.1 is the list of exactly those
capabilities.

### 10.3 The Sims motive and advertisement model

**Mark as partly unverifiable.** The design is widely described: objects
advertise a satisfaction value for a motive, the agent scores the
advertisements against its own motive decay, and the behaviour lives in the
object rather than in the agent. **The last part is the idea worth taking.**

A developer-authored account of the object-programming architecture exists
and is the closest thing to a primary source.[^23] **Precise decay rates and
advertisement formulas are not published.** Do not cite specific numbers.

**What this engine takes.** The inversion: the behaviour lives in the world,
not in the agent. A field is exactly an advertisement broadcast over space,
and the occupation weight vector is exactly the agent's motive weighting.
**The engine's design is the same architecture with the advertisement
aggregated spatially instead of enumerated per object.** That aggregation is
what makes it affordable at one million.

**What this engine gives up.** A specific object cannot advertise a specific
interaction. The field carries only "food is available here". It does not
carry "this particular oven will give you 40 units of hunger relief". The
recovery is the occupation script of section 4.3, which names the interaction
at bake time.

### 10.4 Dwarf Fortress labour and job assignment

**Mark as community-sourced, with one verified performance claim.** The
labour-preference and job-priority system is documented only on the community
wiki. **Do not cite it as an implementation.**

One claim from the neighbouring report is worth repeating because it is
counter-intuitive and it is developer-sourced: **the cost driver in that game
is the item stack count, not the stack size**, because stacks drive hauling
jobs, stockpile scans and paths.[^20] The published cost profile puts over 60
percent of processing in units taking their turns, of which under 10 percent
is pathfinding.

**The lesson for this project.** Job generation is the scaling risk, not job
matching. Section 5.6 costs the matching at 0.06 core-milliseconds, which is
nothing. If a design later generates one hauling job for each item stack,
**the job count becomes the population count and the whole analysis of
section 5 must be redone.** Record that as a constraint on content, not as an
engine parameter.

### 10.5 RimWorld work priorities

**Mark as community-sourced.** The work-priority grid, in which a colonist
carries a priority from 1 to 4 for each of a fixed set of work types, is
documented on the community wiki. No developer specification is published.

**What the structure confirms.** A fixed small set of work types with an
integer priority for each is playable and legible. **That is exactly the
occupation weight vector of section 6.1.** The engine's version differs in
that the weights come from the occupation table rather than from a player
edit, because a player cannot edit one million grids.

### 10.6 Victoria-series employment

**Mark as community-sourced.** The population-unit model, in which a
population unit holds a size, a profession and a wealth level, and in which
professions shift in response to wages, is documented on community wikis.
Report 15 already takes the cohort structure from it.[^6]

**The relevant difference.** In that model a population unit *changes
profession* by a gradual transfer of size between two units. In this engine
an individual changes occupation by a career-graph edge, which is a discrete
event on one row. **The engine's form is strictly more expressive and it
costs less**, because the transfer needs a rate and the edge needs a
predicate.

### 10.7 Walker models in city builders

**Mark as community-sourced.** The Caesar and Anno families route individual
walkers along roads from a producing building, and a building is served when
a walker passes it. No developer figures for agent counts are published.[^22]

**What this engine takes.** The confirmation that a scripted walker with no
plan produces legible city behaviour. That is the argument for the occupation
script of section 4.3, and it is the strongest one available, because the
mechanism is visible to any player.

**What this engine rejects.** The road-only routing. This engine has a real
hex map with a floor-field movement model, so a walker follows a field and
not a road graph.[^3]

### 10.8 The assignment literature

**Verified.** The Hungarian method is Kuhn's, with Munkres giving the
polynomial-time analysis.[^14][^15] The shortest-augmenting-path form is
Jonker and Volgenant's, and it is the practical choice for a dense instance
of a few thousand rows.[^16] The auction algorithm is Bertsekas's.[^17]
Stable matching is Gale and Shapley's.[^19] The standard monograph on the
whole family is Burkard, Dell'Amico and Martello.[^18]

**The result that this report relies on.** The rearrangement inequality
states that the sum of products of two sequences is maximised when both are
sorted in the same order.[^7] That is the theorem that makes the sorted
greedy match exact for a rank-1 benefit matrix, and it is the single most
useful citation in this report, because it converts an approximation into an
exact method under a stated condition.

The generalisation is the Monge property, under which a greedy method solves
a transportation problem exactly. The survey of Monge properties in
optimisation gives the conditions.[^21]

### 10.9 The pedestrian model behind the movement rule

**Verified.** The floor-field cellular automaton of Burstedde, Klauck,
Schadschneider and Zittartz gives each pedestrian a transition weight for
each neighbour built from a static field and a dynamic field, and it is
validated against measured pedestrian data.[^24] Report 10 adopts it for the
movement rule.[^3]

**Why it matters here.** The intent that this report computes is the input to
that model. The gradient selects *which* static field to descend. The floor
field selects *which neighbour tile* to step to. The two are the same idea at
two scales, which is why they compose without an adapter.

### 10.10 What the survey establishes

Three things.

1. **The method is not novel and it should not be.** Utility-based selection
   is a mature tradition with a shipped architecture and a book.
2. **The novelty, such as it is, is the amortisation.** Moving the
   consideration evaluation into a level 1 field is what takes the cost from
   400 nanoseconds to 4. That is an engineering move, not a new model.
3. **Every claim about a shipped game in this section is documented
   behaviour, not implementation.** Two exceptions are marked as
   developer-sourced: the object-programming account and the stack-count
   performance guidance.

---

## 11. Ready-to-apply decision block

**Copy this section into the decision record. It uses the assigned range
D110 to D129. It does not renumber anything and it collides with nothing.**

### Part K — Individual agency and occupations

#### D110. Action selection is a field gradient, not a search

An individual chooses what to do by reading `N` potential field values at its
own level 1 cell, weighting each by its own unmet need and by a weight from
its occupation, and taking the argument of the maximum.

```
cell = l1_cell_of(individual.tile)
w    = occupation_table[individual.occupation].need_weight
score[i] = ((individual.need[i] * w[i]) >> 16) * field[i][cell]
intent   = argmax(score)
```

`N` is 6. The fields are food availability, drink availability, shelter,
safety, work vacancy pressure and social draw. Two of the six already exist:
safety is the complement of the military presence field, and social draw is
the price potential field.

**Cap `N` at 8.** Each field adds 64 KB to the resident working set, and
above 8 the working set no longer fits beside the need array in a 1 MB level
2 cache.

The cost is 3.4 core-milliseconds for a full pass at `N` of 6, which is 3.4
nanoseconds for each individual. **This is a factor of about 98 cheaper than
a decision that gathers from a scattered structure**, and the whole factor is
that the units are sorted by tile and the fields are level 1 planes.

#### D111. Correct the decision cost figure in the needs subsystem

The needs and economy decision is costed at 400 nanoseconds, which gives 16.0
core-milliseconds for 40,000 cohorts and makes the decision pass 92 percent
of that subsystem's cost. **That figure is correct only for a decision that
gathers from a scattered structure.**

Under D110's formulation the same decision costs under 5 nanoseconds. **The
cohort decision line falls from 16.00 core-milliseconds to under 0.05.**
Apply the correction when the two reports merge. The needs subsystem's total
then falls from 1.74 to 1.81 core-milliseconds to about 0.20.

**This does not change the cohort recommendation.** Cohorts remain correct
for needs, consumption and exchange. It changes only what the decision tier
costs, and it removes the argument that decisions are unaffordable at the
individual scale.

#### D112. Intent is sticky. Re-evaluate every 32 ticks, staggered by the level 1 cell

The stagger phase is a fixed integer mix of the level 1 cell index.

```rust
const fn stagger_phase(cell: u32, period_log2: u32) -> u32 {
    let mut x = cell.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x & ((1 << period_log2) - 1)
}
```

**Do not stagger by the entity identifier.** The unit array is sorted by
tile, so an identifier stagger scatters the active set through a 16 MB array
and costs 0.5 to 0.7 core-milliseconds instead of 0.17.

The mix is required. A bare `cell & 31` re-evaluates a regular spatial stripe
of the map on each tick and produces visible banding.

**An individual that crosses a cell boundary may re-decide after 1 tick or
after 63.** This is accepted behaviour, not a defect. Re-deciding on arrival
in a new region is correct.

**Three discontinuities force an immediate re-evaluation:** employment ends,
the workplace is destroyed, and a need crosses a critical threshold. Consume
the existing sparse threshold-crossing list and set a re-evaluate flag.

#### D113. The utility score is an `i64` in Q16.16 and it is never stored

Inputs are Q16.16 for the need and the weight, and Q8.0 for the field value.
Compute in `i64` with one shift. The worst case needs 26 bits.

**A weighted sum with integer weights is exactly reproducible for any
evaluation order and any thread count.** The score never enters simulated
state, because it is a transient comparison quantity.

#### D114. The argmax tie-break is the lowest option index

Scan the options in ascending index with a strict greater-than comparison.
The option indices are distinct, so the order is total.

**Do not break the tie with a keyed random draw.** It would be deterministic
but it adds a generator call to a loop that runs one million times.

#### D115. A score floor stops movement. A flat field is the failure mode

When the largest score is below a bake-time floor, hold the current intent
and do not move. Set the floor at `1 << 14` in Q16.16.

**Without the floor a flat field produces a correlated population-wide
walk.** Every individual becomes a mover. The movement subsystem is costed
against 300,000 movers out of one million, so its cost rises by a factor of
about 3.3, and the mover-subset sort degrades from 2 to 4 core-milliseconds
back to the full 4 to 8.

**The floor is a frame-budget parameter, not a design tuning knob.** Changing
it changes the mover count. Record it as such.

#### D116. What the field gradient cannot express, and the test that decides

The gradient cannot express: a multi-step plan with a required order,
deferred gain, precondition acquisition, saving, negotiation, joint
commitment, and any goal whose target does not exist yet.

**The test. Ask three questions about a desired behaviour.**

1. Can you name a single number that is highest exactly where the behaviour
   should be strongest?
2. Does the behaviour still make sense if the individual forgets everything
   between one tick and the next?
3. If two individuals both do it, do they interfere only through crowding and
   scarcity?

**Three yes answers: add a field plane and a weight column.** Any no answer:
the behaviour needs a sequence or a memory, and it belongs in the character
tier.

#### D117. An occupation carries a fixed short script of at most 8 states

A script is a small cyclic state machine authored at bake time. The
individual stores one `u8` script state, which is 1 MB at one million
individuals. Each state names one field to descend and one condition that
advances the state.

**This is not a planner.** There is no search and no goal. It recovers the
daily round, the commute, the shift and "fetch the tool first", which are the
behaviours a player watches, at a cost of 1 MB and one compare.

It cannot adapt to a changed world. A scripted individual walks to where the
market was until an advance condition fires or the field routes it elsewhere.

#### D118. No exact assignment method is usable. Decompose the problem

A dense instance of one million people against 100,000 slots holds `10^11`
arcs. Any exact method must read the arcs. At 4 bytes each that is 400 GB,
which is 10 seconds of traffic before the algorithm starts.

| Method | Largest `n` inside a 4 core-ms budget |
|---|---|
| Hungarian, Kuhn and Munkres | **about 340** |
| Jonker-Volgenant | about 1,100 |
| Gale-Shapley | about 6,300 |
| Bertsekas auction | tractable when sparse, but see below |

**The auction algorithm is rejected for a further reason.** Its epsilon
scaling gives a data-dependent iteration count, and a data-dependent
iteration count breaks a fixed frame budget. Keep it for caravan assignment
at a few thousand bidders, where it is already recommended.

**The problem cannot be posed at this scale. It must be decomposed into a
global spatial part and a local matching part.**

#### D119. Global routing is a vacancy field over 8 job families. Local matching is a sort

**Part 1.** Each unfilled slot emits a source proportional to its value. The
field diffuses at level 1. Individuals whose intent is work descend it.

**Use 8 job families, not one plane for each occupation.** Eight `u8` planes
are 512 KB and update in 96 microseconds at a period of 64 ticks, which is
1.5 core-microseconds for each tick. Sixty-four planes would need 768
microseconds for one update and would exceed the whole field-layer budget.

**Part 2.** At each settlement, sort the applicants by
`(family_match descending, skill_tier descending, entity_id ascending)`, sort
the slots by `(rank ascending, slot_value descending, slot_id ascending)`, and
scan both in parallel.

Job matching runs at period 100, staggered by an integer mix of the
settlement identifier. **The whole job subsystem costs 0.06 core-milliseconds
for each tick**, of which 0.05 is the reverse-index rebuild.

#### D120. The local match is exactly optimal when the benefit factorises

If the benefit of putting person `i` in slot `j` is `skill(i) * value(j)`,
the benefit matrix has rank 1 and is non-negative. **Sorting both sides and
matching in order then maximises the total exactly.** This is the
rearrangement inequality. Sorted greedy is not an approximation under this
condition.

**Design the content so that the benefit factorises: score a slot by one
value number and a person by one skill number.** The condition breaks when a
slot needs a specific skill rather than more skill, and the family filter is
what fixes that: a person outside the family is not a candidate at all.

**The whole quality loss therefore lives in the global routing**, in two
forms:

- **Level 1 indistinguishability.** An individual cannot tell two workplaces
  apart inside the same 256-tile cell. **The loss is bounded by the spread of
  slot values inside one level 1 cell.**
- **Migration lag.** Crossing one level 1 cell takes at least 256 ticks and
  equalising a 256-cell map takes on the order of `10^4` ticks. **The labour
  market responds over game-months.** This is a feature, not a defect.

**Do not quote an optimality gap percentage.** It depends entirely on
content. **Measure it** with a regression test: freeze a 2,000 by 2,000
instance, run Jonker-Volgenant offline for the optimum, run the field and
greedy method to steady state, and record the benefit ratio against a
threshold.

#### D121. Unemployment needs no special case. A flat vacancy field does

An individual with no slot keeps an unmet work need, so its intent stays
"seek work" and it continues to descend the vacancy gradient. Nothing in the
code tests for unemployment. The consequences arrive through the needs
system: no slot means no wage, no wage means no money, no money means unmet
food.

**The one rule needed is D115's floor.** When vacancy pressure is zero
everywhere, an unemployed population must settle rather than wander.
**Without the floor, mass unemployment becomes mass migration, and mass
migration triples the movement cost.**

#### D122. An occupation is a `u16` index. A career path is a directed acyclic graph

The occupation table row is 32 bytes and holds the family, the script index,
two inventory slot mappings, six need weights, a base stat row, a wage band
and flags. At 256 occupations the table is 8 KB and stays in the level 1
cache.

**Do not cap the occupation count below 65,536.** The `u16` is the cap. The
count that must be capped is the **family** count at 8, because a family
costs a field plane and a field update.

A career edge is 8 bytes and holds the source occupation, the destination
occupation, a minimum skill tier, a minimum age, a required rank and a
required building kind. Store the edges as a compressed sparse row structure
indexed by the source occupation.

**Check acyclicity at bake time with one topological sort. Reject a cycle.**
A run-time check would need a visited set for each individual.

The progression check is a filter, a gather and a scan at period 512,
staggered by an integer mix of the entity identifier. It costs 0.001
core-milliseconds for each tick.

**Here the stagger key is the entity identifier, not the cell**, because the
pass is sparse and has no locality to preserve. **Two stagger keys exist in
this subsystem and the choice is stated: cell for a dense pass, identifier
for a sparse one.**

#### D123. Skill is a `u16`, quantised to 8 tiers before it enters the stat pipeline

Store the fine value as a `u16` so that progression is smooth over a career.
Derive an 8-band tier and pack it into an existing flag byte. The tier
changes only at the career cadence of 512 ticks.

**Two individuals at the same tier work identically.** Record that as a
visible design choice, exactly as the health quantisation into 8 bands is
recorded.

**Skill accumulates by a saturating add into the individual's own row only,
and never through a reduce.** A saturating add is not associative at the cap.
A later optimisation that batches skill gain into a segmented reduce would
break replay only for individuals near the cap, which is the hardest class of
defect to find.

#### D124. The 4-slot post-stage multiplier budget holds. Merge, do not raise the cap

Six candidates compete for 4 slots: health, veterancy, terrain, skill,
fatigue and morale. **Do not raise the cap.** Six slots would cost about 18
core-milliseconds at one million entities and 3 evaluated fields, against 12
at four.

**Three merges resolve it.**

1. **Veterancy and skill are the same axis.** An individual holds one
   occupation at a time. A soldier's skill is its veterancy. **Merge into one
   `proficiency_tier`, fed by the single `skill` column of D123.**
2. **Health and fatigue are the same axis.** Both answer how fit the body is.
   **Merge into one `condition_tier` through an 8 by 8 lookup table** of 64
   bytes. The cost is one byte load, not a multiply.
3. **Morale does not scale a rate. It gates a behaviour.** Whether an
   individual routs, obeys or revolts is a threshold crossing. **Move morale
   out of the multiplier budget and into the existing two-phase
   threshold-predicate pass.**

**The four slots are then: `condition_tier`, `proficiency_tier`, `terrain`,
and one spare.** No change to the per-entity schema rule is needed. The
neighbouring report's worked example should be updated to name
`condition_tier` and `proficiency_tier`.

#### D125. A workplace carries 4 ranked slots by default, capped at 8. Holders are anonymous

A slot row holds a holder identifier, an occupation, a rank and flags, at 8
bytes. At 50,000 workplaces and a mean of 4 slots that is 1.6 MB.

**Cap the slot count at 8, because a tile holds at most 8 units.** A
workplace on one tile cannot usefully hold more workers than can stand in it.
Making the two limits agree removes a class of content bug.

**Slot-holders are anonymous rows, not named entities.** Three reasons.

1. The character tier declares a ceiling of 262,144. Two hundred thousand
   shopkeepers would consume 76 percent of it and leave nothing for the
   nobility.
2. A promotion path already exists: filter, sort, budget. **A slot-holder
   does not need to be named. It needs to be promotable.** A master craftsman
   who accumulates deeds is exactly that case.
3. The forward-column-plus-rebuilt-reverse-index pattern is already chosen
   for formation membership, and the same argument applies without change.

**Add a `workplace: u32` column at 4 MB and pack `rank` into an existing flag
byte.** Build the reverse index by counting sort into a compressed sparse row
structure at the job-matching cadence. **Reuse the existing kernel. Do not
write a second one.**

**A rank is not an office.** A workshop rank has no succession law, no claim
and no policy column. When the master dies, the local match refills the slot
at the next matching tick from whoever is present.

#### D126. The individual inventory is 8 fixed slots and 18 MB

**Use fixed slots, not `(commodity, quantity)` pairs.** A pair form costs 24
MB instead of 18 and, more importantly, turns the aggregation from a map plus
a segmented reduce into a general scatter, which is slower and harder to make
deterministic.

| Slot | Contents | Type |
|---|---|---|
| 0 | Money | `i32`, separate column |
| 1 to 5 | Food, drink, fuel, cloth, tools | `u16` |
| 6 and 7 | Occupation goods A and B | `u16` |

**Slots 6 and 7 are named by the occupation table, not by a per-individual
identifier.** This gives a merchant grain and a smith iron while keeping the
aggregation key to at most 512 `(occupation, slot)` pairs.

**Money is an `i32`, not a `u16`.** A `u16` caps savings at 65,535, which is
far too small for an individual and absurdly generous for rations. An `i32`
is signed, so debt needs no second column. **Keep money at commodity index 0
to match the needs subsystem's numbering.**

```
1,000,000 * 4 bytes  (money)             =  4.0 MB
1,000,000 * 7 * 2    (slots 1 to 7)      = 14.0 MB
                                           -------
                                           18.0 MB
```

#### D127. Personal stores aggregate to level 1 and to settlement pools in one pass

The unit array is sorted by tile index and the level 1 cell index is a fixed
shift of the tile index, so **the array is already grouped by level 1 cell
and the aggregation needs no sort**. Fix the span boundaries in ascending
task order. No atomic is needed.

**The accumulator is `i64`.** A `u16` summed over one million individuals
reaches `6.5 * 10^10` and overflows a `u32` by a factor of 15.

The settlement pool aggregation is not already grouped, because a settlement
is not a contiguous run of tiles. Sort by the pool column at 4 to 8
core-milliseconds, at a period of 10.

**Aggregate slots 6 and 7 into a 512-row `(occupation, slot)` side table
first, then fold that table into the commodity totals in ascending key
order.** Do not sum them into the cell plane directly, because two
individuals with different occupations hold different commodities in the same
slot.

**The level 1 aggregate plane is 4.2 MB and it is the same structure that
feeds the food and drink availability fields of D110.** The aggregation is
not an extra cost added for banking.

The whole aggregation costs **0.20 core-milliseconds for each tick** at a
period of 10.

#### D128. One exact conservation test protects individual banking

> The sum of all individual money, plus all institution money, plus all
> pooled money, is constant under any sequence of `transfer` operations.

The sum is over `i64` accumulators and every operation is an integer
transfer, so the equality is exact and not a tolerance. Run it every tick in
a debug build and every 1,000 ticks in a release build. **A drift means an
unpaired write.**

#### D129. The cadence table and the per-tick budget for individual agency

| Work | Period | Stagger key | Core-ms for each tick |
|---|---|---|---|
| Intent re-evaluation, `N` of 6 | 32 | `mix(l1_cell)` | 0.17 |
| Forced re-evaluation on a discontinuity | 1 | event-driven | 0.02 |
| Skill accumulation | 32 | rides the intent pass | 0.02 |
| Four new potential fields | 4 to 16 | none | 0.034 |
| Vacancy fields, 8 families | 64 | none | 0.002 |
| Job matching | 100 | `mix(settlement_id)` | 0.010 |
| Workplace reverse index | 100 | none | 0.050 |
| Career progression | 512 | `mix(entity_id)` | 0.001 |
| Inventory aggregation | 10 | none | 0.200 |
| **Total** | | | **0.51 core-ms** |
| **Wall-ms at 12 cores** | | | **0.04 to 0.06** |

Storage is about 43 MB: 28 MB of per-individual columns and 15 MB of shared
structures.

**The two largest lines are the inventory aggregation and the intent pass.**
Both are bandwidth over one million rows, not algorithmic work. **That is the
correct shape at this scale: the cost is the size of the population, not the
cleverness of the choice.**

---

## 12. Open questions from this report

**OQ70. Does an individual decide, or does a cohort decide?** This is the one
structural conflict this report leaves open, and it needs the owner, not the
lead.

The owner has decided that a unit is an individual person with individual
experiences. The needs subsystem recommends that decisions run on 40,000
cohorts and states plainly that a unit does not decide, because a decision
costs 400 nanoseconds.

**This report removes the cost objection.** Under D110 a decision costs 3.4
nanoseconds and a full pass over one million individuals costs 0.17
core-milliseconds when staggered. **Individual decision is now affordable.**

Three positions are consistent and the owner must pick one.

| Position | Deciders | Cost | Consequence |
|---|---|---|---|
| A. Individuals decide | 1,000,000 | 0.17 core-ms | Emergent variance. Cohorts remain for consumption accounting only. |
| B. Cohorts decide | 40,000 | 0.007 core-ms | No variance inside a stratum. Everyone in a cohort acts identically. |
| C. Both | 1,040,000 | 0.18 core-ms | Individuals choose where to go. Cohorts choose what to buy. |

**This report recommends C.** An individual's *movement* intent should be
individual, because that is what makes a crowd look like a crowd. A
population's *consumption* choice can be a cohort choice, because the
individual variation in what someone eats is not visible. Position C costs
almost nothing over position B and it delivers the individual experience the
owner asked for.

**OQ71. What is the total population, and is it the same one million?** The
needs subsystem's cohorts represent a population that is separate from and
larger than the one million units. If the one million units *are* the
population, then the cohort layer is a summary of the same rows rather than a
second population, and several figures in both reports change. **State
whether the world holds one million people or one million soldiers plus
tens of millions of civilians.**

**OQ72. How many workplaces exist?** Section 6.5 assumes 50,000. Every slot
and reverse-index figure scales with this number. It is related to the
already-open settlement count question.

**OQ73. How many occupations ship in version 1?** The table is a `u16`, so
the engine does not care. The **family** count is fixed at 8 by D119, so the
real question is whether 8 families partition the occupations that content
wants. Name the eight before the field planes are allocated.

**OQ74. What is the measured optimality ratio of the greedy match?** D120
specifies the measurement but the answer needs content that does not exist.
**Set the regression threshold after the first measurement, not before.**

**OQ75. Are the two occupation-parameterised inventory slots enough?** Slots
6 and 7 give each occupation two goods. A merchant plausibly wants more. The
alternative is a third and fourth parameterised slot at 4 MB, or the pair
form of section 7.2 at 6 MB and a worse kernel. **Decide before the
aggregation kernel is written**, because the pair form changes it.

**OQ76. Does an individual own a building, or only hold a slot in one?**
The character subsystem gives a character an owner column and a rebuilt
reverse index for assets.[^12] This report gives an individual a workplace
column and a rank. Ownership by an anonymous individual is a third relation
and it is not specified. **If a smith owns the forge, that is ownership. If
the smith works in another's forge, that is a slot.** The two relations need
different columns.

**OQ77. What is the script vocabulary?** D117 fixes the script at 8 states
but does not say what a state may do. A state names a field to descend and a
condition to advance. **Enumerate the conditions.** Arrival, a timer, a
threshold on an inventory slot, and a clock phase cover the walker
behaviours. A richer condition set moves the script toward a behaviour tree
and toward a per-individual cost that this report has not budgeted.

**OQ78. Is there a day-night clock, and how many ticks are in a day?** The
occupation script needs one to express a shift. The needs subsystem already
asks the same question for its decay rates. **One answer serves both.**

**OQ79. Does skill decay?** D123 accumulates skill by a saturating add and
never decreases it. A world with no skill decay accumulates a population of
maximum-tier masters over a long game, which flattens the tier distribution
and removes the meaning of the greedy match's sort key. **A slow decay, or a
cap tied to the workplace rank, prevents that.** This is a content question
with an engine consequence, because a decay is a second map kernel over one
million rows.

---

## References

**Verification status.** Footnotes 7, 14 to 19, 21 and 24 are standard
published works and their bibliographic details are stable. Footnotes 8, 9,
10 and 23 concern game industry sources and a concurrent citation check did
not return in time. **Confirm the exact title, year, venue and location of
each of those four before publication.** The technical claims that they
support do not depend on the exact citation, because each is marked in
section 10 as a design tradition or as community-sourced.


[^1]: Cachette project instructions, sections "Hard invariants", "Design principles" and "Target platform". `CLAUDE.md`
[^2]: Merge notes for ADR-0001, sections 2 and 3, owner and lead decisions. `docs/research/reports/MERGE-NOTES.md`
[^3]: Research report 10, Crowd Simulation and Unit Movement, sections 4.1 to 4.8 and 5.1. `docs/research/reports/10-crowd-and-movement.md`
[^4]: Research report 13, Field Operator Algebra, sections 5.2, 9.3, 9.4 and 9.5. `docs/research/reports/13-field-operator-algebra.md`
[^5]: Research report 12, Entity Economy and Modifiers, sections 2.2, 3.2, 3.5, 5.3 and 8.1. `docs/research/reports/12-entity-economy-and-modifiers.md`
[^6]: Research report 15, Needs, Consumption and the Economy, sections 4.3, 5.2, 5.3, 8.1, 12.1, 12.3 and open question 69. `docs/research/reports/15-needs-consumption-and-economy.md`
[^7]: Hardy, G. H., Littlewood, J. E. and Polya, G., 1934. *Inequalities*. Cambridge University Press, theorem 368, the rearrangement inequality.
[^8]: Mark, D., 2009. *Behavioral Mathematics for Game AI*. Course Technology, Cengage Learning. **Confirm the publisher imprint and the ISBN before publication.**
[^9]: Mark, D. and Dill, K., 2010. "Improving AI Decision Modeling Through Utility Theory". Game Developers Conference AI Summit. **Confirm the year and the exact talk title, and add the separate Infinite Axis Utility System talk, before publication.**
[^10]: Rabin, S., editor, 2013 onward. *Game AI Pro* series. CRC Press. Chapters on utility-based and needs-based decision making. **Cite one named chapter, not the series, before publication.** http://www.gameaipro.com
[^11]: Orkin, J., 2006. "Three States and a Plan: The AI of F.E.A.R.". Game Developers Conference 2006. http://alumni.media.mit.edu/~jorkin/gdc2006_orkin_jeff_fear.pdf
[^12]: Research report 14, Character Graph and Inheritance, decisions D70, D77, D78, D83, D88 and D89. `docs/research/reports/14-character-graph-and-inheritance.md`
[^13]: Arm Ltd. *Arm Neoverse N1 Software Optimization Guide*, instruction throughput and memory system tables. https://developer.arm.com/documentation/swog309707/latest
[^14]: Kuhn, H. W., 1955. "The Hungarian method for the assignment problem". *Naval Research Logistics Quarterly*, 2(1-2), pp. 83-97. https://doi.org/10.1002/nav.3800020109
[^15]: Munkres, J., 1957. "Algorithms for the Assignment and Transportation Problems". *Journal of the Society for Industrial and Applied Mathematics*, 5(1), pp. 32-38. https://doi.org/10.1137/0105003
[^16]: Jonker, R. and Volgenant, A., 1987. "A shortest augmenting path algorithm for dense and sparse linear assignment problems". *Computing*, 38(4), pp. 325-340. https://doi.org/10.1007/BF02278710
[^17]: Bertsekas, D. P., 1988. "The auction algorithm: a distributed relaxation method for the assignment problem". *Annals of Operations Research*, 14(1), pp. 105-123. https://doi.org/10.1007/BF02186476
[^18]: Burkard, R., Dell'Amico, M. and Martello, S., 2009. *Assignment Problems*. Society for Industrial and Applied Mathematics.
[^19]: Gale, D. and Shapley, L. S., 1962. "College Admissions and the Stability of Marriage". *The American Mathematical Monthly*, 69(1), pp. 9-15. https://doi.org/10.2307/2312726
[^20]: Research report 11, Resource and Trade Flow, sections 3.3, 3.4 and 13.5. `docs/research/reports/11-resource-and-trade-flow.md`
[^21]: Burkard, R. E., Klinz, B. and Rudolf, R., 1996. "Perspectives of Monge properties in optimization". *Discrete Applied Mathematics*, 70(2), pp. 95-161. https://doi.org/10.1016/0166-218X(95)00103-X
[^22]: Merge notes for ADR-0001, section 12, soft citations to verify before publication. `docs/research/reports/MERGE-NOTES.md`
[^23]: A developer-authored account of object programming in The Sims, used in a Northwestern University course on game design. **This citation is unconfirmed. Establish the exact author, title and location before publication, or remove the footnote and mark the claim as community-sourced.**
[^24]: Burstedde, C., Klauck, K., Schadschneider, A. and Zittartz, J., 2001. "Simulation of pedestrian dynamics using a two-dimensional cellular automaton". *Physica A: Statistical Mechanics and its Applications*, 295(3-4), pp. 507-525. https://doi.org/10.1016/S0378-4371(01)00141-8
