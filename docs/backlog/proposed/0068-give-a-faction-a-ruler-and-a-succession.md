---
id: 0068
title: Give a faction a ruler and a succession
status: proposed
created: 2026-08-31
implements: [ADR-0007 D1, ADR-0007 D2, ADR-0004 D1, ADR-0004 D4, ADR-0014 D1]
changes: []
creates: []
serves: [PRD-0016]
blocked-by: [0066, 0067]
---

## Why

A faction is a label. Nothing holds the position of deciding, so two factions
differ only by where they stand, no death matters more than another, and no
period of a run can be a crisis.

This item gives a faction somebody in charge and a rule by which the position
changes hands. It is the last item in the plan, and that is deliberate: the
succession rule must read the world, and until descent, characters, sites and
work exist there is nothing in the world worth reading.

## What the work does

1. A faction has at most one ruler, and the ruler is a character in the world.
2. When a ruler ends, another character takes the position by one rule the
   world applies: filter the candidates, sort them by a key vector, allocate.
3. The rule admits appointment as well as descent.
4. What the ruler is reaches a unit as a value the unit already reads. No
   unit walks to its faction and then to the ruler.
5. A ruler changes at a frame barrier, so no unit in a tick sees two rulers.

## Impact review

**Governed by.** ADR-0007 D1 and D2 require a key vector and a stable final
field, and the research makes the same rule the centre of the succession
design: **do not accept a comparator from a policy author**, because an
intransitive comparator makes the sort's output depend on the algorithm, and a
tie-break on the identifier does not repair a cycle.[^1] [^2] ADR-0004 D1 and
D4 fix the order.[^3] ADR-0014 D1 makes the ruler a generational identity, so
a vacant position and a dead ruler are two different states.[^4]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-013 gives the faction ceiling and BLK-004 the size of the population that
can hold a position.[^5] [^6] **BLK-011 is resolved and it is the constraint
that shapes the rule**: a character raised from the ranks cannot inherit by
blood but may be appointed, so the succession must admit appointment or the
promotion path of item 0066 leads nowhere.[^7]

**Serves.** PRD-0016.

**Conflict surface.** `crates/cachette-core/src/ruler.rs` is new, and
`crates/cachette-core/src/character.rs` gains a position column.
`crates/cachette-core/src/world.rs` at the barrier. **It shares
`character.rs` with item 0067**, so the two do not run beside each other.

## What is missing before this is refined

**The registry row.** This work states a constraint that no reserved row
holds: **succession is filter, then sort by a key vector, then allocate, and a
policy never supplies a comparison function.** All three conditions of the
scope rule hold.[^8] Every succession law is then a row in a table rather than
code, which is a claim a contributor could reasonably reject in favour of a
callback. Changing it later means rewriting every law. The reasoning is the
intransitivity argument above, and it is not visible anywhere in a sort call.
**Allocate the row in the registry before writing the record.**[^9]

**How the ruler reaches a unit.** PRD-0016 rejects the shape where a unit
follows a link to its faction and then to the ruler, and states no
replacement. The likely answer is that the ruler writes a value the faction
holds, and the unit reads the faction value it already reads. That is a
recommendation and nobody has recorded it. Answer it in the impact review,
because it decides whether this item touches the hot path at all.

**What a run does with no ruler.** PRD-0016 requires the world to state what a
faction without a ruler does. Nothing states it. That is a design answer and
this item cannot invent it.

## Is this worth building yet

**Not until items 0066 and 0067 are complete, and this plan orders it last for
that reason.** A ruler whose succession reads a world with no descent, no
sites and no work has nothing to sort on, so the rule would rank candidates by
their identifier and the record would state a constraint that nothing tests.
That is the shape FND-047 records: a record written for a subsystem nobody had
built.[^10] Build the world the rule reads first.

## Done when

- A faction has at most one ruler, and a watcher asks who it is.
- The ruler is a character that stands somewhere, that a watcher can find, and
  that can end.
- Replacing the ruler in an otherwise identical world makes the faction behave
  differently, and a test asserts it.
- When a ruler ends, the rule chooses another from the world. Nothing outside
  the simulation chooses.
- The rule admits appointment, and a test asserts that a character with no
  ancestry can take the position by appointment and cannot take it by
  descent.[^7]
- A faction can hold the position vacant, and the world states what it then
  does.
- Two claimants resolve by the stated rule, and a test constructs the contest
  rather than hoping for it.
- A watcher reads who has ruled a faction, in order, back to the founding.
- The change happens at a barrier, and a test asserts that no unit in one tick
  sees two rulers.
- The succession considers a bounded candidate set, derived from what the
  world already indexes. A test asserts that the cost does not grow with the
  population.
- A property test asserts that the rulers and the successions are identical at
  1, 2 and 12 threads.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^2]: The character graph and inheritance. `docs/research/reports/14-character-graph-and-inheritance.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^5]: Blockers register, BLK-013. `docs/BLOCKERS.md`
[^6]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^7]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: ADR Registry. `docs/adrs/REGISTRY.md`
[^10]: Findings register, FND-047. `docs/FINDINGS.md`
