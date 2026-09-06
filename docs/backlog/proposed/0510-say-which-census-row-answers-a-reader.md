---
id: 0510
title: Say which census row answers a reader
status: proposed
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

## What is missing before this is refined

A refiner answers these before this item leaves `proposed/`.

- **Which of them the balance harness or the demonstration deck needs.** Read
  both. A row either serves one of them or serves nobody.
- **What a row that serves nobody costs.** Say whether the answer is to delete
  it, or to give it a test that fails when it stops counting.
- **Whether the two declaration sites should stay two.** The Rust table
  declares the list and a Python test declares it again. The test is the check
  that fails when the two disagree, and a refiner says whether that is the
  right shape or whether the test should read the table.[^2]
- **What test each surviving row gets.** A row that counts an act needs a test
  that goes red when it stops counting.[^4]

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-548. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, shapes 1 and 3. `.agents/rules/recurring-defects.md`
[^3]: Findings register, FND-498. `docs/FINDINGS.md`
[^4]: Testing Rules, section 2. `.agents/rules/testing.md`
