# ADR-0194: A generation is scored in shards and combined in candidate order

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

The project already runs one process for each strategy. That gives parallel
experiments. It does not make one experiment faster, so a run that trains one
strategy uses a fraction of the machine.

A worker process does not need the weight vector of a candidate. The trainer
builds a candidate as the centre plus or minus one perturbation, and it draws
the perturbation from a generator keyed on the run seed and the generation
number. A process that holds the centre, the generation number and the range of
candidates it owns rebuilds those candidates exactly.

**Determinism is the property this project cannot recover.** The engine gives
one answer at any thread count, and no result may take its order from which
worker finished first.[^3] A trainer that combined the returns of several
processes in the order the processes answered would break that rule outside the
engine, where no golden state hash watches. The weights of a run would then
depend on the load of the machine, and no run could be repeated.

## Decision

**A generation may be scored in several worker processes. The combination is
ordered by the candidate index, and the answer does not depend on how the work
was split.**

### D1. A shard count changes the spread of the work and nothing else

A run states how many worker processes score one generation. For one centre,
one generation number, one seed set and one population, the weights the run
reaches are identical at every shard count, position for position.

A reviewer finds a violation when a run at one shard count and a run at another
reach different weights from one seed.

### D2. A worker rebuilds its candidates from the seed, and never receives them

A worker receives the centre, the generation number and the range of pairs it
owns. It draws the perturbations of the whole generation from the run seed and
the generation number, and it takes the rows of its own range. No candidate
policy crosses to a worker.

The draw is a function of the run seed and the generation number alone. A
stream advanced by each generation would give a resumed run different
perturbations from the run it continues, and it would give a worker different
perturbations from the trainer.

### D3. The combination sorts on the candidate index

Each shard reports the candidate index it started at. The combination sorts on
that index and joins the score arrays in that order. **Nothing reads the order
in which the shards answered.**

A reviewer finds a violation when a combined array is built from an iteration
over futures as they complete, or from any order other than the candidate
index.

### D4. A shard that fails ends the generation

The combination refuses a set of shards that does not cover every candidate of
the population exactly once, and it names the candidate that is missing. A
worker that raises carries its failure to the caller, and the run ends.

A partial generation must never be scored. A run that trained on a subset of
its population would report a generation that it did not play, and no register
would hold the difference.

### D5. The caller states the process count and the worker count

The caller states how many processes score a generation, and how many engine
workers each process gives its batch. **Neither is derived from the core count
of the machine.** The worker count is a per process count.

A value derived behind the caller would be a second declaration site for a
number the caller already states, and the two would disagree silently.[^4]

### D6. A worker process runs one matrix thread

The mechanism that starts the worker processes holds each matrix library to one
thread. A library that reads no such setting starts one thread for each core,
in every process, and those threads take the cores the engine needs.

## The alternatives this rejects

**Send each candidate's weights to the worker.** This is the obvious shape and
it is rejected for cost, not for correctness. A population of many candidates
over a wide policy sends a large array on every generation, and it grows with
the width of the policy. The seed sends two integers instead.

**Combine the returns as the shards answer.** This is faster to write, and it
makes the weights of a run a function of the load of the machine. The
determinism record forbids it.[^3]

**Let one process open more engine workers.** This is what the project did.
The serial section between two decisions belongs to one interpreter, so more
workers inside one process do not fill the machine.

**Derive the process count from the core count.** This removes one argument
from the caller and adds a second declaration of a number the caller already
gives. It is the defect shape this project names first.[^4]

## Consequences

A single strategy can use a whole machine, so an experiment that trains one
policy is no longer held to the throughput of one interpreter.

A shard boundary must fall between two groups of candidates that share a world.
A league run seats several candidates in one world, so a boundary inside such a
group would build a world with a seat that no process filled.

The project now cannot combine a generation by any key except the candidate
index, and it cannot accept a generation whose shards do not cover the
population.

A run pays the cost of starting a worker process, and each worker builds its
own probe world. The pool holds the processes for the whole run, so a
generation pays that cost once.

The validation pass and the yardstick pass still run in one process. Each plays
one policy on a few seeds, so the whole machine is not the constraint there.

## References

[^1]: ADR-0155, a batch of worlds steps in one call, in index order, decision
D1. `docs/adrs/accepted/adr-0155-a-batch-of-worlds-steps-in-one-call-in-index-order.md`
[^2]: Target platform costs, the trainer process measurement.
`docs/reference/graviton-costs.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D2.
`docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
