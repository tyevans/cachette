---
id: 0050
title: Close the gaps the product shaping opened
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0011, PRD-0013, PRD-0014, PRD-0015, PRD-0017]
blocked-by: []
---

## Why

Six product records were shaped in one session: a founding condition,
consumption, housing, family, a ruler, and work assignment. Shaping them
against the eleven records that existed found four places where two needs
collide and one place where an existing record now reads wrong.

None of these is a defect in code. Each one is a question that a later
architectural decision will have to answer, and answering it early is cheaper
than discovering it inside an implementation. This item records them so the
answers are taken deliberately.

The finding register holds the shape that produced two of them.[^1]

## What this item holds

**One. A unit's need is declared in two records.** PRD-0011 states that a unit
needs something to continue and that failing to get it has a visible
consequence. PRD-0013 states the same need and shapes it across six sections.
Two records now declare one need, and nothing fails when they disagree. Decide
which one owns it. The likely answer is that PRD-0011 keeps the death rule and
PRD-0013 keeps the draw, but that is a decision somebody must make and record.

**Two. Job assignment belonged to nobody.** PRD-0011 defers choosing a job to
unit behaviour. PRD-0009 is unit behaviour and it excludes a group decision
and any goal that outlives a tick, so it refuses the work. PRD-0017 now owns
the assignment. PRD-0011 and PRD-0009 both still point the reader at each
other, and one of them needs an amendment.

**Three. Consumption needs a store, and the store needs a home. Closed.** The
question was whether a unit draws from what it carries or from a shared store.
Item 0056 answered it and is complete: a unit draws against the store of the
site it belongs to, and the draw runs by cohort rather than unit by unit.[^2]
A decision record holds the constraint.[^3] Neither product record needs an
amendment, because neither one claimed the answer.

This point needs no further work. It stays here so that a reader who finds the
question again reads the answer instead of asking it a second time.

**Four. Two records both limit population growth.** PRD-0011 says the
population responds to what a faction has. PRD-0014 says growth slows when
there is nowhere to live. Two independent limits on one quantity produce a
result that depends on which one runs first. Decide which limit is the limit,
or decide how they compose exactly.

**Five. A dwelling has no owner, and a family may need one. Closed.** The
question was whether the household of PRD-0015 and the place to live of
PRD-0014 are the same thing. They are not, and the relation between them is
now stated: a dwelling is stored and a household is derived from it.[^4] A
unit carries the slot of the dwelling it lives in, and a household is every
unit that carries one slot. Nothing stores a household and nothing declares
one.

Three things follow. A household forms when people share a roof and dissolves
when they stop. An inheritance is a transfer of a slot, so PRD-0014 keeps its
bound and PRD-0015 keeps its bound. A household needs no kinship rule, because
it is a fact about a place rather than a fact about a family.

PRD-0015 said that units who are related and who live together form a
household. That reading is now wrong, and the record was corrected. The work
that derives a household is filed.[^5]

**Six. PRD-0012 changes what an existing record assumed.** Several shaped
records reason about a world that already holds its target population.
PRD-0012 makes the population start small and grow, so the early ticks of a
run now have a different shape from the ticks a cost statement was written
against. No cost statement is wrong, because each one states a shape and not a
number. Confirm that, rather than assume it.

## What is missing before this is refined

Two of the six points are closed above, and point two has an owner. Four
points remain, and the item stays in `proposed/` until they are worked out.

- The impact review. No decision record has been read against points one,
  two, four and six, so the item cannot name which records govern them.
- The split. Four questions in one item is still more than one piece of work.
- The order. Point one and point four both concern PRD-0011, and the
  dependency between them is not worked out.

## Done when

- Each of the six points above has an owner: a product record that states the
  need, or a decision record that states the constraint, or a closed row that
  says the question does not arise.
- No two product records declare one need without a stated owner.
- No product record defers work to a record that excludes it.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-057. `docs/FINDINGS.md`
[^2]: Backlog item 0056. `docs/backlog/complete/0056-draw-consumption-from-a-site-store-by-cohort.md`
[^3]: ADR Registry, row 0063. `docs/adrs/REGISTRY.md`
[^4]: Decisions register, DEC-039. `docs/DECISIONS.md`
[^5]: Backlog item 0103. `docs/backlog/proposed/0103-derive-a-household-from-the-dwelling-slot.md`
