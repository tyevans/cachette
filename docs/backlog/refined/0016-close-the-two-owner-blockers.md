---
id: 0016
title: Close BLK-013 and BLK-014, and re-derive the rows they governed
status: refined
created: 2026-08-30
implements: []
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

The project owner answered both open blockers. The world is a rhombus. The
maximum faction count is 63 or fewer, so a faction is a bit in one 64-bit
mask with one value reserved for no faction.

Two rows in the decision registry were written under the assumption the
blockers left. Row 0017 says a tile is indexed by odd-r offset. The rhombus
answer removes the conversion that row exists to manage, so the row states
the wrong claim. Row 0053 says a faction is a bit in a mask, and the answer
confirms it.

A blocker that is answered and not closed is worse than an open one, because
a reader sees a live question that the project has already settled.[^1]

## Impact review

**Governed by.** No record governs a register. The definition of done
requires that a resolved blocker has its row closed and its outcome recorded
in the same change as the work.[^2]

**Changes.** No accepted record. Row 0017 changes its claim in the registry.
The row is `Proposed` and has no file, so nothing is superseded and no number
is retired.[^3]

**Creates.** None. Item 0017 writes the record for the re-derived row.

**Blockers.** This item closes BLK-013 and BLK-014. It opens none.

**Precedent.** FND-039 records that the orientation held open questions that
no register held. The orientation now points at the register, so closing the
rows is the only edit the answer needs there.[^4]

**Serves.** PRD-0002. The renderer cannot map a tile to the screen until the
world shape is fixed.

## Done when

- BLK-013 and BLK-014 move to the resolved section, each with the outcome and
  the reasoning, in the same commit as the rows they unblock.
- The faction ceiling and the world shape are in the scale constants table,
  marked as decided rather than derived.[^5]
- Registry row 0017 states the rhombus claim.
- A finding records what the project believed about the tile index and what
  is true, because row 0017 stated the opposite claim before this change.
- Every document that names an open blocker count agrees with the register. A
  whole-tree search for the two numbers comes back consistent, and the search
  command is in the commit body.
- The record check passes.

## Outcome

Filled in on completion.

## References

[^1]: Blockers register. `docs/BLOCKERS.md`
[^2]: Definition of Done. `.claude/rules/definition-of-done.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Findings register, FND-039. `docs/FINDINGS.md`
[^5]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
