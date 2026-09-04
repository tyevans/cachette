---
id: 0460
title: Bind the character and lineage subsystem to the control plane
status: complete
created: 2026-09-03
implements: [ADR-0040 D1, ADR-0043 D1, ADR-0046 D1, ADR-0085 D1, ADR-0085 D2, ADR-0085 D3, ADR-0014 D2, ADR-0014 D3, ADR-0002 D1, ADR-0004 D1, ADR-0054 D1, ADR-0054 D4, ADR-0066 D1, ADR-0107 D2, ADR-0078 D1, ADR-0104 D4]
changes: []
creates: []
serves: [PRD-0045, PRD-0046, PRD-0015, PRD-0016]
blocked-by: []
---

## Why

The core holds a character tier. It makes a character, bears a child of two
characters, ends a character, links a unit to the character it was raised
into, reads the parents of a character, walks the ancestors and the
descendants, computes an exact relation between two characters, writes how
much a character is thought of, sets the schedule that raises people, reads
what a unit has done, and reads and writes the level of deeds at which the
engine raises somebody.

No line of the bindings crate names any of those thirteen methods, and no
Python line names them either. The whole subsystem is a capability that
nothing invokes, which is a shape this project lists in its own rule.[^1] It
is the largest instance of that shape in this engine so far.

The consumer is a game in which a player, often a language model, directs a
group of simulated people. A language model reasons about a named person with
parents, a reputation and a history. It cannot reason about a count. This
subsystem turns a population into people, and today none of it crosses the
boundary.

Two accepted product records state the need behind the engine work, and
neither was ever reachable from a caller.[^2] [^3] Two shaped records state
the need behind this item: one for what a caller reads, one for what a caller
writes.[^4] [^5]

## Impact review

**Governed by.**

- ADR-0040 D1. Python is a control plane. The boundary carries an instruction
  and an answer, never the population. Every read here answers about a set,
  and every write takes a set.[^6]
- ADR-0043 D1. The tier of a shape decides the shape of its interface. A
  character is not the mass tier, so a singular read is allowed where the
  answer is about one person. A read that would ask a caller to walk a graph
  one edge at a time is not.[^7]
- ADR-0046 D1. Every refusal crosses as a typed error under one root
  class.[^8]
- ADR-0085 D1, D2 and D3. An entity crosses as one opaque number that Python
  cannot build, and the engine resolves it against the generation. A dead
  identity is refused rather than answered for the next occupant of the
  slot.[^9]
- ADR-0014 D2 and D3. An identity is a slot index and a generation, and the
  identity of somebody who is gone never resolves again.[^10]
- ADR-0002 D1. No floating point number crosses in simulated state. How much a
  person is thought of and the relation between two people are both Q16.16
  fixed-point values, and both cross as their raw integer.[^11]
- ADR-0004 D1. Iteration order is explicit. Every answer here states its
  order.[^12]
- ADR-0054 D1 and D4. A character is the character tier, declared at creation,
  and an entity never changes tier. A unit that is raised is not turned into a
  character; a character is created beside it and linked to it.[^13]
- ADR-0066 D1. The living character is one of the four fixed shapes.[^14]
- ADR-0078 D1. Descent is a bounded record and it is append-only. A row of it
  outlives the character it names.[^15]
- ADR-0104 D4. The engine cuts the promotion at a rank, inside a budget.[^16]
- ADR-0107 D2. The prose of each binding lives in the Rust doc comment, and
  the published reference is generated from an import of the compiled
  module.[^17]

**Changes.** No record changes. The core keeps its behaviour.

**Creates.** No decision record. Every claim this work needs is already in the
records above, and a record that only described the binding would be a
description and not a constraint.[^18] The shape choices go to the decisions
register instead.

**Blockers.** BLK-007 governs every cost figure, so this work states none.[^19]
BLK-004 answers the ceiling on the living character population, so this work
invents no number.[^20] BLK-011 answers what a person raised from the ranks
inherits, and the answer is nothing, so a lineage answer with no ancestor is a
real answer.[^21]

**Precedent.** Item 0341 bound the build verbs and found the same shape: the
core methods were public and their own Rust tests passed the whole time.[^22]
Its review states that every test must start at the Python boundary for that
reason, and this item follows it.

**The risk this item carries.** A relation query and an ancestor walk invite a
caller to follow one edge at a time across the boundary, which is the loop
ADR-0040 D1 forbids. The read must answer with a whole structure. A second
risk is that a relation query returns a plausible wrong answer: swapping the
ancestor list for the descendant list gives a list of the right type and the
wrong content, and no type check sees it.

## Done when

- A caller reads every living character, or every living character of one
  faction, as parallel columns in one call.
- A caller reads the parents, the ancestors and the descendants of one
  character in one call, and each person named carries the identity the engine
  minted and a flag that says whether they are alive.
- A caller reads the relation between one character and a set of other
  characters in one call, as exact Q16.16 values.
- A caller reads what a set of units has done, and which character each was
  raised into, in one call for each question.
- A caller makes, bears and ends people, and writes how much a set of people is
  thought of, each in one call.
- A caller sets and reads the level of deeds, and sets the schedule that raises
  people.
- Every write resolves every identity and checks every argument before it
  changes anything, and one refusal leaves the living population unchanged and
  raises a typed error.
- A test drives each path from the Python boundary, and a defect put back into
  each path makes that test fail.
- A test asserts that the descendant read answers with the descendants and not
  with the ancestors.
- The doc comment of every new member states each argument, its type, its unit,
  and the error class the call raises. A fixed-point value says so.
- The doc comment states what a caller may not control.
- The whole check command runs green, and the two determinism tests pass at 1,
  2 and 12 threads.

## Outcome

**Twelve members are new on the `World` class, and they cover all thirteen core
methods.** The reads are the living population as columns, the whole lineage of
one person in one call, the relation of one person to a set of others, what a
set of units has done, which character each of a set of units was raised into,
and the level of deeds. The writes make people, bear children, remove people,
write how much a set of people is thought of, set the level of deeds, and set
the schedule that raises people.

**Four core methods fold into two.** The parents read, the ancestor walk and
the descendant walk became one lineage read, because three calls invite a
caller to walk the record one step at a time. The relation read became
set-valued for the same reason.

**Three small core changes were needed.** The core held no way to resolve a
character identity, and a boundary that refuses a stale identity cannot be
written without one. The arena held no accessor for the generation of a slot,
which that resolution needs, and the same accessor already exists on the
soldier arena. One method that encodes the sex column was made public, so that
the boundary reads the encoding rather than writing a second copy of it. No
core behaviour changed and the golden state hash did not move.

**Four findings and seven decisions went to the registers.** FND-470 records
the search that measured the gap. FND-471 records that the engine reports a
line has ended and no boundary read can ask. FND-472 records that a character
identity and a unit identity are the same number. FND-473 records that nothing
writes the renown and nothing reads it. DEC-260 to DEC-264 hold the shape
choices, and DEC-265 and DEC-266 are open with a recommendation each.

**One blocker opened.** BLK-150 asks what raises and lowers renown. Two blocker
numbers were allocated to this work and stay unused.

**One item came out of it.** Item 0461 holds the repair for the overlapping
identity ranges and the read that asks whether a line has ended.

**What changed from the plan.** The plan named a `line_ended` answer on the
lineage read. It was written and then removed, because every read at the
boundary takes a living identity and the answer is false for every living
identity. The plan did not foresee that a character identity and a unit
identity are the same number, and the doc comments and one test now name that
hazard. The plan also carried a check that the world holds the faction, and a
defect put back into it caught nothing, because the arena already refuses. The
check was removed as a second declaration site of one rule.

**The review holds the detail.**[^23]

## References

[^1]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^2]: PRD-0015, a unit has parents and children. `docs/product/accepted/prd-0015-a-unit-has-parents-and-children.md`
[^3]: PRD-0016, somebody is in charge. `docs/product/accepted/prd-0016-somebody-is-in-charge.md`
[^4]: PRD-0045, a god knows its congregation by name. `docs/product/shaped/prd-0045-a-god-knows-its-congregation-by-name.md`
[^5]: PRD-0046, a god raises somebody up. `docs/product/shaped/prd-0046-a-god-raises-somebody-up.md`
[^6]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^7]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^8]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
[^9]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^10]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^11]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^12]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^13]: ADR-0054, an entity belongs to one of three tiers, declared at creation. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^14]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^15]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
[^16]: ADR-0104, a soldier is promoted from a level that never falls. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
[^17]: ADR-0107, the Python reference is generated from the compiled module. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^18]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^19]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^20]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^21]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^22]: Review of item 0341, bind the build verbs. `docs/reviews/0341-bind-the-build-verbs.md`
[^23]: Review of item 0460, bind the character and lineage subsystem. `docs/reviews/0460-characters-and-lineage.md`
