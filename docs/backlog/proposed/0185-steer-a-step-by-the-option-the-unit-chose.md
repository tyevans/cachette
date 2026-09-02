---
id: 0185
title: Steer a step by the option the unit chose
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0009]
blocked-by: []
---

## Why

**The engine computes a decision for every unit on every tick, and then
throws the decision away.** The movement pass reads the option column, tests
that it holds a value, discards the value, and draws a uniform direction. A
unit that chooses to forage takes the same distribution of steps as a unit
that chooses to climb.[^1]

Everything upstream of that column is paid for and unused. The level 1
rebuild, the cell summary, the need column, the option weights and the stagger
schedule all feed one bit: whether the unit moves at all.

This is the break that makes every other coupling invisible. The product
record this project points at is unmet at the action, although each of its
statements passes at the choice.[^2]

## What the work does

1. After the level 1 rebuild, the engine derives one exit direction for each
   cell and each option. The direction names the neighbouring cell that ranks
   highest on the field that option reads.
2. A cell that ranks highest against its own neighbours holds no exit
   direction.
3. The movement pass reads the option of the unit and the exit direction of
   its cell, and takes that direction. A unit in a cell with no exit direction
   keeps the uniform draw it takes today.

**The flow field is the shape this project prefers.** A set-valued command
permits a cheaper algorithm, and a field over the cells costs the cell count
rather than the unit count.[^3] The alternative, where each unit scores its
six neighbours, costs the population and gives the same answer for every unit
of one cell.

## The record this needs

**This item creates an architecture decision record, and no number is reserved
for it.** The claim is that movement takes its direction from the choice
through a per-cell field, and never from a per-unit search. It passes the
three-condition test: a contributor would reasonably make each unit score its
own neighbours, the cost of choosing otherwise is the population against the
cell count, and the reason the cheap method gives the same answer is invisible
in the loop.[^4] Allocate the registry row before the work starts.[^5]

## What is missing before this is refined

- The impact review, and the registry row.
- Whether the derived field is a projection recomputed each frame or carried
  state. A projection recomputed each frame stays inside the rule that level 0
  is the only truth. Carried state raises the question DEC-067 already
  holds.[^6]
- What fixes the order when two neighbouring cells rank equal. The direction
  index is the candidate, and it must be stated rather than assumed.
- Whether a whole cell moving in one direction reads as a crowd or as a block.
  Only a run settles this.
- What this does to the movement admission, which refuses a full tile. A cell
  that streams into one face may exhaust the capacity of the tiles there.

## Done when

Stated when the item is refined. It must include the test FND-180 asks for:
pin the option column to one value and assert that a test then fails.[^7]

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-180. `docs/FINDINGS.md`
[^2]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^5]: Definition of Done, section 2. `.claude/rules/definition-of-done.md`
[^6]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^7]: Testing Rules, section 2a. `.claude/rules/testing.md`
