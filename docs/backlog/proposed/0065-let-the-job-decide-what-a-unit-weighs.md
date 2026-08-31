---
id: 0065
title: Let the job decide what a unit weighs
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0009, PRD-0017]
blocked-by: [0063, 0064]
---

## Why

Item 0064 gives every unit the same weights, so every unit wants the same
things and a faction of farmers behaves like a faction of soldiers. Item 0063
gives a unit a job that nothing reads.

This item joins them. A job is an index into a table of weights, so the job
changes what the unit prefers without adding a branch to the choice and
without multiplying the verbs. That is the project's own principle: a type
parameterises a verb, it does not multiply it.

It is the smallest change that makes the whole plan visible at once. A watcher
who moves a unit from one job to another sees it behave differently, and every
subsystem below it is what produced the difference.

## What the work does

1. A job is an index into a shared table. The table holds the weights the
   choice multiplies by.
2. Changing a unit's job changes what it prefers, with no branch in the
   choice.
3. The behaviour a job drives is visible: a watcher can name the job and
   predict the preference.

## Impact review

**Governed by.** This work implements no decision of an accepted record
directly. It reads two records that do not exist yet: the choice record that
item 0064 writes into row 0064, and the assignment record that item 0063
allocates.[^1]

**Blockers.** BLK-007 governs every cost figure, so this item states none.

**Serves.** PRD-0009 and PRD-0017.

**Conflict surface.** `crates/cachette-core/src/choose.rs` and
`crates/cachette-core/src/assign.rs`, both of which items 0064 and 0063
create. It touches no other file, so **once those two land, this item runs
beside anything.**

## What is missing before this is refined

**Both governing records.** The impact review cannot name the decisions of a
record that nobody has written. Item 0064 writes ADR-0064 and item 0063
allocates and writes the assignment record; until both exist, this item cannot
state which decisions govern it, and a review that names them anyway would be
guessing. **That is the whole of what is missing.** The work itself is small
and its shape is clear.

**One thing to check when it is refined, and not before.** A weight table
indexed by job is a second declaration site if the assignment also holds a
notion of what a job is for. One of the two must be the only one, and a check
must fail when they disagree.[^2]

## Done when

- A job is an index into a shared table, and adding a job adds a row rather
  than a branch.
- A unit whose job changes prefers different things, and a test asserts the
  change through the public interface.
- Two units of different jobs in one identical situation choose differently,
  and a test asserts it.
- No branch on the job appears inside the scoring loop.
- The table is declared in one place, and a check fails when a second
  declaration disagrees with it.[^2]
- A property test asserts that the choices are identical at 1, 2 and 12
  threads.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR Registry, row 0064. `docs/adrs/REGISTRY.md`
[^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
