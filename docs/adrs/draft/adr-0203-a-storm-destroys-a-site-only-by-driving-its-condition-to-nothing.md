# ADR-0203: A storm destroys a site only by driving its condition to nothing

## Context

This engine simulates a hex world. The weather over that world is a field on a
lattice of cells coarser than the tile field. A storm is a travelling low that
the field carries: it puts a pressure deficit on the cells it reaches, the
deficit drives the wind, and the deficit makes the air pour out its water as
rain.[^1]

**A storm cost a player nothing.** It moved water and changed nothing else. A
faction saw the picture change and lost no building, no food and no people. A
hazard that takes nothing is scenery, and a player who cannot lose to it cannot
plan against it.

The project owner asked for four effects: an upgrade wears, units caught in the
open take losses, the food a tile carries is flattened and the ground is
soaked, and a site can be destroyed outright.

Three forces made the fourth effect hard.

**A threshold destroys without counter-play.** The simple rule is that a storm
above some severity removes what stands under it. That rule discards a
player's investment on a draw the player cannot answer. A faction that spent
work on a wall and a faction that let the same wall rot lose the same wall.

**An upgrade already has a condition and a repair loop.** A site carries a
condition, an army takes condition from a hostile site, weather takes condition
from a wet one, a fire takes more, and a worker puts condition back. One pass
owns every removal, and the collapse event is the only record that anything
stood there. A second damage quantity beside that condition would be one fact
in two places, with nothing that fails when the two disagree.[^2]

**Water must not appear from nowhere.** A weather pass moves water and never
scales it, so what the ground gains, the air gives up.[^3] A damage pass that
added drops to the ground would put water into the world that no cell gave up,
and the account that every weather pass holds would stop balancing.

A further force applies to the whole of this work. The engine must give one
answer at any thread count, and a random draw must be keyed and never
stateful.[^4] [^5] A draw keyed on the wrong field is repeatable and wrong, and
this project has already shipped that defect once.[^6]

## Decision

### D1: A storm harms the world only through the sinks that already exist

A storm takes condition from an upgrade through the pass that owns wear. It
takes food from a tile through the ledger that records every take. It takes a
unit through the despawn that every other end of a unit goes through.

The storm module states the rate of each harm. Each pass applies the rate it
reads. No pass restates how a deficit becomes harm, and no pass holds a damage
quantity of its own.

### D2: A storm destroys a site by driving its condition to nothing, and by no other route

There is no severity at which a site falls whatever its condition. A site falls
when the wear pass takes the last of its condition, in the same way a site
falls to an army or to a fire.

A faction that keeps a site in repair therefore survives a storm that breaks a
neglected one. The repair loop is the counter-play for the whole of this
record.

### D3: A storm creates no water, and the ground under it wets through the rain

The deficit cuts the capacity of the air above a cell, so the air pours out
what it holds and the ground gains it. That is an exact move of water the
weather field already holds.

No pass adds drops to the ground of a cell in the name of damage.

### D4: Shelter is what a faction built

A unit that stands on ground carrying a finished upgrade is not in the open,
and a storm draws no loss for it. A unit on bare ground is in the open.

The upgrade map is the one statement of what stands on a tile, so it is the one
statement of what shelters.

### D5: A loss draw is keyed on the storm system, the frame and the whole unit identity

The storm damage pass owns a system identifier that no other system takes. Each
loss draw keys the frame into the frame slot and the whole identity of the unit
into the entity slot.

The identity carries a generation, so a unit spawned into the slot of a unit a
storm took draws its own answer. The frame is in the key, so a unit that stands
in one storm for two ticks draws twice.

A test must exist for each field of the key. Changing the field must change the
draw. A determinism test cannot see a draw that is repeatable and wrong.[^7]

### D6: The harm rates are parameters behind the blocker that governs them

Nobody has said what weather should be worth. Each rate is a named constant
beside the rule that reads it, and each has a row in the balance register that
names the blocker.[^8] [^9]

No rate here is a measurement, and no figure of this record states one.

## Consequences

**A storm now changes the state hash.** It removes units, it removes upgrades
and it writes the depletion ledger, so a run with a storm in it reaches a
different state from the same run without one. The golden state file must be
recorded again.

**A faction can lose a site to weather it did not cause.** A storm forms over a
warm sea without a caller, so a faction can lose ground to a thing no player
did. That is the point of the effect, and the repair loop is the answer to it.

**The project cannot add a severity threshold later without superseding this
record.** A rule that destroys a site at a stated severity contradicts D2.

**A shelter rule now reads the upgrade map.** A change to what an upgrade means
changes who is caught in the open. That is the price of one declaration site
rather than two.

**The pass costs the footprint of the storms at tile pitch.** The storm count
and the reach of a storm both have ceilings, so the cost is bounded by those
ceilings and never by the extent of the world. A world that carries no storm
walks nothing. **No measurement on the target platform holds this figure**, and
the blocker that says which cost figures are measured governs it.[^8]

**A storm that stands over ground nobody built on takes only food.** The
upgrade harm has nothing to act on and the unit harm has nobody to act on, so a
storm over wilderness is still scenery. That is correct, and it is worth
stating so that nobody reads a quiet storm as a defect.

## References

[^1]: ADR-0140, weather is a field over the level 1 cell lattice, decisions D1 and D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^2]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^3]: ADR-0141, a weather pass moves water and never scales it, decision D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^4]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^6]: Testing rules, section 2. `.agents/rules/testing.md`
[^7]: ADR-0014, entity identity is an index plus a generation, decision D6. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: Blockers register, BLK-007 and BLK-130. `docs/BLOCKERS.md`
[^9]: Balance register, the weather rows. `docs/reference/balance.md`
