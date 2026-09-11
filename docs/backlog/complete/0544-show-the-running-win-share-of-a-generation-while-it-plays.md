---
id: 0544
title: Show the running win share of a generation while it plays
status: complete
created: 2026-09-11
implements: [ADR-0001 D1, ADR-0194 D3]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A training generation of the paid size plays for several minutes. During that
time the heartbeat line gives the finished episodes, the live count, the ticks
and the rate. It gives nothing about how the games end. The project owner asked
to see the running win share, the running mean reward and the running mean
ticks for each game while a generation plays.

Each finished episode already returns its wins, its games, its ticks and the
reward of each candidate. The trainer reads these values after the generation
and not before. This item prints them in the heartbeat, and shows them on the
in-flight line of the progress dashboard.

## Impact review

**Governed by.** ADR-0001 D1 states that one binary gives one answer at any
thread count.[^1] The heartbeat reads the order in which the workers finish.
That order must reach no score, so the running figures stay a print. ADR-0194
D3 states that the combination of a generation sorts on the strategy, the
candidate and the seed, and never on the order a worker finished.[^2] This item
does not change the combination. ADR-0194 is a draft record.

**Changes.** None. No record describes the heartbeat fields.

**Creates.** None. A print that reaches no score is not a constraint that a
contributor could reasonably choose otherwise.

**Blockers.** None. The item states no cost figure, so BLK-007 does not govern
it.

**Serves.** No product record. The project owner stated the need directly.

**Precedent.** The findings register records that a reader which told a
heartbeat from a generation by its fields read a heartbeat as a generation.[^3]
The new fields include `mean` and `won`, which a generation line also carries.
Both readers test the `working` marker first, so a heartbeat still cannot reach
the generation path.

## Done when

- The heartbeat of a queued training generation holds the wins over the games,
  the win share, the mean reward for each game and the ticks for each game.
- The new fields follow the elapsed time, and every older field keeps its
  order.
- The docstring says that the running share reads the completion order and
  reaches no score. It also says that the share reads a little high until the
  queue drains.
- A heartbeat of a measured pass holds none of the new fields.
- The progress reader parses a heartbeat with the new fields and a heartbeat
  without them.
- The reader adds the shards of one pass as counts, and a test proves that a
  mean of the shard shares gives a different number.
- The in-flight line of the dashboard shows the running result when it exists.
- One new test is shown to fail with the defect put back.

## Outcome

The heartbeat of a queued training generation now ends with four fields: the
wins over the games, the win share, the mean reward for each game and the world
ticks for each game. A measured episode carries no win count, so a measured
pass prints the older line. The fields follow the elapsed time, so the reader
on a running machine still reads every field it knew.

The plan changed in one place. The heartbeat prints the wins as a count over
the games, and not only as a share. A reader that has only a share cannot add
two shards as counts. The reader takes the two counts and the mean. It derives
the ticks for each game from the ticks and the games, and it does not read the
printed value.

The ticks are world ticks. One seated world holds several games, so in a
seated generation the ticks for each game read short by the seat count. The
docstring states this, and the dashboard label says ticks a game.

Three tests went red with the defect put back: a tally that counted one game
for each finished episode instead of the games of that episode.

No register entry moved. No finding was corrected, no blocker opened or
closed, and no record changed.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0194, a generation is scored in shards, decision D3. `docs/adrs/draft/adr-0194-a-generation-is-scored-in-shards.md`
[^3]: The progress reader, the comment on the in-flight marker. `scripts/train_progress.py`
