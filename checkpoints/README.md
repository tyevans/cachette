# Stored policies (index)

This is an index. Each policy has a manifest beside it that states what it
scored, what it was fitted on, and the schema versions it needs.

**Every file here loads.** A weight file is a function of one observation
layout, one action layout, one world extent and one faction count. The loader
reads the fit a file states, compares it against the world the caller names,
and refuses a file that disagrees.[^C3]

**A file the loader refuses is removed rather than retired in place.** An index
that lists a file nobody can load costs a reader the time it takes to find that
out. Nine policies that stated observation version 3 went that way when the
engine wrote version 4, and later groups went the same way.

The section on what is gone says what each removed group reached. A
measurement stays here when its file cannot.

## Watch one play

Every policy here was trained on a 48 by 48 world of three factions. The
demonstration refuses a policy that does not fit the world it plays, so name
that world.

    uv run python -m cachette.demo --extent 48 --factions 3 \
      --policy 0=checkpoints/ring/obs6-people-gen1.npz

`--policy` repeats, and `N=path` names the faction that holds it. Three
policies play each other like this:

    uv run python -m cachette.demo --extent 48 --factions 3 \
      --policy 0=checkpoints/ring/obs6-people-gen1.npz \
      --policy 1=checkpoints/ring/obs6-land-gen1.npz \
      --policy 2=checkpoints/ring/obs6-conquer-gen1.npz

## What is here

Every file here holds a linear policy over the ring observation, and every one
of them loads.

| Policy | Score | Generation | Search |
|---|---|---|---|
| `ring/obs6-people-gen1` | 12051.5 | 1 | 64 candidates, 8 seeds |
| `ring/obs6-land-gen1` | 8873.8 | 1 | 64 candidates, 8 seeds |
| `ring/obs6-wealth-gen1` | 4233.9 | 1 | 64 candidates, 8 seeds |
| `ring/obs6-conquer-gen1` | 870.4 | 1 | 64 candidates, 8 seeds |

**Each row is generation 1 of a run that asked for 40.** The run was stopped
after two generations. A row is an early centre of a search and it is not a
level that any run held. Read none of them as a standard of play.

## What is gone, and what it reached

A removed file leaves its measurement here. A reader learns what the project
tried without being offered a file that cannot run.

### The dense observation

Four policies were fitted against a dense observation of 184 positions, over a
reward that weighed held ground at one. **They stated observation version 4 and
the engine writes 6, so the loader refused all four.** This index said so and
kept them anyway. Removing them completes a job that stopped at the prose.

| Policy | Score | Generation | Search |
|---|---|---|---|
| `dense/obs4-land-dense-mlp-gen99` | -1043.7 | 99 | 256 candidates, 1 seed, a fixed projection into 24 units |
| `dense/obs4-land-dense-linear` | -1071.1 | 87 | 256 candidates, 1 seed, linear |
| `dense/obs4-land-dense-mlp` | -1080.5 | 19 | 256 candidates, 1 seed, a fixed projection into 24 units |
| `dense/obs4-conquer-sparse-mlp` | -1167.4 | 15 | 256 candidates, 1 seed, a fixed projection into 24 units |

The first row supersedes the third: the same run and the same strategy, 80
generations later.

**A ring score is positive and a dense score is negative, and the sign is the
reward and not the play.** A loss now costs a tenth of what a win pays, where
it used to cost the same. The terms of each play style also read new field
names. Nothing about the two groups is comparable.

### The frozen projection over the ring observation

Two policies of the ring run held a fixed random projection into 24 units and
trained only the readout from it. The project removed that policy kind, and the
loader now refuses a file that names it.[^C6]

| Policy | Score | Generation | Search |
|---|---|---|---|
| `ring/obs6-land-net-gen1` | 9197.3 | 1 | 64 candidates, 8 seeds, a fixed projection into 24 units |
| `ring/obs6-conquer-net-gen1` | 978.2 | 1 | 64 candidates, 8 seeds, a fixed projection into 24 units |

**Each of the two led the linear policy of its own reward, and the kind was
removed on its architecture and not on a measured loss.** Both rows come from
the run that the table of what is here holds, so a reader can compare them
reward by reward. One generation of one run is one measurement, and the
section on reading a score says how little a single difference carries.

## A ring file states a version it was not fitted under

Each ring file was fitted under observation version 5 and states version 6. The
manifest beside it records both numbers and the reason.

The bump added no quantity and moved no position. The schema gained entries
that describe the shape of the spatial part, and the one token field became
four fields over the same positions. The observation of one fixed world is
identical under both versions, position for position, and the four token fields
begin where the one field began and hold the same count between them.

**A retag is correct only against that evidence.** Nine files that stated
version 3 were removed rather than retagged, because the layout had genuinely
changed and the weights described quantities that had moved.

## Read the score for what it is

**The score is a mean return over 128 fixed seed worlds. It is not a win
share.** Less negative is better. A return that weighs held ground at 1.0 does
not compare with one that weighs it at 0.1, and neither compares with a win
share.

**Read a difference of 200 as noise.** The same measurement over the same 128
worlds moved between -1043.7 and -1247.2 across four generations of one run,
on a centre that moved by one step each time. Each row above is the best its
run reached, not a level that run holds.

**No policy here plays well.** The built-in controller wins 0.367 of 256
episodes when it holds every seat, and one seat of a symmetric three-faction
game takes one third of the wins whatever the controller does. That figure is
the chance line, not a standard of play. Nothing measured so far reaches
it.[^C1]

**A score from one run does not compare with a score from another.** The seed
filter refused only a world that seated nobody. It now refuses a world that
seats fewer factions than the run asked for, and a world with a seat that
reaches no food. Every seed after the first refusal shifted, so a run measured
after the change plays a different set.[^C4]

## How much water a world holds does not predict the game

A world of the training extent holds about a third water at the median, and
the share reaches above nine tenths in a small tail. **The wetter bands are
the bands the controller wins most often, and their episodes run the
longest.** Fewer rivals reach the seat. Do not read a wet world as a hard
one.[^C5]

## A game that ends stops the demonstration

The demonstration raises `the world cannot describe its own units` when a
policy asks for an observation after a game ends.[^C2]

## References

[^C1]: Findings register, FND-645. `docs/FINDINGS.md`
[^C2]: Findings register, FND-644. `docs/FINDINGS.md`
[^C3]: ADR-0193, an observation names another faction by a position relative to the reader. `docs/adrs/draft/adr-0193-an-observation-names-another-faction-by-a-position-relative-to-the-reader.md`
[^C4]: Findings register, FND-666. `docs/FINDINGS.md`
[^C5]: Findings register, FND-667. `docs/FINDINGS.md`
[^C6]: Findings register, FND-686. `docs/FINDINGS.md`
