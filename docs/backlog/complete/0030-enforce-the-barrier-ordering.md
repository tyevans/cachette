---
id: 0030
title: Enforce the barrier ordering that ADR-0018 calls a decision
status: complete
created: 2026-08-31
---

ADR-0018 states that the bridge rebuild runs after the structural apply at
the barrier, and calls that ordering "a decision and not an implementation
detail". Its central consequence follows from it: every identity in the unit
array is live for the whole frame, so no caller pays a resolution branch.

**In the code that ordering is between one operation and nothing.** No
structural apply exists in the step yet. The rebuild is last because nothing
follows it, and a comment says a later apply goes above it. Nothing fails if
a contributor puts one below.

The consequence is therefore vacuously true, not enforced. When the
structural apply lands, a test must drive the step through a despawn and
assert that no dead identity reaches the unit array. A comment is not the
mechanism this project accepts for this class of fact.

Found by the review of item 0020.

## Impact review

**Governed by.** ADR-0018 D3 states that the rebuild runs at the barrier,
after the structural apply, and calls the ordering a decision. Its consequence
is that every identity in the unit array is live for the whole frame.

**Changes.** No record changes.

**Creates.** No record. The record states the constraint; this work is the
mechanism that enforces it.

**Blockers.** None.

## Outcome

The structural apply exists now: the admission step places every unit it
granted. The ordering was still not enforced, and reversing it changed
nothing.

**Two callers refreshed the structure, and the second hid the first.** The
step refreshed at the barrier, and the level 1 rebuild refreshed again at its
own start. A barrier that ran in the wrong order left the structure stale, and
the second refresh repaired it quietly. One decision in two places, with
nothing that fails when they disagree.

The step now refreshes once and calls the level 1 rebuild directly. The
rebuild refuses a stale structure, so a barrier out of order is refused rather
than repaired. The public wrapper still refreshes, because its purpose is a
caller that changed level 0 outside a frame.

**Four tests read the ordering from outside.** A rebuild that ran before the
apply leaves the structure describing the arena as it was, and the arena has
moved since, so it is stale when the step ends. That is what they read. One
drives the step through a despawn, which is the case the item named. One
asserts that the fixture moves a unit in every frame, because a frame that
moved nothing leaves the structure fresh whatever order the barrier ran in.

Reversing the ordering fails all four.
