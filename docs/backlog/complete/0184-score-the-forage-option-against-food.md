---
id: 0184
title: Score the forage option against food
status: complete
created: 2026-09-02
implements: [ADR-0007 D1, ADR-0064 D1, ADR-0004 D3]
changes: []
creates: []
serves: [PRD-0009]
blocked-by: []
---

## Why

The option set holds a row named `forage`. It scores the mean stub value of
the level 1 cell.

**The stub value is noise.** The tile value pass draws a number for every tile
on every tick and adds one, minus one, or nothing. No other system reads it,
and no other system writes it. A unit that forages therefore walks toward a
random walk.[^1]

Item 0183 puts the food of a cell into the summary. This item points the
option at it. After both, the choice pass scores a quantity that another
system writes, and a watcher can check the choice against the ground: the
explanation reports a food value, and the deposit under the unit holds that
food.[^2]

## What the work does

1. The `forage` row of the option table reads the mean food of the cell
   instead of the mean stub value.
2. The option table stays a table of values. The pass calls no content
   code.[^3]
3. The option order does not change. The `forage` row keeps its index, so the
   tie-break order is the order it was.[^4]

## Impact review

**Governed by.** Six decisions govern this work.

ADR-0064 D1 states that a unit scores a fixed option set and takes the highest
score. The set keeps its size and its members. Only the field that one row
reads changes.[^5]

ADR-0064 D2 states that a score is transient and that only the choice reaches
state. This item stores no score.[^6]

ADR-0064 D3 states that an option at or below a floor holds the choice, and
that the floor is a frame-budget parameter. The floor does not change. The
mean food of a cell is larger than the mean height of a cell, so more units
reach the floor through this option than reached it before, and the mover
count rises. BLK-007 governs the size of that rise, because no measurement
exists on the target platform.[^7] [^8]

ADR-0064 D5 states that a tie breaks by the lowest option index. The order is
unchanged, so the tie-break is unchanged.[^9]

ADR-0007 D1 and D3 state that content supplies a key vector and never a
comparator, and that the engine never calls content code from inside a
sort. The option table stays a table of values.[^3]

ADR-0004 D1 and D3 state that iteration order is explicit and that a reduction
which is not order-free needs a slot. The option scan is unchanged.[^4]

**Changes.** No record changes. No record names the field that the `forage`
row reads.

**Creates.** No record. The change replaces one table entry with another and
states no new constraint.[^10]

**Blockers.** BLK-007 governs the cost figures. No measurement exists on the
target platform.[^8]

**Precedent.** FND-180 records that the engine computes a decision for every
unit and then discards it. FND-181 records that the rules against inert work do
not find inert data, and states the falsification: pin the value to a constant
and watch the suite stay green. This item is not done until that pin makes a
test fail.[^11] [^12]

## Answers to what was missing

**Does the stub value keep a reader?** Yes, for now. The viewer paints the tile
value at level 0, and the summary still holds the value total. After this item
no option row reads the mean value, so the last engine reader of the summary
field is gone. Item 0188 already holds the question of whether the pass that
computes the stub value should stay, and this item does not answer it.[^13]

**Does the option order change?** No. See decision 3 above.

## Done when

- The `forage` row reads the mean food of the level 1 cell.
- The option order and the option count are unchanged.
- A test drives the engine, puts one unit in a food-rich cell and one in a
  food-poor cell, and asserts that only the first forages.
- A test pins the food total of the summary to a constant, and the test above
  fails. The failure is reported, and the source is restored.
- The explanation reports the food value that the `forage` row read.
- The whole check command runs green.

## Outcome

The `forage` row reads the mean food of the level 1 cell. The option order, the
option count and the floor are unchanged.

**The behaviour changed, and two golden files record it.** The gathering
scenario and the soldier scenario now choose differently, because a hungry unit
scores the food under it instead of a random walk. Both files were recorded
again from this source. No other golden file moved.

**The stub value keeps one reader.** The viewer paints the tile value at level
0. No option row reads the mean value any more, and the summary field it comes
from is now unread. Item 0188 holds the question of whether the pass that
computes the stub value should stay, and this item did not answer it.

**Registers.** FND-183 was added. No blocker opened or closed.

**Evidence.** Three pins were run separately, and the source was restored after
each. Pinning the stored food total of every tile to a constant failed both new
choice tests. Pinning the mean food to a constant failed both. Pointing the
`forage` row back at the stub value failed both.

A fourth pin failed to falsify anything, and FND-183 records it: pinning the
food total **accessor** left all twelve choice tests green, because the mean
divides the private field and never calls the accessor. A pin is evidence only
when it reaches the consumer.

## References

[^1]: What a unit does in a tick, section 1. `docs/research/what-a-unit-does-in-a-tick.md`
[^2]: Backlog item 0183, carry the food of a cell into the level 1 summary. `docs/backlog/complete/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
[^3]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D3. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^4]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^6]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^7]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^10]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^11]: Findings register, FND-180. `docs/FINDINGS.md`
[^12]: Findings register, FND-181. `docs/FINDINGS.md`
[^13]: Backlog item 0188, show the food of a tile and the reason a unit chose. `docs/backlog/proposed/0188-show-the-food-of-a-tile-and-the-reason-a-unit-chose.md`
