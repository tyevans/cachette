---
id: 0235
title: Give a register number one authority a writer can consult
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**One question has two authorities and neither can see the other.**

Each register carries a next-number line, and a writer claims the number before
writing the row. That remedy was installed after two writers collided, and it is
correct for writers who commit between claims.[^1]

The line answers from merged history. A writer on a branch takes a number, and
the line does not change for anybody else until the branch merges. A dispatcher
issues ranges above the line for exactly that reason, and those ranges live in
prompts, which no register can read and no check can see.

Four collisions happened in one session, and every writer followed the
documented procedure correctly. FND-219 holds the case and the evidence.[^2]

**The collision costs nothing and the repair costs a sweep.** Both writers are
correct at the moment it happens, both rows are good, and both branches are
green. It surfaces at the merge, and moving a row means finding every citation
of the old number across records, registers, reviews, backlog items and source
comments. One of the four has already needed that.

**This is the third face of one shape.** A record's status is carried by the
directory its file sits in, so accepting it breaks every citation of the
path.[^3] A retired number is a row the citation check cannot resolve, so a
document explaining why a number went cannot name it.[^4] A number is allocated
by a line that cannot see uncommitted work. Each stores registry state where it
cannot be read atomically, and none of them fails until two readers disagree.

The three may share one repair or may not. Deciding that is part of refining
this item.

## What is missing before this is refined

- The impact review, and whether the three faces above are one item or three.
- Whether the answer is an allocator a writer can consult, a check that fails on
  a duplicate number at merge, or a numbering scheme that cannot collide.
- Whether a check is enough on its own. A check that fails at merge turns a
  silent collision into a loud one, and does not stop the sweep it causes.
- Whether the next-number line survives. It is a cached answer to a question the
  tree can answer, and a cached answer is the shape this project meets most
  often.[^5]
- What happens to a number already taken on an unmerged branch when the scheme
  changes.
- Whether the registers and the record registry take the same answer. The record
  registry allocates through a table of rows rather than a line, and it has its
  own collision history.[^6]
- **The product registry has no next-number line at all.** It says to assign the
  number there before writing the record, and it gives no place that states
  which number is next, so a writer reads the last row of the table. That is the
  exact procedure FND-038 replaced everywhere else, still in use in one
  register, and it produced a collision the same day it was found: a product
  number was taken from the last visible row and the row above it was already
  merged on another branch. Whatever answer this item reaches has to cover the
  register that never got the first fix.
- Whether the citation check should resolve a number it cannot see. It refused
  three numbers that exist on other branches while FND-219 was being written, so
  a register entry cannot cite a row that has not merged. That is the same
  refusal 0198 carries for a retired number, reached from a third direction.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-038. `docs/FINDINGS.md`
[^2]: Findings register, FND-219. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-083. `docs/DECISIONS.md`
[^4]: Backlog item 0198. `docs/backlog/proposed/0198-tell-a-mention-of-a-record-number-from-a-citation.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: ADR Registry. `docs/adrs/REGISTRY.md`
