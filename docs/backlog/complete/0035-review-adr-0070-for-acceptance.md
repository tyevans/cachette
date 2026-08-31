---
id: 0035
title: Review ADR-0070 for acceptance
status: complete
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

The review is written and it recommends acceptance without a single
amendment.[^1] **The record stays a draft, and that is the finding.**

**The record holds against the code, name by name.** Every number the panel
states was traced to one of the three sources D1 names. There is no loop over
the world anywhere in the reporting code: every loop in the panel walks its
own arrays, six faction slots, five ground kinds, and its own list of lines.
The live population is a stored count and the region readout is one array
element. D2 holds too: no world-wide count appears, nothing is sampled, and a
reading the engine cannot give prints a dash.

**D1 and ADR-0067 D2 are separate constraints.** Each has a violation the
other permits. A census kept on the world violates ADR-0067 D2 and satisfies
ADR-0070 D1. A loop over the population inside the reporting code violates
ADR-0070 D1 and satisfies ADR-0067 D2. The item asked for this check and the
answer is to keep them apart.

**The strongest evidence came from a case the record predicted.** The panel
now reports a summary of a block of tiles, which reaches beyond the window and
is therefore a number about ground the drawing pass did not paint. The record
answers it already: showing one needs a structure the engine already
maintains, and never a new engine field. The case arrived after the record was
written and fitted without an amendment.

**It stays a draft because its dependency is a draft.** ADR-0070 rests on
three decisions of ADR-0067, and a draft binds nothing. An accepted record
whose foundation binds nothing can be made false by an edit to a file nobody
is required to leave alone. Accepting it with a note would put the condition
in prose that nothing checks, and the status vocabulary has no value meaning
"accepted, but".

An item carries the review of ADR-0067, and ADR-0070 is accepted in the same
change if that review accepts it.[^2]

## References

[^1]: Review 0035, the head-up display record. `docs/reviews/0035-the-head-up-display-record.md`
[^2]: Backlog item 0047. `docs/backlog/complete/0047-review-adr-0067-for-acceptance.md`
