# Report 43: A first model that is not a policy

This report answers one question. What should this project learn first that is
not a direct map from one observation to one action integer?

**The short answer is an action-value outcome model.** It reads the
observation the engine already publishes. It predicts, for each row of the
action table, whether the reader's seat wins the episode. It fits on the
episodes the trainer already runs. It costs no engine change, no extra
simulated tick, and the same arithmetic per decision that the present policy
costs.

**Two numbers carry the argument.**

**The first is 188.** An evolution strategy buys one scalar for one episode. An
episode holds about 188 decisions, and the outcome of that episode labels every
one of them. One completed run of 6.7 hours played 33,280 episodes and read
33,280 scalars from about 6.26 million decisions. **The project already pays
for the surplus and discards it at the end of every generation.**

**The second is 1851.** That is the trainable weight count of the present
policy. The alignment law caps it. The cosine between the step a generation
takes and the direction it estimates is the square root of the pair count
divided by the trainable count. At 128 pairs and 1851 weights that cosine is
0.263.

**A supervised fit has a gradient, so the law does not apply to it at all.**
The first model that is not a policy is therefore also the first model this
project is allowed to make large.

Four further answers follow.

**A learned world model is about twenty-six times cheaper to run than the real
engine for the same lookahead, and that is not the reason to reject it.** The
engine is exact and it is already there. The model must predict 4819 numbers ten
ticks ahead. Its label is no denser per decision than the label that trains the
outcome model.

**A small pretrained language model fails on the trainer and on the hardware,
not on the forward pass.** Section 8 gives the arithmetic. A published latency
figure for a model of that family on a datacentre accelerator, batched, comes
in under the simulation cost of the decision it would decide. The project has
no such accelerator in its cost basis. The alignment law also forbids the
present trainer from moving a model of that size, at any population the project
can pay for.

**Imitation of the built-in controller is built, run and measured.** It was the
cheapest first step and the project took it. The fit scored within two points
of one constant answer on held-out episodes. Section 7 says why the outcome
label repairs the confound that measurement carried, and that is the strongest
single argument in this report.

**The evaluation problem dissolves rather than being solved.** Every yardstick
so far returns the chance line of one third, because the label is one game. A
per-decision label gives about 96,000 scored predictions from a 512-episode
holdout, and the score is continuous. Section 10 says when a rating pool
becomes necessary and why it is not yet.

## 0 Provenance, and what this report could not verify

The author read the code in this repository, read the registers, and read four
earlier reports. **The author ran no training, started no instance, and took no
measurement of this engine.** A paid training run held the machine.

The report holds four kinds of claim, and each one is marked.

**Read.** The author read the source or a register and states what it says.

**Measured.** The figure comes from a measurement this project already
recorded, or from a published measurement. The footnote names where it is
recorded.

**Derived.** The author computed the figure from a measured one. Every derived
figure states its arithmetic.

**Reasoning.** The author argues from the code and from published work. No
measurement supports it. Each such passage says so.

**The author could not verify five things.**

The author could not verify any cost figure on the target platform. One blocker
governs every cost figure of this project, and it governs every figure here.
The target platform register holds no learner throughput row at all.[^1]

The author could not verify that any published, peer-reviewed work
reinforcement trains a sub-billion-parameter language model to play a strategy
game. A search found none. Section 8.5 reports the one near instance that
exists and states what it is.

The author could not verify that the weights of a pretrained language model
carry any prior which transfers to an invented integer encoding of a hex world.
No published material makes that claim, and this report does not assume it.

The author could not verify the throughput of a large matrix product on this
project's hardware. The one measured arithmetic figure is a small-matrix figure
and it is a poor base for extrapolation. Section 8.3 says where the
extrapolation is assumed rather than measured.

The author could not verify the operation count of a forward pass of the
language model family by any published figure, because the vendor publishes
none. Section 8.3 derives it from the parameter count and states the
assumption.

## 1 Where the project stands

**Read.** Every learner so far is a feed-forward policy. It maps one
observation to one action integer. An evolution strategy trains it. The
strategy perturbs a centre along random directions and scores each perturbation
over a whole episode. It then ranks the scores and steps along the ranked sum by
a fixed step size.[^2]

**Read, and this report verified the width from the source.** The engine
publishes a fixed-width observation in an egocentric ring frame.[^3] The
blocks are these.

| Block | Positions |
|---|---|
| Ring stack, 151 cells at 25 channels | 3775 |
| Entity tokens, four sets | 624 |
| Every other scalar and order statistic | 319 |
| Trade board, 8 goods at 5 statistics | 40 |
| Frontier and pressure by sector | 32 |
| Layout reserve | 29 |
| Total | 4819 |

The ring stack holds 14 rings. Ring 0 holds one cell, ring 1 holds six
sectors, and each ring above holds twelve, so the stack holds 151 cells.[^4]
**No source file and no document states the total of 4819**, because the schema
sums the declared field widths at run time.

**Read.** The action table holds 29 rows over twelve verbs, at three factions.
Four verbs take one argument each: a resource kind, an upgrade category, a
faction, or a unit type.[^5] **No row of the table names a tile, a cell or a
direction.**

**Measured.** The training world is 48 tiles on a side and holds three
factions. The tick limit is 2500 and the decision interval is 10 ticks. One
episode runs 1881 ticks at the mean, so it holds about 188 decisions.[^6]

**Measured.** One completed run trained for 6.7 hours over 130 generations of
256 candidates on one cell of 32 engine workers. One generation took 183
seconds at the median. One cell scores 5036 episodes an hour.[^6]

**Measured.** No policy has beaten the chance line of one third against the
built-in controller. One seat of a symmetric three-faction game wins one third
of its games whatever the controller does, so the chance line needs no
measurement.[^7]

**Measured, and this is the ceiling every method is judged against.** The
trainer loop was split over one decision of one world on one thread. The
simulation cost 14.9808 milliseconds. The policy arithmetic cost 0.0092
milliseconds, which is 0.06 percent of the decision.[^8]

**Measured, and this report verified the arithmetic against the source.** The
project owner supplied a figure of 40,221 multiply-accumulate operations for
one structured policy decision. The figure reproduces exactly from the code at
the present layout.

| Stage | Operations |
|---|---|
| Scalar projection, 16 by 421 | 6,736 |
| Ring sector convolution, 14 by 12 by 25 by 3 | 12,600 |
| Ring channel mix, 14 by 12 by 25 by 4 | 16,800 |
| Ring band mix | 336 |
| Token embeddings, four sets | 2,496 |
| Trunk, 12 by 73 | 876 |
| Readout, 29 by 13 | 377 |
| Total | 40,221 |

**Read.** Of those weights, 6736 sit in a fixed random projection that the
trainer never touches. **The trainable count is 1851.**[^9]

**Measured.** The alignment law states that the cosine between the step and
the direction is near the square root of the pair count divided by the
trainable count. Scoring on one world roughly halves it.[^10] At 128 pairs and
1851 trainable weights the cosine is 0.263 before scoring noise.

## 2 The axis every candidate is judged on

**Derived.** The table gives what one simulated tick of the training world
yields under each method. The episode is 1881 ticks and holds 188 decisions.

| Method | What one episode labels | Labels for each tick |
|---|---|---|
| Evolution strategy | one scalar | 0.00053 |
| State-value outcome model | 188 outcomes | 0.100 |
| Action-value outcome model | 188 pairs of a row and an outcome | 0.100 |
| Imitation, per decision window | 166 label distributions | 0.088 |
| Learned world model, per decision | 188 transitions | 0.100 |
| Learned world model, per tick | 1881 transitions | 1.000 |

**Every candidate beats the evolution strategy on this axis by about 188.**
The factor is structural and belongs to no candidate. An episode return is one
number for a whole game. Every other label sits on a decision.

**Derived, and this is the primary finding.** The completed run played 33,280
episodes and read 33,280 scalars. The same episodes held about 6.26 million
decisions. **The surplus signal is already paid for and the trainer discards
it.**

**Reasoning, and this is the second finding.** The alignment law is the ceiling
on the policy family, and a supervised model escapes it. The law describes a
random search in a parameter space. A supervised fit computes a gradient
directly, so its accuracy does not fall with the parameter count. **The
trainable count of a supervised model is bounded by the data, and the data is
6.26 million labelled decisions rather than 33,280 scalars.** No candidate in
this report needs a new simulated tick to obtain its label.

## 3 Candidate A: a state-value outcome model

**What it learns.** A function from one observation to the probability that the
reader's seat wins the episode that observation came from.

**The label, and where the signal comes from.** The outcome of the episode.
The engine records the end of a game as a public fact, and the environment
already reads it.[^11] **The label needs no reduction of any kind**, and
section 7 says why that is the property that matters most.

**Signal density.** 188 labels for each episode, which is 0.100 for each
simulated tick, against 0.00053 for the evolution strategy.

**What it costs to build.** Nothing from the engine. The environment publishes
the observation, the reward and the end of the game, and it holds the
loop.[^12] The fitting machinery also exists: the imitation module holds a
general cross-entropy fit by momentum over a fixed step count, and a fit of an
outcome label reuses it unchanged.[^13] The new work is a recorder in the
control plane that keeps every observation of an episode and writes the
outcome beside them.

**Which records govern it.** The observation record binds what the fit reads.
The reward record binds the objective vector it may read beside it.[^3] [^14]
**Neither constrains what the control plane does with either.**

The fit itself needs no record, because a supervised fit in the control plane
binds nothing. **The choice of label does need one.** A future contributor
could reasonably choose a teacher or a weighted return instead. That is the
decision the reserved registry row holds.

**What it costs to run.** A value model of the present shape costs the
arithmetic of the present policy, because the two share every layer except the
readout. Call it 40 thousand operations, against a decision whose simulation
costs 14.98 milliseconds and whose present arithmetic costs 0.0092
milliseconds.[^8] **The arithmetic is free.**

**How it is evaluated.** Not by playing. Fit on one seed set and score on
another. Report the area under the receiver operating characteristic curve for
predicting the outcome. Stratify that score by the fraction of the episode
elapsed. Section 10 says why the stratification is the part that discriminates.

**What determinism costs it.** Nothing on the engine side. Section 9 states
the boundary.

**The defect.** A state value cannot choose an action on its own. It needs a
lookahead, and section 6 prices one. Candidate B removes the defect.

## 4 Candidate B: an action-value outcome model

**What it learns.** A function from one observation to 29 numbers, one for
each row of the action table. Each number predicts whether the seat wins the
episode, given that the seat takes that row now.

**The label.** The outcome of the episode, written against the row the seat
actually took. One decision labels one row, and the fit ignores the other 28
rows of that decision.

**Signal density.** 188 labelled pairs for each episode, which is 0.100 for
each simulated tick.

**Reasoning, and this is why the trainer's own data suits it.** A generation
holds 128 or 256 candidates, and each plays a different policy. One generation
therefore covers the action table more widely than any single policy would. A
fit over that mixture estimates the value of a row under the behaviour of the
population. Choosing the best row under that estimate is one step of policy
improvement. One step of improvement over a known behaviour is the smallest
thing that could beat the thing which generated the data.[^15]

**What it costs to build.** Nothing from the engine, for the same reason as
candidate A. The recorder must additionally keep the row the seat took and the
legality mask of the decision, and the environment already answers both.[^12]

**What it costs to run.** **Exactly the arithmetic of the present policy.**
The present policy already produces 29 numbers and takes the highest legal one.
The action-value model produces 29 numbers of a different meaning and takes the
highest legal one. The forward pass has the same shape, so the figure is the
same 40,221 operations, or 0.06 percent of a decision.

**How it is evaluated.** Two tests, and the project has been running only the
second.

The first is supervised and cheap: the discrimination of candidate A, plus the
calibration. Bucket the predictions and check that a bucket predicting 0.4 wins
0.4 of its episodes.

The second is the win share protocol this project already states. Play 512
holdout seeds in each of the three seats, which is 1536 episodes, and report
the share against the chance line of one third. The acceptance statement is
0.383, which stands 4.2 standard errors above the line at that episode
count.[^7]

**What determinism costs it.** Nothing on the engine side. Section 9.

## 5 Candidate C: a learned world model

**What it learns.** A function from one observation and one action row to the
observation 10 ticks later, or to a summary of it.

**The label.** The next observation. It is self-supervised, so it needs no
reward, no outcome and no controller. It is also the densest label available in
principle. The environment reads an observation after each decision today, and
reading one after each tick would give 1881 transitions for each episode rather
than 188. That change is a control-plane change and it costs nothing from the
engine.

**What it costs to build.** A predictor of 4819 numbers, and a decision about
what a summary would hold. A planner must sit above it, and the planner is the
part that has to justify itself. The engine change is small and the model is
large.

**What it costs to run, and the honest answer surprised this author.** A model
that maps 4819 inputs to 4819 outputs through a hidden layer of 256 costs about
2.47 million operations for one predicted step. The real engine costs 14.98
milliseconds for the same step.[^8] A lookahead of one step over 29 rows costs
about 72 million operations against the engine's 434 milliseconds.

**Derived, and the conversion is stated.** The measured policy path runs 40,221
operations in 0.0092 milliseconds, which is 4.4 million operations for each
millisecond.[^8] The lookahead therefore costs about 16.5 milliseconds against
the engine's 434. **On this project's own figures the learned model is about
twenty-six times cheaper than the engine for the same lookahead.** The measured
rate is a small-matrix rate, and a wider product runs better, so twenty-six is a
lower bound.

**Why it still loses.** Three reasons, and none of them is speed.

The engine is exact and the model is not. A deterministic environment removes
one source of model error. The exact environment is already available, so a
learned model buys nothing that a copy of the world would not buy better.[^16]

The error compounds with depth, and depth is the only reason to want a model at
all. A one-step lookahead is worth little beside an action-value model that
already predicts the outcome.

The label is no denser per decision than the outcome label, and it answers a
question nobody has. Predicting 4819 numbers ten ticks ahead is a harder
function than predicting one bit at the end of the game, and nobody needs the
4819 numbers.

**How it would be evaluated.** By prediction error against held-out
transitions, per channel and against the horizon. That test is honest and it
measures no play at all, which is the difficulty: a model can predict well and
plan badly.

**What determinism costs it.** The model is learner-side. The planner above it
is learner-side. Section 9 states the one condition on the depth.

## 6 Candidate D: a search against the real engine

This candidate learns nothing. It is here because candidate A needs a lookahead
and because an earlier report recommends the real engine over a learned
model.[^16]

**What it costs to run.** A one-step lookahead over 29 rows steps the world 29
times for each decision and restores it 28 times. That is 29 times the
simulation of a decision, so an episode costs 54,520 ticks instead of 1881. At
the measured 14.98 milliseconds for one decision of one world, a decision costs
434 milliseconds of simulation instead of 15.[^8] **A run that costs 6.7 hours
today would cost 8 days.**

**What it costs to build, and this report corrects an assumption it started
with.** The Python world exposes no way to copy a world or to restore one. It
exposes no copy protocol, no pickle support and no serialisation, and the only
construction path builds a fresh world from a seed.[^17] **The core world type
is clonable, however.** A snapshot is therefore a binding-level change and not
an engine-level one, and the exposed whole-world state hash gives a cheap
check that a copy is a copy.[^17]

**Reasoning.** The build cost is small and the run cost is 29 times a
decision. That ordering is the opposite of what this author expected, and it
means the barrier to searching against the real engine is simulation time
alone. **A search therefore pays only when a decision is worth 29 decisions of
simulation, and nothing measured says it is.**

## 7 Candidate E: imitation of the built-in controller

**It exists, it ran, and the register holds the result.** A module records what
the built-in controller does in the learner's seat and fits a policy to
it.[^13]

**Read.** The recorder plays the controller in the seat and reads the commands
of each tick at the frame barrier. It turns the window between two learner
decisions into one distribution over the action table. The fit runs a fixed
step count, and no test of the loss ends it, so two machines fit one dataset to
one answer. The dataset splits by episode and never by window.

**Measured.** A recording of 32 episodes gave 5320 decision windows and 174,834
controller commands. The fit trained on 24 episodes and scored on the 8 it
never saw. Both a linear policy and a small network scored within two points of
one constant answer on the held-out episodes. Both scored below it on the share
of individual commands they matched.[^18]

**Measured, and this is why imitation cannot be the next step.** The controller
emits about 33 commands in the window that the learner answers with one action.
**No reduction of that window to one label is lossless.**[^19] A fit that
failed could have failed on the reduction rather than on the observation, and
the measurement cannot separate the two. The play result carries the same
confound, because the policy acts once for each window and the controller acted
33 times.

**What follows, and it is the strongest argument in this report.** The next
step must carry a label that no reduction touches. **The outcome of an episode
is one fact about one episode.** It is not a summary of many facts, so a model
that fails to predict it has failed on the observation and on nothing else.
That is the case for candidates A and B, and it comes directly out of the
failure of candidate E.

**It also cannot exceed its teacher.** The controller wins about one third of
its games, because the yardstick is symmetric.[^7] A perfect imitation reaches
the chance line and stops. An outcome model has no such ceiling, because its
target is the outcome and not a teacher.

**What determinism costs it.** Nothing. The fixed step count already removes
the one learner-side hazard, and section 9 names it.

## 8 Candidate F: a sequence model over the game

The project owner asked whether a small pretrained language model, and the
LiquidAI family by name, could be reinforcement trained to play, possibly
behind a world encoder. This section answers on the numbers.

### 8.1 What the tokenisation would be

Two options exist and both are priced.

**A text serialisation.** Write the 4819 integers as text. At about three
tokens for each number that is about 14,500 tokens for one observation. The
family publishes a context length of 32,768, so one observation fits and a
history of decisions does not.[^20]

**A world encoder.** Project the array to a small set of embedding vectors, say
64, and prepend a short instruction. Call the prompt 128 tokens.

### 8.2 What the context would hold

The trajectory of the episode so far: the observations, the rows taken, and the
return. That is the shape a decision transformer uses, which treats
reinforcement learning as sequence modelling conditioned on a desired
return.[^21] [^22] **Memory across decisions is the one thing a sequence model
buys that a feed-forward policy does not**, and fog is the reason to want it.

### 8.3 What a forward pass costs

**Measured.** The smallest dense model of the family holds 354,483,968
parameters over 16 layers. Ten of those layers are gated short convolution
blocks and six are grouped-query attention blocks. The hidden width is 1024 and
the
vocabulary holds 65,536 entries.[^20] The tied embedding therefore holds about
67 million of those parameters, so the non-embedding count is about 287 million.

**The vendor publishes no operation count for a forward pass.** This report
derives one.

**Derived, and the assumption is stated.** A forward pass costs about two
floating point operations for each non-embedding parameter and each token,
which is one multiply-accumulate for each parameter and each token.[^23] **That
rule was derived for a dense transformer and this model is a hybrid.** Read the
figure as an estimate of the right order and not as an exact count. The
base of comparison is 40,221 operations for one structured policy decision, at
0.06 percent of a decision.[^8]

| Design | Tokens for one decision | Operations for one decision | Against the present policy |
|---|---|---|---|
| Structured policy today | — | 40,221 | 1 |
| World encoder, 128 tokens | 128 | 36,700,000,000 | 913,000 |
| Text serialisation, 14,500 tokens | 14,500 | 4,160,000,000,000 | 103,000,000 |

**Derived.** On one processor core at an assumed 25 thousand million
multiply-accumulate operations a second, the world encoder design costs about
1.5 seconds for one decision. That is about a hundred times the 14.98
milliseconds of simulation the decision commands. **The assumed throughput is
not a measurement of this project's hardware**, and the one arithmetic
measurement the project holds is a small-matrix figure that does not
extrapolate.

**Measured, and it is the inconvenient result.** The vendor publishes a median
latency for a 230-million-parameter model of the same family, on a datacentre
accelerator. It is about 50 milliseconds at concurrency one, and about 205
milliseconds at concurrency 64.[^24] Batched, that is about 3.2 milliseconds for one
response. **A decision would then cost less than the 14.98 milliseconds of
simulation it commands.** So the forward pass is not what kills this candidate,
if such an accelerator is available.

**Read.** No such accelerator is in this project's cost basis. A cell is 32
engine workers on a 64-core instance, and a cell-hour costs 0.383
dollars.[^25] This project also measured a graphics processor losing to the
processor for policy arithmetic. It lost at every batch size the measurement
tried, including one 128 times larger than a generation.[^8]

### 8.4 What the training loop would be, and why it cannot be this trainer

**The present trainer cannot move a model of that size, and the reason is
arithmetic.** The cosine between the step and the direction is the square root
of the pair count divided by the trainable count.[^10] At 128 pairs the table
gives the ceiling.

| Trainable weights | Cosine at 128 pairs |
|---|---|
| 1851, the present policy | 0.263 |
| 560, the starter tier | 0.478 |
| a few million, a low-rank adaptation | 0.008 |
| 287 million, the whole model | 0.0007 |

**An evolution strategy cannot train a language model here, at any population
this project can pay for.** Four times the population buys twice the alignment
and costs four times as much, so it buys nothing at a fixed budget.[^10]

A gradient method could. **None exists in this project**, and two earlier
reports both recommend against building one first.[^26] [^27] So the sequence
model needs a trainer this project has not built, on hardware it does not own,
for a prior section 8.5 cannot verify.

### 8.5 What is published, and what is not

**Measured, and reported because it is the one near instance.** The vendor's
own examples repository names a community project. That project reinforcement
trains a 2.6-billion-parameter model of this family to play noughts and crosses,
by group relative policy optimisation.[^28] A second example applies the
same method to a 350-million-parameter model of the family, for structured
output compliance rather than for a game.[^28]

**Read that evidence for what it is.** It shows the pipeline is buildable. It
does not show that the method reaches a 4819-position observation, 29 action
rows and 188 decisions in a game of three factions. Noughts and crosses holds
nine cells. **The first project is a teaching course and not a paper.**

**Nothing found.** This author searched for a published, peer-reviewed
reinforcement fine-tune of a sub-billion-parameter language model to play a
strategy game, and found none. Treat the absence as unverified.

**Unverified, and the report does not use it.** The weights of a language model
encode the statistics of text and code. A world encoder feeds them vectors that
no text produced. No published material claims that such weights transfer to an
invented integer encoding of a hex world.

**Reasoning.** If the prior does not transfer, what remains is a small
transformer trained from scratch. That is a reasonable thing to want and it is
not a language model. Its cost is then set by its own size, and it lands back
inside the tier ladder this project already has.[^27]

**One further constraint, read from the licence.** The open weights of this
family carry a licence whose commercial grant is conditioned on the user's
annual revenue staying below ten million United States dollars.[^29] A project
that intends to ship must read that before it builds on the weights.

### 8.6 The role a language model does have, and it is not this one

**Read.** This project already plans a language model as a player. A god is a
person or a language model, and it acts through the control plane.[^30] A
negotiation is a conversation, and the control plane holds the words while the
engine holds the terms.[^31]

**That role is prompt-driven and needs no reinforcement training, no world
encoder and no gradient.** Conflating it with the tactical policy is the error
to avoid. The god player reasons in words about a few named things. The
tactical policy chooses one of 29 rows every 10 ticks, 188 times a game, in a
population of hundreds. Those are different problems and they want different
machines.

## 9 What determinism costs each candidate, and which side of the boundary

**The boundary is sharp, and this report states it once.**

**Engine side, and binding.** Simulated and aggregated state holds no floating
point number. A solver runs a fixed iteration count and never a convergence
test. Every random draw is keyed on a counter, never on thread-local state.
Every event is plain data with declared padding. The core crate holds no Python
binding, the step releases the interpreter lock for its whole length, and no
Python runs inside a system.[^32]

**Learner side, and not bound by those rules.** A model that lives in the
control plane may use floating point arithmetic. It writes neither world state
nor the event log, and the two determinism tests cover exactly those two
things. The engine's guarantee is that one seed and one action log give one
game. A learner whose own arithmetic reassociates still produces an action log
that the engine replays exactly.[^33]

**Candidates A, B, C and E are entirely learner-side.** Their fits are float
arithmetic in Python. **No determinism rule of this project is touched.** A
stochastic policy, if one is ever wanted, is also learner-side, because the
engine still receives one action integer.

**One learner-side cost is real, and it is reproducibility rather than
determinism.** A replay buffer whose order varies gives two models from one
dataset. The remedy is the discipline the engine already uses: key the shuffle
on the tuple of run, epoch and index, so one dataset gives one set of weights.
The imitation module already applies the same idea to its stopping rule, which
is a fixed step count and never a test of the loss.[^13]

**One cost crosses back to the engine side.** Candidate D needs the binding to
copy a world or restore one. A world copy is engine state and it must reproduce
the whole-world state hash. The core type is clonable and the hash is already
exposed, so the work is small.[^17] The expense of candidate D is simulation
time, not this.

**One thing is forbidden outright.** No candidate may consult a model inside a
step. The step releases the interpreter lock for its whole length, no Python
runs inside a system, and the crate boundary makes a mid-step callback a
compile error.[^32] Every planning method must therefore run between two
decisions, from the control plane, and may send only action integers.

**An adaptive search depth is allowed, with one condition.** The fixed
iteration rule governs a solver in the engine, not a search in the control
plane. A search in Python may choose its own depth. It must key that depth on
the decision index, and never on a clock or a time budget. A depth that follows
the load of the machine gives two action logs from one seed.

## 10 How the acceptance test changes

**The chance line makes every present yardstick blind, and the reason is the
label.** A win share is one bit for one game. The standard error of a share
near one third is 0.030 at 256 episodes.[^7] A validation of 128 episodes
therefore cannot separate a policy that wins 0.36 from one that wins 0.40.

**A per-decision label removes the problem rather than working around it.** An
outcome model is scored on 188 predictions for each held-out episode, so a
holdout of 512 episodes gives about 96,000 scored predictions. The area under
the curve is continuous, so it distinguishes a model that carries a little
signal from one that carries none.

**Stratify the score by the fraction of the episode elapsed, and that is the
part which discriminates.** A model that predicts the outcome from the last
decision of a game has learned nothing useful, because the game is already
decided. A model that predicts it from the midpoint has found the deciding
quantity. **Read the curve at the midpoint and nowhere else.**

**The policy still needs the win share protocol, unchanged.** Play 512 holdout
seeds in each of the three seats, rotate the seat, and accept at 0.383.[^7]

**A rating pool is not needed yet, and this report says when it becomes
needed.** The opponent is the built-in controller, and the controller does not
adapt. A fixed opponent gives a fixed scale, so a win share against it does not
drift. A rating becomes necessary when the project trains against its own
checkpoints, because the scale then moves under the measurement. **Nothing in
the tree implements a rating today.** The multi-seat runner scores candidates
against each other inside one world and states plainly that its win share is
not comparable with a single-seat share.[^34]

## 11 The recommendation

**Build the action-value outcome model of section 4.**

It reads the observation the engine publishes today. It needs no engine change
and no new simulated tick. It costs the same arithmetic per decision as the
present policy, at 0.06 percent of a decision. It multiplies the labelled
signal of an existing run by about 188.

It escapes the alignment law, because a supervised fit has a gradient. Its own
acceptance test is continuous and does not saturate at the chance line. It lives
entirely in the control plane, so it touches no determinism rule of this
project.

**Do not build the world model.** Its run cost is defensible and its case is
not. It answers a question nobody asked, from a label no denser per decision
than the outcome, against an engine that is exact and already there.

**Do not build the sequence model.** The trainer cannot move it, the hardware
is not in the cost basis, and the prior that would justify it is unverified.
Keep a language model for the god player, where the project already places one.

**Do not repeat imitation.** It ran, and its label carries a reduction its own
measurement cannot see past.

**Hold the engine search.** It costs 29 decisions of simulation for one
decision, and its build cost is small. Revisit it when a value model exists and
something says a decision is worth 29.

### 11.1 The measurement that comes first, and what would change my mind

**The discriminating measurement is not the action-value model.** It is the
simplest possible version of candidate A.

**Run this.** Play about 512 episodes with the policies the project already
stores, on seeds no policy has trained on. Record the observations, the rows
taken and the outcome. Fit a logistic regression from the observation to the
outcome, by the fixed-step cross-entropy fit that already exists in the
tree.[^13] Report the area under the curve on held-out seeds, stratified by the
fraction of the episode elapsed.

**It costs one recording pass and one fit.** No engine change. No trainer
change. No decision record. The episodes are the same episodes an evaluation
pass already plays, and a 512-episode pass costs about six minutes on one
cell.[^6]

**What it decides.**

If the curve at the midpoint of an episode sits near 0.5, **the observation
does not carry the deciding quantity.** No model over that array earns its
keep, whatever its family. The answer is then the observation, exactly as the
imitation finding concluded, and this report's recommendation is dead.[^18]

If the curve at the midpoint sits above about 0.7, the observation carries the
outcome, and the action-value model of section 4 is worth building.

**This measurement repairs the confound of the imitation result.** That result
could not separate a failed representation from a lossy window
reduction.[^19] The outcome label carries no reduction, so a failure here has
one cause and one only.

**The second measurement, if the first passes.** Fit the action-value model on
the same data. Measure whether choosing greedily over it beats the policy that
produced the data, on the win share protocol of section 10. That is one
step of policy improvement over a known behaviour, and it is the smallest claim
worth making.

## 12 What this report found that an existing document does not say

**Four things, and the register should hold them if the project acts on this
report.**

**An earlier report recommends behaviour cloning as a cheap warm start, and a
later measurement contradicts it.**[^16] The clone was built and it scored
within two points of one constant answer on held-out episodes.[^18] The
register is the current statement and the report is not.

**The same report recommends planning with the real engine rather than with a
learned model, and it treats the engine copy as available.**[^16] The Python
world exposes no copy and no restore. The core type is clonable, so the gap is
a binding and not an engine. The recommendation still names a capability that
does not exist today.[^17]

**The observation total of 4819 positions is stated nowhere.** The schema sums
its declared field widths at run time, and two documents state older totals
that the layout has passed. That is the redundant declaration shape without a
second declaration site, which is the safe form of it. A reader of either of
those documents still gets a stale figure.

**No register entry names a value model, a world model or a critic.** The
project has never opened the question this report answers, so nothing here
contradicts a decision. That is why the reserved registry row is new rather
than a supersession.

## References

[^1]: Blockers register, BLK-007, and the target platform costs register. `docs/BLOCKERS.md`
[^2]: The evolution strategy the trainer uses. `python/cachette/learn/search.py`
[^3]: ADR-0195, the observation of a faction is a fixed-width scale-free table in an egocentric frame. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^4]: The ring frame and its cell counts. `crates/cachette-core/src/obs_ring.rs`
[^5]: The action schema and its verbs. `crates/cachette-core/src/action.rs`
[^6]: Report 40, what a well-trained policy needs, section 1. `docs/research/reports/40-what-a-well-trained-policy-needs.md`
[^7]: Findings register, FND-645, and report 40 section 9. `docs/FINDINGS.md`
[^8]: Report 38, where the training time goes, sections 2 and 3. `docs/research/reports/38-where-the-training-time-goes.md`
[^9]: The structured policy and its towers. `python/cachette/learn/structured.py`
[^10]: Findings register, FND-668. `docs/FINDINGS.md`
[^11]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^12]: The learner environment and the readers it may call. `python/cachette/learn/env.py`
[^13]: The imitation recorder and its fixed-step fit. `python/cachette/learn/imitate.py`
[^14]: ADR-0196, a reward is a bounded weighted objective vector. `docs/adrs/draft/adr-0196-a-reward-is-a-bounded-weighted-objective-vector.md`
[^15]: Sutton, Barto, "Reinforcement Learning: An Introduction", second edition, 2018, chapter 4 on policy iteration. http://incompleteideas.net/book/the-book-2nd.html
[^16]: Report 22, reinforcement learning approaches for one faction, sections 1 and 9. `docs/research/reports/22-reinforcement-learning-approaches.md`
[^17]: The Python world binding and its construction path. `crates/cachette-py/src/world/`
[^18]: Findings register, FND-643. `docs/FINDINGS.md`
[^19]: Findings register, FND-642. `docs/FINDINGS.md`
[^20]: Liquid AI Team, "LFM2 Technical Report", 2025, the model table and the layer composition. https://arxiv.org/abs/2511.23404
[^21]: Chen, Lu, Rajeswaran, Lee, Grover, Laskin, Abbeel, Srinivas, Mordatch, "Decision Transformer: Reinforcement Learning via Sequence Modeling", 2021. https://arxiv.org/abs/2106.01345
[^22]: Janner, Li, Levine, "Offline Reinforcement Learning as One Big Sequence Modeling Problem", 2021. https://arxiv.org/abs/2106.02039
[^23]: Kaplan and others, "Scaling Laws for Neural Language Models", 2020, section 2.1 and table 1, the forward pass at about two operations for each non-embedding parameter and each token. https://arxiv.org/abs/2001.08361
[^24]: Liquid AI, "LFM2.5-230M: Built to Run Anywhere", 2026, the accelerator latency figures. https://www.liquid.ai/blog/lfm2-5-230m
[^25]: Report 41, a handbook for training a policy, section 6.1. `docs/research/reports/41-a-handbook-for-training-a-policy.md`
[^26]: Report 40, what a well-trained policy needs, section 4.4. `docs/research/reports/40-what-a-well-trained-policy-needs.md`
[^27]: Report 41, a handbook for training a policy, sections 6.1 and 6.3. `docs/research/reports/41-a-handbook-for-training-a-policy.md`
[^28]: Liquid AI, the examples repository. https://github.com/Liquid4All/cookbook
[^29]: The LFM Open License, version 1.0, the commercial use threshold. https://huggingface.co/LiquidAI/LFM2-1.2B/raw/main/LICENSE
[^30]: PRD-0049, a god declares war and makes peace. `docs/product/accepted/prd-0049-a-god-declares-war-and-makes-peace.md`
[^31]: Decisions register, DEC-210. `docs/DECISIONS.md`
[^32]: Cachette project instructions, the hard invariants. `CLAUDE.md`
[^33]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^34]: The multi-seat run of a candidate population. `python/cachette/learn/league.py`
