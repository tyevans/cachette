---
id: 0056
title: A learner plays one faction against the controllers
status: Shaped
created: 2026-09-05
---

# PRD-0056 — A learner plays one faction against the controllers

## Who this is for

A researcher or a game developer who wants to train a policy for one faction.
The built-in controllers play the other factions. The person wants the policy
to learn to win the game the engine already plays.

The other two audiences do not need this first.

A modeller who studies an economy wants the world to run on its own rules. A
trained policy replaces one of those rules with a learned one. That makes the
model harder to read, not easier.

A developer who only ships a game needs an opponent that plays well enough.
The controllers already give one. A trained policy is a stronger opponent, and
it is not the first opponent.

## What the person cannot do today

The person cannot hand one faction to an outside policy.

The engine plays every faction with the same controller. A flag stops the
controller for one faction. Nothing then plays that faction. The person holds
the verbs, and nothing else a training loop needs.

Five things are missing, and each one blocks the loop on its own.

**No seat for a policy.** No interface takes a policy and lets it play a
faction while the controllers play the rest.

**No observation a policy can consume.** The readers return named fields for a
person to read. A policy needs one answer of a fixed size, every tick, for
every faction it plays.

**No action a policy can emit.** The verbs take arguments a person writes. A
policy needs a bounded set of choices, and it needs to know which of them are
legal now.

**No reward.** Nothing tells the person whether the faction is winning. The
engine reports one number at the end of a game and one running score.

**No cheap way to run many games.** The person runs one game in one process.
Training needs thousands of games, and one game per process pays for the
interpreter thousands of times.

The person can build all five by hand. Each one is then built against a
surface that moves, and two people build five different versions of it.

## What good looks like

Each statement below can be checked.

- **The harness never shows a learner anything a player of that faction could
  not see.** An accepted record states that rule, and this one takes it
  whole.[^1] No observation a learner reads holds a value the faction has not
  observed. This holds for a summary as well as for a tile.
- A policy takes one faction. The controllers take every other faction. The
  world does not know which faction a policy plays.
- A run of a game reproduces exactly from its seed and its action log. A
  replay gives the same world state, at any thread count, on any machine.
- A learner emits only actions the same interface offers a caller. No action
  reaches the world by a path a caller cannot use.
- The same game is playable by a controller, by a caller and by a policy. The
  world does not change when the player changes.
- The harness reports, on each decision, whether the faction is closer to
  winning. It reports the outcome once, at the end of the game.
- Many games run in one process. The person adds a game without adding a
  process.
- A second training stack uses the harness without a change to the part every
  stack shares.
- A game the harness played reports the same result as the same game played by
  the balance harness across seeds.[^2]

## What this does not do

- It does not choose a learning algorithm, a network shape or a training
  library. The person brings those.
- It does not decide the reward. What a faction should be rewarded for is a
  rule of the downstream game.[^4]
- It does not add a verb the control plane lacks. A policy plays with the
  verbs a caller already has.
- It does not make the world faster. A game costs what a run costs today.
- It does not train a policy. It gives the person what a training run needs.
- It does not judge a policy. Whether a policy plays well is a measurement,
  and the person makes it.
- It does not change how the controllers choose. A controller plays the same
  game after this work as before it.
- It does not decide how any of this is built. That is an architectural
  question, and it belongs elsewhere.

## What it costs at the target scale

The target is 16.7 million tiles and one million units. This record states the
shape of each cost and no figure.

**What a learner reads follows the cells and the factions, and never the
units.** The size of one observation grows with the faction count and with the
number of summary cells the learner looks at. It does not grow with the unit
count or with the tile count. An observation that grows with the population
fails this record, because a policy needs one answer of a fixed size.

**What a game costs follows the ticks.** One game costs the length of the game
times the cost of one tick. The harness adds no reader that walks the world,
so it adds nothing to the cost of a tick.

**What many games cost follows the game count.** Running two games costs twice
one game. Nothing in the harness grows faster than the game count.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^3] A figure belongs in a reference table under that blocker, not in
this record.

## Which blockers govern this

- **One blocker governs every cost figure.**[^3] Every cost statement above
  states a shape. None states a number. The figures belong in a reference
  table when a run on the target platform measures them.
- **One blocker holds the rules of the downstream game.**[^4] A reward weighs
  the things a faction gains. What those things are worth, and what winning
  means, are rules of that game. This record states none of them, and it does
  not choose the reward.
- **One blocker holds the scale the downstream game runs at.**[^5] The cost
  shapes above are written at the engine's target scale. A smaller game
  changes what is affordable, and it changes the order of the work. It does
  not change any statement in this record.

This record depends on factions playing a game to an end. That need is
accepted and not yet met.[^6]

## References

[^1]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: PRD-0053, a game is balanced across seeds. `docs/product/accepted/prd-0053-a-game-is-balanced-across-seeds.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-051. `docs/BLOCKERS.md`
[^6]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
