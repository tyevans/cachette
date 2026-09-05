# Reinforcement Learning Approaches For One Faction

Research report 22. It asks how to train a policy that plays one faction of
the Cachette world against the built-in controllers, and which libraries to
use. Prepared 5 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The game layer design gives each faction one controller inside the
step, four win paths, and a game that ends at a game end record.[^1] This
report treats that world as a reinforcement learning environment and asks what
fits it.

## 0. Provenance, and what this report could not verify

Every claim about a library in this report is a read of its repository, its
package index page, or its documentation on 5 September 2026. The footnote
names the page. Every claim about an algorithm cites the paper that introduced
it.

**No claim here is a measurement taken by this author.** No policy was trained.
No library was installed on the target platform. Every figure in section 6 is
a derived expectation, and one blocker governs every cost figure in this
project.[^2] A statement that this author could not check against a source is
marked **unverified** in the text.

## The environment, as a learner sees it

The design gives these facts.[^1]

- The world is deterministic. One seed and one action log give one game.
- A game holds 4 to 63 factions. Sixty-three is a bit ceiling, and a
  register holds it.[^3]
- An action is a set-valued verb. Python builds a selector, and Rust runs the
  verb over the set.
- An observation is an aggregate that the engine already holds: a score per
  faction, a census, and a level 1 map. The level 1 cell count is bounded.
- A game runs for a few thousand ticks and ends on one of four paths:
  domination, territory at the tick limit, wealth or wonder, and renown.
- The step releases the Python global interpreter lock. One process can hold
  many worlds.
- A balance harness plays a fixed seed set to game end and reports.[^4]

For the learner this means a bounded discrete action table with three parts
(verb, target from a candidate list, magnitude bucket), a flat integer
observation with a small spatial map, a sparse terminal reward, and a cheap,
exactly repeatable environment. Each section below states what follows from
these facts.

## 1. Algorithm families

### 1.1 Findings

**PPO is the default for this shape.** Proximal policy optimisation samples
from the environment, then runs several optimisation epochs on the sample
under a clipped surrogate objective.[^5] It is on-policy, so it needs fresh
samples each iteration. It tolerates a large discrete action space when the
policy masks invalid actions, and masking is a valid policy gradient, not a
heuristic.[^6] Every library in section 5 ships a PPO. What PPO needs from the
environment is a vectorised step over many worlds and an action mask per
world per tick. What it costs is sample count. A sparse terminal reward over a
few thousand ticks makes that cost high, and section 4 gives the remedy.

**IMPALA and V-trace buy throughput, not sample efficiency.** IMPALA separates
actors from the learner and corrects the off-policy lag with V-trace.[^7] It
pays when the environment is slow relative to the network, because actors run
ahead. Cachette steps a world in Rust with the lock released, so the
environment is not the slow part until the world is large. This author expects
the asynchronous gain to be small on one machine. That is a derived
expectation, and section 6 gives the reasoning. Sample Factory ships this
architecture, and section 5 covers it.

**MuZero-style planning fits a deterministic environment, and still does not
pay first.** MuZero learns a model and plans in the latent space with tree
search.[^8] A deterministic environment removes one source of model error, so
the learned model has less to learn. But the exact environment is already
available and cheap, so a learned model buys nothing that a copy of the world
does not. Planning with the real engine is the honest version of this idea.
Sampled MuZero extends the search to large action spaces by sampling
actions.[^9] Both need a search per decision, which multiplies the compute per
tick by the search width. Neither is a first step. Section 9 places them.

**Behaviour cloning from the scripted controller is a cheap warm start.** The
controller emits commands each tick, and Python can read them as a labelled
dataset at no extra cost. A supervised policy trained on that log starts near
the controller instead of near random play. AlphaStar began from imitation of
human replays and reached a stated rank before any reinforcement learning
ran.[^10] Offline RL proper, such as conservative Q-learning, corrects the
value overestimation that a fixed dataset causes.[^11] It is a later option.
The controller dataset is narrow, and a narrow dataset is the case offline RL
finds hardest.

**League self-play is the reference for multi-faction balance, and it is a
second phase.** AlphaStar trained main agents, main exploiters, and league
exploiters together, because a single self-play loop chases its own tail.[^10]
Cachette will need this once the learned policy beats every controller
setting. It costs a population of checkpoints and a matchmaking table. Section
4 gives the order.

**Evolution strategies are a cheap baseline, and the environment suits them.**
Evolution strategies perturb the parameters, run each perturbation to a
return, and step along the weighted average. The method scales to a thousand
workers with common random numbers and no gradient traffic.[^12] Cachette
gives a return only at game end, which is exactly the signal evolution
strategies use. The seed is a common random number by construction. The cost is
one full game per perturbation, and thousands of perturbations per update.

### 1.2 Recommendation

Start with PPO under an action mask. Warm start it with behaviour cloning from
the controller log. Keep evolution strategies as the control experiment that
proves the pipeline runs end to end, because it needs no gradient through the
environment and it will find a degenerate win path fast if one exists. Defer
IMPALA, MuZero and the league.

## 2. The action space

### 2.1 Findings

**A flat one-hot over the whole table breaks.** The table is the product of
the verb count, the target candidate count, and the magnitude bucket count.
With many targets and up to 63 factions as diplomatic targets, the product
reaches tens of thousands of rows. A categorical head over that many rows
learns slowly, because each row gets one gradient per time it is chosen. The
invalid action masking study found that the share of invalid actions grows
with the space, and that masking matters more as it grows.[^6] A flat head
also cannot share knowledge between rows that differ only in target.

**Autoregressive factored actions are the standard answer.** Choose the verb,
condition on it, choose the target, condition on both, choose the magnitude.
AlphaStar used this form.[^10] Each head is small. The verb head has one row
per verb. The magnitude head has one row per bucket. The joint probability is
the product of the three, so the policy gradient is unchanged. Each head takes
its own mask: the verb mask from the faction state, the target mask from the
candidate list, and the magnitude mask from the stock the verb spends.

**A pointer network handles a candidate list of varying length.** A pointer
network scores each input element with attention and selects one.[^13] The
target head becomes a dot product between a query from the verb context and an
embedding of each candidate. A list of 3 candidates and a list of 300 use the
same weights. Padding and a mask handle the varying length within one batch.

**Action masking is the mechanism that makes any of this work.** The policy
sets the logit of each invalid action to a large negative value before the
softmax, so the action is never sampled and adds nothing to the entropy.[^6]
The engine must emit the mask. It knows which verbs a faction may run, which
targets exist, and which magnitudes the stock allows. Python must not derive
the mask by looping over entities, because Python is a control plane and not a
data plane.[^14] The mask is one more aggregate reader.

### 2.2 Recommendation

Add one engine reader that returns the three masks for one faction. Build the
policy as three autoregressive heads, with a pointer over the candidate list
for the target. Do not build a flat table.

## 3. The observation architecture

### 3.1 Findings

**The level 1 map is small and bounded, so a small convolution fits.** The
cell count is a register value, and it is far below the tile count.[^3] A
convolution with two or three layers over a few channels per cell, followed by
a flatten, is enough at that size. The hex lattice has six-fold symmetry, and
a hex-aware convolution exploits it with the same parameter count.[^15] The
gain is real but small. An offset-coordinate square convolution over the hex
map is the cheap first version, because every library supports it.

**The flat vector goes through a multilayer perceptron, and the two are
concatenated.** The scores, the census, the diplomacy row for this faction,
and the tick fraction are a vector of a few hundred integers. Scale each
field to a unit range before the network sees it. Integers cross the boundary
as integers, and the scaling is a learner-side step, so the determinism rule
does not touch it.[^16]

**A transformer over the cell set is an option that does not pay at this
scale.** A set encoder over a bounded cell count is possible. It costs the
square of the cell count in attention. A convolution over the same map costs
the cell count times a constant. The transformer wins when the relation
between distant cells matters more than the local pattern. This author has no
evidence that it does here, and marks the choice as **unverified** either way.

**Recurrence handles fog, and it is the expensive part.** A faction sees only
what its units observe.[^17] What it saw ten ticks ago is not in the current
observation. A gated recurrent unit or a long short-term memory cell over the
concatenated features carries it. Recurrence complicates the rollout buffer,
because the learner must store the hidden state and replay sequences in order.
Every library in section 5 supports it in at least one algorithm. The
alternative is a frame stack of the last few observations, which costs memory
and no code.

### 3.2 Recommendation

Start with a square convolution over the level 1 map, a perceptron over the
flat vector, and a frame stack of four ticks. Add a recurrent cell only when a
measured gap to the controller is unexplained by anything else. Try a hex-aware
convolution as a later experiment.

## 4. Multi-agent considerations

### 4.1 Findings

**One learner against scripted controllers is the correct first phase.** The
controller is one system with a seeded weight vector per faction.[^1] Its
behaviour is stationary across a training run. A stationary opponent is the
case every single-agent algorithm assumes. The other factions are part of the
environment, and the environment is deterministic given the seed.

**Self-play brings non-stationarity, and it must be managed.** When two or
more seats hold learning policies, each one changes the environment of the
others. A single copy playing itself forgets how to beat its past selves.
AlphaStar's league is the reference remedy.[^10] Multi-agent PPO with a shared
critic is the reference for the cooperative case, and it is not this case,
because the factions compete.[^18]

**The four win paths need one scalar reward, and the choice is a design
decision.** Three forms are available.

1. A scalar per path, trained as four value heads, with the policy head fed
   by a weighted sum. The weights are a curriculum knob.
2. A single mixture: plus one for a win by any path, minus one for a loss, and
   zero otherwise.
3. A curriculum: train on one path with a shaped reward, then widen to all
   four.

The mixture is honest and sparse. The per-path form gives the learner four
signals and lets the evaluation ask which path it learned to take. A shaped
reward risks the failure in section 8, because the learner optimises the
shape and not the win.

**The score reader is a shaped reward for free.** The engine already returns
four running values per faction.[^1] The difference of a running value
between ticks is a dense signal that costs no new code. Use it with a small
weight, and turn it down over training.

### 4.2 Recommendation

Train one seat against the controllers. Use the mixture reward as the primary
signal, and the score differences as a shaped term with a weight that decays
to zero. Keep four value heads so the evaluation can attribute the win.
Plan the league, and do not build it until the policy beats every controller
weight setting.

## 5. Libraries

Each entry gives what this author read and where. A version number appears in
a footnote and not in the body, because a version decays.[^19] The claims
below hold for the pages read on 5 September 2026.

### 5.1 CleanRL

CleanRL holds one file per algorithm under the MIT licence.[^20] Its project
file pins Python below 3.11, pins one PyTorch release, and pins one Gymnasium
release.[^21] The most recent commits are from April 2026 and are
documentation fixes.[^22] The pins are the cost. Cachette targets a current
Python, and the pins would hold it back. The benefit is that the PPO file is
short enough to copy into this repository and own. Its author wrote the
invalid action masking study, and a masked variant exists as a separate
repository.[^6] It vectorises with Gymnasium synchronous vector environments
or with EnvPool. No statement about aarch64 exists on the page. As a pure
Python project it runs wherever PyTorch runs.

### 5.2 Stable-Baselines3 and sb3-contrib

Stable-Baselines3 and its contrib package are MIT, require Python 3.10 or
later, and ship pure Python wheels.[^23] [^24] The contrib package holds
MaskablePPO, which takes a mask function through an environment wrapper and
applies it at sampling and at evaluation.[^25] It also holds RecurrentPPO. The
two are separate classes, and this author found no class that combines masking
with recurrence; that absence is **unverified**. Vectorisation goes through
the library's own vector environment classes, which step sub-environments in
one process or across subprocesses. No aarch64 statement exists on the page;
a pure Python wheel runs wherever PyTorch runs.

### 5.3 RLlib

RLlib is part of Ray, Apache 2.0, with wheels for Python 3.10 to 3.14 and for
Linux aarch64.[^26] The current stack is the RLModule API, and an example
shows action masking through an observation dictionary with a mask key and a
subclass of the PPO module.[^27] The example states that it supports PPO
only.[^27] RLlib scales across machines. Its cost is the size of the
dependency and of the API surface. This author judges it a later choice, when
one machine is not enough.

### 5.4 TorchRL

TorchRL is MIT, requires Python 3.10 or later, and ships wheels for Linux
aarch64.[^28] It provides a masked categorical distribution that sets the
log probability of a masked action to negative infinity.[^29] It is a set of
primitives, not a trainer, so the loop is the user's code. It is the one
library here that publishes an aarch64 wheel of its own compiled parts.

### 5.5 PufferLib

PufferLib is MIT and claims training at millions of steps per second.[^30]
The package index lists a source distribution only and no wheel, with a
release from June 2025.[^31] The development branch is active, with commits
in August 2026.[^32] Its vectorisation writes observations, actions, rewards
and terminals into contiguous shared buffers across thousands of environment
instances, and its documented environments are written in C.[^33] That model
matches a Rust world well in principle: one Python call could step many
worlds and fill one buffer. This author could not find a documented path for
an environment that is neither C nor pure Python, and marks the fit with a
Rust environment behind PyO3 as **unverified**. No aarch64 statement exists.
A source distribution builds where a compiler exists, and whether the build
passes on aarch64 is **unverified**.

### 5.6 Sample Factory

Sample Factory is MIT, requires Python 3.8 or later, and ships a pure Python
wheel with a release from June 2023.[^34] The repository has commits in 2026,
and they are fixes.[^35] It runs an asynchronous sampler with many workers
and a learner, and states support for discrete, continuous and hybrid action
spaces.[^36] No action mask statement exists on the page. No aarch64 statement
exists.

### 5.7 Gymnasium and PettingZoo

Gymnasium is the single-agent interface. It is MIT and requires Python 3.10
or later.[^37] Its vector API defines a base class that a custom
implementation may subclass, with synchronous and asynchronous defaults and
an autoreset mode in the metadata.[^38] A custom vector environment that steps
every world in one Rust call is the right shape for Cachette. PettingZoo is
the multi-agent interface, MIT, Python 3.10 to 3.14.[^39] Its documentation
places the action mask in the observation dictionary for its own
environments and in the info dictionary for others, and tells a caller to
check both.[^40] Cachette should follow the observation dictionary form,
because MaskablePPO and the RLlib example both read it there.

### 5.8 EnvPool

EnvPool is a C++ batched environment pool with a thread pool, Apache 2.0,
with wheels for Python 3.12 to 3.14 including Linux aarch64.[^41] It requires
that the environment be written in C++ against its template. Cachette's
environment is Rust behind PyO3, so EnvPool would need a rewrite of the
binding. The value of EnvPool is its shape: one call steps every environment
on a thread pool and returns one batch. A Rust binding that does the same
thing gets the benefit without the dependency.

### 5.9 JAX stacks

PureJaxRL is Apache 2.0 and implements the entire training pipeline in JAX,
including the environment, so the whole loop compiles onto one device.[^42]
Jumanji is Apache 2.0 and ships environments written in JAX.[^43] Both gain
their speed because the environment step is a JAX function that runs on the
device beside the network. Cachette's step is Rust on the host. Each step
would cross from the device to the host and back, and the compiled loop would
break at every step. The JAX stacks give no benefit to this project.

### 5.10 Recommendation

Use Stable-Baselines3 with MaskablePPO from sb3-contrib as the first stack,
behind a custom Gymnasium vector environment that steps every world in one
Rust call. The fallback is a copied and owned CleanRL PPO file with masking
added, on TorchRL's masked distribution, when the SB3 abstractions get in the
way of the three-head policy. Move to RLlib when one machine is not enough.

## 6. Throughput on the target

### 6.1 Findings

**PyTorch publishes CPU wheels for Linux aarch64.** The package index lists a
wheel for each supported Python on the manylinux aarch64 tag.[^44] The
Graviton guide states that the official wheel installs with pip, and it gives
two environment variables that turn on bfloat16 fast math and huge pages on
Graviton3 and later.[^45] A vendor post reports up to 3.5 times the speed of
the prior release for one vision model on a Graviton3 instance.[^46] That
figure is theirs, for their model, and this author did not reproduce it.

**Inference per world is the wrong shape. Batch it.** A policy network of the
size in section 3 costs microseconds per forward pass when batched and
milliseconds when called once per world. The rollout loop should gather the
observations of every world into one tensor, run one forward pass, and
scatter the actions back. Every library in section 5 does this when the
environment is a vector environment.

**The step releases the lock, and that lets worlds step in parallel.** PyO3
releases the interpreter lock through a detach call, and the guide shows
Python threads then running Rust work in parallel.[^47] Cachette releases the
lock for the whole step by rule.[^16] So a vector environment can step its
worlds from a thread pool, or the Rust binding can step them from a rayon
pool inside one call. The second form is better, because it makes one Python
call per tick for the whole batch.

**How many worlds one process holds is a derived expectation.** The world
memory figure in the cost register was taken for a world without settlements
or characters.[^2] A training world will be far smaller than the target scale,
because a few thousand ticks of a 16.7 million tile world is not a training
budget. The number of worlds is the process memory divided by one world plus
one rollout buffer. This author gives no figure. The blocker governs every
cost figure in this project, and the figure that matters here, the frame cost
of a small world with settlements and a controller, is one the blocker names
as not yet measured.[^2]

**IMPALA's gain shrinks here.** Section 1 said this. The reason is that
asynchronous actors pay when the environment step is slower than the network
forward. When the step is a Rust frame over a small world and the forward is
batched on the same cores, the two are close, and a synchronous loop loses
little. This is a derived expectation, and a measurement on the target
platform would settle it.

### 6.2 Recommendation

Build the vector environment as one Rust call that steps a batch of worlds on
a thread pool. Run inference on the CPU in one batch per tick. Measure worlds
per process on a Graviton instance with the benchmark script, and put the row
in the cost register before any figure appears in a record.

## 7. Evaluation

### 7.1 Findings

**Win rate against the controllers is the first metric, and it saturates.** A
policy that wins every game against every weight setting gives no more
signal. A rating system against a pool of past checkpoints does not saturate.
TrueSkill generalises Elo, handles draws, and gives a rating with an
uncertainty.[^48] Draws matter here, because a game that reaches the tick
limit with no winner on the territory path is a draw. Elo suffices when the
pool is small and every match is one against one.

**Win rate per path is the metric that catches a degenerate policy.** Four
value heads and a per-path count in the report show whether the policy wins
by one path only. Section 8 says why that matters.

**The balance harness is already the evaluation runner.** It plays a fixed
seed set to game end, reads the game end record, the score per faction and
the census, and writes a deterministic report with no clock and no thread
count in it.[^4] An evaluation run is the same loop with one seat set to the
learned policy. The report gains one column, the controller of each seat, and
nothing else changes. The externally controlled flag that the design names is
the hook.[^1]

**Determinism gives exact replay.** One seed and one action log give one
game.[^1] The evaluation can store the log instead of the frames, and any
game can be replayed for inspection at the cost of one run. A rating that is
disputed can be recomputed from its logs. No other environment in this field
offers this at no cost, and the learner should keep every log.

### 7.2 Recommendation

Extend the balance harness with a seat controller column and a learned-policy
option. Report win rate per path and per opponent. Rate checkpoints with
TrueSkill against a pool that includes every controller setting and the last
several checkpoints. Store the action log of every evaluation game.

## 8. Risks

**A degenerate win path.** Reward hacking is the case where a learner
satisfies the objective by a loophole rather than by the intent.[^49] Four
win paths are four objectives, and the cheapest one wins. If the wealth
target is reachable by hoarding, the policy hoards and never fights. This is
not a defect in the learner. It is a balance finding, and the harness exists
to record it. The per-path win rate is the detector. The remedy is a balance
value, not a reward term.

**Exploiting the scripted opponent.** A stationary opponent has stationary
holes, and a learner finds them. A policy that beats the controller may lose
to a human or to another policy. The league is the remedy, and section 4
places it after the first phase.

**The sample cost of sparse terminal reward.** One bit of signal per game of a
few thousand ticks is expensive. The shaped term in section 4 and the warm
start in section 1 are the two remedies. Both carry a risk of their own: the
shaped term can be hacked and the warm start can bias. Decay the shaped
weight, and measure the warm-started policy against a cold one.

**Instability with many factions.** More factions make the outcome of a game
depend less on the one learning seat. The advantage estimate gets noisy, and
PPO's clip holds it only so far. Start with four factions and widen.

**Nondeterminism in the learner is acceptable, and here is why.** The learner
runs PyTorch on a thread pool, and its floating point sums reassociate. Two
training runs from one seed give two different networks. This does not touch
the engine's determinism, because the engine's hash covers the world state
and the event log, and the learner writes neither.[^16] The engine's guarantee
is that one seed and one action log give one game, and a nondeterministic
learner produces an action log that the engine then replays exactly. The two
determinism tests remain the engine's tests. A training run is a measurement
with variance, and the evaluation must report over several seeds.

## 9. The recommended first stack

**First choice.** Stable-Baselines3 with MaskablePPO from sb3-contrib, behind
a custom Gymnasium vector environment whose step is one Rust call that
advances a batch of worlds on a thread pool with the interpreter lock
released. The policy is a small square convolution over the level 1 map, a
perceptron over the flat vector, a frame stack of four, and three
autoregressive heads with a pointer over the candidate list. The mask is one
engine reader. Reward is the win mixture plus a decaying score difference
term, with four value heads. Behaviour cloning from the controller log warms
the start. The balance harness is the evaluator, with TrueSkill over a
checkpoint pool. The reasoning is that every part is pure Python over a
PyTorch wheel that exists for aarch64, the masking is a documented class, and
the one piece of new code is the vector environment, which this project needs
under any stack.

**Second choice.** A copied and owned CleanRL PPO file, with TorchRL's masked
categorical distribution for the heads, on the same vector environment. It
costs more code and gives full control of the three-head policy and the
recurrent buffer, which SB3 constrains. Take it when the first stack's
abstractions block the pointer head or the recurrent cell, and not before.

**Deferred.** Evolution strategies as a pipeline check. RLlib at more than
one machine. A league when the policy beats every controller. Planning with
the real engine, not a learned model, if a search ever pays.

## References

[^1]: Design, the living world game layer, sections 1, 5 and 10. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: The balance harness. `python/cachette/balance/__init__.py`
[^5]: Schulman, Wolski, Dhariwal, Radford and Klimov, Proximal Policy Optimization Algorithms, 2017. https://arxiv.org/abs/1707.06347
[^6]: Huang and Ontañón, A Closer Look at Invalid Action Masking in Policy Gradient Algorithms, 2020. https://arxiv.org/abs/2006.14171
[^7]: Espeholt and others, IMPALA: Scalable Distributed Deep-RL with Importance Weighted Actor-Learner Architectures, 2018. https://arxiv.org/abs/1802.01561
[^8]: Schrittwieser and others, Mastering Atari, Go, Chess and Shogi by Planning with a Learned Model, 2019. https://arxiv.org/abs/1911.08265
[^9]: Hubert and others, Learning and Planning in Complex Action Spaces, 2021. https://arxiv.org/abs/2104.06303
[^10]: Vinyals and others, AlphaStar: Grandmaster level in StarCraft II using multi-agent reinforcement learning, DeepMind, 2019, read 5 September 2026. https://deepmind.google/discover/blog/alphastar-grandmaster-level-in-starcraft-ii-using-multi-agent-reinforcement-learning/
[^11]: Kumar, Zhou, Tucker and Levine, Conservative Q-Learning for Offline Reinforcement Learning, 2020. https://arxiv.org/abs/2006.04779
[^12]: Salimans, Ho, Chen, Sidor and Sutskever, Evolution Strategies as a Scalable Alternative to Reinforcement Learning, 2017. https://arxiv.org/abs/1703.03864
[^13]: Vinyals, Fortunato and Jaitly, Pointer Networks, 2015. https://arxiv.org/abs/1506.03134
[^14]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^15]: Hoogeboom, Peters, Cohen and Welling, HexaConv, 2018. https://arxiv.org/abs/1803.02108
[^16]: Project orientation, the hard invariants. `CLAUDE.md`
[^17]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^18]: Yu and others, The Surprising Effectiveness of PPO in Cooperative, Multi-Agent Games, 2021. https://arxiv.org/abs/2103.01955
[^19]: Decision Record Scope, section 4.2. `.agents/rules/adr-scope.md`
[^20]: CleanRL repository, read 5 September 2026. https://github.com/vwxyzjn/cleanrl
[^21]: CleanRL project file, Python `>=3.8,<3.11`, torch `2.4.1`, gymnasium `0.29.1`, read 5 September 2026. https://github.com/vwxyzjn/cleanrl/blob/master/pyproject.toml
[^22]: CleanRL commit history, most recent commit 20 April 2026, read 5 September 2026. https://github.com/vwxyzjn/cleanrl/commits/master
[^23]: Stable-Baselines3 on the package index, release 2.9.0 of 15 June 2026, read 5 September 2026. https://pypi.org/project/stable-baselines3/
[^24]: sb3-contrib on the package index, release 2.9.0 of 15 June 2026, read 5 September 2026. https://pypi.org/project/sb3-contrib/
[^25]: sb3-contrib documentation, Maskable PPO, read 5 September 2026. https://sb3-contrib.readthedocs.io/en/master/modules/ppo_mask.html
[^26]: Ray on the package index, release 2.58.0 of 23 August 2026, read 5 September 2026. https://pypi.org/project/ray/
[^27]: RLlib example, action masking with an RLModule, read 5 September 2026. https://github.com/ray-project/ray/blob/master/rllib/examples/rl_modules/action_masking_rl_module.py
[^28]: TorchRL on the package index, release 0.13.3 of 14 July 2026, read 5 September 2026. https://pypi.org/project/torchrl/
[^29]: TorchRL documentation, MaskedCategorical, read 5 September 2026. https://docs.pytorch.org/rl/stable/reference/generated/torchrl.modules.MaskedCategorical.html
[^30]: PufferLib repository, read 5 September 2026. https://github.com/PufferAI/PufferLib
[^31]: PufferLib on the package index, release 3.0.0 of 23 June 2025, source distribution only, read 5 September 2026. https://pypi.org/project/pufferlib/
[^32]: PufferLib commit history, branch 4.0, most recent commit 20 August 2026, read 5 September 2026. https://github.com/PufferAI/PufferLib/commits/4.0
[^33]: PufferLib documentation, read 5 September 2026. https://puffer.ai/docs.html
[^34]: Sample Factory on the package index, release 2.1.1 of 19 June 2023, read 5 September 2026. https://pypi.org/project/sample-factory/
[^35]: Sample Factory commit history, most recent commit 2 July 2026, read 5 September 2026. https://github.com/alex-petrenko/sample-factory/commits/master
[^36]: Sample Factory repository, read 5 September 2026. https://github.com/alex-petrenko/sample-factory
[^37]: Gymnasium on the package index, release 1.3.0 of 22 April 2026, read 5 September 2026. https://pypi.org/project/gymnasium/
[^38]: Gymnasium documentation, the vector API, read 5 September 2026. https://gymnasium.farama.org/api/vector/
[^39]: PettingZoo on the package index, release 1.27.0 of 13 August 2026, read 5 September 2026. https://pypi.org/project/pettingzoo/
[^40]: PettingZoo documentation, the AEC API, read 5 September 2026. https://pettingzoo.farama.org/api/aec/
[^41]: EnvPool on the package index, release 1.2.6 of 28 August 2026, read 5 September 2026. https://pypi.org/project/envpool/
[^42]: PureJaxRL repository, read 5 September 2026. https://github.com/luchris429/purejaxrl
[^43]: Jumanji repository, read 5 September 2026. https://github.com/instadeepai/jumanji
[^44]: PyTorch on the package index, release 2.14.0 of 2 September 2026, manylinux aarch64 wheels for Python 3.10 to 3.14, read 5 September 2026. https://pypi.org/project/torch/
[^45]: AWS Graviton getting started, PyTorch notes, read 5 September 2026. https://github.com/aws/aws-graviton-getting-started/blob/main/machinelearning/pytorch.md
[^46]: PyTorch blog, Optimized PyTorch 2.0 Inference with AWS Graviton processors, read 5 September 2026. https://pytorch.org/blog/optimized-pytorch-w-graviton/
[^47]: PyO3 user guide, parallelism, read 5 September 2026. https://pyo3.rs/latest/parallelism.html
[^48]: Herbrich, Minka and Graepel, TrueSkill: A Bayesian Skill Rating System, 2007. https://www.microsoft.com/en-us/research/publication/trueskilltm-a-bayesian-skill-rating-system/
[^49]: Amodei and others, Concrete Problems in AI Safety, 2016. https://arxiv.org/abs/1606.06565
