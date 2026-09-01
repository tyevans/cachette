---
id: 0068
title: Give a faction a ruler and a succession
status: refined
created: 2026-08-31
implements: [ADR-0007 D1, ADR-0007 D2, ADR-0004 D1, ADR-0004 D4, ADR-0014 D1]
changes: []
creates: [ADR-0079]
serves: [PRD-0016]
blocked-by: [0067, 0104]
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
4. The ruler contributes a source term to the influence field of its faction.
   A unit reads the level 1 cell it already reads. No unit walks to its
   faction and then to the ruler.
5. A ruler changes at a frame barrier, so no unit in a tick sees two rulers.
6. A faction with no ruler gets no branch. The source term is simply absent.

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
`crates/cachette-core/src/world.rs` at the barrier. It writes a source term
into the influence field that item 0104 builds, and it does not change how that
field is stored or solved. **It shares
`character.rs` with item 0067**, so the two do not run beside each other.

## What DEC-003 means for this item

**A dead character drops its relation edges. It keeps its row, its two parent
edges and its child list.**[^11] [^12] The succession therefore runs, and the
ancestry it reads is intact.

Two consequences bind the design of the rule.

1. **A succession law reads the genealogy of the dead holder, and never the
   social ties of the dead holder.** The eligibility predicate and the key
   vector may use descent, house and birth order, because those survive the
   death. They must not use a non-kin relation of the deceased, because the
   engine has released it by the time the rule runs.
2. **The release and the succession share an order, and this item fixes it.**
   The character-tier barrier consumes the death events and then runs the
   death, the succession and the asset transfer in that order.[^12] The
   release of the relation edges sits inside the death step, so it runs
   **before** the succession. The sequence is therefore:

   1. The barrier consumes the death events.
   2. The death step marks the character dead and releases its relation edge
      set. Its row, its two parent edges and its child list stay.[^11]
   3. The succession runs. It reads descent, house and birth order. It reads
      no relation edge, and by this point there is none to read.
   4. The asset transfer runs.

   **Assert this, do not assume it.** A debug assertion states that the
   relation edge set of the deceased holder is empty when the succession pass
   reads the holder. A test builds a deceased holder that carried relation
   edges, records the successor, then runs the same world with those edges
   absent from the start, and asserts the same successor. A rule that reads an
   edge the release already dropped is a hazard that a single-threaded test
   does not show, because the released memory still reads as something.

The named laws in the research are unaffected. Each one keys on descent, on the
house or on a vote total, and none of them keys on a tie of the deceased.[^2]

## The questions that held this item, and their answers

**The registry row is allocated.** This work states a constraint that no other
row holds: **succession is filter, then sort by a key vector, then allocate,
and a policy never supplies a comparison function.** The registry holds the
row, and the work writes the record.[^9]

All three conditions of the scope rule hold.[^8] Every succession law is then a
row in a table rather than code, which is a claim a contributor could
reasonably reject in favour of a callback. Changing it later means rewriting
every law. The reasoning is the intransitivity argument above, and it is not
visible anywhere in a sort call. The counter-test agrees: an intransitive
comparator makes the output depend on the sort algorithm, which is a
determinism hole.

**How the ruler reaches a unit. Answered.** The writ travels through the
world.[^13] The ruler contributes a source term to the influence field of its
faction, and a unit reads the level 1 cell it already gathers. No unit asks who
rules it, and nothing walks from a unit to its faction.

This answer decides the hot path, and it decides it in this item's favour:
**this item adds nothing to the per-unit gather.** The unit already reads the
field. The ruler writes one source term for each faction, and the number of
factions is bounded and small.

The answer also brings behaviour the engine does not have to pay for. The
influence solve carries terrain conductance, so influence flows around a
mountain rather than through it.[^14] The writ of a ruler therefore runs
strongly near the seat and weakly far from it, and a mountain range obstructs
it. A distant province is less governed than a near one, and no rule states
that.

**The bound.** A ruler sets a field. A ruler does not command a unit. Nothing
in this item may give a ruler a per-unit order, because a per-unit order is a
data plane in Python by another route.

**How the field is stored is not this item.** A proposed record holds it, and
item 0104 builds it.[^15] [^16] That is why this item is blocked on 0104 as
well as on 0067.

**What a run does with no ruler. Answered. Nothing special.**[^17] An absent
ruler is an absent source term. The engine holds no branch for a faction
without a ruler, and no rule asks whether a ruler exists. The solver runs its
fixed iteration count either way.

What a watcher sees follows from the solver and not from a rule. The writ
relaxes from the edge inward, because the periphery is the part the field held
least strongly. The far provinces stop being governed first, and the seat is
the last place to lose its hold. An interregnum is drift rather than a state,
and whoever takes the seat inherits the drift.

**This forbids the obvious implementation.** Do not add a check for a vacant
position on any pass, and do not zero the field when a ruler ends. A branch on
the absence of a ruler costs a check on every pass and produces a worse
result, because it makes the loss instant everywhere rather than gradual from
the edge.

## Is this worth building yet

**Not until item 0067 and item 0104 are complete, and this plan orders it last
for that reason.** Item 0066 is complete. A ruler whose succession reads a world with no descent, no
sites and no work has nothing to sort on, so the rule would rank candidates by
their identifier and the record would state a constraint that nothing tests.
That is the shape FND-047 records: a record written for a subsystem nobody had
built.[^10] Build the world the rule reads first.

**Item 0104 is the second reason.** The writ of a ruler travels through the
influence field, and no influence field exists.[^16] A ruler that writes a
source term into nothing changes nothing, so the statement that replacing the
ruler changes what the faction does could not be tested.

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
- A faction can hold the position vacant. No pass branches on the vacancy, and
  a test asserts that the field of a faction that loses its ruler decays from
  the edge inward rather than dropping everywhere at once.
- The relation edge set of a deceased holder is empty when the succession pass
  reads it. A debug assertion states it, and a test proves that the successor
  is the same whether or not the holder ever carried a relation edge.
- Two claimants resolve by the stated rule, and a test constructs the contest
  rather than hoping for it.
- A watcher reads who has ruled a faction, in order, back to the founding.
- The change happens at a barrier, and a test asserts that no unit in one tick
  sees two rulers.
- What the ruler is reaches a unit through the field it already reads. A test
  asserts that no read path runs from a unit to its faction to its ruler.
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
[^9]: ADR Registry, row 0079. `docs/adrs/REGISTRY.md`
[^10]: Findings register, FND-047. `docs/FINDINGS.md`
[^11]: Decisions register, DEC-003. `docs/DECISIONS.md`
[^12]: The character graph and inheritance, sections 2.2 and 9.8. `docs/research/reports/14-character-graph-and-inheritance.md`
[^13]: Decisions register, DEC-040. `docs/DECISIONS.md`
[^14]: Decisions register, DEC-005. `docs/DECISIONS.md`
[^15]: ADR Registry, row 0060. `docs/adrs/REGISTRY.md`
[^16]: Backlog item 0104. `docs/backlog/proposed/0104-carry-the-writ-of-a-ruler-in-the-influence-field.md`
[^17]: Decisions register, DEC-041. `docs/DECISIONS.md`
