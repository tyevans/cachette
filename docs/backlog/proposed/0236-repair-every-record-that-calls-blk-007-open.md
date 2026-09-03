---
id: 0236
title: Repair every record that calls BLK-007 open when the benchmark lands
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A benchmark harness exists on a branch, and the register says no measurement
exists.** BLK-007 states that every number in the project is derived and that
the blocker needs a benchmark harness and a machine. It is open on the trunk and
on every branch this work can see, and measurements on the target platform have
been reported from a branch that has not merged.

**The moment it closes, a large number of documents state something false, and
nothing fails.** The rules already name this shape and record that it has
happened twice: a record written parametrically under a blocker is correct when
it is written and false the moment the blocker closes, and nothing catches it
because a record is prose.[^1] [^2]

**The scale is the problem.** Sentences of the form "no measurement exists on
the target platform, and every cost figure in this project is derived" appear in
accepted records, in draft records, in a product record, in the decisions
register and in source comments. Each was correct when written. Each is a
separate site.

This item exists before the merge rather than after it, because the sweep is
cheapest when somebody has already found every site and the person closing the
blocker only has to run the search.

## What the work does

Find every site, decide what each should say, and change them in the commit that
closes the blocker. A record that cited the blocker for the *state* of
measurement needs a different repair from one that expressed a *value*
parametrically under it.

## What is missing before this is refined

- The impact review, and the search that finds every site. It is not one string:
  the blocker is cited by number, by the phrase about derived figures, and by
  records that state a value parametrically without naming it.
- Whether closing BLK-007 is even correct. A benchmark of some passes on one
  machine does not make every figure in the project measured, and the blocker as
  written is about the whole class. It may need narrowing rather than closing.
- Which repairs are amendments and which are not. An accepted record whose
  consequence says no measurement exists is a stale consequence, which is the
  case DEC-096 holds open.[^3]
- Whether the reference register should hold the state of measurement in one
  place, so that a record cites the register rather than restating the state.
  That would make the next closure one edit instead of a sweep, and it is the
  same shape as the numbering problem.[^4]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
[^2]: Findings register, FND-042. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^4]: Backlog item 0235. `docs/backlog/proposed/0235-give-a-register-number-one-authority-a-writer-can-consult.md`
