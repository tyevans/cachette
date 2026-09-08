# Report 34: What would move the learner

This report asks one question. Which change to the learner is most likely to
raise its win share against a competent opponent?

The report reads the published work on reinforcement learning in strategy
games. It then reads the learner surface of this project and measures it. It
ranks four changes, states what the budget forbids, and tries to falsify the
diagnosis it was given.

## 0 Provenance, and what this report could not verify

The report holds three kinds of claim, and each one is marked.

**Measured.** The author built worlds of the training shape and read them.
Section 2 holds every measurement, with the run that produced it. Nobody had
taken these numbers before.

**Verified.** The author read the paper and quotes the figure from it. Every
verified figure names its paper in a footnote.

**Derived.** The author computed the figure from a measured or verified one.
Every derived figure states the arithmetic. One blocker governs every cost
figure in this project, and it governs these too.[^1]

**No policy was trained for this report.** The author ran no training run, so
no claim about what a change would achieve is a measurement. Section 4 gives
each change an expected effect, and every expected effect is a judgement.

Two claims come from a secondary source, and each says so where it appears.

## 1 Method, and the moment this measures

The author read the action module, the observation schema, the policy module
and the reward module of this repository. The author then built three worlds
of the training shape and played each with uniform random legal actions. The
shape is 48 tiles wide, 48 tiles high, three factions, a tick limit of 2500,
a horizon of 250 decisions and one decision every 10 ticks. The seeds were 7,
11 and 13. Two games ran the full 250 decisions and one ended after 110.

**Every count in section 2 measures that moment.** The moment is the tip of
the main branch on 7 September 2026. A count belongs in a research report,
because a report is fixed to a moment. The same count in a decision record
decays.[^2]

The author ran no test suite and no full check. The author changed no file
outside this report.

## 2 What the learner surface measures

### 2.1 The action table names no place

The verb set holds twelve verbs and the table holds 29 rows.[^3] Four verbs
carry an argument. The argument is the resource kind, the upgrade category,
the faction or the unit type. Each bound is a small enumeration count.

| Verb | Rows | Argument |
|---|---|---|
| `no_op` | 1 | none |
| `gather` | 3 | resource |
| `build` | 7 | category |
| `relation` | 3 | faction |
| `campaign` | 1 | none |
| `advertise` | 1 | none |
| `trade` | 1 | none |
| `carry` | 1 | none |
| `project` | 1 | none |
| `queue` | 8 | unit type |
| `cross` | 1 | none |
| `settle` | 1 | none |

**No position of the table names a place.** The policy cannot say where to
settle, where to campaign or where to cross. The engine resolves each of
those inside the verb. A draft record states this as a rule and gives its
reasons.[^4]

The legality answer left 13.17 rows legal at the mean decision, of 29.

### 2.2 More than half the observation never moves

The observation holds 184 positions.[^5] The author counted the positions
whose value changed at any decision of any of the three episodes.

| Group | Positions | Positions that move |
|---|---|---|
| The trade board, five fields | 120 | 39 |
| The level 1 cells, eleven fields | 44 | 39 |
| The faction scalars and the relation | 15 | 8 |
| The option weights | 5 | 0 |
| **Total** | **184** | **86** |

**Ninety-eight positions never move.** The trade board takes 120 positions,
which is 65 percent of the array, and 81 of those 120 never move. The whole
spatial picture takes 44 positions. The world is 48 tiles on a side, so the
level 1 lattice holds four cells. The policy therefore reads the map as four
quadrants.

The option weights take five positions and never move. Section 4.3 returns to
that.

The policy encodes each position as a signed logarithm divided by 20, and
appends a bias of one.[^6] Pooled over the three episodes, 55 percent of the
185 features have a standard deviation below 0.01. The bias of one is the
largest single contributor to the squared length of the feature vector.

### 2.3 A linear policy over that array is close to a fixed preference

The author drew 300 random unit-length linear policies and played each over
the recorded observations and masks. Each policy took its modal action at 72.9
percent of decisions.

The author then drew 300 fixed preference orderings that read nothing. Each
one takes the highest-preferred legal row. Each took its modal action at 79.1
percent of decisions.

**A policy that reads the array behaves almost like a policy that reads
nothing.** The gap is 6.2 points of modal share. The legality mask supplies
most of the variety in both cases.

This bears directly on one reported result. The trained verb mix is settle
0.361, build 0.319 and queue 0.297. A fixed preference ordering filtered by
the legality mask produces a mix of that shape. The verb mix is therefore
weak evidence that the policy learned a strategy. It is consistent with a
fixed ordering that the mask filters.

## 3 Against the diagnosis

The brief states that action bandwidth is the bottleneck, and asks for an
attempt to falsify that.

### 3.1 The bandwidth claim has support in the literature

Every system that plays a real-time strategy game well issues an action for
every unit at every step. Gym-µRTS compares two ways to do it. Unit Action
Simulation calls the policy once for each unit. Gridnet predicts an action for
every cell of the map in one forward pass.[^7] Both issue the whole command
set each frame. Neither issues one command.

OpenAI Five issues one action per hero per timestep, and it runs one policy
instance for each of five heroes.[^8] AlphaStar uses an autoregressive head
with a pointer network, so it emits an action and its arguments as a
sequence.[^9]

### 3.2 One published system contradicts it

TStarBot1 plays the full game of StarCraft II with a **flat action space of
165 macro actions and one decision at a time**. It defeated the built-in agent
at every level from 1 to 10, and levels 8, 9 and 10 cheat.[^10] Each macro
action hard-codes a rule of the game and commands a group.

TStarBot1 shows that one macro action per decision is enough to play a real-
time strategy game at a useful level. The count of commands per decision is
therefore not the thing that decides the outcome. What each command reaches
is.

The Cachette action table is a macro action space of the same kind. It holds
29 rows against 165. Not one of its rows names a place.

### 3.3 Two rival explanations fit the evidence at least as well

**The observation does not move.** Section 2.2 and section 2.3 measure this.
A policy whose input is nearly constant cannot condition on the state,
whatever its action rate. A learner that acted 33 times per window on a
constant observation would still play a fixed opening.

**The run is far too short.** The policy holds about 5,300 parameters. An
evolution strategy takes one gradient step for each generation, so 30
generations is 30 steps in a 5,336-dimensional space. The evolution strategies
paper cites the result that the required number of optimisation steps scales
linearly with the dimension for a general non-smooth problem.[^11] Eight flat
generations is not evidence of a plateau. It is a short segment of a long run.

### 3.4 The yardstick is mis-specified, and the plateau is partly an artefact

The brief calls the result a plateau at chance. That reading conflates two
different things.

The controller yardstick is a symmetric three-faction game of the controller
against itself, so its win share is 1/3 by construction. A learner that
reaches 11/32, which is 0.344, has reached **parity with the scripted
controller**. It has not reached chance. A random policy in this game wins far
less, and the reported verb mix of the random policy shows why: it settles at
0.013 against the learner's 0.361.

Rising from 6/32 to controller parity in five generations is a real result.
The flatness after it is what a fixed opponent produces. Once the learner
matches the controller, the win share still carries signal above 1/3, but the
signal is small and the noise is not.

### 3.5 The experiment that decides it

One cheap run separates the bandwidth hypothesis from the two rivals.

**Lower the decision interval from 10 ticks to 2, and change nothing else.**
The environment already takes the interval as a parameter.[^12] The learner
then issues five times as many actions per game, against the same observation,
the same reward and the same opponent.

The evolution strategy tolerates the change. The gradient estimate of an
evolution strategy is independent of the episode length, and the authors
measured near-identical learning curves at frame skips of 1, 2, 3 and 4.[^11]
A policy gradient method would not tolerate it as cheaply, because its
gradient variance grows about linearly with the episode length.

If the win share moves, bandwidth is the bottleneck. If it does not move,
bandwidth is not the bottleneck, and the ranking in section 4 stands. The cost
is the extra crossings of the boundary. **The author did not measure that
cost**, and it is the one thing to check before the run.

## 4 The four changes, ranked

Each change states what to do, the evidence behind it, and the work involved.
The work estimates are judgements, not measurements.

### 4.1 Score the run on a continuous, rival-relative margin

**What to do.** Read the weighting the training runs used. If the terminal
weights carry the score and the shaped terms are empty, the fitness is a
three-valued number. Replace it with a continuous margin. Take the held ground
of the learner's faction, subtract the mean of the rivals, and add the terminal
term on top.

**Why it is first.** The gradient variance of an evolution strategy factors
into two terms, and the first is the variance of the return:

> Var[∇θ FES(θ)] ≈ Var[R(a)] Var[∇θ log p(θ̃; θ)]

That relation is from the paper the trainer's algorithm comes from.[^11] The
second term is a property of the perturbation. **The first term is a property
of the score, and a continuous score has less of it than a three-valued
one.** Nothing else in the loop reduces variance as cheaply.

The rival-relative part has a separate justification. OpenAI Five made every
reward zero-sum by subtracting the mean of the enemies' rewards from each
hero's reward.[^8] In a three-faction game, a faction that holds its ground
while both rivals grow has done badly, and an absolute score cannot say so.

The module is already built for this. It states no weight, it takes the terms
as data, and a weighting whose shaped weights are all zero is the terminal
reward.[^13] The change is a weighting, not a rewrite.

**One warning, from the same literature.** Gym-µRTS reports that its agents
optimised the shaped return with little variance across seeds, while the
sparse win-loss return varied a great deal.[^7] A shaped score can be
maximised without winning. Keep the win term, and keep reporting the win share
beside the score.

**Work.** Small. The reward module takes the weighting as data. The trainer
already reports an absolute number beside the ranked one.

### 4.2 Replace the inert two thirds of the observation

**What to do.** Three parts, in this order.

1. **Cut the trade board down.** It takes 120 of 184 positions and 81 of them
   never move. Summarise it, or drop it until a verb needs it.
2. **Add differences and shares, not only levels.** The array carries held
   tiles, live units, population and the store as levels. A level in a
   logarithm barely moves. The change per window and the share of the world
   total both move, and both are what a decision depends on.
3. **Add rival-relative fields.** The array says how much ground this faction
   holds. It does not say how much the rivals hold. A player who cannot see
   the rivals cannot tell a good position from a bad one.

**The evidence.** Section 2.2 measures the inert positions. Section 2.3
measures the consequence: a linear policy over this array is 6.2 points of
modal share away from a policy that reads nothing.

The evolution strategies paper reports the matching failure. Under random
parameter perturbation, its policies sometimes "encode policies that always
took one specific action regardless of the state that was given as input". The
authors fixed it by changing the parameterisation of the network, not by
training longer.[^11] OpenAI Five normalised each observation by its mean and
standard deviation, then clipped to the range −5 to 5.[^8]

**One caution.** Section 5 explains that the third part faces a product
constraint. A faction sees only what it observes, so a rival-relative field
must be built from what this faction has seen.[^14]

**Work.** Medium. The observation schema is one place, and the change moves
the layout version. A moved version parts every stored weight file from every
world, and the reader refuses the file rather than playing it. The break is
loud by design.[^6]

### 4.3 Expose the option weights as a verb

**What to do.** Add a verb that sets the option weights of the faction. The
accepted record already says the weights are policy, set through one
verb.[^15] The observation already carries them, in five positions.

**Why this is the bandwidth change that the architecture permits.** A unit in
this engine scores a small fixed option set and takes the highest. The weights
say how much a unit wants each thing. One action that changes the weights
therefore changes the standing behaviour of **every unit alive**, and it keeps
changing it until the next such action.

That is the leverage the controller gets from 33 commands, bought with one
integer. It is closer to TStarBot1's macro actions than anything now in the
table: each of its 165 macro actions commands a group, not a unit.[^10]

**A capability sits inert today.** The observation publishes the weights and
no verb changes them. The author measured that they never move across three
episodes. The project rule names this shape, and the findings register holds
two local instances of it.[^16]

**Work.** Medium. It is one verb with one narrowing position, an engine change
to apply it, and a decision record. The draft record on narrowing positions
governs the argument, and a weight index is a bounded enumeration rather than
a place, so the draft permits it.[^4]

### 4.4 Replace the yardstick, and pay for more generations

**What to do.** Two parts.

1. **Measure against a reference pool, not a symmetric self-match.** Keep a
   set of reference opponents of known strength. Anchor the scale at random
   play. Rate the centre of each generation against the pool.
2. **Spend the budget on fewer, longer runs.** Thirty generations is thirty
   gradient steps.

**The evidence for the pool.** OpenAI Five evaluated against a fixed pool of
83 reference agents, rated with TrueSkill, with random play anchored at 0 and
750 games per rating. It played only opponents within 10 rating points, to
gain the most information per game.[^8] A rating against a pool has no
ceiling at 1/3.

Two cautions apply, and both are from the literature. A scalar rating misleads
when the strengths are intransitive; Nash averaging and multidimensional Elo
exist for that case.[^17] For more than two players, α-Rank ranks strategies
by the stationary distribution of an evolutionary Markov chain, and it handles
intransitivity where Elo does not.[^18] **The author read the abstract and the
framework documentation for α-Rank, not the full paper.**

**The evidence for a diverse pool in training, not only in evaluation.**
Gym-µRTS trained against a mixture: 18 of 24 parallel environments played the
2020 competition winner, and the other 6 played three weaker bots. The mixture
lifted the Gridnet cumulative win rate from 0.73 to 0.87. Without it, the
authors saw their agents lose to opponents as simple as a worker rush.[^7]

**Work.** Small for the pool. The run already plays the centre against the
controller on held-out seeds. Adding reference opponents and a rating is a
change to the reporting side.

### 4.5 The large change, stated separately: a spatial argument at level 1

This one is larger than the four above, and it collides with a draft record.
The report states it because the literature is one-sided about it.

**What it would be.** A verb such as `settle` gains a position whose candidate
is a level 1 cell. The policy head becomes convolutional over the lattice, and
it predicts a distribution per cell, as Gridnet does.[^7] The mask becomes an
array over cells and rows, as Gym-µRTS builds it.

**Why the literature wants it.** Gym-µRTS measured invalid action masking as
the difference between playing and not playing. Plain PPO reached a cumulative
win rate of 0.0 against 11 competition bots, under both action formulations.
With the full mask it reached 0.82 for Unit Action Simulation and 0.73 for
Gridnet. A partial mask on the action type alone, without masking the
arguments, reached 0.32 and 0.0.[^7] The masking paper proves that the masked
gradient is a valid policy gradient, and measures that the technique scales as
the invalid action space grows, while an invalid-action penalty does not.[^19]

Cachette already supplies a full mask on its 29 rows. The point stands anyway:
these systems get their leverage from a large, place-indexed action space that
a mask keeps tractable.

**The record that forbids it, and what answers it.** The draft record rules out
a narrowing position that names a place, and gives two costs.[^4]

The first cost is the legality answer. It returns one byte for each row, so a
table indexed by the lattice would make one decision walk the world. **This
cost is real, and it is bounded by the lattice rather than by the
population.** The pyramid already holds the level 1 lattice, and Gym-µRTS
builds a mask of exactly this shape each frame.

The second cost is that a weight file would be parted from every world of
another size. **This cost is a property of a flat table read by a dense weight
matrix, not a property of spatial actions.** The 2023 microRTS competition
winner used a fully convolutional per-cell head and transferred one model
across maps of different sizes; the authors report that transfer learning to
specific maps was critical to the win.[^20] A convolutional head has a
parameter count that does not follow the extent.

**Work.** Large. It needs a record that supersedes the draft decision, an
engine change to the mask and the verb, and a policy architecture that the
evolution strategy can still perturb.

## 5 What is out of reach at this budget

The budget is one 64-core machine for about 6 hours, at about $4.60. That is
384 core-hours. A run of 30 generations of 512 episodes at 2500 ticks is 38.4
million ticks, and at one decision every 10 ticks it is 3.84 million
decisions. Both figures are derived from the brief.

| System | Its compute | Against one run here |
|---|---|---|
| AlphaStar league | 32 TPUv3 per agent for 44 days, over a league | Out of reach by orders of magnitude |
| OpenAI Five | Up to 1536 optimiser GPUs, batches of 1 to 3 million timesteps, ten months | Out of reach by orders of magnitude |
| TStarBot1 | 1 GPU and 3,840 CPUs for 1 to 2 days | About 240 to 480 times this run |
| RAISocketAI | 70 GPU-days, 1.5 billion steps, 10 maps | Out of reach |
| Gym-µRTS best agent | 1 GPU, 3 vCPU, 16 GB, 63.67 hours, about 300 million steps | About 8 times this run in ticks |

Every figure in the table is verified from its paper.[^7] [^8] [^9] [^10]
[^20] The last column is derived.

**Say these are out of reach, and stop considering them.**

- **A league in the AlphaStar sense.** It needs a population of checkpoints,
  a matchmaking table, and main and league exploiters trained together. The
  compute is not the only cost; the machinery is large.
- **AlphaZero-style or MuZero-style search per decision.** Search multiplies
  the compute per decision by the search width. The budget has no room for a
  multiplier.
- **A pixel-scale or entity-transformer observation.** A transformer torso
  over entities is what AlphaStar used.[^9] It needs a GPU and a much larger
  sample count.
- **Training a large network.** Gym-µRTS is the encouraging case here, and its
  best model held 0.22 million parameters. Its authors report no strong
  correlation between model size and performance, and say the techniques
  mattered more than the parameter count.[^7]

**What is in reach.** The Gym-µRTS result is the honest comparator, and it is
close. Its authors call a single CPU and GPU for 2 to 4 days "a reasonable
hardware and time budget that is available to many researchers outside of
large research labs".[^7] This project is within a factor of about 8 of that
in environment steps, on CPU only.

**One further number sets the scale of the sample problem.** The evolution
strategies paper needed 3.79 × 10^7 timesteps to match the score that a policy
gradient method reached in 5 × 10^6 timesteps on Walker2d, a single continuous
control task.[^11] One run here is 3.84 × 10^7 ticks. **One full run buys
about the sample budget of one continuous control task.** That is the strongest
argument for spending the budget on fewer, longer runs.

## 6 Imitation from a scripted opponent

The brief reports a behaviour cloning fit that scored 0.845 window accuracy
against a constant answer's 0.829, and won 0.042 of games. The brief calls
this confounded, because the label reduces about 33 commands to one action.

**The literature says the confound is the whole explanation.**

Imitation from a bot works in a game of this shape. The 2023 microRTS
competition winner was built that way. Its authors cloned the behaviour of
prior competition agents, spent 23 GPU-days on the clone, then fine-tuned it
with PPO for another 49 GPU-days. They report that the fine-tuned clone
defeated the two prior competition winners in a benchmark without performance
constraints.[^20] The clone read a per-cell action head, so its label was the
whole simultaneous command set.

AlphaStar's supervised agent, trained on human replays, played better than 84
percent of active players before any reinforcement learning ran. **The author
read this from the DeepMind article, not from the Nature paper.**[^9]

**The many-commands-to-one-label problem has a standard solution, and it is
the per-cell head.** Gridnet predicts an action for every cell in one step and
the environment ignores the cells with no unit.[^7] The label is then the
whole command set, and no reduction is needed. Cachette cannot use that
solution today, because the action table names no place. Section 4.5 states
what it would cost.

**Two further notes on why cloning fails.**

The accuracy figures given are close to the class prior. A constant answer
scores 0.829, so the label set is dominated by one class. Window accuracy is
the wrong instrument under that imbalance, and a 1.6-point gain over the
constant answer is weak evidence either way.

Behaviour cloning also compounds its errors. The classification regret of
naive cloning grows as O(T²ε) in the horizon T, while an interactive method
such as DAgger achieves O(Tε).[^21] An episode here runs 250 decisions, so the
horizon term is large. A clone that is slightly wrong drifts into states the
controller never visits, and it has no label there.

## 7 Reward and episode length

The brief reports that about 67 percent of games are unresolved at the tick
cap, and the winner is then decided by held ground.

**Practitioners shape the reward, and they anneal the shaping away.**

Gym-µRTS gives +10 for winning, −10 for losing, +1 per resource harvested, +1
per worker produced, +0.2 per building, +1 per valid attack and +4 per combat
unit produced. Its authors say the weights were picked by hand with very
little tuning, and that they avoided very large win rewards.[^7] The 2023
competition winner mixed rewards over the course of training, starting with
shaped terms and finishing with the win-loss reward alone.[^20]

OpenAI Five gives a win 5 and gold 0.006 per unit, and states that its shaped
reward is modelled loosely on potential-based shaping while the guarantees do
not apply.[^8] Those guarantees come from a 1999 result: a shaping term that
is the difference of a potential function over states preserves the optimal
policy, and no other form of shaping does.[^22] **A shaping term built as a
potential difference cannot change which policy is best.** A term built any
other way can.

TStarBot1 is the counter-case, and it matters. It used a ternary reward of
1, 0 and −1 with **no shaping at all**, and beat the cheating built-in
agent.[^10] It also used 3,840 CPUs. Sparse rewards work when the sample count
is large enough. This project's sample count is not.

**For an unresolved game, the tie-break is the reward.** Held ground at the
tick cap already decides 67 percent of games. Scoring the run on the held
ground margin, as section 4.1 recommends, is therefore not a proxy for the
objective. It is the objective, made continuous.

## 8 Where the literature is thin, and where this project is unusual

**Nobody publishes on evolution strategies for a strategy game.** Every
strategy game result the author found uses a policy gradient method, a search
method, or a supervised start. The evolution strategies literature is Atari,
MuJoCo and continuous control. This project is therefore not on a mapped path,
and it cannot read a matching result off a paper.

The choice is more defensible than it looks. An evolution strategy has an
advantage when the episode is long, when actions have long-lasting effects,
and when no good value function estimate is available.[^11] All three hold
here. **The report does not recommend a switch to PPO**, and the ranking in
section 4 leaves the algorithm alone.

**Three-player evaluation is thin.** Almost every published evaluation is
two-player and zero-sum. Elo, TrueSkill and exploitability all assume it.
α-Rank handles more than two players, and the author read only its abstract
and its framework documentation.[^18] A three-faction symmetric game in which
two seats hold the same scripted controller is an unusual measurement setup,
and the report found no published treatment of it.

**A macro action space with no place index is unusual.** TStarBot1 is the
nearest published system, and its 165 macro actions still reach places through
zone-scoped forms such as an attack from one zone to another.[^10] The author
found no published system whose whole action space names no place.

**The engine is unusually cheap, and that is not exploited.** Gym-µRTS and
TStarBot1 both spend most of their budget on environment steps. This engine
releases the global interpreter lock for the whole step and steps a batch of
worlds. A method whose cost is samples rather than gradients suits it. That is
an argument for the current algorithm, not against it.

## 9 What this report did not do

- It trained no policy and measured no win share. Every claim about what a
  change would achieve is a judgement.
- It did not measure the cost of a shorter decision interval. Section 3.5
  names that as the thing to check before the deciding run.
- It did not read the weighting that the reported runs used. Section 4.1
  assumes the terminal weights carry the score, and the first step of that
  change is to check the assumption.
- It did not read the AlphaStar Nature paper. The two AlphaStar claims come
  from the DeepMind article and are marked where they appear.
- It read the abstract of the α-Rank paper and the framework documentation,
  not the full paper.
- It ran no test suite and no gate.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Decision Record Scope, section 4.3. `.agents/rules/adr-scope.md`
[^3]: The action module. `crates/cachette-core/src/action.rs`
[^4]: ADR-0184, a verb narrows its set by a bounded categorical position, decisions D3 and D4. `docs/adrs/draft/adr-0184-a-verb-narrows-its-set-by-a-bounded-categorical-position.md`
[^5]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^6]: The policy module. `python/cachette/learn/policy.py`
[^7]: Huang, Ontañón, Bamford and Grela, Gym-µRTS: Toward Affordable Full Game Real-time Strategy Games Research with Deep Reinforcement Learning, 2021, read 7 September 2026. https://arxiv.org/abs/2105.13807
[^8]: Berner and others, Dota 2 with Large Scale Deep Reinforcement Learning, 2019, read 7 September 2026. https://arxiv.org/abs/1912.06680
[^9]: Vinyals and others, AlphaStar: Grandmaster level in StarCraft II using multi-agent reinforcement learning, DeepMind, 2019, read 7 September 2026. https://deepmind.google/blog/alphastar-grandmaster-level-in-starcraft-ii-using-multi-agent-reinforcement-learning/
[^10]: Sun and others, TStarBots: Defeating the Cheating Level Builtin AI in StarCraft II in the Full Game, 2018, read 7 September 2026. https://arxiv.org/abs/1809.07193
[^11]: Salimans, Ho, Chen, Sidor and Sutskever, Evolution Strategies as a Scalable Alternative to Reinforcement Learning, 2017, sections 3.1, 3.2, 2.2 and 4.4, read 7 September 2026. https://arxiv.org/abs/1703.03864
[^12]: The learner environment. `python/cachette/learn/env.py`
[^13]: The reward module. `python/cachette/learn/reward.py`
[^14]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^15]: ADR-0156, a faction's option weights are policy, set through one verb. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^16]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
[^17]: Balduzzi, Tuyls, Perolat and Graepel, Re-evaluating Evaluation, 2018. https://arxiv.org/abs/1806.02643
[^18]: Omidshafiei and others, α-Rank: Multi-Agent Evaluation by Evolution, 2019. https://arxiv.org/abs/1903.01373
[^19]: Huang and Ontañón, A Closer Look at Invalid Action Masking in Policy Gradient Algorithms, 2020, Proposition 1 and Table 2, read 7 September 2026. https://arxiv.org/abs/2006.14171
[^20]: Goodfriend, A Competition Winning Deep Reinforcement Learning Agent in microRTS, 2024, read 7 September 2026. https://arxiv.org/abs/2402.08112
[^21]: Ross, Gordon and Bagnell, A Reduction of Imitation Learning and Structured Prediction to No-Regret Online Learning, 2011. https://arxiv.org/abs/1011.0686
[^22]: Ng, Harada and Russell, Policy Invariance Under Reward Transformations: Theory and Application to Reward Shaping, ICML 1999. https://dl.acm.org/doi/10.5555/645528.657613
