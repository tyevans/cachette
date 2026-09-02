---
id: 0070
title: Settle whether the viewer may make the engine wait
status: complete
created: 2026-08-31
serves: [PRD-0002]
---

The product record for the first renderable example stated that the window
never makes the engine wait, that it drops what it cannot draw, and that it
reports the drop. The viewer record decided the opposite, and said so in its
own consequences. One loop steps and then draws, so the two rates are one
number. Nothing drops a frame and nothing reports a drop.

The register holds the answer.[^1] The project owner chose to amend the
product record. The viewer record already decided this correctly, and it
already names its own successor, so it needs no change.

## Outcome

The product record now states that the window shows every step, and that a
viewer which only watches asks the engine for no extra work. Its bounds
section excludes the demonstration binary by name, and states that a person
who must watch a world that steps faster than a screen refreshes has a later
need. Its cost section states the same bound.

The record stays in `shaped/`. A separate item reviews it against the code
and moves it if it passes.[^2] Only a reviewer may move a product record past
`shaped/`.

No engine change follows. Separating the two rates is the later successor,
and it needs a record that does not exist.

## References

[^1]: Decisions register, DEC-022. `docs/DECISIONS.md`
[^2]: Backlog item 0073. `docs/backlog/complete/0073-review-the-first-renderable-example-again.md`

## Outcome

The register settled it. The viewer may make the engine wait in the
demonstration binary, and the product record now says so. The record excludes
the demonstration by name, drops the claim that the window reports a dropped
frame, and states that a watching viewer asks the engine for no extra work.

Nothing in the engine changed. The record that ties the two rates together
keeps its number and its status, and it already named its own successor: an
engine that runs on its own thread and publishes a frame the viewer reads.
That successor needs a snapshot record, which does not exist, and no caller
needs the two rates apart yet.
