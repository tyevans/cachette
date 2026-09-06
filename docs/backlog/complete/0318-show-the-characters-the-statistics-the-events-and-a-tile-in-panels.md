---
id: 0318
title: Show the characters, the statistics, the events and a tile in panels
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: []
---

## Why

**A faction that dies is invisible until it is gone.** The panel counts the
units the drawing pass painted, by colour, in the window. It states no count of
a faction in the world. A faction whose last unit starves therefore vanishes
from the picture with no number falling to zero anywhere.

**The engine keeps four logs of the last tick and the panel reports two
counts from them.** A unit that starved, a store that rationed, a soldier that
was promoted and a site that fell short are each recorded, and a watcher sees a
total rather than what happened.

**A character exists and almost nothing about one is visible.** The panel gives
the count, the newest one, and the deeds that earned the last promotion. The
sex, the descent line, the house and the unit a character belongs to are all
held and none reaches a reader.

**The panel names the tile under the middle of the window.** A watcher cannot
point at a tile.

## Done when

- A per-faction population is one read, not a walk. The engine holds it.
- A panel states the population of each faction in the world, and the label
  says that it is a count of the world.
- A panel states what the last tick logged, newest first, with a bound on how
  many rows it reads.
- A panel states what is known about one character and states nothing that no
  pass writes.
- A panel states the tile the watcher pointed at, its ground, its stock, its
  holder and the units on it.
- Each panel is one file and registers with the standard of item 0316.
- Each panel has a test that renders it and asserts on the lines it holds.

## Impact review

**Governed by.** ADR-0070 D1 holds that a panel starts no pass over the world
and that its cost never follows the world. ADR-0070 D2 holds that a number
nobody computed is stated as absent, never as a zero. ADR-0054 D1 holds that a
caller may walk the character tier and may not walk the mass tier. ADR-0014 D2
holds that an identity is a slot and a generation. ADR-0040 D1 holds that
Python never loops over entities.

**A per-faction population cannot be a walk, and that decides where it goes.**
Counting the units of a faction reads every live unit. At the target scale that
is one million reads for one row of one panel, every frame, which ADR-0070 D1
forbids. The engine therefore holds the count and maintains it where a unit is
created and where a unit ends. The count is derived state, so an invariant
compares it against the columns and fails when the two disagree.[^1]

**Python must not compute it either.** A loop in the control plane crosses the
boundary once for each unit, which ADR-0040 D1 forbids. The binding
returns the whole vector in one call.

**A character panel walks the character tier, and ADR-0054 D1 permits exactly
that.** The tier holds a bounded population. The panel reads a bounded number
of rows from it whatever the tier holds.

**This work creates no decision that needs a record.** Which numbers a
demonstration shows is not a constraint a reviewer needs in writing.[^2] The
one thing that could have been a decision, that a per-faction population is
maintained rather than counted, is the direct consequence of ADR-0070 D1 and
adds nothing to it.

**Blockers.** None govern a value here.

**Registers.** One finding records that no per-faction population existed. No
blocker opens or closes.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`

## Outcome

**Four panels of the deck answer the four questions, and each is one file that
registers with the standard of item 0316.**

**The statistics panel counts the world, not the window.** It states the people
in the world, then the people of each faction beside its colour, then the tiles
each faction holds and the number of sites. A faction whose last unit dies now
falls to zero on a row a watcher can see. The per-faction population is one
read, because the engine holds the count and maintains it where a unit begins
and where a unit ends.

**The events panel states what the last tick logged.** It names the promotions,
the shortages, the rationing and the sites that fell short, newest first, and it
reads a bounded number of rows. A watcher who pauses reads the moment rather
than a total that went up.

**The characters panel names the people.** It gives the population, then a
bounded number of characters with the faction colour and the sex, then one
focused character with the birth tick, the house and the parents. It states
nothing that no pass writes.

**The tile panel answers where the watcher pointed.** It gives the address, the
ground, the food, the wood and the stone left of what the tile admits, the
faction that holds it and the units on it. A mouse press points at a tile, and
the command line points at one without a mouse.

**Each panel is bounded, so its cost never follows the world.** No panel starts
a pass over the tiles or over the units.

Each panel holds its own tests in the viewer. The four together were proved
outside a window through the command line of the demonstration, which now names
the panels and the tile.

The work landed under earlier commits and this item was never moved.[^3]

## References

[^3]: Findings register, FND-526. `docs/FINDINGS.md`
