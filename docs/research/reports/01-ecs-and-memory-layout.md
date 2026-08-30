# ECS Core, Memory Layout, and Cache-Aligned Data Structures

Research input for ADR-0001. Area: entity storage, tile storage, and the
bridge between them.

---

## Executive summary

I agree with the two-storage-regime decision in the context brief. Dense
struct-of-arrays for tiles and archetype-chunked storage for units is the
right split. Below are the refinements I recommend, in priority order.

1. **Chunks earn their place as a query granularity, not as a cache trick.**
   The brief has almost all units in one archetype. In that case chunking
   gives you little cache benefit over a plain contiguous column. It gives
   you a lot of value as a place to hang per-chunk metadata: a bounding box,
   a faction bitmask, a unit-type histogram, and a change tick. Selector
   descent then prunes whole chunks. Write the rationale this way in the ADR.

2. **Use per-chunk change ticks, not per-entity change ticks.** Bevy stores
   two ticks per component per entity. At 1M units and 10 components that is
   80 MB of tick writes per frame. Per-chunk ticks cost about 100 KB. Add a
   per-entity dirty bitplane only for the few fields that need it.

3. **Do not build a full-grid CSR occupancy index.** An offset array over
   16.7M tiles costs 64 MiB. That is more than the whole tile dataset.
   Instead, keep the unit array sorted by tile index and store one range per
   32x32 block. Cost: about 4 MiB. See "The tile-to-unit bridge".

4. **Store tile bitplanes and tile columns in block-tiled order**, not plain
   row-major. Use the same 32x32 block as the L1 aggregation block. Then one
   L1 aggregation step reads one contiguous span per field. Index arithmetic
   stays pure shift and mask, because 4096 and 32 are powers of two.

5. **Set the chunk size to 64 KiB, and allocate chunks from 2 MiB-aligned
   arenas.** 16 KiB is the Unity DOTS number, but Unity chose it for a
   different workload. Hardware prefetchers do not cross 4 KiB page
   boundaries without huge pages, and a 16 KiB chunk gives very short
   per-component runs. Make the size a compile-time constant so you can
   measure it.

6. **Entity ID: `NonMaxU32` index plus `u32` generation, packed in a
   `NonZeroU64`.** This gives `Option<Entity>` the same 8-byte size. Recycle
   free slots FIFO, not LIFO. Retire a slot when its generation overflows.

7. **Do not depend on `std::simd`.** It is still nightly-only. Write
   autovectorization-friendly loops over `chunks_exact`, and use the `wide`
   crate where you need explicit lanes on stable Rust.

8. **Write your own ECS. Do not take a dependency on `bevy_ecs`, `hecs`, or
   `legion`.** Your requirements are narrow and unusual. See "Crate survey".

I disagree with no decision in the brief. I refine three: the CSR bridge
(item 3), the 16 KiB chunk size (item 5), and the implicit assumption that
you need a general ECS at all (item 8 and "Open questions").

---

## 1. How real engines lay out archetype storage

### Unity DOTS

An `EntityArchetype` holds a list of `ArchetypeChunk` objects. Each chunk is
a 16 KiB buffer in unmanaged memory. A chunk holds one array per component
type, plus one array of entity IDs. The arrays are tightly packed. Entity 0
of the chunk sits at index 0 of every array. When an entity leaves a chunk,
the last entity moves into the gap. This is a swap-remove.
([Unity docs](https://docs.unity3d.com/Packages/com.unity.entities@1.0/manual/concepts-archetypes.html),
[ArchetypeChunk API](https://docs.unity3d.com/Packages/com.unity.entities@0.4/api/Unity.Entities.ArchetypeChunk.html))

Unity also puts "shared components" at chunk granularity. All entities in a
chunk share one value. This is how DOTS partitions entities without adding a
component column. Your L1 block ID is a natural shared component.

### Flecs

Flecs stores each archetype as a table of columns. It connects tables with an
**archetype graph**. Each table is a node. Each edge is one component add or
remove. To add a component, the engine follows the edge. If no edge exists,
it does a slower lookup and then creates the edge. Traversal gets faster over
time as edges accumulate.
([Flecs FAQ](https://www.flecs.dev/flecs/md_docs_2FAQ.html),
[Storage in pictures](https://ajmmertens.medium.com/building-an-ecs-storage-in-pictures-642b8bfd6e04))

Flecs reserves entity IDs below `FLECS_HI_COMPONENT_ID` (default 256) for
components. Edges for low IDs use direct array indexing. Edges for high IDs
use a hash map. The hash map path costs 5-10% of a structural change
operation. This is a good trick to copy: keep component IDs small and dense,
and index edges with an array.

Flecs also splits cached queries into an empty-table list and a non-empty
list. This stops query iteration from walking dead archetypes.
([Flecs Queries](https://www.flecs.dev/flecs/md_docs_2Queries.html))

### Bevy

`bevy_ecs` splits archetype metadata from component storage. An `Archetype`
records which components are present. A `Table` holds the data. A `Table` is
a set of `Column` values. A `Column` is a type-erased `BlobVec`, which
behaves like `Box<[T]>`. Bevy does **not** chunk. A column is one
contiguous, growable allocation for the whole table.
([Table docs](https://docs.rs/bevy/latest/bevy/ecs/storage/struct.Table.html),
[Column docs](https://docs.rs/bevy/latest/bevy/ecs/storage/struct.Column.html))

Each `Column` also holds `added_ticks` and `changed_ticks`, both
`Vec<UnsafeCell<Tick>>`, in two separate buffers parallel to the data. Row
`n` of the data matches index `n` of both tick buffers.

Bevy contributors have discussed moving ticks to table level, because the
tick writes use a lot of memory bandwidth. They also note that Bevy's query
system does not let the compiler use AVX2.
([DeepWiki: Archetypes and Storage](https://deepwiki.com/bevyengine/bevy/2.7-archetypes-and-storage),
[Change detection issue #4882](https://github.com/bevyengine/bevy/issues/4882))

Take this as direct evidence for recommendation 2.

### hecs, legion, EnTT

`hecs` is a small archetype ECS. It stores each archetype as a set of
contiguous type-erased columns, like Bevy. It has no chunks and no
scheduler. ([crates.io](https://crates.io/crates/hecs))

`legion` is archetype-based and does chunk, with "chunksets" keyed by tags
(its shared-component equivalent). Its latest published release is 0.4.0.
Check its activity before you depend on it.
([docs.rs](https://docs.rs/crate/legion/latest),
[GitHub](https://github.com/amethyst/legion))

`EnTT` uses sparse sets, not archetypes. Each component type gets a dense
array plus a sparse index array from entity ID to dense slot. Adding or
removing a component touches only that one component's pair of arrays.
([ECS back and forth](https://skypjack.github.io/2019-08-20-ecs-baf-part-4-insights/))

---

## 2. Archetype against sparse set, for your workload

Your workload is: about 1M entities, very few archetypes, near-zero
structural change per frame, and heavy full-set iteration.

This is the best case for archetypes and the worst case for sparse sets.

| Property | Archetype | Sparse set |
| --- | --- | --- |
| Iterate N entities over K components | K linear streams, no indirection | K linear streams over the dense arrays, **but** you must intersect the sets, and rows of different components do not line up |
| Add or remove a component | Move the whole entity between tables | Push or swap-remove in one array |
| Parallel split of a query | Trivial: split a column range | Harder: the intersection is not a contiguous range |
| Zero-copy NumPy view over a field | Direct: the column is the array | Only if the dense array happens to be in the order you want |

A recent academic comparison finds that sparse-set designs give cheaper
entity modification but scale poorly on iteration, and that archetype
designs give better iteration at large entity counts because of cache
locality, at the cost of composition change.
([Staffordshire CGVC 2025 paper](https://eprints.staffs.ac.uk/9315/1/cgvc20251224.pdf))

Sander Mertens's summary matches: archetypes are faster for ad-hoc groups
and parallelize more easily, but add and remove is costly.
([Flecs FAQ](https://www.flecs.dev/flecs/md_docs_2FAQ.html))

**The decisive point for you is not iteration speed. It is
recommendation 7 in the brief.** Because unit type is data and not a
component, and because upgrades are a bitmask and not components, you have
one archetype. A sparse set gives you nothing, because you never pay the
add/remove cost that a sparse set optimizes. Meanwhile, a single archetype
gives you a contiguous column per field. That column is exactly what a
zero-copy NumPy view needs.

Note the honest counterpoint: the Flecs author warns against trusting other
people's benchmarks, because the two designs differ at a basic level. Build
one micro-benchmark of your own iteration loop before you commit.

---

## 3. Chunk sizing

### Why Unity picked 16 KiB

16 KiB is half of a 32 KiB L1 data cache. That was the common L1 size when
DOTS was designed. The intent is that a whole chunk fits in L1 while a job
processes it.

### Why 16 KiB is probably wrong for you

Three reasons.

**Reason 1: your systems do not touch every component.** A movement system
reads position and speed and writes position. Only those columns enter the
cache. The "whole chunk in L1" argument does not apply. What matters is the
length of each column run.

**Reason 2: short runs defeat the hardware prefetcher.** Each component
column in a chunk is a separate memory stream. Intel's L2 streaming
prefetcher does not prefetch across a 4 KiB page boundary, and it tracks a
limited number of streams. With a 16 KiB chunk and a 40-byte entity, you get
409 entities per chunk. A `u32` column is then 1636 bytes. The prefetcher
locks onto a stream, runs for under 2 KiB, and then must lock on again at
the next chunk. Longer runs amortize this cost.

**Reason 3: TLB pressure.** 16 KiB is four 4 KiB pages. See
"Huge pages" below.

### Recommendation

Make the chunk size a compile-time constant. Default it to 64 KiB. Allocate
chunks from a 2 MiB-aligned arena, and allocate the chunks of one archetype
consecutively inside that arena. Then a full-column scan is close to a scan
of one flat array, and it stays inside huge pages.

Align every column start to 64 bytes. Do this so that a SIMD load never
straddles a cache line and so that two rayon workers on adjacent chunks
never share a line.

```rust
pub const CHUNK_BYTES: usize = 64 * 1024;
pub const CHUNK_ALIGN: usize = 64;
```

### If you keep 16 KiB

Then do not treat the chunk as the rayon task unit. Give each rayon task a
run of 16 or 32 consecutive chunks. A 409-entity task is too small to pay
for the scheduling overhead.

---

## 4. Concrete type sketches

### Entity ID

```rust
use core::num::NonZeroU64;

/// Packed as (generation << 32) | (!index).
/// The index uses NonMax encoding, so the all-ones value is free.
/// That free value makes the whole u64 non-zero when generation == 0,
/// so Option<Entity> is 8 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Entity(NonZeroU64);

impl Entity {
    #[inline]
    pub fn index(self) -> u32 { !(self.0.get() as u32) }
    #[inline]
    pub fn generation(self) -> u32 { (self.0.get() >> 32) as u32 }
}
```

Bevy does the same thing. It first made the generation a `NonZeroU32`, so
that `Entity` and `Option<Entity>` both take 8 bytes. It later moved the
niche into the index with a `NonMaxU32`, which frees the generation to hold
any value, including zero.
([Bevy PR #18704](https://github.com/bevyengine/bevy/pull/18704),
[tracking issue #18719](https://github.com/bevyengine/bevy/issues/18719))

Assert the size in a test:

```rust
const _: () = assert!(core::mem::size_of::<Option<Entity>>() == 8);
```

### Entity location table

```rust
/// Indexed by Entity::index(). One entry per allocated slot.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct EntityMeta {
    pub generation: u32,
    pub archetype: u32,
    pub chunk: u32,
    pub row: u32,
}
// 16 bytes. 1M entities => 16 MiB.
```

Keep this table dense and indexed by raw index. Do not use a hash map.

### Free list and ABA safety

Recycle slots **FIFO**, not LIFO. A LIFO free list hands the same slot back
within the same frame. Then a stale ID that a command buffer captured before
the despawn still matches the generation of a different entity, because the
generation only advances once.

```rust
pub struct Entities {
    meta: Vec<EntityMeta>,
    /// FIFO ring. A slot enters here at the frame barrier, not at despawn.
    free: VecDeque<u32>,
    /// Slots freed this frame. Merged into `free` at the next barrier.
    pending_free: Vec<u32>,
}
```

Increment the generation on **free**, not on allocate. Then a stale handle
becomes invalid the moment the entity dies.

On generation overflow (`u32::MAX`), retire the slot: never push it back to
the free list. At 60 frames per second and a full recycle of one slot every
frame, overflow takes 2.3 years. Retirement leaks at most 4 bytes per slot,
and only in a pathological case. This is cheap insurance and removes the ABA
question from the design.

**Where ABA can actually bite you.** Python never sees an entity ID, per
brief decision 7. So the exposure is internal. It is the command buffer and
the tile-to-unit index, both of which hold IDs across the frame barrier.
FIFO recycling plus free-time generation increment covers both.

### Archetype and chunk

```rust
pub struct Archetype {
    /// Sorted, deduplicated component IDs.
    components: Box<[ComponentId]>,
    /// Byte offset of each column inside a chunk, parallel to `components`.
    column_offsets: Box<[u32]>,
    /// Entities per chunk for this archetype.
    capacity: u32,
    chunks: Vec<ChunkId>,
    /// Cached graph edges. Small dense IDs index directly.
    add_edges: Box<[ArchetypeId]>,
    remove_edges: Box<[ArchetypeId]>,
}

#[repr(C, align(64))]
pub struct ChunkHeader {
    pub archetype: u32,
    pub len: u32,
    /// One tick per column. Set when a system writes that column.
    pub changed_ticks: [Tick; MAX_COLUMNS],
    /// Query acceleration. Kept up to date at the frame barrier.
    pub faction_mask: u64,
    pub bounds: AabbQR,
    pub unit_type_mask: u64,
}
```

Note the `changed_ticks` array in the header. That is recommendation 2.

### Column access

```rust
impl Chunk {
    /// # Safety
    /// `C` must be the type registered for `col`, and the caller must hold
    /// exclusive or shared access as the borrow rules require.
    pub unsafe fn column<C: Pod>(&self, col: usize) -> &[C] {
        let off = self.archetype.column_offsets[col] as usize;
        let ptr = self.base.add(off).cast::<C>();
        debug_assert!(ptr.align_offset(64) == 0);
        core::slice::from_raw_parts(ptr, self.header.len as usize)
    }
}
```

Use `bytemuck::Pod` as the bound. It gives you a safe, well-tested way to
say "this type is a plain byte pattern", and it gives you `cast_slice` for
free when you build the NumPy views.

---

## 5. Structural change batching

Your frame has a barrier. Use it. Collect every spawn, despawn, add, and
remove into per-thread command buffers. Apply them all at the barrier, in
deterministic order. This matches brief decisions 9 and 12.

### The algorithm

1. Concatenate the per-thread buffers in a fixed thread order. Do not use
   completion order.
2. Stable-sort the move requests by the key `(src_archetype << 32) |
   dst_archetype`. Use a radix sort. At 1M keys a two-pass radix sort on
   `u64` takes a few milliseconds, and it is deterministic.
3. For each `(src, dst)` run, resolve the archetype graph edge **once**.
   Compute the list of shared columns and the list of dropped columns once
   per run, not once per entity.
4. Move the whole run with one `memcpy` per column per destination chunk.
5. Compact the source.

Steps 3 and 4 are the reason to sort. Without sorting you redo the column
intersection per entity.

### The gotcha that will bite you

Swap-remove moves the **last** entity of a chunk into the gap. If that last
entity is itself pending a move later in the same batch, its recorded row is
now stale.

Two fixes. Both work.

- **Descending row order.** Within one source chunk, process removals from
  the highest row to the lowest. Then a swap-remove never moves an entity
  that you have not already handled.
- **Tombstone and compact.** Mark removed rows in a bitset, do no swapping,
  then compact each dirty chunk once at the end. This is simpler to reason
  about and it vectorizes, because compaction is a filtered copy. I
  recommend this one.

Either way, rewrite `EntityMeta.row` for every entity that physically moved.

### Bulk spawn

Spawning 100k units of one archetype should not go through the generic path.
Give the API a `spawn_batch(archetype, count) -> Range<u32>` that appends
whole chunks and returns a contiguous ID range. Then a verb can fill the
columns with `copy_from_slice`.

---

## 6. Query iteration and vectorization

### The state of SIMD in Rust

`std::simd` (`core::simd`) is **still nightly-only**. It sits behind
`feature(portable_simd)`, tracking issue #86656. The remaining work is to
split the API into independently stabilizable pieces, and to settle mask
types and swizzles. No stabilization date is announced.
([std::simd docs](https://doc.rust-lang.org/std/simd/index.html),
[critical issues #364](https://github.com/rust-lang/portable-simd/issues/364),
[The state of SIMD in Rust in 2025](https://shnatsel.medium.com/the-state-of-simd-in-rust-in-2025-32c263e5f53d))

You ship a library on stable Rust, so you cannot use it. Use these instead:

- **Autovectorization** for the common case. It handles `u8`, `u16`, `u32`,
  and integer arithmetic well.
- **`wide`** for explicit lanes on stable Rust. It gives portable `f32x8`,
  `i32x8`, and similar types.
  ([Using portable SIMD in stable Rust](https://pythonspeed.com/articles/simd-stable-rust/))
- **`core::arch` intrinsics plus `multiversion`** for the few hot kernels
  where you need AVX2 or NEON by hand, with runtime dispatch.

### How to make the compiler vectorize

Write loops in this shape:

```rust
pub fn advance(pos: &mut [i32], vel: &[i32]) {
    // Same length, so no bounds check inside the loop.
    let vel = &vel[..pos.len()];
    for (p, v) in pos.iter_mut().zip(vel) {
        *p = p.wrapping_add(*v);
    }
}
```

Rules that matter, in order of impact:

1. **Slice to equal length first.** This removes the bounds check and lets
   LLVM prove the trip count.
2. **No branches.** Use branchless select. A single `if` inside the loop
   usually stops vectorization dead.
   `*p = if cond { a } else { b }` compiles to a select only if both sides
   are cheap and side-effect free. Prefer arithmetic masks for anything else.
3. **Use `wrapping_*` or `unchecked` arithmetic.** Overflow panics are
   branches.
4. **Keep the number of streams low.** LLVM interleaves two or three slices
   well. Beyond four it often gives up. Split the system into two passes
   instead.
5. **Floating point reductions will not vectorize.** Rust has no fast-math
   on stable, and float addition is not associative. Write four or eight
   manual accumulators by hand, and accept that this changes the result.
   Fix the lane count in the source so the result stays bit-exact across
   machines. This matters for your determinism question.
6. **`chunks_exact` for manual lanes.**

```rust
pub fn sum_u16(xs: &[u16]) -> u64 {
    let mut acc = [0u64; 8];
    let mut it = xs.chunks_exact(8);
    for c in &mut it {
        for i in 0..8 { acc[i] += c[i] as u64; }
    }
    let mut total: u64 = acc.iter().sum();
    for &x in it.remainder() { total += x as u64; }
    total
}
```

7. **Do not use `slice::align_to` to get alignment.** It is safe but it
   produces a variable-length head, which adds a branch. Instead guarantee
   64-byte alignment at allocation time, as in the chunk layout above, and
   then assert it in debug builds.

### Verify, do not assume

Add `cargo-show-asm` to the toolchain and pin a small set of "must
vectorize" kernels. Check the assembly in CI. Autovectorization is fragile,
and a harmless-looking refactor will silently lose it.

---

## 7. Change detection

Bevy's model is a global monotonic tick counter plus two ticks per component
per entity: `added` and `changed`. A query with `Changed<T>` compares the
stored tick against the tick of the last run of that system.
([Column docs](https://docs.rs/bevy/latest/bevy/ecs/storage/struct.Column.html))

The model is correct and easy to reason about. It is also expensive.

### The cost at your scale

| Scheme | Bytes touched per frame, 1M entities, 10 components |
| --- | --- |
| Bevy: 2 ticks per component per entity | 80 MB of writes |
| One tick per component per chunk (64 KiB chunks) | about 25 KB |
| One dirty bit per entity per component | 1.25 MB |

The 80 MB figure is pure write bandwidth, and it is bandwidth you spend even
when nothing reads it. Bevy contributors raise exactly this concern.
([issue #4882](https://github.com/bevyengine/bevy/issues/4882))

### Recommendation

Use a three-level scheme.

1. **Per-chunk, per-column tick.** Always on. Costs one `u32` store per
   column per chunk per system run. This is enough for coarse work like
   "which chunks does the L1 projection need to re-read".
2. **Per-entity dirty bitplane.** Opt in, per component. One bit per entity.
   A rayon worker owns a whole chunk, so it owns whole words of the bitplane.
   No atomics needed if the chunk length is a multiple of 64.
3. **Nothing at all.** The default for components that no projection reads.

Make level 2 a declaration on the component, so the cost is visible in the
type:

```rust
pub trait Component: Pod {
    const TRACKING: Tracking = Tracking::Chunk;
}
pub enum Tracking { None, Chunk, Entity }
```

### The bigger point

Your dirty pyramid (brief decision 5) is the real change-detection system.
The L0 dirty bitset already answers "what changed". The ECS tick system only
has to answer "which unit columns changed", which is a much smaller
question. Do not build a second, general system that duplicates the pyramid.

---

## 8. Dense tile storage

### Layout: block-tiled, not row-major

Store every tile field in 32x32 blocks. Inside a block, use row-major order.
The block is the same 32x32 block that the brief uses for L1 aggregation.

```
index(q, r) = (((r >> 5) * (W >> 5)) + (q >> 5)) * 1024
            + ((r & 31) << 5)
            + (q & 31)
```

Every operation is a shift, a mask, or an add. There is no division, because
4096 and 32 are both powers of two. This directly supports brief decision 3.

**Why this wins.** L1 aggregation for one block reads exactly one contiguous
1024-byte span of a `u8` field, or one contiguous 128-byte span of a
bitplane. In row-major order the same aggregation touches 32 separate spans,
4096 bytes apart, on 32 different pages. Block-tiling turns 32 TLB entries
and 32 prefetch streams into one.

Viewport rendering and flow-field computation are also block-shaped. So is
the dirty pyramid walk.

**What you lose.** A long horizontal scan across the whole map is now
strided. Accept this. You have no such query. If one appears, it can walk
block rows.

### The columns

```rust
pub struct TileGrid {
    pub width_blocks: u32,
    pub height_blocks: u32,
    pub terrain:   AlignedVec<u8>,    // TerrainId
    pub elevation: AlignedVec<i16>,
    pub owner:     AlignedVec<u8>,    // FactionId, 255 = none
    pub moisture:  AlignedVec<u8>,
    pub resource:  AlignedVec<u8>,
    pub flags:     BitPlanes<16>,
    pub dirty_l0:  BitSet,
    pub upgrades:  SparseTiles<UpgradeRec>,
}
```

`AlignedVec<T>` is a 2 MiB-aligned allocation. Use `madvise(MADV_HUGEPAGE)`
on it (see "Huge pages").

### Bitplanes

One plane per boolean attribute. Each plane is a separate
`AlignedVec<u64>`. Do **not** pack several booleans into one byte per tile.

Why separate planes win:

- **Popcount queries.** "How many tiles in this block are forested and
  unowned" is `(a[i] & b[i]).count_ones()` over 16 `u64` words. That is
  16 iterations for a 1024-tile block, not 1024.
- **Set operations.** Union, intersection, and difference of two attributes
  are one `u64` op per 64 tiles.
- **Aggregation is a monoid.** `count_ones` is a sum. Bitwise-or is the
  identity-carrying combine. Both satisfy brief decision 4 exactly.
- **The dirty pyramid is already a bitplane.** Same code path.

```rust
pub struct BitPlanes<const N: usize> {
    /// N planes, each `words_per_plane` u64 values, each plane 64B aligned.
    data: AlignedVec<u64>,
    words_per_plane: usize,
}

impl<const N: usize> BitPlanes<N> {
    #[inline]
    pub fn plane(&self, p: usize) -> &[u64] {
        &self.data[p * self.words_per_plane..][..self.words_per_plane]
    }

    /// Count set bits of plane `p` in block `b`. One block is 1024 tiles,
    /// which is exactly 16 u64 words, so no partial words exist.
    #[inline]
    pub fn block_popcount(&self, p: usize, b: usize) -> u32 {
        self.plane(p)[b * 16..][..16].iter()
            .map(|w| w.count_ones()).sum()
    }
}
```

Note how clean `block_popcount` is. A 1024-tile block is exactly 16 `u64`
words, with no head or tail masking. That falls out of the block-tiled
layout and the power-of-two block size. This is a good argument for keeping
both.

`count_ones` compiles to `POPCNT` on x86-64 and `CNT` plus `ADDV` on
AArch64. Both are cheap. The sum over 16 words autovectorizes.

Consider the `roaring` crate only for genuinely sparse sets, such as the
list of blocks that hold any unit. Do not use it for the dense planes. A
dense plane is already the optimal representation at 16.7M bits.

### Byte budget

Grid is 4096 x 4096 = 16,777,216 tiles, which is 2^24. So a `u8` field is
exactly 16 MiB and one bitplane is exactly 2 MiB. That is a nice property.
Keep the grid a power of two on both axes.

| Field | Type | Bytes per tile | Total |
| --- | --- | --- | --- |
| terrain | u8 | 1 | 16 MiB |
| elevation | i16 | 2 | 32 MiB |
| owner | u8 | 1 | 16 MiB |
| moisture | u8 | 1 | 16 MiB |
| temperature | u8 | 1 | 16 MiB |
| resource type | u8 | 1 | 16 MiB |
| resource amount | u8 | 1 | 16 MiB |
| 16 boolean flags | 16 bitplanes | 2 | 32 MiB |
| **Subtotal, rich schema** | | **10** | **160 MiB** |
| L0 dirty bitset | 1 plane | 0.125 | 2 MiB |
| L1 summaries | 16,384 cells x 256 B | | 4 MiB |
| L2 summaries | 256 cells x 256 B | | 64 KiB |
| **Total tile side** | | | **about 166 MiB** |

A minimum schema of terrain, elevation, owner, and 8 flags costs 80 MiB.

| Schema | Bytes per tile | Total |
| --- | --- | --- |
| Minimum | 5 | 80 MiB |
| Rich (above) | 10 | 160 MiB |
| Very rich (add 4 more u8 and 16 more flags) | 16 | 256 MiB |

**Read this table as a budget, not a plan.** At 16.7M tiles, every extra
byte per tile costs 16 MiB and, more importantly, 16 MiB of memory traffic
for any full-grid pass. Make each field justify itself. Anything that is
rare belongs in a sparse side table, not a column.

### Sparse side tables

Some data is per-tile but rare: an upgrade, a name, a custom script hook.

Use two structures together:

```rust
pub struct SparseTiles<T> {
    /// One bit per tile. The index.
    present: BitPlane,
    /// The payload. Keyed by packed tile index.
    values: hashbrown::HashMap<u32, T>,
}
```

**The bitplane is the index. The map is only the payload.** Any bulk query
("count tiles with an upgrade in this block", "OR the upgrade flag up the
pyramid") reads the bitplane and never touches the map. The map only serves
a point lookup, and point lookups are rare and not on the hot path.

If the set is large enough that the map hurts (say, over 5% of tiles), swap
the map for a rank-select structure: keep a per-block prefix count of set
bits, then the dense payload index is `prefix[block] + popcount(word_prefix)`.
That gives O(1) lookup with no hashing and a fully dense payload array. This
is worth building only if you measure a problem.

---

## 9. The tile-to-unit bridge

The brief says "per-tile occupancy index, CSR-style (offsets + packed unit
array)". **I recommend against the literal form of this.**

A CSR offset array over the full grid needs 16,777,217 entries. At `u32`
that is 64 MiB. That is more than the entire minimum tile schema. It also
means every occupancy lookup, and every rebuild, touches 64 MiB.

At 1M units on 16.7M tiles, over 94% of those offsets are duplicates of
their neighbour. You would spend 64 MiB to store "empty" 15.7 million times.

### Recommendation: block-level CSR plus a sorted unit array

1. Keep an array of unit entity indices sorted by packed tile index. Sort it
   once per frame at the barrier with a radix sort on the `u32` key. 1M
   `u32` keys is a 3-pass or 4-pass radix sort, on the order of 2-5 ms
   single-threaded, and it is deterministic.
2. Store one `(start, len)` pair per 32x32 block.
3. To find the units on one tile, take the block range and binary search
   inside it. A block holds 1024 tiles, so the range is short.

| Structure | Size |
| --- | --- |
| Sorted unit index array (1M x u32) | 4 MiB |
| Sorted tile key array (1M x u32) | 4 MiB |
| Per-block ranges (16,384 x u64) | 128 KiB |
| **Total** | **about 8 MiB** |

That is an 8x saving over the full CSR, and the rebuild cost drops in
proportion.

Add a "block has any unit" bitplane on top. Then a selector descent skips
empty blocks with a popcount, which is exactly the pruning that brief
decision 6 wants.

**If you need per-tile lookup often enough that binary search hurts**, build
the full CSR lazily, per block, and only for blocks that a system asks
about. A per-block CSR is a 1025-entry `u16` array, which is 2 KiB, and it
sits in L1.

---

## 10. Cache-line concerns

### False sharing across rayon workers

Three real hazards.

**Hazard 1: bitset writes are a correctness bug, not just a slowdown.** If
two workers set different bits in the same `u64`, a read-modify-write from
each loses one update. This is not false sharing. It is a data race with a
wrong answer.

Fix: make the parallel split align to whole words, and preferably to whole
cache lines. A 32x32 block is 1024 bits, which is 16 `u64` values, which is
exactly two cache lines. So splitting the work at block granularity solves
this by construction. This is another reason to make the block the unit of
parallelism everywhere.

**Hazard 2: accumulators.** Any per-worker counter must sit on its own cache
line. Use `crossbeam_utils::CachePadded<T>`, or an explicit
`#[repr(align(64))]` wrapper. Do not hand-roll padding with a magic number.

**Hazard 3: chunk headers.** If two workers process adjacent chunks and both
write `changed_ticks`, and the headers are packed together, they share a
line. Fix: put the header inside its own chunk allocation, and align the
chunk to 64 bytes. The sketch above does this with `#[repr(C, align(64))]`.

### Determinism and atomics

An atomic `fetch_or` into a bitset **is** deterministic in result, because
`or` is commutative and associative. The order does not change the answer.
So you may use atomics for dirty marking without breaking brief decision 9.

An atomic `fetch_add` on a float is **not** deterministic. Never do it.
An atomic `fetch_add` on an integer is deterministic in result. It is fine.

Anything where a worker appends to a shared list is order-dependent. Use
per-thread lists concatenated in fixed thread order, per brief decision 12.

### Prefetching

Software prefetch helps in exactly one place in your design: the indirection
in the tile-to-unit bridge, where you read a unit index and then use it to
index a component column. Prefetch about 8 iterations ahead.

```rust
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn prefetch<T>(p: *const T) {
    use core::arch::x86_64::{_mm_prefetch, _MM_HINT_T0};
    _mm_prefetch(p as *const i8, _MM_HINT_T0);
}
```

`core::intrinsics::prefetch_read_data` is unstable. Use `core::arch` and
gate it per architecture. On AArch64 there is no stable intrinsic; use
inline `asm!("prfm pldl1keep, [{0}]", ...)`, which **is** stable.

Do not prefetch a linear scan. The hardware prefetcher already handles that,
and a redundant software prefetch costs an instruction slot.

### Huge pages

Your tile data is about 166 MiB. With 4 KiB pages that is 42,500 page table
entries. A typical L2 TLB holds 1500 to 3000 entries. So a full-grid pass
misses the TLB constantly. Each miss is a page walk.

With 2 MiB pages the same data needs 83 entries. That fits in the TLB with
room to spare.

Published measurements show the effect clearly. Once a working set passes
the L3 cache, TLB misses dominate, and large pages help a lot.
([JVM Anatomy Quark #2](https://shipilev.net/jvm/anatomy-quarks/2-transparent-huge-pages/))

**Recommendation.** Allocate every large array 2 MiB-aligned and call
`madvise(ptr, len, MADV_HUGEPAGE)`. Do not depend on the system setting.
Many distributions ship `enabled=madvise`, which gives huge pages only to
callers that ask.
([kernel docs](https://docs.kernel.org/admin-guide/mm/transhuge.html),
[Disable Transparent Hugepages](https://blog.nelhage.com/post/transparent-hugepages/))

**Caveat.** THP with `enabled=always` causes latency spikes, because
`khugepaged` and direct compaction stall the allocating thread. This is a
well-known production problem. Asking per allocation with `MADV_HUGEPAGE`,
at start-up, on an arena you allocate once, avoids it. Allocate the arena
once. Never grow it in the middle of a frame.

On Windows, large pages need the `SeLockMemoryPrivilege`, which a normal
user does not have. On macOS, use `VM_FLAGS_SUPERPAGE_SIZE_2MB`. Treat huge
pages as a Linux optimization and let the other platforms fall back.

---

## 11. Crate survey

| Crate | Use for | State |
| --- | --- | --- |
| `bytemuck` | `Pod` bound, `cast_slice` for zero-copy views | Mature. Take this. |
| `rayon` | Data parallelism over blocks and chunk runs | Mature. Take this. |
| `hashbrown` | Sparse side tables | Mature. Take this. |
| `wide` | Portable SIMD on stable Rust | Maintained. Take it where you need explicit lanes. |
| `multiversion` | Runtime AVX2 / NEON dispatch | Maintained. Take it for the few hand-written kernels. |
| `crossbeam-utils` | `CachePadded` | Mature. Take this. |
| `nonmax` | `NonMaxU32` for the entity index | Small and stable. Or write the eight lines yourself. |
| `roaring` | Sparse block-level sets | Mature. Use only for genuinely sparse sets. |
| `bitvec` | Bitplanes | **Skip it.** It is a large, complex crate that solves a general problem. Your bitplanes are `[u64]`. Write 100 lines. |
| `slotmap` | Generational keys | **Skip it.** It owns its storage, which fights your SoA columns. |
| `bevy_ecs` | ECS | Usable standalone, but see below. |
| `hecs` | ECS | Small, archetype-based, maintained. See below. |
| `legion` | ECS | Latest release 0.4.0. Chunked, tag-based. Verify activity before depending on it. |

### Why write your own ECS

I recommend you do not depend on any of them. Reasons:

1. **You need per-chunk user metadata** (faction mask, AABB, type
   histogram) for selector pruning. No Rust ECS exposes a hook for this.
2. **You need per-chunk change ticks, not per-entity.** Bevy hard-codes
   per-entity.
3. **You need raw column pointers with a stable layout** for zero-copy
   NumPy. Bevy's `BlobVec` is private and gives no stability guarantee.
4. **You need a specific deterministic structural-change order.** General
   ECS command buffers do not promise this.
5. **You have one archetype.** You are not using 90% of what these crates
   do, and you are paying for the generality in every query.

Your ECS is maybe 2000 lines because your requirements are narrow. Read
`hecs` for the archetype code, because it is small and clear. Read
`bevy_ecs` for the change-detection design, because it is the most thought
through. Copy neither wholesale.

**Counter-argument, stated fairly.** Writing an ECS is a well-known way to
spend six months not shipping a game. If you take this path, timebox it, and
write the query iteration benchmark first, so you know when to stop tuning.

---

## 12. Things that will bite you

1. **Swap-remove invalidates rows that a pending command still references.**
   See section 5. Use tombstone-and-compact.
2. **Autovectorization is silent when it fails.** A refactor loses it and
   nothing warns you. Pin the hot kernels with `cargo-show-asm` in CI.
3. **Concurrent bitset writes lose updates.** Not just false sharing, a
   wrong answer. Align every parallel split to whole `u64` words. Block
   granularity solves this by construction.
4. **The full-grid CSR is a 64 MiB trap.** See section 9.
5. **Per-entity change ticks cost 80 MB of writes per frame.** See
   section 7.
6. **Float reductions are not associative.** If you ever parallelize a float
   sum with rayon, the result changes with the thread count. This breaks
   bit-exact determinism. Either fix the reduction tree shape, or keep all
   aggregated values integer. Given that your monoid rule (decision 4) is
   already about sums and counts, **integer-only aggregation is worth
   considering as a hard rule.** It removes the whole problem.
7. **`madvise(MADV_HUGEPAGE)` at the wrong time stalls the thread.** Do it
   once at start-up, on an arena you never grow mid-frame.
8. **A chunk that is a rayon task is too small.** 409 entities does not pay
   for the scheduling. Batch chunk runs.
9. **The archetype graph edge hash map costs 5-10% of a structural change.**
   Keep component IDs small and dense so the edges are array-indexed, the
   way Flecs does.
10. **`repr(Rust)` field order is not stable.** Every type you expose to
    NumPy or write to disk needs `#[repr(C)]` and a size assertion.

---

## 13. Open questions for the ADR author

1. **Is the ECS earning its keep?** If units really do all live in one
   archetype, per brief decision 8, then what you have is a set of parallel
   `Vec` columns plus a generational free list. That is not an ECS. It is a
   generational SoA arena. Naming it honestly may save you a lot of code.
   The question to answer: **name three archetypes you expect to exist.** If
   you cannot, drop the archetype machinery and build the arena.

2. **What is the chunk size?** I recommend 64 KiB and a measurement. The
   brief says 16 KiB. Neither of us has your benchmark.

3. **Does anything need per-entity change detection?** If the dirty pyramid
   already covers projection invalidation, maybe nothing does.

4. **Bit-exact cross-platform determinism, or not?** This drives whether you
   may use floats at all in aggregated state. It also drives whether you may
   parallelize any reduction freely. Answer this before you write the first
   aggregation kernel, because retrofitting is painful.

5. **Are all boolean tile attributes known at compile time?** If yes,
   `BitPlanes<N>` with a const generic is right. If mods add planes at run
   time, you need a dynamic plane count, which changes the API.

6. **What is the real upper bound on units?** The brief says "hundreds of
   thousands to millions". 200k and 2M are different designs. 2M units at
   40 bytes is 80 MB, and it makes the sorted-occupancy rebuild a real cost.

7. **Does a unit ever occupy more than one tile?** The occupancy design
   above assumes one tile per unit. Multi-tile structures break it.

8. **Is the 32x32 block size fixed?** I have leaned on it hard: it makes a
   bitplane block exactly 16 `u64` words and two cache lines, and it makes
   the tile index pure shift-and-mask. If it might become 16x16 or 64x64,
   say so, because the constants above change.

---

## Sources

- [Unity Entities: Archetypes concepts](https://docs.unity3d.com/Packages/com.unity.entities@1.0/manual/concepts-archetypes.html)
- [Unity Entities: ArchetypeChunk API](https://docs.unity3d.com/Packages/com.unity.entities@0.4/api/Unity.Entities.ArchetypeChunk.html)
- [Unity Learn: Minimizing cache misses](https://learn.unity.com/course/dots-best-practices/unit/part-3-implementation-and-optimization/tutorial/part-3-3-minimizing-cache-misses?version=2022.3)
- [Flecs FAQ](https://www.flecs.dev/flecs/md_docs_2FAQ.html)
- [Flecs Queries](https://www.flecs.dev/flecs/md_docs_2Queries.html)
- [Sander Mertens: Building an ECS #2, Archetypes and Vectorization](https://ajmmertens.medium.com/building-an-ecs-2-archetypes-and-vectorization-fe21690805f9)
- [Sander Mertens: Building an ECS #3, Storage in Pictures](https://ajmmertens.medium.com/building-an-ecs-storage-in-pictures-642b8bfd6e04)
- [skypjack: ECS back and forth, part 4](https://skypjack.github.io/2019-08-20-ecs-baf-part-4-insights/)
- [Staffordshire CGVC 2025: archetype vs sparse set comparison (PDF)](https://eprints.staffs.ac.uk/9315/1/cgvc20251224.pdf)
- [bevy::ecs::storage::Table](https://docs.rs/bevy/latest/bevy/ecs/storage/struct.Table.html)
- [bevy::ecs::storage::Column](https://docs.rs/bevy/latest/bevy/ecs/storage/struct.Column.html)
- [Bevy issue #4882: change detection cost](https://github.com/bevyengine/bevy/issues/4882)
- [Bevy PR #18704: make entity index non-max](https://github.com/bevyengine/bevy/pull/18704)
- [Bevy issue #18719: better entities tracking](https://github.com/bevyengine/bevy/issues/18719)
- [DeepWiki: Bevy archetypes and storage](https://deepwiki.com/bevyengine/bevy/2.7-archetypes-and-storage)
- [std::simd documentation (nightly)](https://doc.rust-lang.org/std/simd/index.html)
- [portable-simd issue #364: critical issues before stabilization](https://github.com/rust-lang/portable-simd/issues/364)
- [The state of SIMD in Rust in 2025](https://shnatsel.medium.com/the-state-of-simd-in-rust-in-2025-32c263e5f53d)
- [Using portable SIMD in stable Rust](https://pythonspeed.com/articles/simd-stable-rust/)
- [Linux kernel: Transparent Hugepage Support](https://docs.kernel.org/admin-guide/mm/transhuge.html)
- [JVM Anatomy Quark #2: Transparent Huge Pages](https://shipilev.net/jvm/anatomy-quarks/2-transparent-huge-pages/)
- [Nelson Elhage: Disable Transparent Hugepages](https://blog.nelhage.com/post/transparent-hugepages/)
- [hecs on crates.io](https://crates.io/crates/hecs)
- [legion on docs.rs](https://docs.rs/crate/legion/latest)
