---
id: 0067
title: Record a parent and walk a line
status: complete
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0004 D1, ADR-0014 D3, ADR-0003 D1]
changes: []
creates: [ADR-0078]
serves: [PRD-0015]
blocked-by: []
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
4. A line that ends is reported as ended.
5. A death releases the relation edges of the character and keeps its
   descent.
6. The record of descent survives the death of the character it names.

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

**Serves.** PRD-0015. This item covers every checkable statement in that
record except the household one, which item 0059 and item 0103 cover between
them.[^15]

**Conflict surface.** `crates/cachette-core/src/descent.rs` is new, and
`crates/cachette-core/src/character.rs` gains the parent columns. Item 0066 is
complete and it created that file, so nothing holds this item now. **It touches
no file outside the character tier**, so it runs beside every item from 0053 to
0065.

## The questions that held this item, and their answers

**DEC-003 is answered, and the answer does not block this item.** A dead
character drops its relation edges. It keeps its row, its two parent edges and
its child list.[^13] A relation edge is a non-kin social tie. A parent edge is
descent. The two are different structures, and only the first one goes.[^7]

The consequences for this item are three.

1. A line is walkable through a dead ancestor, and every statement in the list
   above stands as written. The genealogy is append-only and it holds every
   dead character.[^1]
2. The storage that a death releases is the relation edge set, and never the
   descent record. Say that where the code releases it, and test it.
3. The project accepts that it cannot ask who a dead person knew. Do not add a
   reader for that, and do not keep an edge to serve one.

**The registry row is allocated.** This work states a constraint that no other
row holds: **descent is kept for a bounded set, and a relation is computed by a
bounded recursion, never by walking two lines to a common ancestor.** The
registry holds the row, and the work writes the record.[^9]

All three conditions of the scope rule hold.[^8] Walking the lines is the
obvious implementation and PRD-0015 rejects it, so a contributor could
reasonably choose otherwise. Changing it later means rewriting every reader,
which costs more than the record. The reasoning is not visible in a recursion.
The counter-test agrees: the record fixes the visit order of a walk, and order
is a determinism property.

**The truncation depth.** The relation is exact only to a stated depth, and the
research gives a depth at which no step rounds.[^1] That depth is a parameter
and it belongs in the reference tables, not in the record.[^10] Read it; do not
invent it.

**The household. Answered, and it leaves this item.** A dwelling is stored and
a household is derived from it.[^14] A unit carries the slot of the dwelling it
lives in, and a household is every unit that carries one slot. Nothing stores a
household and nothing declares one.

**A household therefore reads no descent, and this item does not build one.**
The recommendation this plan carried was that a household is the residents of
one site who share a line. That is rejected. A household is a fact about a
place, not a fact about a family, so it depends on the dwelling slot of item
0059 and not on the descent this item records. Point five of item 0050 is
closed.[^11]

Two consequences bind this item.

1. **Do not add a household reader here, and do not add a kinship column to
   serve one.** The work that derives a household is filed separately.[^15]
2. **Descent and residence stay two independent facts.** A parent and a child
   who live apart are still a parent and a child, and two strangers who share
   a roof are one household. No code in this item may assume otherwise.

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
- A death releases the relation edges of the dead character, and a test reads
  the descent of that character after the release.[^13]
- A property test asserts that the parentage is identical, and recorded in the
  same order, at 1, 2 and 12 threads.
- The birth draw is keyed. A test changes the frame and asserts the draw
  changes; a second test changes the mother and asserts the draw changes.[^12]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

**The record of descent is built, and every statement in the list above is
covered by a test that drives the world.** A character records its two
parents. A watcher walks to the ancestors and to the descendants of a
character. The relation between two characters is a Q16.16 value that a
property test checks against a slow reference at exact integer arithmetic. A
line that ends is reported as ended, and a death releases the slot columns and
keeps the descent.

**Descent is keyed on an identity that the record never reissues.** A parent
edge names a row of the record, not a slot and not an entity identity. Each row
keeps the identity the arena minted, so a watcher reads a dead parent through a
living child and the character created next in that slot never answers to it.
ADR-0078 holds the constraint and its registry row holds `Draft`.[^16]

**The defect was put back twice, and once more for the walk order.** A
creation that kept the descent row of the character before it failed one test
of sixteen, and both property tests passed. The fixture built the pedigree from
births alone, so it never freed a slot and never reached the case. A removal
was added to the fixture and a test was added that asserts the fixture reuses a
slot. The same defect then failed three tests. The finding holds the
detail.[^17]

**The relation depth and the record ceiling are in the reference table.** The
depth was not there when this item was written. It is derived from the research
and it is now a row, with the derivation that makes every step of the recursion
exact.[^10]

**One thing in the list above has no code to write.** Point five asks that a
death releases the relation edges of the character. A relation edge is a non-kin
social tie, and no such structure exists in the tree yet. The storage split
this item built is the answer: a fact that a death must release lives in the
slot columns, which the next character overwrites, and descent lives in the
record. The record states that, and a test asserts the half that is
observable.[^7]

**The birth draw needed a subject, and the work chose one.** The item requires
a keyed birth draw and a test for each field of the key. Nothing else in the
item draws, so the birth draws the sex of the child. That concept is new and no
product record asked for it, so the decision register holds the choice with its
options for review.[^18]

Register entries that moved: FND-171 opened, DEC-071 opened. The golden state
hashes were regenerated, because the character hash now writes the descent
count and the sex column into the stream. No scenario in the golden set holds a
character, so no behaviour changed.

## References

[^1]: The character graph and inheritance. `docs/research/reports/14-character-graph-and-inheritance.md`
[^2]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^5]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^7]: Decisions register, DEC-003. `docs/DECISIONS.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: ADR Registry, row 0078. `docs/adrs/REGISTRY.md`
[^10]: Budgets and costs. `docs/reference/budgets.md`
[^11]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
[^12]: Testing Rules, section 2. `.claude/rules/testing.md`
[^13]: The character graph and inheritance, section 2.2. `docs/research/reports/14-character-graph-and-inheritance.md`
[^14]: Decisions register, DEC-039. `docs/DECISIONS.md`
[^15]: Backlog item 0103. `docs/backlog/proposed/0103-derive-a-household-from-the-dwelling-slot.md`
[^16]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
[^17]: Findings register, FND-171. `docs/FINDINGS.md`
[^18]: Decisions register, DEC-071. `docs/DECISIONS.md`
