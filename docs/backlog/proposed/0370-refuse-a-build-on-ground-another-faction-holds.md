---
id: 0370
title: Refuse a build on ground another faction holds
status: proposed
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

## References

[^1]: Findings register, FND-380. `docs/FINDINGS.md`
[^2]: Backlog item 0341, bind the build order and the upgrade removal to the control plane. `docs/backlog/complete/0341-bind-the-build-order-and-the-upgrade-removal-to-the-control-plane.md`
[^3]: Decisions register, DEC-161. `docs/DECISIONS.md`
[^4]: Blockers register, BLK-036. `docs/BLOCKERS.md`
