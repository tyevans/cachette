---
id: 0047
title: Review ADR-0067 for acceptance
status: refined
created: 2026-08-31
implements: []
changes: [ADR-0067]
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

ADR-0067 states that the viewer reads the world and never writes to it. It is
a draft, so nothing may cite it as binding.

Two records rest on it. ADR-0070 cites three of its decisions as the
boundaries it extends, and the review of ADR-0070 found the record sound and
recommended against accepting it, because an accepted record whose foundation
binds nothing can be made false by an edit to a file nobody is required to
leave alone.[^1] ADR-0068 D4 cites it once for the same reason.

The viewer crate exists and is substantial. The record can be read against the
code rather than against the intent, which is the first thing a review here
must do.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free: read the record against the
code, be an agent that did not write it, and state what it tried to reject.

**Changes.** ADR-0067 moves from `Draft` to `Accepted`, or the review returns
it with objections. The registry row and the file location change with it, and
every citation of the draft path moves. The citation check finds them.

**Creates.** None.

**Blockers.** None. ADR-0067 states no cost figure.

**Precedent.** A review of ADR-0070 already exists and holds the shape.[^1] It
also holds one thing this review must resolve rather than repeat: whether
ADR-0067 D2 and ADR-0070 D1 state one constraint or two. That review concluded
two, from the side of ADR-0070, and this one must reach the same conclusion
from the other side or say why not.

## Done when

- An agent that did not write ADR-0067 has read all five of its decisions
  against the viewer crate.
- The review states what it tried to reject, and why each rejection failed.
- D4 is checked against the demonstration binary, because it ties the drawing
  rate to the simulation rate and names what would supersede it.
- D3 is checked against the tree: no value that has been a floating point
  number reaches the engine, in any form.
- The registry row holds the outcome, and ADR-0070 is accepted in the same
  change if ADR-0067 is.

## References

[^1]: Review 0035, the head-up display record. `docs/reviews/0035-the-head-up-display-record.md`
