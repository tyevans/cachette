---
id: 0470
title: Expose the economy tuning knobs to Python
status: refined
created: 2026-09-03
implements: [ADR-0040 D1, ADR-0043 D1, ADR-0046 D1, ADR-0085 D3, ADR-0107 D2, ADR-0002 D1, ADR-0062 D1, ADR-0062 D4]
changes: []
creates: []
serves: [PRD-0047]
blocked-by: []
---

## Why

The engine holds every number that governs its economy. The control plane
reaches almost none of them. A developer who wants a settlement to earn more,
owe more, hold more, or return a worked deposit faster has to fork the engine.

That is the outcome the design principle "unit types and upgrades are data,
not code" exists to prevent. The project owner asked whether a downstream game
can be defined largely from Python. These knobs are most of that answer.

The work is a boundary and nothing else. It adds no engine behaviour, no
storage and no pass.

## Impact review

**Governed by.**

- ADR-0040 D1. The boundary carries an instruction and an answer, never the
  population. Each write that names places takes the whole set and answers
  once.
- ADR-0043 D1. The tier of a shape decides the shape of its interface. A
  settlement and a unit are both written in sets. A world-wide value is one
  write for the world.
- ADR-0046 D1. Every refusal the engine makes raises a typed error under the
  one root class.
- ADR-0085 D3. Every identity resolves against its generation before anything
  is written.
- ADR-0107 D2. Every word of prose lives in the Rust doc comment. The type
  stub gains signatures and no prose.
- ADR-0002 D1. Every rate and every quantity crosses as an integer. No
  floating point number crosses in either direction.
- ADR-0062 D1 and D4. A rate belongs to a site, and the stored rate is what
  one tick earns. The doc comment must say that, because a caller who reads it
  as what one application earns writes a defect that nothing catches.
- ADR-0001 D4. One binary gives one answer at any thread count. A value that a
  later frame reads must reach the state hash.

**Changes.** No record. The work adds no decision and contradicts none.

**Creates.** No record. Every value already has a decision behind it.

**Blockers.** None govern the work. BLK-050 holds the wider list of what a
named downstream game needs, and this item answers the economic part of it.

**Precedent.**

- Item 0341 bound the build verbs. It is the closest precedent for the shape
  of a set-valued write that resolves every identity before it writes.
- Recurring defect shape 1 forbids a second declaration site for one value.
  Three of the thirteen names in scope already cross to Python inside an
  existing report, so this item must not publish them a second time.
- Recurring defect shape 3 says a capability that nothing invokes ships inert.
  Each test therefore starts at the Python boundary and asserts that a later
  step changed.

## Done when

- A caller sets production, upkeep, the economy cadence, a settlement store,
  the recovery rules, the deed threshold, the home site of a set of units, and
  the influence source of a set of places, all from Python.
- A caller reads back the deed threshold, the recovery rules and the influence
  of a faction at a place.
- Every doc comment states when the value may be set, its unit and its scale.
- A test steps the world after each write and asserts that the simulation
  changed. A knob that could not be proved this way is named in the report.
- A refused value raises a typed error and leaves the world unchanged.
- Each rule that the tests hold is broken once, and the test fails.
- The determinism tests pass at 1, 2 and 12 threads.
- Every gate passes.
