# Report 40: What a well-trained policy needs

This report answers one question. What must this project build, and in what
order, for a learned policy to play better than the built-in controller?

**The short answer is that the trainer optimises one map at a time.** Each
generation scores its whole population on one world. The score of a candidate
is then dominated by that world. The direction the trainer estimates is the
direction that wins that world, and the next generation draws another world.
The run therefore walks, and the measured record shows a walk.

**One map for each generation is not by itself the defect.** A map-specific
direction averages away across generations, so long as the objective the
ranking uses stays the same objective. It does not. In 38 percent of the
generations of the measured run, every candidate shared one outcome, so the
ranking fell to the shaping term and the generation optimised held ground
instead of winning. **That is a bias, and no number of generations removes
it.**

Four further answers follow.

**The reward weights changed nothing, because only the order of the scores
reaches the update.** The trainer ranks the scores and takes a step of fixed
size. Two weightings that sort the population the same way give the same run.
Two of the six strategies in the code sort it the same way at almost every
generation.

**The observation and the action table are both weaker than the diagnosis so
far has stated.** The policy reads the map as four quadrants and cannot name a
place in any verb. Sixty-five percent of the observation is the trade board.

**The measurement protocol has been the wrong instrument.** The project
compares mean returns under different weightings, which are not comparable.
The one comparable quantity is the win share, the baseline for it is exactly
one third, and it needs no measurement at all.

**The four settings to change are a population of 64, two seeds per
generation, a hidden width of 8, and a smaller weight on held ground.** **The
fourth was a survival term and a measurement retired it. Read section 3.4
before acting on this report.** All four
are settings rather than code. A simulation of the trainer places that point a
quarter above the present one for the same episodes, and section 2.5 states
what the simulation assumes and how to falsify it.

## What has changed since this report

**This report is fixed to the moment section 0 names, and nothing below was
edited afterwards.** The recommended operating point names a policy kind that
no longer exists.

**The project deleted the kind that holds a frozen random projection in its
first layer.** A hidden width is a parameter of that kind, so a run cannot ask
for a hidden width of 8. The trainer now builds two kinds. One is `linear`,
over the whole array. The other is `structured`, which shares a weight across
the sector axis and trains every layer.[^25] The loader refuses a stored file
that names the deleted kind, and it names the kind in the refusal.[^26]

The other three settings of the recommendation stand. Read section 8 for what
a smaller trainable count buys, and read the structured kind as the shape that
now carries it.

**The observation is no longer the array this report reads.** The engine
publishes an egocentric ring frame with an entity token block, and it reached
layout version 7 on 8 September 2026. Section 5 measures the earlier array, so
read its share of the trade board as a fact of that array. A register holds the
parameters of the layout the engine builds now.[^27]

## 0 Provenance, and what this report could not verify

The author read the code in this repository, read the logs of one completed
training run, and computed figures from those logs. The author started no
instance, ran no training, and changed no code.

The report holds five kinds of claim, and each one is marked.

**Read.** The author read the source and states what it does.

**Measured.** The figure comes from a log of a run that already happened, or
from a schema the engine printed on the development machine.

**Derived.** The author computed the figure from a measured one. Every derived
figure states its arithmetic and its assumptions.

**Reasoning.** The author argues from the code and from published work. No
measurement supports it yet. Each such passage says so.

**Simulated.** The author wrote a model of the trainer, ran it, and reports
what it produced. **A simulated figure is not a measurement of this engine.**
Section 2.5 states the model, names the two quantities it guesses, and reports
every result at two values of the larger guess. Experiment 3 of section 11
tests the model against the engine.

**The author could not verify three things.**

The author could not verify the cost of a shorter decision interval. Nobody
has measured the boundary cost at an interval below five ticks, and BLK-007
governs every cost figure of this project.[^20]

The author could not verify the cause of the native faults. One finding
records that the faults are not the shard path, and nothing yet names what
they are.[^1]

The author could not verify that the wheel cache removes the seventeen minute
build on a real launch. The change is in the tree and its commit reports a
local build only.[^2]

## 1 The world, the interface, and the run, as they stand

**Measured.** The engine printed the observation schema and the action schema
for a world 48 tiles on a side with three factions.

The observation holds 184 positions. The table below groups them.

| Group | Fields | Positions | Share |
|---|---|---|---|
| Faction scalars | 12 | 12 | 7% |
| Relation, one for each faction | 1 | 3 | 2% |
| Trade board, five fields over 8 rows and 3 factions | 5 | 120 | 65% |
| Controller option weights | 1 | 5 | 3% |
| Level 1 cells, eleven fields over 4 cells | 11 | 44 | 24% |

The level 1 lattice holds four cells, because a block is 32 tiles on a side
and the world is 48. One cell therefore covers up to 1024 tiles. **The policy
reads the whole map as four quadrants, and one quadrant is 1024 tiles.**

The action table holds 29 rows over twelve verbs. Four verbs carry one
argument each: the resource kind, the upgrade category, the faction, or the
unit type. **No row of the table names a tile, a cell or a direction.**

The training world runs to a tick limit of 2500 ticks. The decision interval
is 10 ticks, so one episode takes at most 250 decisions.

**Measured.** One completed run gives the operating point. The run trained a
policy with a hidden layer of 24 over a reward that weighs held ground at one,
for 130 generations. Each generation held 256 candidates and one seed. The run
took 6.7 hours on one cell of 32 engine workers. One generation took 183
seconds at the median and ran 481,562 world ticks. One episode ran 1881 ticks
at the mean, so one cell scores 1.40 episodes each second, or 5036 episodes
each cell-hour.

## 2 The trainer optimises one map at a time

This section holds the primary finding of the report.

### 2.1 The world sets the score, not the candidate

**Measured.** Over the 130 generations of that run, the share of the 256
candidates that won its world had a mean of 0.167 and a standard deviation of
0.277. The share reached 0.00 in 46 generations and 1.00 in 4.

**Derived.** Suppose every candidate of a generation won with the same
probability 0.167, and suppose the outcomes were independent. The standard
deviation of the mean of 256 such draws is 0.023. The observed deviation
between generations is 0.277, which is 12 times as large. The variance between
generations is therefore about 140 times the variance a fixed difficulty would
produce.

Two things change between generations: the world, and the centre. The centre
moves by one step of a fixed fraction of its own length.

**Measured.** The same run played its centre on 128 fixed seeds every four
generations. Consecutive measurements moved by 53.7 at the mean. The standard
deviation over the last twenty measurements was 69.8. The run improved from
-1179.7 over the first ten measurements to -1111.3 over the last ten.

**Derived.** A win moves the return by 4000 under this weighting, so 53.7 of
return is 0.013 of win share. The centre therefore moves the win share by
about one point across four generations. The world moves it by 28 points
between two generations. **The world dominates the centre by more than a
factor of twenty.**

### 2.2 What the update therefore estimates

**Reasoning.** All 256 candidates of one generation play the same seed. The
difficulty of that world is common to every candidate, so it shifts every
score by the same amount and the ranking removes it. The part that survives is
the interaction between the candidate and that one world.

The trainer estimates that interaction accurately. Section 8 gives the
alignment law: the step of a generation of 128 pairs over 696 trainable
weights points at cosine 0.43 toward the direction it is estimating. The
direction it is estimating is the direction that wins one map. The next
generation estimates the direction that wins a different map.

**The run is therefore a walk between map-specific optima, taken at high
precision.** That is what the record shows. The validation figure oscillates
by about 70 and improves by about 68 over 120 generations.

**A walk of that kind still converges, so this is not the whole defect.** A
map-specific direction that is unbiased for the true direction averages away
across generations. Section 2.5 measures what the trade between seeds and
generations is really worth, and section 2.3 names the part that does not
average away.

### 2.3 The ranked objective also changes shape between generations

**Measured.** In 46 of 132 generations no candidate won, and in 4 every
candidate won. In 10 generations the score spread was exactly zero, so the
guard refused the step.

**Derived.** A generation where every candidate shares one outcome has no win
term to rank. The ranking then falls to the shaping term alone, which is the
change in held ground. Fifty of 132 generations, or 38 percent, ranked on held
ground alone. Eighty-two generations, or 62 percent, ranked mostly on the win
term, because one win is worth 4000 and the whole spread of the shaping term
inside one outcome class is a few hundred.

**The trainer therefore alternates between two objectives.** In 38 percent of
generations it maximises held ground. In 62 percent it maximises winning one
map. Nothing in the run holds those two together.

### 2.4 A hostile world is not a world to take out of the pool

**Measured, by the project owner during a live run.** One generation drew seed
1007. All 256 candidates scored exactly -2000.0, so the guard refused the
step. The built-in controller also loses that world from seat 0, after 207
decisions. Two neighbouring seeds behave differently. The controller loses
seed 1000 at decision 250, and it wins seed 1001 at decision 144.

| Seed | Water fraction | Founding filter | Controller in seat 0 |
|---|---|---|---|
| 1000 | 0.069 | passes | lost after 250 decisions |
| 1001 | 0.332 | passes | won after 144 decisions |
| 1007 | 0.223 | passes | lost after 207 decisions |

**The world is hostile to the seat, and no cheap static test finds it.** All
three pass the founding filter. The water fraction orders them 1000, 1007,
1001 and the outcome orders them differently. An earlier measurement already
established that the water fraction does not predict how an episode ends.[^22]

**Do not filter the seed pool on whether the controller wins from seat 0.**
Two reasons, and the author agrees with the project owner on both.

The holdout is unfiltered, so a filtered training pool optimises a different
distribution of worlds from the one the acceptance test measures. That is a
bias in the estimator, and no length of run removes it.

The filter would remove the hardest worlds. Those are the worlds where a
difference in skill has the most room to show.

**The zero-spread guard is the right mechanism, and it has the right shape.**
It removes one step, not one world. It fires only when the score did not
depend on the candidate at all, which is the only case that carries no
information. It fired in 10 of 132 generations of the measured run.

**The cost of a hostile world is smaller than it looks, and section 3.4 makes
it smaller still.** Held ground orders the candidates of a generation where
nobody won, and it orders them at 0.871. **An earlier version of this report
named a survival term here, and section 3.4 retires it.**

### 2.5 The seed count against the generation count, resolved by simulation

The project owner asked for this trade to be resolved against the alignment
law rather than by preference. **The author gave two wrong answers before the
right one, and states both, because each is the answer a reader would reach
unaided.**

**The first wrong answer.** Raise the seed count to eight. It buys direction
quality and the cost is arithmetic.

**The second wrong answer.** Carry the alignment law through to the
accumulated progress of the walk. Take the step as a fixed length along a
direction of alignment `cos`. Then after `G` generations the signal grows as
`G` times `cos` and the wander grows as the square root of `G`. The ratio is
the square root of `G` times `cos`, which is the square root of the budget
divided by the seed count and by the trainable count. **The population cancels
out, and one seed wins.** That derivation looks clean and it is wrong. It
assumes the centre never approaches the direction it is chasing, so it misses
the point at which per-step quality sets the level and more generations buy
nothing.

**The simulation.** The author wrote the model out and ran it. The objective
is linear. One direction is the true one. Each world gives that direction plus
a world-specific direction, scaled by a ratio the model calls the world
variance. Some share of generations rank a third direction instead, which is
the tie of section 2.3, and that share falls as the seed count rises, because
a generation ties on the outcome only when every one of its worlds ties. The
trainer draws the pairs, ranks the scores, sums the perturbations, normalises
the sum, steps by a fixed fraction, and returns the centre to unit length, in
the way the code does. The budget is a fixed episode count.

**Simulated, not measured.** The model is not the engine. It states a linear
objective and it guesses two quantities the engine has never measured: the
world variance, and how well the tied objective aligns with winning. The
author therefore reports each result at two world variances, and takes only
the conclusions that hold at both.

The trainable count is 696, the learning rate is 0.08, the tie share is 0.38
raised to the seed count, and the budget is 33,280 episodes. That budget is
exactly what the measured run spent: 130 generations of 256 candidates on one
seed. The figure is the cosine between the final centre and the true
direction, over eight repeats.

| Pairs | Seeds | Generations | Cosine at world variance 9 | Cosine at variance 25 |
|---|---|---|---|---|
| 128 | 1 | 130 | 0.581 | 0.412 |
| 128 | 2 | 65 | 0.570 | 0.390 |
| 128 | 4 | 32 | 0.450 | 0.304 |
| 128 | 8 | 16 | 0.299 | 0.211 |
| 32 | 1 | 520 | 0.622 | 0.483 |
| **32** | **2** | **260** | **0.730** | **0.574** |
| 32 | 4 | 130 | 0.703 | 0.537 |

**Three conclusions, and all three hold at both world variances.**

**Eight seeds is much worse than one at this budget.** The author's first
answer loses about half the level the centre reaches. The generation count is
the binding constraint at the budget these runs spend, and eight seeds cuts
that count by eight.

**The best point is a small population and two seeds, not a large population
and one.** Thirty-two pairs at two seeds reaches 0.730 against 0.581 for the
present setting of 128 pairs at one seed. That is a gain of a quarter for the
same episodes. The population is close to neutral at one seed, and it stops
being neutral once a second seed removes most of the tie.

**The trade turns with the budget.** At four times this budget, and at 128
pairs, two seeds reach 0.868 and four reach 0.891, against 0.745 for one seed.
**Reach that point by running longer, not by paying for seeds now.**

### 2.6 What therefore to change, and what it costs

The cost of a generation is the product of the population and the seed count.
The measured rate is 5036 episodes per cell-hour. **A population is twice a
pair count**, because each pair is tried in both directions, so the 32 pairs
of section 2.5 are a population of 64.

| Population | Seeds | Episodes per generation | Minutes | 400 generations |
|---|---|---|---|---|
| 256 | 1 | 256 | 3.0 | 20.4 cell-hours |
| 64 | 2 | 128 | 1.5 | 10.2 cell-hours |
| 128 | 2 | 256 | 3.0 | 20.4 cell-hours |
| 256 | 2 | 512 | 6.1 | 40.7 cell-hours |

**The recommended setting is a population of 64, two seeds, and several
hundred generations.** It costs 10.2 cell-hours for 400 generations. The
measured run spent 6.7 cell-hours for 130 generations. **This setting
therefore buys three times the generations for one and a half times the
cost.** One box runs two such arms at once.

**Two cautions, and both are reasoning rather than measurement.**

The model holds a linear objective, so it cannot see a step that destroys a
policy. A population of 64 takes a noisier step than one of 256, and a real
landscape may punish that. Watch the validation trace of the first fifty
generations before committing a long run.

The model has no guard. A population of 64 ties more often than one of 256,
because fewer candidates give fewer chances to differ. **Held ground at a
weight of 0.1 is what orders those generations**, and section 3.4 states why a
survival term is not.

## 3 Only the order of the scores reaches the update

### 3.1 What the code does

**Read.** The trainer ranks the scores into centred ranks. It then sums the
perturbations weighted by the rank difference of each pair. It then scales that
sum to unit length and takes a step of a fixed fraction. One finding records
that the length of the sum carries no information.[^3]

**The update is therefore a function of the ordering of the score vector and
of nothing else.** Two score vectors with the same ordering give the same
step, the same centre, and the same run.

### 3.2 The consequence for the reward weights

**Read.** The shaped part of the reward is the weighted change of a term since
the previous decision.[^4] The sum over an episode telescopes. The whole
shaped return of an episode is therefore the weight times the change of the
term between the start and the end. **No intermediate structure of the shaping
reaches an evolution strategy**, because the strategy reads one number for one
episode.

The terminal weights are plus 2000 for a win, minus 2000 for a loss and zero
for a draw. Held ground is bounded by the tile count, which is 2304.

**Derived.** Take one seed per generation. The score of a candidate is the
terminal weight of its outcome plus the weight times its net tile gain. The
conquest strategy weighs a tile at 0.1 and the ground strategy weighs it at 1.
The two orderings differ only when some pair crosses. A crossing needs a
weighted tile term above the 2000 gap between two outcome classes. At weight
0.1 the term cannot reach 230, so no crossing is possible. **At one seed per
generation the conquest weighting and the ground weighting are the same
experiment, unless a candidate gains more than 2000 tiles.**

**Measured.** Two strategies ran side by side in one process pair on one box,
with the same optimiser, the same population, the same seed and the same
policy shape. They differ only in that weight. In generation 0 no candidate
won, and the score spread of the ground arm was 363.0 against 36.3 for the
conquest arm. That is a ratio of exactly 10, which is the ratio of the two
weights. The same exact ratio appears in generation 6, and in three later
generations where no candidate won.

**Derived and unverified.** The two arms did later diverge, because their win
shares differ from generation 2 onward. A divergence requires a crossing.
The arithmetic above permits only one crossing: a candidate that lost or drew
while gaining more than 2000 of the 2304 tiles. The author could not verify
that such a candidate occurred, because the run logs no per-candidate score.

**What follows.** One finding attributes a difference between those two arms
to the density of the reward.[^5] That attribution is not established. The two
arms sorted their populations identically in every generation where no
candidate won, which is a third of them. A competing explanation is already
measured: the two arms of the earlier pair differed in policy shape, and the
alignment law accounts for the whole difference.[^3]

**Log the score vector of each generation.** It costs one line of code and one
file. Without it, no reward experiment in this project can be read.

### 3.3 What shaping would help

**Reasoning.** An evolution strategy needs a score that orders the population
the way the true objective orders it. Three properties matter, and density is
not one of them.

**The score must not tie.** A generation where every candidate shares one
outcome ranks on the remainder. Section 2.3 measures that this happens in 38
percent of generations.

**The score must order by the true objective.** The true objective is the win
share over all worlds. Held ground is a proxy. The measured record already
warns that a rise in held ground need not be a rise in wins.[^5]

**The score must place the shaping below the outcome.** A tile weight of 1
over a 2304 tile map can outrank an outcome, which section 3.2 shows. A tile
weight of 0.1 cannot. **Prefer the smaller weight.**

### 3.4 The survival term was wrong, and a measurement retired it

**This section recommended a survival term. The recommendation is withdrawn.**
The original reasoning is kept below, because the way it failed is the useful
part.

**What this section argued.** The observation carries the tick, and the reward
weighs the change of any field that holds one position.[^4] A faction cannot be
eliminated, so a faction that loses every unit keeps its ground and its seat and
may still win at the tick limit.[^21] **The author concluded that an episode
therefore ends early only when some rival wins**, so the end tick measures how
long the seat denied a win.

**What the argument missed.** An episode also ends early when the reading seat
**itself** wins, by domination. **Winners end early.** So a long episode is
evidence of not winning, and weighing the tick positively rewards not winning.

**Measured, and the measurement is what settled it.** A run of 28 candidates
over 8 worlds gave 249 winner-and-loser pairs. For each field it computed the
share of same-world pairs where the winner holds the higher value. A share of
0.5 is a coin.

| Field | Share | Field | Share |
|---|---|---|---|
| `held_tiles` | 0.871 | `population` | 0.735 |
| `seats_held` | 0.827 | `wonder_progress` | 0.528 |
| `store_total` | 0.771 | `best_renown` | 0.500 |
| `live_units` | 0.735 | `end_tick` | **0.444** |

**The end tick orders candidates backwards. Weighing it teaches the search to
lose.** The strategy that carried it, and its bound test, are removed from the
tree.[^23]

**The figures are narrow and the direction is not.** Only 11 of 224 episodes
were won, and all 249 pairs come from 3 maps. The candidates are random rather
than trained. **The gap between 0.871 and 0.444 is wide enough to act on. The
exact figures are not.**

**Two corrections to what stood here, and both matter to a later reader.**

**The stated bound was wrong.** This section said a weight below 0.8 cannot
cross an outcome boundary at a win weight of 2000. **A win and a loss stand two
win weights apart, so the gap is 4000 and the ceiling alone is 1.6.** The author
took the gap for the win weight. A later session made the same error and a test
now holds the arithmetic.[^23]

**The tie this section set out to break is real, and held ground breaks it.**
In the measured run 46 of 132 generations had no winner. Held ground orders the
candidates of those generations, and it orders them at 0.871. **A separate term
was never needed for the tie.**

**The lesson, which is the reason this section is kept.** A derivation about a
reward is a hypothesis about an ordering, and an ordering is cheap to measure.
**Measure it before shipping the weight.** Two independent derivations reached
this wrong answer and the agreement between them was read as evidence. A
companion report holds the failure register entry.[^24]

### 3.5 The one weighting to run

The concrete recommendation is one weighting, not six. **The tick row of this
table is removed, for the reason section 3.4 gives.**

| Term | Weight | Why |
|---|---|---|
| `won` | +2000 | The outcome dominates, as it should |
| `lost` | -2000 | |
| `drawn` | 0 | |
| `held_tiles` | 0.1 | The best proxy measured, at 0.871, and it cannot cross the gap |
| `seats_held` | 20 | The second best, at 0.827, and it adds a level where a seat changed hands |

**The shaped terms then pay at most 290 against an outcome gap of 4000.**

**Report the win share beside every return this weighting produces.** Section
9 holds the protocol.

## 4 Is an evolution strategy the right family here?

### 4.1 What it costs, stated exactly

**Read and derived.** An evolution strategy buys one scalar for one episode. A
policy-gradient method buys one gradient contribution for each decision of
each episode. One episode holds 250 decisions.

The alignment law fixes the price of the scalar. The cosine between the step
and the direction being estimated is about the square root of the pair count
divided by the trainable count.[^3] To double the alignment, multiply the
population by four. The cost of a generation rises with the population.

**The parameter budget an evolution strategy can afford is therefore in the
hundreds, not the thousands.** At 128 pairs the linear policy of 5365 weights
steps at cosine 0.154. A readout of 232 weights steps at 0.743 for the same
cost. Section 8 holds the table.

### 4.2 What a policy-gradient method would buy

**Reasoning.** The argument for switching is not the gradient. It is the
averaging over worlds.

An update of a policy-gradient method pools the transitions of every episode
in its batch. A batch of 512 worlds and 250 decisions holds 128,000
transitions from 512 different maps. The map variance averages out inside one
update. Section 2 shows that the map variance is the dominant term of the
current run, so this is the direct attack on the primary finding.

A policy-gradient method also carries no penalty from the parameter count. Its
estimate does not degrade as the square root of the dimension. The policy
could then hold thousands of weights and read a wider observation.

**The engine stays deterministic.** The policy becomes stochastic, and the
policy lives in the control plane. The engine receives one action integer as
it does now. No determinism rule of this project is touched.

**The hardware is not a barrier.** An earlier report measured that the
simulation holds between 96 and 99 percent of the wall clock of a
generation.[^6] The policy arithmetic does not matter at this size and will not.
A forward and backward pass over a few thousand weights costs nothing beside a
1881 tick episode. No graphics processor is needed.

### 4.3 What it costs to build

**Reasoning.** The environment already steps one decision at a time, returns a
reward for each decision, and reports termination and truncation. Four parts
are missing.

A stochastic policy over the masked action rows. This is a softmax over the
legal rows and one draw.

A value head, and an estimator of the advantage over the 250 decisions.

The clipped objective of proximal policy optimisation, and its optimiser.

A test that proves each part can fail, in the way the project's testing rule
demands.

The author estimates a few hundred lines of Python and its tests. That is a
judgement and not a measurement.

### 4.4 The recommendation

**Do not switch first.** Two cheaper experiments come before it, and both can
change what the switch should be built against. Section 11 orders them.

**Plan to switch.** The averaging argument in section 4.2 is strong, the cost
is bounded, and the alignment law puts a hard ceiling on what an evolution
strategy can carry. If the project intends a policy that reads a wider
observation, it needs a method whose step does not degrade with the parameter
count.

**Keep the evolution strategy as the control.** It is built, it works, and its
step is deterministic given the centre and the generation. A new method must
beat it on the acceptance test of section 9.

## 5 The observation

The brief names this as the primary suspect. This report does not agree that
it is the primary one, and it does agree that it is a real one. Section 2 is
the primary one.

### 5.1 What the array can and cannot represent

**Measured and read.** The array holds 184 positions. Of those, 120 are the
trade board and 44 are the spatial picture. An earlier report counted the
positions that move during three recorded episodes: 86 of 184 moved, and 81
of the 120 board positions never moved.[^7]

**Read.** The five option weight positions hold the weights of the built-in
controller of the reading faction. The learner's seat runs under external
control, so its own controller is off and its weights change nothing. **Those
five positions are structurally dead for a learner.**

**Reasoning.** The spatial picture is four cells of eleven fields. A cell
covers up to 1024 tiles. A policy can therefore express "there are more rival
units in the north-west quadrant than in mine". It cannot express anything
about a place, a frontier, a route, or a chokepoint. It cannot tell a compact
holding from a scattered one. The pyramid summarises, and at this world size
it summarises the whole map into four numbers for each quantity.

**What a player would use, and the array does not hold.** The author lists
these from the strategic areas an earlier report enumerated.[^7]

The location of a rival's settlements, at any resolution finer than a
quadrant.

The renown target and the wonder threshold, against which the array's own
progress figures would mean something. The array carries the tick limit but
not the other two.[^8]

Any statement of a frontier: which of the faction's tiles touch a rival's.

Any per-rival unit count. The two unit fields count the reader's own units and
every other faction's units together.

### 5.2 What evidence would confirm or clear it

**Reasoning.** Three measurements separate a weak representation from a weak
optimiser. Each is cheap.

**The best constant-preference policy.** Play the 29 policies that always
prefer one verb, and take the highest legal row otherwise. Each is a policy
that reads nothing at all. If the best of them matches the best trained
policy, then 130 generations of search bought nothing that an enumeration of
29 gives free, and the interface is the binding constraint. **Cost: 29 times
512 episodes, which is 2.9 cell-hours.** This is the cheapest decisive
experiment in the report.

**A predictability probe.** Fit the episode outcome from the observation at a
fixed decision, over a few hundred recorded episodes. If the outcome is not
predictable from the array at decision 50, no policy can act on the array at
decision 50. This runs on recorded data and costs no engine time beyond the
recording.

**A widened array.** The finer statement is already measured once. A
supervised fit of controller play barely beat one constant answer, and the
finding names the array as the stronger suspect.[^9] That measurement is
confounded by the window reduction, and the finding says so. The probe above
is not confounded, because it predicts the outcome and not an action.

### 5.3 What to change

**Reasoning, ranked.**

Drop the trade board from the policy input, or reduce it to a few aggregates.
It is 65 percent of the array and two thirds of it never moves. Drop the five
dead option weight positions.

Raise the spatial resolution. The pyramid block is 32 tiles on a side, which
gives four cells on this map. A learner-side pooling of level 0 into a fixed
grid of, for example, 8 by 8 cells would give 64 cells. That is a change in
the engine's observation reader, so it moves the observation version and
retires every stored policy.

Add the two missing thresholds: the renown target and the wonder threshold. An
earlier finding already records that the array omits the renown target while
carrying the matching tick limit.[^8]

**Train the first layer.** Section 8 holds this.

## 6 The action space and the decision cadence

### 6.1 One integer cannot say where

**Read.** No row of the 29 names a place. The engine resolves the place inside
the verb.

**Reasoning.** Published work divides on whether one action per decision is
enough. Every system that plays a real-time strategy game at a high level
issues an action for each unit at each step. One published system, TStarBot1,
plays the full game of StarCraft II with a flat space of 165 macro actions and
one decision at a time, and it beats the built-in agent at every level.[^10]
An earlier report of this project holds that comparison in full.[^7]

**The Cachette table is a macro action space of that kind, with 29 rows
against 165, and not one row names a place.** The count is not the difference
that matters. What each row reaches is.

The author's judgement: a spatial argument is the largest single gain
available in the action table, and it is also the most expensive change. It
needs a candidate space over the level 1 cells, which the mixed radix already
supports in principle.[^11] Four cells is a small gain. It becomes a real gain
only after the spatial resolution of section 5.3 rises.

### 6.2 The cadence

**Measured.** A recording of 32 episodes gave 5320 decision windows and
174,834 controller commands, which is 32.9 commands for one window of 10
ticks.[^9] A separate measurement counted 2.045 commands for one faction on
one tick, which is 20.45 for one window.[^12] The two figures disagree and
both are far above one.

**Reasoning.** The learner acts once for each window. A comparison against the
built-in controller therefore compares a rate before it compares a policy, and
one finding states that plainly.[^12]

**Lower the decision interval.** An earlier report proposes 10 ticks to 2, and
states the reason an evolution strategy tolerates the change: the estimate of
such a strategy does not depend on the episode length.[^7] That report also
says the boundary cost of the change was never measured, and it still has not
been.

**The author adds one caution.** A policy-gradient method does not tolerate it
as cheaply. Its variance grows with the horizon. An interval of 2 makes an
episode 1250 decisions. If the project intends to switch method, choose the
interval with that in mind, and prefer to measure both.

## 7 Credit assignment

**Read.** An episode holds up to 250 decisions. The terminal term is plus or
minus 2000 and it is paid once. The shaped term telescopes to the net change
of one quantity, as section 3.2 states.

**Reasoning.** For an evolution strategy this is not a credit assignment
problem at all. The strategy assigns no credit inside an episode. It compares
whole policies by whole episode returns. A horizon of 250 costs it nothing.
The published measurements of that family report near-identical curves across
frame skips.[^13]

**The horizon is therefore not the reason nothing learns.** The reason is in
section 2: one episode of one map is a poor measurement of a policy, and the
run takes one such measurement for each candidate.

**For a policy-gradient method the horizon is a real problem.** A reward that
is 2000 at the last decision and a few tenths at each of the other 249 gives a
value function a 250 step propagation to learn. Three things help, and each is
standard.

A value baseline and a generalised advantage estimate.

A potential-based shaping term. The current shaping already is one, because it
telescopes. **Its dense form has value only to a method that reads each step**,
which is exactly the method being considered. The same weighting that is inert
under an evolution strategy is useful under a policy gradient.

A reward for the quantity the win readers compare. The array holds the wonder
claim, which the wonder reader compares, and the finding that produced that
field says the work alone is the wrong quantity.[^14]

## 8 The parameter budget

### 8.1 What one step buys, and what the whole budget buys

**Measured.** The alignment law and its sweep are recorded.[^3] The cosine
between the step and the direction being estimated is about the square root of
the pair count divided by the trainable count. Scoring on one world roughly
halves it.

**Read.** The trainable count follows the policy shape.

| Shape | Trainable weights |
|---|---|
| Linear | 5365 |
| Hidden 32 | 928 |
| Hidden 24 | 696 |
| Hidden 16 | 464 |
| Hidden 8 | 232 |
| Hidden 4 | 116 |

**Derived.** The table below gives the alignment of one step before scoring
noise, at 32 and at 128 pairs.

| Shape | Trainable | Alignment at 32 pairs | Alignment at 128 pairs |
|---|---|---|---|
| Linear | 5365 | 0.077 | 0.154 |
| Hidden 24 | 696 | 0.214 | 0.429 |
| Hidden 16 | 464 | 0.263 | 0.525 |
| Hidden 8 | 232 | 0.371 | 0.743 |
| Hidden 4 | 116 | 0.525 | 1.000 |

**Simulated.** The alignment of one step is not the quantity to maximise. The
quantity to maximise is where the centre ends after the whole budget. The
model of section 2.5 answers that, at 128 pairs, one seed, a tie share of
0.38, a world variance of 9, and the budget the measured run spent.

| Trainable | Shape | Final cosine of the centre |
|---|---|---|
| 116 | Hidden 4 | 0.766 |
| 232 | Hidden 8 | 0.743 |
| 464 | Hidden 16 | 0.655 |
| 696 | Hidden 24 | 0.581 |
| 5365 | Linear | 0.286 |

**Cutting the trainable count helps monotonically, and it flattens below
232.** The linear policy ends at less than half the level of the hidden 24
policy on the same budget, which is why the linear cell of the measured run
needed about ninety generations to reach what the network cell reached in
about twenty.[^3] Between hidden 4 and hidden 8 the gain is 3 points, so
**take hidden 8 and keep the extra width**, because a narrower projection
states fewer rules.

**The recommended operating point is a hidden width of 8, a population of 64,
two seeds, and several hundred generations.** Section 2.6 holds the cost.

**Do not raise the population to buy alignment.** Four times the population
buys twice the alignment of one step and costs four times as much, so it buys
nothing at a fixed budget. Cutting the trainable count buys the same factor
for nothing.[^3] **Treat the population as a wall-clock parallelism knob and
not as a learning knob.** A cell holds 32 engine workers, so a generation of
128 worlds already fills one.

### 8.2 The learning rate is unresolved, and the model says 0.04 is too small

**Measured, by the project owner.** A pair of live runs compares 0.04 against
0.08 at matched generations. The 0.08 arm leads at every validation point so
far: -1167 against -1195 at generation 3, -1170 against -1294 at generation 7,
and -1110 against -1143 at generation 11. **Every gap is inside the plus or
minus 200 noise band, and the two arms drew different seed pools.** Treat the
learning rate as unresolved, not as tuned. The author states this correction
because an earlier brief called the halving diagnosis-driven, and it was not.

**Simulated.** The model of section 2.5 was run over the learning rate, at 128
pairs, one seed, a world variance of 9, and the budget the measured run spent.

| Learning rate | Final cosine, no tie | Final cosine, tie share 0.38 |
|---|---|---|
| 0.02 | 0.290 | 0.208 |
| 0.04 | 0.526 | 0.390 |
| 0.08 | 0.738 | 0.581 |
| 0.16 | 0.720 | 0.584 |
| 0.32 | 0.552 | 0.420 |

**The model reproduces the owner's observation and explains it.** The optimum
is a broad plateau between 0.08 and 0.16, and 0.04 is on the falling side of
it. A rate below the plateau cannot cross the sphere inside the budget. A rate
above it overshoots and the centre wanders.

**Keep 0.08 and test 0.16. Do not test below 0.08 again.** The author's own
earlier reasoning, that the learning rate cancels out of the signal-to-wander
ratio and is therefore neutral, is wrong for the same reason the second wrong
answer of section 2.5 is wrong: it ignores the saturation of the centre.

### 8.3 The fixed random projection

**Read.** The first layer of the network policy is a random projection from a
fixed seed. The trainer never touches it. Only the readout is trainable.

**Reasoning.** The projection is both a handicap and, at present, a benefit.

It is a handicap because the policy can only form functions of 24 fixed random
directions of the features. The informative variation of this observation is
narrow: 86 of 184 positions move, and 55 percent of the features have a
standard deviation below 0.01.[^7] A random projection spends its width in
proportion to variance, and most of the variance here is in positions that
carry nothing.

It is a benefit because it cuts the trainable count by a factor of 7.7, and
the alignment law makes that worth a factor of 2.8 in step quality.

**The choice conflates two things and nobody has separated them.** The clean
answer takes both gains. Cut the observation to the positions that move, then
train both layers. A trimmed input of about 40 positions with a hidden width
of 8 trains 40 times 8 plus 8 times 29, which is 552 weights. That is close to
the hidden 24 readout in count, and it is fully trainable.

**The experiment that decides it is cheap.** Run three arms at the same
population and seed count: the fixed projection at hidden 24, a fully trained
network at hidden 8 over the full array, and a fully trained network at hidden
8 over the trimmed array. Section 11 places it.

## 9 What "well trained" means, as a checkable statement

### 9.1 The two quantities the project has been confusing

**The mean return is not a measurement of play.** A return depends on the
weighting. A return that weighs held ground at 1 does not compare with one
that weighs it at 0.1. The stored policy index already says this.[^15]

**The win share is a measurement of play, and it is comparable across every
weighting.** Report the win share. Report the mean return only beside the
weighting that produced it.

**Derived.** The two are related when the weighting is known. With no draws,
the return is the win weight times twice the win share minus one, plus the
weighted net tile gain. The best stored policy scores -1043.7 under a win
weight of 2000 and a tile weight of 1. If its mean net tile gain matched the
controller's 245, its win share is 0.178. The controller wins 0.367 of its
episodes. **The best policy this project has trained wins about half as often
as the controller.** That conversion assumes the tile gain, so treat it as an
estimate and measure the share directly.

### 9.2 The baseline needs no measurement

**Measured and read.** The yardstick world gives the learner's seat back to
the built-in controller, so three copies of one controller play a symmetric
game. One seat of a symmetric three-faction game wins one third of the time,
whatever the controller does. One finding states this and records the two
corrections it took to reach it.[^16]

**The chance line is exactly 1/3, it is exact, and measuring it wastes
episodes.** The measured 0.367 over 256 episodes is 1/3 plus 0.55 standard
errors. Stop paying for the yardstick pass. Compare against 1/3.

### 9.3 The protocol

**Derived.** The standard error of a win share near one third is the square
root of two ninths divided by the episode count.

| Episodes | Standard error | 95 percent half-width |
|---|---|---|
| 128 | 0.042 | 0.082 |
| 256 | 0.030 | 0.058 |
| 512 | 0.021 | 0.041 |
| 1024 | 0.015 | 0.029 |
| 2048 | 0.010 | 0.020 |

The episode count needed to detect a given advantage over 1/3, at a one-sided
5 percent level with 80 percent power:

| True win share | Advantage | Episodes needed |
|---|---|---|
| 0.360 | +0.027 | 1956 |
| 0.383 | +0.050 | 569 |
| 0.400 | +0.067 | 317 |
| 0.433 | +0.100 | 143 |

**The protocol.** State each part of it in the run report.

The learner holds one seat. The built-in controller holds the other two.

The holdout is 512 fixed seeds that no training generation and no validation
pass has used. Every seed passes the founding filter.

Play each seed once in each of the three seats, which is 1536 episodes. The
observation names a faction by a position relative to the reader, so one
policy plays any seat.[^17] Rotating the seat removes any seat advantage from
the answer.

**The acceptance statement.** A policy is well trained when its win share over
those 1536 episodes is 0.383 or more. At 1536 episodes the standard error is
0.012, so 0.383 stands 4.2 standard errors above the chance line of 1/3.

**A weaker statement, for a first milestone.** A policy reaches parity with
the built-in controller when its win share over 512 episodes has a 95 percent
interval that contains 1/3 and a lower bound above 0.25.

**The cost is trivial.** 1536 episodes cost 18 minutes on one cell. The
project has been running 128 episode validations, which cannot separate 19
points. Nothing about the cost justified that.

**Report these five numbers and nothing else as the headline.** The win share,
the episode count, the standard error, the seat rotation, and the holdout seed
range. Report the return, the weighting, and the mean held ground below them.

## 10 The harness

Ranked by cost, cheapest first.

**The seventeen minute build is already fixed in the tree, and the fix is
unverified on a launch.** The launcher now caches one wheel keyed on the tree
hash of the crates, the lock file, the manifest and the toolchain pin.[^2] Its
commit reports a local build and a key comparison. **Verify it on the next
launch and record the boot-to-first-episode time.** Cost: one line in a log.

**A watchdog on progress, not on liveness.** Three failure shapes are
recorded: a segmentation signal, a lost worker process, and a live process
that advances no generation.[^18] The third is the expensive one, because it
is silent, and one instance spent about seven hours at half capacity. The
trainer already prints a heartbeat every 30 seconds. **Add a supervisor that
expects the next heartbeat and restarts the cell from its latest checkpoint
when none arrives.** The resume path exists and takes the centre, the
generation counter and the best score. Cost: a small script, and the author
judges it under a day.

**Find the native fault.** One finding records that the fault is not the shard
path and that nothing yet names it.[^1] The core crate is free of the Python
binding for exactly this reason, and the project runs a memory checker over
its unsafe code.[^19] **Enable core dumps on the training instances and keep
the dump.** A fault under load with no dump costs the whole run and teaches
nothing. Cost: an instance setting, and then whatever the dump shows.

## 11 The ordered plan

Each experiment states what it tests, what result kills it, and its cost in
cell-hours. One box holds two cells, so a box-hour is two cell-hours. Every
cost uses the measured rate of 5036 episodes per cell-hour.

**Experiment 1. The best constant-preference policy.** Play the 29 policies
that always prefer one verb over the 512 seed holdout, and report the win
share of each. *Tests:* whether search has produced anything that reading the
observation buys. *Kills the interface hypothesis if:* the best fixed
preference wins clearly less than the best trained policy. *Kills the search
hypothesis if:* the best fixed preference matches or beats it. *Cost:* 2.9
cell-hours. **This is the cheapest decisive experiment and it comes first.**

**Experiment 2. The recommended setting, run long.** One arm at a population
of 64, two seeds, a hidden width of 8, a learning rate of 0.08, and the
weighting of section 3.5, which no longer holds a survival term. Run 400
generations.
Validate on 512 fixed seeds every 20 generations. *Tests:* every change this
report recommends, together. *Kills it if:* the validation trace is flat
between generation 100 and generation 400. *Cost:* 10.2 cell-hours for the
run and 3.4 for the twenty validations, so about 14 cell-hours. **Run it
beside experiment 1 on one box.**

**Experiment 3. The seed count, at equal budget.** Two arms of 200
generations at 256 episodes per generation: 128 pairs at one seed, and 64
pairs at two seeds. Everything else matched to experiment 2. *Tests:* the
simulation of section 2.5, which predicts that the two seed arm ends
clearly higher. *Kills the simulation if:* the one seed arm matches or beats
it. *Cost:* 10.2 cell-hours for each arm, so about 22 cell-hours with the
validations. **This is the experiment that makes the simulation falsifiable,
so run it even if experiment 2 succeeds.**

**Experiment 4. The parameter budget and the projection.** Three arms at the
setting experiment 3 chose, for 200 generations: the fixed projection at
hidden 24, a fully trained network at hidden 8 over the full array, and a
fully trained network at hidden 8 over an array trimmed to the positions that
move. *Tests:* section 8.3. *Kills the trimming if:* the trimmed arm does
worse than the full arm. *Cost:* about 11 cell-hours for each arm, so about 35
cell-hours.

**Experiment 5. The decision interval.** Two arms at the settings experiment 3
chose: interval 10, and interval 2. Measure the boundary cost first, on one
generation, before committing the run. *Tests:* the cadence hypothesis.
*Kills it if:* the win share does not move. *Cost:* the interval 2 arm runs
five times as many decisions over the same ticks, and nobody has measured what
that costs. Measure that first. The tick count is unchanged, so the simulation
cost is unchanged, and an earlier report measured that the simulation holds 96
to 99 percent of the clock.[^6] The author expects a small rise and states
that as reasoning.

**Experiment 6. A policy-gradient learner.** Build the stochastic policy, the
value head, the advantage estimate and the clipped objective. Run it on 512
parallel worlds against the same holdout protocol. *Tests:* section 4.2.
*Kills it if:* it does not beat the evolution strategy control at equal
episode budget. *Cost:* the build is a few hundred lines and its tests. The
run costs what experiment 2 costs, because the episode count sets the cost and
not the method.

**Experiment 7. The spatial observation.** Raise the resolution of the level 1
picture the array carries, or add a spatial argument to the verbs that resolve
a place. *Tests:* sections 5.3 and 6.1. *Cost:* this is engine work, it moves
the observation version, and it retires every stored policy. Do it last, and
do it with a decision record, because it changes a schema the engine owns.

### 11.1 What to do this week

Change four things before any box starts. Each is a setting or one line, and
none is engineering.

Set the weighting to the table of section 3.5, and add no survival term. Set
the population to
64 and the seeds to two. Set the hidden width to 8. Log the score vector of
each generation.

Then run experiment 1 and experiment 2 together on one box. They fit inside
one box-day, and between them they test the interface and every setting this
report changes.

Stop paying for the yardstick pass, because the chance line is exactly one
third. Replace every validation of 128 seeds with one of 512. Report a win
share.

## References

[^1]: Findings register, FND-646. `docs/FINDINGS.md`
[^2]: The commit `Build the engine once for a set of sources, and keep the wheel for the next run`. Read its message for the key and the build command.
[^3]: Findings register, FND-668. `docs/FINDINGS.md`
[^4]: The reward of a faction. `python/cachette/learn/reward.py`
[^5]: Findings register, FND-650. `docs/FINDINGS.md`
[^6]: Report 38, where the training time goes. `docs/research/reports/38-where-the-training-time-goes.md`
[^7]: Report 34, what would move the learner. `docs/research/reports/34-what-would-move-the-learner.md`
[^8]: Findings register, FND-582. `docs/FINDINGS.md`
[^9]: Findings register, FND-643. `docs/FINDINGS.md`
[^10]: Sun and others, TStarBots: Defeating the Cheating Level Builtin AI in StarCraft II in the Full Game, 2018. https://arxiv.org/abs/1809.07193
[^11]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^12]: Findings register, FND-634. `docs/FINDINGS.md`
[^13]: Salimans and others, Evolution Strategies as a Scalable Alternative to Reinforcement Learning, 2017. https://arxiv.org/abs/1703.03864
[^14]: Findings register, FND-568. `docs/FINDINGS.md`
[^15]: The stored policy index. `checkpoints/README.md`
[^16]: Findings register, FND-645. `docs/FINDINGS.md`
[^17]: ADR-0193, a faction's observation names another faction by a position relative to the reader. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
[^18]: Findings register, FND-651. `docs/FINDINGS.md`
[^19]: ADR-0041, a crate split enforces the boundary at compile time. `docs/adrs/draft/adr-0041-a-crate-split-enforces-the-boundary-at-compile-time.md`
[^20]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^21]: Findings register, FND-583. `docs/FINDINGS.md`
[^22]: Findings register, FND-667. `docs/FINDINGS.md`
[^23]: The commit `Remove the survival term, because the tick of the end predicts losing`. Read its message for the 249-pair table.
[^24]: Report 41, a handbook for training a policy, section 9.1. `docs/research/reports/41-a-handbook-for-training-a-policy.md`
[^25]: The strategy table of the trainer. `python/cachette/learn/__main__.py`
[^26]: The stored policy reader of the control plane. `python/cachette/learn/policy.py`
[^27]: Reinforcement learning parameters register, the observation layout. `docs/reference/rl-costs.md`
