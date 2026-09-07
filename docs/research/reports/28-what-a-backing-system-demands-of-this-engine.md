# What A Backing System Demands Of This Engine

Research report 28. It asks what the Cachette interfaces must look like so that
many different reinforcement learning backing systems can drive the engine.
Prepared 6 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The engine simulates a hex world, and it holds one property it cannot
recover once lost, which is determinism.[^1] A learner that plays one faction is
an accepted need, and the built-in controllers play the other factions.[^2]

A backing system is whatever drives the learning. It may be a vectorised
environment interface, a distributed actor and learner framework, a batched
simulator harness, an offline dataset pipeline, an evaluation harness, or a
plain research script. An earlier report surveys the algorithms and the
libraries.[^3] This report asks the follow-up question. It surveys the demands
that a backing system makes, and not the products that make them, because a
product changes and a demand does not.

## 0. Provenance, and what this report could not verify

**This author ran nothing.** No build, no test, no benchmark and no training
run. Other agents held the machine. Every claim about the code comes from
reading the tree on the `integration` branch.

Every performance figure in this report comes from the target platform cost
register, which holds the machine, the commit and the command for each
row.[^4] A figure this author derived from those rows is marked derived. One
blocker governs every cost figure in this project, and it names the cost of a
call across the language boundary as one of the figures nobody has
measured.[^5]

Two documents already cover ground this report does not repeat. One measures
the boundary as a general engine.[^6] One is a design that states the interface
shape the project means to build.[^7] This report treats both as given and adds
what they do not hold.

## 1. The conclusion

**The engine meets almost none of these demands today, and the reason is
absence rather than conflict.** No reader takes a viewing faction. No call
returns a schema. No call steps more than one world. No call reports the seed
that built the world. Nothing serialises a world. Each of these is a thing that
was never written, and not a thing that was written the wrong way. That is the
best position a project can be in, because the accepted records already state
the right shapes and no code contradicts them.[^8] [^9]

**The single largest demand this engine can satisfy, and almost no other
environment can, is exact reproducibility across processes and machines.** Most
environments cannot promise a bit-identical replay. This one can, because one
seed and one command sequence give one world at any thread count.[^1] That turns
the hardest demand a distributed backing system makes, which is moving a world
from one process to another, from a serialisation project into a replay. The
project should expose the parts of that promise a stranger can hold: the seed,
the configuration, the schema version, the action log and the state hash.

**The batch call is right, and the usual argument for it is wrong.** A batch
call is justified by ordering and by thread ownership, not by the cost of a
Python call. The measured frame cost at small extents is in the range of
hundreds of microseconds to a few milliseconds, and a boundary crossing is far
below that.[^4] The reason to step N worlds in one call is that the engine must
own the thread pool and must write each result into the row its index names.
Python cannot do either without giving the ordering to the operating
system.[^9]

## 2. The demands

Each demand below gets three answers: whether the engine meets it today, what
it would cost to meet, and whether meeting it endangers determinism.

### 2.1 Batching and vectorisation

**The demand.** One process holds many worlds. One call steps all of them. The
observations return as one array whose first axis is the world index. A
high-throughput backing system will not tolerate a Python call for each world,
and a shared-buffer sampler will not tolerate a Python object for each world at
all.

**Today.** The engine allows many worlds in one interpreter, and no
process-wide mutable state reaches simulated state.[^10] The step releases the
interpreter for its whole duration.[^11] But the step advances one world by one
tick for each call, so the crossing count follows the world count times the tick
count. No batch type exists.

**The cost.** An accepted record states the batch call, and no code implements
it.[^9] The work is a Rust type that holds N worlds, four batch calls, and a
thread pool inside the call. The record fixes neither N nor the thread count.

**The determinism answer.** This is the demand with the largest determinism
surface in the group, and the record already closes it. A batch call must write
each world's result into the row its index names. A call that returned results
in completion order would give an answer the schedule chose. The engine's tests
must run one batch step at more than one thread count.

**What this report adds.** The usual argument for batching is that a Python call
is expensive. Check that argument against the measured rows. One frame of a
4,096-tile world with no units costs 1.74 milliseconds at one thread and 216
microseconds at four threads.[^4] A crossing is not measured, and a derived
estimate of a few microseconds puts it at a small share of either figure. So the
throughput argument for batching is weak at the extents a training run will use.
The ordering argument is strong and does not depend on any figure. State the
batch call on the ordering argument. A reader who believes the throughput
argument will abandon the batch call the first time a measurement disappoints
them.

### 2.2 Observation

**The demand.** A fixed shape, a fixed integer width, and a fixed bound for each
position. The layout must not move for the life of a training run, because the
first layer of a policy is that layout. A backing system builds its input space
once, at construction.

**Today.** The engine meets none of this. The word for an observation does not
appear in the binding at all. The readers that exist return dictionaries, and a
dictionary states neither a length nor a bound. No reader takes a viewing
faction, so every reader answers with the truth about the whole world.

The core does hold a complete fog module.[^12] It carries what a faction sees
now, which the step derives, and what a faction saw once, which the step carries
forward and the hash covers. An accepted record governs its storage.[^13] None
of it reaches Python.

**The cost.** An accepted record states the observation table and its
schema.[^8] The reader must read aggregates the engine already keeps and the
summary the pyramid already rebuilt. It must start no pass over the tiles or the
units. The expensive part is the faction scoping, because every block of the
observation must pass through the fog rule.

**The determinism answer.** A reader that touches no state risks nothing. The
one hazard is a reader that walks a container whose iteration order is not
fixed. The fog layers are block arrays over a lattice, so a walk in block order
is stable by construction.

**What this report adds. The observation length is a world parameter, and that
makes the boundary cost a dial the researcher holds.** The map block of an
observation follows the level 1 cell count. At the target extent, with the
default block edge, a world holds 16,384 cells.[^4] At 4,096 tiles it holds
four. So the same interface gives an observation of hundreds of kilobytes at
target scale and of a few hundred bytes at a training extent. **Derived: at the
target extent, six fields per cell at eight bytes each give about 786 kilobytes
for one observation, and a thousand worlds then move about 786 megabytes for one
read.** That is the copy cost the boundary would pay, and it dwarfs any crossing
cost. It is also entirely avoidable, because a training run picks the extent.
State this in the reference material, so that nobody trains at target scale by
accident.

**The hard constraint stands above all of it.** A learner must never see what a
player of that faction could not see. The product record states it and the
accepted record binds the reader to it.[^2] [^8] The interface must offer no
argument that asks for the truth. An argument that can be turned on will be
turned on.

### 2.3 Actions

**The demand.** A space a library can describe in one object. A legality mask
per world per decision. A defined outcome for an illegal action.

**Today.** The engine has verbs that a caller and the controller both use.[^14]
It has no action table, no schema and no mask. A caller composes arguments by
hand.

**The cost.** An accepted record states a bounded table that factorises into a
verb, a target and a magnitude by a mixed radix.[^8] The verb rows derive from
the controller's own choice enumeration, so the engine gets the verb set from a
table it already maintains. The legality byte is the expensive part, because it
must read the same candidate lists and refusal rules the verbs read, and
duplicate no rule. A duplicated rule is the first defect shape this project
names.[^15]

**The determinism answer.** Two hazards. The candidate list order must come from
a stable key, or the meaning of a target index moves between runs.[^16] A batch
of actions must apply in an order the data fixes, and never in the order a
caller wrote the rows.

**What this report adds.** Masking is not an optimisation here. It is the
difference between a tractable problem and an intractable one. The invalid
action masking study finds that the share of invalid actions grows with the
space.[^3] This action space is a product of a verb count, a candidate bound and
a bucket count, so it is large and mostly invalid at any tick. Without the mask
a learner spends most of its decisions on refused rows, and the refusal teaches
it nothing except that the row was refused.

**Say what happens to an illegal action, in the interface and not in a
convention.** The accepted record says a refused row is dropped and counted, in
the way a controller command is.[^8] That is the right answer, and a backing
system must be able to read the count. A run whose refusal count rises is a run
whose mask and whose verbs disagree, and that is a defect the record already
makes testable.

### 2.4 Episodes

**The demand.** Something ends an episode. Something computes a return. A
vectorised interface also demands an answer to a question a single environment
never asks: what happens to the world that finished while the others continue.

**Today.** The engine records a game end once and stops the controllers.[^17] It
reads the standing of a faction on each win path.[^17] It computes no reward and
no episode boundary of any other kind.

**The cost.** Low for the reading. Both readers exist. The work is to give them
a fixed shape rather than a dictionary.

**The determinism answer.** The reward must stay outside the engine, and the
reason is sharper than convenience. If the engine computed a reward, somebody
would store it. A stored value that the step reads enters the state hash. Two
experiments that differ only in a reward weight would then produce different
world hashes, and a golden hash test would fail for a reason that has nothing to
do with the simulation. **Keep the reward a Python function of the standing, and
the hash stays a statement about the world.**

**What this report adds. The autoreset convention is a determinism hazard that
nobody has written down.** A vectorised environment interface conventionally
resets a world that finished, inside the same step call, and moves the final
observation into an auxiliary channel.[^18] If the engine does that, the engine
chooses the seed of the new world. A seed the engine chooses is a seed no caller
recorded, and a run then stops being reproducible from values a caller holds.

The remedy is small and must be decided before the batch call is written. **A
caller supplies the seed sequence, and the engine consumes it in index order.**
A batch takes a list of seeds for each world, or a caller resets the finished
worlds itself in a separate call. Either is reproducible. An engine-owned
counter is not.

**Some backing systems never reset mid-episode**, and an offline dataset
pipeline never resets at all. So autoreset must be a mode a caller selects, and
not the only behaviour. The safe default is the one that does nothing: a
finished world reports that it finished, and it stops advancing until a caller
resets it.

### 2.5 Reset and state

**The demand.** Four separate demands hide under one word. Reset from a seed.
Reset to a saved state. Save and restore in the middle of an episode. Two
restored worlds that are bit-identical.

**Today.** The engine builds a world from a width, a height, a seed and a
faction count. It reports none of those back. The agent session store keeps them
in Python for that reason, and its own docstring says so.[^19] The binding holds
no save, no load, no snapshot, no restore and no clone. The registry reclaimed
its reserved rows for a snapshot and for a save format, because nothing asked
for either.[^20]

**The cost.** The four demands cost very differently.

- **Reset from a seed is met, and it is cheap.** Building a world costs 276
  nanoseconds for each tile at every extent measured.[^4] **Derived: a
  4,096-tile world builds in about 1.2 milliseconds, which is under one frame of
  the same world at one thread.** A training run at a small extent can reset
  freely.
- **Reporting the seed back is nearly free**, and it is the first thing to do.
  A world that states the values that built it is a world a caller can write to
  a file and rebuild.
- **Reset to a saved state has no mechanism**, and the honest substitute already
  exists. A seed and an action log rebuild a world exactly. That is a portable
  state format the project gets for nothing.
- **Save and restore in the middle of an episode is a real project.** Every
  simulated structure is already plain data with declared padding, because the
  event rule and the hash rule both demand it, so the ground is prepared. The
  cost is that the format becomes a compatibility surface that every later
  change must respect.

**The determinism answer.** None of this endangers determinism, and the existing
hash test polices all of it. A save that round-trips to a different hash is a
defect the golden hash test finds at once. That is a better starting position
than most engines have.

**What this report adds. Replay is the state format, and a snapshot is only an
optimisation of it.** The cost of restoring by replay grows with the length of
the episode. The cost of restoring from a snapshot does not. So the question is
not whether to build a snapshot. It is at what episode length replay stops being
affordable, and that is a measurement nobody has taken. Do the cheap thing
first, name the measurement, and build the snapshot when the measurement asks
for it.

**A tree search demands the expensive version.** A backing system that plans
with the real engine forks the world at a decision point, many times per
decision. Replay cannot serve that. A cheap clone can. This is the one backing
system family the replay substitute does not reach, and the earlier report
already places planning as a later option.[^3]

### 2.6 Determinism and reproducibility as a feature

**The demand.** A researcher wants to hand a colleague a value and receive the
same numbers back. Most environments cannot offer this. Their randomness is
thread-local, their aggregates are floating point sums, and their results depend
on the thread count.

**Today.** The property holds inside the engine and reaches nobody. The state
hash is one integer that says two runs agree. The world reports no seed. No
schema exists to version. No action log exists to store.

**The cost.** Low, and mostly mechanical. Expose five things and the promise
becomes something a stranger can hold.

1. **The seed, the extent and the faction count**, read back from the world.
2. **The whole configuration by name**, through a schema rather than through
   thirty-three separate setters.[^6]
3. **The schema version, with a digest of the schema beside it.** A stored
   policy records the version it trained under. A learner that loads a policy
   under a different version stops with an error that names both.[^7]
4. **The action log**, as one event type that holds the tick, the faction and
   the action, whatever chose it.[^8]
5. **The state hash**, which already exists.

**The determinism answer.** Exposing these endangers nothing. Every one of them
is a read.

**What it is worth.** Three things, and they are worth stating because the
project may undervalue them.

- **A disputed result is settleable.** A rating computed from logs can be
  recomputed from those logs by anybody.
- **A failing episode is reproducible.** A researcher who finds a pathological
  game keeps the seed and the log, and every later investigation replays that
  exact game.
- **An evaluation costs storage rather than compute.** A log is smaller than the
  frames it produces, and a game replays for the cost of one run.

This author judges reproducibility the strongest single argument this engine has
against an established environment. It should be advertised in the interface,
and not only satisfied inside it.

### 2.7 Throughput

**The demand.** Steps per second, and an account of where the time goes. A
backing system that samples on policy needs a large number of environment steps
for each gradient step.

**Today, with measured figures.** The register holds these, and each row names
the machine, the commit and the command.[^4]

- One frame at the target extent with a million units costs 177.9 milliseconds
  at 12 threads. **Derived: about 5.6 ticks per second for one world.**
- One frame of a 4,096-tile world with no units costs 1.74 milliseconds at one
  thread. **Derived: about 574 ticks per second for one world on one core.**
- The measured world holds no settlement and no character, so every frame figure
  is a lower bound.[^5]
- A world at the target extent holds 545 megabytes resident at one thread and
  876 megabytes at 12.

**What follows, and it is the most important thing in this section. A training
run cannot use the target scale.** At 5.6 ticks per second one game of a few
thousand ticks takes several minutes for one world, and a training run needs
many millions of decisions. At a small extent the engine is fast enough:
**derived, a 16-core machine running one small world per core gives on the order
of nine thousand ticks per second.** The interface must therefore treat the
extent as a parameter that a training run turns down, and no part of the
interface may assume the target scale.

**Where the time goes at scale.** The frame splits into work that follows the
tiles and work that follows the population. The tile work takes threads almost
perfectly. The population work barely takes them at all. Work that follows the
lattice is small enough not to need threads. That is measured on one machine at
one extent.[^4]

**What the interpreter lock costs is not measured, and the register says so.**
The lock is released for the whole step, so no Python code runs while a world
steps.[^11] Every other call holds the lock, including the frame drawing call
and the plan solver.[^6] For a training run that matters only if the loop calls
one of those.

**What a crossing costs is not measured either**, and the blocker names it.[^5]
**This report recommends measuring it, because it decides how small a world can
usefully get.** If a crossing costs a few microseconds, a world of 4,096 tiles
at four threads spends a few percent of its step at the boundary, and a smaller
world spends more. That figure sets the floor on a useful training extent, and
no other measurement gives it.

### 2.8 Process and transport

**The demand.** A backing system places the environment somewhere. In the
learner's process, in a subprocess, behind a socket on another machine, or
behind a shared memory buffer. Each choice costs something and forbids
something.

**Today.** Only the in-process form exists. A server that holds named worlds
across separate calls exists for a different audience, and it shows the
out-of-process shape works.[^21]

**What each choice costs and forbids.**

- **In-process.** The cheapest. It forbids nothing this design needs. It costs
  the risk that a world that fails takes the learner with it.
- **Subprocess.** It costs one serialisation of the observation and the action
  for each step. It buys fault isolation and it removes the interpreter lock as
  a shared resource. It forbids a caller from holding a reference into engine
  memory.
- **Socket.** It costs the subprocess cost plus the network. It buys placement
  on another machine. It forbids any interface that returns a handle, because a
  handle is meaningless across a socket.
- **Shared buffer.** It costs a fixed allocation and a discipline about who
  writes when. It buys the lowest per-step cost at a large world count. It
  forbids a caller from holding a view across a step, because the engine
  overwrites the buffer.

**What this report adds. A socket forbids nothing in this design, and that is a
property worth protecting.** Everything the accepted records ask to cross is a
fixed-length array of integers keyed by world index.[^8] [^9] An array of
integers travels over a socket unchanged. So the same four calls serve every
transport, and only the encoding changes.

**One interface choice would destroy that property, and the project must not
make it.** An interface that returns an opaque handle, or that takes a Python
callable the engine invokes, works in-process and cannot cross a socket. The
crate split already makes a callback inside the step a compile error, so the
engine is protected there.[^22] The Python layer is not protected, and it should
carry the same rule by convention: **the environment core returns arrays and
never handles.**

**A shared buffer needs one more rule.** A record already says that a call
declares whether it copies or borrows.[^23] A borrowed view into engine memory
is a correctness hazard across a step, and the declaration is what makes it
safe. A shared-buffer transport must state, at the call site, that the caller
reads the buffer before the next step and not after it.

### 2.9 Multi-agent

**The demand.** Several factions learning at once. Self-play against past
checkpoints. A scripted opponent that plays well enough to be worth beating.

**Today.** The engine plays every faction with a controller that runs inside the
step and acts only through the caller's verbs.[^14] A flag stops the controller
for one faction. This is an asset most environments lack. A researcher who wants
a non-trivial opponent usually has to train one first.

**The cost.** Low for the shape, because the accepted records already carry a
faction axis. An observation is per faction. An action names a faction. A batch
read is shaped by the world count, the learner count and the observation
length.[^7] So two learning seats cost one more row, not one more mechanism.

**The determinism answer.** The engine is safe here, because every command
applies at the frame barrier and no seat sees another seat's action before it
acts. The risk sits in Python: a loop over seats that reads a hash map and
applies in its iteration order would give an order nothing fixed. Apply by
faction index.

**What this report adds, in three parts.**

**The engine gives simultaneous action, and that fixes which multi-agent
convention fits.** The parallel convention, in which every agent acts at once
and the environment steps once, matches the engine exactly. The turn-based
convention, in which agents act one at a time and the environment steps between
them, does not.[^24] Section 4 refuses it.

**A frozen checkpoint as an opponent needs no engine work at all.** It is a
policy that does not learn. It reads the same observation and returns the same
action array. The engine cannot tell the difference, and it should not be able
to.

**The controller is more than an opponent. It is a labelled dataset.** The
controller chooses an action every tick. If the engine writes the controller's
choice and the learner's action to one log in one encoding, then behaviour
cloning reads that log with no extra code and no extra cost.[^8] That is the
cheapest warm start available to this project, and it exists only if the two
share the encoding. **Two logs would destroy it**, and two logs is what a
contributor would reach for.

## 3. What this engine meets today

| Demand | Met today | Where the gap is |
|---|---|---|
| Many worlds in one process | Yes | No batch call |
| One call steps many worlds | No | An accepted record, no code |
| Interpreter released for the step | Yes | Other calls hold it |
| Fixed-shape observation | No | Readers return dictionaries |
| Faction-scoped observation | No | Fog exists in the core only |
| Action space a library can describe | No | No table, no schema |
| Legality mask | No | Nothing computes it |
| Episode end | Partly | A game end record exists |
| Reward | No, by design | It belongs in Python |
| Reset from a seed | Yes | The seed is not read back |
| Reset to a saved state | No | No save, no snapshot, no clone |
| Bit-identical replay | Yes | No action log to replay from |
| Out-of-process transport | Partly | A named-world server exists |
| A scripted opponent | Yes | Its choices reach no log |
| Several learning seats | Partly | The records carry the axis |

## 4. The smallest set of changes, in order

Each item states what it buys and what it costs. The order is chosen so that
each item can ship alone and each one makes the next cheaper.

**One. Report the seed, the extent and the faction count.** A world states the
values that built it. **Buys:** a run reproducible from values a caller writes
to a file, and it deletes a duplicate that Python holds today.[^19] **Costs:**
almost nothing. Four readers. **Do this first because it is the cheapest thing
in this report and because every other item assumes it.**

**Two. The observation schema and the faction-scoped observation reader.**
**Buys:** the whole read side of every backing system, in one mechanism. It is
also the only path by which fog reaches Python, and nothing else drags it
across. **Costs:** the reader must pass every block through the fog rule and
must start no pass over the tiles or the units. **Endangers determinism:** only
through the walk order of the fog layers, which the block lattice fixes.

**Three. The action schema, the action table and the legality byte.** **Buys:**
the write side. A library builds a discrete space from the schema and masks it.
**Costs:** the legality byte must not restate a rule that a verb owns, and a
test must compare the two over a seed set. **Endangers determinism:** through
the candidate list order, which must come from a stable key.

**Four. One action log for the controller and the learner, in one encoding.**
**Buys:** replay, behaviour cloning and a golden test, all from one event type.
**Costs:** the controller's existing log changes shape and every reader of it
changes. **Endangers determinism:** nothing. It is an event, and events are
already plain data with declared padding.

**Five. The batch call over N worlds.** **Buys:** every high-throughput backing
system, and it moves the ordering decision into the engine where a test can see
it. **Costs:** a Rust type, four calls, and a thread pool inside the call.
**Endangers determinism:** most of anything here, and the accepted record
already states the rule. Test it at more than one thread count.

**Six. The configuration schema, replacing the separate setters.** **Buys:** an
experiment a researcher writes as one value and reproduces. **Costs:** low and
mechanical. It needs a check that fails when a new separate setter appears, or
an agent will add the next one. **Endangers determinism:** nothing.

**Everything after this is optional, and each item serves one family.** A
snapshot serves a tree search. A shared-buffer transport serves one sampler
family. A socket transport serves a cluster. None of them changes the shape of
the six items above, which is the test that the six are the right six.

## 5. What I would refuse to support

Supporting everything is how a boundary becomes unusable for everyone. This
section is the most valuable part of this report. It does not repeat the
refusals of the general boundary report, which cover a caller-composed stage
list, a Python callback inside the step, a floating point observation and a
scripting language for content.[^6] These are the ones a backing system asks for
specifically.

**A step that runs until something interesting happens.** A researcher will ask
for a variable step, so that the learner acts only at decision points. Refuse
it. A data-dependent tick count breaks the lockstep of a batch, because one
world of the batch decides how long every other world waits. It also makes the
cost of a batch step depend on the content of the worlds, which no caller can
predict. Worst of all, "run until interesting" is a convergence test wearing
another name, and a convergence test inside a step is one of the shapes this
project forbids.[^16] **Offer a fixed decision cadence instead.** A caller
states K ticks for each decision, and every world in the batch takes the same
K.

**An engine-owned autoreset.** The engine must not choose the seed of a world it
restarts. A seed the engine chooses is a seed no caller recorded, and the run
stops being reproducible from written values. Take the seed sequence from the
caller.

**An engine-computed scalar reward.** A reward is a rule of the downstream game,
and a blocker holds those rules.[^25] The deeper reason is the hash. Somebody
would store a computed reward, and a stored value the step reads enters the
state hash. Two experiments that differ only in a reward weight would then
disagree on the world hash. Keep the reward in Python, where it enters nothing.

**A per-world Python object as the primary surface.** Every library asks for
this first, and it is the easiest thing to give. Refuse it as the primary
surface. If a per-world object is the real interface, the batch call becomes an
optimisation that nothing uses, and the ordering guarantee lives in a path
nobody takes. **Give the single-world convenience as a wrapper over a batch of
one, and never the reverse.** The direction is the whole decision.

**A view into engine memory that survives a step.** A borrowed array is the
right answer for a large read, and a caller who holds it across a step reads a
world that changed underneath them. Refuse the version with no declaration. A
call that borrows says so at the call site, and the reference material carries
the rule.[^23]

**Pixels as an observation.** A researcher will ask for a rendered frame,
because many environments offer one. Refuse it for training. The drawing call
holds the interpreter while it fills a whole frame, so two worlds cannot render
at once.[^6] A rendered frame is also a lossy encoding of numbers the engine can
hand over directly. Keep the drawing for a human watching a game.

**A debugging flag that shows a learner the truth.** This is the most dangerous
request in the list, because it sounds reasonable. An argument that asks for
full information will be left on by accident, and a policy trained on the truth
is not the policy of a player. The owner has already ruled on this.[^8] The
full-information readers exist for the modeller and for the reproducer, and the
learner's path must not reach them. **Keep the two paths disjoint and check it.**

**A host callback adapter for the device-resident stacks.** Some frameworks
compile the whole training loop, including the environment step, onto one
device. Their speed comes from that compiled loop. This engine steps on the
host, so a callback would break the loop at every step. Refuse to build the
adapter. The harm is not the wasted work. The harm is that somebody would then
benchmark the engine through it, measure a slow environment, and conclude the
engine is slow. **Record the refusal and its reason where a survey would
otherwise put a row.**

**The turn-based multi-agent convention.** Some multi-agent interfaces step the
environment between agents, one agent at a time.[^24] This engine applies every
command at the frame barrier, so no seat can act after seeing another seat's
action within one tick. An adapter that pretended otherwise would either lie
about the ordering or would step the world once per seat. Support the
simultaneous convention only, and say so in the interface.

**A learner that acts through a path a caller cannot use.** A backing system
will eventually want a faster route into the world than the verbs. Refuse it.
The product record states that a learner makes only the acts a caller can
make.[^2] The moment a learner has a private path, the game a policy learned is
not the game a player plays, and the evaluation stops meaning anything.

## 6. One interface, or two

**The judgement: one engine surface, and two Python conveniences over it. The
direction of the wrapping is the whole decision.**

**The argument for two surfaces is real, and it fails on inspection.** A plain
research script wants one world and a step function. A distributed system wants
thousands of worlds, fault isolation, and a world it can move between processes.
Those look like different interfaces. Ask instead what engine calls each one
needs. Both need a schema, an observation, a mask, an action and a step. Neither
needs a call the other does not. The difference between them is the world count
and the transport, and both of those are parameters of the same calls.

**One property makes this possible, and it is not an accident.** Everything the
accepted records ask to cross is a fixed-length array of integers keyed by world
index.[^8] [^9] That representation is transport-neutral. It works in a process,
in a pipe and over a socket without changing. The interface would split only if
one audience needed something the array cannot carry, such as a handle or a
callback, and neither does.

**The single-world convenience must wrap the batch, and not the reverse.** A
research script deserves a plain step function, and building a batch of one to
index row zero is a usability tax that a measured report on this boundary would
count against the project.[^26] So give the script a wrapper. Write it as
roughly thirty lines of Python over a batch of one. Written the other way, with
the batch built out of per-world objects, the batch loses the two things that
justify it: the engine's ownership of the thread pool, and the write-by-index
ordering.

**Name the one place the two audiences genuinely disagree, and settle it.** A
research script wants a failing world to raise an exception at once. A
distributed system wants a failing world to report through its own row and the
other worlds to continue. These are opposed. The accepted record settles it in
favour of the distributed system, because a failure reports through its own
row.[^9] That is right, and the single-world wrapper then raises when row zero
carries a failure. The wrapper turns a row into an exception in one line. The
reverse conversion, from an exception into a row, loses the other worlds.

**The addition a distributed system needs is not a second surface. It is a
portable world identity.** A seed, a configuration, a schema version and an
action log. Together they let a process rebuild a world another process was
running. That is one item added to the same surface, and section 4 places it
first because it is also the cheapest thing in this report.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^3]: Research report 22, reinforcement learning approaches for one faction. `docs/research/reports/22-reinforcement-learning-approaches.md`
[^4]: Target platform costs. `docs/reference/graviton-costs.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Research report 27, what the Python boundary would have to be for a stranger. `docs/research/reports/27-the-boundary-as-a-general-engine.md`
[^7]: Design, one environment core serves every learning stack. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^8]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^9]: ADR-0155, a batch of worlds steps in one call, in index order. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^10]: ADR-0047, many worlds live in one interpreter. `docs/adrs/draft/adr-0047-many-worlds-live-in-one-interpreter.md`
[^11]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^12]: The observation module of the core crate. `crates/cachette-core/src/observation.rs`
[^13]: ADR-0059, fog storage grows with observed area, not with world area. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^14]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^15]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^16]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^17]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^18]: Gymnasium documentation, the vector API and its autoreset modes, read 6 September 2026. https://gymnasium.farama.org/api/vector/
[^19]: The agent session store. `python/cachette/agent/session.py`
[^20]: ADR Registry, the reclaimed rows for a snapshot and for a save format. `docs/adrs/REGISTRY.md`
[^21]: The agent server. `python/cachette/agent/server.py`
[^22]: Project orientation, the hard invariants, rule 4. `CLAUDE.md`
[^23]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/draft/adr-0044-what-copies-and-what-does-not-is-declared-at-the-call-site.md`
[^24]: PettingZoo documentation, the agent-environment-cycle interface, read 6 September 2026. https://pettingzoo.farama.org/api/aec/
[^25]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^26]: Research report 20, what the Python interface should be. `docs/research/reports/20-the-python-interface.md`
