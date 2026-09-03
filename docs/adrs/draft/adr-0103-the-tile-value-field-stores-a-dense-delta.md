# ADR-0103: The tile value field stores a dense delta, never a sparse change list

## Context

The tile value field gives every tile of the world a value. The world holds
more tiles than a project can afford to store carelessly, and the scale
constants table states how many.[^1]

An accepted decision says that a tile field is a generated base and a stored
change.[^2] The base is drawn from the seed and the tile index, so it costs
nothing to hold. The stored part held only the tiles that a frame had changed.
That decision earns a real property: building a world visits no tile and
allocates nothing for the field, which is what a product record asks of the
build.[^3] A test holds the engine to it.[^4]

The stored part was a list of pairs, sorted by tile. A frame produced a run of
changes, and the field merged the run into the list. The merge walked the list
and the run together and wrote both into a second buffer.

**Two properties of that shape were never stated together.** The merge rebuilds
the whole list on every call, so its cost follows the length of the list. And
the list only grows, because an entry is added when a tile first changes and is
never removed.

**A measurement of the target extent found how fast it grows.** The list reaches
almost every tile within ten frames, which is about one second of simulated
time. The findings register holds the table.[^5] After that point the list holds
an entry for nearly every tile, and the merge rewrites nearly every entry on
every frame in order to apply a few.

The stage table measured what that costs as a share of a frame, and the pass
takes no thread count.[^6]

**A sparse structure that saturates is not a sparse structure.** It is a dense
array with an index column beside it and a second buffer behind it. It is
larger than a dense array, because it carries the tile index that a dense array
does not need, and because the merge needs somewhere to build. It is slower,
because the dense array is written where it is read and the list is not.

The open question about spending memory to save time does not apply here.[^7]
That question weighs one resource against the other. This choice does not: the
sparse form is the more expensive one on both axes once it saturates.

## Decision

### D1. The stored part of the tile value field is one delta for every tile

The field holds an array of deltas in tile order. Reading a tile is one index.
Applying a run of changes writes one entry for each change and reads nothing
else.

**The cost of applying a run follows the run.** It does not follow the number
of tiles that have changed before, and it does not follow the size of the
world.

No entry carries a tile index, because the position in the array is the tile.

### D2. The array is allocated at the first change, and never when the world is built

A world that has changed no tile holds no array. Every tile of such a world
reads a delta of zero without touching memory that does not exist.

This keeps the property the generated-base decision earns.[^2] Building a world
still visits no tile and allocates nothing for this field, and the test that
holds the engine to it still passes unchanged.[^4]

### D3. The result of applying a run does not depend on the order of the run

Each change writes its own tile and reads no other tile, so two runs that hold
the same changes in different orders leave the same array.

The caller still passes an ascending run that names each tile once, and the
field asserts it. A repeated tile would be added twice, which states something
the caller did not mean. **The assertion guards the caller's meaning, and no
longer guards the structure**, because the structure no longer depends on the
order.

This is a weaker requirement than the sorted list made, and a weaker
requirement is one fewer way for a parallel stage to become
order-dependent.[^8]

### D4. This decision governs the tile value field, and no other field

**It does not say that every tile field is dense.** The generated-base decision
still holds wherever its premise holds, and its premise is that the stored part
stays small.[^2]

The measurement that overturns the premise is a measurement of one field.[^5]
A field whose changed set stays a small share of the world is still better
sparse, and the upgrade field is the candidate that looks that way today.

A future change to another field must measure that field. **The saturation of
one field is not evidence about another**, and this record is not a licence to
convert them.

## Consequences

**The field costs the extent once any tile has changed.** A world that runs a
frame allocates one delta for every tile, whether the frame changed one tile or
all of them. The budget table holds the figure.[^1]

**The count of changed tiles changes meaning.** It counted entries and now
counts tiles whose delta is not zero. A tile whose changes cancel back to zero
now leaves the count, which the sorted list could not express. The count is
maintained as the array is written, so it is a second place that one fact
lives, and the invariant check derives it from the array and fails when the two
disagree.[^9]

**A reader of a tile range no longer carries a cursor.** The sorted list needed
one, so that a walk over a range cost one pass rather than a binary search for
each tile. A dense slice needs neither.

**The engine can now write the array from the pass that produces the changes.**
The stage that produces them already partitions the world into contiguous tile
ranges and gives each worker its own, so the workers write disjoint parts of
one array and need no join.[^8] This record does not make that change. It makes
it available.

## Alternatives rejected

**Keep the sorted list and merge in place.** Applying a run without rebuilding
the list would avoid the second buffer. It is rejected because inserting into
the middle of a sorted vector moves every later entry, and the changes are
scattered across the world rather than clustered at the end.

**Keep the sorted list and promote it to dense at a threshold.** The field would
start sparse and change shape when the entries passed some share of the tiles.
It is rejected because it holds two representations and a rule for moving
between them, so every reader must handle both and the threshold is a value
that nothing measures. The lazy allocation in D2 gives the part of that idea
that is worth having, which is that an unchanged world costs nothing.

**Store the value rather than the delta.** The field would hold the whole value
of every tile and the generator would go. It is rejected because the generated
base is what makes a world exist without a pass over it, and D2 keeps that.
The base also lets an unchanged tile cost no memory traffic on a read that
misses the array entirely.

**Leave it, and parallelise the merge.** The merge could partition by tile
range. It is rejected because it treats the symptom: the work is rewriting an
array to change a few entries of it, and dividing that work over threads still
does it.

## References

[^1]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^2]: ADR-0088, a tile field is a generated base and a stored change. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^3]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^4]: The proof that building a world visits no tile. `crates/cachette-core/tests/build_visits_no_tile.rs`
[^5]: Findings register, FND-292. `docs/FINDINGS.md`
[^6]: Target platform costs, the stage table. `docs/reference/graviton-costs.md`
[^7]: Decisions register, DEC-105. `docs/DECISIONS.md`
[^8]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^9]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
