---
id: 0317
title: Separate the engine tick from the wall clock in the demonstration
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

**The demonstration steps the engine once for each frame it draws.** The two
rates are one number. A watcher who wants to look at a moment must close the
window, and a watcher who wants the world to run faster than the screen
refreshes cannot ask for it.

**A watcher cannot stop the world and read the panel.** The panel reports the
log of the tick that just ran, and that log is gone one frame later. A person
who sees a promotion or a rationing has one thirtieth of a second to read it.

**A single frame step is the only way to watch one tick.** Without it a
question about what one tick did is answered by writing a picture and reading
a file.

## Done when

- The demonstration draws at the rate of the window and steps the engine at a
  rate the watcher chooses.
- A watcher pauses, resumes, chooses one of a small set of speeds, and asks
  for exactly one tick.
- A paused world still draws, so the camera still moves and the panel still
  reads.
- The picture path is unchanged. It steps a named count and writes one file.
- A test drives the clock through the public interface and asserts the tick
  count the engine reached, at each speed, paused, and after one step.

## Impact review

**Governed by.** ADR-0067 D4 holds that the viewer runs after the step, on the
stepping thread, and states that the drawing rate and the tick rate are one
number. ADR-0094 D1 holds that the caller owns the loop. ADR-0040 D1 holds that
Python is a control plane.

**The record ADR-0067 D4 states a property of the viewer, not of the caller.**
It says the viewer draws after the step and on the same thread. This work keeps
that: every draw still follows the steps of that frame, on one thread. What
changes is how many steps a frame runs, which the record does not fix. The
sentence about one number describes the demonstration loop that existed when
the record was written, so this item makes that sentence stale.

**The clock is in Python and this is why.** The loop belongs to the control
plane. A clock in the engine would be a value the engine holds for the viewer,
which the boundary record forbids.[^1] The clock names no entity, so it is not
a data plane.

**This work creates no decision that needs a record.** A speed set is cheap to
change and a reviewer needs no written constraint to reject a change to it.[^2]

**Blockers.** None.

**Registers.** No blocker opens or closes. One finding records that the
statement about one number no longer describes the demonstration.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`

## Outcome

**The window draws at its own rate and the world runs at the rate a watcher
chooses.** The control plane holds a clock. The clock states how many ticks a
frame owes, in thousandths of a tick for each frame, and the frame runs that
many. It steps nothing itself, so it names no entity and the engine holds no
copy of it.

**A watcher drives it from four keys.** The space bar pauses and resumes. The
full stop runs exactly one tick, whether the world is paused or not. The two
bracket keys move down and up a set of six speeds, from a quarter of a tick for
each frame to eight ticks for each frame. The frame states the speed beside the
tick, and the run prints the whole set before it opens the window.

**A paused world still draws.** The camera still moves, the panel still reads
and the tile panel still answers, because the drawing does not depend on a
step.

**The determinism rule is untouched.** The clock changes how many steps a frame
runs. It changes no step. The engine gives one answer for one seed at any
thread count, and a run at any speed reaches the same state at the same tick
count.

**The picture path is unchanged.** It steps a named count and writes one file.

A slower speed than one tick for each frame gives the frame a phase, and the
frame draws a unit that moved between its two tiles at that share.

The work landed under an earlier commit and this item was never moved.[^3]

## References

[^3]: Findings register, FND-526. `docs/FINDINGS.md`
