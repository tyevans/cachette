# Report 41: A handbook for training a policy

This report is a handbook. It says what the game offers a learner. It says how to
score each part of that, and how to combine those scores. It ends with what this
project has already got wrong.

A companion report answers a different question.[^1] It covers the optimiser,
the alignment law, the trade between seeds and generations, and the acceptance
protocol. **This report does not repeat it.** Read that report for the settings
and this one for the objective.

**The headline is that the reward is misspecified, and the measurement is
plain.** The trained policies grow, build, and expand. They found no city and
they fight little. That is not a failure of the search. It is the policy doing
what the score asked for, and the score asked for held ground.

## 0 Provenance, and what this report could not verify

The author read the engine, read the control plane, read one completed training
run, and computed figures from its log. The author started no cloud instance and
ran no training. The author ran the engine on the development machine to print
two schemas and to check one script.

Each claim carries a kind.

**Read.** The author read the source and states what it does.

**Measured.** The figure comes from a log, a stored report, or a schema the
engine printed.

**Derived.** The author computed the figure from a measured one, and states the
arithmetic.

**Reasoning.** The author argues from the code or from published work. No
measurement supports it. Each such passage says so.

**Three things the author could not verify.**

The author could not verify the win path of any episode. The engine records the
path, and no log of a training run stores it. Section 1.3 therefore states
which paths are reachable by arithmetic and not by count.

The author could not verify the proxy quality of any field. The script that
measures it was corrected in this session and its run had not finished. Section
4 states its method and its correction.

The author could not verify five entries of the failure register against the
register or the tree. Section 8 marks each of them and names its source.

## 1 One measurement, and the arithmetic that explains the play

### 1.1 The shaped term is the objective for most of a run

**Measured.** The completed run trained a policy over a reward that weighs held
ground at 1.0 and the outcome at plus or minus 2000.[^2] It ran 136
generations of 256 candidates on one seed each.

In 47 of the 136 generations no candidate won. That is 34.6 percent. In another
24 generations between one and three percent of candidates won. That is 17.6
percent. **Together, 52.2 percent of generations gave the outcome term almost
nothing to separate.**

**Derived.** The pair count is the sharper way to read it. The update ranks the
population, so what matters is how many candidate pairs the outcome decides. For
a win share `p`, the outcome decides a share `2p(1-p)` of the unordered pairs.
The rest fall to the shaped term.

| Quantity over the 136 generations | Value |
|---|---|
| Mean share of pairs the outcome decides | 0.124 |
| Median share of pairs the outcome decides | 0.039 |
| Generations where the outcome decides under a tenth of pairs | 87, or 64.0 percent |
| Generations where the outcome decides under half of pairs | 135, or 99.3 percent |

**Held ground decided 87.6 percent of all pairwise comparisons of the whole
run.** The outcome decided 12.4 percent. The shaped term was not a hint toward
the objective. It was the objective.

### 1.2 The arithmetic of held ground explains the play without any reward

The reward alone does not explain the play. The game does.

**Read.** A tile is held by the faction of the nearest city that reaches
it.[^3] The reach of a city is a base, plus one step for each block of
finished upgrades that stand on ground the city already held. The reach never
passes a bound.

**Read.** The register states three provisional values.[^4] The base reach is
8 hex steps. Four finished upgrades earn one step. The bound is 16 steps. The
minimum distance between two foundings of one run is 12 steps.

**Derived.** A hex disc of radius `r` holds `3r² + 3r + 1` tiles. So one city
holds up to 217 tiles at the base and up to 817 tiles at the bound. The training
world holds 2304 tiles.

**A single city with 32 finished upgrades therefore reaches 35 percent of the
world.** No settler is needed. Building upgrades is the direct territory
multiplier, and building is one verb with one argument that any live unit
answers.

**Founding a city is a sequence of three verbs with a delay.** A faction queues a
settler, moves it, and founds. The queue costs a unit slot and the founding
spends two people. An evolution strategy assigns no credit inside an episode, so
it must find that whole sequence by luck in one score.

**This is the mechanism, and it is reasoning rather than measurement.** Building
pays territory now, from a verb the mask almost always allows. Settling pays
territory later, through three ordered choices. Under a score that reads
territory, the search finds the cheap path first.

### 1.3 Two of the four win paths cannot fire in this world

**Read.** The engine tries four readers in a fixed order: domination, territory
at the tick limit, a wonder, and renown.[^5] The wonder reader fires when a
victory claim stands on ground the faction holds.[^6] An earlier record
retired that path, and the current one restores it for the wonder alone.[^7]

**Read.** The wonder needs 14400 units of work.[^8] The renown target is 1000
whole points, and one felled enemy unit pays a quarter of a point.[^9] So
the renown path needs 4000 enemy units to fall to one live champion.

**Measured.** The built-in controller played the learner's seat over 256 seeds of
the training world.[^2] Its mean wonder progress was 461.1 units of work. Its
mean best renown was 467008 raw fixed-point units, which is 7.13 whole points.

**Derived.** The controller reaches 3.2 percent of the wonder work and 0.7
percent of the renown target in a mean episode. **Neither path can fire in an
episode of 2500 ticks.** The live game is domination or territory, and territory
is exactly held ground compared at the tick limit.

**What follows, and it reframes the whole diagnosis.** The policies did not learn
a proxy instead of the objective. They learned one of the two live win
conditions. The reward reinforced it, and the game rewarded it as well. **The
reward is still wrong.** It pays for territory continuously. The game pays for
territory only at the tick limit, and only to the faction that holds more than
both rivals.
## 2 The mechanics as training material

A mechanic is training material when three things hold. The policy can observe
it, a verb can touch it, and a reward can score it.

**A reward term must name a field of the observation schema that holds exactly
one position.**[^10] That rule is the binding constraint of this section. A
field over several positions states no position, so the reward refuses it.

**Read and measured.** The table below holds every mechanic the engine
simulates. The observation column names the field or says none. The verb column
names the rows of the action table that reach it. The score column says whether
a reward can weigh it today.

| Mechanic | Observation | Verb | Can a reward score it |
|---|---|---|---|
| Held ground | `held_tiles`, and `cell_own_held_tiles` over 4 cells | `build` grows reach; `settle` adds a city | Yes |
| Seats | `seats_held` | `campaign` | Yes |
| Settlement | none counts a city | `settle`, `queue` | No |
| Unit count | `live_units`, own only | `queue` | Yes, in total only |
| Unit types | none counts a type | `queue` names 8 types | No |
| Upgrades | none counts an upgrade | `build` names 7 categories | No |
| Wonder | `wonder_progress`, `wonder_claim` | `build` on the wonder category | Yes |
| Terrain and water | `cell_open_tiles`, `cell_height_total` | `cross` | No |
| Weather | none | none | No |
| Fire | none | none | No |
| Stores and goods | `store_total`, one sum over every commodity | `gather`, `carry` | Yes, in total only |
| Trade board | 5 fields over 24 rows | `advertise`, `trade` | No |
| Relations | `relation`, one position for each faction | `relation` | No |
| Renown | `best_renown` | none directly | Yes |
| Roads and zones | none | `build` on the road category; `project` | No |
| Population | `population` | `queue`, and growth | Yes |
| Survival | `tick` | every verb, indirectly | Yes |

**Seven of the twelve verbs reach a mechanic that no reward can score.** Those
verbs are `build` in six of its seven categories, `relation`, `trade`,
`advertise`, `carry`, `project`, and `cross`.

**The two mechanics with the most observation positions are the two no reward can
name.** The trade board holds 120 of 184 positions and the relation field holds
3. Both hold more than one position, so both are unweighable. **The policy can
see the trade board and no score can ever ask it to use it.**

### 2.1 Three gaps that are findings rather than omissions

**The policy cannot see the quantity that decides its own territory.** Reach
follows the count of finished upgrades inside a city's ground. No field of the
array counts an upgrade. So a policy that built 31 upgrades cannot see that one
more buys a step of reach. **This is the sharpest observation gap in the array**,
because territory is one of the two live win paths.

**The weather changes the economy and the array says nothing about it.** The step
solves the weather, and a wet cell raises what a unit gathers from a tile. No
field of the array reports air, cloud, ground, or wetness. The policy therefore
plays a partly hidden economy and cannot condition on it.

**The legality answer is honest for two verbs and permissive for two others.**
The build verb asks each live unit whether it could build that category, and the
relation verb asks the refusal directly.[^11] The gather verb and the queue
verb only check that the argument names a row of an enumeration. So the mask
allows a queue of unit type 4, whose table row holds zero in every column and
which can do nothing.[^12] **A learner that queues that row gets a dead unit
and the mask said yes.**

### 2.2 What the policies actually do, measured

**Measured.** A stored behaviour report holds the verb distribution of six
policies over about 600 decisions each.[^13] The report does not record
the world extent or the tick limit of the run that produced it, so read the
shares and not the absolute counts.

| Policy, by the term its reward weighed | Verb shares |
|---|---|
| Ground at 1.0 | `build` 0.863, `gather` 0.053, `advertise` 0.048, `queue` 0.030, `no_op` 0.005 |
| Population at 3.0 | `build` 0.840, `settle` 0.138, `queue` 0.022 |
| Stores | `gather` 0.930, `no_op` 0.040, `queue` 0.030 |
| Untrained | `no_op` 1.000 |
| Uniform over the legal rows | `gather` 0.296, `build` 0.242, `advertise` 0.178, `no_op` 0.129, `queue` 0.118, `relation` 0.018, `trade` 0.007, `campaign` 0.007, `settle` 0.003, `cross` 0.002 |

**This measurement settles the diagnosis.** The ground reward produced a policy
that never founds a city. **The population reward produced a policy that founds
in 13.8 percent of its decisions**, and it reached 22 people against the ground
policy's 8. The reward chose the playstyle, and it chose it visibly.

**No trained policy in the report ever chose `campaign`, `carry`, `project`,
`cross`, or `trade`.** The uniform draw chose four of those five, so the mask
admits them at least sometimes.

**Derived.** The uniform draw took 595 decisions over 29 rows. It never chose
`carry` and never chose `project`. One always-legal row of 29 would appear about
20 times in 595 draws. **The mask therefore almost never admits those two verbs.**
That agrees with what makes them legal. A carry needs a contract, and a project
needs a zoned plan.[^11] That is reasoning from a count, and a direct
measurement of the mask would settle it.
## 3 Playstyles as objectives

The project owner asked for six playstyles: aggressive, protective expansionist,
trade, diplomacy, population growth, and wealth hoarding.

**A playstyle is a weighting, not a policy shape.** Section 2.2 measures that.
The same optimiser and the same network gave a builder under one weighting and a
founder under another.

### 3.1 What each playstyle needs, and whether it exists

| Playstyle | The fields that would score it | Exists today |
|---|---|---|
| Population growth | `population`, and `tick` to keep it alive | Yes |
| Wealth hoarding | `store_total`, and `tick` | Yes |
| Protective expansionist | `held_tiles`, `seats_held`, `tick` | Yes, and it is what the run trained |
| Aggressive | a count of rival units felled, or rival seats taken | Partly. `seats_held` counts seats. `best_renown` rises only when rivals fall, so it is the one available proxy for killing |
| Trade | contracts settled, goods moved, or a store gained through trade | No. The board holds 5 fields over 24 rows and no reward can name one |
| Diplomacy | the relation toward each rival | No. The relation field holds one position for each faction |

**Two playstyles are impossible to score, and the reason is one rule.** A reward
term must name a single-position field.[^10] Trade and diplomacy are the two
mechanics whose whole state is a table over rivals or rows.

**The fix is small and it is engine work.** Add single-position summaries beside
the tables: a count of settled contracts, a total of goods moved, and a mean or a
minimum relation toward the rivals. Each is a new field of the observation
schema, so each moves the observation version and retires every stored
policy.[^14] Do it once, with a decision record, and add every summary in
the same change.

**Aggression is scoreable today and it is scoreable badly.** Renown is the only
field that rises when rivals fall, and it rises at a quarter point for each
unit against a target of 1000.[^9] A weight large enough to matter over a
2500 tick episode is a weight the bound of section 5 refuses. **A count of
felled rival units, as one position, is the field aggression needs.**

### 3.2 Separate runs, not a league

**Read.** The league seats several candidates in the seats of one world. It
scores each by its own return minus the mean return of the other learner seats
of that world.[^15] It takes **one** weighting for the whole run, and its
candidates are all perturbations of one centre.

**A league therefore cannot host two playstyles.** Two reasons, and the second is
the binding one.

The runner takes one weighting and applies it to every seat. The per-seat reward
objects are already keyed by seat, so a per-seat weighting is a small change.

**The trainer ranks the whole population in one ordering.** A ranking mixes
every candidate together. Two candidates scored under two different weightings
are two different quantities, so an ordering over both means nothing. That is
not a small change. It is a different algorithm.

**Playstyle diversity therefore comes from separate runs.** Train one arm for
each weighting, store one policy for each arm, and evaluate them together.

**The evaluation path already exists.** The demonstration binary seats a
different stored policy in each faction of one world.[^16] So a
tournament between separately trained playstyles is available now, and it needs
no new code.

**What a league would buy, and it is not diversity.** The published league of a
strong StarCraft II agent used explicit main agents and exploiter agents. Those
agents held different objectives, and the league needed a population-level
algorithm to combine them.[^17] This project has one centre and one ranking.
**Use the league for sample efficiency, which is what it was built for, and use
separate runs for playstyle.**

## 4 Scoring one mechanic

This section is the teaching one. It states why a proxy becomes the objective,
and it gives the test for whether a proxy is any good.

### 4.1 Why a proxy becomes the objective

**The mechanism is the ranking, and it is exact.** The trainer sorts the scores
of a generation into centred ranks. It then sums the perturbations weighted by
the rank difference of each pair, scales that sum to unit length, and steps by a
fixed fraction.[^18] **The update is a function of the ordering of the score
vector and of nothing else.**

So the question is never "how much does the shaped term contribute". The question
is "which term decides the order". Section 1.1 answers it for the measured run:
held ground decided 87.6 percent of the pairs.

**A proxy that decides the order is the objective.** The search cannot tell the
difference. This is the specification-gaming shape that published work
names.[^19] It arrives here through the ranking rather than through a value
function.

### 4.2 The test for whether a proxy is any good

**The test is a within-world ordering test.** Take one world. Play a spread of
candidates on it. Split them into the ones that won and the ones that did not.
Then ask: over every winner and loser pair, in what share does the winner hold
the higher value of the field?

**That share is the proxy quality of the field on that world.** Read it like
this.

A field at 1.0 orders candidates exactly as winning does.

A field at 0.5 orders them no better than a coin.

**A field below 0.5 orders them backwards. Weighing it teaches the search to
lose.**

**The comparison must be inside one world, and that is the part people get
wrong.** A generation plays one world and ranks the candidates of that world. A
field that predicts winning across worlds and not within one is no use to the
search. Held ground across worlds is partly a statement about the world, and the
ranking removes everything common to the world.

### 4.3 The script, its method, and the correction it needed

**Read.** A script measures this.[^20] It plays a spread of random-weight network
policies over a few worlds. It reads the value of every reportable field at the
end of each episode. It then computes the within-world winner-and-loser share.

**The core method is sound.** Three things about it are right, and one of them is
not obvious.

The within-world pairing is the right unit, for the reason section 4.2 gives.

The candidates come from the search shape, so the spread is a spread the search
could see.

**Reading the field at the end of the episode is equivalent to reading its
change, inside one world.** The reward pays the weighted change since the
previous decision, which telescopes to the change over the episode. Every
candidate of one world starts from the same value, so the end value and the
change order identically. The script never states this, and it is why the method
is valid.

**The author found four defects and corrected each.**

**The candidate distribution was overclaimed.** The docstring said the candidates
are drawn the way a generation draws them. They are unit-norm random weights from
a zero centre. That is generation zero and not a generation of a trained centre.

**The two schema lengths were written by hand.** The script passed 29 and 184 to
the policy constructor. Those are numbers the engine owns, and nothing failed
when the two disagreed. This is the first recurring defect shape of this
project.[^21] The script now takes both from a probe environment.

**A field with no ordered pair sorted into the middle of the table.** The
not-a-number sort key evaluated to zero, which placed an unmeasured field above
every field below a coin. It now sorts last.

**The measure is confounded with how long the episode ran, and this one is
serious.** An accumulating field reads lower on a candidate whose episode was
short. **A winner ends its episode the moment it wins**, so a win truncates every
accumulating field. The raw share therefore reads a quick win as a low value, for
a reason that is not about play. The script now reports a second share beside the
first, over the value divided by the end tick.

**Measured, and far too small to conclude anything.** The corrected script ran
8 candidates over 4 worlds. One episode of 32 was won, so exactly one world could
order anything, and every figure rests on 7 pairs.

| Field | Total | Rate |
|---|---|---|
| `live_units` | 0.643 | 1.000 |
| `population` | 0.643 | 1.000 |
| `seats_held` | 0.500 | 1.000 |
| `best_renown` | 0.500 | 0.500 |
| `held_tiles` | 0.429 | 1.000 |
| `store_total` | 0.429 | 1.000 |
| `wonder_progress` | 0.286 | 0.286 |
| `end_tick` | 0.000 | 0.500 |

**Read this table as a demonstration of the confound and not as an answer.**
Seven pairs cannot separate 0.429 from 0.643. What the table does show is that
the correction changes the sign of the reading. Held ground reads below a coin as
a total, and above a coin as a rate. **Run the script at a size that can
answer**, and section 9 places it.

**One refinement of the companion report follows from the last row.** The
survival term reads 0.000 as a total, and that is arithmetic rather than noise.
A winner ends its episode the moment it wins. So within one world the winner
holds the lowest end tick of any candidate. The companion report calls the term
monotone toward the tick-limit path. That is true, and it does not claim the term
orders winning. **Do not read
it as ordering winning. It orders survival, and the bound of section 5 is what
keeps it from outranking a win.**

## 5 Integrating scoring across mechanics

This is the section the project owner asked to understand. It holds six parts.

### 5.1 The magnitude of a weight is irrelevant. The choice of field is not

**Read.** The trainer ranks the scores.[^18] Two score vectors with the same
ordering give the same step, the same centre, and the same run.

**So a weight matters only where it changes an ordering.** Multiply every weight
by ten and nothing changes. Multiply one weight by ten and something changes only
if some pair crosses.

**Worked example, and it is measured.** Two strategies weigh held ground at 0.1
and at 1.0. Everything else is matched. In generation 0 neither had a winner. The
score spread of the ground arm was 363.0 and of the other arm 36.3. **That is a
ratio of exactly 10, which is the ratio of the weights.** The two arms ranked
their populations identically, because scaling every score by ten preserves the
order.[^1]

**The rule to carry.** Ask of every weight: does it move any pair across another?
If the answer is no, the weight is a label and not a setting. **Tuning a weight
that crosses nothing is a way to spend a day and change nothing.**

### 5.2 A sum of weighted terms is one blended objective

A reward of several terms looks like several objectives. It is not.

**A weighted sum collapses to one number, and one number gives one ordering.**
Two candidates that differ on two terms are compared through the weights. A
candidate that gains 100 tiles and loses 5 people is ranked against one that
gains 50 tiles and gains 5 people by whether `100w_t - 5w_p` exceeds
`50w_t + 5w_p`. **The weights fix an exchange rate, and the search obeys that
rate exactly.**

**So there is no such thing as "also caring about" a term.** Adding a term does
not add a goal. It changes the exchange rate of the single goal the sum defines.

**The practical consequence.** Write the exchange rate down before you write the
weights. State what one tile is worth in people, and what one person is worth in
stores. If you cannot state it, you do not yet know what objective you are
training.

### 5.3 Consecutive generations that optimise conflicting objectives

This is what the measured run did, and it is the reason it walked.

**The mechanism.** In a generation where nobody won, the ranking falls entirely
to the shaped term. In a generation where the win share is near a half, the
outcome decides most pairs. So the objective the ranking expresses **changes
shape between generations**.

**What that does to an evolution strategy.** The strategy takes a step of a fixed
length along a direction estimated from one ranking. If consecutive rankings
express different objectives, the steps point at different optima.

**A map-specific direction averages away. An objective-specific direction does
not.** A world drawn at random gives a direction that is the true direction plus
noise, and the noise cancels over generations. An objective that switches between
two targets gives a direction that is a mixture, and the mixture does not cancel.
**The centre ends between the two optima, and no number of generations moves
it.** The companion report states this and measures the split.[^1]

**Read the centre trace for the signature.** The measured run's validation figure
oscillated by about 70 and improved by about 68 over 120 generations. A walk
between two attractors looks exactly like that: motion without progress.

**The cure is not a smaller learning rate.** A smaller step reaches the same
mixture more slowly. **The cure is to make the ranking express one objective in
every generation**, which is what a term that never ties buys.

### 5.4 The bound, and how to derive it for any term set

**The bound stops a shaped term from outranking the outcome.** Here is the
derivation, and it is arithmetic.

The outcome puts a win and a loss into two classes. A win pays `+W` and a loss
pays `-W`, so the **gap between the two classes is `2W`**, not `W`.

Every shaped term pays into that gap. Let each term `i` have weight `w_i` and an
episode ceiling `C_i`, which is the largest value that term can reach over one
episode.

**The rule is: `Σ |w_i| · C_i < 2W`.**

Below that sum, no candidate that lost can outrank one that won. At or above it,
a candidate that lost slowly can outrank one that won quickly, and the search
learns to lose slowly.

**Worked example.** The world holds 2304 tiles and the tick limit is 2500. The
win weight is 2000, so the gap is 4000. This is the weighting one commit added
and another session was removing while this report was written, so read the
arithmetic and not the shipped state.[^22]

| Term | Weight | Ceiling | Contribution |
|---|---|---|---|
| `held_tiles` | 1.0 | 2304 | 2304 |
| `tick` | 0.5 | 2500 | 1250 |
| | | **Sum** | **3554** |

3554 is below 4000, so the weighting is safe. Raise the survival weight and the
sum crosses at `(4000 - 2304) / 2500`, which is 0.6784. **The weight that commit
shipped is 0.5, and three tests hold the bound.** One of the three proves the
bound can fail.

**Note the ceiling is an episode ceiling and not the schema bound.** The schema
declares the tick bound as the whole range of a 64-bit integer. A tighter bound
would be a measured figure, and a blocker governs those.[^23] The right
ceiling for the tick is the tick limit, which the run sets.

**Two gaps in that check, and both are real.**

**The test covers one weighting.** It is parametrised over a single strategy
name. Every other weighting of the project is unchecked.

**Three weightings weigh a term whose episode ceiling nobody has stated.** The
stores weighting weighs `store_total`, and the people weighting weighs
`population` at 3.0. **Derived:** a net gain of 1334 people would cover the gap at
that weight. Measured population is a single digit, so no run has met it. Nothing
checks it, and the housing bound that would make it unreachable is not stated in
the test.

### 5.5 What a reward curve is, and how to read one

**A reward curve is the value of one term plotted against the episode, for a set
of candidates on one world.** It answers three questions that a single number
cannot.

**Does the term move at all?** A term that is flat across every candidate orders
nothing, whatever its weight. The stores term over a policy that never gathers is
flat.

**When does it separate the candidates?** A term that separates only in the last
ten decisions gives the search almost no discrimination on a world that ends
early. A term that separates by decision 50 discriminates on every world.

**Does its ordering agree with the outcome?** Plot the winners in one colour.
If the winners sit below the losers on the term, the term is a proxy below
chance, and section 4.2 says what that costs.

**How to draw one, from what already exists.** The environment reports the
reward breakdown of every decision, term by term, and the raw change beside the
weighted contribution.[^10] So a curve needs a loop over the decisions of a
recorded episode and nothing new in the engine.

**The one thing a curve cannot tell you.** It cannot tell you the exchange rate
between two terms. That is section 5.2, and it is a judgement about the game
rather than a measurement of it.

### 5.6 The integration this report recommends

**Recommendation. Replace the reward with a two-term score: the outcome, and
survival. Drop held ground entirely for the first arm.**

| Term | Weight | Ceiling | Why |
|---|---|---|---|
| `won` | +2000 | | The outcome should decide every pair it can |
| `lost` | -2000 | | |
| `drawn` | 0 | | |
| `tick` | 0.5 | 2500 | Never ties while two episodes end at different ticks. Pays 1250 of the 4000 gap |

**Four reasons, and the fourth is the one that decides it.**

**Held ground is already a win path.** Territory at the tick limit compares held
ground. Paying for it continuously pays twice for the same thing, and it pays for
it on a schedule the game does not use.

**Held ground is the term that captured the run.** It decided 87.6 percent of the
pairs of 136 generations. Removing it is the smallest change that removes the
bias.

**Survival never ties while two episodes end at different ticks.** It has 250
levels over an episode of 250 decisions, against the two levels of an outcome.
That is what makes the ranking express one objective in every generation, which
is section 5.3.

**Survival is monotone toward the win condition the game can actually
resolve.** A faction cannot be eliminated, so an episode ends before the limit
only when a rival wins.[^24] A faction that reaches the limit is compared on
held ground. **So surviving to the limit is the precondition of the territory
path, and it is a precondition rather than a proxy.**

**Then, and only then, add held ground back at 0.1 as a second arm.** Two arms
that differ in one weight, run at the same population and the same seeds, tell
you what the term is worth. **Log the score vector of each generation**, or
neither arm can be read.[^1]
## 6 Model tiers

**The tier ladder of this project is a ladder of methods and not of sizes.**
The reason is one law. Take the step an evolution strategy takes, and the
direction it estimates. The cosine between the two is about the square root of
the pair count divided by the trainable count.[^25] So the parameter budget is
set by the population, and the population is what a generation costs.

**Read.** The current policy is a network with one hidden layer. **Its first
layer is a random projection from a fixed seed, and the trainer never touches
it.**[^26] Only the readout is trainable.

**Derived.** At the training layout the linear policy trains 29 times 185, which
is 5365 weights. The network at hidden 24 trains 29 times 24, which is 696.
**The network is 7.7 times the smaller search shape, not the larger one.** The
project believed the opposite and the register records the correction.[^25]

### 6.1 The tiers, and what each one costs

**Measured.** The throughput reading of the target platform gives the cost
base.[^27] A 64-core instance costs 0.7658 dollars an hour. One cell
holds 32 engine workers, so a box holds two cells and a cell-hour costs 0.383
dollars. One cell scores 5036 episodes an hour.[^1]

**Derived.** One thousand episodes cost 7.6 cents. A generation of a population
of 64 on two seeds is 128 episodes, so it costs about one cent and takes 1.5
minutes. Four hundred such generations cost 10.2 cell-hours, or 3.90 dollars.

| Tier | Shape | Trainable | Alignment at 128 pairs | Method | Observation it needs |
|---|---|---|---|---|---|
| Starter | Trained two layers over the moving positions, hidden 8 | 560 | 0.478 | Evolution strategy | The present array, indexed down. **No engine change** |
| Present | Fixed projection, hidden 24, trained readout | 696 | 0.429 | Evolution strategy | The present array |
| Present, narrowed | Fixed projection, hidden 8, trained readout | 232 | 0.743 | Evolution strategy | The present array |
| Intermediate | Trained two layers over the whole array, hidden 32 | 6848 | 0.137 | Policy gradient | The present array |
| State of the art | Two convolutions of 32 channels over a spatial grid, then a head | about 16000 | 0.089 | Policy gradient | A finer spatial grid. **Engine change** |

**Derived, and the alignment column is the whole argument.** An evolution
strategy at 128 pairs points at cosine 0.137 toward a 6848-weight direction.
Scoring on one world roughly halves it again.[^25] **An evolution strategy
cannot carry the intermediate tier or the top tier.** Four times the population
buys twice the alignment and costs four times as much, so it buys nothing at a
fixed budget.

**The starter tier is the interesting row, and it is where to go next.** It trains
both layers, it holds fewer weights than the present policy, and it needs no
engine work. The trimming happens on the learner's side, by indexing the array
before the encoder.

**A trimmed and fully trained network takes both gains at once.** The fixed
projection is a handicap and a benefit together. It is a handicap because the
policy can only form functions of 24 fixed random directions, and 55 percent of
the features barely move.[^28] It is a benefit because it cuts the trainable
count by 7.7. **Nobody has separated the two, and the starter tier separates
them.**

### 6.2 What the arithmetic costs, at every tier

**Derived.** A generation of the top tier runs 128 worlds and 250 decisions. A
forward pass over 16000 weights costs about 32000 arithmetic operations. So the
whole generation costs about 1.0 thousand million operations.

**One generation takes 183 seconds at the median.**[^1] So the policy
arithmetic runs at about 6 million operations a second. One core of the target
does far more than that.

**The policy arithmetic is free at every tier, and it stays free.** An earlier
report measured the simulation at between 96 and 99 percent of the wall clock of
a generation.[^29] **No graphics processor is needed at any tier in this
table.** A backward pass costs about twice a forward pass, so a policy gradient
does not change that answer.

### 6.3 Whether a policy gradient changes the tier answer

**It changes it completely, and that is the case for switching.**

**Reasoning.** A policy gradient pools the transitions of every episode of its
batch. Its estimate does not degrade as the square root of the parameter count. A
batch of 512 worlds and 250 decisions holds 128,000 transitions from 512
different maps, so the map variance averages inside one update.

**Four parts are missing and each is standard.** A stochastic policy over the
legal rows, which is a softmax and one draw. A value head. A generalised
advantage estimate over the 250 decisions. The clipped objective and its
optimiser.[^30]

**The engine stays deterministic.** The policy becomes stochastic and the policy
lives in the control plane. The engine still receives one action integer. No
determinism rule of this project is touched.[^31]

**One caution, and it is reasoning.** A policy gradient does not tolerate a
shorter decision interval as cheaply as an evolution strategy does. Its variance
grows with the horizon. An interval of 2 ticks makes an episode 1250 decisions.
**Choose the interval and the method together.**

## 7 Inputs

**Measured.** The engine printed the schema for a world 48 tiles on a side with
three factions. The array holds 184 positions over 30 fields.[^32]

| Group | Fields | Positions | Share |
|---|---|---|---|
| Faction scalars | 12 | 12 | 7 percent |
| Relation, one for each faction | 1 | 3 | 2 percent |
| Trade board, five fields over 8 rows and 3 factions | 5 | 120 | 65 percent |
| Controller option weights | 1 | 5 | 3 percent |
| Level 1 cells, eleven fields over 4 cells | 11 | 44 | 24 percent |

**Read.** The block of the summary lattice is 32 tiles on a side, so a world of
48 holds four cells. **One cell covers up to 1024 tiles.** The policy reads the
whole map as four quadrants.

**Read.** A faction-indexed field is addressed by the distance from the reader and
never by a seat number.[^33] Position zero names the reader. Position `k`
names the faction `k` seats after it. So one policy plays any seat, which is what
the seat rotation of the acceptance protocol needs.

### 7.1 What a player uses that the array does not carry

The companion report names four of these.[^1] The author adds five more, and
the first is the important one.

**The count of finished upgrades inside the faction's own ground.** This is the
quantity that decides reach, and reach decides held ground.[^3] No field
counts it. **A policy that built 31 upgrades cannot see that one more buys a step
of territory.**

**Anything about the weather.** The step solves the weather and a wet cell raises
what a unit gathers. No field reports it. The policy plays a partly hidden
economy.

**Anything about a fire.** The engine simulates a wildfire and no field reports
one.

**The count of the faction's own settlements, and their reach.** The seats field
counts seat tiles and not cities.

**The contracts and the zoned plans.** Two verbs are legal only when those exist,
and the array says nothing about either. **A policy cannot learn when the carry
verb becomes available.**

**Read.** Five positions hold the option weights of the built-in controller of the
reading faction. The learner's seat runs under external control, so its own
controller is off. **Those five positions are structurally dead for a
learner.**[^1]

### 7.2 What belongs in the array against what belongs in a verb argument

**The rule is one sentence. The array describes the choices, and the argument
names one.**

A verb argument names one member of a set the engine can enumerate. The build
verb names one of seven categories, and the array must describe the seven.

The array carries state the policy conditions on. It must hold one number for
each thing the argument can name, or the policy chooses blind.

**The two halves are broken together, and that is why fixing one buys nothing.**
No row of the action table names a tile, a cell, or a direction.[^34] And the
array describes only four quadrants. **A spatial argument over four cells is
almost no argument, and a finer grid with no spatial argument is almost no
input.** Raise both in one change or raise neither.

**Read this as the reason the action table is a macro table.** A published system
plays the full StarCraft II game with a flat space of 165 macro actions and beats
the built-in agent at every level.[^35] This table holds 29 rows and not one
names a place. **The count is not the difference that matters. What each row
reaches is.**

### 7.3 The ranked changes to the input

**Reasoning, cheapest first.**

**Index the array down, on the learner's side.** Drop the five dead option weight
positions and reduce the trade board to a few aggregates. That is 125 of 184
positions, and 81 of the 120 board positions never move.[^28] **No engine
change, and it enables the starter tier.**

**Add the two missing thresholds and the upgrade count.** The renown target, the
wonder work threshold, and the count of finished upgrades in own ground. The
array already carries the matching tick limit, so the omission is an asymmetry
rather than a rule.[^36] Each is one position.

**Add the single-position summaries that trade and diplomacy need.** Section 3.1
names them.

**Raise the spatial resolution.** Pool level 0 into a fixed grid of, for example,
8 by 8 cells. That gives 64 cells against 4.

**Do the last three in one change, with a decision record.** Each moves the
observation version and retires every stored policy.[^14] Paying that cost
four times is three payments too many.
## 8 The failure register

**This section is precedent.** Each entry states what the project believed, what
is true, how it was caught, and the rule that prevents it. Most entries come
from one session, which is itself a finding: **these shapes arrive in clusters,
because one wrong belief supports several conclusions.**

**Each entry names its source.** An entry marked "register" is in the findings
register with its evidence. An entry marked "tree" the author verified against
the code, a log, or a commit. An entry marked "owner" comes from the project
owner and the register does not hold it.

### 8.1 The objective

**1. The shaped term became the objective. Source: tree.**

*Believed.* The reward pays the outcome at plus or minus 2000 and held ground at
1. The outcome is the largest term, so the outcome decides the ranking, and held
ground guides the search toward it.

*True.* A generation ranks its own candidates, and wins are too rare for the
outcome to order a population. Over 136 generations held ground decided 87.6
percent of all pairwise comparisons.

*Caught by* computing the win share of each generation from the run log and
converting it to a pair share.

*Rule.* **Do not ask what a term contributes. Ask what share of the pairs it
decides.** Compute that share from the win share of each generation, before the
run and again after it.

**2. A nearly ternary score gave a whole run nothing to rank. Source: register,
FND-637.**

*Believed.* The trainer reports the spread of a generation, and a spread above
zero means a population the ranking can rank.

*True.* Every spread of a 16 generation run was a multiple of about 673, which is
the outcome term divided by the seed count. A candidate's score was its win count
over six seeds, so it took seven values. The centre never moved.

*Caught by* reading the spreads of the run and noticing they were all multiples
of one number.

*Rule.* **The two failures bracket the design problem.** Entry 1 is shaping too
strong and entry 2 is shaping too weak. Neither is a spread problem. **Both are
answered by a term that never ties and cannot cover the outcome gap.**

**3. The bound on a shaped term was stated wrongly twice. Source: tree.**

*Believed.* A shaped term is safe while it stays below the win weight, which is
2000.

*True.* A win and a loss stand **two** win weights apart, which is 4000. Both the
companion report and the main session took the gap for the win weight.

*Caught by* writing the derivation down and by three tests, one of which proves
the bound can fail.[^22]

*Rule.* **The gap is twice the win weight.** The safe condition is that the sum
of weight times episode ceiling over every shaped term stays below it. **A
comment cannot hold this, because nothing fails when someone raises a weight.**

**4. The bound test covered one weighting of seven. Source: tree.**

*Believed.* The bound is checked.

*True.* The test was parametrised over one strategy name.[^22] Six weightings
were unchecked, and three of those weigh a term whose episode ceiling the test
does not state. **Derived:** the population weighting covers the gap at a net
gain of 1334 people. Measured population is a single digit, so nothing has met
it.

*Caught by* the author reading the parameter list of the test. **A second
session was removing that test while this report was written**, so a reader must
check whether any test holds the bound at all.

*Rule.* **A test parametrised over one name is a test of one name.** Derive the
list from the thing being checked, not from a literal.

### 8.2 The optimiser

**5. A zero-spread generation took a full-size random step. Source: register,
FND-666.**

*Believed.* A generation whose candidates all score the same ranks a set of ties,
so the update carries nothing and costs nothing.

*True.* The rank comes from a stable sort, so an equal set ranks by candidate
index. Every plus half then ranked below its own minus half by the same amount.
The trainer took a step of the full learning rate along a direction the noise
alone chose.

*Caught by* a sweep of 300 seeds that played nine policies on each and found 11
seeds giving nine equal returns.

*Rule.* **A stable sort over equal values is not a tie. It is an ordering by
index.** Guard the case, and name the generation in the log when it fires.

**6. The gradient length was believed to measure the quality of a step. Source:
register, FND-668.**

*Believed.* A generation whose candidates spread wide carries more information.
So the length of the summed gradient measures step quality, and it should scale
the step.

*True.* Perturbations in a space of hundreds of dimensions are near orthogonal.
So the squared length of the weighted sum is the sum of the squared weights,
whatever the ranking is. The ratio is one within a few parts in a hundred for pure
noise, for perfect signal, and for a population that all scored the same. **No
threshold built on the length can work.**

*Caught by* a sweep against a known direction at seven population sizes.

*Rule.* **Before you build a threshold on a quantity, measure that quantity under
a known signal and under known noise.** If the two agree, the quantity is not an
instrument.

**7. The network was believed to be the larger model. Source: register,
FND-668.**

*Believed.* The network adds a hidden layer, so it is the larger search shape and
needs more samples.

*True.* Its first layer is a fixed random projection the trainer never touches.
It trains 696 weights against the linear policy's 5365. **It is 7.7 times
smaller**, and the alignment law accounts for the whole difference between the two
runs.

*Caught by* reading which weights the flattening method returns.

*Rule.* **Count the trainable weights, not the layers.** State the count in the
run report.

**8. Augmented random search direction selection is worse here. Source:
register, FND-668.**

*Believed.* Keeping only the best directions of a generation improves the step,
as the published method reports.[^37]

*True.* It is worse at every noise level measured on this problem. Discarding
directions in a space this large loses more than the selection gains.

*Caught by* a sweep against a known direction at six trainable counts.

*Rule.* **A published improvement is a hypothesis about your problem.** Test it
against your own dimension before adopting it.

**9. The parameter budget is set by the population. Source: register, FND-668.**

*True.* The alignment of a step is about the square root of the pair count
divided by the trainable count. Doubling the alignment needs four times the
population and costs four times as much.

*Rule.* **Choose the model size from the population you can afford, not from the
representation you want.** If you want the representation, change the method.

### 8.3 The worlds

**10. The seed filter checked that a world built. Source: register, FND-666.**

*Believed.* The filter gives the run worlds every faction can sit in. Its name
and its docstring said so.

*True.* The seeding raises only when it seats nobody. A world that seats one
faction of three is accepted and ends on the first tick. A seat that reaches no
food is accepted and starves.

*Rule.* **A filter that keeps what did not raise is not a filter.** Read the
report the thing already produced.

**11. A world can pass the filter and still be unwinnable. Source: tree, through
the companion report.**

*True.* All 256 candidates of one generation scored exactly -2000 on seed 1007.
The built-in controller also loses that world from seat 0, after 207 decisions.
Two neighbouring seeds behave differently. **The filter is not the answer to a
hostile world.**[^1]

*Rule.* **Do not filter the pool on whether the controller wins.** The holdout is
unfiltered, so a filtered training pool optimises a different distribution.
Refuse the step instead of the world.

**12. Water fraction does not predict outcome. Source: register, FND-667.**

*Believed.* A wet world destroys a faction at once, so a filter against water
recovers the lost generations.

*True.* The correlation is near zero, and its sign is the opposite of the belief.
**The wettest band is the band the controller wins most often, and its episodes
run the longest.**

*Caught by* a sweep that measured the water share of every world of the raw range
and tied it to the recorded return and win share.

*Rule.* **A spot check at one extent does not describe another.** The same
generator gives a median of about a third water at every extent. The training
extent carries a fat tail, and the demonstration extent does not.

**13. A per-generation win share is not evidence about a policy. Source:
register, FND-667.**

*True.* A generation plays one world, and the world chooses that number. Two
neighbouring generations recorded win shares differing by more than the whole
distance between chance and certainty, on a centre that moved one step.

*Rule.* **Read the fixed-seed score, at the episode count the register states.**
The comparison inside a generation is sound; only the reported number is noise.

### 8.4 The measurements

**14. The yardstick measures the faction count, not skill. Source: register,
FND-645.**

*Believed.* The built-in controller in the learner's seat is the standard to
reach, and the number it scores measures the quality of the opponent.

*True.* The yardstick world gives every seat to the same controller, so one seat
of a symmetric three-faction game wins **exactly one third**. Measuring it wastes
episodes. The measured 0.367 over 256 episodes is one third plus 0.55 standard
errors.

*Rule.* **Derive a baseline before you measure it.** If symmetry fixes it, the
measurement buys nothing.

**15. Conclusions were drawn from the order of rows without the error beside
them. Source: register, FND-645.**

*True.* Eight worlds quantise a win share into eighths. Twenty-four worlds
separate 19 points. **This finding made that error twice in one day, and it says
so.**

*Rule.* **Size the measurement to the claim.** A win share near one third has a
standard error of the square root of two ninths over the episode count.

**16. Scores were compared across different seed pools and different
weightings. Source: register and tree.**

*True.* Three distinct versions of this error occurred. A win share was compared
against a mean return. Two arms that drew different seed pools were compared at
matched generations. A return under a tile weight of 1 was compared against one
under a tile weight of 0.1.

*Rule.* **The win share is the one quantity comparable across every weighting.**
Report a return only beside the weighting that produced it.[^16]

**17. A difference between two arms was attributed to reward density. Source:
register, FND-650, and the companion report.**

*Believed.* A dense score keeps rising where a nearly ternary one goes flat, so
density is what carried the difference.

*True.* Not established. The two arms sorted their populations identically in
every generation where nobody won, which was a third of them. A competing
explanation is already measured: the two arms differed in policy shape, and the
alignment law accounts for the whole difference.[^25]

*Rule.* **Two arms that differ in two things measure neither.** And **log the
score vector of each generation**, or no reward experiment in this project can be
read.

**18. The learner acts about ten times less often than the controller it is
measured against. Source: register, FND-634.**

*True.* A recording gave 32.9 controller commands for one decision window of 10
ticks. A comparison against the controller therefore compares a rate before it
compares a policy.

*Rule.* **State the cadence of both sides of every comparison.**

**19. Per-seed outcomes were not stored. Source: tree.**

*True.* The population runner returns one reading for each candidate and each
seed. The trainer keeps the win share of the generation and discards the rest.
So a comparison that is naturally paired by seed could only be tested unpaired,
and it lost most of its power.

*Caught by* reading what the trainer keeps against what the runner returns.

*Rule.* **Store the finest thing you measured.** Aggregation is cheap later and
impossible earlier.

**20. Readings arrive flat and were read as one per policy. Source: tree.**

*True.* The runner returns the returns shaped by candidate and seed, and the
readings **flat**, at index candidate times seed count plus seed. Reading the
readings as one per candidate produced four win shares of zero and crashed after
fifteen minutes. **Without a strict length check it would have produced a
plausible and entirely wrong table.**

*Rule.* **When one function returns two arrays of different shapes, check the
shape at the call site.** A wrong index that stays in range is the worst kind.

**21. Episodes were spent measuring a quantity that symmetry fixes. Source:
register, FND-645.**

*Rule.* Same as entry 14. It is listed twice because the cost was paid twice.

### 8.5 The harness

**22. Two cells died of native faults and nothing read the status file. Source:
tree.**

*True.* Six run directories hold a status file, and between them they record
seven faults. Five exited 135 and two exited 139. **Exit 139 is a segmentation
signal and exit 135 is a bus signal, so they are two different faults.** The
register names only the first.[^38] The status files recorded each one
correctly.

*Caught by* the author reading the status files.

*Rule.* **A status file nobody reads is a status file nobody has.** Fail loudly,
or read it in the same script that wrote it.

**23. A live process can stop advancing and nothing reports it. Source:
register, FND-651.**

*True.* One instance ran two strategies for about seven hours. One reached
generation 133 and the other wrote its last line inside generation 4. Both were
in the process table at the end. **The instance spent about seven hours at half
its capacity and nothing said so.**

*Rule.* **A heartbeat that stops carries no signal.** Watch for the absence of
the next one, not for the presence of the last.

**24. Every launch spent about seventeen minutes compiling the engine twice.
Source: tree.**

*True.* The build backend of the package is the Rust builder. So the dependency
sync built the extension while it installed the project. The release build then
built the same sources again, at a different profile. **A run that changed only
a document compiled the same extension.**

*Caught by* comparing the boot time of the instance against the time the trainer
reached generation 0.

*Rule.* **Key a build artefact on the inputs that decide its bytes, and check the
key against a commit that changed nothing else.** The fix is in the tree and it
is unverified on a real launch.

**25. A pattern kill matched more than it was aimed at. Source: owner.**

*True.* A process kill by pattern matched the shell that ran it and killed its own
command mid-sequence. Separately it matched two launchers and destroyed a paid
training run.

*Rule.* **List the matches before you kill them.** The register does not hold this
entry.

**26. A hard kill on a launcher strands billing resources. Source: tree.**

*True.* The launcher traps three signals and terminates the instance and deletes
the security group in the trap.[^39] A kill that cannot be trapped skips
all of it, and the instance keeps billing.

*Rule.* **Stop a launcher through its own stop path.** A trap is not a guarantee.

**27. Editing a running shell script can corrupt its read offset. Source:
owner.**

*True.* The shell reads a script incrementally, so an edit moves the bytes under
the interpreter. This is a property of the shell and not of this project. The
register does not hold this entry.

*Rule.* **Copy a long-running script before you edit it.**

**28. Cloud spend estimated from instance-hours was four times too high.
Source: owner.**

*True.* The estimate nearly stopped a run that had budget left.

*Rule.* **Query the billing interface. Never derive spend from instance-hours.**
The register does not hold this entry.

**29. The decisive measurement was ordered last in a staged run. Source:
owner.**

*True.* A two-hour staged run put the decisive measurement at the end. An
interruption would therefore have kept every result that did not matter.

*Rule.* **Order a staged run by what each stage decides, and put the decisive one
first.** The register does not hold this entry.

### 8.6 The observation and the records

**30. A supervised fit of controller play barely beat one constant answer.
Source: register, FND-643.**

*True.* The finding names the observation array as the stronger suspect, and it
records that the measurement is confounded by the window reduction.

*Rule.* **A confounded measurement names a suspect, and it does not convict
one.** The companion report gives an unconfounded probe: predict the outcome from
the array at a fixed decision.[^1]

**31. A code comment states the opposite of what the engine does. Source: tree.**

*True.* The comment on the wonder upgrade category says that its completion ends
no game. It cites a record that gives the path no reader.[^7] That
record is superseded, the engine holds a wonder reader, and the reader fires when
a victory claim stands on held ground.[^6]

*Caught by* the author reading the reader and the comment together.

*Rule.* **When you supersede a record, search the tree for the number and repair
every place that cites it.** This is the fifth recurring defect shape of this
project.[^21]

### 8.7 The checklist to run before a training run starts

Run down these ten lines. Each one is an entry above.

1. **What share of the pairs will the outcome decide?** Estimate it from the win
   share of the last comparable run. If it is under a half, the shaped term is
   the objective.
2. **Does the sum of weight times episode ceiling stay under twice the win
   weight?** State the ceiling of every term. A term with no stated ceiling is
   unchecked.
3. **Does any shaped term ever tie across a whole generation?** If it can, the
   ranking will fall to something else in those generations.
4. **What is the trainable weight count?** Not the layer count. Compute the
   alignment from the pair count and check it is above about 0.3.
5. **Is the run comparable to the run it will be compared against?** Same seed
   pool, same weighting, same extent, same observation version.
6. **Is the baseline derived or measured?** If symmetry fixes it, do not pay for
   it.
7. **Is the episode count large enough for the difference you expect?** A win
   share near one third needs about 570 episodes to detect 5 points.
8. **Will the run log the score vector of every generation, and the outcome of
   every seed?** If not, it cannot be read afterwards.
9. **Does something read the status file and expect the next heartbeat?** A
   silent stall costs the whole run at half load.
10. **Is the decisive measurement first?** Order the stages by what each one
    decides.
## 9 What to do next

**Replace the reward before any further tuning. That is the answer to the
project owner's question, and it is not close.**

Three reasons. The reward decided 87.6 percent of the pairs of the last run, so
it is the objective and not a hint. Every setting the companion report tunes is
tuned against whatever objective the ranking expresses, so tuning first tunes the
wrong thing. And the replacement is a weighting, so it costs nothing to make.

The list below is consistent with the companion report's plan and it reorders
it.[^1] A step marked new is not in that plan.

**Step 1. Change four settings and one weighting. Cost: none.**

Set the weighting to the outcome plus survival at 0.5, and drop held ground.
**New: dropping held ground is a change to that plan, which keeps it at 0.1.**
Set the population to 64 and the seeds to 2. Set the hidden width to 8. Log the
score vector of each generation and the outcome of each seed.

**Step 2. Run the best constant-preference policy. Cost: 2.9 cell-hours.**

Play the 29 policies that always prefer one verb over the holdout, and report the
win share of each. This is the cheapest decisive experiment available. If the
best fixed preference matches the best trained policy, then search has bought
nothing and the interface is the binding constraint.

**Step 3. Measure the proxy quality of every scoreable field. Cost: about 1
cell-hour. New.**

Run the corrected script at a size that can answer: about 64 candidates over 32
worlds, which is 2048 episodes.[^20] Report the total column and the rate
column together. **This tells you which field to weigh, and it is the
measurement this report could not take.**

**Step 4. Run the recommended setting long. Cost: about 14 cell-hours.**

One arm at the settings of step 1, for 400 generations, validated on 512 fixed
seeds every 20 generations. Run it beside step 2 on one box.

**Step 5. Trim the array and train both layers. Cost: about 11 cell-hours for
each of three arms. New in its ordering.**

Run three arms at the setting step 4 chose. The first is the present fixed
projection at hidden 24. The second is a fully trained network at hidden 8 over
the whole array. The third is the same network over the trimmed array. **This
is the starter tier of section 6, and it needs no engine change.** The
companion report places this fourth; the author places it here because it is
the cheapest change that raises what the policy can represent.

**Step 6. Test the seed count at equal budget. Cost: about 22 cell-hours.**

Two arms of 200 generations at 256 episodes each: 128 pairs on one seed, and 64
pairs on two. This is what makes the companion report's simulation falsifiable,
so run it even if step 4 succeeds.

**Step 7. Add the missing single-position fields, in one change. New.**

The renown target, the wonder threshold, the count of finished upgrades in own
ground, a settled-contract count, a goods-moved total, and a relation summary.
Each is one position. Together they make trade, diplomacy, and aggression
scoreable, and they let the policy see the quantity that decides its own
territory. **Write a decision record, because this moves the observation version
and retires every stored policy.**[^14]

**Step 8. Raise the spatial resolution and add a spatial argument, together.**

Section 7.2 says why neither alone buys much. Do this with step 7 if the two can
share one version bump.

**Step 9. Build the policy gradient learner.**

Section 6.3 states the case and the four missing parts. Keep the evolution
strategy as the control, and make the new method beat it on the acceptance
protocol of the companion report.

**Step 10. Test the decision interval.**

Measure the boundary cost of an interval of 2 on one generation before committing
a run. Choose the interval and the method together.

### 9.1 The three things to do first

**Replace the reward.** It is free, and it is the finding of this report.

**Measure the proxy quality of every scoreable field.** One cell-hour buys the
answer to "which field should the reward weigh", and the project has been
guessing.

**Run the constant-preference baseline.** Until it runs, nobody knows whether
this project has a search problem or an interface problem.

## References

[^1]: Report 40, what a well-trained policy needs. `docs/research/reports/40-what-a-well-trained-policy-needs.md`
[^2]: The log of the ground-and-network run. `runs/graviton/dense-depth/land-net.log`
[^3]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^4]: Balance register, the holding rows and the renown rows. `docs/reference/balance.md`
[^5]: ADR-0148, a game end is recorded once and stops the controllers, decisions D1 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^6]: ADR-0174, a wonder is a win path and a stock total is not. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^7]: ADR-0173, the wealth or wonder path has no reader. `docs/adrs/draft/adr-0173-the-wealth-or-wonder-path-has-no-reader.md`
[^8]: The upgrade table and its work constants. `crates/cachette-core/src/upgrade.rs`
[^9]: The contest pass and the renown it pays for one felled unit. `crates/cachette-core/src/contest.rs`
[^10]: The reward of a faction, and the rule that a term names a single-position field. `python/cachette/learn/reward.py`
[^11]: The legality reader and the win readers. `crates/cachette-core/src/world.rs`
[^12]: The unit type table and its rows. `crates/cachette-core/src/unit_type.rs`
[^13]: The stored behaviour report of six policies. `runs/learn/behaviour.json`
[^14]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D1 and D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^15]: The seated league runner. `python/cachette/learn/league.py`
[^16]: The stored policy index. `checkpoints/README.md`
[^17]: Vinyals and others, Grandmaster level in StarCraft II using multi-agent reinforcement learning, Nature 575, 2019. https://www.nature.com/articles/s41586-019-1724-z
[^18]: The trainer, its ranking and its step. `python/cachette/learn/train.py`
[^19]: Amodei and others, Concrete Problems in AI Safety, 2016, section on reward hacking. https://arxiv.org/abs/1606.06565
[^20]: The proxy quality script. `scripts/proxy_quality.py`
[^21]: Recurring Defect Shapes, shapes 1 and 5. `.agents/rules/recurring-defects.md`
[^22]: The commit `Pay a seat for how long it denied a win, and bound the weight by a test`. Read its message for the derivation and the two corrections.
[^23]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^24]: Findings register, FND-583. `docs/FINDINGS.md`
[^25]: Findings register, FND-668. `docs/FINDINGS.md`
[^26]: The policy module and its fixed projection. `python/cachette/learn/policy.py`
[^27]: The throughput reading of the target platform. `runs/graviton/dense-depth/throughput.txt`
[^28]: Report 34, what would move the learner. `docs/research/reports/34-what-would-move-the-learner.md`
[^29]: Report 38, where the training time goes. `docs/research/reports/38-where-the-training-time-goes.md`
[^30]: Schulman and others, Proximal Policy Optimization Algorithms, 2017. https://arxiv.org/abs/1707.06347
[^31]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^32]: The observation array of a faction and its schema. `crates/cachette-core/src/faction_observation.rs`
[^33]: ADR-0193, a faction's observation names another faction by a position relative to the reader, decision D1. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
[^34]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decisions D1 and D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^35]: Sun and others, TStarBots: Defeating the Cheating Level Builtin AI in StarCraft II in the Full Game, 2018. https://arxiv.org/abs/1809.07193
[^36]: Findings register, FND-582. `docs/FINDINGS.md`
[^37]: Mania, Guy and Recht, Simple random search provides a competitive approach to reinforcement learning, 2018. https://arxiv.org/abs/1803.07055
[^38]: Findings register, FND-646. `docs/FINDINGS.md`
[^39]: The training launcher and its teardown trap. `scripts/graviton-train.sh`
