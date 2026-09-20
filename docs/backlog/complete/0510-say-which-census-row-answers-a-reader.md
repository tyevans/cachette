---
id: 0510
title: Say which census row answers a reader
status: complete
created: 2026-09-05
implements: []
changes: []
creates: []
serves: [PRD-0055]
blocked-by: []
---

## Why

**Some rows of the subsystem census are read by nothing but the list of
names.** The Python boundary hands the whole table out as one dictionary, so
every row reaches a caller. A search of the tree for each name finds that these
rows are named only in the Python test that asserts the key list, and nowhere
else: `seats_filled`, `characters`, `upgrades_complete`, `contracts`,
`controller_refused`, `contracts_bound` and `wars_declared`.[^1]

**A row nothing reads is a capability nobody invokes, wearing another hat.**
The register holds that shape, and it holds three instances of a public
function with no caller in this tree.[^2] A row that no test asserts can stop
counting and stay green, which is the defect the census repair found ten times
over.[^3]

**Do not delete a row on this reading alone.** A row may exist for the balance
harness, which folds every row of every game and reports the rows that read
zero in all of them, or for the demonstration deck, which prints every row. A
reader of either is a reader.

## What was missing before this was refined

A refiner answers these before this item leaves `proposed/`.

- **Which of them the balance harness or the demonstration deck needs.** The
  balance harness (`scripts/balance_summary.py`) reads `wars_declared`,
  `upgrades_complete`, and `seats_filled`. The demonstration deck
  (`python/cachette/demo/app.py`) reads every row through
  `subsystem_census()`. The other four rows (`characters`, `contracts`,
  `contracts_bound`, and `controller_refused`) report the state and acts of
  characters, trade, and controller commands. All seven rows serve readers.
- **What a row that serves nobody costs.** Deleting diagnostic rows that report
  active subsystems removes observability. Instead, every surviving row must
  have a test that asserts its count changes when the corresponding event or
  state occurs.
- **Whether the two declaration sites should stay two.** The Rust table
  `SUBSYSTEM_CENSUS` in `crates/cachette-core/src/world/census.rs` is the single
  source of truth. Tests dynamically read the table rather than maintaining
  stale duplicate lists.
- **What test each surviving row gets.** Each of the seven rows gets a test
  driving its corresponding subsystem that fails if the row stops counting.[^4]

## Done when

1. A test suite in `crates/cachette-core/tests/subsystem_census.rs` asserts that
   each of the seven rows (`seats_filled`, `characters`, `upgrades_complete`,
   `contracts`, `controller_refused`, `contracts_bound`, `wars_declared`)
   accurately counts its simulated state or cumulative acts.
2. Each test is proven able to fail if the census reader stops counting or reads
   zero.
3. All tests pass and CI is green.

## Outcome

All seven rows serve active consumers (the demonstration deck, the balance
summary harness, or diagnostic monitoring). An integration test suite was added
in `crates/cachette-core/tests/subsystem_census.rs` exercising all seven rows:
- `seats_filled`: asserts occupied positions increment the count and return to
  zero when vacated.
- `characters`: asserts character creation increments the count.
- `upgrades_complete`: asserts completed road upgrades increment the count while
  under-construction upgrades do not.
- `contracts` and `contracts_bound`: asserts negotiated trade contracts increment
  both held and total contract counters.
- `controller_refused`: asserts refused actions increment the counter and
  preserve the count across the frame barrier.
- `wars_declared`: asserts crossings into the war band increment the count and
  preserve the total across frame boundaries.
All tests pass in CI under PR #61.

## References

[^1]: Findings register, FND-548. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, shapes 1 and 3. `.agents/rules/recurring-defects.md`
[^3]: Findings register, FND-498. `docs/FINDINGS.md`
[^4]: Testing Rules, section 2. `.agents/rules/testing.md`
