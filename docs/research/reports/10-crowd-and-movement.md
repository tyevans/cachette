# Crowd Simulation and Unit Movement at One Million Units

Research report 10 for the foundational architecture decision record.

## 0. Context

This report covers unit movement for a hex world simulation engine. The
engine core is Rust. The control plane is Python. The target scale is
16.7 million hex tiles and one million units. The engine runs headless on
AWS Graviton servers, so the target instruction set is aarch64 with NEON.

The engine runs a fixed tick at 10 Hz. One tick has 100 ms of wall time.
The record budgets about 400 core-ms for one tick and shares that budget
between movement, combat, planning, visibility and aggregation. Movement
gets a small part of it.

Four constraints bind every recommendation in this report. The project
instructions state them as hard invariants.[^1]

1. **No floating point in simulated state.** Float addition is not
   associative, so a parallel reduction over floats is not reproducible.
   All simulated arithmetic uses integers or Q16.16 fixed-point.
2. **Determinism.** The same binary with the same input must produce the
   same result at any thread count. Ordered iteration, stable sort keys,
   and no dependence on thread completion order.
3. **The read and write phase split.** The frame loop reads the world and
   writes only events, then applies the events and writes the world.
   A movement system may not write a unit position during the read phase.
4. **Graviton.** NEON is the baseline. Cache lines are 64 bytes. The
   memory model is weak, so atomics cost more than on x86. Prefer disjoint
   outputs over atomics.[^2]

The record already decides two parts of the movement design.[^3] It uses a
portal graph over 32x32 pathing chunks for the long-range plan, and small
cached flow fields inside one chunk for the local step. It rejects a
map-wide flow field on memory and time grounds. This report extends the
supporting research on algorithms and scheduling that produced those
decisions.[^4] It does not revisit them.

### 0.1 The question this report settles

The record has not decided how a unit position is represented. Two models
are open.

- **The continuous model.** A unit holds a fixed-point sub-tile coordinate.
  It moves by a small displacement each tick. Many units share a tile.
- **The tile-discrete model.** A unit holds a tile index and nothing else.
  It moves to one of the six neighbour tiles, or it stays. Each tile has a
  capacity.

The choice decides which literature applies. Flocking, reciprocal velocity
obstacles and continuum crowds all assume continuous space. The
tile-discrete model has its own literature, which is the cellular automaton
work on pedestrian and traffic flow.

**This report recommends the tile-discrete model.** Section 5 gives the
cost comparison. Section 11 gives the decision block for the record.

### 0.2 Terms

**Agent.** One simulated moving object. This report writes "unit", which is
the term the project uses.

**Flow field.** An array over cells. Each cell holds the travel cost to a
goal, and the direction that reduces that cost. One backward search builds
it. Every unit then reads its own cell in constant time.

**Flow tile.** A flow field that covers one 32x32 pathing chunk only, with
a chunk exit as its goal.

**Eikonal equation.** The equation that describes the shortest travel time
across a medium with a variable speed. Its solution is a distance field. A
grid shortest-path search is a coarse approximation of it.

**Floor field.** In the cellular automaton literature, a per-cell scalar
that a pedestrian descends. A static floor field holds the distance to the
goal. A dynamic floor field holds a trace of recent occupancy.

**Kernel vocabulary.** The engine expresses parallel work as map, gather,
scatter, reduce, scan, sort, stencil and local join. This report uses those
names.

---

## 1. Flocking: Reynolds boids

### 1.1 What the model is

Reynolds introduced the distributed behavioural model in 1987.[^5] Each
agent applies three steering rules against the agents inside a local
neighbourhood: separation moves away from close neighbours, alignment
matches the average heading, and cohesion moves toward the average
position. The three results combine into one acceleration. Reynolds later
extended the set to about fifteen named behaviours, including seek, flee,
arrive, pursue, obstacle avoidance and path following.[^6]

The model is correct and it is the ancestor of most game steering code.
It is still the wrong model for this engine. Two separate reasons apply.

### 1.2 The neighbour query is the cost, and it is solvable

The naive form tests every pair. At one million units that is 10^12 pair
tests. At one nanosecond for each test that is about 1,000 seconds for one
tick. This is the figure that the literature quotes.

The failure is not fundamental. Reynolds solved it. In 2006 he ran
a large boid crowd at interactive rates on the parallel co-processors of a
games console. He sorted the agents into a uniform spatial grid and scanned only
the adjacent buckets.[^7] The
same technique is standard in particle simulation. The published method
assigns each particle a cell key, sorts the particles by that key with a
radix sort, then builds a start and end index for each cell.[^8] A
fixed-radius neighbour query then reads a small contiguous run.[^9]

**The engine already has this structure.** The record keeps unit indices
sorted by packed tile index, with a start and length pair for each 16x16
block and a bitplane that marks a block as occupied. That is the sorted
uniform grid of the particle literature, arrived at independently. The
engine needs no separate hash grid, and the record is correct to refuse
one.

With the sorted grid, a boid update costs about seven cell reads and a
short arithmetic blend. That is 40 to 80 ns for each unit, so 40 to 80
core-ms for one million units. It fits the tick, but it consumes 10 to 20
percent of the whole budget.

### 1.3 The real objection: boids solve a problem this engine does not have

Alignment and cohesion make a leaderless group hold together and pick a
shared heading. A commanded army does not need either. The flow field
supplies the heading, and the shared goal supplies the cohesion. Alignment
and cohesion would fight the flow field and would need weight tuning to
stop them doing so.

Separation is the only rule that earns its cost. Section 4 shows that a
per-tile density count supplies separation for less arithmetic than the
boid form, and with no vector normalisation.

**Verdict: reject boids as the movement model. Keep the sorted-grid
neighbour structure, which the engine already has for other reasons. Keep
separation, in the cheaper density form.**

---

## 2. Continuum crowds

### 2.1 What the model is

Treuille, Cooper and Popovic published continuum crowds in 2006.[^10] The
method treats a crowd as a continuum rather than as a set of agents. For
each group that shares a goal it builds three fields over the domain: a
density field from the agent positions, a speed field that falls as density
rises, and a unit cost field. It then solves the eikonal equation with the
fast marching method to get a potential field whose gradient every agent
descends.

The important property is that congestion is not simulated for each pair.
It emerges, because density lowers the speed field, which raises the
travel cost, which bends the potential field away from the crowd. Agents
spread out and form lanes without any pairwise rule.

The model draws directly on the continuum theory of pedestrian flow
published by Hughes in 2002.[^11] Narain and colleagues later extended the
continuum treatment to very dense crowds by adding an incompressibility
constraint, and reported hundreds of thousands of agents.[^12]

### 2.2 Why the full method does not fit at level 0

Continuum crowds rebuilds the density field and re-solves the eikonal
equation every frame, over the whole domain, once for each group. At the
target scale that is fatal.

| Item | Figure at 16.7M tiles |
|---|---|
| Cells in one field | 16,700,000 |
| Cost array, u32 for each cell | 67 MB |
| Direction array, u8 for each cell | 16.8 MB |
| Fast sweeping solve, 4 sweeps | 800 to 1,700 core-ms |
| Fields needed for 20 goal groups | 20 times the above |

One field costs twice the whole tick budget. Twenty fields cost 1.7 GB.
The record already reaches the same conclusion for a plain Dijkstra flow
field and forbids a map-wide field. That conclusion holds here and is
stronger, because the eikonal solve is more expensive than the shortest
path search it replaces.

### 2.3 Where continuum crowds does fit: level 1

The engine has a level 1 grid of 65,536 cells. That grid is maintained
every tick by the dirty pyramid walk. It already carries per-cell unit
counts. A density-aware eikonal solve over 65,536 cells is affordable.

| Item | Figure at 65,536 L1 cells |
|---|---|
| Cost array, u32 for each cell | 256 KB |
| Fast sweeping solve, 4 sweeps, fixed-point | 6 to 13 core-ms |
| Eight active fields, refreshed every 30 ticks | 2 to 4 core-ms for each tick |

This is a real and cheap adaptation. Solve the eikonal equation at level 1,
with a speed field derived from the level 1 unit density that the pyramid
already holds. Use the resulting potential as the long-range route
preference. It is density-aware global routing at a cost of a few core-ms
for each tick.

**The recommended structure is therefore continuum crowds at level 1 and
cached flow tiles at level 0.** The coarse field says which region to cross
and steers armies around each other at map scale. The flow tile says which
tile to step to. Neither runs over the whole level 0 grid.

The level 1 field does not replace the portal graph. The portal graph
answers connectivity questions exactly, including "is there any route at
all", which a coarse field answers wrongly across a one-tile gap. Use the
portal graph for the route and the level 1 potential to bias the edge costs
in it.

### 2.4 The one idea to keep at level 0

Take one idea from continuum crowds into the level 0 step: **density raises
local cost**. Section 4.3 shows how to apply it without invalidating the
flow tile cache every tick.

---

## 3. Solving the eikonal equation without floating point

The project bans floats from simulated state. This section answers whether
an eikonal solver can run in integer or fixed-point arithmetic. **It can,
exactly and deterministically.** The reasoning follows.

### 3.1 The three families of solver

**Fast marching.** Sethian published the fast marching method in 1996.[^13]
Tsitsiklis published an equivalent construction from control theory in
1995.[^14] The method visits cells in increasing order of arrival time,
using a heap, and updates each cell once. It costs O(N log N). It is a
Dijkstra search with a different local update rule.

**Fast sweeping.** Zhao published the fast sweeping method in 2005.[^15] It
applies the same local update in a fixed set of alternating sweep
directions across the whole array. In two dimensions it uses four sweep
orderings. It needs no priority queue and costs O(N). For a simple speed
field a fixed number of sweeps converges.

**Fast iterative.** Jeong and Whitaker published the fast iterative method
in 2008.[^16] It keeps a narrow active list without a strict ordering,
which suits wide parallel hardware.

**Bucket queue Dijkstra.** Dial published the bucket queue shortest path in
1969.[^17] With small integer edge costs it costs O(N) with a very small
constant. It is not an eikonal solver. It solves the graph shortest path,
which is a coarser approximation.

### 3.2 The fixed-point update, and why it is exact

The local update solves a quadratic. On a lattice, given the two smallest
upwind neighbour arrival values `a <= b` along two axes, and a local cost
`c` for one step, the Godunov update is:

```
if b - a >= c:
    T = a + c
else:
    T = (a + b + isqrt(2*c*c - (b - a)*(b - a))) / 2
```

Three properties make this safe in fixed-point arithmetic.

1. **The only non-linear operation is a square root.** Integer square root
   is exact. `i64::isqrt` returns the exact truncated integer root. It is
   defined by a total function on the integers with no rounding mode, no
   platform library, and no last-bit variation. The engine already notes
   that a square root is the one transcendental it can afford.
2. **The update is monotone.** A cell value only ever decreases. Values are
   integers bounded below by zero. Therefore the sweep terminates in a
   finite number of iterations, and it terminates at the same value on
   every machine.
3. **The sweep order is fixed.** Fast sweeping is a Gauss-Seidel iteration,
   so it depends on the visit order. That order is a compile-time constant,
   not a thread schedule. It is deterministic by construction.

**The fixed-point eikonal solver is more deterministic than the float one,
not less.** The float version depends on the rounding of a division and of
a square root, and on the compiler's contraction choices. The fixed-point
version depends on neither.

The literature does not appear to publish an integer eikonal solver as a
named result, because the numerical analysis community wants convergence to
the continuous solution rather than exact reproducibility. Fixed-point and
integer arithmetic are however standard in the related field of distance
transforms, where exact integer algorithms are the norm.[^18] The
construction above is an application of known parts, not a new algorithm.

### 3.3 Fast sweeping on a hex lattice

Axial hex coordinates are a sheared square lattice. The six neighbours of
`(q, r)` are `(±1, 0)`, `(0, ±1)`, `(+1, -1)` and `(-1, +1)`. That is the
four-neighbour square set plus one diagonal pair. Four sweep orderings
therefore cover every characteristic direction, exactly as in the square
case. Each update takes the two smallest upwind values from the three axis
pairs.

Fast sweeping is a **stencil** kernel in the engine's vocabulary. It reads
a fixed neighbourhood and writes one cell. It vectorises on NEON along a
sweep line, subject to the Gauss-Seidel dependence, which limits the
vectorisation to the anti-diagonal wavefront. For a 32x32 tile that
constraint does not matter, because the parallelism lives across tiles, not
inside one.

### 3.4 The metric error, and a correction to the existing analysis

The record states that hex removes the diagonal problem, because all six
neighbours sit at equal distance. That is true of the **cost anisotropy**:
a hex grid needs no square-root-of-two fudge factor and it produces no
biased tie-breaks. It is not true of the **metric error**.

A six-connected lattice cannot represent a direction between two neighbour
axes. Travel at 30 degrees off an axis alternates two neighbour steps that
are 60 degrees apart. Two steps of unit cost give a displacement of
`2 cos 30 = 1.732`. The path cost therefore overstates the true distance by
`2 / 1.732 = 1.155`, so **the worst-case error of a six-connected hex
shortest path is 15.5 percent**. An eight-connected square grid with
correct diagonal costs has a worst case of 8.2 percent, and a
four-connected square grid has 41.4 percent.

Hex is better than a four-connected square grid and worse than an
eight-connected one. The eikonal solve removes most of this error, because
its local update interpolates between the two upwind axes instead of
choosing one.

**This does not change the recommendation for the flow tile.** A flow tile
covers 32 cells. A 15.5 percent error over a 32-cell span is a few cells of
detour, and the portal graph fixes the route at the larger scale. It does
change the recommendation at level 1, where a field spans the whole map and
a 15.5 percent bias is a visible and systematic route distortion. Use the
eikonal solver at level 1 and the bucket queue at level 0.

### 3.5 Cost comparison of the two solvers

Measured per 32x32 hex flow tile, 1,024 cells and about 3,000 directed
edges, on one Graviton core.

| Solver | Operations | Estimated time | Metric error |
|---|---|---|---|
| Dial bucket queue Dijkstra | ~3,000 relaxations | 5 to 20 us | 15.5 percent |
| Fast sweeping, fixed-point | ~12,000 updates with `isqrt` | 60 to 150 us | ~2 percent |

At 100 tile builds for each tick, that is 0.5 to 2 core-ms against 6 to 15
core-ms. The bucket queue is 5 to 10 times cheaper and its extra error is
bounded by the chunk size.

**Recommendation: use the Dial bucket queue for level 0 flow tiles. Use
fixed-point fast sweeping for the level 1 field. Write the fast sweeping
solver so that it can replace the bucket queue at level 0 behind one flag,
if route quality inside a chunk ever proves to be a visible problem.**

---

## 4. What the tile-discrete model needs, and what it removes

This section defines the recommended model in full.

### 4.1 The state of a moving unit

| Field | Type | Bytes | Purpose |
|---|---|---|---|
| `tile` | `TileIdx`, `u32` | 4 | The unit's position. Already the sort key. |
| `progress` | `u16` | 2 | Movement accumulator, in cost units. |
| `plan` | `u32` | 4 | Index into the shared plan table. |
| `blocked` | `u8` | 1 | Consecutive blocked ticks. Saturates. |

There is no position component, no velocity component, and no heading. The
tile index is the position, and the engine already stores it as the spatial
sort key. **The tile-discrete model adds no position state at all.**

### 4.2 Speed without sub-tile position

A unit that moves at most one tile for each tick has a speed quantised to
zero or one tile for each tick. That is too coarse. The standard fix is an
accumulator, which is the same device as the action-point model in
turn-based games and as the cell-hop rule in the traffic cellular automaton
that Nagel and Schreckenberg published in 1992.[^19]

```
progress += speed(unit_type, upgrades)          // integer, per tick
step_cost = terrain_cost[terrain(target)]        // integer
if progress >= step_cost:
    attempt the step
    on success: progress -= step_cost
```

`speed` and `step_cost` are integers on a common scale. A scale of 256 per
tile gives 256 speed grades and needs a `u16` accumulator with headroom.
This gives arbitrary speed resolution and exact terrain costs with no
sub-tile coordinate and no fixed-point multiply.

### 4.3 The per-tile density array

The record's tile-to-unit bridge is a sorted unit array with a start and
length pair for each 16x16 block. Finding the units on one tile means
searching inside a block range.

**That is too slow for the movement step, and this is a real defect in the
current steering design.** The record states that the separation term reads
the occupancy index of the six neighbours and the own tile for 20 to 40 ns.
Under the block-level bridge, each of those seven reads is a search in a
range of up to 256 entries. Seven searches cost hundreds of nanoseconds,
not tens.

The fix is a dense derived array.

| Structure | Type | Size at 16.7M tiles |
|---|---|---|
| Per-tile unit count | `u8` for each tile | 16.8 MB |

Three properties make this affordable.

1. **It is a projection, not a source of truth.** The sorted unit array
   remains authoritative. The count array is derived, exactly as level 1 is
   derived from level 0. It fits the existing read-model framing.
2. **It is maintained by delta, never rebuilt.** Only tiles that a unit
   entered or left change. At 300,000 movers that is at most 600,000 byte
   writes, not a 16.8 MB clear.
3. **It serves three purposes at once.** It is the separation term, it is
   the capacity check, and it is the density term of continuum crowds. One
   structure, three uses.

A `u8` count saturates at 255, which is far above any sensible tile
capacity. If the 16.8 MB is unwelcome, a 4-bit count halves it to 8.4 MB
and caps capacity at 15, at the cost of a shift and a mask on every read.
Start with `u8`.

The tile-discrete model removes an 8-byte position component for each unit,
which is 8 MB at one million units. The net memory change is therefore
about plus 9 MB.

### 4.4 The transition rule

This is the floor field cellular automaton of Burstedde, Klauck,
Schadschneider and Zittartz, published in 2001.[^20] Their model gives each
pedestrian a transition weight for each neighbour cell, built from a static
floor field that holds the distance to the goal and a dynamic floor field
that holds recent occupancy. The model reproduces lane formation, clogging
at a bottleneck, and the faster-is-slower effect, and it is validated
against measured pedestrian data.

The engine's version:

```
for each of the 6 neighbours n of the unit's tile:
    if n is impassable:            skip
    score[n] = flow_cost[n] + W_DENSITY * density[n]
choose the neighbour with the lowest score
break ties by neighbour index, which is a fixed order
```

`flow_cost` is the cached cost field of the flow tile. **This is why the
flow tile must store the cost field and not only the direction byte.** The
record already budgets 2,048 bytes of cost beside 1,024 bytes of direction
for each tile, so the data is present. The direction byte becomes a fast
path for the uncongested case.

Applying density here, at the moment of choice, rather than inside the flow
tile build, is the key design move. It gives the density-aware spreading
behaviour of continuum crowds while leaving the cached flow tile
independent of density, so the cache does not invalidate every tick. The
cache key stays `(chunk_id, exit_portal_id)` exactly as the record decides.

`W_DENSITY` is an integer constant. A value near one quarter of a typical
step cost makes a unit prefer a detour of one tile rather than enter a tile
that already holds four units.

### 4.5 Conflict resolution: the discrete model's replacement for collision avoidance

In continuous space, two units that want the same space must negotiate.
That negotiation is what reciprocal velocity obstacles solve, at high cost.
In the discrete model there is no negotiation. There is an admission rule,
and it is a sort followed by a segmented scan.

The rule runs in four ordered sub-steps.

1. **Departure count.** Reduce the intent list by source tile. This gives,
   for each tile, how many units are leaving it this tick. **The intent
   list is already grouped by source tile**, because the units are sorted
   by tile index and each unit emits its intent in that order. So this is
   a segmented reduce over a contiguous list, and it needs no sort.
2. **Admission.** Sort the intents by target tile. Each target tile now
   owns one contiguous segment. For each segment, admit at most
   `capacity - (density[target] - departures[target])` intents, in the
   segment's existing stable order. Reject the rest.
3. **Position write.** Scatter the accepted target tiles into the unit
   array.
4. **Density write.** Apply the departures, then apply the arrivals.

Sub-step 1 exists so that a marching column can advance in one tick. Without
it, a solid file of units in a corridor would block itself, because the
tile ahead still appears full. Subtracting the departures lets the whole
column step together.

**Every sub-step is deterministic without an atomic.** The intents are
written to one buffer for each span and concatenated in span index order.
The sort is the engine's stable radix sort, keyed on the target tile and
then on the unit's stable key. The segments after the sort are disjoint, so
the admission scan parallelises across segments with no contention. The
departure pass and the arrival pass are separated by an ordering constraint
so that the two writes to the density array never collide.

Two failure cases need a rule.

- **A head-on block.** Two units want each other's tiles through a
  one-tile corridor. Under the departure rule both depart and both arrive,
  so a direct swap succeeds. A true deadlock needs three or more units and
  is rare.
- **A persistently blocked unit.** Increment `blocked` on rejection. Above
  a threshold, take a lateral step chosen by the counter-based random
  generator keyed on `(MOVE, frame, entity, 0)`, or mark the plan stale and
  re-plan. This is the friction and stochastic-choice device of the
  cellular automaton pedestrian models.[^21] The keyed generator keeps it
  deterministic.

### 4.6 Where this sits in the frame loop

The intent computation is a pure read. It reads the unit array, the flow
tile cache and the density array, and it writes only intent records. It
belongs in the execute phase, which is the phase that reads the world and
writes events.

The admission rule writes. It belongs in the apply phase. It is not a
plain event apply, because it makes a decision, so state it as an ordered
sequence of four sub-steps inside one system. **It adds no barrier.** The
record fixes the loop at five barriers and says not to add more. Ordered
sub-steps inside one system are not barriers.

### 4.7 The kernel vocabulary mapping

| Step | Kernel | Notes |
|---|---|---|
| Build a flow tile, bucket queue | sort and scan | Buckets are the sort. |
| Build the level 1 field, fast sweeping | stencil | Fixed sweep order. |
| Choose the target neighbour | map and gather | Gathers hit L1 cache. |
| Emit an intent | scatter | One buffer for each span. Disjoint. |
| Count departures by source tile | reduce, segmented | Already grouped by source. |
| Admit against capacity | scan and local join | Segments are disjoint. |
| Write the new tile index | scatter | Disjoint by construction. |
| Maintain the density array | scatter, two passes | Departures, then arrivals. |
| Re-sort the unit array | sort | Near-sorted. Movers only. |
| Move aggregate counts at level 1 | reduce and map | The coarse model. |

Every step is in the vocabulary. Nothing needs a new primitive.

### 4.8 The sorted-unit-array invariant is preserved and improved

The engine keeps unit indices sorted by tile index so that spatial kernels
are sequential scans. The tile-discrete model strengthens this.

- The sort key is the position. There is no separate quantisation step and
  no chance of the key disagreeing with the position.
- A unit moves to an adjacent tile, so the key changes by a small amount in
  a block-tiled index. The array stays nearly sorted.
- Only the units that moved need re-sorting. Sort the mover subset and
  merge it into the stationary majority. At 300,000 movers out of one
  million that is 2 to 4 core-ms rather than a full 4 to 8 core-ms sort.

In the continuous model the sort key is derived from the position by a
division, the array is disturbed by every unit that crosses a boundary, and
the derivation is one more thing that can drift.

---

## 5. The decision: continuous or tile-discrete

### 5.1 Cost, at one million units and 300,000 movers

Both tables assume one Graviton core at about 2.5 GHz, 12 cores for the
wall-time column to stay comparable with the record's existing budget
table, and units sorted by tile index.

**Tile-discrete.**

| Step | Scale | Core-ms |
|---|---|---|
| Choose the target neighbour | 1M units | 15 to 30 |
| Emit and concatenate intents | 300k | 0.8 to 1.6 |
| Radix sort intents by target tile | 300k | 1.5 to 3 |
| Departure count | 300k | 0.5 to 1 |
| Admission scan | 300k | 0.5 to 1 |
| Write tiles and maintain density | 300k | 1 to 2 |
| Re-sort the unit array, movers only | 300k into 1M | 2 to 4 |
| Flow tile builds, bucket queue | 100 tiles | 0.5 to 2 |
| **Total** | | **22 to 45** |

Wall time at 12 cores: **1.9 to 3.8 ms**.

**Continuous, with the three-term steering blend.**

| Step | Scale | Core-ms |
|---|---|---|
| Gather the flow direction and 7 densities | 1M units | 25 to 45 |
| Three-term blend with a fixed-point normalise | 1M units | 35 to 75 |
| Integrate the position in Q16.16 | 1M units | 3 to 6 |
| Detect a tile crossing and emit an event | 1M units | 2 to 4 |
| Re-sort the unit array, not near-sorted | 1M | 4 to 8 |
| Flow tile builds, bucket queue | 100 tiles | 0.5 to 2 |
| **Total** | | **70 to 140** |

Wall time at 12 cores: **5.8 to 11.7 ms**.

The blend dominates. It needs a vector normalisation for each unit. In
fixed-point that is an integer square root and a division, which is 20 to
40 cycles, and it must be done for every unit every tick whether or not the
unit is near anyone.

**The continuous model costs about three times more and delivers no more
tactical fidelity.** Units still clip and jostle in both models, because
neither runs reciprocal velocity obstacles.

### 5.2 What the discrete model gives up, and what replaces it

| Given up | Replacement | Adequate? |
|---|---|---|
| Smooth on-screen motion | The renderer interpolates between the source and target tile using the tick fraction. This is view state, so it may use floats. | Yes. |
| Continuous speed | The integer movement accumulator of section 4.2. | Yes. |
| Sub-tile formation offsets | Formations become tile patterns, which suits a hex strategy game better. | Yes. |
| Pairwise avoidance between crossing streams | Lane formation emerges from the density term, which is the published behaviour of the floor field model.[^20] | Yes, for mass movement. |
| Exact avoidance for a named unit | None. Run reciprocal velocity obstacles for a capped set, or do not offer it. | Accept the limit. |
| Sub-tile physics and projectiles | None. Projectiles are not simulated positions in this design. | Out of scope. |

### 5.3 The retrofit direction, which decides the risk

This is the argument that settles the question.

Going from discrete to continuous later is cheap. Add a position component
and change one kernel. The portal graph, the flow tiles, the density array,
the intent event, the sorted unit array and the admission rule all survive
unchanged, because they are all keyed on the tile.

Going from continuous to discrete later is expensive. It discards the
tuned steering weights, the fixed-point overflow audit of the position
arithmetic, and every design decision that assumed a sub-tile offset.

**Discrete first is both the cheaper model and the lower-risk order.**

### 5.4 Verdict

**Adopt the tile-discrete model.** Take the flow tile cost field, the
per-tile density array, the density-biased local choice, the integer
movement accumulator and the sort-and-admit conflict rule. Reject sub-tile
positions for version 1 and revisit only if a specific game requirement
proves the tile grain too coarse.

---

## 6. Reciprocal velocity obstacles and optimal reciprocal collision avoidance

### 6.1 What they are

Fiorini and Shiller introduced the velocity obstacle in 1998.[^22] For one
agent and one moving obstacle, the velocity obstacle is the set of
velocities that lead to a collision within a time horizon. The agent picks
a velocity outside that set.

Van den Berg, Lin and Manocha published reciprocal velocity obstacles in
2008.[^23] The reciprocal form assumes the other agent avoids too, which
removes the oscillation that the naive form produces.

Van den Berg, Guy, Lin and Manocha published optimal reciprocal collision
avoidance in 2011.[^24] Each agent builds one half-plane constraint for
each nearby agent and solves a small two-dimensional linear program for the
velocity closest to its preferred one. The result is provably
collision-free under its assumptions and it is the standard implementation.

Karamouzas, Skinner and Guy later showed that real pedestrian interactions
follow a power law in time-to-collision, and derived a force from it.[^25]
The social force model of Helbing and Molnar remains the other widely used
continuous formulation.[^26]

### 6.2 The cost, stated honestly

Optimal reciprocal collision avoidance costs roughly 1 to 5 microseconds
for each agent for each frame, dominated by the neighbour query and the
linear program.

| Agents | Core-ms for one tick | Against a 400 core-ms budget |
|---|---|---|
| 1,000 | 1 to 5 | 0.3 to 1.3 percent |
| 100,000 | 100 to 500 | 25 to 125 percent |
| 1,000,000 | 1,000 to 5,000 | 250 to 1,250 percent |

At one million units it costs two and a half to twelve times the **entire**
tick budget, before movement, combat, visibility and aggregation get
anything. On a 64-core Graviton instance at 10 Hz there is 6,400 core-ms of
raw capacity in a tick, so a one-million-agent solve is arithmetically
possible only by spending 15 to 80 percent of the whole machine on
collision avoidance. That is not a trade any strategy simulation should
make.

The linear program also needs floating point or a careful fixed-point
rewrite. The published implementations use doubles and depend on the
tolerance handling around degenerate constraints. A deterministic
fixed-point port is real work with a real risk of subtle behaviour change.

### 6.3 Verdict

**Reject globally.** In the tile-discrete model, reject entirely: there are
no velocities to negotiate, and the admission rule already guarantees that
two units never occupy the same tile beyond its capacity.

If a future unit class genuinely needs exact avoidance, that class needs
continuous positions, which means it needs a second movement model. Do not
build that for version 1. If it is ever built, cap the class at about one
thousand units, which costs 1 to 5 core-ms.

---

## 7. Flow fields as shipped in games

Two shipped titles are documented.

**Supreme Commander 2.** Emerson described the flow field tile system in
Game AI Pro.[^27] The map is divided into sectors. A coarse graph over
sector portals gives the route. A small flow field is computed inside one
sector at a time and cached, keyed on the sector and the exit. This is the
design the record already adopts, and the record credits it correctly.

**Planetary Annihilation.** The title moves large armies over spherical
planets. Its developers demonstrated a flow field movement system in public
material, and stated that flow fields are their answer to moving large
numbers of units and formations.[^28] The published detail is thinner than
for Supreme Commander 2, so treat it as a second confirming case rather
than as a design source.

Both cases confirm the same three lessons, and the engine's design already
follows all three.

1. Never build a field over the whole world.
2. Split the problem into a coarse plan and a local field.
3. Key the local field cache on the geometry and the exit, not on the
   command that asked for it.

The hierarchical plan layer is HPA*, published by Botea, Muller and
Schaeffer in 2004, which reports paths within about one percent of optimal
at a small fraction of the search cost.[^29]

**Nothing here needs changing.** This section exists to confirm that the
record's decision matches shipped practice, and to record the sources.

---

## 8. Level of detail and hierarchical crowds

### 8.1 What the literature does

Crowd level of detail reduces the fidelity of agents that the observer
cannot see or cannot see well. The surveys and the crowd simulation
literature describe geometric, animation and behavioural levels of
detail.[^30] The common pattern is three tiers: full individual simulation
near the observer, simplified individual simulation at mid range, and
aggregate or statistical simulation far away.

Narain and colleagues show the far tier done well: a dense crowd simulated
as a continuum with an incompressibility constraint, with individual agents
recovered from the flow field.[^12]

### 8.2 What this engine already has

The record already decides a four-stage plan for unobserved regions: freeze
with a deterministic catch-up, then an active set in the style of Factorio,
then a coarse layer for genuinely global processes, and never a generative
promotion from a summary. The record also records Factorio's lesson that
the coarse layer must be a **different model** at a **lower rate**, not a
downsampled copy of the fine model.

### 8.3 What to add: the aggregate movement model

The coarse layer needs a movement model, and the record does not define
one. Define it as an integer transfer of counts between level 1 cells.

- A level 1 cell holds, for each faction, a vector of unit counts by unit
  type. The pyramid already builds this.
- The coarse step moves an integer count from one cell to a neighbour cell
  along the level 1 potential field of section 2.3. The transfer amount is
  the count times a rate, evaluated in integers with the remainder carried,
  so the total is conserved exactly.
- Individual identity is held in a side list attached to the cell. On
  activation, the units are materialised onto level 0 tiles inside the
  cell, in the side list's order.

This is a compressible flow formulation, in the spirit of the continuum
models, applied to level of detail rather than to visual realism. It is
exactly conservative because it is integer arithmetic with carried
remainders, which the float formulations cannot promise.

Cost: 65,536 cells times a few factions times a few unit types, run every
32 ticks. That is microseconds. Fidelity: a frozen army arrives at the
right place at the right time and with the right composition, and it has
no individual tactical behaviour on the way.

**Do not build this for version 1.** Build the freeze first, as the record
says. Record the model so that stage 3 has a defined shape when it arrives.

---

## 9. What this report rejects, and why

| Rejected | Reason |
|---|---|
| Boids as the movement model | Alignment and cohesion duplicate what the flow field and the shared goal already give, and they fight it. Separation is cheaper as a density term. 40 to 80 core-ms for no benefit. |
| Naive pairwise neighbour search | 10^12 pair tests at one million units. The sorted grid the engine already has removes it. |
| Continuum crowds at level 0 | 800 to 1,700 core-ms for one field and 67 MB of arrays. Twice the whole tick budget. |
| A map-wide flow field of any kind | Already rejected by the record on the same arithmetic. Confirmed. |
| Reciprocal velocity obstacles at scale | 1,000 to 5,000 core-ms at one million units, which is 2.5 to 12 times the whole budget. Meaningless in a discrete model. |
| The social force model | Continuous, float-shaped, needs a per-pair force sum, and gives no benefit a density term does not. |
| A separate spatial hash for neighbours | Duplicates the sorted unit array and adds a second invalidation path. The record already refuses it and is right. |
| Sub-tile continuous positions for version 1 | Three times the movement cost, an extra fixed-point normalisation, extra state, and no fidelity gain without reciprocal velocity obstacles that the engine cannot afford. |
| A float eikonal solver | Banned by the project, and worse: it is less reproducible than the fixed-point form, not more accurate in any way that matters here. |

---

## 10. Open questions from this report

1. **What is the tile capacity?** The admission rule needs a number.
   Capacity one gives a crisp board-game feel and severe corridor
   congestion. Capacity four to eight gives armies that flow. This is a
   game design decision and it must be made before the movement kernel is
   written, because it interacts with the density term's constant and with
   whether a `u8` or a nibble count array is correct. It also interacts
   with the record's open question on unit stacks.
2. **Is the 16.8 MB per-tile density array acceptable?** It is 21 percent
   of the 80 MiB minimum tile schema, and 10 percent of the 160 MiB rich
   schema. The alternative is a 4-bit count at 8.4 MB
   with a shift and mask on every read. Measure both on the target.
3. **Does the renderer's tick interpolation look acceptable at 10 Hz?**
   A 100 ms interpolation between adjacent tiles is the single visible
   cost of the discrete model. It cannot be judged from arithmetic. Build
   the milestone 3 movement demonstration and look at it.
4. **How many distinct movement plans are live at once?** The flow tile
   cache hit rate depends on it, and this report inherits that risk from
   the record without adding to it.
5. **Should the level 1 eikonal field replace or only bias the portal
   graph search?** This report recommends bias, because a coarse field
   answers connectivity wrongly across a one-tile gap. Confirm with a
   measurement once both exist.
6. **What is `W_DENSITY`?** The constant that trades detour length against
   congestion. It is a tuning value, not a structural one, but it must be
   an integer constant and it must be in the replay header if it is ever
   made configurable.

---

## 11. Proposed decision block for the record

**This block is ready to apply. It is written in the record's style. It
replaces D45 and adds two new decisions. It does not change D44, D46 or
D50.**

---

#### D45 (replacement). Unit positions are tile-discrete. Movement is a local choice plus a capacity-checked admission

**A unit's position is its tile index. There is no sub-tile coordinate.**
The tile index is already the spatial sort key, so this adds no state. It
removes an 8-byte position component, which is 8 MB at one million units.

Speed is an integer accumulator, not a displacement:

```
progress  += speed(unit_type, upgrades)
step_cost  = terrain_cost[terrain(target)]
if progress >= step_cost { attempt the step; progress -= step_cost }
```

This gives arbitrary speed resolution and exact terrain costs with no
fixed-point multiply and no sub-tile position.

The local choice replaces the three-term steering blend:

```
for each passable neighbour n of the unit's tile:
    score[n] = flow_cost[n] + W_DENSITY * density[n]
pick the lowest score; break ties by neighbour index
```

`flow_cost` is the cost array of the cached flow tile, so **the flow tile's
cost array is the primary product and the direction byte is a fast path**.
Applying density at the moment of choice rather than inside the flow tile
build is what keeps the flow tile independent of density, so the
`(chunk_id, exit_portal_id)` cache key of D44 survives unchanged. This is
the floor field cellular automaton rule, which is validated against
measured pedestrian data and which reproduces lane formation and clogging.

Conflicts resolve by sorting, not by negotiating. Four ordered sub-steps
inside one system, adding **no barrier** to D28:

1. Reduce intents by source tile to get a departure count for each tile.
2. Sort intents by target tile with the D50 stable radix sort. Each target
   owns one segment. Admit at most
   `capacity - (density[target] - departures[target])` from each segment,
   in the segment's stable order. Reject the rest.
3. Scatter the accepted target tiles into the unit array.
4. Apply departures to the density array, then apply arrivals.

Sub-step 1 exists so that a marching column advances in one tick rather
than blocking itself. Every sub-step uses disjoint outputs and no atomic,
which D1 requires on the weak-memory target. A rejected unit increments a
saturating `blocked` counter; above a threshold it takes a lateral step
chosen by the D21 counter-based generator keyed on `(MOVE, frame, entity,
0)`, or marks its plan stale.

**Intents are computed in the execute phase and admitted in the apply
phase.** Intent computation is a pure read and writes only intent records.
Admission writes, so it belongs after barrier 3.

**Cost at one million units and 300,000 movers: 22 to 45 core-ms, so 1.9 to
3.8 ms of wall time on 12 cores.** The continuous alternative costs 70 to
140 core-ms, mostly in the fixed-point vector normalisation that the
three-term blend needs for every unit every tick.

The renderer interpolates between the source and target tile using the tick
fraction. That is view state, not simulated state, so it may use floats.

**What this gives up:** smooth simulated motion, sub-tile formation
offsets, and exact pairwise avoidance. Units still clip in the continuous
model too, because ORCA is not affordable: 1 to 5 microseconds for each
agent is 1,000 to 5,000 core-ms at one million units, which is 2.5 to 12
times the whole tick budget. In the discrete model ORCA is not merely
unaffordable, it is meaningless.

**Why this order and not the other:** discrete to continuous is a cheap
retrofit, because the portal graph, flow tiles, density array, intent event
and admission rule are all keyed on the tile and all survive. Continuous to
discrete discards tuned weights and a fixed-point overflow audit.

Reject boids. Alignment and cohesion duplicate what the flow field and the
shared goal already supply, and they fight them; the sorted-grid form costs
40 to 80 core-ms for no benefit.

#### D51 (new). The per-tile unit count is a dense derived array

| Structure | Type | Size at 16.7M tiles |
|---|---|---|
| Per-tile unit count | `u8` for each tile | 16.8 MB |

**This closes a defect in the previous D45.** That decision claimed the
separation term reads the occupancy of seven tiles in 20 to 40 ns. Under
D15 the tile-to-unit bridge is a block-level `(start, len)` pair plus a
sorted array, so a per-tile lookup is a search inside a range of up to 256
entries. Seven such searches cost hundreds of nanoseconds. The dense array
is what makes the stated figure true.

The array is a **projection**, not a source of truth. The D15 sorted unit
array stays authoritative. Maintain the count by delta at 300,000 movers,
which is at most 600,000 byte writes; **never rebuild it**, because a
16.8 MB clear costs about 0.4 ms of pure bandwidth.

One structure serves three purposes: the separation term, the capacity
check of D45, and the density term of continuum crowds. A 4-bit count
halves it to 8.4 MB and caps capacity at 15, at the cost of a shift and a
mask on every read. Start with `u8` and measure.

Net memory against the continuous alternative: plus 16.8 MB for the array,
minus 8 MB for the removed position component, so about plus 9 MB.

#### D52 (new). Solve the eikonal equation at level 1 in fixed-point. Keep the bucket queue at level 0

**The eikonal equation runs exactly in fixed-point arithmetic.** The
Godunov local update, given the two smallest upwind values `a <= b` along
two axes and a step cost `c`:

```
if b - a >= c { T = a + c }
else          { T = (a + b + isqrt(2*c*c - (b-a)*(b-a))) / 2 }
```

Three properties make it deterministic. The only non-linear operation is an
integer square root, which is exact and has no rounding mode and no
platform library. The update is monotone, so values only decrease and are
bounded below by zero, which bounds the iteration count. The fast sweeping
sweep order is a compile-time constant, not a thread schedule. **The
fixed-point solver is more reproducible than the float one, not less.**

Fast sweeping is a **stencil** kernel. On the axial hex lattice the six
neighbours are the four-neighbour square set plus one diagonal pair, so the
four standard two-dimensional sweep orderings cover every characteristic
direction.

**Correction to D44's hex claim.** Hex removes the cost anisotropy of a
square grid, which is what "no diagonal problem" correctly means. It does
**not** remove the metric error. A six-connected lattice overstates a
30-degree diagonal by `2 / (2 cos 30) = 1.155`, so **the worst-case error
of a hex shortest path is 15.5 percent**. An eight-connected square grid's
worst case is 8.2 percent. Hex is better than four-connected and worse than
eight-connected.

Where each solver goes:

| Level | Solver | Cells | Cost | Error |
|---|---|---|---|---|
| L0 flow tile | Dial bucket queue | 1,024 | 5 to 20 us | 15.5 percent |
| L0 flow tile | Fast sweeping, fixed-point | 1,024 | 60 to 150 us | ~2 percent |
| L1 field | Fast sweeping, fixed-point | 65,536 | 6 to 13 core-ms | ~2 percent |

**Use the bucket queue at level 0.** It is 5 to 10 times cheaper and its
error is bounded by the 32-cell chunk span, which the portal graph corrects
at the larger scale. Write the fast sweeping solver so it can replace the
bucket queue behind one flag if route quality inside a chunk is ever
visibly wrong.

**Use fast sweeping at level 1**, with the speed field derived from the
level 1 unit density the pyramid already maintains. This is continuum
crowds applied where it is affordable: a density-aware global potential
over 65,536 cells for 256 KB and 6 to 13 core-ms. Refresh eight fields
every 30 ticks for about 2 to 4 core-ms for each tick. Over a full-map
span, the 15.5 percent lattice error is a systematic route distortion, so
level 1 is where the eikonal solve earns its cost.

**The level 1 field biases the portal graph edge costs. It does not replace
the portal graph search.** A coarse field answers connectivity wrongly
across a one-tile gap, and the portal graph answers it exactly.

#### Additions to the per-tick cost budget table

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Movement, tile-discrete, replacing the old row | 1M units | 22-45 | 1.9-3.8 |
| L1 eikonal fields, amortised over 30 ticks | 8 fields | 2-4 | 0.2-0.4 |

The old "Movement and steering" row at 20 to 40 core-ms is replaced. It
understated the continuous model, which this report measures at 70 to 140
core-ms, and it assumed a per-tile occupancy read that D15 does not
provide.

#### Addition to the byte budget table

| Structure | Size |
|---|---|
| Per-tile unit count (D51) | 16.8 MB |
| L1 eikonal fields (8 x 65,536 x u32) | 2 MB |

#### Addition to the "day one, and unretrofittable" list

**The tile-discrete position model.** It is not literally unretrofittable
in the expensive direction, which is the point: discrete to continuous is
cheap and continuous to discrete is not. Choose discrete first for that
reason as much as for the cost.

#### Addition to the open questions

**OQ17. What is the tile capacity?** *Blocks:* D45 and D51. Capacity one
gives a board-game feel and severe corridor congestion. Capacity four to
eight gives armies that flow. It fixes whether a `u8` or a 4-bit count is
correct, it sets `W_DENSITY`, and it interacts with OQ9 on unit stacks. It
is a game design decision and it must be made before the movement kernel is
written.

---

## References

[^1]: Cachette project instructions, section "Hard invariants". `CLAUDE.md`
[^2]: Research report 07, Target Platform and Value Types. `docs/adrs/background/adr-0001/07-target-platform-and-value-types.md`
[^3]: ADR-0001, Foundational Architecture, decisions D1, D15, D21, D28, D29, D44, D45, D46, D49 and D50. `docs/adrs/draft/adr-0001-foundational-architecture.md`
[^4]: Research report 06, Algorithms and Scheduling, sections 3 and 4. `docs/adrs/background/adr-0001/06-algorithms-and-scheduling.md`
[^5]: Reynolds, C. W., 1987. "Flocks, Herds, and Schools: A Distributed Behavioral Model". Computer Graphics (SIGGRAPH '87 Proceedings), volume 21, number 4, pages 25-34. https://doi.org/10.1145/37402.37406
[^6]: Reynolds, C. W., 1999. "Steering Behaviors For Autonomous Characters". Game Developers Conference 1999, pages 763-782. https://www.red3d.com/cwr/steer/gdc99/
[^7]: Reynolds, C. W., 2006. "Big Fast Crowds on PS3". Sandbox '06, Proceedings of the 2006 ACM SIGGRAPH Symposium on Videogames, pages 113-121. https://doi.org/10.1145/1183316.1183333
[^8]: Green, S., 2010. "Particle Simulation using CUDA". NVIDIA CUDA SDK technical document. https://developer.download.nvidia.com/assets/cuda/files/particles.pdf
[^9]: Hoetzlein, R., 2014. "Fast Fixed-Radius Nearest Neighbors: Interactive Million-Particle Fluids". GPU Technology Conference 2014, session S4117. https://on-demand.gputechconf.com/gtc/2014/presentations/S4117-fast-fixed-radius-nearest-neighbor-gpu.pdf
[^10]: Treuille, A., Cooper, S., Popovic, Z., 2006. "Continuum Crowds". ACM Transactions on Graphics, volume 25, number 3 (SIGGRAPH 2006), pages 1160-1168. https://doi.org/10.1145/1141911.1142008
[^11]: Hughes, R. L., 2002. "A continuum theory for the flow of pedestrians". Transportation Research Part B: Methodological, volume 36, number 6, pages 507-535. https://doi.org/10.1016/S0191-2615(01)00015-7
[^12]: Narain, R., Golas, A., Curtis, S., Lin, M. C., 2009. "Aggregate Dynamics for Dense Crowd Simulation". ACM Transactions on Graphics, volume 28, number 5 (SIGGRAPH Asia 2009), article 122. https://doi.org/10.1145/1618452.1618468
[^13]: Sethian, J. A., 1996. "A fast marching level set method for monotonically advancing fronts". Proceedings of the National Academy of Sciences, volume 93, number 4, pages 1591-1595. https://doi.org/10.1073/pnas.93.4.1591
[^14]: Tsitsiklis, J. N., 1995. "Efficient algorithms for globally optimal trajectories". IEEE Transactions on Automatic Control, volume 40, number 9, pages 1528-1538. https://doi.org/10.1109/9.412624
[^15]: Zhao, H., 2005. "A fast sweeping method for Eikonal equations". Mathematics of Computation, volume 74, number 250, pages 603-627. https://doi.org/10.1090/S0025-5718-04-01678-3
[^16]: Jeong, W.-K., Whitaker, R. T., 2008. "A Fast Iterative Method for Eikonal Equations". SIAM Journal on Scientific Computing, volume 30, number 5, pages 2512-2534. https://doi.org/10.1137/060670298
[^17]: Dial, R. B., 1969. "Algorithm 360: Shortest-path forest with topological ordering". Communications of the ACM, volume 12, number 11, pages 632-633. https://doi.org/10.1145/363269.363610
[^18]: Borgefors, G., 1986. "Distance transformations in digital images". Computer Vision, Graphics, and Image Processing, volume 34, number 3, pages 344-371. https://doi.org/10.1016/S0734-189X(86)80047-0
[^19]: Nagel, K., Schreckenberg, M., 1992. "A cellular automaton model for freeway traffic". Journal de Physique I, volume 2, number 12, pages 2221-2229. https://doi.org/10.1051/jp1:1992277
[^20]: Burstedde, C., Klauck, K., Schadschneider, A., Zittartz, J., 2001. "Simulation of pedestrian dynamics using a two-dimensional cellular automaton". Physica A, volume 295, numbers 3-4, pages 507-525. https://doi.org/10.1016/S0378-4371(01)00141-8
[^21]: Kirchner, A., Nishinari, K., Schadschneider, A., 2003. "Friction effects and clogging in a cellular automaton model for pedestrian dynamics". Physical Review E, volume 67, article 056122. https://doi.org/10.1103/PhysRevE.67.056122
[^22]: Fiorini, P., Shiller, Z., 1998. "Motion Planning in Dynamic Environments Using Velocity Obstacles". International Journal of Robotics Research, volume 17, number 7, pages 760-772. https://doi.org/10.1177/027836499801700706
[^23]: van den Berg, J., Lin, M. C., Manocha, D., 2008. "Reciprocal Velocity Obstacles for Real-Time Multi-Agent Navigation". IEEE International Conference on Robotics and Automation 2008, pages 1928-1935. https://doi.org/10.1109/ROBOT.2008.4543489
[^24]: van den Berg, J., Guy, S. J., Lin, M. C., Manocha, D., 2011. "Reciprocal n-Body Collision Avoidance". Robotics Research: The 14th International Symposium ISRR, Springer Tracts in Advanced Robotics volume 70, pages 3-19. https://doi.org/10.1007/978-3-642-19457-3_1
[^25]: Karamouzas, I., Skinner, B., Guy, S. J., 2014. "Universal Power Law Governing Pedestrian Interactions". Physical Review Letters, volume 113, article 238701. https://doi.org/10.1103/PhysRevLett.113.238701
[^26]: Helbing, D., Molnar, P., 1995. "Social force model for pedestrian dynamics". Physical Review E, volume 51, number 5, pages 4282-4286. https://doi.org/10.1103/PhysRevE.51.4282
[^27]: Emerson, E., 2013. "Crowd Pathfinding and Steering Using Flow Field Tiles". In Rabin, S. (editor), Game AI Pro: Collected Wisdom of Game AI Professionals, CRC Press, chapter 23. https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf
[^28]: Savage, P., 2013. "Planetary Annihilation devs show planet creation tech, clever unit pathfinding". PC Gamer, report on an Uber Entertainment developer demonstration of the flow field movement system. https://www.pcgamer.com/planetary-annihilation-devs-show-planet-creation-tech-clever-unit-pathfinding/
[^29]: Botea, A., Muller, M., Schaeffer, J., 2004. "Near Optimal Hierarchical Path-Finding". Journal of Game Development, volume 1, pages 7-28. https://webdocs.cs.ualberta.ca/~mmueller/ps/hpastar.pdf
[^30]: O'Sullivan, C., Cassell, J., Vilhjalmsson, H., Dingliana, J., Dobbyn, S., McNamee, B., Peters, C., Giang, T., 2002. "Levels of Detail for Crowds and Groups". Computer Graphics Forum, volume 21, number 4, pages 733-741. https://doi.org/10.1111/1467-8659.00631
