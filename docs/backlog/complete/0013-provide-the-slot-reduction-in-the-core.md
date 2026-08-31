---
id: 0013
title: Provide the slot reduction for a reduction that is not order-free
status: complete
created: 2026-08-30
implements: [ADR-0004 D3]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

ADR-0004 D3 requires that a reduction which is not order-free writes into a
slot indexed by a stable key, and that the combine step reads the slots in
index order.[^1] Minimum, maximum and first-wins all qualify.

The crate honours the rule once, inside the step that joins the event log. The
mechanism is written into that one function and nothing else can reach it. The
next system that needs a minimum will write its own, and the rule will hold
only as long as each author remembers it.

## Impact review

**Governed by.** ADR-0004 D1 requires an explicit stable order for every
iteration that feeds a result. ADR-0004 D2 states which reductions need no
ordering work. ADR-0004 D3 states the slot rule this item implements. ADR-0002
D1 keeps every accumulator exact, so the combine is exact.[^1] [^2]

**Changes.** None. The step keeps its behaviour; it becomes a caller of the
shared mechanism rather than the only holder of it.

**Creates.** None. ADR-0004 already holds the claim, and a module arrangement
is not a decision.[^3]

**Blockers.** None. The mechanism does not depend on the world shape or on the
faction count.

**Precedent.** The recurring-defect rule warns against a capability that
nothing invokes: the test must drive the engine, not construct the reduction
and exercise it directly.[^4]

**Serves.** No product record.

## Done when

- The core exposes a slot reduction that takes a stable key for each unit of
  work and a combine step that reads the slots in index order.
- Minimum, maximum and first-wins are expressible through it.
- The step in the world uses it, so the mechanism has a real caller.
- A property test asserts that the result is identical at 1, 2 and 12 threads,
  including when values tie.
- The thread-count test covers a scenario whose reduction ties, because a tie
  is where the order shows.
- The perturbed build fails the new test, so the test has a proven failure
  mode.[^5]
- No floating point appears anywhere in the mechanism, and the float gate
  passes.
- `just check` runs green.

## Outcome

`crates/cachette-core/src/slots.rs` holds the mechanism. `Slots::filled`
builds one entry for each unit of parallel work, `combine` folds them in
index order, and `first_wins`, `minimum` and `maximum` are the three
reductions that depend on order when values tie. The world step is a caller,
so the mechanism is not inert.

Two things changed from the plan. The perturbation switch moved out of the
world and into `Slots::combine`, so the probe now reaches every reduction
rather than only the log join; that is a change to existing determinism
machinery, not a pure addition. And `Candidate` shipped with derived
comparison traits whose tie rule disagreed with `minimum` and `maximum`; the
code review found it and the derives are gone.

Five mutations were applied and all were killed. Under the probe feature six
of the eight tests fail, and the probe recipe now asserts that, so the
failure mode gates a merge.

## References

[^1]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^4]: Recurring Defect Shapes. `.claude/rules/recurring-defects.md`
[^5]: Testing Rules. `.claude/rules/testing.md`
