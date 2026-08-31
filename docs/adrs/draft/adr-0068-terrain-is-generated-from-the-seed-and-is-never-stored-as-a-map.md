# ADR-0068: Terrain is generated from the seed and is never stored as a map

Status: Draft

## Context

The world needs ground. Every tile must have a kind and a height, and every
later subsystem asks the ground a question: a territory asks where a border
can run, a deposit asks what a tile is made of, a route asks what it must go
around.

The project must decide where the ground lives. There are two shapes, and the
choice reaches every subsystem that reads a tile.

The first shape stores the ground. Generation runs once, writes a column for
each field, and every later read is a load. This is what the tile storage
record already provides for, because a tile field is a dense column and the
tile count never changes.[^1]

The second shape computes the ground. Nothing is stored except the seed and
the extent, and a read runs the generator. The cost moves from memory to
arithmetic, and it moves from world creation to every read.

The target tile count is the largest count in the project. A column for each
ground field is therefore one of the largest allocations the engine makes. The
record that bans floating point makes the second shape harder than it looks:
the usual generator is a lattice of noise interpolated with a smooth curve,
and every published form of that is written in floating point.[^2] Float
addition is not associative, so a field built from it is not a function of its
inputs alone once anything sums or reorders.

Terrain is also the first field in this project that is a pure function of an
address rather than a consequence of a step. That property is what makes the
choice available at all, and it is not available for a field that a system
writes.

## Decision

### D1. The ground is a pure function of the world seed and the tile address, and the engine stores no ground

The engine holds the seed and the extent. It holds no array of tile kinds and
no array of heights. A caller that wants a tile calls a function with an
address, and receives the tile.

The function reads the seed and the address. It reads nothing else. It reads
no tick, no neighbour, no previously computed tile, and no mutable state.

Three properties follow, and each is a thing a reviewer can check.

- Two worlds built from one seed have one ground, without comparing anything.
- The ground cannot go stale, because there is no stored copy to disagree with
the generator.
- The ground is the same at any thread count and in any visit order, because
the order of the reads is not an input.[^3]

The alternative is to store the ground in dense columns. It is rejected for
two reasons. The saving is arithmetic, and the cost is the largest allocation
in the engine. The engine pays that cost whether or not anything reads a tile.
A stored ground is also a second declaration of a value that the seed already
fixes. This project has recorded what a second declaration site costs when
nothing fails as the copies drift apart.[^4]

The alternative is not rejected forever. A cache of generated tiles is a later
decision, and it is a decision about a cache, not about where the ground
lives. This record binds the source of truth, so a cache that disagreed with
the generator would be a defect rather than a variant.

### D2. Every step of the generation is exact integer or fixed-point arithmetic, through the arithmetic module

The lattice values, the interpolation weights, the smooth curve, the octave
weights and the thresholds are all exact.[^5] Every operation on them goes
through the arithmetic module.[^6]

This is a constraint and not a preference, because the usual generator is a
sum, and a sum in floating point depends on its order. The ban already forbids
the type. The reason it is restated here is narrower. A terrain generator is
the first place in this project where the obvious reference implementation is
a float one. A contributor who ports one reaches for a float literal whose
type is inferred.

Truncation is accepted. Fixed-point multiplication truncates towards negative
infinity, so a generated value differs from the value a real-number derivation
would give. That difference is fixed, reproducible and part of the definition
of the field. It is not an error to correct.

Two consequences of exactness are load-bearing. The field has no denormal
behaviour near zero, so the same threshold gives the same kind on every
machine. The field also does not depend on how a compiler contracts a multiply
and an add. That contraction differs between the target and the development
machines.

### D3. The lattice draws from the counter-based generator, keyed on the lattice node

Every lattice value is a draw. The key is the tuple the record requires: a
system identifier that the terrain generator owns alone, a frame, an entity,
and a draw index.[^7]

The generator binds the four slots as follows. The frame slot holds a
constant, because the ground does not change with time. The entity slot holds
the lattice node address, with both components in it. The draw index holds the
field and the octave, so no two fields and no two octaves share a key.

**Both components of the node address must reach the key.** A key that carries
one component gives a field that varies along one axis and is constant along
the other. Both determinism tests pass over that world, at every thread count
and on every run, because the defect repeats exactly. Only a test of the key
itself finds it, and the testing rule requires one test for each field of the
key.[^8]

The alternative is a table of permutation values, which is what a classical
gradient noise uses. It is rejected because a table is state, a table must be
seeded, and a table that two worlds share makes two worlds correlate. The
counter-based generator already gives a keyed value with no state, and the
project has one of those.[^7]

### D4. The engine says what a tile is and never what a tile costs

The generator gives a tile a kind, a height, and whether a unit may stand on
it. It does not give a tile a movement multiplier, a defence value, or a
yield.

Passability is here because it is a property of the ground: water admits no
foot soldier whatever the game rules say. A crossing cost is not a property of
the ground. It is a property of a rule about the ground. An open choice
governs where that rule lives, and this record must not settle it by
accident.[^9]

## Consequences

**The ground costs nothing to hold and something to read.** A whole-world
sweep of the ground is arithmetic rather than a linear scan of memory. A
subsystem that sweeps the ground every frame is therefore a design mistake
under this record, and the record makes it visible as one.

**A field that a system writes cannot live here.** The moment anything changes
the ground, the ground stops being a function of the seed. Erosion, a crater,
a cleared forest and a built road are each a stored field. Each sits beside
the generated ground, and each needs its own record. This record does not
forbid them. It states that they are not terrain.

**The whole-world hash must cover the generated tiles, not the seed.** The
ground is part of the world, and the world is hashed each frame against a
stored file.[^10] Hashing the seed and the extent does not meet that
requirement. Those are the inputs of the generator. A change to a threshold or
to the octave count moves every tile of every world and leaves both inputs
untouched, so a hash over the inputs would not move either.

**Changing the generator changes every world, and a test says so.** There is
no stored world to migrate and no file to keep. A change to a threshold or to
an octave count gives every existing seed a different world, and the golden
file then fails. That failure is the intended report, and the person who
changes the generator re-records the file. A project that ships saved games
will need a generator version as well, and that is a new record.

**A viewer draws the ground by asking for it.** The viewer reads what the
screen shows and no more, which is what the viewer record requires of every
read it makes.[^11]

**A cache becomes a real option and a real risk.** The generated ground is the
source of truth, so a cache is free to exist and is required to agree. A cache
that is allowed to disagree is the failure this project has already recorded
twice, in a different form.[^4]

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: Report 02, the hex grid and the level of detail pyramid. `docs/research/reports/02-hex-grid-and-lod-pyramid.md`
[^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^8]: Testing rules, section 2. `.claude/rules/testing.md`
[^9]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^10]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^11]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
