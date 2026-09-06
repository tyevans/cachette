# ADR-0166: The temperature of a cell is carried state that a season and the sky drive

## Context

The engine holds weather as water over a lattice of level 1 cells. Three
records give that lattice a model. One makes the wind carried state that a
pressure difference accelerates.[^1] One makes the water ride the wind by an
exact integer move.[^2] One makes water enter the air where it is hot and fall
where the air cools.[^3]

**All three read a heat, and none of them lets that heat change.** The first
states that the heat of a cell is a function of the mean height of the cell and
of the share of it that holds open water, and that nothing stores it.[^1] The
third states plainly that nothing in it varies with time, and that a driving
term that varied over time would be a separate decision.[^3] This record is
that decision.

The project owner asked on 6 September 2026 for a temperature that varies over
time, in the same way that the water on the ground varies over time.

**A heat fixed by terrain gives a wind that settles and never moves again.**
The terrain does not move, so the pressure is fixed, so the wind reaches a
steady field and stays there. The first record says so in its own
consequences.[^1] The picture still moves, because the source of the water is a
keyed draw at scattered cells, but the pattern that carries it does not. A
watcher who follows a front sees one fixed set of lanes.

Three forces pull against each other here.

**A field that varies only by terrain is a static field.** A temperature
derived from the height of a cell answers the same at every tick. It costs
nothing, it cannot drift, and it buys nothing that the height did not already
give.

**A field driven only by a clock is not weather either.** A temperature that is
a function of the tick and the terrain is still derived. It varies, but every
cell varies in step, so no part of the map can be warm while another part is
cold for a reason that the map produced.

**Only state gives a temperature its own history.** A parcel of warm air that
the wind carries onto a cold ridge has to remember that it was warm. That
memory is the whole reason the pattern separates, and memory is state.

The project owner asked for a small model rather than a climate model. This
record therefore names the fewest drivers that give a field which varies over
both space and time, and it refuses the rest.

## Decision

### D1. The temperature of a cell is simulated state, and it lags its driver

**The engine stores one temperature for each level 1 cell, and the temperature
enters the state hash.** It is a bounded whole number of degrees on a scale
this project owns. It is not a floating point number and no part of it is a
fraction of anything.[^4] It folds into the hash in ascending cell index order,
beside the wind and the two water planes. The order is fixed and the hash is
order-sensitive.[^5] [^6]

**Each pass moves the stored temperature a share of the way toward the
temperature that the world asks for. It never assigns that temperature.** The
share is what makes the field lag its driver, and the lag is the whole reason
the engine carries the temperature rather than deriving it.

**This changes ADR-0160 D1**, which states that the wind is state and the heat
is not. Every other decision of that record stands unchanged, and the wind
stays state for the reason that record gives.

The alternative is the derived heat that the three companion records assume. It
is rejected because a derived heat has no memory, so no warm parcel survives
the cell it left, and the field cannot separate one part of the map from
another.

### D2. Four terms drive the temperature, and one of them is a cycle

**The temperature that the world asks of a cell is the sum of four terms.**

1. **The mean height of the cell takes degrees away.** High ground is cold.
2. **The share of the cell that holds open water adds degrees.** Water is warm,
   and it is the source of the water that the air carries.
3. **A season cycle adds or takes degrees by the tick.** The cycle is a
   triangle over a fixed period. It is a whole-number function of the tick, it
   reads no clock, and it takes no draw.
4. **The water in the air over the cell takes degrees away, to a bound.** Cloud
   stands between the ground and the sun.

**The first two are the heat that ADR-0160 already names.** This record keeps
them and adds the other two. Terms 1 and 2 give a field that varies over space.
Term 3 gives a field that varies over time. **Term 4 is what makes the two
interact.** The air over a cell is the output of the transport, which the wind
drives, which the temperature drives. So a cell that the weather reached is
colder than a cell it missed, and that difference is not a function of the
terrain and the tick.

The alternative is the cycle alone. It is rejected because every cell then
varies in step, and a field that varies in step is one number with a map
painted on it.

Every quantity in this decision is a value that no measurement has chosen, and
the balance register holds a row for each.[^7]

### D3. The wind carries the temperature, and the carry cannot run away

**A cell takes, from each neighbour, a share of the difference between the two
temperatures, and that share rises with the part of that neighbour's wind that
points toward the cell.** So a wind off warm water warms the ground it reaches,
and a wind off a cold ridge cools it.

**The shares of the six neighbours sum to less than one whole.** So the new
temperature of a cell lies inside the range of the temperatures the pass read,
and the field cannot run away at any wind and over any number of passes. The
speed ceiling of the wind is what bounds each share.[^1]

**The pass reads a settled plane and writes a separate plane.** It never reads
a plane that the same pass is writing, so the answer does not depend on which
cells a thread reached first.[^6] It writes only the cell it is computing, so a
parallel pass writes disjoint output and needs no atomic operation.[^6]

The temperature is not water and this record does not make it conserve. It is
an intensive value: two cells at one temperature give one temperature, not two.
So the exact-move rule of the transport record does not apply, and this record
does not claim it.[^2]

## Consequences

**The engine stores a second value for each cell that it did not store
before.** The storage grows with the lattice and not with the world, and not
with the number of units. It costs the lattice whether the air holds water or
not. This is a shape and not a figure, and one blocker governs every cost
figure in this project.[^8]

**The wind no longer settles.** The pressure difference that drives it now
varies with the season and with the cloud, so the wind field moves, and a front
that a watcher follows takes a different lane at a different part of the cycle.
That is the behaviour the project owner asked for, and it is what a fixed heat
cannot give at any value.

**Three readers of the heat become four.** The pressure that drives the wind,
the evaporation, the fall and now the carry all read the temperature. The
companion record already warns that this is one value with several readers, and
that storing it in one of them would put a second declaration in the tree.[^3]
This record stores it in exactly one place, and every reader takes it from
there.

**A world that loads a saved temperature and a world that recomputes one are
different worlds.** Anything that writes the world must write the temperature,
as it must write the wind.

**The season is global, and this record does not give the world a latitude.**
Every cell takes the same season offset at one tick. The difference between two
cells at one tick comes from the terrain, from the cloud and from the wind, not
from where they lie north and south. A latitude would be a separate decision,
and this record does not make it.

**The temperature is not conserved, so no account reports a defect in it.** The
water account is the only reader that sees a lost drop, and it says nothing
about a lost degree.[^2] A test of the carry has to state its own bound, and
this record states it in D3 so that a reviewer can find a violation.

## References

[^1]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^2]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decisions D1 and D3. `docs/adrs/accepted/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^3]: ADR-0162, water enters the air where it is hot, and it falls where the air cools, decisions D1 and D2. `docs/adrs/accepted/adr-0162-water-enters-the-air-where-it-is-hot-and-falls-where-the-air-cools.md`
[^4]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^5]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^6]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^7]: Balance register, the weather section. `docs/reference/balance.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
