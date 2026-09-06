---
id: 0370
title: Refuse a build on ground another faction holds
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-036]
---

## Why

The project owner answered that a unit builds only on ground that its own
faction holds. Nothing checks it. The pass that collects the build intents of a
step reads the build order of each live unit and the tile it stands on, and it
reads no holder and no faction. A unit of one faction builds on ground another
faction holds, and it finishes. The findings register holds the measurement and
the world that produced it.[^1]

The control plane can now order a build, so the gap is reachable from a game
rather than from a Rust test alone.[^2]

## What is missing before this can be refined

- Where the check goes. The intent pass is the only place that runs at every
  step, and a check at the moment of the order does not hold a build that
  continues after the ground changes hands. The decisions register holds the
  reasoning that put the rule in the core.[^3]
- What happens to a build in progress when the ground changes hands. One
  blocker holds that question and the project owner owns it.[^4] The work must
  express the answer as a parameter rather than invent it.
- Whether a refused build is silent or reported. A unit that stops building
  because the ground changed hands is a thing a watcher wants to see, and the
  engine reports no event for it today.
- Whether unheld ground counts. Every tile of a new world is held by nobody,
  and every test of the build verbs builds on such ground. A rule that names
  only the holder would stop a build on empty ground as well.

## Outcome

Closed by item 0484, which refuses a wider set. A build on a tile the
builder's faction does not hold is refused, whoever holds it and whether
anybody holds it, unless the kind is a road. One function states the rule, and
the build verb and the build intent pass both call it, so a build whose ground
changed hands stops as well.[^5]

The four open questions were answered there. The rule sits in the core, in one
function that both paths call. A build in progress keeps the work it did, and
the entry stays, because the blocker that asks whose it becomes is still
open.[^4] A refused order is dropped and counted where the controller gave it,
and no event reports it. Unheld ground counts: a unit builds only a road on
ground nobody holds.

## References

[^1]: Findings register, FND-380. `docs/FINDINGS.md`
[^2]: Backlog item 0341, bind the build order and the upgrade removal to the control plane. `docs/backlog/complete/0341-bind-the-build-order-and-the-upgrade-removal-to-the-control-plane.md`
[^3]: Decisions register, DEC-161. `docs/DECISIONS.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^5]: Backlog item 0484, hold ground only within reach of an owned city. `docs/backlog/complete/0484-hold-ground-only-within-reach-of-an-owned-city-and-refuse-a-build-outside-it.md`
