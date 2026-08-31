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
