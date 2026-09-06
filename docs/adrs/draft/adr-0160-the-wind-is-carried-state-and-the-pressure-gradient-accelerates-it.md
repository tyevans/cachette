# ADR-0160: The wind is carried state, and the pressure gradient accelerates it

## Context

The engine holds weather as water over a lattice of level 1 cells. A cell of
the lattice holds the water in the air above it and the water on its ground,
and a tile holds none.[^1] A spread pass hands each neighbour an equal share,
so the transfer has no direction at all.[^2]

**A measurement shows what that costs.** One storm raised at the strength
ceiling stays at the cell it was raised on for two ticks, halves each tick, and
is indistinguishable from a calm world within twenty ticks. The maximum never
travels. It flattens in place. The same measurement shows that the height and
the water share of a cell vary strongly across the world, so the terrain a
model would read has contrast even though the water field has none.[^3]

The project owner asked on 5 September 2026 for weather that a watcher can
follow across the map, and for a model of wind, heat and ground rather than a
direction bolted onto diffusion. A front must keep moving after the thing that
made it has eased.

**A directionless field cannot be repaired with a value.** The share a cell
hands a neighbour is a scalar, and no scalar makes a symmetric kernel carry a
maximum one way rather than another. So the engine needs a quantity it does not
hold: a direction and a speed for each cell.

Two forces pull against each other here.

**Derived is cheaper and cannot drift.** A quantity computed each tick from the
world needs no storage, enters no hash, and can never disagree with the world
it came from. The engine already prefers that shape: a level above the tiles is
a derived projection and never a second truth.[^4]

**Only state gives a front momentum.** A wind computed from the pressure of the
moment is the pressure of the moment. It changes when the pressure changes and
it stops when the pressure stops. A front that outruns the thing that raised it
needs the wind of the previous tick as an input, and that is state.

The project owner settled this on 5 September 2026 and chose momentum. This
record states the choice and the cost.

## Decision

### D1. The wind of a cell is simulated state

**The engine stores one wind for each level 1 cell, and the wind enters the
state hash.** The wind is a bounded integer vector in the fixed two-axis basis
of the lattice. It is not a floating point number, it is not an angle, and no
part of it is a fraction of anything.[^5]

The field folds into the hash in ascending cell index order, beside the two
water planes it moves. The order is fixed and the hash is order-sensitive, so a
reader does not have to prove that the order does not matter.[^6] [^7]

**The wind is state and the heat is not.** The heat of a cell is a function of
the mean height of that cell and of the share of it that holds open water. The
level 1 summary already carries both as exact integer totals, and the step
rebuilds that summary before the weather solve runs.[^1] [^4] So the heat needs
no storage and it cannot drift from the world. A solve that computes the heat
of every cell into a scratch buffer holds no state, because that buffer never
crosses a tick boundary and never enters the hash.

The alternative is a wind derived each tick from the pressure alone. It is
rejected because it is the pressure alone: a derived wind has no memory, so no
front outlives its cause, and the owner asked for the behaviour that memory
gives.

### D2. The pressure difference accelerates the wind and never sets it

**Each pass moves the wind of a cell toward what the pressure difference across
that cell asks for, by a bounded step.** It does not assign the asked-for wind.
The step is what makes the wind lag the pressure, and the lag is the whole
reason the engine carries the wind at all.

The pressure of a cell follows its heat. The difference across a cell is taken
against its neighbours in the fixed direction order, and the arithmetic goes
through the arithmetic module.[^5] [^6]

The step size is a value that no measurement has chosen, and the balance
register holds the row.[^8]

### D3. A drag term bleeds the momentum, and a ceiling bounds the speed

**Each pass takes a share of the speed of a cell away before the acceleration
is added.** That share is the drag. Without it a wind under a steady pressure
difference gains the same step every pass and never stops gaining, so the field
would never settle and a front would accelerate without limit.

**The speed of the wind of any cell never passes the ceiling.** A reviewer can
find a violation of that statement in one read, and a runaway is the failure
mode of every system that carries momentum.

The drag and the ceiling do different work, and the record states both because
one is not enough. The drag is the mechanism: it settles the wind where the
step and the share balance, and in a settled field the ceiling is never
reached. The ceiling is the guarantee, and it is not free-standing. **The
ceiling exists so that the transport can conserve.** A pass carries water at
most one cell, so the share a cell sends must stay below what it holds, and the
speed ceiling is what bounds that share.[^9] A ceiling chosen for tidiness
would be decoration. This one is load-bearing, and the record that governs the
transport says why.

Both values are rows of the balance register.[^8]

### D4. A pass reads the wind the previous pass left, and the pass count is fixed

**A cell reads the wind of its neighbours from a settled plane and writes a
separate plane.** It never reads a plane that the same pass is writing. A pass
that read a half-written field would give an answer that depends on which cells
a thread reached first, which is the way a parallel pass stops being
order-independent.[^7]

The pass writes only the cell it is computing, so a parallel pass writes
disjoint output and needs no atomic operation.[^7]

**The solve runs a fixed number of wind passes.** It reads no clock, it tests
no residual, and it takes no branch on what the field holds. The project made
this decision for the influence solve and this record takes the same one for
the same reason: a convergence test makes the pass count depend on the
arithmetic, and the reduction it invites makes the result depend on the thread
count.[^10] The pass count is a row of the balance register.[^8]

## Consequences

**The engine stores a value for each cell that it did not store before.** The
storage grows with the lattice and not with the world, and not with the number
of units. It costs the lattice whether the air holds water or not, because a
wind exists over dry ground. This is a shape and not a figure, and one blocker
governs every cost figure in this project.[^11] **The project owner chose this
cost for the behaviour it buys**, and a reader who wants the cheaper shape
should read D1 and take the argument up there, not treat the storage as an
oversight.

The wind is state, so a world that loads a saved wind and a world that
recomputes one are different worlds. Anything that writes the world must write
the wind.

**The heat of a cell does not change over time, so the wind settles.** Nothing
in the engine varies the heat with a season or with a time of day. Under a
fixed terrain the pressure is fixed, so the wind reaches a steady field and
stays there. The picture a watcher sees still moves, because the source of the
water is a keyed draw at scattered cells and scattered ticks, and a steady wind
carries those parcels across the map.[^3] A driving term that varies over time
would make the wind itself move, and this record does not add one.

**A wrong ceiling breaks conservation rather than looking untidy.** The
transport rule depends on the bound this record states, so a later change that
raises the ceiling without changing the transport share puts water where the
account cannot follow it.

The step gains work that runs whether anything is happening or not. That is the
same shape the weather solve already has, and the product record that asks for
a condition that costs nothing when nothing happens is no better served than
before.[^12]

## References

[^1]: ADR-0140, weather is a field over the level 1 cell lattice, decisions D1 and D3. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^2]: ADR-0141, a weather pass moves water and never scales it, decision D1. `docs/adrs/draft/adr-0141-a-weather-pass-moves-water-and-never-scales-it.md`
[^3]: Research report 26, the scale of the weather, sections 4, 5 and 7. `docs/research/reports/26-the-scale-of-the-weather.md`
[^4]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^8]: Balance register, the weather section. `docs/reference/balance.md`
[^9]: ADR-0161, water rides the wind, and every transfer is an exact integer move, decision D2. `docs/adrs/draft/adr-0161-water-rides-the-wind-and-every-transfer-is-an-exact-integer-move.md`
[^10]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: PRD-0004, the world has weather that a watcher can read, what it costs at the target scale. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
