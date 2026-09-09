# Reinforcement Learning Parameters (Register)

This document is a **register**. It holds every value the learner seat needs
and does not have. A design document and a decision record cite a row here and
hold no figure.[^1] [^2]

**Every reward weight in this register is unset.** The reward weighs the
things a faction gains. What those things are worth, and what winning is
worth, are rules of the downstream game, and one blocker holds those rules.[^3]
A guessed weight is a rule of a game nobody has written down.

**The world a run plays is set, and a probe measured it.** The world is not a
rule of the downstream game. It is a choice of training run, a run states it
on the command line, and a measurement answers it.

A control plane module applies the weights. It states none of them, and it
refuses to run while one of them is unset. The refusal names each unset weight
and names the blocker.[^4]

## Format for a row

| Column | Holds |
|---|---|
| Value | The name of the value, as the module names it |
| Read by | The reader that reads it |
| Set | `unset`, and what sets it |
| Blocker | The blocker that governs it |
| Derivation | Empty until a value is written. Then how the value was reached, and the commit |

## The shaped terms

A shaped term weighs one quantity in one of two forms. A level term weighs the
reading of that quantity at each decision. A change term weighs the change of
it since the previous decision. Each name below names one field of the
observation array of a faction, and the schema of the world declares where that
field sits.[^5]

**A weight multiplies the bounded value the schema declares.** Every position
of the observation array is a Q16.16 value between minus one and one, so 65536
is one unit and no row carries a raw count. A term that names a compressed
magnitude moves less at a large quantity than at a small one, because the map
is a logarithm. A caller that writes a value into a row below states the unit
in the derivation column.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| `held_tiles`, the weight of the compressed held tile count | The reward of a faction | unset, the project owner | BLK-050 | |
| `domination_progress`, the weight of the seat share of the faction | The reward of a faction | unset, the project owner | BLK-050 | |
| `live_units`, the weight of the compressed live unit count | The reward of a faction | unset, the project owner | BLK-050 | |
| `population`, the weight of the compressed population total | The reward of a faction | unset, the project owner | BLK-050 | |
| `store_total`, the weight of the compressed store total | The reward of a faction | unset, the project owner | BLK-050 | |
| `best_renown`, the weight of the compressed best renown | The reward of a faction | unset, the project owner | BLK-050 | |
| `wonder_progress`, the weight of the wonder work share | The reward of a faction | unset, the project owner | BLK-050 | |
| `wonder_track_progress`, the weight of the wonder share of the faction | The reward of a faction | unset, the project owner | BLK-050 | |

A caller may weigh any other field of the observation array that holds one
position. The rows above are the terms the project reserved, and the module
names the same eight.

**A weight over a change telescopes, and the same eight names take a weight
over a level.** An evolution strategy sums the reward of every decision of the
episode with no discount, so a sum of changes collapses to the last reading
less the first. A change weight therefore pays one number for a whole episode.
A level weight is paid on every decision, and it divides the published value by
the unit the engine published for that field. The rows above answer for either
form, and one finding holds the measurement.[^14]

**Every shaped weight the strategy table of the trainer states is a level
weight.** Each weight of that table read a change once, so the whole table
trained against a terminal reward under weights that read as dense.[^24] A row
above therefore answers for the level form in every run the project plays
today.

## The terminal outcomes

A terminal outcome is paid once, when the run ends. Three outcomes end a run.

**There is no elimination outcome.** A faction that loses every unit and every
person keeps its held ground and its seat, so it may still win at the tick
limit. A finding holds the measurement.[^6]

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| `won`, what the faction gets when a reader names it the winner | The reward of a faction | unset, the project owner | BLK-050 | |
| `lost`, what the faction gets when a reader names another winner | The reward of a faction | unset, the project owner | BLK-050 | |
| `drawn`, what the faction gets when the run reaches the tick limit with no winner | The reward of a faction | unset, the project owner | BLK-050 | |

## The terminal timing terms

A terminal timing term is paid once, when the run ends. It weighs a level and
not a change, so it is the only kind of term an undiscounted episode return
carries. A sum of changes over a whole episode collapses to the last reading
less the first, so a change term contributes the same amount whatever the
policy did in between.[^12]

**A game that runs to the tick limit ends won or lost.** The engine compares
held ground at the limit and records a winner. Weaker play shifts a game toward
the limit, and a policy that does nothing at all reaches it more often.

**A game at the limit is not evidence of indecisive play on its own.** A limit
below the tick a game resolves at ends a decisive game early, and one finding
holds that correction with its distribution.[^17] Read this term as a reward
for a fast win, and read the limit as a separate measurement.

**The row below pays a win alone.** A term that paid the time left on any
outcome would pay a faction for losing quickly, so a faction that gave up
early would outscore a faction that held on and lost narrowly at the limit. A
finding holds the reasoning.[^13]

**A weight of zero is the default of this row.** Every other weight of the
reward refuses to run while it is unset. This one defaults, because a stored
score measured before the row existed must stay comparable with a score
measured after it.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| `won_early`, what the time left before the tick limit pays on a win | The reward of a faction | unset, the project owner | BLK-050 | |

## The observation layout

A draft record makes the width of the observation a constant, and it holds no
figure of its own.[^9] The width is a function of the rows below, so a change to
one of them changes the width and retires every policy trained before it. A
decision register holds the open choice, and a research report derives a
recommendation for each row.[^10] [^11] The project took the recommendation of
the report for the first build, and the choice the register still holds is
whether to price each block against a corpus of episodes.

**Every row below is set, and the builder of the engine produces the array.**
The builder writes an egocentric ring stack, an entity token block, a frontier
block and the scalar fields, and the schema of a world declares where each one
sits.[^5] **Every value below is provisional.** Each one comes from the
recommendation of the report and not from a measurement of what a position is
worth. A pass that prices a block replaces the value it prices.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Ring count of the spatial stack | The observation builder | 14, provisional | — | The ring index of a tile is the bit length of its hex distance from the centre, so the count is one more than the bit length of the greatest distance of the largest world the project supports. Declared as `RING_COUNT` over a cap of 13[^19] |
| Sector count of a ring, for each ring | The observation builder | 1 at ring 0, 6 at ring 1, 12 at every ring above, provisional | — | Ring 0 holds the centre tile, which has no direction. Ring 1 covers the six tiles at distance one, so a twelfth sector there would hold no tile in any world. The sector rule of the frame states this, and the cell list reads the rule[^19] |
| Channel count of a spatial cell | The observation builder | 25, provisional | — | Declared as `RING_STACK_CHANNELS`, beside the channel name list that is the one declaration of the order[^19] |
| Distance band of each ring, in tiles | The observation builder | Ring 0 covers distance 0. A ring above 0 covers the band from two to the power of one less than its index, up to one less than two to the power of its index. Provisional | — | The band follows the bit length rule of the ring index, and it states no figure of its own[^19] |
| Token count of each token set | The observation builder | 8 settlements, 6 rivals, 8 threat clusters, 8 candidate sites, provisional | — | The token counts of the entity token block. A faction below sixth on threat does not drive a decision, and the order statistics of the layout cover the whole field[^20] |
| Channel count of a token, for each token set | The observation builder | 24 settlement, 24 rival, 20 threat cluster, 16 candidate site, provisional | — | The channel name list of each set is the one declaration of it, and one constant sums the four products[^20] |
| Bit cap of a compressed magnitude | The observation builder | 40 bits, provisional | — | The largest quantity the observation can carry is a fixed-point store total summed over the unit ceiling of the world. Declared as `MAGNITUDE_CAP_BITS`[^22] |
| Class count of the goods taxonomy | The observation builder | 8, provisional | — | Declared as `GOOD_CLASS_COUNT`. The taxonomy is fixed, so a new good does not change the width. The engine holds one commodity, so a class it holds no commodity for reads zero[^21] |
| Class count of the unit type taxonomy | The observation builder | 8, provisional | — | Declared as `UNIT_CLASS_COUNT`, which reads the row count of the unit type table rather than states a number of its own[^21] |
| Reserved position count | The observation builder | 13, provisional | — | Declared as `LAYOUT_RESERVE`. A revision that claims a position takes it from here and holds the length, so no position moves. Four revisions have claimed positions, and the commit message of each states what it took[^21] |
| Layout revision integer | The policy fit of the control plane | 7 | — | Declared as `OBSERVATION_VERSION`. It rises when a position moves or a published value changes, and each rise retires every stored policy[^21] |
| Window length of every change position, in ticks | The observation builder | No tick window. The `window_ticks` position stays reserved | — | The engine carries a window of events and no window of readings. The event history keeps a short memory and a long memory of every kind, at shifts of 4 and 8 bits. One counter cannot separate a spike from a trend, and one position cannot state both lengths[^23] |

## The world a training run plays

A training run states its world on the command line: the extent in columns and
rows, the faction count, the tick limit and the decision interval. **Every row
below is set, and a probe measured it.**[^15] The launcher asks the trainer for
the world through one flag, so it holds no copy of any of them.[^16]

### What the extent buys, and what it does not

**The observation and the action table do not grow with the extent.** The
observation of a faction is a fixed-width table over an egocentric log-polar
frame, and the action table names verbs with candidate coordinates rather than
tiles.[^9] A probe read both lengths at five extents from 48 to 256 and got one
pair of numbers. One trained policy therefore fits every extent, and a change
of extent retires nothing.

**The rings of the frame grow with the logarithm of the extent.** The ring
index of a tile is the bit length of its hex distance from the centre, and the
greatest hex distance in a rhombus world is the width plus the height less two.
The rings a world can fill is therefore one more than the bit length of that
distance, so **each doubling of the extent buys exactly one ring**. The frame
holds 14 rings, which the target scale of 4096 by 4096 fills.

| Extent | Tiles | Rings the world can fill | Rings a played world filled | Level 1 cells |
|---|---|---|---|---|
| 48 by 48 | 2304 | 8 | 7 | 4 |
| 96 by 96 | 9216 | 9 | 8 | 9 |
| 128 by 128 | 16384 | 9 | 9 | 16 |
| 192 by 192 | 36864 | 10 | 9 | 36 |
| 256 by 256 | 65536 | 10 | 10 | 64 |

The filled column comes from the channel that says how much of a spatial cell
lies inside the world, read from one seed. It is at or below the bound, because
the frame is egocentric and a faction does not start in a corner. The level 1
column divides each side by the block edge of 32 tiles and rounds up.

### The room a faction has to grow

**A larger world that still yields one settlement has bought nothing.** The
probe plays whole episodes with the built-in controller in every seat and reads
the settlement count of the learner's seat as the episode runs. The share of
episodes in which that faction ever held more than one settlement rises with
the extent, and so does the count it ends with.

| Extent | Episodes | Episodes that founded a second settlement | Median settlements at the end | Median highest settlements | Median held tiles |
|---|---|---|---|---|---|
| 48 by 48 | 16 | 0.38 | 0.5 | 1.0 | 56 |
| 96 by 96 | 16 | 0.63 | 1.5 | 2.5 | 716 |
| 128 by 128 | 16 | 0.88 | 3.0 | 4.5 | 832 |
| 192 by 192 | 12 | 0.75 | 9.5 | 10.0 | 3149 |
| 256 by 256 | 12 | 0.92 | 6.5 | 7.5 | 1732 |

**Half the episodes of a 48 by 48 world end with the seat holding nothing.**
Eight of sixteen end at zero settlements. No episode of a 256 by 256 world
does, and its median seat ends with six settlements. That is the measurement
that says a larger world is worth its price.

The rise is not monotone between 192 and 256, and twelve episodes cannot
separate the two. Read the step from 48 to 128 as the signal, because it is
large and it is monotone over three points.

**The tick at which a faction founds its second settlement barely moves.** It
is about 420 ticks at extent 48 and about 500 at every extent above it. A
larger world does not delay the growth; it removes the ceiling on it.

### The tick limit each extent needs

The tick limit ends an episode that no victory ended, and the engine then
compares held ground and names a winner. **A limit below the tick a game
resolves at replaces the outcome with that comparison.** One finding holds the
correction: the cluster of games at the limit is the limit and not indecisive
play.[^17]

The rows below come from episodes played under a tick limit of 12000, so the
end tick each row reads is the tick the game resolved at. The controller arm
gives every seat to the built-in controller. The idle arm holds the learner
seat and plays the no-op at every decision, which is what an untrained policy
does, and it brackets the other side of the limit.

**Read the share of games that resolve under a candidate limit.** That is the
quantity a limit is chosen against, and it needs no quantile of a small
sample. A game that does not resolve under the limit still ends, by a
comparison of held ground, which is the one ending a policy cannot aim at.

| Extent | Arm | Episodes | Resolve by 2500 | by 4000 | by 5000 | by 6000 | by 8000 |
|---|---|---|---|---|---|---|---|
| 48 by 48 | controller | 16 | 0.62 | 0.88 | 0.88 | 0.88 | 0.94 |
| 48 by 48 | idle | 16 | 0.69 | 0.88 | 0.88 | 0.88 | 0.88 |
| 96 by 96 | controller | 16 | 0.19 | 0.31 | 0.62 | 0.81 | 0.81 |
| 96 by 96 | idle | 16 | 0.38 | 0.50 | 0.88 | 1.00 | 1.00 |
| 128 by 128 | controller | 16 | 0.25 | 0.69 | 0.81 | 0.94 | 0.94 |
| 128 by 128 | idle | 16 | 0.31 | 0.75 | 0.94 | 0.94 | 1.00 |
| 192 by 192 | controller | 12 | 0.17 | 0.67 | 0.83 | 0.92 | 1.00 |
| 192 by 192 | idle | 12 | 0.25 | 0.58 | 0.83 | 1.00 | 1.00 |
| 256 by 256 | controller | 12 | 0.25 | 1.00 | 1.00 | 1.00 | 1.00 |
| 256 by 256 | idle | 12 | 0.08 | 0.58 | 0.92 | 1.00 | 1.00 |

**The current limit of 2500 truncates most games at every extent above 48.**
It truncates four games in five of a 96 by 96 world. A run at a larger extent
that keeps this limit trains against a game that almost always ends on the
ground comparison.

**A larger world resolves sooner and more tightly, not later.** The highest
end tick of a 256 by 256 world is 3676 over twelve controller episodes, and
the highest of a 48 by 48 world is above 12000. A faction with room grows,
and a faction that grows reaches a win path. The intuition that a larger
world needs a longer game is wrong above extent 128, and the measurement is
the reason to say so.

| Extent | Arm | Median end tick | Upper quartile | Highest |
|---|---|---|---|---|
| 48 by 48 | controller | 1787 | 2749 | above 12000 |
| 48 by 48 | idle | 1703 | 2603 | above 12000 |
| 96 by 96 | controller | 4221 | 5725 | above 12000 |
| 96 by 96 | idle | 3686 | 4442 | 5973 |
| 128 by 128 | controller | 3675 | 4418 | 11858 |
| 128 by 128 | idle | 2994 | 3672 | 6771 |
| 192 by 192 | controller | 3520 | 4631 | 6006 |
| 192 by 192 | idle | 3452 | 4772 | 5521 |
| 256 by 256 | controller | 2763 | 3421 | 3676 |
| 256 by 256 | idle | 3508 | 4333 | 5890 |

**Sixteen and twelve episodes cannot state a ninetieth percentile.** Read the
resolve share and the median. Read the highest as the tail and not as a bound.

**Every end tick row above was measured with the renown target at 1000 units,
and the target is now 50.** A lower target opens the renown path, so a game
can end on it and every game ends at or before the tick it ended at before.
The rows are therefore an upper bound on the current engine and are marked
provisional. A repeat run of the probe replaces them.

| Value | Read by | Set | Blocker | Derivation |
|---|---|---|---|---|
| Tick limit of a training run, at every extent from 48 to 256 | A training run | 2500 by default, and any limit a caller names. The probe recommends 6000, provisional | — | The trainer default is 2500. The recommendation is the lowest round limit at which four fifths of the games of every measured extent and arm resolve, at renown target 1000 |
| Extent of a training run | A training run | 48 by 48 by default, and any extent a caller names | — | The trainer default. The probe recommends a larger one |
| Faction count of a training run | A training run | 3 by default, and any count a caller names | — | The trainer default |

**One limit serves every extent measured.** A limit of 6000 gives a resolve
share at or above 0.81 on every row of the table above, and at or above 0.92
on every row from extent 128 upward. A limit of 8000 buys little and costs a
third more ticks in every episode that runs to it.

**A tick limit is not a cost figure.** It is a rule of the episode a run
plays, and the engine is deterministic, so a limit measured on a development
machine answers for the target platform unchanged.[^18] The cost of running
under that limit is a machine figure, and the target register holds it.[^8]

## The spread of a generation, against the terminal quantum

**Every row below is set, and one measurement took all of them.** The figures
say how far the shaped part of a candidate score separates the candidates of
one generation, and how far one seed changing its outcome moves one candidate.
A decision record must not hold them, because a later measurement changes
them.[^2]

An evolution strategy ranks the candidates of one generation and moves the
centre along the ranking. The score of a candidate is the mean over its seeds.
The shaped part is the objective vector combined under the weights of the
style. The terminal part is what the outcome of each episode pays.

**One seed that changes from a loss to a win moves a candidate mean by the win
weight less the loss weight, over the seed count.** That is the terminal
quantum below. The engine names a winner at the tick limit, so no episode of
the measured generation was drawn.

| Value | Reading |
|---|---|
| `population` | 24 |
| `seeds` | 6 |
| `episodes` | 144 |
| `won_episodes` | 8 |
| `shaped_spread_range` | 43.81 |
| `shaped_spread_deviation` | 12.38 |
| `terminal_spread_deviation` | 28.33 |
| `total_spread_deviation` | 27.52 |
| `terminal_quantum` | 33.33 |
| `shaped_range_over_quantum` | 1.31 |

**The terminal part carries more of the spread than the shaped part does.**
Its deviation over the candidates is 28.33 against 12.38 for the shaped part,
and the deviation of the whole score is 27.52, which is nearly the terminal
part alone. One seed changing its outcome moves a candidate by 33.33, and the
whole shaped separation between the highest and the lowest candidate of the
generation is 43.81. An audit argued this from the weights alone and marked
the comparison unverified. The measurement confirms it.

**The correction is a rule of the downstream game, and one blocker holds
it.**[^3] Lowering the terminal weights states how much winning is worth
against the shaped objectives, and that ratio is not a figure a measurement
answers. Raising the seed count lowers the quantum in proportion, and it costs
episodes that one blocker on cost figures governs.[^8] A test reads the rows
above and fails when a weight of the style table or the seed count moves the
quantum, so the next change to either must take a new measurement rather than
inherit this one.[^25]

**These rows are not machine figures.** The engine is deterministic and the
reward is integer arithmetic over what the engine publishes, so the reading is
a property of the world, the seeds and the weights alone.[^18] A separate
register holds what the target platform measured, and it holds none of
these.[^26]

### The conditions of the measurement

The measurement played one generation of the shape a paid run plays: 24
candidates over 6 seeds, in the world this register states, under the
`aggressive` style with the structured policy kind. The candidates are the
antithetic pairs of generation 0 around a centre of zeros, at the sigma the
trainer configuration declares. The commit body holds the command.

**The play style table states the weights the quantum divides, and this
register states none of them.** Those weights are starting values that a
researcher edits, and the terminal rows of this register stay unset. The
quantum below therefore reads the table and not a row here.

**Generation 0 is the generation the measurement reaches cheaply, and it is
not every generation.** A later generation holds a centre that has moved, so
its shaped spread may differ. Read these rows as the spread at the start of a
run.

## What this register does not hold

**It holds no discount.** A discount belongs to the learning algorithm, and the
product record says plainly that it chooses no learning algorithm.[^7] The
person brings one.

**It holds no cost figure.** One blocker governs every cost figure this project
holds, and it says which figures are measured and which are derived.[^8] A cost
of the learner seat belongs in a row under that blocker when a run on the
target platform measures it.

## References

[^1]: Documentation Rules. `.agents/rules/documentation.md`
[^2]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
[^3]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^4]: The reward of a faction. `python/cachette/learn/reward.py`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D1. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: Findings register, FND-583. `docs/FINDINGS.md`
[^7]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: ADR-0195, the observation of a faction is a fixed-width scale-free table in an egocentric frame, decisions D1 and D9. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^10]: Decisions register, DEC-282. `docs/DECISIONS.md`
[^11]: Research report 42, what a policy should be able to see, section 9. `docs/research/reports/42-what-a-policy-should-be-able-to-see.md`
[^12]: Findings register, FND-679. `docs/FINDINGS.md`
[^13]: Findings register, FND-692. `docs/FINDINGS.md`
[^14]: Findings register, FND-700. `docs/FINDINGS.md`
[^15]: The world scale probe. `scripts/world_scale.py`
[^16]: Findings register, FND-693. `docs/FINDINGS.md`
[^17]: Findings register, FND-705. `docs/FINDINGS.md`
[^18]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^19]: The ring frame of the observation. `crates/cachette-core/src/obs_ring.rs`
[^20]: The entity token block of the observation. `crates/cachette-core/src/obs_token.rs`
[^21]: The observation of a faction. `crates/cachette-core/src/faction_observation.rs`
[^22]: The simulation arithmetic, the compressed magnitude. `crates/cachette-core/src/sim_math.rs`
[^23]: The event history. `crates/cachette-core/src/event_memory.rs`
[^24]: The strategy table of the trainer. `python/cachette/learn/__main__.py`
[^25]: The test of the terminal scale. `tests/test_learner_terminal_scale.py`
[^26]: Target platform costs. `docs/reference/graviton-costs.md`
