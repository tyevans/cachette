---
id: 0496
title: Let a faction set the weight it gives each option, and let the controller favour its own ground
status: complete
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

## Impact review

**Governed by.** ADR-0156 decisions D1, D2, D3, D4, D5, and D7 govern this work.
D1 requires each faction to have its own weight vector in simulated state,
hashed into the state hash. D2 adds the own-ground, rival-ground, and
unheld-ground weights to this vector. D3 uses the single verb
`set_faction_weights` for all actors. D4 requires the built-in controller to
set weights favouring own ground by drawing preference strength from the seed.
D5 maintains engine physics (fixed option set, scan order, score as a product,
weights never acting as a fence). D7 adds the unit faction to the choice key
so that answer tables cache per cell, faction, bucket, and carry class.

**Changes.** `FactionWeights` gains three `u8` fields: `own_ground`,
`rival_ground`, and `unheld_ground`. `CellAnswers` caches answers per faction
present in the cell. The Python bindings and stubs expose the three new fields.
`SUBSYSTEM_CENSUS` adds counts for units on own and rival ground.

**Creates.** None.

**Blockers.** BLK-050 governs the downstream balance values. The three weights
are expressed as balance parameters with provisional defaults.

**Precedent.** FND-569 records that the weight vector reader and setter must
expose every weight that the simulated state holds.

**Architectural review answers:**
1. **The three weights:** Expressed as `u8` parameters in `FactionWeights` within
   `WEIGHT_LOW..=WEIGHT_HIGH`. The built-in controller sets `own_ground > unheld_ground > rival_ground >= WEIGHT_LOW`
   using a keyed draw from the seed for the preference delta.
2. **Laden units returning home:** Options with `Ranked::Carry` (`deliver`) do
   not rank the ground and receive a neutral ground factor (`Fix32::ONE`).
   A laden unit returning home is never penalized for traversing unheld or
   rival ground.
3. **No pull field:** Per ADR-0156 D5, the own-ground term is a score multiplier
   and never a fence or pull field. Units on foreign ground simply score options
   on that ground lower.
4. **Campaign cohorts:** Campaign orders assign objectives directly; regular
   unit options use the policy weights without special-cased physics.
5. **Learner observation:** The learner reads its own faction's weights through
   `Reading.weights` in `faction_observation.rs`. It does not read rival weights.
6. **Census reporting:** `SUBSYSTEM_CENSUS` reports `units_on_own_ground` and
   `units_on_rival_ground`.

## Done when

- `FactionWeights` holds `own_ground`, `rival_ground`, and `unheld_ground`.
- The built-in controller seeds weights favouring own ground over unheld and rival ground.
- The choice pass scales `Ranked::Cell` options by the cell holder weight for the unit's faction.
- `Ranked::Carry` options remain neutral and bypass ground penalties.
- `CellAnswers` lazily evaluates and caches choices per `(faction, bucket, carry)`.
- `world.set_faction_weights` and Python bindings support the expanded vector.
- `SUBSYSTEM_CENSUS` reports `units_on_own_ground` and `units_on_rival_ground`.
- A dedicated test verifies faction-differentiated choices, including a test proven able to fail.
- All gates run clean.

## Outcome

Built. `FactionWeights` holds three new `u8` fields: `own_ground`, `rival_ground`,
and `unheld_ground`, preserving 8-byte layout, `Pod`, and `Zeroable`.
`WEIGHT_COUNT` is derived as `size_of::<FactionWeights>() as u32` (8).
The built-in controller seeds weights favouring own ground over unheld and rival
ground (`own_ground > unheld_ground > rival_ground >= WEIGHT_LOW`) using
counter-based PRNG draws 5..=7 on controller seeding.
In the choice pass, `CellAnswers` tracks and caches choices per
`(faction, bucket, carry)`. Options with `Ranked::Cell` scale by the cell
holder factor (`own_ground`, `rival_ground`, or `unheld_ground` mapped to `Fix32`
with neutral 128 as `Fix32::ONE`). Options with `Ranked::Carry` use
`ground_factor.max(Fix32::ONE)`, ensuring laden units returning home are never
penalized on foreign ground and win ties with roam options on own ground.
`set_faction_weights` and Python bindings expose the new weights with bound
validation (`1..=255`). `Reading.weights` exposes all eight weights to the
learner observation. `SUBSYSTEM_CENSUS` reports `units_on_own_ground` and
`units_on_rival_ground`.

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: The choice pass of the core crate. `crates/cachette-core/src/choose.rs`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D2 and D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: ADR-0156, a faction's option weights are policy, set through one verb. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^5]: Balance register, the choice. `docs/reference/balance.md`
[^6]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^7]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D3. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
[^8]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^9]: Backlog item 0478, let a faction raise a campaign against a faction at war. `docs/backlog/complete/0478-let-a-faction-raise-a-campaign-against-a-faction-at-war.md`
[^10]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^11]: Findings register, FND-569. `docs/FINDINGS.md`
