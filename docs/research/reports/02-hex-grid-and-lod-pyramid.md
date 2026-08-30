# Hex Grid Representation, LOD Aggregation, and Spatial Indexing

Research input for ADR-0001. Area: hex coordinates, memory layout, the hex
hierarchy problem, the LOD pyramid, monoid aggregation, dirty propagation,
and query acceleration.

## Executive summary

These are the recommendations. The detail sections give the evidence.

1. **Use axial `(q, r)` as the logical coordinate.** Derive cube `s = -q-r`
   when you need rotation, reflection, or distance. Never store offset or
   doubled coordinates in the API.
2. **Store tiles at an offset (odd-r) index, not at a raw axial index.**
   Convert at the array boundary with one shift and one add. This wastes no
   memory on a rectangular world, and it makes each block a near-square in
   world space. A raw axial store wastes 50% of the array for a rectangular
   world, and its blocks have a 1.73:1 aspect ratio, which weakens query
   pruning. This is a small refinement of decision 3 in the context brief.
   If the world is a rhombus by design, store raw axial and skip the
   conversion.
3. **Use a chunked (tiled) layout: 32x32 blocks, row-major inside a block.**
   Keep struct-of-arrays *globally*, but index the arrays with the chunked
   index. Each field of each chunk is then 1024 contiguous elements. This
   gives sequential SIMD reduction for aggregation with no extra structure.
4. **Do not use Morton or Hilbert order for tiles.** Order the *chunks* in
   Morton order if you want more locality. Per-tile Morton makes neighbor
   math 5x more expensive and breaks contiguous block scans.
5. **Choose exact parallelogram nesting, not H3-style aperture 7.** Aperture 7
   costs you power-of-two index math, cache alignment, and honest geometry.
   It buys you a hexagonal parent shape that no player will notice at L1
   zoom. This agrees with the context brief.
6. **Reject summed-area tables.** A single tile write dirties a whole quadrant
   of a SAT. A mip-style pyramid updates in O(levels) per write.
7. **Split aggregates into two classes.** Group-like statistics (sum, count,
   histogram) take a signed delta straight up the chain. Monoid-only
   statistics (min, max, bitwise-or) need a block recomputation. Two tricks
   remove most of that cost: store a *count of children at the extremum*
   with min and max, and store a *popcount per bit* instead of a bare OR
   mask. Both turn a monoid into a group.
8. **Track dirtiness per chunk, not per tile.** A 4096x4096 grid with 32x32
   chunks has 16384 chunks. A flat bitset over them is 2 KB. It fits in L1
   cache. You do not need a hierarchical bitset, `hibitset`, or roaring
   bitmaps at this scale.
9. **For query pruning, store a lower bound and an upper bound for every
   field a selector filters on.** Min and max for numbers. AND mask and OR
   mask (or popcounts plus a total) for categories. Without both bounds you
   can prune a subtree away, but you cannot accept a subtree whole, and the
   accept case is the larger win.

---

## 1. Hex coordinate systems

Red Blob Games is the reference for this section.[^rbg]

### 1.1 The four systems

| System | Fields | Neighbors | Distance | Rotation | Storage |
|---|---|---|---|---|---|
| Cube `(x,y,z)`, `x+y+z=0` | 3 | 6 fixed offsets | `(|dx|+|dy|+|dz|)/2` | permute and negate | 1 field is redundant |
| Axial `(q,r)` | 2 | 6 fixed offsets | derive `s`, then cube | derive `s`, then cube | compact |
| Offset (odd-r / even-q) | 2 | offsets change per row | convert to cube first | convert to cube first | rectangle, no waste |
| Doubled | 2 | 6 fixed offsets | `(|dcol| + max(0, (|drow|-|dcol|)/2))` | awkward | rectangle, no waste |

Cube and axial support vector arithmetic. You can add and subtract them.
Offset coordinates cannot do this, because the neighbor offsets depend on
whether the row index is even or odd. Doubled coordinates support addition
and subtraction, but their rotation and reflection rules are ugly.

### 1.2 Recommendation: axial for logic, offset for the array index

Separate the *logical coordinate type* from the *storage index*. They do not
have to be the same thing.

Use axial `(q, r)` as `i32` in every public type, every selector, and every
piece of geometry code. Convert to cube in-register when you rotate,
reflect, or measure distance. The conversion is one subtraction.

Use an odd-r offset index for the array position. The conversion is one
shift and one add in each direction.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Axial { pub q: i32, pub r: i32 }

impl Axial {
    #[inline] pub const fn s(self) -> i32 { -self.q - self.r }

    #[inline]
    pub fn distance(self, o: Axial) -> u32 {
        let dq = self.q - o.q;
        let dr = self.r - o.r;
        ((dq.abs() + dr.abs() + (dq + dr).abs()) / 2) as u32
    }

    /// Rotate 60 degrees. Apply six times to return to the start.
    #[inline] pub const fn rot_cw(self)  -> Axial { Axial { q: -self.r, r: self.q + self.r } }
    #[inline] pub const fn rot_ccw(self) -> Axial { Axial { q: self.q + self.r, r: -self.q } }

    /// Reflect across the q axis.
    #[inline] pub const fn reflect_q(self) -> Axial { Axial { q: self.q + self.r, r: -self.r } }
}

pub const NEIGHBORS: [Axial; 6] = [
    Axial { q:  1, r:  0 }, Axial { q:  1, r: -1 }, Axial { q:  0, r: -1 },
    Axial { q: -1, r:  0 }, Axial { q: -1, r:  1 }, Axial { q:  0, r:  1 },
];

/// Axial to odd-r offset column. Arithmetic shift, so negative r is correct
/// in two's complement.
#[inline]
pub const fn axial_to_col(a: Axial) -> i32 { a.q + ((a.r - (a.r & 1)) >> 1) }

#[inline]
pub const fn offset_to_axial(col: i32, row: i32) -> Axial {
    Axial { q: col - ((row - (row & 1)) >> 1), r: row }
}
```

Why not store raw axial? A 4096x4096 axial parallelogram is a 60-degree
rhombus in world space. If you want a rectangular world, you must allocate a
bounding parallelogram. Over 4096 rows the shear moves the row start by 2048
columns, so you need 6144 columns of storage for a 4096-wide rectangle. That
is 25.2M cells for 16.7M live tiles: 50% waste. At 16 bytes per tile that is
136 MB of dead memory.

Why not store raw axial *and* accept a rhombus world? That is a valid
choice, and it removes the conversion entirely. Take it if the game design
allows a rhombus map. Section 3.4 explains the other cost, which is block
aspect ratio.

### 1.3 Ranges, lines, and rings

- **Range N around a center**: iterate `dq` in `-N..=N`, and `dr` in
  `max(-N, -dq-N)..=min(N, -dq+N)`. This yields `3N(N+1)+1` cells with no
  rejection test.
- **Ring at radius N**: step to `center + NEIGHBORS[4] * N`, then walk N
  steps in each of the 6 directions. Cost is `6N`.
- **Line**: linear interpolation in cube space, then cube rounding. Round
  each of the three components, then correct the one with the largest
  rounding error so the sum stays zero. Add a small epsilon to the start
  point to break ties consistently. Ties on exact edges are a classic
  determinism bug.
- **Field of view and pathing**: prefer a hierarchical flow field over the
  pyramid, as the context brief already plans (decision 11).

---

## 2. Memory layout

Target: 4096x4096 = 16.78M tiles. Assume 16 bytes per tile across all
struct-of-arrays fields, so about 268 MB at L0.

### 2.1 The candidates

**Global row-major.** `index = row * cols + col`. Neighbor access touches
rows `row-1`, `row`, `row+1`. For u8 fields a row is 4 KB, so three rows are
12 KB and fit in a 32 KB L1 cache. A full sequential scan is therefore fine.
A random neighbor lookup costs up to 3 cache line fills. Block aggregation
is the problem: a 32x32 block is 32 separate runs of 32 bytes, spread over
128 KB of address space. The hardware prefetcher sees 32 short streams and
helps little.

**Chunked (tiled).** Split into 32x32 blocks. Store the blocks in an array,
and store the tiles row-major inside each block. A 32x32 block of a u8 field
is 1024 contiguous bytes: 16 cache lines, one stream, perfect for a SIMD
reduction. Interior neighbor access stays inside the block. Only the 124
border tiles of a 1024-tile block cross a boundary, which is 12% of tiles.

**Morton (Z-order).** Interleave the bits of `col` and `row`. Locality is
good at every scale with no tuning, and the parent index is `code >> 2k`.
Two costs kill it here. First, neighbor math needs masked-carry
("tesseral") arithmetic, roughly 5 to 6 operations per axis instead of one
add. Second, a contiguous Morton run is a square block, so a *viewport
rectangle* or a *row scan* becomes many disjoint runs, and you need
BIGMIN/LITMAX range splitting to enumerate them.

**Hilbert.** Better locality than Morton, and no jumps between successive
indices. But encode and decode cost several times more than Morton, and
there is still no cheap neighbor step. Use it for disk layout or streaming
order, not for a hot in-memory index.

### 2.2 Quantified comparison for block aggregation

Aggregate one 32x32 block of a u8 field.

| Layout | Distinct memory streams | Cache lines touched | Prefetcher effective |
|---|---|---|---|
| Chunked | 1 | 16 | yes |
| Global row-major | 32 | up to 64 (unaligned 32-byte runs straddle lines) | weakly |
| Morton | 1 | 16 | yes |

Chunked and Morton tie on aggregation. Chunked wins on neighbor cost and on
viewport scans. Therefore chunked wins overall.

### 2.3 Recommendation

Use **global struct-of-arrays, indexed by a chunked index**. Do not build a
per-chunk struct with its own arrays. One `Vec<u8>` per field over all 16.7M
tiles is enough, because the chunked index already puts each chunk's 1024
entries of each field next to each other. This is the simplest structure
that has every property you want.

```rust
pub const CHUNK_BITS: u32 = 5;
pub const CHUNK_W: usize = 1 << CHUNK_BITS;        // 32
pub const CHUNK_TILES: usize = CHUNK_W * CHUNK_W;  // 1024
const LOCAL_MASK: u32 = CHUNK_W as u32 - 1;

pub struct GridDims { pub chunk_cols: u32, pub chunk_rows: u32 }

#[inline]
pub fn tile_index(d: &GridDims, col: u32, row: u32) -> usize {
    let cx = col >> CHUNK_BITS;
    let cy = row >> CHUNK_BITS;
    let lx = (col & LOCAL_MASK) as usize;
    let ly = (row & LOCAL_MASK) as usize;
    let chunk = (cy * d.chunk_cols + cx) as usize;
    chunk * CHUNK_TILES + ly * CHUNK_W + lx
}

/// L1 cell index of a tile. This is the chunk index itself.
#[inline]
pub fn l1_index(d: &GridDims, col: u32, row: u32) -> u32 {
    (row >> CHUNK_BITS) * d.chunk_cols + (col >> CHUNK_BITS)
}
```

Optional refinement: order the chunks in Morton order in the chunk array
instead of row-major. This costs one 16-bit table lookup per axis when you
compute a chunk index, and it improves locality for viewport scans and for
L2 aggregation. It costs nothing per tile. Measure before you adopt it.

Two extra benefits of the chunked layout, both of which the ADR should note:
- The chunk is the natural unit for the dirty bitset (section 6).
- The chunk is the natural unit for rayon parallelism, because chunks are
  disjoint (context brief decision 9: "aggregate boundary = parallelism
  boundary").

---

## 3. The hex hierarchy problem

Hexagons do not tile into larger hexagons. This is a fact of the plane, not
a limitation of any library. Every hexagonal hierarchy therefore makes a
compromise. Below are the real options and the real costs.

### 3.1 Aperture 7 (H3)

H3 gives each cell seven children at the next resolution.[^h3] To make seven
smaller hexagons approximate one larger hexagon, the lattice rotates between
levels. The rotation angle is `atan(sqrt(3) / 5)`, which is about 19.106
degrees, and the linear scale factor is `sqrt(7)`.

The H3 documentation is explicit about the compromise: **logical containment
in the index is exact, but geographic containment is only approximate.**[^h3]
The parent's drawn boundary cuts through its children. A child on the border
of a parent is partly inside the parent's polygon and partly outside.

What this means for aggregation:

- Exact aggregation **is** possible. Define the parent's aggregate as the
  fold over its seven logical children. That fold is exact, complete, and
  disjoint. Nothing is double counted.
- What is *not* exact is the correspondence between the aggregate and the
  drawn shape. If you render an L1 hexagon and say "this region has 400
  population", that claim is wrong at the boundary by up to roughly half a
  child cell.

The real costs of aperture 7 for this project:

1. **7 is not a power of two.** Parent lookup is a base-7 digit shift, not a
   bit shift. Planar aperture-7 indexing uses generalized balanced ternary
   over Eisenstein integers.[^eisen] Neighbor stepping needs a lookup table
   with carry propagation, not a single vector add.
2. **Block sizes never align to cache lines.** Levels have 7, 49, 343 cells.
   You cannot pick a SIMD width or a chunk size that divides them cleanly.
3. **You lose contiguous storage.** A parent's seven children are not
   contiguous in any simple linear order over a plane. You would need an
   explicit child index table.
4. **Pentagons.** H3 has 12 pentagons per resolution because it projects an
   icosahedron onto a sphere. A *planar* aperture-7 hierarchy has no
   pentagons, so this cost does not apply to a flat game world. Do not use
   this as an argument against aperture 7; it is not a real cost here.

### 3.2 Aperture 3 and aperture 4

Aperture 3 gives 3 children per parent and rotates 30 degrees per level.
Aperture 4 gives 4 children per parent and **does not rotate**, because the
larger hexagon is an exact scale-by-2 of the lattice orientation. Aperture
`2^(2k)` families have been formalized as factor rings of Eisenstein
integers.[^eisen]

Aperture 4 looks attractive: power-of-two fanout and no rotation. But it is
still not an exact partition. A hexagon of twice the edge length has 4 times
the area, and it overlaps 7 smaller hexagons partially. You get a
power-of-two ratio without exact nesting, which is the worst of both.

### 3.3 Rhombille

Three 60-degree rhombi tile one hexagon, and rhombi do self-tile at any
scale.[^rhombille] This is the honest way to get an exactly-nesting
hierarchy on a hex lattice: store rhombi, render hexes. It is worth knowing
about, but it triples your cell count at L0 for no gameplay gain. Do not use
it.

### 3.4 Exact parallelogram (rhombus) blocks — recommended

Take a power-of-two block in the storage index space. The parent index is a
right shift. The partition is exact, disjoint, and complete. The fold is a
plain monoid fold with no boundary correction and no double counting.

If you block on **raw axial** coordinates, the block is a 60-degree rhombus
in world space. For a pointy-top hex of width `w` and height `h`, a 32x32
axial block spans `48w` horizontally and `24h` vertically. With
`w = sqrt(3)` and `h = 2`, that is 83.1 by 48.0 units, an aspect ratio of
1.73:1. The ratio of the block's circumradius to its inradius is also 1.73,
against 1.155 for a regular hexagon. This matters for query pruning: a
conservative bounding radius around an anisotropic block admits more false
positives, so a radius query descends into more subtrees than it needs.

If you block on the **offset index** instead, the block is a near-rectangle
in world space with staircase edges. A 32x32 odd-r block spans `32w = 55.4`
by `24h = 48.0` units, an aspect ratio of 1.15:1. Pruning is tight, a
rectangular viewport maps to a rectangular chunk range, and the world has no
wasted array cells.

The staircase edge does not hurt anything, because the aggregate is defined
over the *index set*, not over a geometric polygon.

### 3.5 Recommendation and the perceptibility question

**Use power-of-two blocks in offset index space.** Render the L1 and L2
cells however you like.

Is the visual difference perceptible? Consider the numbers. A 4096x4096 grid
with 32x32 blocks gives 128x128 L1 cells. At the zoom level where a player
sees L1, one L1 cell covers a small part of the screen. The player is
reading colour, a label, and a number. They are not counting the sides of
the cell, and they cannot see the L0 tiles that back it.

Three rendering options, in increasing quality:
1. Draw the rhombus or rectangle directly. Cheapest. Looks like a
   coarse tile map.
2. Draw a hexagon centred on the block centroid, sized to the block area.
   The hexagon does not exactly cover the block's tiles. This is the same
   lie that H3 tells, and it costs nothing, since the aggregation is still
   exact.
3. Draw the outline of the block's member tiles as a single merged polygon
   ("blob"). Exact and hexagon-flavoured. Compute it once per block layout,
   not per frame.

Option 3 gives the honest hex look with exact aggregation. It removes the
only real argument for aperture 7.

**Failure mode to record in the ADR:** if you ever need to interoperate with
a real geospatial dataset, or to publish cell IDs that a geospatial tool
understands, parallelogram blocks are a dead end and H3 is the answer. This
project is a game world with no geographic reference frame, so this does not
apply.

---

## 4. The LOD aggregation pyramid

### 4.1 Shape and cost

Choose a fanout `f` per level. Level `k` has `N / f^(2k)` cells. Pyramid
overhead above L0 is `1 / (f^2 - 1)` of the L0 cell count.

| Fanout | L1 cells (from 4096^2) | L2 cells | Overhead over L0 count |
|---|---|---|---|
| 8 | 512 x 512 = 262144 | 64 x 64 = 4096 | 1.6% |
| 16 | 256 x 256 = 65536 | 16 x 16 = 256 | 0.4% |
| 32 | 128 x 128 = 16384 | 4 x 4 = 16 | 0.1% |

Fanout 32 makes L2 useless: 16 cells is not a "region scale" map, it is four
quadrants. Fanout 16 for both levels gives 65536 L1 cells and 256 L2 cells,
which is a reasonable region map. Fanout 8 gives more L2 detail but 4x more
L1 cells to maintain.

**Recommendation: fanout 16 at both levels**, so a 16x16 chunk of 256 tiles.
This makes the storage chunk 256 tiles rather than 1024. A 256-byte u8 field
run is still 4 cache lines and still SIMD-friendly. Revisit if the "city
scale" design wants a larger footprint.

Note the real memory cost is not the cell count; it is the payload width.
A `CellSummary` of 128 bytes at 65536 L1 cells is 8 MB. The pyramid is
cheap. Wide summaries are not. Budget the summary struct explicitly.

### 4.2 Mipmap-style reduction

The standard approach and the right one. Each level is a full array of
summaries. Rebuilding a cell reads its `f^2` children and folds them.
Because of the chunked layout, those children are contiguous.

A point write propagates in `O(levels)` work: 2 levels here.

### 4.3 Summed-area tables — reject

A summed-area table stores the prefix sum over both axes, so any
axis-aligned rectangle sum costs 4 lookups regardless of size. It is a poor
fit for this project, for four reasons.

1. **Mutation is catastrophic.** Changing one tile changes every SAT entry
   below and to the right of it. For a 4096x4096 grid, a write at the
   top-left dirties 16.7M entries. There is no incremental fix; this is
   inherent to the prefix-sum structure. The whole design in the context
   brief assumes frequent local writes.
2. **Value growth.** SAT entries hold the sum of everything above and left.
   A u8 field over 16.7M tiles needs a u40 accumulator, so u64 in practice.
   That is 134 MB for one field. In floating point you lose precision at the
   far corner.
3. **Sum-like only.** SAT needs an invertible operation. It cannot do min,
   max, or argmax at all.
4. **Rectangles only.** Selector queries here are radial, region-shaped, and
   predicate-driven, not axis-aligned rectangles.

Record SAT as considered and rejected. It is a read-optimal, write-hostile
structure, and this workload writes constantly.

### 4.4 Fenwick / binary indexed trees

A 2D Fenwick tree gives `O(log^2 n)` point update and `O(log^2 n)`
rectangle prefix sum. It fixes SAT's update cost. It keeps SAT's other two
limits: sums only (it needs a group with an inverse), and rectangles only.

Compare against the mip pyramid for our actual query: "aggregate over a
fixed pyramid cell". The mip pyramid answers that in **one** array read.
Fenwick needs `O(log^2 n)` reads. The pyramid wins because our query
granularity is fixed, not arbitrary.

**Recommendation: do not build a Fenwick tree now.** Note it as the answer
if a future feature needs "sum of field X over an arbitrary rectangle" at
high frequency. It composes with the pyramid rather than replacing it.

---

## 5. Monoid and semigroup framing for aggregates

The context brief's decision 4 is correct and important. This section
sharpens it in one place: **a monoid is enough to build the pyramid upward,
but it is not enough to update the pyramid incrementally.** For that you need
a group, which is a monoid plus an inverse.

### 5.1 The statistic table

| Statistic | Associative | Identity | Group (has inverse) | Delta update | Notes |
|---|---|---|---|---|---|
| sum | yes | 0 | yes | yes | `agg += new - old` |
| count | yes | 0 | yes | yes | |
| histogram `[u32; K]` | yes | all zeros | yes | yes | `h[old] -= 1; h[new] += 1` |
| min / max | yes | +inf / -inf | **no** | conditional | see 5.2 |
| bitwise OR | yes | 0 | **no** | conditional | see 5.3 |
| bitwise AND | yes | all ones | **no** | conditional | dual of OR |
| mean | — | — | — | yes | store `(sum, count)`, divide at read |
| variance | yes | zeros | yes | yes | store `(n, sum, sum_sq)`; numerically fragile |
| dominant / argmax | **no** | — | — | — | store histogram, argmax at read |
| median / percentile | **no** | — | — | — | approximate from a bucketed histogram |
| distinct count | via HLL | HLL zero | no | no | HLL merges by elementwise max; cannot remove |
| top-K | approximate | — | no | no | Space-Saving merges, but is lossy |

Two entries deserve comment.

**argmax with a tiebreak is a monoid.** If you carry the pair
`(value, tiebreak_key)` and combine by taking the larger value, breaking ties
on the smaller key, that combine *is* associative and commutative. So
"largest city in this region" is a legal L1 field, provided the tiebreak key
is a stable identifier. Plain argmax over a mutable field is not
delta-updatable, for the same reason max is not.

**Variance is a monoid but is numerically fragile.** Naive `sum_sq` loses
precision. If you need it, use Chan's parallel update formula, which
combines two `(n, mean, M2)` triples. It is associative. It is also a
determinism hazard under parallel folds; see 5.5.

### 5.2 Min and max: the extremum-count trick

The problem: an L1 cell holds `min = 3`. A child changes from 3 to 9. Is the
new min still 3? You cannot tell from the aggregate alone. You must rescan
all `f^2` children.

The fix: store the extremum **and a count of children that achieve it**.

```rust
#[derive(Clone, Copy)]
pub struct MinTracked { pub value: i16, pub count: u16 }
```

Update rules for a child changing `old -> new`:

- `new < value`  : set `value = new`, `count = 1`. No rescan.
- `new == value` : `count += 1` if `old != value`. No rescan.
- `old != value` : nothing changes. No rescan.
- `old == value && new > value` : `count -= 1`. If `count > 0`, no rescan.
  **Only if `count` reaches 0 do you rescan the children.**

For real data with many equal values (terrain type, elevation on a plain),
`count` is usually well above 1, so the rescan almost never fires. For
uniformly random continuous data, `count` is usually 1, so it fires often.
Know which case your data is in. Combining two cells upward is still just
`min` and a count sum on ties.

### 5.3 Bitwise OR: the popcount trick

The problem: an L1 cell holds `faction_mask = 0b0110`. A unit of faction 1
leaves. Is bit 1 still set? Only a rescan tells you.

The fix: store a **count per bit** instead of a mask.

```rust
pub struct FactionPop { pub per_bit: [u16; 16] }

impl FactionPop {
    #[inline] pub fn mask(&self) -> u16 {
        let mut m = 0u16;
        for (i, &c) in self.per_bit.iter().enumerate() {
            m |= ((c != 0) as u16) << i;
        }
        m
    }
}
```

`per_bit` is a histogram, so it is a group. Increment and decrement update
it exactly with no rescan. The mask is derived at read. The cost is 32 bytes
per cell for 16 factions, against 2 bytes for a bare mask. At 65536 L1 cells
that is 2 MB instead of 128 KB. Pay it. It converts your most common
categorical aggregate from monoid-only to group.

This same trick applies to any OR mask over a small alphabet: terrain
present, unit type present, capability present.

### 5.4 The two update paths

Design the pyramid update as two passes over the dirty set.

**Path A, delta (sums, counts, histograms, popcounts).** The write site
already knows `old` and `new`. It applies the signed delta to the L1 cell
and the L2 cell directly. Cost `O(levels)` per changed tile. No block read.

**Path B, recompute (min, max, and anything that failed the fast path).**
The write site marks the chunk dirty. At the frame barrier, a worker reads
all `f^2` children and folds. Cost `O(f^2)` per dirty chunk, but it is a
contiguous SIMD scan of 256 or 1024 elements, which is a few hundred
nanoseconds.

Crossover: path A wins when few tiles change per chunk. Path B wins when
many do. A concrete rule: if the number of changed tiles in a chunk exceeds
about `f^2 / 8`, recompute the whole chunk instead of applying deltas.
Track the count while you queue writes; do not compute it later.

### 5.5 Determinism

Floating-point addition is not associative. If you fold a chunk in parallel
with a non-deterministic reduction order, you get non-deterministic results.
This bears directly on the context brief's open question about determinism.

Two rules make it safe:
1. **Prefer integer and fixed-point aggregates.** Integer addition is
   associative, so any fold order gives the same answer. This is the easy
   fix and it covers almost every field here (counts, histograms, elevation
   sums, population).
2. **If a float aggregate is unavoidable, fix the fold order.** Fold
   sequentially inside a chunk, in index order. Fold chunks into a parent in
   chunk-index order, never in completion order. This gives bit-exact
   results on one platform. Cross-platform bit-exactness needs more
   (FMA contraction control, no fast-math, a fixed libm), which is a
   separate ADR question.

---

## 6. Dirty propagation

### 6.1 Do not build a per-tile dirty bitset

A per-tile dirty bitset over 16.7M tiles is 2 MB. Scanning 2 MB to find a
handful of set bits costs roughly 70 microseconds at 30 GB/s. That is a
large fraction of a frame budget for no information gain.

Hierarchical bitsets fix the scan cost. `hibitset` uses layers of `u64` with
a factor of 64 per layer; it is dense and allocates by maximum index.
`hi_sparse_bitset` is a sparse variant that reports faster inter-set
operations and lower memory than `hibitset`, because its memory depends on
used blocks rather than maximum index.[^hisparse] Roaring bitmaps compress
well and have excellent set algebra.[^roaring]

**But you do not need any of them at this scale, because you should not
track dirtiness per tile in the first place.**

### 6.2 Track dirtiness per chunk

With fanout 16, a 4096x4096 grid has 65536 chunks. A flat bitset over 65536
bits is **8 KB**. With fanout 32 it is 16384 chunks and **2 KB**. Either
fits in L1 cache. An exhaustive scan is 1024 or 256 `u64` loads, which is on
the order of a microsecond, and you can skip words that are zero with a
single compare.

L1's own dirty bitset covers 65536 cells: 8 KB. L2's covers 256 cells: 32
bytes.

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct DirtyLevel { words: Box<[AtomicU64]> }

impl DirtyLevel {
    pub fn new(bits: usize) -> Self {
        let n = (bits + 63) / 64;
        Self { words: (0..n).map(|_| AtomicU64::new(0)).collect() }
    }

    /// Safe to call from any worker. Idempotent.
    #[inline]
    pub fn mark(&self, i: u32) {
        self.words[(i >> 6) as usize]
            .fetch_or(1u64 << (i & 63), Ordering::Relaxed);
    }

    /// Clear and collect indices in ascending order. Deterministic.
    pub fn drain_into(&self, out: &mut Vec<u32>) {
        out.clear();
        for (w, word) in self.words.iter().enumerate() {
            let mut bits = word.swap(0, Ordering::Relaxed);
            while bits != 0 {
                out.push((w as u32) * 64 + bits.trailing_zeros());
                bits &= bits - 1;   // clear lowest set bit
            }
        }
    }
}
```

`bits &= bits - 1` with `trailing_zeros` is the standard set-bit walk. It
costs about 2 cycles per set bit on any modern CPU.

### 6.3 Optional sub-chunk masks

If a chunk has 256 tiles, a per-tile sub-mask is 4 `u64` words, or 32 bytes
per chunk. It lets a recompute pass skip untouched tiles.

Judge this by measurement, not by intuition. A full SIMD reduction over 256
contiguous `u8` values is roughly 16 AVX2 instructions. The branch and mask
logic to skip tiles may cost more than the reduction it saves.
**Recommendation: do not build sub-chunk masks first. Build them only if
profiling shows chunk recompute is hot.** Record this as an open question.

### 6.4 Parallel update

The pattern:

```
for level in 0..=1 {
    dirty[level].drain_into(&mut indices);   // sorted, deterministic
    indices.par_iter().for_each(|&cell| {
        recompute(level, cell);              // disjoint writes: no locks
        dirty[level + 1].mark(parent_of(level, cell));
    });
}
```

Three properties make this correct.
- **Disjoint writes.** Each worker writes only its own cell. No mutex, no
  false sharing if summaries are 64-byte aligned.
- **Idempotent parent marking.** `fetch_or` from many workers onto the same
  parent word is safe and order-independent. Contention is low because the
  write happens once per dirty chunk, not once per changed tile.
- **Deterministic input order.** `drain_into` yields ascending indices, so
  the work set is identical on every run regardless of thread scheduling.

Level 0 is not in this loop. Tile writes mark chunks dirty directly, from
the command apply step.

---

## 7. Spatial query acceleration with the pyramid

### 7.1 Three-valued hierarchical descent

Evaluate a selector predicate against a cell summary and return one of three
answers.

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Verdict { None, All, Some }

pub trait Predicate {
    /// Conservative test against a summary. Must never return None or All
    /// when the true answer is Some. Returning Some is always safe.
    fn test_cell(&self, c: &CellSummary) -> Verdict;
    fn test_tile(&self, t: TileRef) -> bool;
}
```

Descent:
- `None` : prune the whole subtree. Zero further work.
- `All`  : accept the whole subtree. Emit the cell's tile range without
  descending. **This is the larger win, and most designs forget it.**
- `Some` : descend to the children; at L0, test each tile.

### 7.2 What summary fields make pruning work

The rule: **for each field a selector can filter on, store a lower bound and
an upper bound.** One bound alone gives you `None` pruning but never `All`
acceptance.

| Predicate form | Fields needed | `None` test | `All` test |
|---|---|---|---|
| `field > k` | `min`, `max` | `max <= k` | `min > k` |
| `field in [a,b]` | `min`, `max` | `max < a or min > b` | `min >= a and max <= b` |
| `terrain == t` | `hist[t]`, `tile_count` | `hist[t] == 0` | `hist[t] == tile_count` |
| `faction in M` | `or_mask`, `and_mask` | `or_mask & M == 0` | `and_mask & M == M` |
| `has any unit` | `unit_count` | `unit_count == 0` | — |
| `owner == p` | `owner_hist[p]`, `tile_count` | `== 0` | `== tile_count` |

Note that the popcount vector from section 5.3 gives you both the OR mask
(`count > 0`) and the AND mask (`count == total`) for free. That is a second
reason to store popcounts rather than a bare mask.

A concrete summary sketch:

```rust
#[repr(C, align(64))]
pub struct CellSummary {
    pub tile_count:    u32,
    pub unit_count:    u32,
    pub elevation_sum: i64,
    pub elevation_min: MinTracked,   // i16 + u16
    pub elevation_max: MaxTracked,   // i16 + u16
    pub terrain_hist:  [u16; 16],    // 32 B: terrain presence, dominance, All test
    pub faction_pop:   [u16; 16],    // 32 B: OR mask, AND mask, counts
}
// 4 + 4 + 8 + 4 + 4 + 32 + 32 = 88 bytes -> pad to 128.
// 65536 L1 cells * 128 B = 8 MB. 256 L2 cells: negligible.
```

### 7.3 Failure mode: scattered predicates

Hierarchical descent helps when matches are **clustered**. If a predicate
matches 5% of tiles scattered uniformly, then almost every L1 cell returns
`Some`, and you pay the full descent plus the summary reads on top of a
linear scan. You are then slower than a flat SIMD scan of L0.

Mitigation: add a cost model. Evaluate the predicate at L2 first. If the
fraction of `Some` verdicts exceeds a threshold (start at 50%), abandon the
descent and run a flat SIMD scan over the L0 arrays. A flat scan of 16.7M
`u8` values with AVX2 runs at roughly 16 GB/s, so about 1 ms. That is the
worst case you must not exceed, and it is a good fallback.

Record this in the ADR: **the pyramid is an optimisation with a guaranteed
fallback, not the only query path.** Building the flat path first, and the
descent second, is the safer order.

### 7.4 Units versus tiles

Tile terrain changes rarely. Unit positions change every frame. If both feed
the same pyramid, unit motion dirties every chunk every frame and the terrain
aggregates get recomputed for nothing.

**Recommendation: two pyramids over the same cell grid.** A tile pyramid with
a slow dirty cadence, and a unit pyramid rebuilt or delta-updated every
frame. They share the index math and the dirty machinery, so the extra code
is small. The unit pyramid should be delta-only (counts and histograms) so
it never needs a recompute pass, which the section 5.3 popcount trick makes
possible.

---

## 8. Open questions for the ADR author

1. **Fanout.** 16 or 32 per level? 16 gives a useful L2 (256 cells); 32 does
   not (16 cells). This also fixes the storage chunk size, so decide it
   before anything else.
2. **World shape.** Rhombus or rectangle? A rhombus lets you store raw axial
   and drop the offset conversion. A rectangle needs the offset index but
   wastes no memory and prunes better.
3. **Are aggregates integer-only?** If yes, parallel folds are deterministic
   for free. If any float aggregate is needed, fold order becomes a
   correctness constraint, not a performance one.
4. **Sub-chunk dirty masks.** Needed, or is a full chunk recompute cheap
   enough? Do not decide this without a benchmark.
5. **Which fields are min/max?** Every min/max field costs a possible
   rescan. Check whether the field's value distribution makes the
   extremum-count fast path effective, or whether a bucketed histogram
   would serve the actual query better.
6. **The `All` verdict.** Does the selector evaluation API have a way to
   return "all tiles in this range" without materialising the tile list? If
   not, the largest pruning win is unavailable, and the summary AND masks
   are wasted memory.
7. **Rendering L1 and L2 cells.** Rhombus, centroid hexagon, or merged blob
   outline? This is the only place the hierarchy choice is visible, so it
   deserves a mock-up before the ADR is final.

---

## Sources

[^rbg]: Amit Patel, "Hexagonal Grids", Red Blob Games.
  <https://www.redblobgames.com/grids/hexagons/>
  Companion implementation guide:
  <https://www.redblobgames.com/grids/hexagons/implementation.html>

[^h3]: Uber H3, "Indexing" (highlights).
  <https://h3geo.org/docs/highlights/indexing>
  Quote: "every hexagonal cell ... has seven child cells below it in this
  hierarchy"; "logical containment in the index is exact" while geographic
  containment across resolutions is approximate. See also
  <https://h3geo.org/docs/core-library/overview/>

[^eisen]: "Designing aperture 2^2k hexagonal grids and their indexing as
  factor rings of Eisenstein integers", Theoretical Computer Science.
  <https://www.sciencedirect.com/science/article/pii/S0304397523005704>

[^rhombille]: Gamelogic, "What is a rhombille grid?"
  <https://gamelogic.co.za/2013/06/27/what-is-a-rhombille-grid/>
  Note: three rhombi group into one hexagon, with coordinates derived from
  hex coordinates plus an orientation index.

[^hisparse]: `hi_sparse_bitset` crate.
  <https://crates.io/crates/hi_sparse_bitset> and
  <https://lib.rs/crates/hi_sparse_bitset>
  Reports lower memory and faster inter-set operations than `hibitset`,
  which allocates by maximum index.

[^roaring]: Roaring Bitmaps. <https://roaringbitmap.org/>,
  Rust port <https://github.com/RoaringBitmap/roaring-rs>,
  C implementation with SIMD <https://github.com/RoaringBitmap/CRoaring>

Additional reference for aperture-4 hexagonal DGGS and rhombic structures:
<https://www.tandfonline.com/doi/full/10.1080/17538947.2024.2316112>
