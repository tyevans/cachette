# Influence Maps

Research report 09 for the foundational architecture decision record.

## 0. Context

Cachette is a world simulation engine. The core is Rust. The control plane
is Python. The engine simulates a hex world at three levels of detail.
Level 0 holds 16,777,216 individual tiles in a 4,096 by 4,096 grid. Level 1
holds 65,536 cells in a 256 by 256 grid. One level 1 cell summarises 256
tiles in a 16 by 16 square. Level 2 holds 256 blocks in a 16 by 16 grid.
One level 2 block summarises 256 level 1 cells, which is 65,536 tiles. The
target scale is 16,777,216 tiles, one million units and up to 1,024
factions.[^1]

An influence map is a scalar field over the grid. It answers a question of
the form "how dangerous is this place" or "how valuable is this place". The
draft decision record budgets 8 influence maps of 65,536 cells for each
faction, at about 2 MiB for each faction.[^2]

Research report 08 removed fog of war as the binding per-faction cost. It
reduced fog from 21.0 MB for each faction to 0.95 MB for a typical rendered
faction. It then named per-faction influence maps as the new binding cost,
at about 2 GB for 1,024 factions, and asked for this report.[^3]

This report corrects that figure, then removes the cost.

### 0.1 The six findings

**Finding 1. The 2 MiB figure is too low, not too high.** Eight planes of
65,536 cells at 4 bytes for each cell is 2.00 MiB exactly. The arithmetic
holds. The 4-byte cell is a float in the source analysis, and the project
bans floats from simulated and aggregated state.[^1] An iterated diffusion
also needs a second plane to read from while it writes. The honest figure
for the design as written is **4.00 MiB for each faction**, and 8.00 MiB if
the cell takes the `Accum` type. At 1,024 factions that is 4.00 GiB, not
2 GB.

**Finding 2. Six of the eight planes are not per-faction.** Terrain value,
resource potential, movement conductance and pollution do not depend on the
observer. Threat and opportunity are not stored quantities at all. They are
masked sums over the military presence of other factions, and the diplomacy
relation plane already supplies the mask. **Two planes for each faction
remain: military presence and economic presence.**

**Finding 3. The cell is a `u8`, and saturating addition is a valid
monoid.** Every consumer of an influence map compares values. No consumer
reads an absolute magnitude. A `u8` cell at a fixed reference scale gives
4.4 octaves of usable gradient, which covers the decision range. Saturating
unsigned addition satisfies `sat(sat(a+b)+c) = sat(a+b+c)`, so it is exactly
associative and commutative. It meets the aggregation invariant with no
special case.

**Finding 4. Threat is derived by complement, not by summation.** Store one
shared plane that holds the sum of the military presence of every faction.
The threat to one faction is that shared plane minus the presence planes of
its own alliance. An alliance holds about four members, so the derivation
costs four plane subtractions instead of sixty-two plane additions.

**Finding 5. Level 1 is the correct resolution, and the rule that fixes it
is quantitative.** A level 1 cell is 16 tiles across. A smooth field
sampled at pitch `h` with decay length `lambda` carries a sampling error of
order `(h / lambda)^2`. Level 1 is adequate when the decay length is at
least 64 tiles, which is four cells. Below that length the quantity is a
local combat query, not an influence map, and the neighbour search answers
it.

**Finding 6. The new total is 2.90 MiB at 1,024 factions.** The reduction
against the corrected 4.00 GiB is 1,414 times. Fog of war becomes the
binding per-faction cost again, by a factor of 34 for a rendered faction.

---

## 1. Terms

**Plane.** One scalar field over the 65,536 level 1 cells. One plane holds
one concern for one faction, or one concern shared by all factions.

**Cell value.** A `u8`. The value 255 means one reference unit of
influence. The reference unit is a compile-time constant.

**Source.** A unit, a structure or a settlement that injects influence into
the cell that holds it.

**Decay length.** The distance over which a field falls by a factor of two.
This report states it in level 1 cells unless it names tiles.

**Block.** One level 2 block. It covers 256 level 1 cells. A dense byte
payload for one block is 256 bytes.

**Conductance.** A `u8` for each level 1 cell. It states how freely
influence crosses that cell. A mountain range carries a low conductance.

**Dominant pair.** A record for each level 1 cell. It names the faction
with the highest military presence, that presence, the faction with the
second highest presence, and that presence.

**Tier.** A class of faction. Report 08 defines three tiers for fog:
rendered, active and passive. This report reuses the same three tiers.[^3]

---

## 2. The corrected cost

### 2.1 The stated figure, checked

| Quantity | Value |
|---|---|
| Planes for each faction, as budgeted | 8 |
| Level 1 cells | 65,536 |
| Bytes for each cell, as budgeted | 4 |
| Product | 2,097,152 B = **2.00 MiB exactly** |

The arithmetic is correct. Two corrections apply to the inputs.

### 2.2 Correction 1 — the 4-byte cell is a float

The algorithm report describes each plane as 16,384 floats.[^4] The project
bans floating point from simulated and aggregated state, because float
addition is not associative and an aggregate must combine exactly in any
order.[^1] A diffusion result feeds the level 1 summary, so it is
aggregated state. The cell must be an integer type.

The legal replacements are `Fix32`, which is an `i32` in Q16.16, and
`Accum`, which is an `i64`. `Fix32` keeps the 4-byte cell and the 2.00 MiB
total. `Accum` doubles the total to 4.00 MiB.

### 2.3 Correction 2 — the cell count in the source analysis is stale

The algorithm report states 16,384 level 1 cells and 64 KB for each
plane.[^4] The decision record fixes the fanout at 16 over three levels,
which gives 65,536 level 1 cells for a 4,096 by 4,096 map.[^2] The decision
record's own memory table uses 65,536. The correct dense cost for one plane
at 4 bytes is therefore 256 KiB, not 64 KB. The 2.00 MiB per-faction total
already uses the correct cell count. The per-plane figure in the algorithm
report does not.

### 2.4 Correction 3 — the diffusion needs a second plane

An iterated stencil that stays deterministic under parallel execution must
read one plane and write another. A Jacobi sweep does this. A Gauss-Seidel
sweep reads and writes one plane, and its result depends on the sweep order,
which changes when the worker count changes. Gauss-Seidel is therefore
banned by the determinism contract.[^1]

If each faction holds its own second plane, the per-faction cost doubles
again. Section 6.4 shows that the second plane belongs to the worker, not to
the faction, so this cost becomes a small fixed total. The point stands
against the design as written.

### 2.5 The corrected table

| Design as written | Cell type | Planes | Double buffer | For each faction | At 1,024 |
|---|---|---|---|---|---|
| As budgeted | float, banned | 8 | no | 2.00 MiB | 2.00 GiB |
| Minimum legal | `Fix32` | 8 | no | 2.00 MiB | 2.00 GiB |
| Legal and deterministic | `Fix32` | 8 | yes | **4.00 MiB** | **4.00 GiB** |
| Pyramid accumulator type | `Accum` | 8 | yes | 8.00 MiB | 8.00 GiB |

**Report 08 understated the problem by a factor of 2 to 4.** The correct
starting figure is 4.00 MiB for each faction and 4.00 GiB at 1,024
factions.

---

## 3. The consumers

Enumerate the consumers before choosing the representation. The consumer set
fixes the resolution, the precision and the update rate.

### 3.1 The consumer table

| Consumer | Question it asks | Reads | Resolution | Cadence |
|---|---|---|---|---|
| C1. Threat assessment | How dangerous is this region for me | threat, derived | L1 | 8 to 32 ticks |
| C2. Territory and borders | Which faction controls this cell | dominant pair | L1 | on event |
| C3. Path cost modifier | Which route avoids danger | threat, derived | L1 | 8 to 32 ticks |
| C4. Target selection | Which enemy concentration do I attack | dominant pair, threat | L1 | 8 to 32 ticks |
| C5. Economic gradient | Where is supply, where is demand | economic presence | L1 or L2 | 64 ticks |
| C6. Migration and settlement | Where should a new settlement go | value, dominant pair | L2 | 64 to 256 ticks |
| C7. Contested detection | Do two factions both hold this cell | dominant pair | L1 | on event |
| C8. Python observation | An array for a controller or a learner | any plane | L1 or L2 | on delivery |

### 3.2 The property that all consumers share

**Every consumer compares values. No consumer reads an absolute
magnitude.** C1 thresholds. C2, C4 and C7 take an argument of the maximum.
C3 and C5 follow a gradient sign. C6 ranks candidate cells. C8 hands an
array to a controller that was trained on that array.

Three consequences follow.

1. The cell may be quantised hard. A quantisation that preserves the order
   of two cells preserves the answer.
2. The absolute scale may be a fixed constant. No consumer needs the field
   to be normalised.
3. A monotone saturation at the top of the range is safe. A cell at the
   maximum is "as dangerous as the representation records", and every
   consumer treats it that way.

### 3.3 Which consumers need a per-faction plane

C2, C6 and C7 read a shared structure only. They ask which faction is
strongest, not how strong one named faction is. The dominant pair answers
all three.

C1, C3 and C4 read a per-faction threat field. Section 5.3 derives that
field without storing it.

C5 reads the economic presence of one named faction. This is the only
consumer that needs a stored, private, per-faction plane other than military
presence.

**The stored per-faction plane count is therefore 2, not 8.**

---

## 4. Resolution

### 4.1 The sampling rule

Influence is a smooth, diffusive quantity. Represent it on a grid of pitch
`h`. Let `lambda` be the decay length of the field in the same units. The
relative error of the sampled field against the continuous field is of order
`(h / lambda)^2`, because the leading truncation term of a second-order
stencil is quadratic in the pitch.

| Level | Pitch in tiles | Error at lambda = 256 tiles | at 64 | at 16 |
|---|---|---|---|---|
| L0 | 1 | 0.002% | 0.02% | 0.4% |
| L1 | 16 | 0.4% | 6.3% | 100% |
| L2 | 256 | 100% | — | — |

**Adopt the rule: use level 1 when the decay length is at least 64 tiles,
and use level 2 when it is at least 1,024 tiles.**

### 4.2 What the rule decides

Military presence and economic presence both have decay lengths of hundreds
of tiles. A faction's threat projection reaches as far as its armies can
march in the time an opponent needs to react. Level 1 satisfies the rule for
both.

Migration and settlement pressure change over hundreds of ticks and act over
thousands of tiles. Level 2 satisfies the rule for both, at 256 cells, which
is 256 bytes for one plane.

**Level 0 is never correct for an influence map.** A quantity with a decay
length of a few tiles is a local query. The batched nearest-neighbour sweep
over the sorted unit index answers it directly, at a cost the record already
budgets.[^4] Building a 16.7-million-cell field to answer a question about
seven tiles is the error that the record's full-map-pass rule already
forbids: a full pass over level 0 costs at least 3.3 ms of wall time, and
the record allows two or three of those for each tick.[^2]

### 4.3 The error that level 1 introduces, by consumer

| Consumer | Tolerance | Verdict at L1 | Verdict at L2 |
|---|---|---|---|
| C1. Threat | ordinal over regions | adequate | too coarse |
| C2. Borders | a border is 16 tiles wide at L1 | adequate; refine at the boundary | too coarse |
| C3. Path cost | the portal graph is already an L1 structure | exact match | too coarse |
| C4. Target selection | picks a region, not a tile | adequate | marginal |
| C5. Economic | already an average | adequate | adequate |
| C6. Migration | already an average | adequate | adequate |
| C7. Contested | defined on a cell | exact match | too coarse |
| C8. Python | consumer chooses | adequate | adequate |

C2 needs one note. A level 1 border places the boundary to within 16 tiles.
Refine the border to level 0 only inside the cells that the boundary
crosses. The multi-source search that produces the border already works this
way.[^4] The border cell count is proportional to the perimeter, which is
about 1,000 cells for a large faction, so the refinement covers 256,000
tiles rather than 16.7 million.

---

## 5. Representation

### 5.1 The cell type and the fixed-point scale

**Use a `u8` for each cell. The value 255 represents one reference unit of
influence. The reference unit is a compile-time constant named
`INFLUENCE_UNIT`.**

This is a Q0.8 unsigned fixed-point value against a fixed reference. It is
not a Q16.16 value. State the reason, because the project uses Q16.16
everywhere else.[^2] Q16.16 exists so that a position and a stat share one
scale across many arithmetic stages. An influence cell enters no such
pipeline. It is produced by one kernel and consumed by comparison. The
narrow type buys a factor of 4 in size and a factor of 4 in vector lanes.

Reject a per-plane exponent, which is block floating point in integer
clothing. The exponent would change when the maximum changes, and a
rescaling changes every stored value. That makes the plane sensitive to the
history of its own maximum, which is a determinism hazard for no gain.

**The usable gradient range.** Let the decay be 8 cells for each halving.
Adjacent cells then differ by a factor of `2^(-1/8)`, which is a drop of 8.3
percent. A `u8` resolves an 8.3 percent step down to a value of about 12.
The usable range is therefore `log2(255 / 12)`, which is 4.4 octaves, or
about 35 cells of reach. At 16 tiles for each cell that is 560 tiles.
**That reach exceeds the operating radius of any consumer in section 3.**

**Saturation is a valid monoid.** Define the combine as saturating unsigned
addition at 255. For any `a`, `b` and `c`, `sat(sat(a + b) + c)` and
`sat(a + sat(b + c))` both equal `min(a + b + c, 255)`. The operation is
exactly associative, commutative, and has the identity 0. It therefore
satisfies the aggregation invariant as a case (a) quantity in every respect
except one: it has no inverse above the saturation point, so it is not a
group there.[^2] Handle this the way the record already handles a bitwise
fold: declare an explicit recompute path. The recompute path is the plane
update itself, which runs on a fixed cadence, so the witness that bounds the
recompute rate is the cadence.

**Widen at the fold.** Accumulate a `u8` plane into a level 2 summary as an
`Accum`, which is an `i64`. A `u8` field summed over 65,536 cells reaches
16,711,425, which fits in a `u32`. The record's accumulator-width rule
forbids relying on that margin, and `i64` arithmetic is free on the
target.[^2]

### 5.2 The container

Reuse the block index of the fog container, with a simpler leaf set.[^3]

```rust
enum Plane {
    Empty,                    // every cell in this block is 0
    Dense(Arc<[u8; 256]>),    // 256 L1 cells, 4 cache lines
}

struct InfluencePlane {
    blocks: [Plane; 256],     // index is the L2 block number
}
```

Three points differ from the fog container, and each has a reason.

**There is no `Array` leaf.** A fog `Array` leaf stores a `u16` offset for
each member, against one bit for each cell in a `Bits` leaf, so it wins
below about 6 percent density. An influence cell already costs one byte, and
an offset-plus-value pair costs two bytes, so an array form wins only below
50 percent occupancy inside a block. Diffusion fills the blocks that it
touches, so occupancy inside a touched block is high. **Two leaf forms are
correct here. Four are not.**

**There is no `Full` leaf.** `Full` encodes one repeated value. A fog layer
has one non-zero value. An influence plane has 255 of them, and a block of
uniform influence is not a case that occurs.

**A block is 256 bytes, not 8,192.** The block index is the level 2 block
number in both containers, so a dirty scan is four `u64` words in both. The
payload is smaller because a level 1 block holds 256 cells, not 65,536
tiles.

The header cost is 16 bytes for each block, so 4,096 bytes for each plane.

### 5.3 Bounded support is real, and it is quantifiable

A fog layer is sparse because a faction sees a small part of the map. An
influence plane appears not to be sparse, because diffusion spreads. Test
that.

Let the decay be a factor `d` for each cell. A source at the maximum value
of 255 falls below the quantisation floor of 1 at radius `r`, where
`255 * d^r < 1`. Solving gives `r = ln(255) / -ln(d)`.

| Decay for each cell `d` | Halving length | Support radius `r` in cells | Cells in support |
|---|---|---|---|
| 0.70 | 1.9 | 15.5 | 800 |
| 0.80 | 3.1 | 24.8 | 2,050 |
| 0.90 | 6.6 | 52.6 | 9,220 |
| 0.95 | 13.5 | 108 | 38,800 |

**The support is bounded because the representation is quantised.** At a
halving length of 3 to 7 cells, a single compact source cluster covers 2,000
to 9,000 of 65,536 cells, which is 3 to 14 percent.

Translate this into blocks. A block is a 16 by 16 square of cells. A support
disc of radius 25 cells spans 50 cells, so it touches about 16 blocks. A
faction with several clusters touches more. Measured against the whole map
of 256 blocks:

| Faction state | Blocks touched | Bytes for one plane |
|---|---|---|
| Early, one cluster | 16 | 8.1 KiB |
| Mid, several clusters | 40 | **14.0 KiB** |
| Late empire | 120 | 34.0 KiB |
| Whole map | 256 | **68.0 KiB** |

Each figure includes the 4,096-byte header. **The worst case for one plane
is 68.0 KiB.** That is the whole cost of the graceful end, and it is 3.8
times smaller than one dense `Fix32` plane at 256 KiB.

### 5.4 The shared planes

Six structures do not depend on the faction. Store one of each.

| Shared structure | Cell type | Bytes for each cell | Size | Serves |
|---|---|---|---|---|
| Terrain value | `u8` | 1 | 64 KiB | C6 |
| Resource potential | `u8` | 1 | 64 KiB | C5, C6 |
| Movement conductance | `u8` | 1 | 64 KiB | the diffusion kernel |
| All-faction military sum | `u16` | 2 | 128 KiB | C1, C3, C4 |
| Dominant pair | see below | 6 | 384 KiB | C2, C4, C6, C7 |
| L2 mirrors of all of the above | | | 3 KiB | C6, coarse pruning |

The all-faction military sum needs a `u16`. The maximum is 63 addressable
factions at 255 each, which is 16,065, and that exceeds a `u8`. It fits a
`u16` with room for 194 more contributors.

The dominant pair holds four fields for each cell: a `u16` for the leading
faction identifier, a `u8` for its presence, a `u16` for the second faction
identifier, and a `u8` for its presence. That is 6 bytes. Store it as four
parallel arrays, not as an array of structures, so that a scan of the margin
reads only the two `u8` arrays.

### 5.5 Threat by complement

Do not store a threat plane for each faction. Derive it.

```
threat_to(me)[c] = all_military[c]
                 - sum over f in friendly(me) of military[f][c]
```

`friendly(me)` is the own bit of the faction, unioned with its alliance row
in the diplomacy relation plane. The relation plane is three `u64` rows for
each faction, which is 1,536 bytes in total and stays in the level 1 data
cache for the whole tick.[^3]

The naive form sums the presence of every hostile faction. At 62 hostile
factions that is 62 plane additions, or 4.06 million byte operations. The
complement form subtracts the presence of every friendly faction. An
alliance holds about four members, so it is 4 plane subtractions, or 262,144
byte operations. **The complement form is 15 times cheaper, and the saving
grows as the faction count grows.**

The complement form is exact, not approximate, provided that the shared sum
is the true sum. Compute the shared sum as a `u16` accumulation, which does
not saturate, and clamp only at the point of use.

Two properties follow, and both are useful.

**A faction outside the mask domain still contributes.** The shared sum adds
magnitudes. It does not need a mask bit. A passive faction whose presence
enters the shared sum is therefore counted, even though it owns no
addressable bit. Report 08 reserves bit 63 as an overflow bit for exactly
those factions.[^3] The complement form needs the overflow bit only for the
friendly set, not for the hostile set.

**A non-addressable faction reads as hostile.** The subtraction removes only
friendly factions, and the friendly set comes from the masked relation
plane, which cannot name a non-addressable faction. A passive faction that
should count as an ally is therefore counted as a threat. **The error is
one-directional and it over-reports danger.** That is the safe direction for
every consumer in section 3. State the limit in the interface. A faction
that needs a named ally outside the mask domain must be promoted to an
addressable tier.

### 5.6 What the dominant pair cannot answer

The dominant pair is a compressed alternative to N separate planes. State
its limits precisely, because the limits are what force the per-faction
planes to survive.

| Query | Dominant pair | Verdict |
|---|---|---|
| Who controls this cell | leading identifier | answers |
| Is this cell contested | leading presence minus second presence | answers |
| Which two factions meet here | both identifiers | answers |
| How strong is the leader | leading presence | answers |
| **What is my presence here, when I am third or lower** | not stored | **fails** |
| **What is the total presence of my alliance here** | not stored | **fails** |
| **How many factions have any presence here** | not stored | **fails** |
| **Which way does my own field rise, where I am not the leader** | not stored | **fails** |

The fourth failure is the decisive one. A faction that wants to expand needs
the gradient of its own field in cells that another faction currently leads.
That is precisely where the dominant pair holds no record of it. **The
dominant pair is a shared summary. It is not a replacement for the
per-faction military plane.** It removes the need for a stored territory
plane, a contested plane and a border plane, which is three of the eight
budgeted planes.

### 5.7 Transposing the faction dimension

Report 08 evaluated a transposed grid for fog, where each tile holds a `u64`
faction mask, and rejected it at level 0.[^3] Test the same idea for
influence.

A transposed influence grid holds one value for each faction in each cell.
At `u8` and 63 addressable factions that is 63 bytes for each cell, so
4.03 MiB at level 1. Compare that against 63 separate planes at 14.0 KiB
typical, which is 0.86 MiB, and 63 separate planes at 68.0 KiB worst, which
is 4.19 MiB.

| Form | Typical | Worst | Fixed |
|---|---|---|---|
| 63 separate sparse planes | 0.86 MiB | 4.19 MiB | no |
| One transposed grid, 63 `u8` for each cell | 4.03 MiB | 4.03 MiB | yes |

**The transpose loses on typical size by 4.7 times and ties on the worst
case.** It also loses on update traffic for the same structural reason that
report 08 identified for fog: it moves 63 bytes of a cache line to update
one byte of information. Its only gain is the all-faction query for one
cell, and section 5.6 shows that the dominant pair answers the useful part
of that query for 384 KiB.

**Reject the transposed influence grid. Adopt the dominant pair instead.**

---

## 6. Computation

### 6.1 The five candidates

Cost each method for one plane of 65,536 cells. The target is a Graviton
core. NEON is baseline on aarch64, so one 128-bit register holds 16 `u8`
lanes with no runtime feature check.[^5] Assume about 2 GHz and an achieved
rate of two vector operations for each cycle on a simple in-cache kernel.[^6]

**Method 1. Iterated Jacobi stencil.** Each cell becomes a weighted blend of
itself and its six hex neighbours, scaled by the local conductance. One
iteration costs 65,536 cells at 7 loads and one store. In vector form that
is 4,096 lanes-groups at about 9 operations, so about 37,000 vector
operations, or about 18 microseconds. Influence propagates one cell for each
iteration, so reaching 32 cells costs 32 iterations and about 590
microseconds.

**Method 2. Separable integer recursion.** A hex grid has three axis
directions. Run a first-order recursive filter along each axis, forward and
backward, for six linear passes. The recursion is
`y[i] = x[i] + ((a * y[i-1]) >> 16)`, with an `i32` state and an `i64`
intermediate. This propagates an exponential decay across the whole line in
one pass, so the cost does not depend on the range. Six passes over 65,536
cells is 393,216 cell updates. The recursion is sequential along one line
and there are 256 independent lines for each axis, so interleave 16 lines in
one NEON register. That gives about 25,000 vector operations, or about 12
microseconds.

**Method 3. Two-grid pyramid propagation.** Restrict the source plane from
level 1 to level 2, solve at level 2, prolong the result back to level 1,
and run a few Jacobi iterations at level 1 to remove the interpolation
error. The level 2 grid holds 256 cells, so its solve is free. Restriction
and prolongation are one pass each. With 8 correcting iterations the total is
about 10 passes, or about 180 microseconds.

**Method 4. Multi-source bucketed Dijkstra with source tags.** Seed one
bucket queue with every source cell of every faction, tagged with the
faction identifier. One pass produces, for each cell, the nearest source and
its distance, over all factions at once. The level 1 graph holds 65,536
nodes and about 393,216 directed edges. With integer buckets this is about
1.5 million operations, or about 0.7 milliseconds, **once for every
faction**.

**Method 5. Closed-form falloff summed over sources.** For each source, add
a tabulated falloff to every cell inside the support radius. At a support
radius of 25 cells one source touches about 2,050 cells. At 200 source
clusters for one faction that is 410,000 additions, or about 200
microseconds, and it grows linearly with the source count.

### 6.2 The comparison

| Method | Complexity | Traffic for each pass | Time for one plane | Range dependent | Respects terrain | Deterministic in integers |
|---|---|---|---|---|---|---|
| 1. Jacobi, 32 iterations | O(N r) | 128 KiB | 590 us | yes | yes | yes |
| 1b. Jacobi, 8 iterations | O(N r) | 128 KiB | 150 us | yes | yes | yes |
| 2. Separable recursion | O(N) | 128 KiB | **12 us** | no | no | yes |
| 3. Two-grid pyramid | O(N) | 132 KiB | 180 us | no | yes | yes |
| 4. Multi-source Dijkstra | O(N) with buckets | 512 KiB | 700 us, all factions | no | yes | yes, with a tiebreak |
| 5. Closed-form scatter | O(k r^2) | source dependent | 200 us at k=200 | no | no | yes |

Every method converges deterministically in integer arithmetic, with two
conditions.

**Condition 1.** The Jacobi and the recursive kernels must use a truncating
right shift, not a rounding divide by a variable. A truncating shift is
exact and reproducible on every target.

**Condition 2.** The Dijkstra must break ties on the ordered tuple of
distance, faction identifier and cell index. A bucket queue with that
tiebreak has one defined output. Never iterate a hash container.[^1]

### 6.3 The recommendation

**Use method 2 for the economic presence plane. Use method 1b, seeded by
method 3, for the military presence plane. Use method 4 for the dominant
pair, on an event trigger only.**

The split follows from one property. The separable recursion is 12 times
faster than the Jacobi form and its cost does not grow with the range, but
it cannot respect a terrain barrier. Its recursion runs along a straight
line and a mountain range does not stop it. A military threat field must
respect a barrier, because a defended pass is the whole point of a
chokepoint. An economic potential field does not, because trade and supply
are already routed by the portal graph rather than by the field.

The military plane therefore pays for conductance. Reduce its cost by
seeding, not by cutting iterations blindly. Restrict the sources to level 2,
solve there, prolong back to level 1, and then run 8 Jacobi iterations. The
prolongation already carries the far field, so the 8 iterations only correct
the near field within 8 cells. **This is the point at which the existing
pyramid earns its place.** The level 2 grid, the restriction path and the
dirty bitsets all exist for the summary pyramid.[^2] The influence solve
reuses them without new machinery.

Method 5 remains useful in one place. A passive faction holds fewer than
about 20 source clusters, so the closed-form scatter costs under 20
microseconds and needs no stored plane at all. Section 7.3 uses it.

### 6.4 The double buffer belongs to the worker

A Jacobi iteration reads one plane and writes another. Do not give each
faction a private second plane. Give each worker two scratch planes of
65,536 bytes each and ping-pong between them.

The update processes one faction at a time on one worker. Different workers
process different factions, so the outputs are disjoint and the update needs
no atomic operation. This is the disjoint-output rule that the weak ARM
memory model requires.[^5]

At 8 workers the scratch total is 8 times 128 KiB, which is 1.00 MiB. It is
fixed and it does not grow with the faction count.

### 6.5 Determinism of the update

Five rules make the update deterministic.

- Process factions in ascending faction identifier. That is the sort key.
- Process blocks in ascending level 2 block number inside one plane.
- Give each worker a disjoint set of factions. No worker writes a plane that
  another worker reads in the same phase.
- Fix the iteration count as a constant. Never stop on a convergence test
  that reads a running maximum, because the maximum depends on the order in
  which the workers finish.
- Break every argument-of-the-maximum tie on the ordered tuple of value,
  faction identifier and cell index.

The fourth rule matters most and it is easy to violate. A relaxation solver
usually stops when the residual falls below a threshold. That test makes the
iteration count depend on the arithmetic, which is fine, but it invites a
parallel reduction of the residual, which is not. **Fix the iteration count
at 8 for the military plane and at 1 for the recursive kernel. A fixed
iteration count is the deterministic form.**

The engine keys every random draw on the tuple of system, frame, entity and
draw.[^1] An influence update draws no random number, so that invariant is
not at risk.

### 6.6 The frame loop

The frame loop splits into a read half and a write half. Phases 1 to 4 read
the world and write only events. Phases 5 to 8 write the world.[^2]

**Run the influence update at the end of phase 8, after fog.** It reads the
unit positions that phase 6 settled and the level 1 summaries that phase 7
produced. It writes only the influence planes.

A selector or a controller that reads an influence value runs in phases 1 to
4. It reads the planes that phase 8 produced on an earlier tick. That is a
lag of at least one tick, and the record already accepts a one-tick lag for
derived data.[^2] Section 7 shows that the cadence makes the real lag longer
than one tick, and that this is acceptable.

Give the module two types. `InfluenceRead` exposes the point query, the
plane views and the dominant-pair views. `InfluenceWrite` exposes the
update. Phase 8 is the only phase that receives `InfluenceWrite`.

### 6.7 Graviton notes

- **Use `u8` lanes.** A 128-bit NEON register holds 16 `u8` lanes against 4
  `i32` lanes. The narrow cell buys a factor of 4 in throughput as well as
  in size.
- **Use the saturating instructions.** `UQADD` and `UQSUB` saturate in
  hardware, so the clamp costs nothing.
- **Align each block payload to 64 bytes.** A block payload is 256 bytes,
  which is exactly 4 cache lines on the target, so two blocks never share a
  line and parallel writes never falsely share.[^5]
- **Make the line size a compile-time constant.** Apple Silicon uses 128
  bytes and development happens there.[^5]
- **Keep the working set in the level 2 cache.** One plane is 64 KiB, one
  conductance plane is 64 KiB and one scratch is 64 KiB. The whole kernel
  working set is under 256 KiB, so it never reaches main memory.
- **Prefer disjoint outputs over atomics.** A relaxed atomic emits a real
  barrier under the weak ARM memory model.[^5] Section 6.4 removes every
  atomic from the update.

---

## 7. Update cadence

### 7.1 The cadence that the resolution already permits

An influence map changes at the speed at which its sources move. A unit
moves about one tile for each tick at a 10 Hz timestep.[^2] A level 1 cell
is 16 tiles across, so a unit changes its cell membership about once in 16
ticks.

**A plane recomputed every 8 ticks is stale by at most half a cell. That
error is below the resolution of the representation, so it is free.** This
is the decisive cadence argument. A finer cadence does not produce a
different answer at level 1; it produces the same answer more often.

### 7.2 The amortised schedule

Do not update every faction on the same tick. Round-robin the factions
across the cadence window, so the per-tick cost is flat.

| Tier | Cadence | Factions | Plane updates for each tick |
|---|---|---|---|
| R, rendered | every 8 ticks | 8 | 8 factions x 2 planes / 8 = 2.0 |
| A, active | every 32 ticks | 55 | 55 x 1 / 32 = 1.7 |
| P, passive | on demand | any | 0 |

Round-robin by the faction identifier modulo the cadence. That order is
fixed and holds no dependence on the worker count.

Two structures update on an event, not on a cadence.

**The dominant pair.** Recompute it when a settlement is founded or
destroyed, when a faction is created or removed, or when an alliance
changes. Those events are rare. Between events, refresh only the cells that
a moved army touches, by comparing the moved faction's presence against the
stored leading and second presence. That is a bounded, local update.

**The all-faction military sum.** Rebuild it once for each cadence window,
after every tier R and tier A plane in that window has settled. One rebuild
costs 63 plane additions, which is 4.06 million byte operations, or about
130 microseconds. At a window of 8 ticks that is 16 microseconds for each
tick.

### 7.3 Dirty-block restriction

Restrict every update to the blocks that changed and to their neighbours.
Diffusion spreads by one cell for each Jacobi iteration, so an 8-iteration
solve on a dirty block also affects the 8-cell margin around it, which
reaches into the neighbouring blocks. Mark a block dirty when a source
inside it moves, and mark its six neighbours as well.

The dirty block count for a mid-game faction is about 12 of the 40 blocks it
occupies, so the restriction cuts the update cost by about a factor of 3.
State this as a reduction, not as a change of the worst case. The worst case
is still every touched block.

The level 2 dirty bitset is 32 bytes and the level 1 dirty bitset is 8 KiB.
Both already exist.[^2]

### 7.4 Staleness, by consumer

| Consumer | Cadence it reads | Staleness | Tolerable |
|---|---|---|---|
| C1. Threat | 8 or 32 ticks | 0.8 s or 3.2 s | yes; an AI decision cycle is seconds |
| C2. Borders | on event | none | yes |
| C3. Path cost | 8 or 32 ticks | 0.8 s or 3.2 s | yes; a route is replanned on arrival at a portal |
| C4. Target selection | 8 or 32 ticks | 0.8 s or 3.2 s | yes; a target commitment lasts longer than that |
| C5. Economic | 64 ticks | 6.4 s | yes; production cycles are minutes |
| C6. Migration | 64 to 256 ticks | 6.4 s to 25.6 s | yes; settlement decisions are rare |
| C7. Contested | on event | none | yes |
| C8. Python | on delivery | as above | the caller chooses the plane |

One consumer needs a warning. C3 modifies a path cost. A stale threat field
can route a column into danger that appeared 3 seconds ago. Mitigate this at
the steering layer, not at the influence layer. The per-unit steering blend
already reacts to local conditions each tick.[^2] The influence field
chooses the corridor. The steering blend avoids the ambush.

### 7.5 Cost of the whole schedule

| Work | Rate | Time for each occurrence | Core-ms for each tick |
|---|---|---|---|
| Tier R military plane, Jacobi 1b seeded | 1.0 for each tick | 150 us | 0.150 |
| Tier R economic plane, separable | 1.0 for each tick | 12 us | 0.012 |
| Tier A military plane, Jacobi 1b seeded | 1.7 for each tick | 150 us | 0.258 |
| All-faction sum rebuild | 1 in 8 ticks | 130 us | 0.016 |
| Threat by complement, 8 factions | 1.0 for each tick | 8 x 8 us | 0.064 |
| Dominant-pair local refresh | each tick | 30 us | 0.030 |
| Dominant-pair full rebuild | on event | 700 us | amortised near 0 |
| **Total** | | | **0.53** |

Apply the dirty-block restriction of section 7.3 and the two Jacobi lines
fall by about a factor of 3, which gives a total near 0.25 core-ms.

**The record's existing budget line for influence maps is 1 to 3 core-ms and
0.1 to 0.3 wall-ms.**[^2] The schedule above fits inside it, with the
restriction and without it. Report the line unchanged, and record the
measured value against it.

The update parallelises across factions with disjoint outputs, but at 0.53
core-ms it is not worth spreading over many workers. Run it on 2 workers.
The wall time is then about 0.27 ms.

---

## 8. Tiering by faction kind

Apply the three tiers that report 08 defines for fog.[^3] A faction belongs
to exactly one tier. The control plane sets the tier at creation and may
change it later.

### 8.1 Tier R — rendered

A human player or a recorded observer watches this faction.

- Military presence: a level 1 sparse plane, `u8`, method 1b seeded by
  method 3.
- Economic presence: a level 1 sparse plane, `u8`, method 2.
- Threat: derived by complement at read.
- Cadence: every 8 ticks.

Typical cost 28.0 KiB. Worst case 136 KiB.

### 8.2 Tier A — active

An artificial-intelligence controller drives this faction. No client draws
its influence field.

- Military presence: a level 1 sparse plane, `u8`.
- Economic presence: **not stored.** A controller reads the shared resource
  potential plane and its own settlement list. That answers the economic
  question without a private field.
- Threat: derived by complement at read.
- Cadence: every 32 ticks.

Typical cost 14.0 KiB. Worst case 68.0 KiB.

### 8.3 Tier P — passive

A minor faction, a neutral power or a frozen faction.

- Military presence: a level 2 plane only, 256 cells at `u8`, so 256 bytes.
  Its sources still enter the shared all-faction military sum, so other
  factions still perceive it.
- Economic presence: not stored.
- Level 1 detail: derived on demand by method 5, the closed-form scatter. A
  passive faction holds under about 20 source clusters, so one derivation
  costs about 20 microseconds.
- Cadence: on demand only.

Cost 256 bytes, fixed.

### 8.4 The tier table

| Tier | Military | Economic | Resolution | Cadence | Typical | Worst |
|---|---|---|---|---|---|---|
| R, rendered | L1 sparse plane | L1 sparse plane | L1 | 8 ticks | 28.0 KiB | 136 KiB |
| A, active | L1 sparse plane | shared plane only | L1 | 32 ticks | 14.0 KiB | 68.0 KiB |
| P, passive | L2 plane, 256 B | none | L2 | on demand | 256 B | 256 B |

### 8.5 Tier promotion

Promotion from tier P to tier A or R must build a level 1 military plane.
Run method 5 over the faction's sources, then run one seeded Jacobi solve.
That costs about 170 microseconds. Promotion is a single-tick operation.

Promotion loses no detail, because the level 1 plane is rebuilt from the
sources rather than refined from the level 2 plane. **This differs from fog,
where a promoted faction loses its level 0 explored detail permanently.**[^3]
An influence plane is a pure function of the current source positions, so it
is always reconstructible. Fog is a history, so it is not.

Demotion from tier R or A to tier P discards the level 1 plane and keeps the
level 2 restriction of it. Demotion is therefore also lossless in the sense
that matters, because promotion rebuilds rather than refines.

---

## 9. Totals

### 9.1 Shared cost, independent of the faction count

| Structure | Size |
|---|---|
| Terrain value plane | 64 KiB |
| Resource potential plane | 64 KiB |
| Movement conductance plane | 64 KiB |
| All-faction military sum, `u16` | 128 KiB |
| Dominant pair, 6 B for each cell | 384 KiB |
| L2 mirrors | 3 KiB |
| Worker scratch, 8 workers x 2 planes | 1,024 KiB |
| **Shared total** | **1,731 KiB = 1.69 MiB** |

The worker scratch is the largest shared item. It is fixed, it does not grow
with the faction count, and section 6.4 explains why it belongs to the
worker rather than to the faction.

### 9.2 Totals at three faction counts

Use the tier split that report 08 recommends: 8 rendered, 55 active, and the
remainder passive.[^3] At 64 factions there is no need for a passive tier,
so use 8 rendered and 56 active.

| Factions | Split | R | A | P | Shared | **Typical** | **Worst** |
|---|---|---|---|---|---|---|---|
| 64 | 8 R, 56 A | 224 KiB | 784 KiB | 0 | 1,731 KiB | **2.68 MiB** | **6.47 MiB** |
| 256 | 8 R, 55 A, 193 P | 224 KiB | 770 KiB | 48 KiB | 1,731 KiB | **2.71 MiB** | **6.45 MiB** |
| 1,024 | 8 R, 55 A, 961 P | 224 KiB | 770 KiB | 240 KiB | 1,731 KiB | **2.90 MiB** | **6.64 MiB** |

The worst column applies the worst case for each tier at the same time.

### 9.3 Against the corrected starting figure

| Faction count | Design as written, corrected | This design, typical | Reduction |
|---|---|---|---|
| 64 | 256 MiB | 2.68 MiB | 96x |
| 256 | 1.00 GiB | 2.71 MiB | 378x |
| 1,024 | 4.00 GiB | 2.90 MiB | **1,414x** |

The total grows by 0.22 MiB between 64 factions and 1,024 factions. **The
design is effectively independent of the faction count**, because 1.69 MiB
of the total is shared and the passive tier costs 256 bytes for each
faction.

### 9.4 What becomes the binding per-faction cost

Compare the per-faction cost of every subsystem after this change.

| Subsystem | Tier R | Tier A | Tier P |
|---|---|---|---|
| Fog of war[^3] | 0.95 MB | 0.48 MB | 8 KB |
| **Influence maps** | **28.0 KiB** | **14.0 KiB** | **256 B** |
| Diplomacy relation rows | 24 B | 24 B | 0 |

**Fog of war becomes the binding per-faction cost again, by a factor of 34
for a rendered faction and 35 for an active faction.** At 1,024 factions,
fog costs 50.7 MB typical and influence costs 2.90 MiB.

The binding architectural limit is unchanged. It is the faction mask width,
which report 08 fixes at 63 addressable factions plus one overflow bit.[^3]
Nothing in this report moves that limit in either direction.

Report 08's second open question asked how influence maps tier. **This
report closes it.**

---

## 10. The faction cap interaction

The project owner intends a maskable 64-bit faction identifier used across
fog, ownership and diplomacy. Report 08 proposes 63 addressable factions
plus bit 63 as an overflow bit, with an unbounded passive class.[^3] That
proposal may not be settled. State how this design behaves under both
models.

### 10.1 Under a hard cap of 64 factions

Every faction is addressable. No passive tier is needed, although it stays
available as a cost reduction for dormant factions. The threat complement
subtracts only friendly factions, and every friendly faction has a bit, so
the derivation is exact for every faction. The total is 2.68 MiB.

### 10.2 Under the two-class model

Sixty-three factions are addressable and the rest are passive. The design
changes in exactly one place, and section 5.5 states it: a passive faction
contributes to the shared military sum but cannot be named in the friendly
mask, so it reads as hostile to every faction. The error over-reports
danger, which is the safe direction.

### 10.3 Under a wider mask, such as 128 bits

The design is unaffected. The all-faction military sum needs a wider cell if
the addressable count exceeds 257, because 257 factions at 255 each exceeds
a `u16`. Widen that one plane to `u32`, which takes it from 128 KiB to
256 KiB. Nothing else changes.

### 10.4 The property to note

**The influence design does not depend on the mask width.** Its per-faction
structures are keyed on the `FactionId`, which is a `u16` and therefore
supports 65,535 factions.[^5] Its shared structures are keyed on the cell.
Only one derivation, the friendly-set subtraction, touches the mask, and it
touches only the small friendly set.

**Do not let the influence design influence the mask-width decision.** That
decision belongs to fog, ownership and diplomacy, and report 08 states the
case for it.

---

## 11. Delivery to Python

The Python control plane must never loop over cells.[^1] The interface
therefore hands over arrays only.

| Call | Returns | Kind |
|---|---|---|
| `military_plane(faction)` | `uint8[65536]` | Copy into a scratch buffer |
| `economic_plane(faction)` | `uint8[65536]` | Copy into a scratch buffer |
| `threat_plane(faction)` | `int16[65536]` | Copy into a scratch buffer |
| `all_military_sum()` | `uint16[65536]` | **Zero-copy view** |
| `terrain_value()` | `uint8[65536]` | **Zero-copy view** |
| `resource_potential()` | `uint8[65536]` | **Zero-copy view** |
| `conductance()` | `uint8[65536]` | **Zero-copy view** |
| `dominant_faction()` | `uint16[65536]` | **Zero-copy view** |
| `dominant_strength()` | `uint8[65536]` | **Zero-copy view** |
| `second_faction()` | `uint16[65536]` | **Zero-copy view** |
| `second_strength()` | `uint8[65536]` | **Zero-copy view** |
| `military_stack(faction_list)` | `uint8[k, 65536]` | Copy into a scratch buffer |

Five notes follow.

The three per-faction calls copy, because the internal form is 256 blocks
and a flat array does not exist in memory. The decision record requires the
documentation to say "copies" wherever the engine gathers.[^2] The expansion
writes 64 KiB, which is about 2 microseconds. That is cheap enough that the
copy needs no defence.

`threat_plane` copies because the plane does not exist until it is asked
for. It is the complement derivation of section 5.5, evaluated into a
scratch buffer. It returns `int16` because a subtraction of a `u16` sum can
go negative for a faction whose alliance holds most of the military presence
in a cell. Clamp at the point of use, not in the engine, so that the sign
carries information about relative strength.

**The eight shared views are genuinely zero-copy.** Each is one flat,
contiguous array that already exists. The control plane can compute
contested cells, border cells and alliance strength entirely with NumPy
operators, with no loop and no per-cell call. Reshape each to a 256 by 256
array at the Python side, at no cost.

`military_stack` exists for reinforcement-learning use. It returns a
`(k, 65536)` array that a caller reshapes to `(k, 256, 256)`. That is
exactly the observation tensor shape that a convolutional policy expects. At
8 factions the copy is 512 KiB and takes about 15 microseconds.

Every view lives inside a scope, as the record's view protection
requires.[^2] A view taken in the Python phase is invalid after the seal
barrier.

---

## 12. Proposed decision text

The text below is a new decision for the draft record, plus two amendments.
Do not apply it to the record. The record is a draft under review.

---

> #### D51. Influence maps are quantised level 1 planes, two for each faction, with a shared dominant-pair summary
>
> Store an influence map as a plane over the 65,536 level 1 cells. Store one
> `u8` for each cell. The value 255 represents one reference unit of
> influence, and the reference unit is a compile-time constant. Do not use a
> per-plane exponent; a rescaling would make a plane depend on the history of
> its own maximum.
>
> **The combine is saturating unsigned addition at 255.** For any `a`, `b`
> and `c`, `sat(sat(a+b)+c)` and `sat(a+sat(b+c))` both equal
> `min(a+b+c, 255)`. The operation is exactly associative, commutative and
> has the identity 0, so it satisfies D16 with no special case. It has no
> inverse above the saturation point, so declare it under D16 case (b) with
> the update cadence as the recompute witness. Accumulate a plane into a
> level 2 summary as `Accum`, which is `i64`, as the accumulator-width rule
> in D4 requires.
>
> **Store two planes for each faction, not eight.** The eight budgeted planes
> divide into three classes.
>
> | Class | Planes | Where they live |
> |---|---|---|
> | Faction-independent | terrain value, resource potential, movement conductance | one shared plane each |
> | Per-faction sources | military presence, economic presence | one plane each, for each faction |
> | Derived at read | threat, opportunity, territory, contested | not stored |
>
> **Derive threat by complement.** Maintain one shared plane holding the sum
> of the military presence of every faction, as a `u16` because 63 factions
> at 255 each reaches 16,065. Then
> `threat_to(me) = all_military - sum(military[f] for f in friendly(me))`.
> The friendly set is the own bit unioned with the alliance row of the
> diplomacy relation plane. An alliance holds about four members, so the
> derivation costs four plane subtractions instead of sixty-two additions,
> which is 15 times cheaper. A faction outside the mask domain contributes to
> the shared sum but cannot be named as friendly, so it reads as hostile. The
> error over-reports danger, which is the safe direction.
>
> **Store one shared dominant-pair summary.** Hold four parallel arrays over
> the level 1 cells: the leading faction identifier as `u16`, its presence as
> `u8`, the second faction identifier as `u16`, and its presence as `u8`.
> That is 6 bytes for each cell, so 384 KiB for all factions together. It
> answers territory, borders, contested cells and target selection, and it
> replaces three of the eight budgeted planes. **It cannot answer "what is my
> presence here" when the asking faction is third or lower, and it cannot
> answer "which way does my own field rise where another faction leads".** A
> faction that expands needs exactly that gradient, so the per-faction
> military plane is not replaceable by the summary.
>
> **Reject a transposed influence grid.** One value for each faction in each
> cell is 63 bytes for each cell, which is 4.03 MiB fixed at level 1, against
> 0.86 MiB typical for 63 sparse planes. It also moves 63 bytes of a cache
> line to update one byte. This is the same finding that D48 records for fog
> at level 0.
>
> **Store each plane as 256 level 2 blocks with two leaf forms.**
>
> | Leaf | Payload | Bytes | Used when |
> |---|---|---|---|
> | `Empty` | none | 0 | every cell in the block is 0 |
> | `Dense` | `[u8; 256]` | 256 | any cell is non-zero |
>
> There is no `Array` leaf, because a cell already costs one byte and an
> offset-plus-value pair costs two. There is no `Full` leaf, because a block
> of uniform influence does not occur. The block index is the level 2 block
> number, which is the same index that D48 uses for fog, so the dirty scan is
> four `u64` words in both. Align each block payload to 64 bytes; 256 bytes
> is exactly 4 cache lines on the target.
>
> **The support is bounded, and the bound follows from the quantisation.** A
> source at 255 with a per-cell decay `d` falls below 1 at radius
> `ln(255) / -ln(d)`. At a halving length of 3 to 7 cells that is 25 to 53
> cells, so one compact cluster covers 3 to 14 percent of the map. A mid-game
> faction touches about 40 of 256 blocks, so one plane costs 14.0 KiB. The
> worst case is every block, which is 68.0 KiB.
>
> **Level 1 is the resolution. Level 0 never is.** A field of decay length
> `lambda` sampled at pitch `h` carries an error of order `(h/lambda)^2`. A
> level 1 cell is 16 tiles, so level 1 is adequate when the decay length is
> at least 64 tiles, and level 2 is adequate at 1,024 tiles. A quantity with
> a decay length of a few tiles is a local query, and the batched neighbour
> sweep of D50 answers it. Refine a border to level 0 only inside the cells
> that the border crosses.
>
> **Compute the economic plane with a separable integer recursion. Compute
> the military plane with a seeded Jacobi solve.**
>
> | Plane | Method | Passes | Time | Respects terrain |
> |---|---|---|---|---|
> | Economic presence | first-order recursion along 3 hex axes, forward and backward | 6 | 12 us | no |
> | Military presence | restrict to L2, solve, prolong, then 8 Jacobi iterations with conductance | about 10 | 150 us | yes |
> | Dominant pair | multi-source bucketed Dijkstra with source tags, all factions at once | 1 | 700 us | yes |
>
> The recursion is `y[i] = x[i] + ((a * y[i-1]) >> 16)` with an `i32` state
> and an `i64` intermediate. Use a truncating shift, never a rounding divide.
> The recursion propagates any range in one pass but cannot stop at a terrain
> barrier, which is why the military plane pays for the Jacobi form. The
> seeding step is a restriction and a prolongation over the existing pyramid,
> so the 8 Jacobi iterations only correct the near field. **This is the reuse
> of the D17 pyramid that the influence solve needs; add no new hierarchy.**
>
> **Fix the iteration count. Never stop on a convergence test.** A residual
> test invites a parallel reduction whose result depends on the worker count.
> Eight iterations for the military plane and one pass for the recursion are
> constants.
>
> **Determinism.** Process factions in ascending `FactionId` and blocks in
> ascending level 2 block number. Give each worker a disjoint set of
> factions, so the update needs no atomic operation, as the weak ARM memory
> model requires under D1. Break every argument-of-the-maximum tie on the
> ordered tuple of value, faction identifier and cell index. Never iterate a
> hash container.
>
> **The Jacobi double buffer belongs to the worker, not to the faction.**
> Give each worker two scratch planes of 65,536 bytes and ping-pong between
> them. At 8 workers that is 1.00 MiB fixed, against 64 KiB for every faction
> if each faction held its own.
>
> **Update on a cadence, amortised, and restricted to dirty blocks.** A unit
> moves about one tile for each tick at 10 Hz and a level 1 cell is 16 tiles,
> so a source changes cell about once in 16 ticks. **A recompute every 8
> ticks is stale by at most half a cell, which is below the resolution of the
> representation.** Round-robin the factions across the cadence window by
> `FactionId` modulo the cadence, so the per-tick cost is flat. Recompute the
> dominant pair on an event, not on a cadence: a settlement founded or
> destroyed, a faction created or removed, or an alliance change. Mark a
> block dirty when a source inside it moves, and mark its six neighbours as
> well, because 8 Jacobi iterations reach 8 cells past the block edge.
>
> **Tier the factions, using the same three tiers as D48.**
>
> | Tier | Military | Economic | Resolution | Cadence | Typical | Worst |
> |---|---|---|---|---|---|---|
> | R, rendered | L1 sparse plane | L1 sparse plane | L1 | 8 ticks | 28.0 KiB | 136 KiB |
> | A, active | L1 sparse plane | shared plane only | L1 | 32 ticks | 14.0 KiB | 68.0 KiB |
> | P, passive | L2 plane, 256 B | none | L2 | on demand | 256 B | 256 B |
>
> A tier P faction derives a level 1 plane on demand by a closed-form falloff
> summed over its sources, which costs about 20 microseconds under about 20
> clusters. Promotion out of tier P rebuilds the level 1 plane from the
> current sources and loses nothing, because an influence plane is a pure
> function of the source positions. This differs from fog, where a promoted
> faction cannot recover its level 0 explored history.
>
> **Shared cost, independent of the faction count.**
>
> | Structure | Size |
> |---|---|
> | Terrain value, resource potential, movement conductance | 192 KiB |
> | All-faction military sum, `u16` | 128 KiB |
> | Dominant pair, 6 B for each cell | 384 KiB |
> | L2 mirrors | 3 KiB |
> | Worker scratch, 8 workers | 1,024 KiB |
> | **Total** | **1.69 MiB** |
>
> **Total cost.**
>
> | Factions | Split | Typical | Worst |
> |---|---|---|---|
> | 64 | 8 R, 56 A | 2.68 MiB | 6.47 MiB |
> | 256 | 8 R, 55 A, 193 P | 2.71 MiB | 6.45 MiB |
> | 1,024 | 8 R, 55 A, 961 P | 2.90 MiB | 6.64 MiB |
>
> The total grows by 0.22 MiB between 64 factions and 1,024 factions, because
> 1.69 MiB is shared and a passive faction costs 256 bytes.
>
> **The frame loop.** Run the update at the end of phase 8 of D28, after fog.
> It reads the unit positions that phase 6 settled and the level 1 summaries
> that phase 7 produced, and it writes only the influence planes. Give the
> module an `InfluenceRead` type and an `InfluenceWrite` type, and give
> `InfluenceWrite` to phase 8 only. A consumer in phases 1 to 4 reads a plane
> produced on an earlier tick. The lag is the cadence, not one tick, and
> section 7.4 of the supporting report shows every consumer tolerates it.
>
> **Delivery to Python.** Return `military_plane(faction)`,
> `economic_plane(faction)` and `threat_plane(faction)` as copies into a
> reusable Rust-owned scratch buffer, and say "copies" in the documentation
> as D35 requires; each is 64 KiB and costs about 2 microseconds. Return
> `all_military_sum()`, `terrain_value()`, `resource_potential()`,
> `conductance()`, `dominant_faction()`, `dominant_strength()`,
> `second_faction()` and `second_strength()` as zero-copy views. Return
> `military_stack(faction_list)` as a `(k, 65536)` copy for
> reinforcement-learning observation tensors. Every view lives inside a scope,
> as D36 requires.
>
> **The design does not depend on the faction mask width.** Per-faction
> planes key on `FactionId`, which is a `u16`. Shared planes key on the cell.
> Only the friendly-set subtraction touches the mask, and only over the small
> friendly set. Above 257 addressable factions, widen the all-faction sum
> plane from `u16` to `u32`, which takes it from 128 KiB to 256 KiB. Nothing
> else changes. Do not let this decision affect the mask-width choice; that
> choice belongs to D48.
>
> **Revisit trigger.** Move a plane to level 0 only if a measured profile
> shows a consumer that needs a decay length under 64 tiles and that the
> batched neighbour sweep cannot serve. Widen the cell from `u8` to `i16`
> only if a measured comparison shows that the argument-of-the-maximum cell
> choice disagrees with an `i16` reference on more than 1 percent of queries.

---

> #### Amendment to D50
>
> Replace the influence-map bullet with the following text.
>
> **Influence maps.** Two `u8` planes for each faction over the 65,536 level
> 1 cells, plus five shared planes and a dominant-pair summary. See D51. Do
> not maintain 4 to 8 planes for each faction; six of the eight are either
> faction-independent or derived at read. Do not use a float cell; D4 bans
> it. The multi-source bucketed Dijkstra that this decision describes is the
> correct algorithm for the dominant-pair summary, and it runs once for every
> faction rather than once for each faction.

---

> #### Amendment to the memory budget table
>
> Replace the line "Influence maps (8 maps x 65,536 cells) | about 2 MiB"
> with these three lines.
>
> | Structure | Size |
> |---|---|
> | Influence, shared planes and worker scratch | 1.69 MiB |
> | Influence, for each rendered faction | 28.0 KiB typical, 136 KiB worst |
> | Influence, for each active faction | 14.0 KiB typical, 68.0 KiB worst |
>
> The old line understated the cost of the design it described. Eight planes
> of 65,536 cells at 4 bytes is 2.00 MiB, but the 4-byte cell was a float,
> which D4 bans, and a deterministic Jacobi sweep needs a second plane. The
> honest figure for that design is 4.00 MiB for each faction, so 4.00 GiB at
> 1,024 factions. The replacement is 2.90 MiB in total at 1,024 factions.

---

## 13. Open questions from this report

1. **Does the military plane need terrain conductance?** The report assumes
   yes, and pays 150 microseconds for a seeded Jacobi solve instead of 12
   microseconds for a separable recursion. **The measurement that decides
   it:** run a controller against both planes over 1,000 ticks and count the
   ticks on which the chosen target cell differs. Adopt the recursion if the
   disagreement is under 1 percent.

2. **Is a `u8` cell enough?** Section 5.1 argues that 4.4 octaves of usable
   gradient covers every consumer. **The measurement that decides it:**
   compute a reference plane at `i32` and the production plane at `u8`, then
   count the cells on which the argument of the maximum over a faction set
   differs. Widen to `i16` if the rate exceeds 1 percent.

3. **Is a cadence of 8 ticks visible?** Section 7.1 argues that the
   staleness is below the resolution. **The measurement that decides it:**
   run a controller at cadence 1 and at cadence 8 and compare the decision
   streams. This cannot be evaluated before a controller exists.

4. **What is the real decay length for each concern?** Every cost in section
   5.3 depends on it. The support radius, and therefore the block count and
   the size, follow directly. A designer must set it. Until then, use a
   halving length of 4 level 1 cells, which is 64 tiles.

5. **Does the dominant pair need a third place?** Section 5.6 shows that two
   places answer territory, borders and contested cells. A three-way border
   is a real case in a grand-strategy world. A third place costs 3 more bytes
   for each cell, so 192 KiB. Decide it against a scenario, not against a
   cost.

6. **Should the economic plane be a plane at all?** Section 8.2 removes it
   from tier A on the argument that the shared resource potential plane plus
   a settlement list answers the same question. If that argument holds for
   tier A, test whether it also holds for tier R. If it does, the per-faction
   plane count falls from 2 to 1 and the tier R cost halves to 14.0 KiB.

---

## References

[^1]: Cachette project instructions, sections "Hard invariants" and "Design principles". `CLAUDE.md`
[^2]: ADR-0001, Foundational Architecture, decisions D4, D5, D9, D16, D17, D19, D28, D29, D35, D36, D45, D50, the memory budget table and the per-tick cost budget. `docs/adrs/REGISTRY.md`
[^3]: Research report 08, Fog of War Representation, sections 3.1, 6.3, 6.4, 8, 12 and 13. `docs/research/reports/08-fog-of-war-representation.md`
[^4]: Research report 06, Algorithms and Scheduling, sections 8.1, 8.2, 8.4, 9.1 and 10. `docs/research/reports/06-algorithms-and-scheduling.md`
[^5]: Research report 07, Target Platform and Value Types. `docs/research/reports/07-target-platform-and-value-types.md`
[^6]: Arm Neoverse N1 Software Optimization Guide, instruction throughput and memory system latency tables. https://developer.arm.com/documentation/swog309707/latest
