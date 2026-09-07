---
id: 0517
title: Build the action table and the legality answer, and let a caller act by one integer
status: proposed
created: 2026-09-06
implements: [ADR-0154 D4, ADR-0154 D5, ADR-0154 D6, ADR-0176 D1, ADR-0176 D2, ADR-0176 D3, ADR-0176 D4]
changes: []
creates: []
serves: [PRD-0056]
blocked-by: [BLK-007]
---

## Why

**A learner needs the set of actions it may take, in a form a network can
index.** An accepted record says an action is one integer that indexes a bounded
table the engine declares, that the engine answers which rows are legal at this
tick, and that one log records the controller's choice and the learner's action
in one encoding.[^1] [^2] [^3]

**Nothing implements any of it.** No action table exists, no legality reader
exists anywhere in the tree, and no verb takes an action integer. A learner can
drive a loop today only by calling the ordinary verbs one at a time, and it has
no answer about which of them would refuse.[^4]

**One decision of that record must be repaired before this starts.** The record
factorises the table into a verb, a target and a magnitude. The built-in
controller's choice enumeration does not have that shape: one choice names two
things, several name nothing, and none carries a quantity. A later record
replaces the factorisation with a per-verb list of argument positions and keeps
the rest.[^5] **That record is a draft, and a draft binds nothing.** This item
cannot be refined until a reviewer accepts it or rejects it.

**It depends on item 0495.** A legality answer must not name a target the
faction cannot see, so the readers that answer for one faction come first.[^6]

**This is the largest item of the learner seat.** The record forbids restating a
refusal rule: the answer reads the same candidate lists and the same refusal
rules the verbs read, and it duplicates no rule.[^2] Those rules live inside the
verbs, spread through the world module, so honouring the record means giving
each verb a path that reports a refusal without acting. A survey names that as
the hidden cost of the record, and warns that anybody who sizes this work from
the record alone will underestimate it.[^4]

## What is missing before this can be refined

- **The record it implements is not accepted.** ADR-0176 changes ADR-0154 D4,
  and it carries the status `Draft`.[^5] [^7] The work waits on the review. If a
  reviewer rejects it, this item implements a different encoding and the front
  matter of this file changes with it.

- **How each verb reports a refusal without acting.** The record forbids a
  second copy of any refusal rule.[^2] The work must say which shape it takes:
  a check function beside each verb that the verb itself calls, a dry-run flag
  on the verb, or a single pass that resolves every candidate list and asks each
  verb once. The first two are one declaration site and the third may not be.
  The record names the failure and not the remedy.

- **What the legality answer costs, and whether a learner may ask for it every
  tick.** The answer is one byte for each row of the table, and it must resolve
  every candidate list of every verb to produce them.[^2] No figure for that
  exists, and one blocker says every cost figure of this project stays derived
  until the target platform measures it.[^8] The work must express the cost
  parametrically and cite that blocker.

- **What the log layout change costs.** The record requires one row that carries
  the whole action integer, so that no field of an action lives anywhere
  else.[^3] The controller command row holds a kind and a one-byte argument
  today, so the row widens. It is plain data with declared padding that enters
  the state path, so the change moves every golden file and every reader of that
  log.[^4] [^9] The work must list what it moves, and the commit body holds that
  list rather than any record.[^10]

- **Which bucket edges exist at all.** A magnitude is a bucket position, and the
  edges of its buckets are rows of the balance register.[^11] [^5] The register
  holds no such row. The work must say whether any verb of the enumeration takes
  a quantity, and it must add a row for each edge it needs rather than write the
  edge into the code.

- **How a test proves the answer and the verbs agree.** The record asks for a
  test that compares the two over a seed set, and says a row the answer allows
  and the verb then refuses is a defect.[^2] A fixture that models the typical
  case supplies no refused row, so the test would measure the fixture. The work
  must build a world that produces refusals, and it must put the defect back and
  watch the test stay green before it claims the case is covered.[^12]

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^5]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares. `docs/adrs/draft/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^6]: Backlog item 0495, build the observation plane and let every reader answer for one faction. `docs/backlog/complete/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^10]: Commit Message Rules. `.agents/rules/commits.md`
[^11]: Balance register. `docs/reference/balance.md`
[^12]: Testing Rules, sections 2 and 2a. `.agents/rules/testing.md`
