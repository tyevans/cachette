---
id: 0344
title: Measure whether a fight makes a front line
status: refined
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: [BLK-052]
---

## Why

A design sketch for combat resolves a fight for each level 1 cell, as a small
table over unit types. A cell summarises a block of tiles, and the block edge is
a power of two set by one constant in the bridge.

**A fight resolved for a whole block kills units across the whole block.** So
the casualties may not form a front line, and an army may read as a smear. Two
factions have never been run into contact in this engine, so there is no
evidence either way. One blocker holds that gap.[^1]

The measurement is cheap and it decides between two designs. Building either one
first risks throwing it away.

## Impact review

**Governed by.** ADR-0001 D1 requires that one binary gives one answer at any
thread count, so the measurement must not change with the thread count of the
barrier.[^2] ADR-0002 D1 forbids a floating point number in simulated or
aggregated state, so every figure here is an integer.[^3] ADR-0018 D3 says the
unit-to-tile bridge rebuilds at the barrier, and the measurement reads it after
a rebuild and never during one.[^4] ADR-0004 D1 requires an explicit iteration
order, so the harness groups by an integer key and walks a sorted vector.[^5]

**Changes.** No record. This item writes no combat pass and adds no engine
capability. It adds one test file to the core crate.

**Creates.** No record. The decision that the measurement settles already has a
row in the decisions register.[^6]

**Blockers.** BLK-052 governs this item, and this item closes it.

**Precedent.** The testing rule says a fixture that models the typical case
supplies no extreme, and the findings register holds two instances of that
shape.[^7] This item therefore measures four arrangements and not one, and one
of the four exists to show that a thin arrangement reports no smear.

**The provisional casualty rule is thrown away.** Nothing kills a unit in a
fight today, so the measurement needs a rule to place casualties on tiles. That
rule lives in the test file, it states no game rule, and item 0345 owes it
nothing.

**The run does not need the target platform.** A band is a shape and not a cost,
so a development machine serves. The report names the machine anyway.

**Serves.** PRD-0030, a developer builds a game the engine did not
anticipate.[^8]

## Done when

- The band that holds the middle 90 percent of the casualties is reported in
  tiles, at the tile and at the level 1 cell.
- The defect is put back: the same fixtures resolve at the cell on purpose, and
  the band is reported for that run beside the other.
- The tank test is stated structurally, with the threshold applied before the
  sum and after it, and the two runs differ in that order and nothing else.
- A case where the model gives a result a player would call wrong is named.
- Every figure names the machine that produced it, and every number says
  whether it came from the engine or from a model.
- The whole check command runs green.

## References

[^1]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: Decisions register, DEC-144. `docs/DECISIONS.md`
[^7]: Testing rules, section 2a. `.claude/rules/testing.md`
[^8]: PRD-0030, a developer builds a game the engine did not anticipate. `docs/product/shaped/prd-0030-a-developer-builds-a-game-the-engine-did-not-anticipate.md`
