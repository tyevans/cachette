---
id: 0073
title: Review the first renderable example again
status: complete
created: 2026-08-31
serves: [PRD-0002]
---

The first review of the product record for the first renderable example
found two of its statements unmet, and one of its two cost properties unmet.
The record stayed in `shaped/`.

Both gaps are now closed. One item made the viewer show that no tile is over
its capacity. The other settled whether the viewer may make the engine wait,
and the product record was amended to state what the project built.[^1]

The remaining work is the review itself. Run the demonstration and check each
statement in the record against what it does. Do not move the record on the
strength of the two items having closed. Running the code is what found the
gaps.

Only a reviewer may set a product record past `Shaped`, so this item asks for
that judgement rather than making it.

Refine this before you start it.

## References

[^1]: Backlog item 0070. `docs/backlog/complete/0070-settle-whether-the-viewer-may-make-the-engine-wait.md`

## Outcome

The review ran against the code, statement by statement, as this item
required. Every statement and both cost properties are met, so the product
record moved to `shipped/`.

Two qualifications are recorded rather than hidden. The claim that the window
shows every step rests on the shape of the demonstration loop, which steps
once, draws once, and skips nothing. A binary loop carries no test, so that
claim holds only while the loop stays skip-free. The capacity assertion in the
movement suite proves that movement never raises a tile above its capacity. It
does not prove that a tile is never above capacity, because a spawn may
over-fill one until the dense occupancy count lands.

## References

[^2]: Decisions register, DEC-020. `docs/DECISIONS.md`
