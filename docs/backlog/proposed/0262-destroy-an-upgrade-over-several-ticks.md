---
id: 0262
title: Destroy an upgrade over several ticks
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0008]
blocked-by: []
---

## Why

**A unit cannot destroy an upgrade.** The engine removes an upgrade instantly,
by address, through a control-plane call. No verb lets a unit do it, and no
work accumulates against a removal.

The project owner answered this on 3 September 2026. Destruction takes work.
A unit that destroys an upgrade does work over several ticks, in the same way
that a unit that builds one does. The instant removal stays, because some
needs ask for it, and it stays as a control-plane call.[^1]

**The owner named the two as two verbs, and wrote the word "perhaps".** Destroy
is the unit-level verb that takes work. Reclaim is the faction-level verb that
is instant. Read the two names as the owner's current thinking rather than as a
settled interface, and decide the names when this item is refined.[^1]

**The two paths are not the same act, and the engine must not merge them.**
One is a unit spending ticks. The other is the control plane setting a state.
A single code path that serves both would give the control plane a unit's
cost, or give a unit the control plane's authority.

The building side of this already exists and gives the shape to follow. A
build accumulates work against a tile over several ticks, and a clamp absorbs
every contribution after the work is done.[^2] A destruction is the same
mechanism with the opposite sign, and the same question about what stops a
unit that has finished.

## What the work does

Add the verb that a unit uses to destroy an upgrade, driven by accumulated
work rather than by one call. Keep the instant removal as it is. Say in the
implementation which of the two paths each caller takes.

## What good looks like

A unit stands on an upgrade, is ordered to destroy it, and the upgrade goes
after several ticks rather than at once. The control-plane removal still
removes an upgrade in one call. A test drives both and shows they are
different paths.

## What it does not do

It does not decide who may order a destruction. Anyone may destroy an
upgrade, and no faction rule restricts it.[^1]

It does not decide whether an upgrade changes hands when the ground does.
That question is still open, and it has its own row.[^3]

It does not make a unit choose to destroy anything. Nothing makes a unit
choose to build either, and one item holds that gap.[^4]

## References

[^1]: Blockers register, BLK-034, resolved. `docs/BLOCKERS.md`
[^2]: Backlog item 0058, build an improvement over several ticks. `docs/backlog/complete/0058-build-an-improvement-over-several-ticks.md`
[^3]: Blockers register, BLK-035. `docs/BLOCKERS.md`
[^4]: Backlog item 0180, let a unit choose to build. `docs/backlog/proposed/0180-let-a-unit-choose-to-build.md`
