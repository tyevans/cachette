# ADR-0088: A tile field is a generated base and a stored change

## Context

This engine simulates a hex world of tiles. The largest count in the project
is the tile count, so every choice about how a tile field is held is a choice
about the largest allocation the engine makes.

An earlier record fixed the default shape. A tile field is one contiguous
array with one element for each tile, and the engine stores a structure of
arrays.[^1] That shape is right for a field a system writes on every frame,
and it is what a contributor reaches for.

Two later records chose another shape for two fields, and neither said what
the choice was in general. The ground is a pure function of the seed and the
tile address, and the engine stores no ground.[^2] A tile stock is generated
the same way, and the engine stores only what somebody took from a tile.[^3]

So three shapes now exist in one engine, and nothing states the rule that
picks between them. A contributor who adds a tile field reads the first record
and writes a dense column. A reviewer who wants to refuse that has no written
constraint to hold up.

The need that makes this hard is a stated one. A developer who changes a seed
must see the new world at once, so building a world must not cost a pass over
every tile before the first frame.[^4] A dense column cannot meet that. The
column must be filled, and filling it is the pass.

The second force pulls the other way. A generated field costs arithmetic on
every read, and a reader that sweeps the world pays that price again on every
sweep. The ground record accepted that trade for a field nothing writes.[^2]
A field that a system writes looks like it cannot take the same trade, because
a generated value has nowhere to put the change.

The tile stock record is the one that shows it can. It generates the stock and
stores what was taken, and the two combine on a read.[^3] That is a general
shape, and it was recorded as a fact about a stock rather than as a shape.

## Decision

### D1. A tile field is a generated base and a stored change, unless it cannot be

A tile field has two parts. The first is a pure function of the world seed and
the tile index. The engine stores none of it. The second is what the systems
have changed on that tile, and the engine stores one entry for each tile that
holds a change. A read combines the two.

Three properties follow, and a reviewer can check each one.

- Building a world visits no tile of the field and allocates nothing for it.
  The seed and the extent are the whole of a new field.
- The stored part grows with what the systems have changed, never with the
  size of the world alone.
- Two worlds built from one seed hold one field, without comparing anything.

**A field cannot take this shape when its base is not a function of the seed
and the index.** A field whose initial value depends on where a settlement
was placed, or on what a previous frame decided, has no base to generate. Such
a field is a dense column under the earlier record, and this record does not
change that.[^1]

The alternative is the dense column for every field. It is rejected because
the engine would then pay the largest allocation it makes for a field before
anything reads a tile of it, and the product need forbids that.[^4] The
alternative is also a second declaration of a value the seed already fixes,
and this project has recorded what a second declaration site costs when
nothing fails as the copies drift.[^5]

The alternative of generating with no stored part is rejected for a field a
system writes. There would be nowhere to put the change.

### D2. The stored part is held in tile order, and a change is merged as an ascending run

The stored entries are sorted by tile index. A lookup is a binary search over
them, so the answer does not depend on the order in which the changes arrived.

A frame adds its changes as one ascending run, merged in one pass. It does not
insert one entry at a time, because inserting into the middle of an array
moves every later entry, and a frame at the target scale touches many tiles.

**A parallel frame sorts its joined run by tile index. It never relies on the
order the output slots joined in.** Each worker reads one contiguous range of
tiles and writes to its own slot, so the ranges are disjoint and each tile
appears in one run at most.[^6] The sort is what makes the merged field the
same at any thread count, and the sort key is the tile index, which is stable
and unique.[^7]

### D3. The whole-world hash covers the value of every tile, not the seed and the stored part

The hash writes the combined value of each tile, in ascending tile order.

Hashing the seed, the extent and the stored part is not the same thing. Those
are the inputs of the generator. A change to the generator moves every tile of
every world and leaves all three untouched, so a hash over them would not
report the change.[^8] The ground record already states this for the ground,
and the argument is the same one.[^2]

The hash therefore visits every tile. That is correct and it is not a
contradiction of D1. D1 governs what the engine stores and what building a
world costs. It does not promise that every reader is cheap, and a hash of the
whole world is a reader of the whole world.

### D4. Every part of the field is exact integer or fixed-point arithmetic, through the arithmetic module

The generated base, the stored change and their sum are all exact.[^9] Every
operation on them goes through the arithmetic module.[^10]

The reason is narrower than the general ban. A read of this field is an
addition performed on every read, rather than once when a column was filled.
An addition that was not associative would make the value of a tile depend on
how many times a reader had asked for it.

The base is a keyed draw. The key is the tuple the record requires: the system
identifier of the tile field, a frame, an entity and a draw index.[^11] The
frame slot holds a constant, because the base does not change with time. The
entity slot holds the tile index.

**The tile index must reach the key.** A key without it gives one value to
every tile. Both determinism tests pass over that world, at every thread count
and on every run, because the defect repeats exactly. Only a test of the key
itself finds it.[^12]

## Consequences

**A whole-column read is a copy, and its name must say so.** The engine holds
no array to lend, so a caller that wants the whole field receives a new one.
The call site declares what copies.[^13]

**A reader that sweeps the field every frame is a design mistake, and this
record makes it visible as one.** The ground record says the same of the
ground.[^2] The two fields now share the property and share the warning.

**The first level of the pyramid is such a reader.** It sums the value of
every tile of every block, so it sweeps the field at every barrier and once
when the world is built. This record does not resolve that. An item holds
it.[^14]

**A read costs a draw and a search.** Neither grows with the number of tiles
that have never changed. The search does grow with the number that have, and
the ground record accepted the same growth for the stored take.[^3]

**Nothing states the cost.** No measurement exists on the target platform, so
this record states the shape of the growth and no figure.[^15]

**This record narrows the dense column record without superseding it.** Three
tile fields now sit outside it, and the earlier record still describes every
field that this one excludes. Whether that record should be superseded to say
so is an open choice, and a register row holds it.[^16] A reader who meets the
disagreement is served by this record, which resolves it.

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena, decision D2. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^3]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^4]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^5]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: ADR-0009, parallel stages write disjoint outputs. `docs/adrs/REGISTRY.md`
[^7]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^12]: Testing rules, section 2. `.claude/rules/testing.md`
[^13]: ADR Registry, row 0044. `docs/adrs/REGISTRY.md`
[^14]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
[^15]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^16]: Decisions register, DEC-068. `docs/DECISIONS.md`
