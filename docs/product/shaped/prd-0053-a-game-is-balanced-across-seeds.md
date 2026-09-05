---
id: 0053
title: A game is balanced across seeds
status: Shaped
created: 2026-09-05
---

# PRD-0053 — A game is balanced across seeds

## Who this is for

A developer who builds a strategy game on this engine. The developer must
know that the game the engine plays is fair before a player joins it.

A researcher needs this second. A researcher who compares two runs must know
where the difference comes from. It must come from the change under study and
not from a seat that always wins.

## What the person cannot do today

A developer cannot tell whether the game is fair.

Once factions play a game to an end, one run shows one winner. The developer
cannot tell from one run whether that winner was lucky. The seat may be the
best seat, or the way it won may be the only way that works. A mechanism that
never fires in one run may never fire at all, or may have been unlucky.

This has three costs.

The developer tunes blind. A change to a rate changes the winner of one run.
The developer cannot tell whether it changed the game.

The developer cannot trust a quiet subsystem. A count of zero in one run is a
question. A count of zero in every run is a defect.

A researcher cannot control for the seed. Two runs that differ by one setting
also differ by everything the seed decides. Nothing says how large that second
difference is.

## What good looks like

Each statement below can be checked.

- One command plays a fixed set of seeds to the end and reports on the set.
- The report states, for each way to win, the share of seeds it won. No way
  to win takes more than its stated share.
- The report states, for each seat, the share of seeds it won. No seat takes
  more than its stated share.
- The report states the share of seeds that ended before the tick limit. That
  share is above a stated floor.
- The report states, for each mechanism the engine holds, the seeds in which
  it never fired. That list is empty.
- The report names the seed set and names every seed that failed a statement.
  A developer can replay the failure.
- The command does not gate a merge. It runs on the schedule the slow checks
  run, and before any change to a game value.
- The same seed set gives the same report, at every thread count, on every
  run.

## What this does not do

- It does not make the game fair. It says whether the game is fair. Tuning is
  the work that follows the report.
- It does not state the shares or the floor. What share is fair is a rule of
  the downstream game, and nobody has stated it.
- It does not choose the seed set. Which seeds and how many are values, and a
  register holds them.
- It does not measure performance. How long a game takes to play is a
  benchmark, and a benchmark is a separate thing.
- It does not judge a player. No player exists in this work.
- It does not decide how the report is produced. That is an architectural
  question, and it belongs in a decision record.

## What it costs at the target scale

The cost driver is the number of seeds times the length of one game, not
anything about one step.

One game costs what a run costs. The harness costs that, times the seed count.
Nothing in the harness adds to the cost of a step.

Three properties follow. A solution must have all three.

- The cost grows with the seed count and with the tick limit, and with nothing
  else.
- The harness reads what the engine already reports at the end of a game. It
  adds no reader that walks the world.
- The harness is long by design, so it does not gate a merge. A check that
  costs a game per seed cannot run on every commit.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^1]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape and not a number.
- **One blocker holds the rules of the downstream game.**[^2] The fair share
  of each way to win and of each seat are rules of that game. So are the floor
  on games that end, the tick limit and the seed set. This record states none
  of them.

This record depends on factions playing a game to an end. That need is
shaped and not yet met.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^3]: PRD-0048, a developer watches factions play a game to an end. `docs/product/shaped/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
