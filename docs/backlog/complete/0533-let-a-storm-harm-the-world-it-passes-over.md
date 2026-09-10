---
id: 0533
title: Let a storm harm the world it passes over
status: complete
created: 2026-09-09
implements: [ADR-0203 D1, ADR-0203 D2, ADR-0203 D3, ADR-0203 D4, ADR-0203 D5, ADR-0203 D6, ADR-0001 D1, ADR-0002 D1, ADR-0003 D1, ADR-0004 D1, ADR-0006 D1, ADR-0072 D5, ADR-0140 D3]
changes: []
creates: [ADR-0203]
serves: [PRD-0048]
blocked-by: []
---

## Why

**A storm cost a player nothing.** It put a pressure deficit on the cells it
reached, drove the wind and rained. A faction saw the picture change and lost
no building, no food and no people. A hazard that takes nothing is scenery.

The project owner asked for four effects. An upgrade wears. Units caught in
the open take losses. The food a tile carries is flattened and the ground is
soaked. A site can be destroyed outright.

The owner also settled the rule for the fourth effect. **Destruction is the
end state of wear and not a separate roll.** A raw severity threshold discards
a player's investment with no counter-play. Driving the condition to nothing
gives the counter-play back, because a site kept in repair survives a storm
that breaks a neglected one.

## Impact review

**Governed by.** ADR-0001 D1 requires one answer at any thread count. ADR-0002
D1 forbids a floating point number in simulated state. ADR-0003 D1 requires
every draw to be keyed on the system, the frame, the entity and the draw
index. ADR-0004 D1 requires an explicit iteration order. ADR-0006 D1 requires
an event type to be plain data with declared padding. ADR-0072 D5 requires the
world to record where a quantity went when it leaves. ADR-0140 D3 fixes the
order in which a pass reads the weather field.

**Creates.** ADR-0203, whose row the registry holds.[^1] It states that a storm
harms the world only through the sinks that already exist, and that it destroys
a site only by driving its condition to nothing.

**Changes.** No record changes. ADR-0203 extends ADR-0140, which states what
the weather field is and states no rule for what it takes from the world.
Every decision of ADR-0140 stands.

**Blockers.** BLK-130 governs every harm rate, and it is open. BLK-007 governs
the cost of the pass, and no measurement on the target platform holds it.[^2]
Each rate is a named constant beside the rule that reads it, and each has a row
in the balance register that names the blocker.[^3]

**Precedent.** The findings register records that a draw keyed on the wrong
field is repeatable and wrong, and that both determinism tests pass against
it. The loss draw therefore needs a test for each field of its key.[^4]

**Relation to item 0476.** That item is the other weather harm: what a flooded
cell costs a store, production, a unit and movement. It reads the water on the
ground. This item reads the pressure deficit.[^5] The two are separate
readings of one subsystem, and neither replaces the other.

## Done when

- A storm takes condition from a finished upgrade through the pass that owns
  wear, and no second site removes an upgrade.
- A site falls only when the wear takes the last of its condition, and the
  collapse names the storm as the cause.
- A site a worker keeps in repair outlives one a storm finds neglected.
- A storm takes units that stand on bare ground, and takes none that stand on
  ground carrying a finished upgrade.
- A storm flattens the food the tiles under it carry, and leaves the food of a
  tile it does not reach alone.
- The world still balances after a storm flattens food.
- The ground under a storm rises further than the ground beside it, and no
  pass adds drops of its own.
- Each field of the loss draw key has a test that changes the field and
  watches the draw change.
- One test is proven able to fail, by putting the defect back and watching it
  go red.

## Outcome

Every statement above holds. The pass sits beside the fire pass in the step,
and the upgrade harm sits in the pass that already owned wear.

**The conservation check found a real defect.** The first version wrote the
flattened food into the depletion ledger and named no destination for it, so
the account of the world stopped balancing on the first storm. The flattened
food now goes to the total that records what left the world, which is the
total a dead unit's load already went to.

**The food share alone flattened nothing.** Most tiles carry a few whole units
of food, and a share of a few units truncates to zero. A floor of one whole
unit answers it, which is the answer the ground drying pass already gives to
the same shape. The balance register holds the row.

**One neighbouring fixture needed repair.** A test that waited three thousand
ticks for a worker to finish a terrace now replaces a worker that a storm
takes. The commit body names it.

**The golden state file needs recording again.** This changes simulated state.

Registers moved: the balance register gained four rows, and the registry
gained the row for ADR-0203. No blocker opened or closed. No finding was
recorded, because nothing the project believed turned out false.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: Blockers register, BLK-007 and BLK-130. `docs/BLOCKERS.md`
[^3]: Balance register, the weather rows. `docs/reference/balance.md`
[^4]: Findings register, the keyed draw. `docs/FINDINGS.md`
[^5]: Backlog item 0476. `docs/backlog/proposed/0476-let-weather-harm-upgrades-stores-production-units-and-movement.md`
