---
id: 0422
title: Let a god take the people of another god
status: complete
created: 2026-09-03
implements: [ADR-0132 D1, ADR-0132 D2, ADR-0132 D3, ADR-0132 D4, ADR-0132 D5, ADR-0133 D1, ADR-0133 D2, ADR-0133 D3, ADR-0133 D4, ADR-0133 D5, ADR-0133 D6, ADR-0134 D1, ADR-0134 D2, ADR-0134 D3]
changes: []
creates: [ADR-0132, ADR-0133, ADR-0134]
serves: [PRD-0035, PRD-0036]
blocked-by: []
---

## Why

The project owner asked for conversion, and decided the central question. A
downstream game needs a god that converts people. Asked whether conversion
changes the faction of a unit outright, or whether allegiance is a separate
value a unit carries, the owner answered that it changes the faction outright.

Nothing in the engine moved a unit from one faction to another. A unit took its
faction when it was created and kept it until it died. A developer who wanted
the effect had to end the unit and create a new one, which breaks the identity,
loses what the unit carried, and reports a death and a birth where a conversion
happened.

Two product records hold the need. One is the change itself. One is the
observation of it.[^1] [^2]

## Impact review

**Governed by.** ADR-0001 D1 and D3 require one answer at any thread count and
forbid a convergence test; the pass marks in parallel, sorts on the slot, and
runs a fixed amount of work. ADR-0002 D1 and D2 forbid a floating point number
and route every arithmetic step through the arithmetic module; the count that
converts is exact integer arithmetic on a 128-bit intermediate. ADR-0003 D1
keys every draw; the pass owns a system identifier that no other pass shares,
and it takes two draws for each group on a tile. ADR-0004 D1 fixes the
iteration order; the marks are sorted on the arena slot before anything is
applied. ADR-0006 D1 and D2 make an event plain data delivered at the barrier.
ADR-0009 D1 makes each thread write its own output. ADR-0014 D1 and D2 make an
identity survive; a convert keeps its slot and its generation. ADR-0018 D2 and
D3 govern the derived unit structure and the barrier. ADR-0053 D1 and D2 fix
what a faction is. ADR-0065 D1 makes a cohort the units of one faction at one
site, so the table is derived again after every conversion. ADR-0070 D1 makes
the per-faction population a maintained count rather than a walk. ADR-0072 D5
holds the conservation equality that a load must not break. ADR-0087 D1 fixes
the influence solve. ADR-0111 D2 and D4 fix where the presence relation is
folded and how it refuses a stale read.

**Changes.** No record changes. Nothing here contradicts one.

**Creates.** Three records, and each states a claim a future contributor could
reasonably choose otherwise on.

- ADR-0132 says conversion changes the faction and adds no second allegiance,
  and it says what a convert keeps and loses. A contributor could reasonably
  add an allegiance value, and could reasonably strip a convert of its home.
- ADR-0133 says a unit converts to the faction that leads the influence field
  at its cell, that strict dominance alone stops a flip loop, and that
  conversion is not gated on territory. A contributor will reach for a
  cooldown, and will reach for the presence gate that the trade verbs use.
- ADR-0134 says a god reads conversion as an event log and as the counts it
  already reads. A contributor will reach for a running total of conversions.

Each passes the three conditions of the scope rule.[^3] Choosing otherwise
costs more than changing it later in every case, because each answer shapes
what a game built on the engine can be. And the reasoning is not visible in the
code: nothing in the pass says why there is no cooldown column.

**Blockers.** Two opened. BLK-122 holds what belief costs a god, and the engine
charges nothing. BLK-123 holds what a convert does with a seat and a home at a
site of its old faction, and the engine keeps both. Neither stops work.

**Product records.** PRD-0035 and PRD-0036, and this item answers both.[^1]
[^2]

**Precedent.** The findings register holds two entries this work produced.[^4]
[^5]

## What was built

A pass in the step reads the influence field at the level 1 cell that covers
each occupied tile, finds the faction that leads there, and converts the units
of every other faction in proportion to the margin. It takes one keyed draw for
each group on a tile and never one for each unit. A verb converts a named set
outright, all or nothing. Both apply through one function, which writes the
faction, clears the orders, moves the character, emits the event and derives
the cohort table again.

The unit arena gained a faction setter that moves its own maintained count for
each faction and raises the revision, so the presence relation refuses a stale
read rather than answering one.

A log reports one event for each unit that changed hands, and the verb and the
log both cross the Python boundary.

## Done when

- A unit changes faction and keeps its identity. **Done.**
- The maintained per-faction count agrees, and the arena check passes. **Done.**
- The presence relation stops calling a convert a foreigner. **Done.**
- Each field of the draw key reaches the draw. **Done.**
- A converted unit does not flip back while the field stands still. **Done.**
- The result does not change with the thread count. **Done.**
- Each rule was broken on purpose and the test that covers it went red. **The
  review names each one.**
- The whole check command runs green. **The review states each gate and its
  output.**

## Outcome

The work is in the review.[^6] The review states what triggers conversion, how
every per-faction total stays correct, what stops a flip loop, what a god reads
to see it happen, the defects that were put back, and every gate.

## References

[^1]: PRD-0035, a god takes the people of another god. `docs/product/shaped/prd-0035-a-god-takes-the-people-of-another-god.md`
[^2]: PRD-0036, a god sees where it is winning people. `docs/product/shaped/prd-0036-a-god-sees-where-it-is-winning-people.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^4]: Findings register, FND-433. `docs/FINDINGS.md`
[^5]: Findings register, FND-434. `docs/FINDINGS.md`
[^6]: Review of item 0422. `docs/reviews/0422-conversion.md`
