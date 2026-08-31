---
id: 0054
title: Hold a tile for a faction
status: complete
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0004 D1, ADR-0012 D2, ADR-0023 D1]
changes: []
creates: [ADR-0053]
serves: [PRD-0006]
blocked-by: []
---

## Why

A faction is a label on a unit. No tile belongs to anybody, so the world has
no sides. A developer cannot express a boundary, cannot give a unit a place to
belong to, and cannot make a decision matter beyond the unit that made it.

This item gives a tile a holder. It is a small change and a large one: it is
the first fact the world carries about a side rather than about an entity.

## What the work does

1. A tile carries a holder, which is a faction or nobody.
2. The holder changes during a run, by one rule the world applies, and the
   rule reads the terrain.
3. A faction answers what it holds without a pass over the world.
4. The level 1 cell carries the holding as an extensive field, so the answer
   comes from the pyramid.

## Impact review

**Governed by.**

- ADR-0012 D2. Each tile field is its own dense column, so the holder is one
  column and not a plane for each faction.
- ADR-0002 D1. No holding value is a floating point number. A spread rule
  that scores a tile scores it in integer or Q16.16 arithmetic.
- ADR-0004 D1. The spread visits tiles in an explicit stable order, and a
  contested tile resolves by a stable key rather than by which thread wrote
  last.
- ADR-0023 D1. A holding count is a summary field, so it combines exactly and
  in any order. The accumulator widens, because a per-tile count summed over
  the target tile count overflows a narrow type.[^1] [^2] [^3] [^4]

**Changes.** No record changes.

**Creates.** ADR-0053. The registry reserves the row and states the claim: a
faction is a bit in a mask, and a relation is a plane.[^5] This item writes
that record. The claim passes the three-condition test: a contributor could
reasonably store one boolean plane for each faction, PRD-0006 rejects that
shape explicitly, and the reasoning is not visible in a holder column.[^6]

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
faction ceiling is 63, from the scale constants table, and it is read rather
than invented.[^7] The item allocates no faction count of its own; item 0020
already found three faction ceilings where one was enforced, and a fourth is
the shape this project keeps finding.[^8]

**Precedent.** FND-049 records that the term which grows with the number of
things dominates the term that grows with the number of tiles. A spread rule
must therefore cost the area that changed and not the area that exists.[^9]

**Serves.** PRD-0006.

**Conflict surface.** `crates/cachette-core/src/holding.rs` is new.
`crates/cachette-core/src/world.rs` at the step, the state hash and the
invariant check; `crates/cachette-core/src/pyramid.rs` gains a summary field;
`crates/cachette-view` gains a holder layer. It touches `pyramid.rs`, which no
other item in this plan touches, and `world.rs`, which most of them do. **Run
it beside item 0053 only if one of the two lands first**; both edit the step.

## Done when

- A tile answers who holds it, and nobody is a representable answer.
- No tile is held by two factions, and a test asserts it over a long run.
- The holding changes during a run, and a test asserts that the terrain
  changes where it goes.
- A faction answers what it holds at a cost that grows with the holding, and
  a test asserts the answer against a full pass over the tiles.
- A level 1 cell reports the holding exactly, and the existing equality test
  covers the new field.
- A property test asserts that the holdings are identical at 1, 2 and 12
  threads, including a tile that two factions reach on the same tick.
- The fixture holds a contested boundary rather than a single faction, and the
  commit body says how that was checked.[^10]
- ADR-0053 is written, the registry row moves to `Draft`, and the record holds
  no cost figure and no count.
- `just check` runs green.

## Outcome

Done, with one statement of the product record left open.

**What was built.** A tile carries a holder, which names a faction or nobody.
A spread rule runs at the barrier of each frame. A unit standing on a tile
claims it, a claim spreads from a held tile to its neighbours, and the ground
sets the support that a claim must raise. Open water admits no holder. The
world keeps a running count for each faction, a list of the tiles that
somebody holds, and one faction mask for each block. Level 1 carries the held
ground as an extensive field.

**What changed from the plan.** Two things.

The plan expected level 1 to answer what a faction holds. It answers how much
ground is held, and it does not say by whom, because a summary field indexed
by the faction is the shape the record rejects. The per-faction answer is a
running total instead, and the block masks answer where. The record states
both.

The plan named a holder rule that spreads. The rule as written visits the edge
of a holding and not its area, because a tile inside a holding cannot change
hands. The findings register holds the reasoning and the evidence.

**What is not done.** The viewer draws no holder layer, so a watcher cannot
yet see a boundary. Item 0085 holds that work.

**Registers.** FND-066 and FND-067 opened. The registry row for ADR-0053 moved
to `Draft`. No blocker opened or closed. Items 0084 and 0085 opened.

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0023, an aggregate combines exactly, in any order. `docs/adrs/REGISTRY.md`
[^5]: ADR Registry, row 0053. `docs/adrs/REGISTRY.md`
[^6]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^9]: Findings register, FND-049. `docs/FINDINGS.md`
[^10]: Findings register, FND-051. `docs/FINDINGS.md`
