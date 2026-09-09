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

**Nothing. Every stored policy has been removed, and the index states what
each one reached.**

A weight file is a function of one observation layout and one action layout.
The engine now writes observation version 6 and action version 2, and no file
this project ever wrote states both. An index that lists a file nobody can
load costs a reader the time it takes to find that out, so the files go and
their measurements stay.

## What is gone

Every row below names a file this project removed, and the measurement it
carried. A file goes when the loader refuses it, and the measurement stays
because a reader wants to know what a layout managed.

### The ring observation, before a place argument

Four policies trained on a 48 by 48 world of three factions, one generation
into a run that asked for forty. Each is an early centre of a search and not
a level any run held. The scores are mean returns over 128 held-out worlds,
and a return under one objective does not compare with a return under
another.

| File | Score | Search |
|---|---|---|
| `ring/obs6-people-gen1` | 12051.5 | 64 candidates, 8 seeds, no hidden layer |
| `ring/obs6-land-gen1` | 8873.8 | 64 candidates, 8 seeds, no hidden layer |
| `ring/obs6-wealth-gen1` | 4233.9 | 64 candidates, 8 seeds, no hidden layer |
| `ring/obs6-conquer-gen1` | 870.4 | 64 candidates, 8 seeds, no hidden layer |

They were refused by the action version and not by any fault of their own. A
place argument gives the action table one row for each cell of the
observation frame, so a policy fitted against 29 rows cannot score 180.

### What the dense observation reached, before the ring stack

Four policies of an older observation, kept here as a record of what that
layout managed. The best reached -1043.7 over 128 held-out worlds at
generation 99, against -1071.1 for a policy of the same run with no hidden
layer. A negative score is the older reward, which paid a loss the same
weight it paid a win, and it does not compare with any score above.

### What a frozen projection reached

Two of the removed ring policies trained a readout over a fixed random
projection of the observation. **They beat their siblings that trained a
whole matrix**: 9197.3 against 8873.8 on held ground, and 978.2 against 870.4
on conquest. The kind is gone because a fixed projection cannot learn a
representation, and not because it lost.

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
