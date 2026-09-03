---
id: 0039
title: A rejected unit is not stuck
status: proposed
created: 2026-08-31
---

Admission refuses a unit whose target is full. The unit holds its tile and
asks for the same tile on the next frame, against the same crowd.

ADR-0056 D5 says the engine counts the rejections of a unit, and that above a
threshold the unit takes a lateral step or marks its plan stale and plans
again.[^1] The lateral step comes from the keyed generator, on the tuple of
system, frame, entity and draw.

Nothing implements this. Admission is built and D5 is not, so a unit in a
crowd is stuck until the crowd moves.

The work needs a rejection count for each unit, which is a column in the
arena and therefore a storage decision. The entity storage record fixes the
four shapes and their column sets, so the impact review must say whether a
rejection count is a column of the soldier shape or a value the movement
system holds for the frame it runs in.[^2]

The threshold is content. Refine it as a register row with an assumption, in
the way the admission pass count was.

## The reason this item was held back is gone

This item carried a section titled "Do not build this yet". Its reason was
that a unit has no plan, so a unit draws a fresh uniform direction on each
frame and does not repeat the choice that failed.

**That reason is false.** A unit takes its direction from the exit of its own
cell, for the option it chose.[^3] The cell, the option and the direction all
hold from one frame to the next, so a unit refused at one target asks for that
same target again. The findings register holds the same mechanism seen from the
ground side: a direction the ground refused repeated on every frame and a unit
stood still for ever.[^4]

**The ground fall-back does not cover this case, and it says so.** A direction
the ground refuses now falls back to a keyed draw at the next draw index.[^4] A
target that is full is a different refusal, and the movement pass states in its
own words that admission owns it. So the repeat that D5 exists for is the one
case that no fall-back reaches.

**The repeat happens in the world the demonstration builds.** Driven for 400
ticks, a unit held its tile against a target the ground admits 61 times, and
one unit was refused on five consecutive frames. The commit body holds the
command and the numbers.

## What refining this must answer

- Whether the rejection count is a column of the soldier shape or a value the
  movement pass holds for one frame. The count must survive the frame barrier
  to reach a threshold above one, so the second answer needs an argument.
- What the threshold is, and which register row holds it.
- Whether the lateral step is a draw over the six neighbours or a draw over the
  neighbours that are not the refused one. A draw that can return the refused
  direction repeats the refusal with probability one in six.
- What the state hash does when a rejection count enters the unit row, and
  which golden file moves.

## References

[^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D5. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/complete/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^4]: Findings register, FND-315. `docs/FINDINGS.md`
