# ADR-0162: Water enters the air where it is hot, and it falls where the air cools

## Context

The engine holds weather as water over a lattice of level 1 cells. Water enters
the air by one rule and leaves it by another. A cell of open water lifts a
fixed quantity when a keyed draw says so, and the odds follow the share of that
cell that holds open water. A share of the air then falls on every cell in
every solve, and the share rises with the mean height of the cell. A share of
the ground water then leaves the world into a running total.[^1]

**A measurement shows that this saturates the world.** Over a thousand ticks of
the demonstration world, with no storm raised at all, every cell of the lattice
holds ground water above the wet mark at every tick. The driest cell holds
nearly twice the mark and the middle cell holds more than four times it. So the
one reader that a simulation pass takes from the weather answers the same
everywhere, and the rule that gives a gatherer more on wet ground gives every
gatherer more.[^2] [^3]

**The ground water itself is not flat.** It runs nearly fourfold from the
driest cell to the wettest. The value that hides that range is the mark, and
the mark is a value rather than a decision.

Two companion records give the lattice a wind and make the water ride it.[^4]
[^5] They move water. They do not say where it enters or where it leaves, and
the present rules for those two answer without reading anything the model now
holds. A source that ignores the heat and a fall that ignores the air
temperature would put a directional transport on top of a field that fills
itself uniformly, and the picture would move without ever separating.

The project owner asked on 5 September 2026 for rain that arrives rather than
sitting everywhere, and for coasts and mountains to get their own weather
without anyone scripting either.

## Decision

### D1. Evaporation reads the heat of the cell, and it lifts from wet ground as well as from open water

**A cell lifts water into its air at a rate that rises with its heat, from the
open water it holds and from the water on its ground.** A hot cell lifts more
than a cold one. Water lifted from the ground leaves the ground plane and
arrives in the air plane of the same cell, so it moves within the account
rather than entering it.

The heat of a cell is a function of the mean height of that cell and of the
share of it that holds open water, and nothing stores it.[^4] So a low coast is
hot, a high ridge is cold, and no generator and no caller has to place either.

**A lift from open water still raises the running total**, because that water
was never in the account before. A lift from the ground does not, because it
was. The account of the transport record therefore still balances, and the
invariant check still reports it.[^5]

The alternative is the present rule, which lifts a fixed quantity from open
water alone at odds that read only the water share. It is rejected because it
makes the source of every drop the sea, so an inland cell can only ever receive
what the transport brings it, and the ground can never give anything back.

### D2. Rain falls where the air cools

**The share of the air that falls on a cell rises with the cooling that the air
meets there.** Air that arrives at a cell colder than the cell it came from
drops more of what it carries. Air over cold ground drops more than air over
warm ground.

This one rule gives two behaviours that nobody writes down. Air that crosses
from warm water onto a cold ridge cools and rains, so the near side of high
ground is wet. It arrives at the far side carrying less, so the far side is
dry.

The present rule reads the mean height alone.[^1] It is a special case of this
one and it is rejected as the whole rule, because height without the direction
of travel cannot tell one side of a ridge from the other.

Every quantity in D1 and D2 is a value that no measurement has chosen, and the
balance register holds a row for each.[^6]

### D3. A storm is an injection, and the gate of ADR-0142 is unchanged

**The divine power puts water into the air of the cells it names, and the model
then carries that water away.** The verb means the same thing it meant before:
a god names places and a strength, the engine turns the strength into a
quantity, and the quantity enters the air of each named cell.

**The gate stands.** A god may put weather on a cell only when its own faction
holds at least one tile inside that cell, and the whole reasoning for that gate
is untouched by how the water moves afterwards.[^7] Nothing in this record, and
nothing in the two companion records, changes what a god may reach.

**What changes is what a storm does after it lands.** Under the present rules
it sinks where it fell. Under the model it enters a system that carries it, so
a god raises a storm that then travels, rains where the air cools, and reaches
ground the god could not have named. That is a larger power in effect and the
same power in reach, and the record states it plainly rather than leaving a
reader to discover it.

## Consequences

**Weather stops being uniform, and the reader that a simulation pass takes
starts to mean something.** The rule that gives a gatherer more on wet ground
becomes a difference between places rather than a constant.[^3] A game balanced
against the constant will change.

**Setting the wet mark is still separate work, and it is not made unnecessary
by this record.** The measurement shows that the mark alone hides a fourfold
range in the ground water that exists today.[^2] A model that separates the map
further does not repair a mark that is far below every value it reads. Both are
values, and the balance register holds them.[^6]

**Water now leaves the ground into the air as well as out of the world.** So
the ground of a hot cell dries faster than the ground of a cold one, and a
place can be dried by its own heat rather than only by time. The account gains
one more path a drop can take, and the check that reads the account is the only
thing that would notice if that path lost one.[^5]

**The engine reads the heat of a cell more than once in a solve.** The heat
feeds the pressure that drives the wind, the evaporation of D1 and the fall of
D2. That is one derived value with three readers, and a later change that
stored it in one of the three would put a second declaration of it in the
tree.[^8]

**The cost follows the lattice.** Every rule here is one pass over the cells,
at a fixed pass count, reading a summary the step already rebuilt. It does not
follow the tile count and it does not follow the number of units. That is a
shape and not a figure, and one blocker governs every cost figure in this
project.[^9]

**Nothing here varies with time.** The heat of a cell is a function of terrain,
and terrain does not move. So the pattern that these rules produce is driven by
where the water enters and by the wind, not by a season or a time of day. A
driving term that varied over time would be a separate decision, and this
record does not make it.

## References

[^1]: ADR-0141, a weather pass moves water and never scales it, decisions D1 and D2. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^2]: Research report 26, the scale of the weather, sections 2, 3 and 6. `docs/research/reports/26-the-scale-of-the-weather.md`
[^3]: ADR-0143, wet ground yields more to a gatherer, decisions D1 and D2. `docs/adrs/draft/adr-0143-wet-ground-yields-more-to-a-gatherer.md`
[^4]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decisions D1 and D2. `docs/adrs/draft/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^5]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decisions D1 and D3. `docs/adrs/draft/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^6]: Balance register, the weather section. `docs/reference/balance.md`
[^7]: ADR-0142, a god inflicts weather only on ground its own faction holds, decisions D1 and D2. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
[^8]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
