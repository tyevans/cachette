---
id: 0496
title: Let a faction set the weight it gives each option, and let the controller favour its own ground
status: proposed
created: 2026-09-05
implements: [ADR-0156 D1, ADR-0156 D2, ADR-0156 D3, ADR-0156 D4, ADR-0156 D5, ADR-0156 D7]
changes: []
creates: []
serves: [PRD-0009, PRD-0006, PRD-0056]
blocked-by: [BLK-050]
---

## Why

**A worker wanders outside the ground its own faction holds, and the game then
makes no sense.** The project owner said so on 5 September 2026. A unit should
care about improving its own faction's land. When road planning lands, some
units should take road work as it is needed.

The choice pass cannot express that today. A unit scores a small fixed set of
options and takes the highest. Each score is one multiplication of how much the
unit wants a thing by how much of that thing is near, and how much is near
comes from the level 1 cell the unit stands in.[^1] No option reads the holder
of a tile, and no summary field of a cell says whose ground it is.[^2] The
engine also holds one weight set for every unit alive, so no faction can differ
from another in what its units want.

**The owner then corrected the shape.** The built-in controllers should work
this way as far as they can, and a learner will reach its own ideas. A
preference written into the pass would be physics, and a learner that plays one
faction could never choose otherwise.[^3]

This item gives each faction its own weight set, adds an own-ground term to
that set, and writes the set through one verb that a caller, the built-in
controller and a learner all use. **The built-in controller sets weights that
favour the ground its own faction holds, and it draws the strength of the
preference from the weight vector the seeding already draws for it.** A learner
may set the opposite weights, and that is the game played correctly. A record
states the whole rule.[^4]

**This item touches `fn step`. Only one worker may hold it at a time.** It
waits for the pass that holds `fn step` before it to merge.

## What is missing before this can be refined

- **What the three weights should be.** The own-ground weight, the
  rival-ground weight and the unheld-ground weight are all unset. The balance
  register holds the three rows, and one blocker governs them, because the
  rules of the downstream game are not written down.[^5] [^6] The work must
  express each as a parameter and state the shape of a first provisional value
  with its derivation, rather than invent a number.
- **Whether a laden unit returning home should read the term at all.** A unit
  that holds a home site and carries at least the carry mark is laden, and it
  takes the option that sends it home.[^7] That option is ranked by the return
  field and not by a cell summary field, and the ground under the unit on the
  way home is often ground nobody holds. A term applied to it would slow a
  delivery for a reason nobody asked for. The record does not say whether the
  term applies to every option or only to the options that rank the ground.
- **Whether a unit already outside its own ground should be pulled back, or
  should merely stop preferring to stay.** The two are different rules. A pull
  needs a direction toward held ground, which is a field, and the return field
  gives the direction to the nearest site rather than to the nearest held
  tile.[^8] A weight that only lowers a distant option leaves the unit where it
  is until something else moves it. The owner's words do not settle which one
  he asked for.
- **How this meets a campaign that deliberately marches onto another faction's
  ground.** A faction at war raises a campaign, and the campaign sends a cohort
  onto ground the other faction holds.[^9] A rival-ground weight low enough to
  keep a worker home would also weaken every option the campaign wants. The
  work must say whether a campaign unit reads a different weight set, whether
  the campaign order overrides the choice, or whether one weight set serves
  both.
- **Whether the weight set belongs in the observation a learner reads.** A
  learner acts through a bounded action table and reads a bounded observation,
  and the engine declares both.[^3] A learner that writes its own weights and
  cannot read them back is acting without state. A learner that reads another
  faction's weights reads something that faction never showed it, and the
  observation rule refuses that.[^10] The work must decide which of the two, and
  the record does not.
- **What the census should report, so that the effect is visible rather than
  silent.** A watcher must be able to see that units stay on their own ground.
  The engine counts what its passes do behind one feature switch. The work must
  say which counts it adds: the units standing on ground their own faction
  holds, the units standing on ground another faction holds, and the choices
  the own-ground term changed. Without a count, the change is a behaviour
  nobody can measure.

## Done when

Filled in when the item moves to `refined/`.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: The choice pass of the core crate. `crates/cachette-core/src/choose.rs`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D2 and D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: ADR-0156, a faction's option weights are policy, set through one verb. `docs/adrs/draft/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^5]: Balance register, the choice. `docs/reference/balance.md`
[^6]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^7]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
[^8]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^9]: Backlog item 0478, let a faction raise a campaign against a faction at war. `docs/backlog/complete/0478-let-a-faction-raise-a-campaign-against-a-faction-at-war.md`
[^10]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
