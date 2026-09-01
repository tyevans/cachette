---
id: 0067
title: Record a parent and walk a line
status: proposed
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0004 D1, ADR-0014 D3, ADR-0003 D1]
changes: []
creates: []
serves: [PRD-0015]
blocked-by: [0066]
---

## Why

A character stands in no relation to any other character. The population is a
set of strangers, and it is the same set after every member has been replaced.
A death reaches nobody, a watcher cannot follow anything across a generation,
and nothing can pass to somebody by right.

This item records where a character came from. Item 0068 reads it to decide
who takes a position when its holder dies.

## What the work does

1. A character records its parents, and a character with no parents is a
   representable state rather than an invented one.
2. A watcher walks from a character to its ancestors and to its descendants.
3. The relation between two characters is a value the world computes exactly.
4. A line that ends is reported as ended and releases what it held.
5. The record of descent survives the death of the character it names.

## Impact review

**Governed by.** ADR-0002 D1 makes the relation an exact value; the research
shows that it is exactly representable in Q16.16 down to the truncation depth,
because every step of the recursion halves a value.[^1] ADR-0014 D3 makes a
recorded parent name an identity that is never reissued, so a reference to a
dead parent resolves to that parent or to nothing.[^2] ADR-0004 D1 fixes the
visit order of every walk, because a label that depends on visit order is a
determinism hole.[^3] ADR-0003 D1 keys a birth draw, and the research notes
that it keys on the mother, because the child has no identity when the draw
happens.[^4]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-004 gives the size of the population that carries a line.[^5] **BLK-011 is
resolved and this item must honour it**: a character raised from the ranks
receives no invented ancestry, so a line that starts at zero must be
expressible and must not be a special case in the recursion.[^6]

**Serves.** PRD-0015.

**Conflict surface.** `crates/cachette-core/src/descent.rs` is new, and
`crates/cachette-core/src/character.rs`, which item 0066 creates, gains the
parent columns. **It touches no file outside the character tier**, so once item
0066 lands it runs beside every item from 0053 to 0065.

## What is missing before this is refined

**DEC-003 is open, and this item cannot start under an assumption.** The
question is whether a dead character keeps its relation edges. The register
carries a recommendation to drop them and states what retention costs at the
target, and the recommendation has not been taken.[^7] The answer decides
whether a line that ends releases storage or keeps it, which is one of the
five statements this item is meant to satisfy. **Close DEC-003 before refining
this item.**

**The registry row.** This work states a constraint that no reserved row
holds: **descent is kept for a bounded set, and a relation is computed by a
bounded recursion, never by walking two lines to a common ancestor.** All three
conditions of the scope rule hold.[^8] Walking the lines is the obvious
implementation and PRD-0015 rejects it. Changing it later means rewriting every
reader. The reasoning is not visible in a recursion. **Allocate the row in the
registry before writing the record.**[^9]

**The truncation depth.** The relation is exact only to a stated depth, and the
research gives a depth at which no step rounds.[^1] That depth is a parameter
and it belongs in the reference tables, not in the record.[^10] Read it; do not
invent it.

**The household.** Point five of item 0050 asks whether the household of
PRD-0015 and the place to live of PRD-0014 are the same thing.[^11] **This is
the item where the question becomes cheap, and it is where it should be
answered.** The recommendation this plan carries is that a household is
derived, not stored: it is the residents of one site who share a line, and both
halves already exist by the time this item runs. Confirm or reject that in the
impact review.

## Done when

- A character born in the world has a recorded parent, and a watcher asks who
  it is.
- A character with no parents is representable, its relation to every existing
  character is zero, and no parent is invented. A test asserts it.
- A watcher walks from a character to its ancestors and to its descendants.
- The relation between two characters is exact, and a property test asserts
  the value against a slow reference implementation.
- A character is never recorded as its own ancestor, and a test asserts the
  refusal.
- A line that ends is reported as ended.
- The record of descent survives the death of the character it names, and a
  test reads a dead parent through a living child.
- A property test asserts that the parentage is identical, and recorded in the
  same order, at 1, 2 and 12 threads.
- The birth draw is keyed. A test changes the frame and asserts the draw
  changes; a second test changes the mother and asserts the draw changes.[^12]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The character graph and inheritance. `docs/research/reports/14-character-graph-and-inheritance.md`
[^2]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^5]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^7]: Open decisions register, DEC-003. `docs/DECISIONS.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: ADR Registry. `docs/adrs/REGISTRY.md`
[^10]: Budgets and costs. `docs/reference/budgets.md`
[^11]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
[^12]: Testing Rules, section 2. `.claude/rules/testing.md`
