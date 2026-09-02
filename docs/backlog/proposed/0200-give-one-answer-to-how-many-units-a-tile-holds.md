---
id: 0200
title: Give one answer to how many units a tile holds
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: [DEC-081]
---

## Why

**The engine holds two answers to one question.** One function composes the
capacity of the ground with the capacity a finished upgrade gives, and returns
the larger. Admission calls it, and so does the public reader that reports what
a tile holds. Three other callers read the ground alone: the bound on the
positions of a site, the seating of a founded group, and the room that the
founding survey estimates.

A fourth caller is the one a person sees. The drawing pass counts a painted
tile as at its capacity against the ground alone, and paints an over-full
marker above that number.

A finished made way states a capacity above every value in the terrain
table. On such a tile, admission admits more units than the position table
believes the tile holds, and a watcher reads a correctly filled tile as
over-full. Nothing fails when the two disagree.

The fold that reports the largest capacity is the sharp end. It says in its own
words that a caller which must hold one entry for each unit that can stand on a
tile reads it. It walks the terrain kinds only, so it does not see the crossing
capacity, and the row width of the position table is that fold. The guard that
clamps a site to the width carries a comment saying the clamp takes no effect
today. On a roaded tile it does.

The finding holds the evidence.[^1]

**No run reaches this today.** No engine rule issues a build order, so an
upgrade exists only when the control plane orders one.[^2] The divergence is
reachable through the public interface and is not reached by the
demonstration.

## What is missing before this is refined

- The impact review.
- **The register row must close first.** It holds whether the three callers ask
  a different question or the same one, and it recommends that they ask a
  different one.[^3] The work follows the answer and must not invent it.
- Which caller, if any, changes. The recommendation changes the fold and no
  caller, and it must still answer the drawing pass, because a false marker is
  what a watcher sees.
- What a test asserts. A test that builds a road, admits units onto the tile,
  and reads the position bound would show the two answers in one run.
- Whether the record that governs the composition needs its own change. A
  review already removes the sentence that claims every caller composes, and a
  second confirms the correction.[^4] [^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-193. `docs/FINDINGS.md`
[^2]: Backlog item 0180, let a unit choose to build. `docs/backlog/proposed/0180-let-a-unit-choose-to-build.md`
[^3]: Decisions register, DEC-081. `docs/DECISIONS.md`
[^4]: Review 0199, the influence, tile field, upgrade and housing records. `docs/reviews/0199-the-influence-tile-field-upgrade-and-housing-records.md`
[^5]: Review 0204, the two corrected records. `docs/reviews/0204-the-two-corrected-records.md`
