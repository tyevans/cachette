---
id: 0304
title: Stop a refused direction from freezing a unit
status: complete
created: 2026-09-03
implements: [ADR-0003 D1, ADR-0004 D1, ADR-0022 D1, ADR-0024 D4, ADR-0068 D4, ADR-0091 D1, ADR-0091 D4]
changes: [ADR-0091]
creates: []
serves: [PRD-0009]
blocked-by: []
---

## Why

**A unit against a shoreline stood still for ever.** The exit field holds one
direction for each level 1 cell and each option, and a cell covers a block of
32 tiles on a side. The ground under one tile of that block may refuse the
direction the block holds. The movement pass answered that refusal by writing
no intent, so the unit stayed put.

Every input to the direction holds from one frame to the next: the cell, the
option and the ranked summaries. The refusal therefore repeats exactly, and the
unit is not delayed by one frame. It is stopped.

A second cause sat above it. No summary field says whether a unit may stand in
a cell, so the rank could name a cell that admits nobody at all. The findings
register holds both, with the measurement.[^1]

## What the work did

1. A cell that admits no unit is dropped from the rank before any value is
   compared.
2. A unit whose direction the ground refuses takes a keyed draw at the next
   draw index of the same system and frame, and steps that way instead.

## Impact review

**Governed by.**

- **ADR-0091 D1.** No unit reads a neighbouring cell and no unit scores one.
  The fall-back reads one tile, which is the tile the unit would step onto, and
  the pass already read that tile to answer the ground. No neighbourhood is
  searched.
- **ADR-0091 D4.** The rank compares strictly in ascending direction index and
  starts at the value of the cell itself. The refusal of a closed cell happens
  before the comparison, so the tie-break rule is untouched.
- **ADR-0003 D1.** Every draw is keyed on the system, the frame, the entity and
  the draw index. The fall-back takes its own index, because reusing the first
  index would hand the unit the direction that was just refused.
- **ADR-0004 D1.** Iteration order is explicit. Neither change reads a thread
  or a join, and the two determinism tests pass.
- **ADR-0068 D4.** Ground that admits no unit is stated by the capacity table.
  Both changes read that table through the passability rule and the open tile
  count, and neither writes a second rule.
- **ADR-0024 D4.** The open tile count is the count the open share reads. The
  rank reads it rather than testing passability again.
- **ADR-0022 D1.** Level 0 is the only truth. The exit field is derived again
  at each rebuild and carries nothing.

**Changes. ADR-0091 gains D5 and D6.** The record is a draft, so it was edited
in place rather than superseded.[^2] D4 already said that the field leaves no
unit without a rule. That was true of the case it named, and the rule it left
the other unit was a direction nothing could take.

**Creates.** No record. Both changes sit inside a record that already exists.

**Blockers.** None. BLK-007 governs the cost figures this work would state, and
it states none.[^3]

## Done when

- No cell exit names a cell that admits nobody. A test asserts it over the
  whole field.
- A unit whose cell exit the ground refuses leaves the tile it started on. A
  test drives the step and does not call the movement pass.
- Each test was watched to fail with its own defect put back, and to pass with
  the other defect put back. Both were.
- The fixture reaches the case. The closed-ground test runs at a seed measured
  to hold one, because the ordinary seed does not.
- The two determinism tests pass and the golden files record the new
  behaviour.

## Outcome

Both tests pass, and each was proven to fail with its own defect restored. The
golden state hash moved in two scenarios: the crowd scenario diverges at its
last frame and the gathering scenario at frame 14, with every earlier frame
unchanged.

**The first version of the closed-ground test could not fail.** Written at the
seed this project uses elsewhere, it passed with the defect put back, because
no cell of that world ranks closed ground above itself. A sweep of forty seeds
found the case in eleven. The test now runs at one of them.

**Neither change routes a unit around an obstacle.** A field at block pitch
cannot answer a tile. That is a different claim and it has no record.

## References

[^1]: Findings register, FND-315. `docs/FINDINGS.md`
[^2]: ADR Registry, the status rule. `docs/adrs/REGISTRY.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
