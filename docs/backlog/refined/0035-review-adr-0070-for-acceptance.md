---
id: 0035
title: Review ADR-0070 for acceptance
status: refined
created: 2026-08-30
implements: []
changes: [ADR-0070]
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

ADR-0070 states that the head-up display adds no pass over the world. The
panel that implements it is in the viewer crate, and the tests hold the
claim. The record is a draft, so nothing may cite it as binding.

An author may set `Draft`. Only a reviewer may set `Accepted`, and the
reviewer must be an agent that did not write the record. The record was
written together with the code it governs, so the author cannot be surprised
by it. The review must find what surprise would have found.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free. ADR-0067 D2 is the record
ADR-0070 sits next to, and the reviewer must check that the two do not
overlap or contradict.

**Changes.** ADR-0070 moves from `Draft` to `Accepted`, or the review returns
it with objections. The registry row changes with it.

**Creates.** None.

**Blockers.** None. ADR-0070 states no cost figure, so BLK-007 does not hold
it.

**Precedent.** The registry records three record-number collisions and the
delegated review rule that followed them. A review that lists no attempted
objection did not happen.

## Done when

- An agent that did not write ADR-0070 has read it against the viewer crate,
  not against the intent.
- The review states what it tried to reject, and why the rejection failed.
- The review confirms that ADR-0070 D1 and ADR-0067 D2 state different
  constraints, or it recommends merging them.
- The review confirms that the code does what the record says, name by name.
- The registry row holds the outcome.

## Outcome

Filled in when the item moves to `complete/`.
