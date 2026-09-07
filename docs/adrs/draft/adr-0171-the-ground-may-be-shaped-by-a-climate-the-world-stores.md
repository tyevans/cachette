# ADR-0171: The ground may be shaped by a climate the world stores

## Context

The ground of this world is generated and never stored. An accepted record
holds that decision. It says the engine holds the seed and the extent, holds no
array of tile kinds and no array of heights, and answers a tile by calling a
function with an address. **It says that the function reads the seed and the
address, and that it reads nothing else.**[^1]

Three properties follow from that clause, and the record names all three: two
worlds built from one seed have one ground; the ground cannot go stale, because
there is no stored copy to disagree with the generator; and the ground is the
same at any thread count and in any visit order.[^1]

**The engine can now build a world whose ground reads a third input, and the
accepted clause is therefore false as an absolute statement.**

The reason is a request about how the world looks. A world generated from noise
alone puts a forest beside a desert with nothing between them, because nothing
in the generator knows which places are wet. The weather subsystem does know.
It carries water over a lattice of cells and it holds a temperature for each of
them.[^2] What the weather leaves over a place, averaged over a long enough
run, is a climate, and a climate is what decides whether a place holds forest.

Three forces pull against each other.

**The ground must not read the weather of the running world.** The weather
reads the ground: the heat of a cell rises with its open water share and falls
with its mean height.[^3] A ground that read the running weather would close
that loop, and a loop that runs twice builds a different world each time.

**A stored ground is the thing the accepted record exists to refuse.** Dense
columns of tile kinds are the largest allocation the engine would hold, and
they are a second declaration of a value the seed already fixes.[^4]

**A capability that nothing invokes is worse than no capability.** A record
that describes a shaped ground as how this engine builds a world would state an
intent as a fact, which is a defect shape this project already tracks.[^5]

## Decision

### D1. The ground is a pure function of the seed, the address and one stored climate field

**A world may hold a climate field, and the terrain function reads it.** This
changes the accepted clause that the function reads the seed and the address
and nothing else.[^1] It reads the seed, the address, and the climate of the
cell that covers the address.

**The stored field is small and it is not a map of tiles.** It holds one entry
for each weather cell, and the weather lattice is coarser than the tile lattice
by a wide margin at the default pitch.[^2] The refusal that the accepted record
makes is a refusal of a dense per-tile ground, and that refusal is unchanged.
No array of tile kinds and no array of heights exists.

**Two of the three properties the accepted record names survive unchanged.**
The ground cannot go stale, because the climate field is not a copy of the
ground and nothing derives the ground twice. The ground is the same at any
thread count and in any visit order, because the order of the reads is still
not an input.

**The first property gains one term.** Two worlds built from one seed have one
ground when they were built with the same climate. The climate is itself a pure
function of the seed, the extent, the weather pitch and the tick count of the
spin, so two worlds built from one seed by one constructor still have one
ground.

### D2. The climate is produced by running the weather over the base ground, once, before the world runs

**The engine runs the weather forward over an empty world for a fixed tick
count, accumulates the temperature and the standing water of each weather cell,
and stores the two totals.** The classification of a tile then reads those
totals.

**The spin reads the base ground alone.** The ground that the spin hands the
weather is folded from the terrain that the seed alone gives. The spin never
hands the weather a shaped ground. That is what breaks the loop, and it is the
whole reason this shape is admissible at all.

**The classification runs once, after the spin, and never feeds back.** There
is no second pass and no settling.

**The climate changes what a tile is made of and never how high it stands.**
The height and the open water share are the only two terrain fields the weather
reads, and the kinds the climate selects between are all passable, so the
folded ground is the same whether it comes from the base terrain or from the
shaped terrain. This is not a coincidence of the current kinds. It is the
condition under which D2 holds, and a kind that changed the height or the
passability would close the loop that D2 opens.

### D3. The spin runs a fixed tick count, and no test decides when it stops

**The spin walks a stated number of ticks and stops.** It runs no convergence
test and reads no time budget.[^6] It walks the ticks in ascending order and
the cells in ascending index order.[^7] The weather solve it drives is keyed on
the seed and the tick, holds no thread-local state, and gives one answer at any
thread count.[^8]

**The stored field enters the state hash.**[^9] It is stored, the step reads
it, and a stored value the step reads is in the hash.[^10]

### D4. The shaping is available and it is not the default

**A world built by the ordinary constructors holds a quiet climate field, and
its ground is the ground the seed alone gives.** A caller asks for a shaped
ground by name.

**This record states what the engine can do and not what the project intends to
do.** Nothing in the engine, in the bindings or in the control plane package
asks for a shaped ground today. A reader who takes this record for a
description of how worlds are built would be wrong, and the record says so
rather than leaving the reader to find out.

**The default is what protects the accepted record's guarantee in practice.**
Every world the engine builds today reads the seed and the address and nothing
else. This record widens what is permitted; it does not change what happens.

### D5. Four readers of the ground still read the base ground, and that is a known gap

**The terrain function is not the only reader of the ground, and the others
have not moved.** A reader that answers from the base ground while another
answers from the shaped ground is one fact in two places, with nothing that
fails when the two disagree.[^11]

This decision records the gap rather than hiding it. **The gap is a reason to
leave the shaping off by default, and closing it is a condition of turning it
on.** A backlog item holds the work.

## Consequences

**A world that asks for a shaped ground pays the spin at construction.** The
cost is a fixed number of weather solves over an empty world, and it falls on
world creation and never on a step. No measurement of it exists on the target
platform.[^12]

**A world that asks for a shaped ground hashes differently from one that does
not.** The climate field is in the hash, so the two diverge from the first
frame even at one seed.

**The project cannot let a kind that the climate selects change a height or a
passability.** Either would put the shaped ground back into what the weather
reads, and the loop that D2 breaks would close again.

**The accepted record keeps every other decision.** The ground is still
generated, the arithmetic is still exact, and the refusal of a dense stored
ground stands.[^4]

**This record must be revisited before the shaping becomes the default.** Two
things hold it back: the four readers of D5, and the absence of any caller. A
record that described the shaping as the default today would be false, and this
one is written so that turning it on is a change to one decision rather than a
rewrite.

## References

[^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^2]: ADR-0140, weather is a field over the weather cell lattice, decisions D1 and D4. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^3]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D1. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^4]: ADR-0068, terrain is generated from the seed and is never stored as a map, decisions D1 and D3. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^5]: Recurring defect shapes, shape 3. `.agents/rules/recurring-defects.md`
[^6]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^10]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^11]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^12]: Blockers register, BLK-007. `docs/BLOCKERS.md`
