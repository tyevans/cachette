---
id: 0440
title: Build weather, and the power of a god to inflict it on a place
status: complete
created: 2026-09-03
implements: [ADR-0140 D1, ADR-0140 D2, ADR-0140 D3, ADR-0141 D1, ADR-0141 D2, ADR-0141 D3, ADR-0142 D1, ADR-0142 D2, ADR-0142 D3, ADR-0142 D4, ADR-0143 D1, ADR-0143 D2, ADR-0143 D3]
changes: []
creates: [ADR-0140, ADR-0141, ADR-0142, ADR-0143]
serves: [PRD-0004, PRD-0040]
blocked-by: []
---

## Why

The world holds terrain, and terrain does not move. Every condition a unit
meets is fixed when the generator runs, so no situation can arise that the
generator did not already place. An accepted product record states that
need.[^1] The project owner then asked for a second thing: a god must be able
to put weather on a place, because a god that only redirects its own units has
no act a bystander can see.

## Impact review

**Governed by.** ADR-0001 D4 requires that one binary give one answer at any
thread count, and that the whole state hash each frame. ADR-0002 D1 and D2
forbid a floating point number in simulated state and route the arithmetic
through one module. ADR-0003 D1 requires a keyed draw. ADR-0004 D1 fixes every
iteration order. ADR-0009 D1 requires disjoint parallel writes. ADR-0022 D1 and
D2 make level 0 the only truth and every level above it derived. ADR-0023 D1
requires an exact combine. ADR-0072 D5 is the pattern for a conserved quantity
with a running account. ADR-0073 D3 is where the gather rate is read. ADR-0087
D1 requires a fixed iteration count in a field solve. ADR-0121 D2 decides the
opposite granularity for a fight, and this work states why that does not carry
over.

**Changes.** None. No record is superseded.

**Creates.** ADR-0140, ADR-0141, ADR-0142 and ADR-0143. All four are drafts.

**Blockers.** BLK-007 governs every cost figure, so this work states shapes and
no numbers. BLK-130 is opened by this work, and it holds every quantity the
weather system carries.

**Precedent.** FND-402 records that a fight at level 1 granularity smears, and
this work states why weather does not. FND-051 and FND-048 record that a
uniform fixture hides a defect, and the effect test therefore uses a world that
holds no open water so that only a god can wet the ground.

## Done when

- The world holds a condition that varies over the map and over time, with no
  caller.
- The condition conserves exactly, and the world invariant check reports the
  account.
- The condition is bounded and never falls below zero.
- Terrain influences the condition.
- The condition influences a unit, and a test points at the difference.
- A god puts weather on a set of places in one all-or-nothing call, and a gate
  bounds the power.
- A watcher reads the condition through the published interface.
- Both determinism tests pass at one, two and twelve threads.
- The whole check command runs green.

## Outcome

Weather is two quantities of water over the level 1 cell lattice: what stands
in the air above a cell, and what has fallen onto the ground of that cell. Both
are whole numbers of drops.

The sea lifts water on its own, keyed on the frame and the cell. A fixed number
of spread passes moves the air between neighbouring cells by exact integer
transfer. High ground then takes more of the air onto the ground, and part of
the ground dries. Two running totals make the account exact, and the world
invariant check reads it.

Wet ground yields more to a gatherer. That is the one wiring. The three passes
that could have read weather and do not are recorded with a recommendation for
each.

A god puts weather on a set of places, and it may act only on a cell where its
own faction holds ground. The call is all or nothing, the strength has a
ceiling, the set has a ceiling, and a faction waits between storms.

**Changed from the plan.** Nothing about wind was built. A wind plane would
have been a third quantity that only the spread reads, so the isotropic spread
stayed and the choice is recorded.

**Registers.** FND-450 to FND-454 record five corrections. DEC-235 to DEC-237
close three choices and DEC-238 opens one. BLK-130 opens. The four record rows
are in the registry as drafts, and they are in the record priority index.

**The golden state hash moved on every scenario.** Weather is simulated state,
so the field enters the whole-world hash. The files were recorded again from
this source and verified at more than one thread count.

## References

[^1]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
