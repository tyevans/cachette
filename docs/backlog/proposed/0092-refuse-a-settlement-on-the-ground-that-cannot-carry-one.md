---
id: 0092
title: Refuse a settlement on the ground that cannot carry one
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0006, PRD-0012, PRD-0014]
blocked-by: []
---

## Why

A settlement stands on any tile the world holds, and that includes water.

The settlement arena refuses a tile another settlement holds, and it refuses
an address outside the world. It applies no rule about the ground. The item
that built the arena states this in its outcome, and it states it correctly:
no record forbids it, and that item stated no new rule.[^1]

The project owner states that this is not wanted in the long term.

A unit is already refused on water. The world reads the ground and passes the
refusal into the spawn and into movement admission. A settlement reads nothing.
So the world holds two answers to one question, and one of them is silent.

## The question this item must answer first

**Is the ground that carries a settlement the same ground that carries a
unit?**

The two answers give different work.

If they are the same, the fix is small. A settlement founding calls the
refusal the spawn already calls, and passability stays one property.

If they differ, the ground needs a second property. A unit crosses a mountain
and a settlement may not stand on one. A record must then state what that
property is and why the project cannot derive it from passability. A record
that states a value the code can compute is the shape the scope rule
refuses.[^2]

**This item stays in `proposed/` until that question is answered**, because
the impact review cannot name the records it creates before then. The
question is a judgement, not information the project lacks, so it belongs in
the decisions register rather than the blockers register.

## What makes this more than one call site

Item 0071 records that passability already has two declaration sites: the
capacity table states that a capacity of zero is the whole of passability,
while `is_passable` matches on the water kind by name, and the second site is
the one every engine caller uses.[^3] A third caller must not add a fourth
site.

Whoever takes this item takes 0071 with it, or waits for it. Adding a
settlement rule on top of a fact that is already declared twice is the shape
the recurring defect rule names first.[^4]

## Done when

- A settlement founding on ground that cannot carry one is refused, and the
  refusal is a typed error rather than a silent failure.
- The rule reads one declaration site. A whole-tree search finds no second
  site that states which ground carries a settlement.
- A test founds a settlement on water and asserts the refusal. The fixture
  asserts that the world it built actually holds water, because a test world
  narrower than the generator lattice holds one kind of ground.[^5]
- The defect is restored and the test is watched failing before the item is
  claimed done.
- Any world the founding work builds places its settlements on ground the
  rule accepts.
- `just check` exits 0.

## References

[^1]: Backlog item 0052, the outcome. `docs/backlog/complete/0052-provide-the-settlement-column-set.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Backlog item 0071. `docs/backlog/proposed/0071-derive-tile-passability-from-tile-capacity.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Findings register, FND-054. `docs/FINDINGS.md`
