---
name: launching-cachette-training
description: Use when launching, sizing, stopping or diagnosing a Cachette reinforcement learning training run on a rented instance, or when a run's throughput, utilisation or reported scores look wrong
---

# Launching Cachette Training

## Overview

A training run costs money per minute and produces a number someone will act
on. Two failures dominate: a run that spends the box on nothing, and a run
that reports a figure which does not mean what its label says.

**A run is not sized until the plan is printed, and a policy is not measured
until a win share names its seed set.**

## Before you spend: print the plan

`--print-plan` costs nothing and answers every sizing question. Read these
fields and refuse to launch until each is right.

| field | what to check |
|---|---|
| `workers_each` | cores divided by strategies. Nobody chose it unless you did |
| `measurement_share` | above 0.5 means most episodes measure rather than train |
| `estimated_minutes` vs `cap_minutes` | the estimate is a floor; reality has come in below it |
| `generations_reached` | not `generations_asked`. A configuration asking 20 delivered 3 to 9 |
| episodes for each worker | `population x seeds x strategies / cores`. Below ~6 and one episode's tail dominates a generation |

## Concurrency

- **`--shards`, `--workers` and `--baseline-workers` are retired.** The
  trainer refuses them and points to `--pool`. One trainer process runs every
  strategy of a run with `--pool <cores>`, and the trainer splits the workers.
  Read `workers_each` from `--print-plan`; do not set it.
- **Several strategies in one run divide one box.** Three strategies at
  population 256 and 12 seeds on 64 cores reach generation 2 in six hours; one
  reaches about 7 to 9. Give a heavy strategy its own box. The launcher runs
  one box for each invocation, and concurrent launches are safe: each teardown
  terminates only its own instance. Its closing orphan list names the other
  live boxes, so do not act on it while they run.
- **Raising the population is usually the right move.** It raises the
  episodes for each worker, and it improves the alignment of a step as the
  square root of the pair count. **Reach for population before reaching for a
  smaller box.**
- **More engine threads for one world step is slower** at every extent this
  project trains: extent 128 gives 439 ticks a second at one thread and 215 at
  sixteen. Parallelise across worlds, never inside a step.

## Never judge throughput from a measurement pass

This trap has been hit three times and cost a restart each time.

- **The tick rate decays inside every pass** as worlds develop: one baseline
  ran 1806, then 989, then 187 ticks a second. A point rate is meaningless
  without its position in the pass.
- **Compare whole-pass durations**, not point rates. One baseline reached 98
  percent at 583 seconds; that number needs no interpolation.
- **A baseline or yardstick pass does not fill the box** and says nothing
  about training throughput. Wait for a training generation.
- A rate measured on one worker does not multiply by the worker count.
  Per-worker throughput collapses under concurrency.

## The instruments, and what each refuses to hide

Read these on every generation line. They exist because a run once published
four policies that could not play while every number said they were strong.

| instrument | meaning | failure value |
|---|---|---|
| `varies` | does the unmasked argmax change within an episode | `0.00` means a fixed preference order, not a policy |
| `top-share` | share of decisions on the most common action | near `1.0` is a constant. Published failures read 0.96 |
| `agreed` | rank agreement of the generation | `0.00` means the centre does not move |
| `aligned` | `sqrt(pairs / trainable)` | compare against the law; it should match |
| `won` | win share | compare against the controller's bar, with its error |

**`agreed 0.00` at generation 0 is a coin flip, not a stall.** Pure noise
clips to zero half the time at population 24. Only three consecutive zeros
indicate a stall, and the cause is too few seeds around an untrained centre.

## Reading a score without fooling yourself

- **A shaped return is not a win rate and not a percentage.** A return of
  86.90 against a bar of 50.00 means 1.74 times the controller's shaped
  return, nothing more.
- **Demand the seed set.** A selection maximum over ten validation passes on
  the seeds that chose the centre is not a holdout. Both now carry names that
  say which they are.
- **The bar has an error bar.** A win share near one third carries a standard
  error of 0.042 over 128 seeds and 0.029 over 256. Two policies must differ
  by roughly twice that before the measurement separates them. A run over 24
  worlds once called four different policies the same thing.
- A trained start is **below** chance: a zero centre wins about 0.05 where a
  uniform legal draw wins 0.333 and the controller 0.37. Reaching chance is
  the first milestone.
- **The bar is 0.333, and it is structural.** Three factions, every game
  decided, so an even seat takes a third. The built-in controller measured
  0.33 over 64 games and a seat that sends no action at all measured 0.000.
  Measure the bar with `--baseline-only --holdout 64`; it costs ten minutes
  and it has refuted two confident diagnoses.

## Seeds decide whether a generation counts

`--seeds` sets how many worlds score one candidate, and it is the lever that
decides whether the ranking can see anything.

At 3 seeds one run lost 5 of 16 generations to `carried no information`, and
one of the lost generations held candidates whose mean win share beat the
controller. At 12 seeds the next run lost none in five, and reached a held-out
0.42 where the first never passed 0.070.

Seeds cost linearly and do not cancel: episodes to cross is
`2 x trainable x seeds`, whatever the population.

**Raise `--validation` with them.** Six validation seeds cannot separate 464
from 489, so a run froze its published centre for eleven generations on
noise. Twenty-four seeds cost a few hundred episodes against a hundred
thousand.

## Training the readout alone

`--train-readout-only` trains only the structured policy's readout: 180 action
rows × (trunk width + 1), 3,780 weights at the default widths. The towers and
trunk keep their seeded draw, bit for bit. The observation and the action table
do not grow with the extent, so the count is the same at every extent. Alignment
at population 128 rises from 0.078 to 0.130.

- It refuses a linear strategy before anything is rented.
- Check the plan's `# note` lines: "trains N of M weights" and "aligns X".
- `--resume` refuses a checkpoint written under the other setting.

## Choosing strategies

Two tables exist and they cannot be mixed in one run. `--styles` selects the
play style table; `--only` selects the default table, whose `-structured`
suffix is a policy shape rather than a style.

Pick styles whose win path is reachable. Measured over 24 games: renown 13,
wonder 7, domination 4, **territory 0**. Check the style's own terminal
weights before trusting it — one style pays nothing for a win and its own
description says it does not want to win, which is incoherent under win-share
selection.

## Run it on the instance, not the dev box

Engine-bound work belongs on the Graviton spot instance. A 64 core instance
reaches 18,000 to 21,000 ticks a second at extent 48 for about $0.77 an hour;
a 16 core dev box gives about 400 in aggregate under concurrency. The local
run loses on electricity as well as on time, so there is no trade to weigh.

Local is for tests, lint and record checks.

## Stopping, and surviving a reclaim

`--stop <dir>` keeps the weights and the resume point, then verifies nothing
is left. Read its closing lines: two empty tag listings mean no orphaned
instance, security group or key pair. Never kill by pattern.

**A resume point on the instance protects the run, not the result.** The
trainer writes one every generation, and that survives the trainer failing. It
does not survive the machine going away, and a reclaimed spot instance never
lets `--stop` run. One run reached a held-out win share of 0.42 and lost every
weight to a reclaim two hours in.

The follower fetches the weights and the report on every poll, so a reclaim
costs at most one poll. The trainer also keeps `NAME-genNNN.npz` for every
generation, and it writes each weight file atomically. Check that
`fetched N weight files and M other files` appears with N above zero. The line
`no weights arrived this round` means the weight copy failed.

**The follower is the only copy.** Nothing goes to S3. It runs inside the
launcher on this machine, so a sleeping laptop or a dead launcher stops the
fetch while the box trains on. Start the launcher detached, under
`systemd-inhibit --what=sleep`, and never Ctrl-C a launching terminal: its
teardown terminates the box with no final copy. `--attach <dir>` reconnects
safely. A reclaim still restarts training from nothing (backlog 0539). `--stop` and `--attach` both report what is already local before they
touch the instance, so "the instance gave nothing back" is not the same as
"nothing survived".

## Common mistakes

| Mistake | Reality |
|---|---|
| "The baseline cache will hit, so this is cheap" | Check the key first. It is `engine, observation_version, world, seeds, scoring` |
| "Utilisation is low, downsize the box" | Under-utilisation is usually the shape of the work, not the size of the box. Raise population before shrinking cores |
| "Throughput looks bad, reshard now" | Not from a measurement pass. One such restart made a run 2.5 times slower |
| "It scored above the yardstick, so it plays well" | The yardstick is a shaped return unless it names a win share |
| "40 generations fit, so ask for 40" | Ask what `generations_reached` says, and leave slack for a derived rate |

## Red flags

- About to launch without reading `--print-plan`.
- About to quote a tick rate without its position in the pass.
- About to compare two figures whose seed sets you cannot name.
- About to call a local measurement free.
- About to report a rising score without checking `varies`.
