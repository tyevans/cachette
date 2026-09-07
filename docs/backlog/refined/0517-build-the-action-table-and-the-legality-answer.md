---
id: 0517
title: Build the action table and the legality answer, and let a caller act by one integer
status: refined
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
the rest.[^5] **A reviewer accepted that record on 6 September 2026, so it
binds.** The encoding this item builds is the one that record states.

**It depends on item 0495.** A legality answer must not name a target the
faction cannot see, so the readers that answer for one faction come first.[^6]

**This is the largest item of the learner seat.** The record forbids restating a
refusal rule: the answer reads the same candidate lists and the same refusal
rules the verbs read, and it duplicates no rule.[^2] Those rules live inside the
verbs, spread through the world module, so honouring the record means giving
each verb a path that reports a refusal without acting. A survey names that as
the hidden cost of the record, and warns that anybody who sizes this work from
the record alone will underestimate it.[^4]

## The architectural impact review

**The records that govern this work.** ADR-0154 D1, D3, D4, D5 and D6 govern the
schema, the fog rule, the encoding, the legality answer and the log. ADR-0176 D1
to D4 replace the factorisation of D4 with the mixed radix. ADR-0144 D2 governs
the verbs a learner may reach. ADR-0006 D1 governs the log row. ADR-0004 D1 and
D4 govern the order. PRD-0001 and PRD-0056 state the need.

**The records this work changes.** None. It implements the two above and
contradicts neither.

**The records this work creates.** None. Every decision it makes is one the two
records already state, and the registry allocates no new number for it.[^7]

**The blockers that hold it.** BLK-007 holds every cost figure of this project.
The work therefore states the cost of the legality answer as a measurement of
the machine that took it, and it takes no figure into a record.[^8]

## The questions this item held, and their answers

**How each verb reports a refusal without acting.** The shape is a check
function beside each verb that the verb itself calls. The build verb, the
relation verb, the raise verb, the settle verb and the project order each gained
one, and each of them now reads its own rule rather than repeats it. The
legality answer calls the same checks. A dry-run flag was rejected because it
would thread a parameter through eleven verbs and their callers, and a single
resolving pass was rejected because the answer would then hold a second copy of
what each verb decides.

**What the legality answer costs.** The table is small, because no verb of the
enumeration carries a tile or a unit. A benchmark reports the answer beside one
step of the same world, and the register holds no figure from it, because a
blocker holds every cost figure of this project.[^8]

**Whether a learner may ask for it every tick.** Yes. The answer is a small
fraction of one step on the machine that measured it.

**What the log layout change costs.** The command row drops the kind column and
the argument column, and it gains one action column that carries the whole
action. The row keeps its size and its declared padding, because the narrow
column sits after the wide ones.[^9] Every reader of the kind column moves to
the schema, and the commit body lists them.[^10]

**Which bucket edges exist at all.** None. No verb of the choice enumeration
takes a quantity, so no verb declares a bucket position and the balance register
gains no row.[^11]

**How a test proves the answer and the verbs agree.** One test runs every row of
the table for every faction over five seeds and three tick counts, on a copy of
the world for each row, and compares the answer against what the verb did. The
test asserts that the fixture supplied both a legal row and a refused row, so it
cannot pass by measuring a world that refuses nothing.[^12]

## Done when

- A caller reads one schema that names each verb, its argument positions, the
  bound of each position and the stride of each position.
- No file outside the core crate states a verb number, a position or a bound.
- Every row of the table encodes to one integer and decodes back to the same
  verb and the same arguments, by arithmetic over the schema alone.
- No bound follows the population.
- A learner acts by one integer through the public interface, and the world
  changes.
- The engine answers one byte for each row, and the no-op byte is always one.
- The answer and the verb agree over a seed set, and the fixture supplies both a
  legal row and a refused row.
- The answer names no target the faction has not observed, and a test fails when
  it does.
- One log row carries the whole action, for a controller choice and for a
  learner action alike.
- The cost of one legality answer is measured beside the cost of one step.

## What this item does not do

- **The reward, the batch step and the wrapper.** A learner acts and reads, and
  nothing scores it yet.
- **A mask for one unit.** The table names no unit, and a per-unit action space
  is a separate claim.
- **The cost of the answer at the target scale.** The table length follows the
  faction count alone, and it never follows the population. The cost of one row
  of it does follow the population, because the build verb reads its refusal
  once for each unit of the faction. A later item may bound that read.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: Research report 31, the state of the learner surface. `docs/research/reports/31-the-state-of-the-learner-surface.md`
[^5]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^6]: Backlog item 0495, build the observation plane and let every reader answer for one faction. `docs/backlog/complete/0495-build-the-observation-plane-and-let-every-reader-answer-for-one-faction.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^10]: Commit Message Rules. `.agents/rules/commits.md`
[^11]: Balance register. `docs/reference/balance.md`
[^12]: Testing Rules, sections 2 and 2a. `.agents/rules/testing.md`
