---
id: 0057
title: End a unit that a shortage starves
status: refined
created: 2026-08-31
implements: [ADR-0006 D1, ADR-0006 D2, ADR-0004 D1, ADR-0014 D3, ADR-0020]
changes: []
creates: []
serves: [PRD-0011, PRD-0013]
blocked-by: [0056]
---

## Why

A unit cannot die. Nothing about a unit is at risk, so no choice about a unit
carries weight and no loss can happen. A shortage that has no end is a number
that nothing reads.

Item 0056 produces the shortage. This item gives it a consequence: a condition
a watcher can name, a condition that gets worse and recovers, and an end when
it lasts too long.

## What the work does

1. A unit that failed its draw accumulates a deficit. The deficit rises while
   the shortage lasts and falls when it ends.
2. The deficit is a condition a watcher can read and name, not a hidden value.
3. A deficit at its maximum sets a bit in a dense plane. After the barrier one
   ascending scan of that plane ends the marked units.
4. What a dead unit carried is accounted for, so the conservation sum still
   balances.

## Impact review

**Governed by.**

- ADR-0006 D1 and D2. The end of a unit is an event: plain data with declared
  padding, and applying it is pure. No `bool` field, and a `u8` instead.
- ADR-0004 D1. The scan of the plane runs in ascending index order and in no
  other order. A bitwise write into disjoint words is commutative, so the
  plane is identical at any thread count; the scan of it must be ordered all
  the same, because the deaths are applied in that order.
- ADR-0014 D3. The generation advances when the engine frees a slot. A unit
  that starves must never hand its identity to the unit spawned next in that
  slot. PRD-0011 states this as a requirement and the accepted record already
  holds it.
- ADR-0020 batches a structural change at the barrier and applies it by
  tombstone and compact. A death is a structural change, so it goes through
  that path.[^1] [^2] [^3] [^4]

**Changes.** No record changes.

**Creates.** No record. **This is a deliberate judgement against the scope
rule, and here is the reasoning.**[^5] The threshold-crossing mechanism is the
claim, and ADR-0063 holds it: a need is a rate with a threshold, and crossing
it is a fact. This item is the first reader of that claim, not a second claim.
The death rule itself fails condition two of the scope test: it is a rate
against a bound, and changing it is a parameter change, not a rewrite.

**This item resolves point one of item 0050.**[^6] PRD-0011 owns the rule by
which a unit ends. PRD-0013 owns the draw and the condition. This item
implements the first and reads the second, so the need is declared once.

**Blockers.** BLK-007 governs every cost figure, so this item states none.

**Precedent.** FND-048 records that a determinism test cannot see a broken
invariant, and this work adds an invariant that only the conservation sum
sees.[^7] The testing rule states the sharper form of the same point: a
determinism test cannot tell correct from consistently wrong, so the draw that
keys the death must be tested field by field.[^8]

**Serves.** PRD-0011 and PRD-0013.

**Conflict surface.** `crates/cachette-core/src/cohort.rs`,
`crates/cachette-core/src/soldier.rs` at the despawn path, and
`crates/cachette-core/src/world.rs` at the step and the event log. **It cannot
run beside item 0056** or **item 0060**; all three write the death path.

## Done when

- A unit that fails its draw enters a condition, and a watcher reads the
  condition by name through the public interface.
- The condition gets worse while the shortage lasts, and it recovers when the
  shortage ends. A test asserts both directions.
- A shortage that lasts long enough ends the unit, and the bound is a
  parameter rather than a constant in the kernel.
- The dead unit's identity never resolves to the unit spawned next in its
  slot, and a test asserts it.
- The conservation sum still balances after a death, and a test asserts it
  across a run in which many units die.
- A property test asserts that the same seed gives the same deaths, in the
  same order, at 1, 2 and 12 threads.
- A test perturbs the scan order behind a test-only switch and asserts that
  the determinism test then fails.[^8]
- The fixture starves some units and not others, and the commit body says how
  that was checked.[^9]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0006, an event is plain data and applying it is pure. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^4]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
[^5]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^6]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
[^7]: Findings register, FND-048. `docs/FINDINGS.md`
[^8]: Testing Rules, sections 1 and 2. `.claude/rules/testing.md`
[^9]: Findings register, FND-051. `docs/FINDINGS.md`
