# ADR-0172: A unit reaches its tile by a second field at tile pitch, seeded only where a destination lies

## Context

Movement in this engine takes its direction from a field over cells and never
from a search by a unit. A record holds that decision. A unit that holds an
option reads one entry of its own cell, steps to the neighbouring tile in that
direction, and reads no neighbouring cell and scores no neighbour.[^1] The whole
cost shape of movement rests on it: the cost follows the cell count and never
the population.[^2]

**A unit sent to a named tile could not reliably arrive at it.** The destination
field holds one direction for each cell, and a cell covers a block of tiles. A
unit followed the gradient until it entered the cell holding its target, and the
field then had nothing left to say. The reach of a seed cell is zero, so the
step down returned no exit and the unit fell back to the keyed draw that a
record already gives it for that case.[^3] It then walked at random inside the
cell until something else stopped it.

**This was measured and not read.** A probe reported the cell of the target, the
tick a unit entered that cell, the ticks it spent inside, and the tick it first
stood on the target tile. A unit sent twenty tiles walked seventy-one steps,
entered the target cell early, spent sixty-one ticks inside it, and never stood
on the target tile at all.

**Three verbs name a tile and not a cell.** The settle verb founds a city on the
tile the unit stands on. The build verb works the tile the unit stands on. A
delivery reads the tile the unit stands on. A movement rule that arrives within
a block therefore serves none of them.

Three forces pull against each other.

**A per-unit search is the thing this project refuses.** Any rule that computes
from the address of a unit toward the address of a seed is a search, however
short, and the record forbids it by name.[^1]

**A field at tile pitch over the whole world costs the world.** The lattice of
cells is smaller than the tile lattice by the square of the block edge, and that
ratio is why a field is affordable at all.

**A unit must read one entry.** That is the property every cost argument in the
movement design rests on, and a rule that made a unit read two things would
weaken it whatever the second thing was.

## Decision

### D1. A second field at tile pitch answers the last block, and a unit still reads one entry

**The engine derives a second field, at the pitch of a tile, over the tiles of
each block that holds a destination seed and over no other block.** A unit reads
one entry of it, keyed on the tile it stands on.

**A unit reads no neighbouring tile and scores no neighbour.** The record that
forbids a per-unit search is unchanged, and this record does not weaken it.[^1]
There is still one field derived for a set, and never a path for each unit.

**The field is seeded only where a destination lies.** A send names a small set
of target tiles, so the blocks that hold one are few, and the storage follows
the seeded blocks and never the world. A block with no seed holds no entry.

**A tile the field does not reach holds no entry, and the unit there reads the
coarse field**, which is the answer it read before this field existed. The two
fields compose without a rule that names either one.

### D2. A unit standing on its destination reads an arrival and takes no step

**The field carries one value that is not a direction, at an index one past the
last direction index.** A unit that reads it stands on the tile it was sent to,
and it takes no step.

This is the other half of the defect. Without it an arrived unit reads no
direction, falls back to the keyed draw, and walks off the tile it had just
reached. **An arrival must be a value the field can hold, and not the absence of
one**, because the absence already means something else: it means the field does
not reach here.

### D3. The relaxation runs a fixed count derived from the block edge, and no pass tests whether it settled

**The pass count is twice the block edge.** It reads no clock, tests no residual
and takes no branch on what the field holds.[^4] The project made the same
decision for every other field it solves, and for the same reason: a convergence
test makes the pass count depend on the arithmetic, and the reduction it invites
makes the result depend on the thread count.

**Twice the edge is the count at which a detour of the block edge again is
admitted.** A straight run across a block costs the edge. A route around ground
the plane cannot cross costs more, and the second edge is what buys it. This is
a derivation and not a budget: it follows from the shape of a block and not from
a measurement.

**The relaxation routes around ground the plane cannot cross**, so a unit inside
the last block does not walk into water. That is the property that a rule
computing straight toward the seed would lose.

### D4. A seed is stored as a tile and the cell is derived

**The field stores the tiles it was seeded at, and derives the blocks from
them.** It does not store both. A stored block beside a stored tile would be one
fact in two places, with nothing that fails when the two disagree.[^5]

## The alternatives this rejects

**A local rule inside the destination cell.** A unit inside the last block steps
toward the seed by comparing its own address with the address of the seed. It is
cheaper and it needs no second field. **It is refused because it computes from
the address of a unit toward a seed, which is the shape the movement record
forbids by name.**[^1] It also scores no ground, so it walks a unit into water
inside the block.

**A tolerance.** A send is satisfied when a unit reaches the cell of its target.
This is right for a verb whose work is cell-scale, and wrong for every verb that
names a tile. Three verbs name a tile, so a tolerance would leave all three
unable to complete.

**A path for each unit inside the last block.** The block is small, so the
search is short. It is refused because a short search is still a search, and the
cost follows the population rather than the seeded blocks. The registry already
retired a number that specified a path structure before anything needed one.[^6]

## Consequences

**The engine derives two fields for one send.** The coarse field crosses the
world and the fine field crosses the last block. A reader that asks which field
answered a unit gets one answer, because the fine field answers where it reaches
and the coarse one answers everywhere else.

**The storage follows the seeded blocks.** A send at one tile seeds one block. A
send at a scattered set seeds one block for each, and a set scattered across the
world costs a block for each destination. No measurement of it exists on the
target platform.[^7]

**A unit can now be sent to a tile, so a verb may require one.** The settle verb
does. Before this record no verb could rely on a unit standing anywhere in
particular, so any verb that named a tile would have failed for a reason its own
caller could not see.

**The keyed draw stays the rule for a unit the fields do not answer.** A unit
outside every seeded block and outside the reach of the coarse field takes the
draw, and that path is unchanged.[^3]

**The arrival value bounds the direction index space.** A field entry now
carries one more value than there are directions, so a reader that assumed the
entry ranges over the directions alone is wrong.

## References

[^1]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^5]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^6]: ADR Registry, the retired numbers. `docs/adrs/REGISTRY.md`
[^7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
