---
id: 0024
title: Write the viewer record
status: complete
created: 2026-08-30
---

Registry row 0067 holds the claim: the viewer reads a published frame and
never writes to the world.

The record must state what the viewer may do with floating point. ADR-0002
allows floating point in rendering, because rendering does not feed back into
the world. The viewer is where that permission is used, so the boundary needs
stating once, where a reviewer can find a violation.

Refine this at sprint 4 planning.

## Impact review

**Governed by.** ADR-0002 D4 allows floating point outside simulated state,
and the viewer is where that permission is first used. ADR-0001 owns the
determinism claim the viewer must not touch. ADR-0017 D4 says the engine
stores the shape and the viewer draws it. PRD-0002 states the need.

**Changes.** Registry row 0067 changed its claim. It read "the viewer reads a
published frame and never writes to the world", which named a mechanism the
project decided not to build yet. The row was `Proposed` with no file, so
nothing was superseded.

**Creates.** ADR-0067.

**Blockers.** None. BLK-007 governs every cost figure and the record states
none.

## Done when

- The record states one claim a reviewer could reject on its own.
- It says where floating point begins and that it never returns.
- It says what the engine may not hold because a viewer wants it.
- It states the rejected alternative for the frame handoff, and what would
  supersede the decision.
- The record check and the citation check pass.

## Outcome

ADR-0067 is written with five numbered decisions and the row is `Draft`.

Two things changed from the plan. The plan expected the record to state that
the viewer reads a published frame, which is what the registry row claimed.
The project owner chose the simpler handoff at sprint 4 planning: one loop
steps the engine and then draws. The row now states what the record decides.

The record also carries a decision the plan did not anticipate. The viewer
lives in its own crate, and the core never depends on it. That is what makes
"the engine holds no value that exists for the viewer" a compiler check
rather than a reviewer's judgement, which is the same argument the project
already used for the Python boundary.
