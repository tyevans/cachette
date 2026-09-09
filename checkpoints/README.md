# Stored policies (index)

This is an index. Each policy has a manifest beside it that states what it
scored, what it was fitted on, and the schema versions it needs.

**Every file here loads.** A weight file is a function of one observation
layout, one action layout, one world extent and one faction count. The loader
reads the fit a file states, compares it against the world the caller names,
and refuses a file that disagrees.[^C3]

**This index holds no policy right now.** The observation reached version 7,
and every stored policy read version 6. The section on what is gone holds what
each one measured, and the files themselves are kept outside this directory so
that nothing this project measured is thrown away. A reader who wants a policy
to play must train one.

**A file the loader refuses is removed rather than retired in place.** An index
that lists a file nobody can load costs a reader the time it takes to find that
out. Nine policies that stated observation version 3 went that way when the
engine wrote version 4, and later groups went the same way.

The section on what is gone says what each removed group reached. A
measurement stays here when its file cannot.

## Watch one play

The demonstration refuses a policy that does not fit the world it plays, so
name the world the policy was trained on. A policy trained on a 48 by 48 world
of three factions plays like this:

    uv run python -m cachette.demo --extent 48 --factions 3 \
      --policy 0=<path to a weight file>

`--policy` repeats, and `N=path` names the faction that holds it, so three
policies play each other by naming three paths.

**No path is named here on purpose.** This index named two files that had
already been removed, and a reader following it met a directory that does not
exist. A command that names a live file goes stale the next time the
observation moves.

## What is here

Nothing. The table below the break holds every policy this project has stored,
and why each one went.

## What is gone

### The place action table at observation version 6

Eight policies from one run on a 48 by 48 world of three factions, at
observation version 6 and action version 2. **The observation reached version
7 on 8 September 2026 and the loader refuses all eight.** The raise was
mandated rather than accidental: positions moved and published values changed,
which is the case the version rule exists for.

The files are kept under `archive/obs6-act2-policies/` rather than deleted,
because the project owner asked for these weights and a version raise is not a
reason to destroy them. They cannot be loaded by any current build.

A score is a mean return over 128 held-out seeds under that strategy's own
weighting, against the built-in controller measured on the same seeds under
the same weighting.

| File | Score | Yardstick | Beats it | Generation | Kind |
|---|---|---|---|---|---|
| `place/obs6-act2-people-gen9` | 13808.4 | 11313.1 | yes | 9 | linear |
| `place/obs6-act2-people-structured-gen3` | 13637.6 | 11313.1 | yes | 3 | structured |
| `place/obs6-act2-land-gen7` | 11582.2 | 9940.8 | yes | 7 | linear |
| `place/obs6-act2-land-structured-gen5` | 9912.0 | 9940.8 | no | 5 | structured |
| `place/obs6-act2-wealth-gen5` | 5028.0 | 5205.8 | no | 5 | linear |
| `place/obs6-act2-wealth-structured-gen5` | 4724.3 | 5205.8 | no | 5 | structured |
| `place/obs6-act2-conquer-structured-gen7` | 1115.1 | 1417.4 | no | 7 | structured |
| `place/obs6-act2-conquer-gen9` | 1033.2 | 1417.4 | no | 9 | linear |

**A rating run then measured these eight against each other and against the
controller, and the scores above do not survive it.** The rating played every
unordered triple of the nine players over one world each, with three cyclic
rotations so that every player held every seat an equal number of times: 252
games, 84 worlds, no undecided game. Seven of the eight rate below the
controller they trained against, and every one of those gaps clears two
standard errors.

| Player | Win share | Elo against the controller |
|---|---|---|
| `wealth-structured` | 0.667 | +53.1 |
| the built-in controller | 0.583 | 0.0 |
| `conquer-structured` | 0.357 | −146.5 |
| `land` | 0.286 | −199.7 |
| `wealth` | 0.286 | −199.7 |
| `people` | 0.238 | −239.7 |
| `people-structured` | 0.214 | −261.8 |
| `conquer` | 0.214 | −261.8 |
| `land-structured` | 0.155 | −326.5 |

**A score against a yardstick is not a measure of play.** Every row above beat
or missed its own bar under its own weighting, and the rating says how each
one actually played. The two orderings disagree: `people` stands furthest
above its own bar and rates seventh of nine.

`wealth-structured` is the one exception, and it is indistinguishable from the
controller rather than better: the interval runs from −74.6 to 180.7, and the
controller takes 0.556 of their games. It won every one of its 56 wins by the
wonder path and none by any other, and it reached the tick limit in 0.238 of
its games against 0.68 to 0.80 for the rest. It found a different game to play.

**Two facts explain the table.** Every shaped weight of all eight strategies
was a difference of a field since the previous decision, and a sum of
differences collapses to the endpoints under an undiscounted episode return,
so all eight trained against a terminal reward.[^C7] And no policy founds a
settlement: the action row that queues a settler was legal on every decision
of one policy's episodes and that policy took it zero times, because nothing
rewarded it.[^C8]


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

**A conquest score no longer compares with a score measured today.** The
reward of the conquest strategies now pays one terminal term for the time a
win left on the clock, and no other strategy carries it. Each conquest row
above was measured before that term existed, so it holds a return under a
weighting the run no longer uses. Every other row still compares.[^C6]

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
[^C6]: Findings register, FND-692. `docs/FINDINGS.md`
[^C7]: Findings register, FND-700. `docs/FINDINGS.md`
[^C8]: Findings register, FND-698. `docs/FINDINGS.md`
