---
id: 0125
title: Show a watcher a deposit recovering
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0018]
blocked-by: []
---

## Why

The product record requires that a watcher can see a deposit recover, and can
tell a recovering deposit from a full one.[^1] The engine will hold the state
and the window will not show it. That is the gap a review already recorded
against other work, so the project knows the shape.

**What refining this must answer.** What the watcher sees: a shade of the
resource marker, a count, or both. Whether the window reads the recovered
amount through the same call a gatherer uses, so that the two cannot disagree.
Whether the head-up display reports what the drawing pass read, which the
viewer records already require.

## What the tree already holds

**The premise of this item is false, read on 5 September 2026.** The window
already shows the live stock of a deposit against what the ground gave. The
inspector prints it as "N of M", and prints "none" when the ground gave
nothing.[^A1] [^A2] An overlay ramp paints the same quantity.[^A3] Both read the
depletion ledger a gatherer writes, through one reader that subtracts what was
taken from what the ground gave.[^A4]

**What is absent is a marker that names recovery as such.** A watcher cannot
tell a deposit that is recovering from a deposit that is simply part drained,
because both read as the same partial stock.

**So this item is now that marker, and nothing else.** Do not build a second
stock reader. One exists, two callers read it, and a second would be one fact in
two places.

## Impact review

Not done. This item stays in `proposed/` until it is written.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^A1]: The inspector deposit rows. `crates/cachette-view/src/panel/inspector.rs`
[^A2]: The glass food reader. `crates/cachette-view/src/glass.rs`
[^A3]: The stock overlay and its ramp. `crates/cachette-view/src/overlay.rs`
[^A4]: The tile stock reader, which subtracts what was taken. `crates/cachette-core/src/stock.rs`
