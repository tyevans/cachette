# What is wrong with training and evaluation

This report audits the reinforcement learning training path and the evaluation
path of this project. It lists what is wrong with each, ranked by what the
defect costs, and it gives the evidence for every item.

A paid training run produced four policies. The project owner played them and
reported that each one takes a single unit and wanders for a thousand ticks.
Every number the run produced said the policies were strong. The run reported
a return 1.74 times the built-in controller's on the same objective. This
report answers one question: how did the training loop and the evaluation both
report success while producing policies that cannot play?

Two causes are already recorded and this report does not rediscover them. The
first says that the four policies are fixed preference orders over the 180
action rows, and that the engine's legality answer supplies what looks like
situational play.[^F707] The second says the cause sits at the feature layer,
because 3,546 of 4,819 encoded positions never change, so a weight over such a
position can only add a fixed offset to one action row.[^F708] This report
checks whether each cause is fully accounted for, and it finds that neither
one explains the reported success. **A separate set of defects made the run
report success, and those defects survive the feature repair now in
flight.**

## 0. What this report measured, and what it did not reach

**Every figure below measures one moment.** The moment is the tip of the main
branch on 9 September 2026, at the commit that named the unit of a published
return.[^GIT] A count belongs in a research report, because a report is fixed
to a moment. The same count decays in a decision record.[^SCOPE]

The audit read the source and it derived arithmetic from figures the project
already holds. Six workers took measurements in parallel. **The machine was
oversubscribed while they ran**: the load average was between 24 and 31 on 16
cores, because six audit workers and other agents shared it. Every item that
needed a fresh engine measurement is therefore marked with what it rests on.
An item marked UNVERIFIED names the measurement that is missing.

The whole audit took about one hour of wall clock. No gate ran. No test suite,
no determinism test and no golden state hash ran, because an audit does not
own them.

## 1. Nothing in the selection path measures play, and the one quantity that does is computed and thrown away

**What is wrong.** The trainer keeps the centre that scored highest on the
validation seeds, and the score is the mean shaped return under the same
weighting the search maximises. The validator plays the seeds and reduces the
result to one mean.[^TRAIN] The reduction happens at line 320, where the
validator returns the mean of the population record and nothing else. **The
win share of that same pass is already computed.** The population record
carries it as a property at line 307, which counts the episodes whose outcome
was a win and divides by the episode count.[^RECORD] The validator never reads
that property. So the quantity that measures play is available at the moment
of selection, in the same object, and the selection discards it.

The generation record does keep the win share, and the trainer prints it on
every generation line. The dashboard also prints a win verdict, but only from
a finished strategy. At line 543 the progress reader compares the trained win
share against the controller's and prints whether the policy beats it.[^PROG]
That row comes from the held-out pass, which runs only after a strategy ends.
When it is absent the dashboard prints, at line 550, that the held-out figure
is not measured until the strategy ends, and it falls back to the validation
figure. **The paid run was cut by the wall clock cap before any strategy
ended, so that verdict never rendered once.** For the whole run the dashboard
showed a shaped return and no statement about winning.

**No gate requires a rating before a policy is published.** The project had
already learned that a yardstick score and a rating disagree. A rating run
measured the eight earlier policies against each other and against the
controller over 252 games, and seven of the eight rated below the controller
they had each beaten on their own bar. The policy that stood furthest above
its own bar rated seventh of nine.[^INDEX] That rating is dated one day before
the run under audit. The four policies of this run were never rated head to
head at all. No rating result for them exists anywhere in the tree.

**What it costs.** This is the defect that produced the whole episode. Every
other item below explains why the search found a constant. This item explains
why nobody noticed. The project spent a paid instance, published four files
and wrote an index entry, and the first contradicting evidence came from a
person watching the screen.

**Confidence.** High. Each claim above is a line of source.

## 2. The published number is the selection score over the seeds that did the selecting, and the honest holdout never ran

**What is wrong.** The trainer draws three seed sets, and they are disjoint by
construction. The training pool starts at seed 1000, the validation set at
20,000 and the held-out set at 50,000.[^MAIN] The validation set chooses the
centre. The held-out set is the only set that never influenced the choice.

The four published manifests do not carry a held-out figure. Each one carries
a validation score and a validation seed count of 128.[^MANIFEST] The stored
policy index describes that figure as a mean shaped return over 128 held-out
seeds.[^INDEX] **It is not a held-out figure. It is the maximum over the
validation passes of the run, taken on the very seeds that selected the
centre.** The trainer writes that number into the weight file at line 195, as
the validation score, and the validator raises its stored best at line 334
whenever a pass scores higher.[^TRAIN]

The honest measurement exists in the code. The report writer plays the stored
centre, the untrained policy, a random policy and the controller over the
256-seed held-out set, and it attaches the result to the run report at line
996.[^MAIN] That pass runs after the training loop returns. **The run under
audit was cut by the wall clock cap before any strategy returned, so the
held-out pass never ran for any of the four styles.**

The bias is not small. The run took a validation pass every two generations
over twenty generations, which is ten passes, and it published the maximum of
them. A maximum over ten noisy draws is optimistic by a known amount, and
nothing in the trainer estimates it or reports it. One of the four files was
published although its own manifest records that it did not reach its
yardstick: it scored 48.84 against a bar of 49.03, and the manifest states the
comparison as false.[^MANIFEST]

**What it costs.** Every published figure of the run is a selection maximum
presented as an unbiased measurement, and the label in the index says the
opposite of what the number is. A reader who compares one of those figures
against a figure from a later run compares two different quantities.

**Confidence.** High for the mechanism and for the labelling. The size of the
optimistic bias is UNVERIFIED, because it needs the spread of the validation
passes of that run and the run logs are not in the tree.

## 3. The search ran below the alignment the project had already measured, and nothing computed the figure

**What is wrong.** One register already holds the law that governs a step of
this search. It says that the cosine between the step the trainer takes and
the direction it is trying to find is near the square root of the pair count
divided by the trainable count, and that the noise of scoring on one world
halves it.[^F668] **Nothing in the trainer, the search, the run configuration
or the launcher computes that figure, and nothing warns when it is small.**

The run used a population of 24 and it trained a structured policy of 5,354
weights.[^LAUNCH] [^INDEX] The derivation of section 12 gives an alignment of
0.047 for one generation, and 0.024 after the scoring noise the register
measured. So about 98 percent of every step of the paid run was noise.

The other policy shape is worse. A linear policy trains one weight for each
observation position and each action row, which is 867,600 weights at this
layout, and the same population gives an alignment of 0.0037.

**What it costs.** The register gave the law, and the law says the population
would have to rise sixteenfold to double the alignment. The run was configured
without applying it. This is the cheapest arithmetic in the whole path and it
was never done.

**Confidence.** High. The law is a recorded measurement, the population and
the weight count are recorded values, and the arithmetic is in section 12.

## 4. The step is a fixed length whatever the direction is worth, so the centre wanders further than it climbs

**What is wrong.** The search normalises the summed gradient to unit length
and then moves the centre a fixed fraction of its own length along it. Line
388 takes the unit vector of the gradient. Line 390 moves a unit-length centre
by the learning rate. Line 391 moves any other centre by the learning rate
times the length of the centre.[^SEARCH]

This is correct given that rank shaping has already discarded the scale of the
reward, and the docstring says so. **The consequence is that a generation that
points 2 percent of the way toward the truth moves the centre exactly as far
as a generation that points perfectly.** At a learning rate of 0.3 each
generation turns the centre by 16.70 degrees. Section 12 derives the rest: over
twenty generations the directed part of that travel is 15.8 degrees and the
undirected part is 74.7 degrees. **The centre wanders 4.7 times further than
it climbs.**

The validator then keeps the best of ten samples of that walk. So the stored
best centre of the run is close to the luckiest point of a random walk, judged
by a quantity that does not measure play. That is a complete account of a flat
validated curve with a rising best, and it needs no appeal to a local
optimum.

**What it costs.** The run bought twenty generations of a walk and selected a
point on it. Raising the population is the only lever that changes this, and
the register says the cost of that lever is quadratic in the gain.

**Confidence.** High for the arithmetic and for the source lines. The
translation of a step into a rotation of the centre assumes that a drawn
direction is near orthogonal to the centre, which the register established for
a space of this dimension.[^F668]

## 5. Sigma is not a local perturbation, and only the small side is guarded

**What is wrong.** The search draws each perturbation as a unit direction and
scales it. For a policy kind whose choice survives a positive scaling, the
scale is sigma itself, because such a centre is held at unit length. For every
other kind the scale is sigma times the length of the centre. Both branches
are at lines 182 and 183.[^SEARCH]

The run used a sigma of 1.5.[^LAUNCH] **So one perturbation is one and a half
centre lengths.** Section 12 derives the geometry: a candidate sits 56.3
degrees away from the centre, and the two halves of one antithetic pair are
112.6 degrees apart. A finite difference over an arc of 113 degrees does not
estimate a local direction. The generation is closer to a random search over
policies of a similar norm than to a gradient estimate.

The choice of 1.5 is documented and the reasoning is the wrong way round. The
run configuration records that a perturbation of 0.25 changed 0.8 percent of
the choices of a trained policy and that one of 1.5 changed about a third of
them.[^MAIN] So sigma was raised until the candidates behaved differently.
That fixes the spread of the scores, which the search needs in order to rank
at all, and it destroys the locality the gradient estimate needs. **The
docstring of the scaling function warns only that a sigma too small gives
every candidate the same choices, and it names no upper bound.**[^SEARCH]

**What it costs.** The two knobs interact and the project has measured only
one direction of one of them. A small sigma gives a ranking of ties, which the
search detects. A large sigma gives a ranking of unrelated policies, which
nothing detects.

**Confidence.** High for the geometry, which follows from the source. Medium
for the claim that the estimate is thereby worthless, because the audit did
not measure the score surface at that radius.

## 6. The terminal weights put a coarser quantisation on a generation than the shaped signal it must rank

**What is wrong.** Every play style except one sets a win at 100.0, a loss at
minus 100.0 and a draw at zero.[^STYLES] The engine names a winner at the
tick limit from held ground, so an episode that runs out of ticks still ends
won or lost rather than drawn.[^VICTORY] A generation scored 24 candidates
over 6 seeds.[^LAUNCH] The score of a candidate is the mean over its seeds.

**So one seed changing from a loss to a win moves a candidate's mean by 33.3,
and one changing from a draw to a loss moves it by 16.7.** The four yardsticks
of the run were 50.00, 10.44, minus 9.05 and 49.03, and the four published
scores were 86.90, 86.35, 49.47 and 48.84.[^MANIFEST] The quantisation is
therefore of the same order as the whole quantity being compared. A ranking of
24 candidates on 6 seeds is decided mostly by which candidate drew the luckier
outcomes.

**What it costs.** This is the scoring noise that the alignment law says halves
the cosine of a step, and here it is worse than the law's own measurement,
because the terminal term is discrete and large. Together with item 3 it means
the search was ranking noise for most of every generation.

**Confidence.** High for the arithmetic, which follows from the weights and
the seed count. UNVERIFIED for the comparison against the shaped spread: the
audit did not finish measuring the typical spread of the shaped part between
candidates of one generation, and that measurement is what would turn this
item from an argument into a demonstration. The collapse threshold of the
progress reader is set at an absolute spread of 1.0, which suggests the shaped
spread is of order one to ten, but that is an inference from a threshold and
not a measurement.[^PROG]

## 7. The centre of the structured kind grows without bound, and the search says so and does nothing

**What is wrong.** The structured policy declares that its choice does not
survive a positive scaling of its weights, at line 514.[^STRUCT] The search
therefore never renormalises that centre. Every step adds a near-orthogonal
vector of 0.3 of the centre's length, so the length grows by a factor of 1.044
each generation, which is 2.37 over twenty generations. The docstring of the
step function states this consequence, says that a run of this length pays it
as a slowly rising saturation of the hyperbolic tangent layers, and states
that the search holds no bound.[^SEARCH]

**What it costs.** The policy saturates as the run proceeds, so a late
generation reads a coarser function of the observation than an early one. That
works against the very thing the run is trying to learn. The effect over
twenty generations is modest, and the cost rises with the length of a run.

**Confidence.** High. The code states the mechanism and the docstring states
the consequence.

## 8. The collapse watch reads one strategy of four, and its threshold is an absolute reward

**What is wrong.** The launcher ends a paid run when the search has stopped,
so that a collapsed run does not keep billing. The check asks the progress
reader whether the run collapsed, and the reader answers from the last
strategy in its list at line 298.[^PROG] **The four strategy processes
interleave their lines into one shared log**, so the last strategy in the list
is whichever one happened to be parsed last. Three of the four are never
tested.

The threshold is worse. A generation counts as collapsed when its spread falls
under 1.0, and the spread is a difference of two returns. The comment beside
the constant admits that the threshold is in the units of the reward and that
a caller who changes the reward scale must change it too.[^PROG] That is one
rule stored in two places with nothing that fails when they disagree, which is
the defect shape this project names first.[^DEFECTS] The eight retired
policies scored in the thousands under their weightings, where a spread under
1.0 could never fire.[^INDEX]

**What it costs.** The guard that protects the money is attached to one
arbitrary quarter of the run and to a number that only holds for one reward
scale.

**Confidence.** High. Both are single lines of source and the second carries
its own admission.

## 9. A third of the run's episodes measure rather than train, and the launcher documents them as nearly free

**What is wrong.** Per style the run played 20 generations of 24 candidates
over 6 seeds, which is 2,880 training episodes. It took a validation pass
every two generations, which is ten passes of 128 seeds, plus one 128-seed
yardstick pass, which is 1,408 episodes. **Measurement is 32.8 percent of the
episodes of a run.** Section 12 gives the arithmetic.

The launcher states that the validation passes are nearly free, because
validation plays one policy, and it compares about two minutes for 128 worlds
against about ten minutes for a generation.[^LAUNCH] **The comparison
contradicts its own episode counts.** A generation plays 24 times 6, which is
144 worlds. A validation pass plays 128. Both run through the same batch at
the same worker count, so a validation pass costs about 89 percent of a
generation and not a fifth of one.

**What it costs.** A third of the paid compute went to a quantity that item 1
shows does not measure play. Reducing the validation set or the validation
frequency would have bought more generations, and the alignment law says more
generations are worth less than more population, so the correct trade was to
spend that third on the population instead.

**Confidence.** High for the episode counts. Medium for the wall clock claim,
because the audit did not time a generation and a validation pass on the
target machine.

## 10. Each strategy receives a quarter of the machine, and the sizing reasoning assumes the whole of it

**What is wrong.** The launcher starts one trainer process for each strategy
and divides the cores between them, at line 849, where the worker count of
each process is the core count divided by the strategy count.[^LAUNCH] A run
of four styles on 64 cores therefore gives each style 16 workers. **The
reasoning that sized the validation set assumes 64.** The comment that
justifies 128 validation seeds states that 128 worlds on 64 workers takes
about two minutes.[^LAUNCH]

The consequence is visible in what the run delivered. It asked for 20
generations. The four published centres come from generations 3, 7, 7 and
9.[^MANIFEST] The index says the run was still training when the files were
taken.[^INDEX] The run reached under half of the generations it was
configured for, and the wall clock cap ended it.

**What it costs.** The run's cost model and its actual throughput disagree by
the strategy count, so the number of generations a run will finish is not
predictable from its arguments. That is what turned a 20-generation
configuration into a 9-generation result on a paid instance.

**Confidence.** High for the worker split and the generation numbers. The
attribution of the shortfall to the worker split is an inference, marked
UNVERIFIED, because the run's own timings are not in the tree.

## 11. The run-level controller figure is measured under one strategy's weighting and reported without qualification

**What is wrong.** The run report holds one entry named for the controller.
The report writer measures it under the first named strategy's weighting, and
it prints it on a line that a comment says the dashboard reads.[^MAIN] A
reader of that line sees a controller figure with no strategy beside it. Every
strategy later measures its own controller figure against its own weighting,
so the two exist side by side.

This shape has already cost the project once in the same file. The commit that
published the four policies records that the four processes write one shared
log, that the controller row there names no strategy, and that two parses of
that log paired the same figure with two different styles.[^PUBLISH]

**What it costs.** A figure that only answers for one strategy is presented as
if it answered for the run. That is one value with two declaration sites and
undocumented precedence.[^DEFECTS]

**Confidence.** High for the source. The claim that a reader did in fact take
the wrong figure is recorded in the commit and not measured here.

## 12. The derivation

This section holds the arithmetic of items 3, 4, 5, 7 and 9 in one place, so
that a reader can check it without repeating it.

The inputs are the population, the seeds and the two search constants of the
paid run, the trainable count of the policy it trained, and the law of one
step.

| Input | Value | Where it comes from |
|---|---|---|
| population | 24 candidates, so 12 antithetic pairs | the launcher default arguments[^LAUNCH] |
| seeds in a generation | 6 | the launcher default arguments[^LAUNCH] |
| generations asked for | 20 | the launcher default arguments[^LAUNCH] |
| sigma | 1.5 | the launcher default arguments[^LAUNCH] |
| learning rate | 0.3 | the launcher default arguments[^LAUNCH] |
| validation seeds, and the pass interval | 128, every 2 generations | the launcher default arguments[^LAUNCH] |
| trainable weights, structured policy | 5,354 | the stored policy index[^INDEX] |
| observation positions, action rows | 4,819 and 180 | the published manifests[^MANIFEST] |
| the alignment law | the cosine is near the square root of the pairs over the trainable count, and one-world scoring noise halves it | the findings register[^F668] |

The alignment of one generation. The square root of 12 divided by 5,354 is
0.0473. The scoring noise halves it to 0.0237. For a linear policy the
trainable count is 180 times 4,820, which is 867,600, and the square root of
12 divided by that is 0.0037.

The rotation of one step. The search moves the centre by the learning rate
times the length of the centre, along a direction of unit length. A drawn
direction in a space of this dimension is near orthogonal to the centre, so
the step turns the centre by the arc tangent of 0.3, which is 16.70 degrees.

The travel over a run. The directed part accumulates, so it is 20 times 16.70
times 0.0473, which is 15.8 degrees. The undirected part accumulates as a
random walk, so it is the square root of 20 times 16.70, which is 74.7
degrees. The ratio is 4.7.

The geometry of one perturbation. The perturbation length is sigma times the
length of the centre, so it is 1.5 lengths. The perturbation is near
orthogonal to the centre, so the candidate sits at the arc tangent of 1.5 from
it, which is 56.31 degrees. The two halves of one pair sit twice that apart,
which is 112.62 degrees.

The growth of the centre. A step of length 0.3 near orthogonal to a centre of
length 1 gives a new length of the square root of 1 plus 0.09, which is 1.0440.
Over 20 generations that is 2.37.

The share of the episodes that measure. Training plays 20 times 24 times 6,
which is 2,880 episodes. Validation plays 10 passes of 128, plus one yardstick
pass of 128, which is 1,408 episodes. The measuring share is 1,408 divided by
4,288, which is 0.328.

The quantisation of the terminal term. A win pays 100 and a loss pays minus
100, and a candidate's score is the mean over 6 seeds. One seed moving from a
loss to a win moves the mean by 200 divided by 6, which is 33.3. One seed
moving from a draw to a loss moves it by 100 divided by 6, which is 16.7.

## 13. What this audit did not reach

Six workers took the measurements that need a running engine, and the machine
was oversubscribed while they ran. The items below are the ones this report
cannot close. **Each one is a measurement and not an argument, so the next
agent can take it directly.**

| What is missing | Why it matters |
|---|---|
| The share of episodes that end in a win, a loss, a draw and a tick limit, over a decent seed count | Item 6 rests on the terminal weights firing. The engine names a winner at the tick limit, so they should fire, but the share is what sets the size of the quantisation against the shaped signal |
| The decomposition of one style's episode return into its terminal part and its shaped part | It would say whether the reported 1.74 ratio is a difference in winning or a difference in shaping, and the owner's observation says it cannot be a difference in winning |
| The typical spread of the shaped part between the candidates of one generation | It closes item 6. If the shaped spread is under 16.7, the ranking that drives every update is decided by outcome luck |
| Whether each weight of the play style table is on the scale its description claims, term by term[^STYLES] | The descriptions state importances and the file states importance divided by a measured spread, so the spreads are volatile figures at a second declaration site |
| Whether the structured towers receive any gradient while the readout sits at zero | If a tower parameter moves no action score, its share of the 5,354 weights is dead weight, and the alignment law says dead weight dilutes every step |
| The split of the 5,354 trainable weights across parameter groups | It is the input to the item above |
| The refusal share of a fixed-row policy, and whether the top row of a published file is even accepted | A policy the engine mostly refuses is close to a no-op whatever it chooses |
| The per-block table of observation positions that ever change, separating an unwritten channel from a legitimately constant one | An earlier report audited the write sites and its own preface says later passes wrote much of the dead width it measured, so the current split is unknown[^POLICY] |
| Which learner test stays green when a policy is made to ignore its observation, when the varying part of the observation is zeroed, and when the rank shaping is made constant | The testing rule says putting the defect back is the only proof that a test reaches a case, and this report can name no test that would have caught the four recorded defects[^TESTING] |

**The last row is the most important thing this report does not hold.** Two
checks are cheap and neither exists: the share of decisions on which a policy
emits its most common action, and whether the highest-scoring row over the
unmasked rows ever changes within an episode.[^F707] This audit confirms that
neither exists in the test tree. It did not reach the wider question of which
existing test should have caught each recorded defect, and that question
matters as much as the defects.

## 14. Checked, and found sound

This section exists so that the next reader does not audit these again.

**The search builds no threshold or scaling on the length of the gradient.**
One register established that the length of the summed gradient carries no
information, so nothing built on it can separate a strong generation from a
weak one.[^F668] The code honours that. The only gate is whether the spread of
the generation is greater than zero, and the spread is taken on the raw
returns, because the play sets the ranked entry of a generation to the mean
return of each candidate.[^SEARCH] [^ROLLOUT] A generation of equal scores
moves nothing, which is correct, because the rank of an equal score is the
index of the candidate and the step would then follow the noise alone.

**The rank shaping is correct.** It maps the sorted order onto the range minus
a half to a half, so it centres to zero mean over the population.[^SEARCH]

**The perturbation of a generation is reproducible and shard-safe.** The noise
is drawn from the run seed and the generation number alone, so a resumed run
draws the perturbations of the run it continues, and a worker process draws
the row that a pair has in every process.[^SEARCH]

**The three seed sets are disjoint.** The training pool starts at 1000, the
validation set at 20,000 and the held-out set at 50,000, and the counts of the
paid run are far smaller than the gaps.[^MAIN]

**The yardstick is a matched comparison.** The validator measures the
controller on the same validation seeds, under the same weighting, in the same
world with the seat given back to the engine. The environment honours that by
never marking the seat externally controlled.[^TRAIN] [^ENV] The two numbers
of a validation line are comparable. What they are not is a measure of play,
which is item 1 and not a fault of the yardstick.

**The engine does name a winner at the tick limit.** The territory reader
returns nothing before the tick limit and compares held ground after it, and
the environment enables the win readers.[^VICTORY] [^ENV] So an episode that
runs out of ticks ends won or lost. A drawn outcome needs a tie in held
ground, not a timeout. The audit tested the opposite hypothesis and refuted
it.

**The two log paths of the launcher do not disagree.** The collecting function
reads one path and the following function tests another, but the progress
render copies the log to the path the follower tests, so the collapse check
does see a file.[^LAUNCH] The audit tested this and found no defect.

**A change term no longer telescopes silently.** One register established that
a sum of first differences collapses to the endpoints under an undiscounted
episode return, so every shaped weight of the earlier run was really a
terminal reward.[^F692] The reward module now offers a level weight beside the
change weight, states the difference in its own prose, and refuses a weighting
that reads one field as both.[^REWARD]

## References

[^F707]: Findings register, FND-707. `docs/FINDINGS.md`
[^F708]: Findings register, FND-708. `docs/FINDINGS.md`
[^F668]: Findings register, FND-668. `docs/FINDINGS.md`
[^F692]: Findings register, FND-692. `docs/FINDINGS.md`
[^GIT]: The commit `Name the unit of a published return, which is not a win rate`.
[^SCOPE]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
[^DEFECTS]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^TESTING]: Testing Rules, sections 1 and 2a. `.agents/rules/testing.md`
[^TRAIN]: The trainer, the validator and the checkpoint writer. `python/cachette/learn/train.py`
[^SEARCH]: The evolution strategy, the rank shaping and the perturbation scale. `python/cachette/learn/search.py`
[^ROLLOUT]: The play that scores one generation. `python/cachette/learn/rollout.py`
[^RECORD]: The population record and the generation record. `python/cachette/learn/record.py`
[^MAIN]: The run entry point, the seed draws and the argument defaults. `python/cachette/learn/__main__.py`
[^ENV]: The environment of one episode. `python/cachette/learn/env.py`
[^REWARD]: The reward module. `python/cachette/learn/reward.py`
[^STRUCT]: The structured policy. `python/cachette/learn/structured.py`
[^STYLES]: The play style table. `python/cachette/learn/play_styles.toml`
[^PROG]: The progress reader and the dashboard. `scripts/train_progress.py`
[^LAUNCH]: The training launcher. `scripts/graviton-train.sh`
[^MANIFEST]: The manifests of the four published policies. `checkpoints/styles/`
[^INDEX]: The stored policy index. `checkpoints/README.md`
[^VICTORY]: The game end readers of the engine. `crates/cachette-core/src/world/victory.rs`
[^PUBLISH]: The commit `Publish the four play style policies, and read each file's own world`. Read its message for the four scores and the shared log hazard.
[^POLICY]: What a policy cannot see. `docs/research/what-a-policy-cannot-see.md`
