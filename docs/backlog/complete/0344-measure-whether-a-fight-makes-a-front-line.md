---
id: 0344
title: Measure whether a fight makes a front line
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

A design sketch for combat resolves a fight for each level 1 cell, as a small
table over unit types. A cell summarises a block of tiles, and the block edge is
a power of two set by one constant in the bridge.

**A fight resolved for a whole block kills units across the whole block.** So
the casualties may not form a front line, and an army may read as a smear. Two
factions had never been run into contact in this engine, so there was no
evidence either way. One blocker held that gap.[^1]

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

**Blockers.** BLK-052 governed this item, and this item closes it.

**Precedent.** The testing rule says a fixture that models the typical case
supplies no extreme, and the findings register held two instances of that shape
before this one.[^7] This item therefore measures four arrangements and not one,
and one of the four exists to show that a thin arrangement reports no smear.

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

## Outcome

**Done. A fight resolves at the tile.**

The band that holds the middle 90 percent of the casualties is 1 tile wide at
the tile, in all four arrangements. It runs from 1 to 30 tiles wide at the level
1 cell, against a block edge of 32. The furthest casualty of a cell resolution
stood 36 tiles from the nearest enemy. Between 67 and 72 percent of the
casualties of a cell resolution stood on a tile that held no enemy. The review
holds the table, the method and the machine.[^9]

**What changed from the plan.** Nothing in the plan. Two results arrived that
the plan did not ask for. The first is that an arrangement two tiles deep
reports no smear, so one typical fixture would have closed the blocker with the
wrong answer. The second is that ordinary ground holds 8 units and the admission
rule reads the capacity and not the faction, so an army packed to that capacity
cannot be entered and therefore cannot be fought by a same-tile rule.

**Registers.** BLK-052 is resolved and BLK-080 is open.[^1] [^10] DEC-144 is
closed on Option B, and DEC-170 is open.[^6] [^11] Findings FND-390 to FND-393
hold the measurement, the fixture lesson, the tile capacity bound and the tank
test.[^12]

## References

[^1]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^2]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: Decisions register, DEC-144. `docs/DECISIONS.md`
[^7]: Testing rules, section 2a. `.claude/rules/testing.md`
[^8]: PRD-0030, a developer builds a game the engine did not anticipate. `docs/product/shaped/prd-0030-a-developer-builds-a-game-the-engine-did-not-anticipate.md`
[^9]: Review, does a fight make a front line. `docs/reviews/0344-does-a-fight-make-a-front-line.md`
[^10]: Blockers register, BLK-080. `docs/BLOCKERS.md`
[^11]: Decisions register, DEC-170. `docs/DECISIONS.md`
[^12]: Findings register, FND-390 to FND-393. `docs/FINDINGS.md`
