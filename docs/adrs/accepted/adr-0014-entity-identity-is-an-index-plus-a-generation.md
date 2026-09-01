# ADR-0014: Entity identity is an index plus a generation

## Context

A unit is an entity in a generational arena.[^1] Something must name an entity
so that one structure can refer to an entity that another structure owns.

Several structures hold a reference to an entity across the frame barrier. A
command buffer records an order against an entity in one frame and applies it
in the next. The bridge that maps a tile to the units on it is rebuilt from
references it captured earlier.[^2] Between the capture and the use, the
entity can die.

The engine therefore needs a name that survives the death of the thing it
names, and that reports the death rather than hiding it. A name that silently
resolves to a different entity is the failure this record prevents. That
failure is deterministic, so both determinism tests reproduce it, but it
reaches the wrong entity in every run.[^3]

A slot in the arena is reused. Reuse is what makes the name hard. The arena
must be dense, so it cannot leave a dead slot empty forever.

## Decision

### D1. An entity identity is a slot index and a generation, held as one value

The slot index names a row in the entity location table. The generation counts
how many times that slot has been used.

The identity is one opaque value. The engine never stores the index and the
generation as two separate fields that a caller can set apart.

The entity storage is the only thing that builds an identity. Every other
caller receives one, reads its parts through accessors, and passes it on. A
caller that assembles an identity from an index it chose has defeated the
generation, because the index it chose came from somewhere that could not
know whether the slot still holds the same entity.

### D2. Resolving an identity can fail, and the caller must handle the failure

Resolution compares the generation in the identity against the generation in
the location table. A mismatch means the entity is dead. A dead identity
resolves to nothing.

The engine never treats a mismatch as an error to report and then continue
past. A caller either handles the absent entity or skips it.

### D3. The generation advances when the engine frees a slot, not when it allocates one

The identity of a dead entity becomes invalid at the moment the entity dies. A
generation that advanced on allocation would leave every stale identity valid
until something else claimed the slot.

### D4. A freed slot returns to use in first-in first-out order

A freed slot enters a queue. The engine takes the oldest freed slot when it
allocates. It never takes the newest.

The reason is the generation range, not correctness. D3 already makes a
stale identity fail: the generation advances on the free, so the identity of
the dead entity no longer matches the slot, whichever slot the engine
allocates next. Last-in first-out reuse is safe by that argument alone.

Last-in first-out reuse is unsafe over time. It hands the same slot back at
once, so that slot takes every generation increment while other slots take
none. Its counter reaches the end of its range early, and the engine retires
the slot.[^8] First-in first-out reuse spreads the increments across the
whole freed set, so no slot wears out before the others.

A slot freed during a frame becomes available at the frame barrier and not
before. Structural change already batches at the barrier.[^4]

### D5. A slot whose generation cannot advance is retired

The generation has a finite range. When a slot reaches the end of that range,
the engine never returns the slot to the queue.

Retirement leaks one slot. Reuse after wraparound would make two different
entities share one identity, which is the exact failure this record exists to
prevent. The project trades one leaked slot, for each slot that exhausts its range, against the removal of the case.

### D6. A generation starts at one, never at zero

The first generation of every slot is one. No slot ever holds generation
zero.

The identity packs the generation and the slot index into one value, and that
value is never zero, so an absent identity costs no extra space.[^9] A slot
index of zero at a generation of zero packs to zero, which the identity
cannot hold. The first entity that the engine ever allocates takes slot zero,
so a generation that started at zero would leave that one entity without a
representable identity.

The failure is silent in the worst way. It appears once, at the first
allocation, and only for one slot. Every test that allocates a second entity
first would pass.

Starting the generation at one removes the case for every slot at once. The
project rejects the alternative, which is to forbid the allocator from ever
issuing slot zero: that wastes a slot, and it puts the rule in the allocator
where each future allocator must remember it, rather than in the identity
where it holds once.

Generation zero therefore means that a slot has never been used.

### D7. The location table is a dense array indexed by the slot

The engine finds an entity by subscripting the location table with the slot
index. It never looks an entity up in a hash map.

A hash map costs a hash on the hot path. It also introduces an iteration order
that no key fixes, and an unfixed iteration order is a determinism defect.[^5]

### The alternative this rejects

**A bare index with no generation.** This is smaller and faster, and it needs
no comparison on resolution. The project rejects it because a stale index
resolves successfully to whichever entity now holds the slot. Nothing fails,
and the wrong entity receives the order. The defect is silent, and it appears
only when a death and a reuse straddle a captured reference.

**A globally unique identifier and a hash map.** This never collides and needs
no retirement rule. The project rejects it because every resolution becomes a
hash lookup, resolution is on the hot path, and the map introduces an
iteration order that the project must then fix by hand.

## Consequences

**Identity is common to all four entity shapes.** An identity names an entity,
not a shape.[^6] The identity carries no shape tag. The location table holds
the shape, so a caller that needs the shape resolves the identity first.

**Every holder of an identity must resolve before it acts.** A structure that
holds an identity across the barrier cannot assume the entity still exists.
Resolution is a real branch on the hot path, and the project accepts it.

**The engine can never compact the slot index space.** Compaction would move
an entity to a different slot and invalidate every identity that names it.

The location table therefore holds one entry for each slot the arena has ever
opened. That count is the high water mark of the live population plus the
retired slots, and it never falls.

**A test can prove that a stale identity fails.** Free an entity, allocate
until the slot returns, and resolve the old identity. The result must be
absent. This test is cheap and it is the direct check of D2, D3 and D4.

**The retirement rule leaks, and a hostile workload can grow the leak.** A
workload that recycles one slot without pause retires it eventually, and then
retires its successor. The
project accepts this, because the alternative is a shared identity. A survey
of arena designs reaches the same reuse rule and the same retirement rule.[^7]

## References

[^1]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^2]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: ADR-0020, structural change batches at the barrier and applies by tombstone and compact. `docs/adrs/REGISTRY.md`
[^5]: ADR-0004, iteration order is explicit, and unordered reductions need slots. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^7]: Report 01, the entity component system core and the memory layout, section 4. `docs/research/reports/01-ecs-and-memory-layout.md`
[^8]: ADR-0014, decision D5, in this record.
[^9]: ADR-0011, every value type is a newtype with a declared size and alignment. `docs/adrs/REGISTRY.md`
