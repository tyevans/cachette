---
id: 0071
title: Derive tile passability from tile capacity
status: complete
created: 2026-08-31
implements: [ADR-0056 D4, ADR-0068 D4]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The terrain module holds one fact about the ground in two places.

The capacity table gives each terrain kind the number of units that may stand
on it. Water gets zero. Every other kind gets the ordinary capacity. The
comment above that table states that passability is the capacity being zero,
and that nothing else states it.

The passability reader beside it states it again. The reader matches on the
water kind by name. It never reads the capacity.

The two agree today, and they agree by accident. Water is the one kind with a
capacity of zero. A kind added with a capacity of zero and a name that is not
water would report itself passable and would admit nobody. Nothing fails, and
no test compares the two readers.

This is shape 1 of the recurring defect rule: one value declared twice, with
nothing that fails when the copies disagree.[^1] The findings register holds
the instance.[^2] Item 0092 is refined and waits on this item, because it adds
a third reader of the ground unless the second one goes first.[^3]

## Which site survives, and why

**The capacity table survives. The kind-name match goes.**

The passability reader stays as a reader, and its body becomes the capacity
being greater than zero. A reader is not a declaration site once it derives
its answer from one.

Three reasons pick the capacity table.

- An accepted record already puts the capacity in the terrain table and
  forbids the engine to hold a capacity of its own.[^4] Nothing gives
  passability the same standing.
- The capacity carries more information. A capacity answers how many stand
  here, and passability answers only whether anybody does. The wider fact can
  derive the narrower one. The narrow one cannot derive the wide one.
- The capacity match is exhaustive over the kinds, so the compiler refuses a
  new kind that states no capacity. **The compiler becomes the check that the
  comment pretended to be.** A name match has no such property: it accepts a
  new kind in silence, by the default arm.

## What happens to every caller

**No caller changes.** The reader keeps its name, its signature and its
constant qualifier, so every call site keeps working and nothing moves.

The engine reads passability in the pyramid summary, in the holding spread, in
the founding survey, in the founding choice, in the group spawn, in the tile
query, in the spawn refusal, and in the movement admission. The tests read it
in the terrain suite and in the two founding suites. The engine reads the
capacity in the founding survey, in the group spawn, and in the movement
admission table.

This item states no count of those sites, because a count decays. The search
that produced the list is:

```
grep -rn "is_passable\|capacity(" crates/ python/ scripts/
```

The work runs that search again and puts its output in the commit body.[^5]

**One test needs attention.** The terrain suite holds a test that lists the
four passable kinds by name and asserts each. That test states the same fact a
third time, in the test tree. It must assert the equivalence over every kind
instead, so that a new kind joins the assertion without an edit.

## Impact review

**Governed by.**

- ADR-0056 D4. The capacity is a data-driven property of the terrain, and the
  engine holds no capacity constant of its own. This item makes passability a
  consequence of that table rather than a second rule beside it.[^4]
- ADR-0068 D4. The terrain says what a tile is and never what a tile costs, and
  it names passability as a property of the ground. Passability therefore stays
  in the terrain module. This item moves it to one reader inside that module,
  and it moves it nowhere else.[^6]

**Changes.** No record changes. The comment in the terrain module changes, and
a comment is not a record.

**Creates.** No record. The judgement is deliberate and it follows the scope
rule.[^7] Condition one fails: a contributor cannot reasonably choose the other
way, because ADR-0056 D4 already puts the capacity in the table and a second
declaration contradicts it. Condition three fails: the reasoning is one
comparison in the body of the reader, and a reviewer sees it there.

**This is not a determinism decision, so the counter-test does not apply.**
The reader is a pure function of the kind at every call site, and the change
alters no ordering, no key and no accumulator.

**Blockers.** BLK-007 governs every cost figure, so this item states none.[^8]
BLK-009 is resolved and fixes the ordinary capacity and the crossing
capacity.[^9] This item invents neither value and moves neither. DEC-017 holds
the open choice of where a crossing multiplier lives, and it does not bear on
this item, because a multiplier is not a capacity.[^10]

**Precedent.**

- FND-060 records the two sites and asks for the second to be derived away
  rather than reconciled.[^2]
- FND-070 records that a restored defect must be affordable, or the proof
  produces no answer.[^11] The restored defect here is the old body of the
  reader, which is one line and costs one test run.

**Serves.** No product record. The list is empty on purpose. The item repairs a
defect that no recorded need asks for. It unblocks item 0092, which serves two
product records.[^3]

**Conflict surface.** `crates/cachette-core/src/terrain.rs` at the passability
reader and at the comment above the capacity table.
`crates/cachette-core/tests/terrain.rs` at the passability test. Nothing else
changes. **It cannot run beside item 0092**, which calls the reader this item
changes.

## Does the removal change behaviour today

**No.** Water has a capacity of zero. Every other kind has the ordinary
capacity, which is greater than zero. The two readers therefore return the same
answer for every kind that exists.

The verification is a reading of the capacity match, which is exhaustive over
the kinds and states a literal zero for water alone. The work must confirm it
by running the test suite and the golden state hash unchanged. **A golden hash
that moves means the change was not behaviour-neutral, and the work stops.**

**The equivalence test is a tautology after the change, and the item says so
rather than claiming evidence it does not have.** Comparing the two readers
proves nothing once one calls the other. The evidence that the defect is gone
is that one declaration site remains, which a whole-tree search shows, and that
the compiler refuses a kind with no capacity.

## Done when

- The passability reader returns the capacity being greater than zero, and it
  matches no kind by name.
- A whole-tree search finds one declaration of which ground admits a unit, and
  the search command sits in the commit body.[^5]
- The comment above the capacity table describes what the code does, and it
  claims no check that does not exist.
- The terrain test asserts passability over every kind against that kind's
  capacity, and it names no kind in a hand-written list.
- A test-only kind with a capacity of zero and a name that is not water reports
  itself impassable. If the kind set cannot carry such a kind, the item says so
  in the outcome and states the compiler exhaustiveness as the evidence
  instead.
- The old body of the reader is put back, and the new test is watched failing,
  before the item is claimed done. The restored defect is the smallest change
  that violates the claim.[^11]
- The golden state hash files do not move, and the commit body says that they
  did not.
- The two determinism tests pass, and the thread-count test runs at more than
  one thread count.[^12]
- FND-060 records the outcome in its own entry.
- `just check` exits 0.

## Outcome

**Done.** The passability reader returns the capacity being greater than zero.
It matches no kind by name. A small function takes the capacity and answers
whether the ground admits a unit, so the rule can be stated for a capacity that
no kind carries today. The capacity table is the one declaration, and its match
is exhaustive over the kinds.

**The kind list moved into the source.** The terrain kind now carries an array
of every kind, and the length of that array is the kind count. The test reads
the array, so it names no kind of its own. A new kind that the array omits is a
compile error. The old test listed the four passable kinds by hand.

**No caller changed.** The reader keeps its name, its signature and its
constant qualifier. No golden state hash file moved.

**The proof that the test can fail.** The work restored the name match and set
the water capacity to the ordinary value. The equivalence test then reported
that water answered the two questions differently. The restored defect is two
lines and one test run.

**The kind set cannot carry a test-only kind with a capacity of zero.** The
kind is a plain enumeration, and a state hash and a viewer palette both read
it. An extra kind behind a feature would move the hash and would need a colour.
The evidence is therefore the compiler exhaustiveness over the capacity match,
and a test of the rule against a capacity of zero that no kind carries.

**FND-060 is closed in its own entry.**

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Findings register, FND-060. `docs/FINDINGS.md`
[^3]: Backlog item 0092. `docs/backlog/refined/0092-refuse-a-settlement-on-the-ground-that-cannot-carry-one.md`
[^4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^5]: Commit Message Rules, after a sweep. `.claude/rules/commits.md`
[^6]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^7]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: Blockers register, BLK-009. `docs/BLOCKERS.md`
[^10]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^11]: Findings register, FND-070. `docs/FINDINGS.md`
[^12]: Testing Rules, section 1. `.claude/rules/testing.md`
