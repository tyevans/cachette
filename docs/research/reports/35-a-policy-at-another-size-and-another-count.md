# Report 35: A policy at another size and another count

This report asks one question. A policy trained today is bound to one world
size and one faction count. What must change so that a learned policy plays a
world of another size and another count?

The report answers in four parts. It first measures what actually varies, from
the engine and the learner package. It then reports what published systems do
about size and about count. It then states what each approach costs on one
processor box with a gradient-free optimiser. It ends with a recommendation
and with a plain statement of whether this is the right problem now.

**The short answer is that this is mostly the wrong problem this week.** The
action space is already size-invariant, and the observation is not. But a
measurement in section 2 says the trainer selects no feature at all in the
observation it already has. Section 11 gives the two parts of the work that
are not premature, and says why the rest is.

## What has changed since this report

**This report is fixed to the moment section 0 names, and nothing below was
edited afterwards.** One policy kind it measures no longer exists.

**The project deleted the kind that holds a frozen random projection in its
first layer.** Section 7.5 states the trainable dimension of that kind and
prices two hidden widths against it. The trainer now builds two kinds. One is
`linear`, over the whole array. The other is `structured`, which shares a
weight across the sector axis and trains every layer.[^44] The loader refuses
a stored file that names the deleted kind, and it names the kind in the
refusal.[^45]

Read the dimension arithmetic of section 7.5 for its method. Do not read a
hidden width there as an operating point that a run can ask for.

**The observation is no longer the array this report measures.** The engine
publishes an egocentric ring frame with an entity token block, and it reached
layout version 7 on 8 September 2026. Section 1 measures the earlier array,
whose length followed the block lattice. That is the defect this report asked
about, and the frame answers it. A register holds the parameters of the layout
the engine builds now.[^46]

## 0 Method, and the moment this measures

The author read the engine, the Python binding and the learner package. The
author then built worlds of ten shapes through the binding and read both
schemas from each one. The author loaded every stored weight file and measured
where its weight sits. The author read the two training reports in the run
directory.

**Every count in this report measures one moment.** The moment is the tip of
the main branch on 7 September 2026. A count belongs in a research report,
because a report is fixed to a moment. The same count decays in a decision
record.[^1]

The author ran no test suite and no full check.

The report marks each claim. A claim marked **measured** came from a run the
author made. A claim marked **read** came from a source file or a record. A
claim marked **published** came from a paper, and the footnote gives the
source. A claim the author could not confirm is marked **unverified**.

---

## 1 What actually varies

### 1.1 The observation length follows the block lattice, and it is coarse

**Measured.** The observation length of one world is this sum.

```
length = 17 + 41 * faction_count + 11 * cell_count
cell_count = ceil(width / 32) * ceil(height / 32)
```

The engine partitions the world into blocks. The block edge exponent is five,
so a block covers 32 tiles by 32 tiles.[^2] The observation, the fog layer and
the summary level share that one lattice.[^3] Eleven fields of the thirty hold
one position for each cell, and the schema declares each one.[^4]

**Measured.** These lengths came from ten worlds built through the binding.

| Width | Height | Factions | Cells | Observation length | Action length |
|---|---|---|---|---|---|
| 16 | 16 | 3 | 1 | 151 | 29 |
| 32 | 32 | 3 | 1 | 151 | 29 |
| 48 | 48 | 3 | 4 | 184 | 29 |
| 64 | 64 | 3 | 4 | 184 | 29 |
| 96 | 96 | 3 | 9 | 239 | 29 |
| 128 | 128 | 3 | 16 | 316 | 29 |
| 256 | 256 | 3 | 64 | 844 | 29 |
| 48 | 48 | 2 | 4 | 143 | 28 |
| 48 | 48 | 5 | 4 | 266 | 31 |
| 48 | 48 | 8 | 4 | 389 | 34 |

Three facts follow, and each one matters to a proposal.

**The length does not grow with the tile count. It grows with the cell
count.** A world of 65,536 tiles gives 844 positions. A world of 2,304 tiles
gives 184. The ratio of tiles is 28 to 1, and the ratio of lengths is 4.6 to
1.

**Two worlds of different extent share one length.** Every world from 33 to 64
tiles on each axis holds four cells. A policy of the right length therefore
tells nothing about the world it was trained on. Report 33 measured this case
and a findings row holds it.[^5] [^6]

**The training world holds four cells.** One of those cells covers 1024 tiles,
which is 44 per cent of the world. Report 33 states the consequence: no
spatial skill is learnable at that resolution.[^5]

**Measured, by the same formula.** At the target scale of 16.7 million tiles
the lattice is 128 cells by 128 cells.[^7] The observation length is then
180,364 positions, and a linear policy over it holds 5,230,585 parameters. The
policy in the run directory holds 5,133.

### 1.2 What varies with the faction count

**Read.** Six things follow the faction count. The first four change the
length, and the last two change a bound only.[^4]

| Field | Positions | Indexed by |
|---|---|---|
| `relation` | `faction_count` | Absolute faction number |
| `board_good` | `faction_count * 8` | Absolute faction number, then board row |
| `board_quantity` | `faction_count * 8` | Absolute faction number, then board row |
| `board_wants` | `faction_count * 8` | Absolute faction number, then board row |
| `board_asking_good` | `faction_count * 8` | Absolute faction number, then board row |
| `board_asking_quantity` | `faction_count * 8` | Absolute faction number, then board row |

The `faction` field and the `seats_held` field carry a bound that follows the
faction count. Their length does not change.

**Read.** The two writers index by the absolute faction number and never by a
position relative to the reader.[^4] The relation writer walks the faction
numbers in order and asks for the relation from the reader to each one. The
board writer walks the faction numbers in order and copies each board.

**This is the seat-order failure mode, and this project already has it.** A
policy at seat 0 learns a weight for position 12, and that position holds its
relation to faction 0, which is itself. A policy at seat 1 reads the same
position, and there it holds a relation to a rival. The training loop rotates
the seat of each candidate with the seed, so one candidate plays every seat in
one generation.[^8] The policy must therefore fit one weight to two different
meanings.

**What happens at five factions.** The `relation` field grows from three
positions to five. Every field above it moves by two positions. A policy
trained at three factions reads `board_good` where the five-faction array
holds the last two relations. The fit check refuses the load, so nothing runs
on the wrong layout.[^9] Without that check the policy would read a shifted
array and would raise nothing.

**Not everything follows the faction count.** The eleven cell fields are
relative to the reader. The engine reports the units of the reader and the
units of everybody else as two numbers, and never one number for each
faction.[^4] A record forbids a field with one position for each faction over
the whole world.[^10] That rule is the reason the cell fields survive a change
of count and the relation field does not.

### 1.3 The action space does not vary with the world at all

**Measured.** The action length is `26 + faction_count`. It is 29 at 48 tiles
on a side and 29 at 256 tiles on a side.

**Measured.** The action table of the training world holds twelve verbs.

| Verb | Rows | Argument position | Bound | Follows |
|---|---|---|---|---|
| `no_op` | 1 | none | | |
| `gather` | 3 | `resource` | 3 | Content table |
| `build` | 7 | `category` | 7 | Content table |
| `relation` | 3 | `faction` | 3 | **Faction count** |
| `campaign` | 1 | none | | |
| `advertise` | 1 | none | | |
| `trade` | 1 | none | | |
| `carry` | 1 | none | | |
| `project` | 1 | none | | |
| `queue` | 8 | `unit_type` | 8 | Content table |
| `cross` | 1 | none | | |
| `settle` | 1 | none | | |

**No verb names a tile, a cell, a block or a unit.** The engine resolves the
target of every spatial verb at the tick the action applies. A record states
that a verb of that kind declares no argument position.[^11] Report 33 lists
the absence of a place argument as gap 14 of 14.[^5]

**The action space is therefore already invariant to the world size.** Only
one verb follows the faction count, and it names a faction by absolute number.
That is the same seat-order defect as section 1.2, in the action rather than
in the observation.

This is the most useful finding of section 1. The usual hard half of this
problem is the action space, because a policy that names a tile must name one
of 16.7 million. This project does not have that problem, because the engine
resolves the place. **The whole of the size problem sits in the observation.**

### 1.4 What the fit check enforces, and why

**Read.** A stored weight file carries seven numbers: the observation version,
the action version, the observation length, the action length, the width, the
height and the faction count. The reader refuses a file whose seven numbers
are not the seven numbers of the world it is asked to play.[^9]

The reasoning is the band argument of section 1.1. **The two lengths alone do
not separate two worlds.** A policy trained on a world 48 tiles on a side has
the length of a world 64 tiles on a side. It loads, it reads an array of the
length it expects, and it plays a world it never saw. Nothing raises, because
nothing has a shape to disagree about. The fit therefore carries the extent
beside the lengths.[^9] An accepted record states that the length is a
function of the world parameters, which is what makes the extent part of the
fit.[^12]

**Read.** The file also stores a `seat` entry, and the fit does not check it.
Every stored file names seat 0.

**A proposal that removes a field from the fit must say what replaces it.**
The fit is not friction. It is the only thing between a stored file and a
silent wrong answer. A size-invariant policy earns the right to drop the width
and the height, and it earns that right by being invariant, not by asserting
that it is.

---

## 2 The finding that changes the priority

This section reports a measurement the brief did not ask for. It changes the
order of the work, so it comes before the research.

**Measured.** The author loaded each stored weight file and summed the
absolute weight over the positions of each field group. The table compares the
share of the weight against the share of the positions. A ratio of one means
the group holds exactly the weight its size predicts.

Checkpoint of the evolution run, generation 11:

| Group | Positions | Share of positions | Share of weight | Ratio |
|---|---|---|---|---|
| Scalars | 12 | 6.5% | 6.5% | 1.00 |
| Seat-ordered fields | 123 | 66.5% | 65.8% | 0.99 |
| Weight vector | 5 | 2.7% | 2.8% | 1.02 |
| Cell fields | 44 | 23.8% | 24.3% | 1.02 |
| Bias | 1 | 0.5% | 0.7% | 1.37 |

Checkpoint of the imitation run:

| Group | Positions | Share of positions | Share of weight | Ratio |
|---|---|---|---|---|
| Scalars | 12 | 6.5% | 13.0% | 2.00 |
| Seat-ordered fields | 123 | 66.5% | 32.3% | 0.49 |
| Weight vector | 5 | 2.7% | 4.0% | 1.46 |
| Cell fields | 44 | 23.8% | 46.0% | 1.94 |
| Bias | 1 | 0.5% | 4.7% | 8.65 |

**The weights of the evolution run are flat to within two per cent.** Every
group holds the weight its position count predicts. The generation 9 file
gives the same four ratios.

**Measured, against a null.** A ratio near one is only evidence when a reader
knows the spread. The evolution centre starts at zero and accumulates
rank-weighted Gaussian perturbations, so the null is a matrix whose entries
are independent draws from one distribution. The author drew 4,000 such
matrices of this shape and measured the same five shares.

| Group | Share of positions | Null mean | Null spread | Null 95 per cent band |
|---|---|---|---|---|
| Scalars | 6.49% | 6.48% | 0.26% | 5.98% to 6.99% |
| Seat-ordered fields | 66.49% | 66.49% | 0.49% | 65.53% to 67.43% |
| Weight vector | 2.70% | 2.71% | 0.16% | 2.38% to 3.04% |
| Cell fields | 23.78% | 23.77% | 0.44% | 22.93% to 24.63% |
| Bias | 0.54% | 0.54% | 0.07% | 0.40% to 0.69% |

**Every group of every evolution checkpoint sits inside that band.** The
largest departure is minus 1.5 spreads, on the seat-ordered group. The
flattened weights have an excess kurtosis of minus 0.014, and their largest
standard score is 4.07 against the 3.9 that 5,365 independent draws give. The
bias column sits 2.2 to 2.7 spreads high in two of the three files, and inside
the band in the third. **That is not evidence.**

**The claim is therefore the strong one, and it is also the plain one. The
optimiser changed nothing this measurement can detect.** The centre is a
random walk from the origin. It was not shaped toward isotropy; it never left
it.

**Read.** The run's own untrained baseline is a matrix of zeros, which always
takes the no-op.[^43] It has no weight and therefore no profile, so it cannot
serve as this comparison. The drawn null above is the comparison it lacked.

**The imitation policy is differentiated, and it is far outside the same
band.** Its five shares sit between 7.6 and 69.8 spreads from the null mean.
Its excess kurtosis is plus 29.3 and its largest standard score is 14.0. It
puts twice the predicted weight on the scalars and on the cell fields, and
half the predicted weight on the seat-ordered fields. It puts no weight at all
on the position that holds the reader's relation to itself, which is a
constant. That is what a fitted model looks like, and it shows that the
measurement can tell the two cases apart.

**Read from the run report, not measured by the author.** The holdout of the
evolution run covers 24 held-out seeds at three factions. The author did not
re-run it. The training pool, the validation set and the holdout share no
seed.[^43]

| Player | Episodes | Win rate | 95 per cent interval | Held tiles | Live units |
|---|---|---|---|---|---|
| Trained policy | 24 | 20.8% | 4.6% to 37.1% | 136 | 3.7 |
| Random legal action | 72 | 18.1% | 9.2% to 26.9% | 293 | 6.0 |
| Untrained policy | 24 | 4.2% | 0.0% to 12.2% | 159 | 5.1 |
| Built-in controller | 24 | 37.5% | 18.1% to 56.9% | 243 | 5.6 |

Chance at three seats is 33.3 per cent.

**The sample cannot support a comparison between any two of these four rows.**
The author computed the difference of each pair against its standard error.

| Comparison | Difference | Standard error | Standard scores |
|---|---|---|---|
| Trained less random | +2.8 points | 9.4 points | +0.29 |
| Trained less untrained | +16.7 points | 9.2 points | +1.80 |
| Controller less trained | +16.7 points | 12.9 points | +1.29 |
| Controller less random | +19.4 points | 10.9 points | +1.79 |

**The trained policy is not distinguishable from a random legal action.** That
is the true statement, and it is weaker than the statement that it is barely
better. Nothing here separates the trained policy from the built-in controller
either. Only the comparison against the do-nothing baseline approaches a
signal, and it does not reach one.

These are unpaired intervals. The four players met the same 24 seeds, so a
paired test over the per-episode outcomes would be stronger. **The run report
stores only the aggregates, so the author could not run one.** A run that
stored the outcome of each episode would answer this at no extra compute.

At these rates an unpaired comparison of the trained policy against random
would need about 1,700 episodes for each arm. **The holdout is 70 times too
small to settle the question it was built to settle.**

**What follows.** The weight measurement is the load-bearing one, because it
does not depend on the sample size of the holdout. A size-invariant
architecture is a change to the function class. It helps a trainer that is
fitting a function and is limited by the class. This trainer is not fitting a
function. Widening the observation from 184 positions to 844, or replacing a
matrix with a convolution, changes nothing about a search that has selected
nothing in 184.

Section 11 returns to this.

---

## 3 What must be true for a policy to survive a change of size or count

A reviewer can check a proposal against these seven statements. Each one is
checkable by reading the proposal, and the last one is checkable only by a
measurement.

1. **The parameter count does not depend on the world extent.** A weight whose
   index is a cell index fails this. A weight shared over cells passes it.
2. **The parameter count does not depend on the faction count.** A weight whose
   index is a faction number fails this.
3. **The policy is equivariant under renaming the other factions.** Give the
   rivals new numbers, keep the reader's own seat, and the chosen action must
   carry the same renaming. A policy that reads a field indexed by absolute
   faction number fails this today.
4. **The policy is invariant to the reader's own seat number.** Two identical
   worlds that differ only in which seat the reader holds must give the same
   action, relabelled.
5. **Every quantity the policy reads is intensive, or the policy divides it by
   the count it sums over.** An extensive quantity over a cell grows with the
   cell, and the engine already declares which summary fields are extensive and
   which are intensive.[^13] The observation publishes `cell_tiles` for exactly
   this division, because an edge cell covers fewer tiles than a whole
   block.[^4]
6. **The proposal states where the spatial reduction happens, and what it
   loses.** A reduction that turns a lattice into a fixed vector throws
   something away. The proposal must name it.
7. **The evaluation runs at a size and a count that the training never saw.**
   A proposal with no cross-size and no cross-count measurement is an
   assertion. Section 4 shows that the published work also fails this often.

Statements 3 and 4 are the ones this project fails today, at three factions,
before any change of count.

---

## 4 Size invariance in the published work

### 4.1 What is measured, and what is only asserted

**Published.** A fully convolutional policy head with a globally pooled value
head is independent of the board size. The Polygames work states the mechanism
plainly. The policy head is fully convolutional, so it works independently of
the input size. Global pooling then replaces each channel by its maximum and
its average. The value head therefore fixes no size either.[^14]

**The Polygames transfer claim is one sentence with no numbers.** The paper
says a model trained at 13 by 13 was immediately strong at 19 by 19, and that
it needed some fine-tuning for the result it reports.[^14] Treat this as a
mechanism that works and a transfer result that was not measured.

**Published, and measured.** Work on Hex builds a network only from
board-size-independent parts. That network generalises from one board size to
several larger and smaller ones, without fine-tuning. Fine-tuning then learns
faster on the larger board.[^15] The same work finds that the architecture
choice, and not convolution alone, decides whether the knowledge transfers. A
later study confirms that this architecture and a graph network both transfer
between board sizes, while other convolutional architectures do not.[^16]
**Unverified:** the author could not read the board sizes and the win rates of
the first source.

**Published, and closest to this project.** Work on a real-time strategy game
adds a spatial pyramid pooling layer to a grid encoder, so that one model
reads any grid size without a structural change.[^17] Two reported findings
matter here. The agents suit larger maps better than smaller ones. Training
across a range of map sizes produces adaptability rather than specialisation.
**Unverified:** the author could not read the map sizes and the win rates.

**Published.** A large strategy-game agent runs a transformer over a list of
entities. It runs a residual network over a downsampled minimap. It joins the
two by scattering the entity embeddings into the map.[^18] The entity half is
invariant to the unit count up to a cap. **The spatial half is not
size-agnostic.** It reads a minimap of one fixed resolution.

**Do not cite the relational reinforcement learning work for size transfer.**
It runs on a grid fixed at 12 by 12, and it generalises along the number of
boxes in the solution path, not along the board size.[^19] The measured result
is real and it is about combinatorial extrapolation.

### 4.2 Which of these fits a hex pyramid that already summarises into blocks

The brief asks whether the aggregated observation is an advantage. **It is an
advantage, and not the one the brief expects. Testing the idea changes the
recommendation.**

The pyramid does not hold the cell count fixed. It holds the cell *size* fixed
at 32 tiles by 32 tiles. The cell count therefore grows with the world, from 4
cells at the training size to 16,384 cells at the target scale.

That is the wrong invariant for a fixed-length vector policy, and it is
exactly the right invariant for a convolutional policy. A convolution wants a
lattice of constant physical pitch and a variable extent, because its kernel
then means one thing at every world size. A downsampled minimap of fixed
resolution, as the large strategy-game agent uses, means something different
at each map size. **This project holds the better of the two inputs for a
size-invariant trunk.** It currently feeds that input to the one architecture
that cannot use it.

So the fit is:

- **A fully convolutional trunk over the cell lattice fits well.** The eleven
  cell fields are eleven channels over a rectangle of cells. The parameter
  count of a convolution does not depend on the extent.
- **A pooled reduction over the cell lattice fits well and is cheaper.** A
  spatial pyramid pooling layer, or a set reduction over cells, turns any
  lattice into a fixed vector. This is the published approach closest to the
  problem.[^17]
- **A graph network over cell neighbourhoods fits, and buys little now.** The
  lattice is a rectangle, so a convolution already carries the neighbourhood.
- **An entity transformer does not apply.** The observation carries no entity
  list. It carries a lattice and a scalar block. The engine's control-plane
  rule forbids a per-entity boundary, so an entity list is not available and is
  not wanted.[^20]

---

## 5 Count invariance in the published work

**Published.** Any permutation-invariant function of a set has the form
`rho(sum of phi(element))`, and with expressive enough parts this is a
universal approximator for such functions.[^21] This is the licence for
pooling over rivals rather than indexing them.

**Published, and measured.** A permutation-invariant critic built from a graph
convolution gives an identical output under any relabelling of the agents. It
scales to thirty times more agents, with 15 to 50 per cent higher test return
than a critic that concatenates the agents in a caller-chosen order.[^22] The
paper frames the problem exactly as section 1.2 frames it: a concatenated
input in a user-specified order changes the output when the order changes,
although the environment did not.

**Published.** A set transformer gives permutation invariance with attention
rather than a sum. Its induced-point block reduces the cost from the square of
the set size to a product with a small constant.[^23] This is the answer when
a sum over rivals loses too much.

**Published.** Mean field multi-agent learning replaces the interaction among
many agents. One agent instead interacts with the average of its neighbours.
The cost for each agent then stops following the population.[^24]

**Published.** A generalist agent for a cooperative card game plays every
setting from two to five players with one network. It encodes the state as
text. A bit string would change shape with the player count, and text does
not.[^25] It reports one asymmetry that matters to a curriculum: an agent
trained with more players generalises down to fewer players more easily than
the reverse.

**Unverified.** An attention critic for multi-agent learning argues that
attention scales well with the agent count. The author found no experiment
there that trains at one agent count and evaluates at another.[^26] Another
entity-graph work claims strong zero-shot transfer across team sizes, and the
author could not confirm the team sizes or the scores.[^27]

### 5.1 The two failure modes, named for this project

**A policy that learns a seat order rather than a relation.** This is not a
future risk here. Section 1.2 measured it: 123 of 184 positions are indexed by
absolute faction number, and the training loop rotates the seat within one
generation.[^8] The published fix is to make each rival's block relative to
the reader and then pool over the rivals.

**A policy that cannot go from three rivals to seven.** A sum pool over rivals
gives an answer at any count, and the answer changes scale with the count. A
mean pool holds the scale and loses the count. The published guidance is to
supply both, and to supply the count as its own feature. **Unverified:** the
author found no measurement of the sum-against-mean choice in a strategy game.

The asymmetry from the card-game work suggests training with more factions
than the deployment target, not fewer.[^25]

---

## 6 Transfer and curriculum

### 6.1 What the published work measures

**Published, and measured.** Training a policy on a limited set of
procedurally generated levels overfits badly. With 100 training levels the
agent reaches 99.45 per cent on the training set and 66.79 per cent on
held-out levels. With 4,000 levels the gap falls to 11.83 points. With 16,000
levels it falls to 1.66 points, and the authors state that overfitting is
still noticeable.[^28] A broader benchmark reaches the same conclusion over
sixteen environments.[^29]

**Published.** Reinforcement learning agents overfit robustly, and the
stochasticity usually injected into an environment neither prevents the
overfitting nor detects it.[^30] **This matters directly here.** Adding seeds
to the world generator does not prove that a policy generalises.

**Published.** Generating levels during training gives generalisation within
the same distribution, for some games and not for others. A
difficulty-progression curriculum reaches better performance from less
data.[^31]

### 6.2 The gap in the literature

**The author found no published measurement of a policy that fails at a larger
map because it exploited small-map structure.** The literature varies the
identity of the level and the difficulty of the level. It does not vary the
map dimension and attribute a failure to it. The closest measurement points
the other way: in a real-time strategy game with a pooled encoder, the agents
suited larger maps better than smaller ones.[^17]

**If this project needs that fact, it must measure it.** Section 9 gives the
cheapest measurement that would produce it.

### 6.3 The specific risk in this project's training regime

**Measured, from the holdout of the run report.** The four players average
between 3.7 and 6.0 live units, and between 136 and 293 held tiles of 2,304.

**Read.** The whole map is four cells, and one cell covers 44 per cent of the
world.[^5]

A regime of five units on a four-cell map contains no frontier, no chokepoint,
no supply line and no army. A policy that plays it well has learned the
opening of a game and nothing after it. The project targets one million units.

**This is not a size-invariance problem, and no architecture fixes it.** A
size-invariant network trained on this regime learns the same opening and
applies it at every size.

---

## 7 The cost of invariance on one processor box, without gradients

### 7.1 What the trainer is today

**Read.** The trainer runs a mirrored-sampling, rank-shaped evolution
strategy. The population is 16. Each candidate has a mirror, so one generation
draws 8 directions. The perturbation scale is a fraction of the length of the
centre. A pool of worker processes scores one generation and combines the
shards in candidate order.[^32]

**Measured.** The trained policy holds 5,133 parameters.

**A population of 16 searches 8 directions in 5,133 dimensions.** Section 2
measured the outcome: the weights are isotropic after eleven generations.

### 7.2 What each proposal costs

The parameter count of a convolution does not depend on the extent of its
input. A trunk of C channels and a kernel of K cells holds `C_in * C_out * K`
weights whatever the lattice size. **A convolutional or pooled trunk therefore
does not raise the parameter count with the world size.** It raises the
forward-pass cost with the world size, which is a different budget.

The table states the parameter count of each proposal at the training size and
at the target size. The counts are arithmetic over the shapes, not
measurements.

| Proposal | Parameters at 48 by 48 | Parameters at 4096 by 4096 |
|---|---|---|
| Linear over the flat array, today | 5,133 | 5,230,585 |
| Linear over a pooled cell summary | About 2,000 | About 2,000 |
| One hidden layer over a pooled summary | About 4,000 | About 4,000 |
| Small convolutional trunk over the lattice | About 20,000 | About 20,000 |

**The pooled proposals are cheaper than the policy the project runs today.**
That is the answer to the brief's cost question, and it is the opposite of
what the brief expected. Invariance is expensive when it replaces a small
fixed vector with a large network. Here it replaces a matrix whose width
follows the world with a matrix whose width does not.

The convolutional proposal is about four times the current parameter count,
and it needs a forward pass over the lattice rather than one matrix product.

### 7.3 What an episode costs at each size

**Measured.** The author ran one episode at each of four sizes, with a random
legal action in the learner seat and the built-in controller in the other two
seats. The tick limit was 2500 and the horizon was 250 decisions.

| Width | Height | Observation length | Decisions | Seconds | Seconds for each decision |
|---|---|---|---|---|---|
| 48 | 48 | 184 | 64 | 6.2 | 0.10 |
| 96 | 96 | 239 | 250 | 110.2 | 0.44 |
| 256 | 256 | 844 | 250 | 98.7 | 0.39 |
| 512 | 512 | 2,956 | 250 | 142.6 | 0.57 |

**The cost for each decision grows far more slowly than the tile count.** A
world 256 tiles on a side holds 28 times the tiles of the training world and
costs about 4 times as much for each decision. The cost follows the population
and the settlements, not the ground.

**The episode length is the real cost, and it grows the wrong way.** The
episode at 48 tiles on a side ended after 64 decisions. Every larger episode
ran to the horizon at 250 decisions and did not resolve. A wider world
therefore costs about 4 times as much for each decision *and* about 4 times as
many decisions. **Widening the training world to 256 tiles on a side is
roughly a 16-fold cost, not a 4-fold one.**

That does not remove the recommendation in section 9.2, and it changes what
the recommendation must carry with it. A wider world needs a resolution rule,
or a generation budget that admits an unresolved game, or both. Section 6.3
already notes that the present regime is the opening of a game. A wider world
makes more of the game an opening.

### 7.4 Whether a gradient-free optimiser can carry it

**Published.** The reference work on gradient-free evolution strategies states
that the method scales to over a thousand parallel workers.[^33] It solved a
three-dimensional walking task in ten minutes at that scale. **Unverified:**
the author read the abstract and could not confirm the population size for
each iteration or the parameter count of the networks.

**The population here is 16.** The author found no source that recommends a
population of 16 for a search in thousands of dimensions.

**Published, and partly unverified.** The standard tutorial on the
covariance-adapting evolution strategy gives a default population size that
grows with the logarithm of the dimension.[^34] The author could read only the
abstract, so the exact formula is unverified. **The direction is what matters
here: the recommended population rises with the dimension, and this project
holds it fixed at 16.**

**Published.** Linear policies match the state of the art on standard
continuous control benchmarks.[^35] A basic random search trains such policies
without any gradient, and it is competitive there.[^42] **A small parameter
count is therefore not itself the limit.**

**The recommendation that follows.** A pooled trunk holds a few thousand
parameters. That is inside what a gradient-free optimiser on one box can
search, and it is roughly what the project already searches. **A convolutional
trunk of twenty thousand parameters is a different project on this
optimiser.** Say so rather than proposing it quietly. If the project wants a
convolution, it wants back-propagation, and back-propagation on a processor
box is a decision with a cost the project has not measured.[^36]

### 7.5 What a larger population buys, in numbers

**Read.** The trainer takes half the population as antithetic directions.[^32]
A population of 16 therefore searches 8 directions in one generation, and a
population of 256 searches 128.

**Read.** The trainable dimension is not the same for the two policy kinds.
The linear policy trains every entry of one matrix, which is 5,365 weights at
the training shape. The network of one hidden layer holds a fixed random
projection in its first layer and trains only the second, which is the action
count times the hidden width.[^32] A hidden width of 32 gives 928 trainable
weights, and a hidden width of 177 gives 5,133.

The estimator of an evolution strategy averages the directions it drew, so its
signal against its noise rises with the square root of the direction count and
falls with the square root of the dimension. The table takes that ratio as a
proxy, against the measured run as the unit.

| Policy | Directions | Dimension | Proxy | Against the measured run |
|---|---|---|---|---|
| Linear, population 16 | 8 | 5,365 | 0.039 | 1.0 |
| Linear, population 256 | 128 | 5,365 | 0.155 | 4.0 |
| Hidden width 177, population 256 | 128 | 5,133 | 0.158 | 4.1 |
| Hidden width 64, population 256 | 128 | 1,856 | 0.263 | 6.8 |
| Hidden width 32, population 256 | 128 | 928 | 0.371 | 9.6 |

**The fixed projection is the reason the network can beat the linear policy
here.** It searches a smaller space for the same forward pass. A run that
matches the two on total weights gives the network no advantage at all.

**The episode budget is the other half, and it is the larger half.** The
measured run played 1,152 training episodes over 18 generations. A generation
of 256 candidates at one seed plays 256 episodes, so 4.5 generations spend the
whole budget of the measured run. Eighteen such generations play 4,608
episodes.

**One seed for each candidate is safe only when the pairing cancels the
seed.** The runner puts a perturbation and its mirror on the same seat of the
same seed, and it ranks the margin against the other seats of that world.[^8]
A run that turns that off, and scores a candidate on the absolute return of
one episode, ranks the seed rather than the candidate. **Check that the run
report records the relative scoring as on.**[^43]

---

## 8 What the schema would have to become

Three changes exist. They are independent, and they rank by cost.

### 8.1 No engine change, and a check

**Measured.** The binding publishes the width of the cell lattice, and that
width agrees with the observation lattice at four shapes the author
built.[^37] Three were square and one was not. A world 96 tiles wide and 64
tiles high gives a published width of 3, and its cell fields hold 6 positions.

**Read.** The binding publishes no lattice height. The observation schema
publishes the length of each cell field, which is the cell count.[^4] A caller
gets the height by dividing the cell count by the published width. A Python
encoder can therefore fold the cell part of the array into eleven channels
over a rectangle of cells. It can then reduce that array. This needs no change
to the engine.

**This needs no record and no version bump.**

**It does need a check.** The published width comes from the pyramid layout,
and the observation is written from the fog layout. That is one number
declared in two places, which is the first recurring defect shape of this
project.[^38] Add a check that the cell count of the schema divides exactly by
the published width. A comment naming which one wins is not enough.[^38]

### 8.2 Make each rival's block relative to the reader

The `relation` field and the five board fields carry the seat-order defect of
section 1.2 and section 5.1. Making them relative to the reader moves the
field set, so it raises the observation version and it invalidates every
stored file.[^12]

**This needs a record.** The claim is one sentence a reviewer can check: *a
faction's observation names another faction by a position relative to the
reader, never by a seat number.* An existing record already forbids a field
with one plane for each faction, and this record would extend that reasoning
from the plane to the ordering.[^10]

The record would state the ordering rule, because a relative order needs one.
A stable key is the project's usual answer.[^39]

### 8.3 Give the observation its own lattice pitch

Report 33 states this option and recommends against taking it now.[^5] It
contradicts the record that says the observation shares the fog and summary
lattice, so it needs a record of its own.[^3] A findings row warns that a
second address space over one world samples the wrong cells and fails
nowhere.[^40]

**Do not take this until a policy uses the lattice it already has.**

---

## 9 The recommendation

### 9.1 The first step, in one working session

**Measure the transfer that costs nothing to measure.**

A world 48 tiles on a side and a world 64 tiles on a side share the
observation length, the action length, the cell count and the faction count.
They differ only in extent. That pair is the one case where the current policy
can play another world with no change to the policy at all.

The step is:

1. Add a switch to the loader that reports a fit mismatch rather than raising,
   for the width and the height only. Keep the refusal for the two versions,
   the two lengths and the faction count.
2. Play the stored imitation policy on the held-out seeds at 48 by 48 and at
   64 by 64. Play the random baseline on both.
3. Report the four win rates and the four returns.

**This answers a question nobody has answered, and the answer changes the
plan.** If the policy holds its margin over random at the larger extent, the
fit check is protecting against a theoretical loss, and the project can widen
the training world with confidence. If the margin collapses, the project has
the small-map-structure measurement that section 6.2 says nobody has
published.

The imitation checkpoint is the right subject, because section 2 shows it is
the only stored file that carries information.

### 9.2 The path behind it, ranked by evidence

**Rank 1. Fix the optimiser before the architecture.** Section 2 measures a
search that selects nothing. The population is 16 in 5,133 dimensions. The
reference work on this method scales to over a thousand parallel workers.[^33]
Nothing below this rank is worth doing while the weights stay isotropic.
*Evidence: measured, in this project.*

**Rank 2. Make each rival's block relative to the reader.** Section 1.2 shows
the defect is live at three factions, and section 5 gives a measured 30-fold
scaling result for the permutation-invariant alternative.[^22] This is a
prerequisite for any count invariance and a repair of a present defect.
*Evidence: measured in this project, and measured in the published work.*

**Rank 3. Train at 256 tiles on a side.** That gives an 8 by 8 lattice and 64
cells. It changes no engine code. Report 33 already recommends it.[^5] Until
the lattice says something, no size-invariant trunk has anything to be
invariant over. **Section 7.3 measures the price: about 16 times the cost of
one generation, because a wider world costs more for each decision and
resolves less often.** Take this rank with a resolution rule or with a larger
budget, and not on its own. *Evidence: read from the code, measured for cost,
and argued.*

**Rank 4. Pool the cell lattice in Python.** Section 8.1 shows this needs no
engine change. Section 7.2 shows it is cheaper than the policy today. The
published spatial-pooling result is the closest match to this problem, and its
numbers are unverified.[^17] *Evidence: published, partly unverified.*

**Rank 5. Adopt a convolutional trunk.** Section 7.4 says this needs
back-propagation, which is a different project on this hardware. Take it only
after ranks 1 to 4, and take it as a stated decision about the optimiser.
*Evidence: the mechanism is published and sound; the zero-shot transfer claim
most often cited for it is one unmeasured sentence.[^14]*

---

## 10 The risks in the current checkpoints

**Every stored file is one world.** All ten name a world 48 tiles on a side
with three factions, and all name seat 0.

**Six of the ten files cannot load at all.** They state observation version 1
and a length of 176. The engine now publishes version 3 and a length of 184.

**The evolution checkpoints carry no information.** Section 2 measured their
weight as flat to within two per cent across every field group. Treat them as
a starting point, not as an asset. A plan that assumes they encode strategy is
planning against noise.

**The imitation checkpoints carry information, and it is information about one
regime.** Section 6.3 measures that regime: about five units, four cells, and
the opening of a game.

**The holdout is 24 episodes.** A 2.7 point difference between the trained
policy and a random legal action is not a result at that sample size.

---

## 11 Whether this is the right problem now

**Mostly no. Say it plainly.**

Size invariance is a change to the function class. It pays when a project is
limited by the function class. This project is limited by three things that
sit below the function class:

1. The optimiser selects no feature in the observation it has (section 2).
2. The trained policy is not distinguishable from a random legal action, and
   the holdout is too small to separate any two players in it (section 2).
3. The training world holds four cells, so there is no spatial structure for a
   spatial architecture to exploit (section 6.3).

A size-invariant policy trained under those three conditions would be a
size-invariant policy that plays badly at every size.

**Two parts of this work are not premature, and they are not size
invariance.**

**The seat-order defect is live today.** 123 of 184 observation positions and
one of twelve verbs index a faction by its absolute number, while the training
loop rotates the seat. That is a defect at three factions. It happens to also
be the thing that blocks count invariance later, which is a reason to take it
now and not a reason to wait.

**Widening the training world costs nothing in code.** It is the difference
between a lattice of 4 cells and a lattice of 64. Report 33 recommends it, and
this report agrees.[^5]

**One more thing is worth doing because it is cheap and it settles an
argument.** The measurement in section 9.1 takes one session and produces
evidence that the published literature does not contain.

---

## 12 What the author could not determine

- **Whether the cost table of section 7.3 holds under a trained policy.** The
  author measured one episode at each size, with a random legal action in the
  learner seat. A trained policy resolves more games, and a resolved game is
  shorter. One blocker governs every cost figure of this project, and these
  figures were measured on a development machine and not on the target
  platform.[^36] [^41]
- **The board sizes and the win rates of the Hex transfer work, and the map
  sizes and the win rates of the spatial-pooling work.** Both sources are the
  measured evidence behind ranks 4 and 5, and the author could not read either
  document's numbers.[^15] [^17]
- **Whether the published lattice width agrees with the observation lattice at
  every world shape.** The author measured agreement at four shapes, one of
  them not square. Section 8.1 asks for a check rather than a fifth
  measurement.
- **Whether the isotropy in section 2 is a property of this run or of the
  algorithm at this population.** The author measured three checkpoints of one
  run against a drawn null. A run at a larger population would separate the
  two, and section 2 gives the band it must leave.
- **Whether the four holdout players differ once the episodes are paired.**
  The run report stores only the aggregates. A paired test needs the outcome
  of each episode, and nothing writes it.
- **The cost of one training run in money.** The brief states about five
  dollars. The author did not verify it, and a blocker governs cost
  figures.[^36]
- **Whether any published work measures a small-map policy failing at a larger
  map for the reason this project fears.** The author found none. That is a
  negative result and it may be wrong.

---

## References

[^1]: Decision Record Scope, section 4.3. `.agents/rules/adr-scope.md`
[^2]: The block layout, the default block edge exponent. `crates/cachette-core/src/bridge.rs`
[^3]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^4]: The faction observation module. `crates/cachette-core/src/faction_observation.rs`
[^5]: Report 33, what a learner can see, say and be scored on, section 5. `docs/research/reports/33-what-a-learner-can-see-and-say.md`
[^6]: Findings register, FND-635. `docs/FINDINGS.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: The seated league runner. `python/cachette/learn/league.py`
[^9]: The policy module, the fit and its reader. `python/cachette/learn/policy.py`
[^10]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^11]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decision D2. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^12]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D2. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^13]: ADR-0024, every summary field is declared extensive or intensive. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^14]: Polygames: Improved Zero Learning. Cazenave and others, ICGA Journal 42(4), 2020. https://arxiv.org/abs/2001.09832
[^15]: A transferable neural network for Hex. Gao, Yan, Hayward and Müller, ICGA Journal 40(3), 2018. https://journals.sagepub.com/doi/full/10.3233/ICG-180055
[^16]: From Images to Connections: Can DQN with GNNs learn the Strategic Game of Hex? 2023. https://arxiv.org/abs/2311.13414
[^17]: Enhancing deep reinforcement learning for scale flexibility in real-time strategy games. Lemos and others, Entertainment Computing, 2024. https://www.sciencedirect.com/science/article/abs/pii/S1875952124002118
[^18]: Grandmaster level in StarCraft II using multi-agent reinforcement learning. Vinyals and others, Nature 575, 2019. https://www.nature.com/articles/s41586-019-1724-z
[^19]: Relational Deep Reinforcement Learning. Zambaldi and others, 2018. https://arxiv.org/abs/1806.01830
[^20]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequences. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^21]: Deep Sets. Zaheer and others, NeurIPS 2017. https://arxiv.org/abs/1703.06114
[^22]: PIC: Permutation Invariant Critic for Multi-Agent Deep Reinforcement Learning. Liu, Yeh and Schwing, CoRL 2019. https://arxiv.org/abs/1911.00025
[^23]: Set Transformer. Lee and others, ICML 2019. https://arxiv.org/abs/1810.00825
[^24]: Mean Field Multi-Agent Reinforcement Learning. Yang and others, ICML 2018. https://arxiv.org/abs/1802.05438
[^25]: A Generalist Hanabi Agent. Sudhakar and others, ICLR 2025. https://arxiv.org/abs/2503.14555
[^26]: Actor-Attention-Critic for Multi-Agent Reinforcement Learning. Iqbal and Sha, ICML 2019. https://arxiv.org/abs/1810.02912
[^27]: Learning Transferable Cooperative Behavior in Multi-Agent Teams. Agarwal, Kumar and Sycara, 2019. https://arxiv.org/abs/1906.01202
[^28]: Quantifying Generalization in Reinforcement Learning. Cobbe and others, ICML 2019. https://arxiv.org/abs/1812.02341
[^29]: Leveraging Procedural Generation to Benchmark Reinforcement Learning. Cobbe and others, ICML 2020. https://arxiv.org/abs/1912.01588
[^30]: A Study on Overfitting in Deep Reinforcement Learning. Zhang and others, 2018. https://arxiv.org/abs/1804.06893
[^31]: Illuminating Generalization in Deep Reinforcement Learning through Procedural Level Generation. Justesen and others, NeurIPS 2018 Deep RL Workshop. https://arxiv.org/abs/1806.10729
[^32]: The trainer and the shard pool. `python/cachette/learn/train.py`
[^33]: Evolution Strategies as a Scalable Alternative to Reinforcement Learning. Salimans and others, 2017. https://arxiv.org/abs/1703.03864
[^34]: The CMA Evolution Strategy: A Tutorial. Hansen, 2016. https://arxiv.org/abs/1604.00772
[^35]: Towards Generalization and Simplicity in Continuous Control. Rajeswaran and others, NeurIPS 2017. https://arxiv.org/abs/1703.02660
[^42]: Simple random search provides a competitive approach to reinforcement learning. Mania, Guy and Recht, 2018. https://arxiv.org/abs/1803.07055
[^43]: The learner command line, the seed split and the baselines. `python/cachette/learn/__main__.py`
[^36]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^37]: The Python binding crate, the level 1 lattice width. `crates/cachette-py/src/lib.rs`
[^38]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^39]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^40]: Findings register, FND-569. `docs/FINDINGS.md`
[^41]: ADR-0008, the primary target is aarch64. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^44]: The strategy table of the trainer. `python/cachette/learn/__main__.py`
[^45]: The stored policy reader of the control plane. `python/cachette/learn/policy.py`
[^46]: Reinforcement learning parameters register, the observation layout. `docs/reference/rl-costs.md`
