---
id: 0030
title: Enforce the barrier ordering that ADR-0018 calls a decision
status: proposed
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

Refine this with the item that adds the structural apply. Found by the review
of item 0020.
