# ADR-0194: A generation is scored one episode at a time, and combined in candidate order

## Context

The control plane trains a policy with an evolution strategy. One generation
holds a population of candidate policies and a set of seeds, and it plays one
episode for each pair of the two. Every one of those episodes is independent.

The trainer plays them in one process. It builds a batch of worlds and steps
the whole batch in one call, and the batch orders its results by the index of
the world.[^1] Between two decisions, the process stacks the observations,
builds the action masks and runs the policy matrix products. **That section
runs in one interpreter, and every engine worker of that process waits for
it.** A worker count therefore stops buying throughput well below the core
count of the target server, and a measurement on the target states the size of
the effect.[^2]

More threads inside one world step do not help either. A measurement of the
engine alone found one thread faster than sixteen at every extent this project
trains on.[^5] **So there is no engine parallelism to give up inside a
worker**, and the machine fills with processes rather than with threads.

A worker process does not need the weight vector of a candidate. The trainer
builds a candidate as the centre plus or minus one perturbation, and it draws
the perturbation from a generator keyed on the run seed and the generation
number. A process that holds the centre, the generation number and the
candidate index it owns rebuilds that candidate exactly.

**The episodes of one generation differ greatly in length.** A game ends when
one faction wins or when the tick limit stops it, and the audit of a paid run
found the ends spread over more than a factor of two. A process that owns a
block of the population therefore waits on its own longest game while its
cores idle.

**Determinism is the property this project cannot recover.** The engine gives
one answer at any thread count, and no result may take its order from which
worker finished first.[^3] A trainer that combined the returns of several
processes in the order the processes answered would break that rule outside the
engine, where no golden state hash watches. The weights of a run would then
depend on the load of the machine, and no run could be repeated.

## Decision

**One task is one episode. A pool of worker processes takes the tasks from one
queue, and the combination is ordered by the candidate index.**

### D1. The worker count changes the spread of the work and nothing else

For one centre, one generation number, one seed set and one population, the
weights the run reaches are identical at every worker count, position for
position.

A reviewer finds a violation when a run at one worker count and a run at
another reach different weights from one seed.

### D2. A worker rebuilds its candidate from the seed, and never receives it

A worker receives the centre, the generation number and the candidate it owns.
It draws the perturbations of the whole generation from the run seed and the
generation number, and it takes the row of its own candidate. No candidate
policy crosses to a worker.

The draw is a function of the run seed and the generation number alone. A
stream advanced by each generation would give a resumed run different
perturbations from the run it continues, and it would give a worker different
perturbations from the trainer.

The search scales that draw by the layers of the centre, so that it revises
each layer of a policy by the same fraction of what that layer holds. The
centre reaches the worker, so the worker derives the same scaling the trainer
derives and no second input joins the draw.

### D3. The combination sorts on the strategy, the candidate and the seed

Each result reports the strategy it played for, the candidate index it started
at, and the position of its seed in the set of the generation. The combination
sorts on those three and builds the score array in that order. **Nothing reads
the order in which the workers answered.**

An episode is a pure function of the policy and the seed. That is what makes a
queue admissible: the worker that takes an episode, and the moment it takes it,
reach no part of the answer.

A reviewer finds a violation when a combined array is built from an iteration
over futures as they complete, or from any order other than that key.

### D4. An episode that fails ends the generation

The combination refuses a set of results that does not cover every candidate of
the population on every seed exactly once, and it names what is missing. A
worker that raises carries its failure to the caller, and the run ends.

A partial generation must never be scored. A run that trained on a subset of
its population would report a generation that it did not play, and no register
would hold the difference.

### D5. One number names the pool, and it takes the core count by default

The caller names the worker processes of the queue, and nothing else. A caller
that names none takes the cores of the machine.

**This reverses an earlier decision of this record**, which had the caller name
a process count and a per process worker count, neither derived from the
machine. That shape let a run name a split that the trainer could not honour,
and let a run override the cores of a pass that no queue splits. A number that
nobody can state wrongly is worth more here than a number that is stated once.

The engine runs one thread for the world of one episode, whatever else a
configuration says, because one episode holds one world.

An argument that no longer means anything ends the run and names what replaced
it. A flag that is accepted and ignored is worse than a flag that is gone.

### D6. A worker process runs one matrix thread

The mechanism that starts the worker processes holds each matrix library to one
thread. A library that reads no such setting starts one thread for each core,
in every process, and those threads take the cores the engine needs.

### D7. One queue holds every strategy, and no barrier joins two of them

A run trains several strategies at once, and every one of them submits its
episodes into the one queue. A generation of one strategy waits for the
generation before it, and it waits for no episode of another strategy.

A reviewer finds a violation when the run waits for every strategy at a
generation boundary. That wait would leave the machine idle for as long as the
slowest strategy of the generation, which is the cost this decision removes.

## The alternatives this rejects

**Give each worker process a block of the population.** This is what the
project did. A block ends when its own slowest episode ends, so every block
waits on its longest game, and the block count cannot pass the number of
antithetic pairs. A run that asks for more processes than the population holds
pairs then leaves processes with no work at all.

**Send each candidate's weights to the worker.** This is the obvious shape and
it is rejected for cost, not for correctness. A population of many candidates
over a wide policy sends a large array on every generation, and it grows with
the width of the policy. The seed sends two integers instead.

**Combine the returns as the workers answer.** This is faster to write, and it
makes the weights of a run a function of the load of the machine. The
determinism record forbids it.[^3]

**Let one process open more engine workers.** The serial section between two
decisions belongs to one interpreter, so more workers inside one process do not
fill the machine.

**Keep a knob for the pool size and a knob for the threads of a worker.** Each
one names a choice that the measurements have already made. A knob that cannot
be right invites the misconfiguration this record exists to remove.[^4]

## Consequences

The parallelism of a generation is bounded by its episode count rather than by
its pair count, which is half the population.

A league run seats several candidates in one world. Such a group is the one
task that holds more than one episode, because a task that held part of a group
would build a world with a seat that no worker filled.

The project now cannot combine a generation by any key except the strategy, the
candidate and the seed, and it cannot accept a generation that does not cover
the population.

One process now trains every strategy of a run. The run writes the log of each
strategy under its own name, because a reader opens one strategy by name.

A run pays the cost of starting a worker process, and each worker builds its
own probe world. The pool holds the processes for the whole run, so a run pays
that cost once.

The validation pass and the yardstick pass still step one batch of worlds in
the process of their own strategy. Each plays one policy on a set of seeds, and
the queue does not split them.

## References

[^1]: ADR-0155, a batch of worlds steps in one call, in index order, decision
D1. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^2]: Target platform costs, the trainer process measurement.
`docs/reference/graviton-costs.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D2.
`docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: Findings register, FND-714. `docs/FINDINGS.md`
