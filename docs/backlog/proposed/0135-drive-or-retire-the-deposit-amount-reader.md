---
id: 0135
title: Drive or retire the deposit amount reader
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The resource module holds a function that computes what a deposit holds at a
tick, from the stored take, the tick and the period. Nothing calls it. A search
of the engine crate, the bindings and the tests finds the definition and no
caller.

The world reads a stock by a different route. It reads the raw stored take and
subtracts. That read is correct only because recovery runs earlier in the same
step, so every clock is already at the current tick when anything reads.

A review of the record that governs recovery found this. The record had stated
the pure function as the reason two readers at one tick agree. The record now
states the ordering instead, because the ordering is what the engine actually
relies on.[^1]

This is the third shape of the recurring defect rule: a capability that nothing
invokes, whose own test would pass because the test calls it directly.[^2]

## The second declaration site

The unused function reaches the period of a kind by one route, and the recovery
pass reaches it by another. The two give one answer today. Nothing fails if they
stop agreeing, and the reader that would notice is the one nobody calls.

## What the work does

One of two things, and the item must choose before it is refined.

**Drive it.** Route the world's stock read through the function, so a read is
correct because the amount is computed rather than because a pass ran first. The
property then holds under a change to the order of the step.

**Retire it.** Delete the function and state in the record that the ordering is
the whole of the guarantee. The record already says this.

Driving it is the better end state and costs more. Retiring it is honest and
leaves the ordering load-bearing.

## The questions this item must answer before it is refined

**Whether a read outside the step needs to be correct.** The control plane reads
between frames today, when the order has already run. A reader that could
observe a stale stock mid-step would decide this item.

**What the second route to the period costs if the function is driven.** Two
routes to one value is shape 1, and driving the function makes both live.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0080, a depleted deposit recovers by ageing the stored take, decision D4. `docs/adrs/draft/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^2]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
