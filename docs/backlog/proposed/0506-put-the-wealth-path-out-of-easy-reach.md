---
id: 0506
title: Put the wealth path out of easy reach
status: proposed
created: 2026-09-05
implements: []
changes: [ADR-0148]
creates: []
serves: [PRD-0053]
blocked-by: []
---

## Why

**The wealth clause ends every game, and the balance register cannot stop it.**
A sweep of 32 seeds of the demonstration world, each played to 20000 ticks,
ended 32 games on the wealth-or-wonder path. The stock clause ended 29 of them
and the wonder clause ended the other 3. Domination and territory ended none.
The project owner asked for wealth to sit far out of reach, so that domination
and territory decide a game.[^1]

**The value has a ceiling that is one seventh above where it stands.** A
faction founds one settlement, the commodity count is one, and a store is a
`Fix32`, so the stock total of a faction clamps just below 32768 whole units.
The target stands at 28672, which is seven eighths of the clamp. The store
rises steadily and reaches the clamp, so the reader fires at every target the
engine permits, given enough ticks. Raising the row buys ticks and nothing
else.

**A row that cannot be set is worse than a row that is unset.** The stock
target has a register row, and the row invites a reader to believe the value is
free. It is not. A worker who raises it will measure a later end tick and
report a success, and the path will still end every game.

## What is missing before this is refined

- **Which of the three answers this item takes.** Scale the target by the
  settlements a faction owns, widen the type the store holds, or drop the stock
  clause from the reader. Each is a different record change.
- **What the store clamp is for.** The clamp is a property of the quantity
  type, so a wider type touches every store in the engine and not only this
  reader. State the cost.
- **Whether the wonder clause stays as it is.** The wonder bar of 2400 fires in
  3 of 32 seeds at 20000 ticks. Say whether that share is what the owner wants,
  and state the share rather than the value.
- **What the reader is called after the change.** The reader is one function
  over two clauses. If the stock clause goes, the name and the path name change
  with it.
- **What the balance harness then checks.** The win-path share row is unset, so
  the harness reports and does not fail. Say whether this item sets it.

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-543. `docs/FINDINGS.md`
