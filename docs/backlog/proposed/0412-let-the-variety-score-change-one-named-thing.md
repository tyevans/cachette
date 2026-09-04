---
id: 0412
title: Let the variety score change one named thing
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0032]
blocked-by: [BLK-110]
---

## Why

The engine counts how many different luxuries stand on a tile, in a level 1
cell, on the ground one faction holds, and in the whole world. Nothing in the
engine reads any of those numbers. The control plane reads them, and a game
built on the engine makes its own rule from them.[^1]

The project owner suggested that variety could change the worker policy of a
faction, and he said that he did not know. A blocker holds the question.[^2]

**Do not start this item while that blocker is open.** The rule on records
forbids inventing a value that an unanswered question governs.[^3] A rule
invented here would reach every world, and no measurement and no record would
choose it.

## What the work does, once the blocker closes

1. Take the effect the owner named. Wire the score to that one pass, and to no
   other.
2. Write the decision record for the rule. State what reads the score, what
   the score changes, and by how much.
3. Put the numbers in the reference tables and cite them. A record holds no
   value that a measurement can change.[^3]
4. Test what the effect depends on. Change the variety and the outcome must
   change. A test that only proves the outcome repeats proves nothing.[^4]
5. Regenerate the golden state hash files, because the behaviour moves.

## The candidate the owner named

A site opens positions in proportion to what it lacks, and what a site wants
is a row of targets, one for each kind of gatherable work. Variety could raise
a target. The rise is content, and it is exactly the value the blocker holds.

## References

[^1]: Decisions register, DEC-200. `docs/DECISIONS.md`
[^2]: Blockers register, BLK-110. `docs/BLOCKERS.md`
[^3]: Decision Record Scope, sections 4.1 and 4.5. `.claude/rules/adr-scope.md`
[^4]: Testing rules, section 2. `.claude/rules/testing.md`
