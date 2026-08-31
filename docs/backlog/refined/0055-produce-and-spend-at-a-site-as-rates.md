---
id: 0055
title: Produce and spend at a site as rates
status: refined
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0002 D3, ADR-0004 D2, ADR-0066 D1]
changes: []
creates: [ADR-0062]
serves: [PRD-0007, PRD-0008]
blocked-by: [0052, 0053]
---

## Why

A site holds a store and nothing fills it. A unit takes from a tile and
carries what it took, and the amount stops there. Nothing turns work into a
quantity that a place keeps.

This item makes production and upkeep the same thing: a rate attached to a
site, applied at an interval. Consumption, housing, work assignment and trade
all read the store this item fills. It is the item that turns the settlement
of item 0052 from a row into an economy.

## What the work does

1. A site carries a production rate and an upkeep rate for each commodity.
2. A rate applies at a stated interval, not on every tick, and the interval is
   a parameter of the schedule rather than a constant in the kernel.
3. The application is one segmented reduction over the sites, in site order.
4. A conservation test covers the store: what a site produced, minus what it
   spent, plus what it held, equals what it holds.

## Impact review

**Governed by.**

- ADR-0066 D1. A settlement is fixed to a tile and holds pooled stores. The
  rate columns belong to that shape and to no other.
- ADR-0002 D1 and D3. A rate is Q16.16 and a store is an integer. The
  accumulator widens, because a rate applied over many intervals overflows a
  narrow type. FND-011 records the same overflow in a progress accumulator,
  and this is the same arithmetic.[^1]
- ADR-0004 D2. The reduction over sites is order-free, because integer
  addition is exactly associative and commutative.
- ADR-0005 D1 and D2 apply if any part of this work iterates: the count is
  fixed and no step stops on a convergence test.[^2] [^3] [^4]

**Changes.** No record changes.

**Creates.** ADR-0062. The registry reserves the row and states the claim:
production and upkeep are rates attached to a site.[^5] The claim passes the
three-condition test. A contributor could reasonably charge upkeep to each
unit on each tick; BLK-008 rejects that and this record is where the rejection
lives.[^6] Changing it later means rewriting every consumer.

**A dependency the registry states, and this item does not take.** Row 0062
depends on row 0055, an ordered modifier pipeline for an effective stat.[^5]
**This item writes ADR-0062 with a base rate and no pipeline, and says so in
the record.** One source modifies a rate today, so a pipeline would be a
capability nothing invokes, which is the third recurring defect shape.[^7] The
scope rule's first condition also fails: with one modifier there is no
decision. **Do not write ADR-0055 until a second modifier source exists.** The
registry row stays reserved and the dependency stays true.

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-008 is resolved and its resolution is binding: consumption is pooled, not
charged to each unit on each tick.[^6] The settlement count and the commodity
recommendation come from the register, not from this item.[^8] [^9]

**Precedent.** FND-012 records that integer decay carries a permanent negative
bias, and FND-016 records that a capacity cap is not a negative rate. Both bear
directly on an upkeep rate applied by a right shift.[^10] [^11]

**Serves.** PRD-0007 asks that what leaves a tile arrives somewhere exactly.
PRD-0008 asks that effort accumulate. This item holds the accumulation. It
builds no improvement; item 0058 does that.

**Conflict surface.** `crates/cachette-core/src/site.rs`, which item 0052
creates, and `crates/cachette-core/src/world.rs` at the step. **It cannot run
beside item 0056**, because both add a stage to the site pass and both edit
the same reduction.

## Done when

- A site produces into its store at an interval, and the store rises by
  exactly the rate multiplied by the intervals that passed.
- A site spends from its store at an interval, and a store that cannot pay
  reports a shortfall rather than going negative.
- The interval is a schedule parameter. No kernel holds it as a constant.
- A property test asserts that the store total is the same whatever order the
  sites were reduced in, and at 1, 2 and 12 threads.
- A property test asserts the conservation equality over a long run.
- A test asserts the shortfall case at the boundary: a store one unit short.
- The negative bias of the rate shift is asserted in a test rather than
  described in a comment, and the test names the direction.
- ADR-0062 is written, the registry row moves to `Draft`, the record states
  no rate value and no cost figure, and it says plainly that no modifier
  pipeline exists yet.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-011. `docs/FINDINGS.md`
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0005, a solver runs a fixed iteration count. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^5]: ADR Registry, rows 0062 and 0055. `docs/adrs/REGISTRY.md`
[^6]: Blockers register, BLK-008. `docs/BLOCKERS.md`
[^7]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^8]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^9]: Open decisions register, DEC-001. `docs/DECISIONS.md`
[^10]: Findings register, FND-012. `docs/FINDINGS.md`
[^11]: Findings register, FND-016. `docs/FINDINGS.md`
