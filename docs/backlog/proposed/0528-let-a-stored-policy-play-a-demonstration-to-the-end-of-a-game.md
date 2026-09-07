---
id: 0528
title: Let a stored policy play a demonstration to the end of a game
status: proposed
created: 2026-09-07
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A watcher can seat a stored policy and cannot watch it finish.** The
demonstration now gives a faction to a weight file, and the faction acts. The
observation reader then refuses, and the run stops with a view error that says
the world cannot describe its own units.

Three probes place the condition. A world that steps with nobody acting reads
every observation for four hundred ticks and never refuses. A world with one
seated policy takes fifty-one decisions over five hundred ticks and never
refuses. A world with three seated policies refuses at tick one hundred and
ten. No renderer ran in any of the three, so the drawing pass is not involved.
The findings register holds the readings.[^1]

The verbs the three policies ran were the settling verb, the queue verb and the
build verb. Each of those adds a unit or a site, so the derived unit structure
is the first thing to look at.

## What refining must answer

- Which pass rebuilds the derived unit structure, and what the observation
  reader needs of it that a rebuild has not yet given.
- Whether the condition is a missing rebuild, or a reader that asks for the
  structure at a tick when no rebuild is owed.
- Whether the learner environment can reach it. It runs the same reader after
  the same verbs, and no training run has reported it, so the difference must
  be named rather than assumed.
- Which decision records govern the derived unit structure and its lifetime.

## References

[^1]: Findings register, FND-644. `docs/FINDINGS.md`
