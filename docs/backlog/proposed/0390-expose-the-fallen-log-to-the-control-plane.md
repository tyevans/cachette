---
id: 0390
title: Expose the fallen log to the control plane
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

The engine now resolves a meeting between two factions and writes one event for
each unit that fell. The event names the tick, the unit, the tile, the faction
and the type. **No binding reads it**, so a caller in the control plane watches
its faction population fall and cannot see where or to what.

A fight that nobody can read is a fight that nobody can repair. The research
report names the event as one of the five things a contest needs, and it names
reading it as the reason.[^1]

Three other logs have the same gap, and one item already holds them: the log of
a unit a shortage ended, the log of a unit a step promoted, and the log of a
site that fell short.[^2] This is a fourth log with the same shape, and the
three logs that are exposed already set the form a new one takes.

## What is missing before this can be refined

- Whether this is its own item or a row of item 0319. The four logs share one
  shape and one repair, and refining either should decide.
- What columns the control plane wants. The event carries five fields, and the
  exposed logs return a dictionary of arrays rather than every field.

## References

[^1]: Research report 21, what a god needs from this engine, section 4.5. `docs/research/reports/21-what-a-god-needs.md`
[^2]: Backlog item 0319, expose the three logs the control plane cannot see. `docs/backlog/proposed/0319-expose-the-three-logs-the-control-plane-cannot-see.md`
