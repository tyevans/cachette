# What a policy cannot see

This report audits the observation the engine publishes for one faction. It
answers two questions. How much of the published width is inert? And what does
the simulation compute that a policy cannot read?

The audit read the source. **It ran no engine probe**, because four other
agents held the machine and one took throughput measurements. The load average
was 2.6 over one minute when the audit started. Every count below therefore
comes from reading a write site, and not from an array a world produced.

## 1. What the array holds

The layout declares 93 fields and 4,819 positions.[^1] The audit derived the
total from the field table and the block constants, and it matches the figure
the project already carries.

| Block | Positions |
|-------|-----------|
| Egocentric ring stack | 3,775 |
| Entity tokens | 618 |
| Frontier by sector | 32 |
| Every scalar field | 394 |
| Total | 4,819 |

The ring stack holds 25 channels over 151 cells. The token block holds four
sets: 8 settlements over 24 channels, 6 rivals over 23 channels, 8 threat
clusters over 20 channels, and 8 candidate sites over 16 channels.

## 2. The headline: the reserved fields are the honest part, and they are small

42 fields carry the value form `reserved`. Those 42 fields hold **121
positions**, which is 2.5 percent of the width. A reserved field declares
bounds of zero and zero, so a reader can find it.[^2]

**A further 1,401 positions read zero forever and declare a real value form.**
That is 29 percent of the width, and it is eleven times the reserved count. One
more position holds the constant one.

| Category | Positions | Declared honestly |
|----------|-----------|-------------------|
| Reserved fields | 121 | Yes |
| Ring channels dead in every cell | 453 | No |
| Ring channels dead beyond ring 3 | 600 | No |
| Token channels never written | 268 | No |
| Good-class positions above class 0 | 71 | No |
| Objective weight positions above the fifth | 7 | No |
| Upgrade legality above the last category | 1 | No |
| Frontier reach headroom | 1 | No |
| Inert total | 1,522 | |

**The reserved count is not the measure of the problem, and it never was.** It
is the measure of the part somebody wrote down. The answer to "how much more of
this is there" is: eleven times as much, and none of it is marked.

The rest of this section gives the evidence for each row.

### 2.1 The engine holds one commodity, and the layout holds eight good classes

`COMMODITY_COUNT` is one.[^3] The work commodity table maps all three resource
kinds onto commodity zero, so `good_class_of` returns zero for every kind.[^4]

Five fields hold one position for each of eight good classes: the stock, the
stock share, the net flow, the production rate and the consumption rate. The
settlement scan fills an array of one element, and the class index reads that
array. Classes 1 to 7 therefore read zero in all five fields. That is 35
positions.

The trade board holds 40 positions, as eight classes over five statistics. The
board pass takes the class of each advert through the same mapping, so every
advert lands in class zero. Classes 1 to 7 read zero in all five statistics.
That is 35 more positions. The fifth statistic of class zero is the price
change over a window, which reads zero as well. The board therefore holds 4
live positions of 40.

`stock_share_of_class` at class zero divides the stock of commodity zero by the
whole stock of the faction, and the whole stock is the stock of commodity zero.
**The position holds the constant one in every state of every world.** The
layout already reserved `held_inside_reach_share` for exactly this reason, and
the doc of that field states the rule: a slot that holds one value in every
state is inert, and a policy spends capacity on it.[^5]

**None of the 71 dead good-class positions carries a warning.** Four fields
declare the form `magnitude`, one declares `share`, and the board declares
`statistic`. A reward term that read the food stock of class 3 would read a
real form and a real bound, and it would get zero.

### 2.2 Three ring channels are hardcoded to zero

The channel table gives each of 25 channels one expression. Three of them are
the literal `Fix32::ZERO`: `memory_age`, `own_strength` and
`rival_strength`.[^6] They read zero in all 151 cells, which is 453 positions.

The field declares the form `statistic`, so the schema publishes bounds of
minus one and one for all 453. **This is the single largest inert region of the
observation, and it is nine percent of the whole width.**

The two strength channels are the ring-stack twins of `military_strength`,
which the scalar block reserves with a stated reason. The reason is good: no
record says how to combine attack and armour.[^5] The ring stack states no
reason and declares no reserve.

### 2.3 Five more ring channels go dark beyond ring 3

The near pass reads level 0 tiles and covers rings 0 to 3, which is 31 cells.
The far pass reads block summaries of the level 1 lattice and covers rings 4 to
13, which is 120 cells.

The far pass adds nine totals to a cell: the admitted tiles, the tiles seen
now, the open tiles, the height total, the food total, the value total, the
rival held tiles, the own units and the rival units.[^7] The near pass adds
those and five more: the water tiles, the resource tiles, the ground water
total, the burning tiles and the height deviation.[^8]

Five channels therefore read zero in 120 of 151 cells: `water_share`,
`resource_share`, `ground_water_density`, `hazard_share` and
`height_deviation`. That is 600 positions.

**This defeats the gate channel.** The layout states that a cell outside the
world reads zero in every channel, and that the gate channel `area_inside_world`
separates an absent value from a quantity of zero.[^9] A far cell inside the
world reads a positive gate. A reader therefore takes its `water_share` of zero
as a real absence of water. The gate works at the granularity of a cell, and
the failure sits one level below it, at the granularity of a channel.

A policy trained on a world large enough to reach ring 4 learns that fire never
burns and water never stands more than three rings from its own centre. On a
world too small to reach ring 4 the loss does not arise, and the training world
of this project may be one of those. The audit could not settle that without
running the engine.

### 2.4 The rival token set publishes 5 channels of 23

The rival token writer sets five channels: `validity`, `settlement_ratio`,
`relation_to_rival`, `relation_from_rival` and `rival_settlement_distance`.[^10]
Eighteen channels are never written, over six tokens, which is 108 dead
positions of 138.

**Seven of the eighteen are already computed in the same function that fails to
write them.** The reading struct holds a per-seat held tile count, a per-seat
unit count, a per-seat finished upgrade count, a per-seat best renown, a
per-seat wonder work total, the war test of each rival, and one observation
confidence.[^11] The channels `held_tile_ratio`, `unit_ratio`, `upgrade_ratio`,
`renown_ratio`, `wonder_ratio`, `war` and `observation_confidence` are the
signed relations and shares of exactly those numbers. Writing them costs no new
pass.

Eight more are honest gaps for a stated reason: `strength_ratio`,
`store_ratio`, `tile_gain_ratio`, `reach_area_ratio`, `trade_ratio`,
`trade_volume_share`, `power_share_trend` and
`rival_settlement_distance_trend`. The scalar block reserves the same quantities
with reasons, and those reasons hold here.[^5] Three remain open:
`population_ratio`, `shared_border_share` and `unit_mix_distance`.

The other three token sets are less severe. The settlement set writes 14 of 24,
so 80 positions are dead. The threat set writes 11 of 20, so 72 are dead. The
site set writes 15 of 16, so 8 are dead.

### 2.5 One token channel is written with the wrong quantity

`strength_balance` of the threat set is written as the signed relation of the
own unit count against the rival unit count.[^12] Its name says strength, and
the same token set declares `strength`, `strength_share` and `own_strength`,
all of which read zero.

This is the pattern the brief calls the most valuable and the hardest to see. A
reader that wanted a strength comparison finds one channel that answers and
three that do not, and the one that answers counts units. **The engine has one
rule for strength in the scalar block, which is to reserve it and say why, and
another rule in the token block, which is to substitute a unit count under a
strength name.**

### 2.6 The smaller rows

`objective_weight` declares twelve positions. The engine holds five controller
weights, and the field doc says so.[^5] Seven positions read zero and declare
the form `relation`.

`upgrade_legality` declares eight positions. `UPGRADE_CATEGORY_COUNT` is
seven.[^13] One position reads zero.

The frontier block sets one position from the constant
`RESERVED_REACH_HEADROOM`, which is zero.[^14] The field declares the form
`statistic`.

### 2.7 Duplicated fields

The audit found no duplication that the layout does not already declare.

`wonder_progress` of block A and `wonder_track_progress` of block C write the
same accumulator, and both docs say so. `idle_unit_share` of block A and
`free_unit_share` of block K share one arm of the writer, and both docs say so.

Two ring channels share a numerator. `own_held_share` divides the held tiles of
a cell by the observed tiles, and `own_reach_share` divides the same count by
the in-world tiles. The pass doc states the pair and the reason.[^15] The two
values differ only where the faction has not seen the whole cell.

**The population case is the exception, and it is already known.** This audit
found no second instance of a field that silently publishes another field's
quantity.

### 2.8 Two fields sum a hazard and a non-hazard

`held_under_hazard_share` divides the burning held tiles plus the wet held
tiles by the held tiles seen now.[^16] `settlements_under_hazard_share` counts a
settlement whose tile is burning **or** wet.

Fire kills units and wet ground does not. The layout publishes fire and water
separately in the same block, as `fire_share_held` and `water_share_held`, so
the combined field adds a hazard to a ground condition and calls the sum a
hazard. A tile that is both burning and wet counts twice, and the bounded share
then clamps the result.

The value is not wrong arithmetic. It is a name that promises one thing and a
write site that reads two. Rank it low: the two parts are published beside it,
so a policy can recover them.

## 3. What the engine knows and does not publish

The audit read the simulation modules through three parallel surveys, then
verified the three load-bearing claims against the source directly.

Proposals are ranked. Each states the quantity, the system, the value form,
whether the width follows the world, and the cost of a read.

### 3.1 The nearest enemy settlement, and its distance

**System.** The campaign objective search.[^17]

The built-in controller receives, on every frame it has no live campaign, the
tile of the nearest enemy settlement measured by hex distance from its own
seat. It first looks for a relief target, which is its own settlement standing
on ground a faction at war with it now holds. Failing that it takes the nearest
settlement of any faction at war with it.

**The search reads the whole board and no fog.** It walks the settlement arena
for the tile and the faction of every site, and it reads the holding for the
ground under each one. The audit read the function and confirms this: no call in
it passes a faction to a fog-scoped reader.

**A policy reads no part of this.** The observation gives it a settlement count,
a border length, a contested border share, and a rival token whose
`rival_settlement_distance` names the distance to the nearest rival settlement
of one token subject. It never names a distance to an enemy settlement, and it
never names one of its own settlements as besieged.

**Why a policy would act differently.** A domination win requires holding every
live rival's seat. A seat is a settlement tile. A policy that cannot see the
distance to an enemy settlement cannot choose to march on one, and it must
discover the target by wandering. The built-in controller took 21 of the run's
22 domination wins.[^18] It is also the only agent in the run that receives a
settlement target.

**Form.** Two positions. A share of the widest distance for the nearest enemy
settlement, and a share for the nearest of the reader's own settlements that
stands on ground a hostile faction holds. Add one flag for each to say whether
the target exists.

**Bounded.** Yes. Four positions, whatever the world holds.

**Cost.** The search already runs. It costs the settlement count, which the
observation already walks twice. Publishing it costs nothing new.

**Rank 1.** It is the largest asymmetry the audit found, it explains a measured
result, and the pass exists.

### 3.2 The ration fill ratio

**System.** The consumption draw of the cohort pass.[^19]

The draw ledger keeps three running totals for each commodity: the amount
demanded, the amount granted, and the amount unmet. The rate ledger of the
production pass keeps two more: the upkeep that no store could pay, and the
production that spilled at a store ceiling.[^20]

**A policy reads none of them.** It reads the stock, the production rate, the
consumption rate and their signed relation. Those are levels and rates. **A
level says nothing about whether the level was enough.** A faction whose stock
holds steady at zero and whose consumption is entirely unmet reads the same
stock and the same net flow as a faction in balance at zero.

The event memory carries `own_units_starved` as a share of the live units, over
a short window and a long window.[^21] That fires after the deaths. The fill
ratio falls first.

**Why a policy would act differently.** Starvation is the one economic failure
that removes units, and a unit count drives three of the four win paths. A
policy that reads a falling fill ratio can move a unit off a gather order or
open a trade before it loses the unit. A policy that reads only stock and rate
learns the lesson from the corpse.

**Form.** A share for each commodity: the granted amount over the demanded
amount. One position at present, and eight under the good-class taxonomy the
layout already declares. A second share for the spillage over the production
gives the waste side.

**Bounded.** Yes. It follows the commodity taxonomy, which the layout fixes.

**Cost.** The ledger is a per-pass struct and not a table. The world must retain
the last ledger of each pass, which is a constant, or the observation must
re-derive the ratio over the settlements of the faction. Either is bounded by
the settlement count.

**Rank 2.** A high-value quantity that the engine already computes and throws
away. It needs one piece of retained state.

### 3.3 The influence margin over the reader's own ground

**System.** The influence field, which drives conversion.[^22]

The field holds one influence value for each faction at each level 1 cell. A
unit converts where another faction reaches its cell more strongly than its own
faction does.

**A policy reads nothing of it.** It cannot see which of its own cells a rival
now leads, and it therefore cannot see that its own units are about to change
hands.

**Why a policy would act differently.** Conversion takes a unit without a
fight, so no contest signal precedes it. A policy that could read a negative
margin over its own ground could garrison the cell, move the unit, or press the
relation of the rival that causes it.

**Form.** Three positions, as order statistics over the cells the faction
holds. The lowest margin as a signed relation, the mean margin as a signed
relation, and the share of held cells where a rival leads.

**Bounded.** Yes at three positions. **A per-cell publication is rejected**: it
would need one position for each held cell, and the width must not follow the
holding.

**Cost.** The pass reads the influence plane at each cell the faction holds,
which is the level 1 cell count of the holding. That follows the ground the
faction holds and not the world, and the ring stack already walks the held tile
list.

**Rank 3.** A real mechanic that is invisible until it fires. The cost is the
one thing to check before building it.

### 3.4 The wonder work that remains, and the settlement that holds it

**System.** The victory claim walk.[^23]

The walk returns, for each faction, a pair: the largest standing victory claim
on its held ground, and the work behind either the finished claim row or the
best site building toward one. The observation publishes the work as a share of
the work the wonder row asks for, which is a real gradient.

**The walk discards which site holds that work.** A policy that reads a wonder
progress of 0.4 cannot tell which of its eight settlements to send a builder
to.

The settlement token set already declares a `wonder_progress` channel and never
writes it. Filling that channel answers the question with no new field.

**Form.** One share for each settlement token, in the channel the set already
declares.

**Bounded.** Yes. Eight positions, which the layout already holds.

**Cost.** The walk visits the sparse upgrade map, which the observation already
walks. It must keep the tile of the best site rather than the value alone.

**Rank 4.** Cheap, and it turns a published aggregate into an actionable one.

### 3.5 The population of each of the reader's own settlements

**System.** The cohort table.[^24]

The settlement token set declares `population` and `population_share` and
writes neither. The engine reads the residents of a site in bounded time, and
the observation already sums them for the faction total.

**Why a policy would act differently.** A policy chooses which settlement takes
a build order or a founding party. The faction total and the mean per settlement
say nothing about which one is large. Founding drains a settlement, and a policy
cannot see which one can afford it.

**Form.** Two shares for each settlement token, in the channels the set already
declares: the residents over the faction population, and the residents as a
compressed magnitude.

**Bounded.** Yes. Sixteen positions, which the layout already holds.

**Cost.** The settlement scan already reads the residents of each own site.
Publishing the per-token value costs nothing new.

**Rank 5.**

### 3.6 The fire kill total and the burning tile count

**System.** The fire field.[^25]

The field keeps a running total of the units fire has burned, and a count of
the tiles now burning. Both are world-wide and not per-faction.

The observation reserves `units_lost_to_hazard` because the engine holds a
per-frame log and not a window.[^5] **The running total is not a window and not
a frame. It is a monotone total, which is a third thing.** The event memory
carries `own_units_burned` as a decayed share, so the rate is already visible.
The total adds the level.

**Form.** One compressed magnitude for the burning tile count.

**Bounded.** Yes.

**Cost.** Constant.

**Rank 6.** Low value. The event memory already gives a policy the signal it
needs to act, and the totals are world-wide rather than the reader's own.

### 3.7 Proposals the audit rejects

State a rejection rather than a proposal, so nobody proposes it again.

**A per-unit condition share.** The engine holds a fed, short and starved
condition for each unit, and no aggregate of it. A share of the population in
each condition would be bounded and useful. Deriving it costs one pass over the
population, which the observation must not do. The ration fill ratio of section
3.2 answers the same question at the settlement count.

**The presence relation.** The engine keeps, for each ordered pair of factions,
whether any unit of one stands on ground the other holds. Publishing it needs
one position for each rival, and no slot of this layout names a seat.[^26] As an
order statistic over the field it collapses to a share of rivals that trespass,
which the contested border share already approximates.

**The tile value total.** The engine sums the value of every tile of the world.
It is not aggregated by faction, so it says nothing a faction can act on. The
ring stack already publishes a value density for each cell.

**The soldier deed column and the house sizes.** Both are real unpublished
state. Neither drives a win path, and the audit cannot say a policy would act
differently.

**A per-tile trade price history.** The trade module holds no price field, no
history and no moving average anywhere. There is nothing to publish.

## 4. The four win paths

The brief asks what a policy can observe about its progress on each path.
Across 252 measured games renown ended no game, and the built-in controller
took 21 of 22 domination wins.[^18]

**Two of the four paths are well served, and the audit says so plainly.**

### 4.1 Territory at the tick limit: well served

The reader fires at the tick limit for the contender with the most held
tiles.[^27] The win condition is therefore an ordering against rivals, and not a
level.

A policy reads `ground_gap`, which is the signed relation of its held tiles
against the strongest rival, and `ground_rank`, which is the rivals it leads.
Those two **are** the win condition. It reads `tick_share` and
`remaining_ticks` for the clock. The held tile count of every faction it has
seen arrives as the seven order statistics of `power_held_tiles`, with a
confidence.

One caution. `ground_progress` divides the held tiles by the world passable
tiles, and that is not the win condition. It is a small number that never
approaches one. A reward term built on `ground_progress` would train a policy to
paint tiles rather than to lead.

### 4.2 Wonder: well served

The reader fires for the first faction with a standing victory claim, and only
the wonder row carries a non-zero claim.[^28] `wonder_track_progress` divides
the work behind the best wonder candidate by the work the wonder row asks for.
That is a true gradient toward the win.

A policy also reads the leader share, the gap and the rank. What it cannot read
is **which of its settlements** holds that work, which is section 3.4.

### 4.3 Domination: the published progress does not reach one when the win fires

The reader fires on either of two conditions.[^29]

The first is by seats. The candidate must hold the seat of every faction that
is not eliminated and that holds a seat row, including its own, and at least one
rival seat must exist.

The second is by units. Every rival must hold zero units and the candidate must
hold at least one.

`domination_progress` divides the seats the faction holds by the **seated
faction count**. The requirement of the reader is the seats of the **live**
factions. **The denominator never shrinks, and the requirement does.** A faction
that eliminates three rivals of seven, then takes the three remaining seats,
satisfies the reader while `domination_progress` reads four sevenths. The
signal a policy would climb tops out below one at the moment it wins.

The second condition is unpublished. `power_units` gives the own share of the
seen unit total, the leader share and a concentration, from which a policy could
infer that rivals are nearly empty. Nothing states which rivals are eliminated,
and elimination is what gates the seat test.

Add these. The audit proposes two positions and no new pass: the seats held over
the **live** seated count, and the share of seated factions that are eliminated.
Both come from readers the engine already has.

### 4.4 Renown: the observation is not the problem

The reader fires when the highest live renown of a faction reaches the renown
target.[^30] The target is 1000.[^31]

Renown has one source in the engine. A felled unit gives `renown_per_fell` to
the champion of the faction that felled it, and the champion is the live
character with the highest renown.[^32] `renown_per_fell` is a quarter of one
point.[^33]

**A renown win therefore needs 4,000 units felled by one faction, and the whole
amount sits on one mortal character.** If that character dies, the faction's
best renown falls to its next best live character, and the published progress
falls with it.

`renown_progress` divides the best renown by 1000. After forty kills it reads
0.01. **A policy reads a value that is flat and near zero for the whole game.**
There is no gradient in it to climb, and the gradient that exists is not a
gradient of anything the policy controls: it is a by-product of felling, which
the war path already rewards.

**The renown path is not a missing observation. It is a balance figure and a
single point of failure.** Publishing `renown_per_fell` would let a policy
compute the kills it needs, and it would then read 4,000 and do something else.
Publishing the champion's identity or its survival risk would help a policy
protect the character it depends on. Neither changes the arithmetic. **Do not
spend observation width on this path until the target and the rate change.** A
blocker already holds the threshold question.[^34]

## 5. Verdict, ranked

Fix in this order.

1. **The domination denominator.** `domination_progress` cannot reach one when
   the reader fires. It is the one case where the observation misstates a win
   condition, and it is two positions and no new pass. It also bears directly on
   the measured result that no trained policy has taken a seat.
2. **Publish the campaign objective.** The built-in controller reads an unfogged
   nearest-enemy-settlement target and a policy reads nothing like it. Four
   positions, and the pass already runs.
3. **Mark the 1,401 undeclared dead positions.** Three actions, in one change.
   Reserve the three hardcoded ring channels. Reserve the eighteen unwritten
   rival channels and the rest of the unwritten token channels. Either fill the
   good-class positions or cut the taxonomy to the commodity count. **The point
   is not to fill them. The point is that a reader can find them.** A dead
   position with a real form is worse than a dead field, because nothing marks
   it and a reward term reads it as truth.
4. **Write the seven rival token channels the reading struct already holds.**
   108 dead positions of 138 in that set, and seven of the eighteen cost no new
   work.
5. **Rename or reserve `strength_balance`.** It publishes a unit count under a
   strength name, in a set that reserves three strength channels beside it.
6. **The ration fill ratio.** The highest-value quantity the engine computes and
   discards. It needs one piece of retained state.
7. **The five ring channels that go dark beyond ring 3.** Either give the block
   summary a water, resource, ground-water and burning total, or state the near
   band in the channel name so a reader cannot mistake the zero. Decide first
   whether the training world reaches ring 4.
8. **The influence margin, the per-settlement wonder progress, and the
   per-settlement population.** Real gaps, moderate value, and the last two use
   channels the layout already declares.
9. **Nothing on the renown path.** Its problem is 4,000 kills and one mortal
   character, and no field fixes that.

**Two categories are smaller than feared, and the report says so.** The audit
found no second instance of a field that publishes another field's quantity
under a different name, so the population case looks isolated. And the reserved
fields are largely reserved for good reasons: a window that does not exist, a
strength rule that no record states, a store that no fog admits. The 121
reserved positions are the part of this layout that behaved correctly. **The
problem is the 1,401 positions that did not declare themselves at all.**

## 6. A finding for the register

The audit took **FND-695**. Lift the text below into the register unchanged.
Do not treat the number as reserved until the register holds it.

> **FND-695. The reserved count measured the marked dead width, not the dead
> width.**
>
> **What the project believed.** The observation of a faction held 42 reserved
> fields, and those fields were the inert part of the layout. A reserved field
> declares bounds of zero and zero, so a reader could find every position that
> reads zero forever.
>
> **What is true.** The 42 reserved fields hold 121 positions, which is 2.5
> percent of the 4,819 positions. A further 1,401 positions read zero in every
> state and declare a real value form. The largest groups are three ring stack
> channels hardcoded to zero across all 151 cells, which is 453 positions; five
> ring stack channels that the far pass never accumulates, which is 600
> positions in the 120 cells beyond ring 3; and 268 token channels that no
> writer sets, of which 108 sit in the rival set of 138.
>
> **The evidence.** The ring channel table gives `memory_age`, `own_strength`
> and `rival_strength` the expression `Fix32::ZERO`. The far accumulation pass
> adds nine totals to a cell and the near pass adds fourteen, so `water_share`,
> `resource_share`, `ground_water_density`, `hazard_share` and
> `height_deviation` read zero in every far cell. The rival token writer sets
> five of 23 channels. `COMMODITY_COUNT` is one and the work commodity table
> maps every resource kind onto commodity zero, so seven of eight good classes
> read zero in five fields and in the trade board.
>
> **What follows.** A count of reserved fields is not a measure of inert width.
> A dead position that declares a real form is worse than a reserved field,
> because the schema states real bounds for it and a reward term reads it as
> truth. Two rules follow. Reserve a position the moment nothing writes it,
> whatever field it sits in. And add a check that fails when a declared position
> reads zero across a fixture set that varies every input, so a channel cannot
> go dark without something failing.

## References

[^1]: The observation layout of one faction, the field table. `crates/cachette-core/src/faction_observation.rs`
[^2]: The observation layout of one faction, what a reserved field means. `crates/cachette-core/src/faction_observation.rs`
[^3]: The settlement module, the commodity count. `crates/cachette-core/src/site.rs`
[^4]: The position module, the work commodity table. `crates/cachette-core/src/position.rs`
[^5]: The observation layout of one faction, the doc of each reserved field. `crates/cachette-core/src/faction_observation.rs`
[^6]: The ring stack block, the channel table. `crates/cachette-core/src/obs_ring_stack.rs`
[^7]: The ring stack block, the far accumulation pass. `crates/cachette-core/src/obs_ring_stack.rs`
[^8]: The ring stack block, the near accumulation pass. `crates/cachette-core/src/obs_ring_stack.rs`
[^9]: ADR-0195, the observation of a faction is a fixed-width scale-free table, decision D8. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^10]: The entity token block, the rival token writer. `crates/cachette-core/src/obs_token.rs`
[^11]: The observation layout of one faction, the reading struct and the gather pass. `crates/cachette-core/src/faction_observation.rs`
[^12]: The entity token block, the threat token writer. `crates/cachette-core/src/obs_token.rs`
[^13]: The upgrade module, the category count. `crates/cachette-core/src/upgrade.rs`
[^14]: The frontier block, the reserved reach headroom. `crates/cachette-core/src/obs_frontier.rs`
[^15]: The ring stack block, the holding accumulation pass. `crates/cachette-core/src/obs_ring_stack.rs`
[^16]: The observation layout of one faction, the hazard share writer. `crates/cachette-core/src/faction_observation.rs`
[^17]: The campaign objective search of the world. `crates/cachette-core/src/world/campaign.rs`
[^18]: The measured league run of 252 games, reported in the observation audit brief of 8 September 2026. Unverified by this report.
[^19]: The cohort module, the draw ledger. `crates/cachette-core/src/cohort.rs`
[^20]: The rates module, the rate ledger. `crates/cachette-core/src/rates.rs`
[^21]: The event memory module, the memory kind list. `crates/cachette-core/src/event_memory.rs`
[^22]: The influence module, the influence field. `crates/cachette-core/src/influence.rs`
[^23]: The victory readers of the world, the victory claim walk. `crates/cachette-core/src/world/victory.rs`
[^24]: The cohort module, the resident reader. `crates/cachette-core/src/cohort.rs`
[^25]: The fire module, the fire field. `crates/cachette-core/src/fire.rs`
[^26]: Findings register, FND-647. `docs/FINDINGS.md`
[^27]: The victory readers of the world, the territory reader. `crates/cachette-core/src/world/victory.rs`
[^28]: The victory readers of the world, the wonder reader. `crates/cachette-core/src/world/victory.rs`
[^29]: The victory readers of the world, the domination reader. `crates/cachette-core/src/world/victory.rs`
[^30]: The victory readers of the world, the renown reader. `crates/cachette-core/src/world/victory.rs`
[^31]: The balance module, the renown target. `crates/cachette-core/src/balance.rs`
[^32]: The character pass of the world, the renown award. `crates/cachette-core/src/world/character.rs`
[^33]: The contest module, the renown of one felled unit. `crates/cachette-core/src/contest.rs`
[^34]: Blockers register, BLK-150. `docs/BLOCKERS.md`
