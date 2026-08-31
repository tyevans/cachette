---
id: 0023
title: Implement sort-then-admit movement
status: complete
created: 2026-08-30
implements: [ADR-0056 D2, ADR-0056 D3, ADR-0056 D4, ADR-0007 D2]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A move is an intent. A separate admission step grants it, sorts by a stable
key, admits in that order, and respects the tile capacity.

Until this work the step admitted every intent. Two soldiers could choose one
tile and both enter it, and any number could stand on one tile.

This is where determinism is easiest to lose and hardest to see, so the
thread-count test must cover a tile that is oversubscribed.

## Impact review

**Governed by.**

- ADR-0056 D2: a move is an intent, and a separate step admits it. The two
  halves never interleave.
- ADR-0056 D3: admission sorts by target tile then by unit identity, scans
  each segment in that order, reads the occupancy of a target from the derived
  structure, releases room only for an admitted departure, applies the
  departures after the scan, and runs a fixed number of passes.
- ADR-0056 D4: the capacity is a property of the ground, read from the terrain
  table. The movement kernel holds no capacity value.
- ADR-0007 D2: the sort takes a key vector, never a comparator. Admission uses
  the bounded key sort, whose ordering field is the tile index.
- ADR-0018 D3: the derived structure rebuilds at the barrier, and admission
  reads what the last barrier built.
- ADR-0005 D1: a solver runs a fixed iteration count. The pass count is fixed
  and declared. The engine never runs to a fixpoint.
- ADR-0001 D4: one binary gives one answer at any thread count.

**Changes.** No record changes.

**Creates.** No record. Every claim this work implements is already written.
Three choices came up that no record holds, and each is a register row rather
than a record, because each is a value or a call site rather than a
constraint.

**Blockers.** BLK-009 is resolved and the scale constants table holds the
capacity values. This work reads them through the terrain table and states no
figure of its own.

**What this does not do.** Item 0039 holds D5. ADR-0056 D5 says a rejected unit is not stuck: the
engine counts rejections and above a threshold the unit steps sideways or
replans. Nothing implements that. It needs a rejection count for each unit,
which is a column and therefore a storage decision, and it is raised as its
own item.

## Outcome

Admission is built and every decision above is honoured.

**The rejected departure is the whole of D3.** An intent is not a departure.
Only an admitted departure releases room. The record names the failure: three
tiles in a line, the middle and the far tile both full, the unit in the middle
rejected at the far tile, and a unit behind it admitted into the middle on the
strength of a departure that never happened. Both that defect and a rule that
ignores the capacity were put back, and two tests failed on each.

**Neither determinism test could see the rule.** Every populated golden
scenario spread its units over open ground, so no target ever reached its
capacity, and the golden files did not move when admission landed. A `crowd`
scenario was added that fills a patch of ground to the capacity of each tile.
Removing the capacity check changes that file and no other. The thread-count
test gained the same shape, because the item asks for an oversubscribed tile
and a spread population is not one.

**The invariant is that no tile gains a unit beyond its capacity.** It is not
that no tile is ever above its capacity. A spawn does not read the capacity,
so a caller may over-fill a tile. The register holds that open choice.[^2]

**Two call sites now rebuild the derived structure.** Admission reads the
occupancy of a target from it, and a spawn made between two frames leaves it
stale. The step opens by giving those changes their barrier. The rebuild at
the end is still the barrier of the frame and still runs last. The register
holds the choice and an item holds the record it may need.[^3] [^4]

**The pass count is content.** Two passes admit a chain of two. The register
holds the value and what would settle it.[^1]

**Admission costs about twice the step it was added to.** The figures are in
the commit message, because a measured figure decays and a commit message does
not.[^5] They come from a development machine and not from the target, so they
say the shape and not the number.[^6] The largest single term is the ground: a
frame generates the ground of a target twice, once to test whether it admits a
unit and once to read its capacity. An item holds the two ways to remove the
second read.[^7]

## References

[^1]: Decisions register, DEC-019. `docs/DECISIONS.md`
[^2]: Decisions register, DEC-020. `docs/DECISIONS.md`
[^3]: Decisions register, DEC-021. `docs/DECISIONS.md`
[^4]: Backlog item 0040. `docs/backlog/proposed/0040-record-where-an-out-of-frame-change-gets-its-barrier.md`
[^5]: Commit Message Rules. `.claude/rules/commits.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: Backlog item 0041. `docs/backlog/proposed/0041-read-the-ground-once-for-each-target.md`
