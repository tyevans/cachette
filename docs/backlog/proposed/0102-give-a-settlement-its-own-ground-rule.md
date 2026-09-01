---
id: 0102
title: Give a settlement its own ground rule, separate from passability
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: [ADR-UNALLOCATED]
serves: [PRD-0006, PRD-0012]
blocked-by: [0071, 0092]
---

## Why

The world has one ground property. It answers whether a unit may stand on a
tile, and every caller reads it.

That property answers two questions at once. A unit crosses a mountain. A
settlement cannot occupy one. Today the two answers are the same, and they are
the same by accident rather than by decision.

The project owner decides that a settlement reads a suitability rule of its
own.[^1] The tile kind gains a second property, and the settlement reads that
one instead of passability.

Item 0092 refuses a settlement the ground that carries nobody at all, and it
reads the passability of the tile to do it.[^2] That item is correct and it
stays correct. Every ground it refuses stays refused under the wider rule,
because ground that carries no unit carries no settlement either. This item
widens the rule. It does not rewrite the earlier one.

## What the work does

1. The tile kind gains a suitability property beside the capacity.
2. The settlement founding reads the suitability property, not passability.
3. A check fails when the two properties disagree in a way the record forbids.
4. A record states the claim and states what the two properties may not do.

## What is missing before this is refined

- **The registry row.** This item creates a record, and the backlog rule
  requires the row before the item is refined.[^3] The dispatcher allocates
  the number.
- **The rule between the two properties.** Ground that carries no unit carries
  no settlement. The reverse does not hold. Whether that implication is a
  compile-time property of the table, a runtime check, or a test is the
  decision the record makes, and this item does not make it here.
- **The value for each ground kind.** Which kinds carry a unit and refuse a
  settlement is a data question, and today the mountain is the only case
  anybody has named.

## The cost this rule takes on

**Every new ground kind now costs two values instead of one, and the second is
a second declaration site.**[^1] Shape 1 of the recurring defect rule names
this exactly: one fact in two places, both authoritative, nothing that fails
when they disagree.[^4]

The project already carries one instance of the shape on this same property.
Item 0071 records that passability has two declaration sites, and the findings
register holds the instance.[^5] [^6] **That is why item 0071 and item 0092 both
run before this one.** Item 0071 makes the passability reader derive its answer
from the capacity, so this item adds the second property to a table that has
one declaration site rather than two.

The record must therefore carry the check, not only the property. A second
value with no check that constrains it against the first is the defect the rule
predicts.

## Done when

- The registry holds the row, and this item names the record it creates.
- The record states the claim and states the rule between suitability and
  passability.
- The settlement founding reads suitability. A whole-tree search finds no
  settlement caller that still reads passability, and the search command is in
  the commit body.[^7]
- A check fails when the two values disagree in a way no ground kind intends.
  Ground that carries no unit and admits a settlement fails the check. The
  check is code that runs, not a comment that names which value a settlement
  reads. A comment that says which copy loses is evidence that the second copy
  should not exist.[^4]
- A test puts a ground kind into the disagreeing state and watches the check
  fail, so the check is known to reach the case.[^8]
- A test founds a settlement on ground a unit crosses and a settlement may not
  occupy, and asserts the refusal by name.
- A test asserts that item 0092's refusal still holds, so the widening did not
  drop the narrower rule.
- The rule is put back, and the tests are watched failing, before the item is
  claimed done. The restored defect is the smallest change that violates the
  claim.[^8]
- `just check` exits 0.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-035. `docs/DECISIONS.md`
[^2]: Backlog item 0092. `docs/backlog/refined/0092-refuse-a-settlement-on-the-ground-that-cannot-carry-one.md`
[^3]: Backlog guide, the line between proposed and refined. `docs/backlog/README.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Backlog item 0071. `docs/backlog/complete/0071-derive-tile-passability-from-tile-capacity.md`
[^6]: Findings register, FND-060. `docs/FINDINGS.md`
[^7]: Commit Message Rules, after a sweep. `.claude/rules/commits.md`
[^8]: Findings register, FND-070. `docs/FINDINGS.md`
