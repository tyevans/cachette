---
id: 0349
title: Let a command change the faction of a set of units
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-050]
---

## Why

A soldier carries a faction, and nothing changes it after a spawn. A downstream
game names converting people as one of the six things its players must do.

**Half of that need is one column write.** Every downstream reader already reads
the faction column: the holding rule, the population count, the return field and
the founding. So the write itself is small.

**The other half is content and nobody has stated it.** A conversion in a god
game is the outcome of belief, proximity, persuasion and time, and the engine
models none of those. One blocker holds what the game means by it.[^1]

This item is the command alone. It gives the control plane a way to move a set
of units from one faction to another, and it decides nothing about when that
should happen.

## What is missing before this can be refined

- What the game means by conversion.[^1] The command is useful either way, but
  an item that claimed to answer the need would be claiming a gap it does not
  close.
- What a conversion does to what the unit holds: its home site, its position at
  a site, its carried load and its gather order. Each of those names a faction
  somewhere.
- Whether an event records it. The engine writes a log when a unit starves and
  when a unit is promoted, and the bindings expose neither.[^2]

## References

[^1]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^2]: Backlog priority index, item 0319. `docs/backlog/PRIORITY.md`
