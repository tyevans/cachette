---
id: 0039
title: A rejected unit is not stuck
status: proposed
created: 2026-08-31
---

Admission refuses a unit whose target is full. The unit holds its tile and
draws again next frame, against the same crowd, and it may draw the same
answer for many frames.

ADR-0056 D5 says the engine counts the rejections of a unit, and that above a
threshold the unit takes a lateral step or marks its plan stale and plans
again. The lateral step comes from the keyed generator, on the tuple of
system, frame, entity and draw.

Nothing implements this. Admission is built and D5 is not, so a unit in a
crowd is stuck until the crowd moves.

The work needs a rejection count for each unit, which is a column in the
arena and therefore a storage decision. The entity storage record fixes the
four shapes and their column sets, so the impact review must say whether a
rejection count is a column of the soldier shape or a value the movement
system holds for the frame it runs in.

The threshold is content. Refine it as a register row with an assumption, in
the way the admission pass count was.

## Do not build this yet

A unit has no plan. It draws a uniform direction from the keyed generator each
frame, so a unit refused at one target draws a fresh direction on the next
frame and does not repeat the choice that failed.

The condition D5 exists for is a unit that keeps trying the same next tile,
and that condition needs a plan. Building a rejection count now would add a
column, a threshold and a lateral step that nothing needs, and the project has
a name for that shape: a capability nobody invokes.[^1]

**Refine this with the work that gives a unit a plan.** Until then D5 is
unimplemented on purpose, and this item is the record of why.

## References

[^1]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
