---
id: 0411
title: Record where a luxury lives
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0032]
blocked-by: []
---

## Why

The engine holds a luxury on a tile. A site reads the luxuries of the tile it
stands on, and it stores none of its own. The register holds that decision and
the two options it rejected.[^1]

The decision passes all three parts of the test for whether a decision needs a
record. A future contributor could reasonably put a luxury on a site, because
a site is sparse and the storage looks smaller. The choice is expensive to
reverse, because the field enters the state hash and a move invalidates every
stored hash. The reasoning is not visible in the code, because the code shows
only the shape that was chosen.[^2]

The record was not written with the work, because the registry allocates the
number and four other workers were writing at the same time. Taking a number
without the registry is the collision this project has already recorded three
times.[^3]

## What the work does

1. Take a number from the decision record registry, and set the row to
   `Draft` before writing the file.
2. Write the record. Title it with the claim, not with the subject.
3. State the constraint, the forces, the rejected options and the
   consequences. Cite the evidence in footnotes.
4. Repair the citation in the decisions register, so it names the record
   rather than the backlog item.

## What the record must state

- A luxury is a property of the ground, and the ground does not move.
- Level 0 is the only truth, so a level 1 cell can equal the exact
  combination of the tiles it covers.
- The variety of a faction is a fold over the tiles that faction holds, and
  that fold needs the luxury on a tile.
- A luxury on a site would move when the site moved, and a settlement that
  was lost would take its luxuries out of the world.
- Two homes for one fact is the defect shape this project meets most
  often.[^3]

## What the record must not state

- No count of luxuries, no storage figure and no cost figure. Those decay,
  and the reference tables hold them.[^2]
- No module arrangement. Record the constraint, not where the code lives.

## References

[^1]: Decisions register, DEC-201. `docs/DECISIONS.md`
[^2]: Decision Record Scope, sections 1 and 4. `.claude/rules/adr-scope.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
