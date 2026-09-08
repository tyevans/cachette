# Stored policies (index)

This is an index. Each policy has a manifest beside it that states what it
scored, what it was fitted on, and the schema versions it needs.

## Watch one play

Every policy here was trained on a 48 by 48 world of three factions. The
demonstration refuses a policy that does not fit the world it plays, so name
that world.

    uv run python -m cachette.demo --extent 48 --factions 3 \
      --policy 0=checkpoints/conquer/conquer-obs3-cellA-gen9.npz

`--policy` repeats, and `N=path` names the faction that holds it. Three
policies play each other like this:

    uv run python -m cachette.demo --extent 48 --factions 3 \
      --policy 0=checkpoints/conquer/conquer-obs3-cellA-gen9.npz \
      --policy 1=checkpoints/conquer/conquer-obs3-cellB-gen11.npz \
      --policy 2=checkpoints/imitate/imitate-obs3-mlp.npz

## What is here

The win share is on the validation seed worlds, which never move.

| Policy | Wins | Search |
|---|---|---|
| `conquer/conquer-obs3-cellA-gen9` | 11 of 32 | 64 candidates, 8 seeds |
| `conquer/conquer-obs3-cellA-gen5` | 10 of 32 | 64 candidates, 8 seeds |
| `conquer/conquer-obs3-cellB-gen11` | 9 of 32 | 16 candidates, 32 seeds |
| `conquer/conquer-obs3-cellB-gen5` | 8 of 32 | 16 candidates, 32 seeds |
| `conquer/conquer-obs3-interval5-gen5` | 5 of 32 | 16 candidates, 32 seeds, one decision for each 5 ticks |
| `conquer/conquer-obs3-gen9` | 3 of 8 | 64 candidates, 8 seeds |
| `conquer/conquer-obs3-gen11-latest` | 3 of 8 | 64 candidates, 8 seeds |
| `imitate/imitate-obs3-linear` | 0.042 | fitted to controller commands |
| `imitate/imitate-obs3-mlp` | 0.042 | fitted to controller commands |

## Read the win share against chance, not against the controller

**The controller yardstick is not a standard of play.** The yardstick world
gives the learner seat back to the built-in controller, so all three factions
of that game are the same controller. One seat of a symmetric three-faction
game takes one third of the wins, whatever the controller does.

    chance         10.7 of 32
    yardstick      13 of 32
    standard error 2.7 of 32

The best policy here reaches 11 of 32. That is chance, within the error of
the measurement. A policy that reaches the yardstick has reached chance and
is not a policy that plays well.[^C1]

The two figures measured over eight seed worlds are coarser still. Eight
worlds quantise the win share into eighths, and a score of three eighths
reads as 0.375 whatever the truth is nearby. Do not compare an eighth against
a thirty-second.

## The two imitation policies play differently, on purpose

They were fitted to the commands of the built-in controller, not trained to
win. The controller issues about 33 commands in the span of one learner
decision, so one action must stand for a window of many. Watching one is the
quickest way to see what that reduction costs: it repeats one gather row and
does little else.

## A game that ends stops the demonstration

The demonstration raises `the world cannot describe its own units` when a
policy asks for an observation after a game ends.[^C2]

## References

[^C1]: Findings register, FND-645. `docs/FINDINGS.md`
[^C2]: Findings register, FND-644. `docs/FINDINGS.md`
