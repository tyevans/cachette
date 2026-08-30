# Simulation Algorithms and Parallel Scheduling

Research input for ADR-0001. Area: the algorithms that must stay fast at
1M+ entities, and the schedule that runs them.

Read `00-context-brief.md` first. This document uses its vocabulary
(L0/L1/L2, selector, verb, monoid aggregation, dirty pyramid).

---

## 1. Executive summary

These are the recommendations. Section numbers point to the detail.

1. **Do not compute map-wide flow fields.** A 4096x4096 hex grid holds
   16.7M tiles. One full Dijkstra pass over it costs 50-200 ms and touches
   more than 130 MB. Use a two-level scheme instead: a portal graph over
   32x32 chunks for the long-range plan, and small 32x32 "flow tiles"
   computed on demand for local steering. This is Emerson's flow-tile
   design. It costs 5-30 us per tile and it caches well. (§3)

2. **Cache flow tiles by (chunk, destination-portal), not by command.**
   Many commands share a destination. A shared cache turns N commands into
   far fewer field computations. Give the cache an LRU limit of about 4096
   tiles (about 8 MB). (§3.4)

3. **Make the per-unit step a table lookup plus a small steering blend.**
   Read one direction from the flow tile. Add a short-range separation
   term from the occupancy index. Do not run per-unit obstacle search.
   Budget 20-40 ns per unit. 1M units then cost 20-40 ms of core time,
   which is 2-4 ms on 12 cores. (§3.6)

4. **Use the dirty pyramid as the spatial index. Do not add a second
   uniform grid.** The pyramid already exists, it is already maintained,
   and it already carries the fields that selectors filter on. A separate
   hash grid would duplicate state and add an invalidation path. (§4.1)

5. **Adopt a static, precomputed schedule, not a dynamic work-stealing
   executor over systems.** Derive the conflict graph from declared
   component access at build time. Compile it into an ordered list of
   parallel stages. This gives deterministic results and near-zero
   scheduling overhead. Bevy's dynamic executor is a poor fit here,
   because dynamic dispatch order can leak into results. (§5)

6. **Get parallelism from data, not from systems.** With 1M units, one
   system split across 12 cores with `par_chunks` beats 12 different
   systems run at once. Prefer wide data-parallel stages. (§5.3)

7. **Use a fixed simulation timestep. Decouple it from render.** Run the
   sim at 10-30 Hz. Render and interpolate at display rate. A fixed step
   is a precondition for determinism and for replay. (§6)

8. **Freeze unobserved regions in v1. Do not simulate them coarsely.**
   Coarse background simulation has a hard sub-problem: promotion. You
   must materialize plausible L0 detail from an L1 summary, and then
   re-summarize on exit, without contradiction. Ship freeze first. Add
   a "budget aggregate" tier later. Add generative promotion last, or
   never. (§7)

9. **Invest in batched algorithms.** Multi-source BFS, sort-merge joins
   instead of per-item hash lookups, and bulk diffusion for influence maps
   all get cheaper when the whole set is known at once. This is the real
   payoff of set-valued commands. (§8)

10. **Hand-vectorize about five kernels, no more.** Movement integration,
    influence-map diffusion, bitset operations, the selector predicate
    scan, and the L0 to L1 reduction. Use the `wide` crate on stable Rust
    today. Keep `std::simd` behind a nightly feature flag. It is still
    unstable. (§9)

**Where this report disagrees with the context brief:** the brief's
decision 11 says `move_to` for 5000 units "computes one hierarchical flow
field". One field is correct only when the units share a destination and
a region. In the general case you compute one *plan* and several *flow
tiles*, and you reuse the tiles across commands. The saving is real. The
wording understates the machinery. See §3.4.

The brief also leaves L2 block size open. With 32x32 blocks, L2 holds
only 4x4 = 16 cells at 4096x4096. That is too coarse to prune usefully.
See §4.2.

---

## 2. Budget: what a frame can afford

Set the numbers first. All later claims refer to them.

Assume a modern desktop CPU with 8-16 cores, about 40 GB/s of usable
memory bandwidth, and about 3.5 GHz.

| Quantity | Value |
|---|---|
| L0 tiles | 16,777,216 |
| Bytes per tile (dense SoA, hot fields) | 8 |
| One full sweep of hot tile data | 134 MB |
| Time for that sweep, bandwidth bound | 3.3 ms |
| Units | 1,000,000 |
| Bytes per unit (hot: position, velocity, type, state) | 32 |
| One full sweep of hot unit data | 32 MB |
| Time for that sweep | 0.8 ms |
| Sim tick target | 10-30 Hz (33-100 ms) |
| Core-milliseconds per tick at 30 Hz, 12 cores | ~400 |

Two rules follow.

**Rule A. A full-map pass is not free.** Each one costs at least 3.3 ms
of pure bandwidth. You can afford two or three per tick, not twenty.
Every algorithm below must scale with the *dirty* set or the *active*
set, not with the map.

**Rule B. Per-unit work has a hard ceiling.** At 1M units and 400
core-ms per tick, you get 400 ns of core time per unit for everything.
Movement, combat, and AI must share it. A per-unit A* search costs
10-100 us. It exceeds the whole budget by two orders of magnitude.
This is the argument for flow fields, stated as arithmetic.

---

## 3. Mass pathfinding

### 3.1 Why per-unit search fails

A* on a grid of N cells explores O(N log N) nodes in the worst case.
For k agents you pay that k times. At k = 100,000 the cost is
prohibitive. The literature agrees on this point and reaches for
fields instead.
([How To RTS: Basic Flow Fields](https://howtorts.github.io/2014/01/04/basic-flow-fields.html))

### 3.2 Flow fields: the shape of the answer

A flow field inverts the problem. You run one search *backwards* from
the destination across the whole region. You store, per cell, the
direction that leads home. Every agent then reads its direction in O(1).

The build cost equals one Dijkstra pass: O(N log N) with a heap, or
O(N) with a bucket queue on small integer costs. The per-agent cost
afterwards is one memory read. The more agents share a destination,
the better the amortized cost.

Supreme Commander 2 shipped this technique. Planetary Annihilation
followed. Both moved thousands of units at once.
([Emerson, *Crowd Pathfinding and Steering Using Flow Field Tiles*, Game AI Pro](https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf))

Continuum Crowds is the academic ancestor. Treuille, Cooper and Popovic
solved a continuous eikonal equation over a density field, which unified
global planning and local avoidance. It produced excellent motion. It
is too costly here. The fast-marching solve runs over the whole domain
each frame, and the density field must be rebuilt each frame.
([How To RTS: Continuum Crowds](https://howtorts.github.io/2014/01/09/continuum-crowds.html))

Take one idea from Continuum Crowds and leave the rest: **unit density
belongs in the cost function.** Add a small density term to the local
cost when you build a flow tile. Crowds then spread out instead of
queueing into one lane. Do not attempt the full continuum solve.

### 3.3 The cost of a map-wide field: concrete figures

Assume a hex grid, so 6 neighbours per cell.

- Nodes: 16.7M. Edges: about 50M directed.
- Integrated-cost array, u16 per cell: 33.5 MB.
- Direction array, u8 per cell (6 directions plus "none"): 16.8 MB.
- A bucket-queue Dijkstra touches each edge once. 50M relaxations at
  about 3-5 ns each gives **150-250 ms single-threaded**.
- Even a plain multi-source BFS with uniform cost costs about 50-80 ms.

**Conclusion: never build a map-wide field.** Not per frame, not per
command. The memory alone (50 MB per destination) rules out caching
more than one.

### 3.4 Recommended design: portal graph plus flow tiles

Use two levels. This matches the brief's L0/L1 split.

**Level 1: the portal graph (the plan).**

Partition L0 into 32x32 chunks. A 4096x4096 map holds 128x128 = 16,384
chunks. On each chunk border, find the maximal runs of mutually passable
cells. Each run becomes a *portal* node. Typical maps give 4-10 portals
per chunk, so about 100,000 portal nodes and about 400,000 edges. Intra-
chunk edges carry the precomputed walk cost between portal pairs of the
same chunk. Inter-chunk edges are free.

This is HPA* (Botea, Muller and Schaeffer) adapted to the existing chunk
structure. HPA* reports near-optimal paths, usually within 1% of true
cost, at a small fraction of the search cost.
([Botea et al., *Near Optimal Hierarchical Path-Finding*](https://webdocs.cs.ualberta.ca/~mmueller/ps/hpastar.pdf))

Cost of one A* over the portal graph: a few thousand node expansions in
the bad case, so **50-500 us**. Memory: about 100k nodes at 32 bytes plus
edges, so **under 10 MB**. Precompute once at load. Update per chunk when
terrain changes.

Use L2 to prune this search further. If L2 says a whole region has no
passable connection toward the goal, skip its portals.

**Level 2: flow tiles (the steering).**

Do not build the whole field. Build a 32x32 flow field *inside one chunk
only*, with the chunk's exit portal as the goal. That is 1024 cells and
about 3000 edges. A bucket-queue Dijkstra over it costs **5-20 us**. The
result is 1024 bytes of direction plus 2048 bytes of cost.

A unit reads: which chunk am I in, which flow tile applies to my plan,
what direction does my cell hold. That is two or three cache lines.

Build flow tiles lazily. Build one only when a unit is about to enter the
chunk. Most chunks on a long path never need a tile, because the group
gets redirected before it arrives.

**The cache is where the win lives.**

Key the cache on `(chunk_id, exit_portal_id)`, not on the command. Two
different commands that route 5000 units each through the same chunk
toward the same portal share one tile. This is the correct reading of
the brief's decision 11.

Recommended cache: 4096 entries, LRU, about 12 MB with cost arrays. Clear
entries whose chunk is marked dirty in the terrain bitset.

Expected steady-state cost for a large battle: 50-200 tile builds per
tick, so **0.25-4 ms of core time**, trivially parallel across builds.

### 3.5 Incremental update when terrain or ownership changes

Three tiers of invalidation, cheapest first.

1. **Flow tile invalidation.** A tile depends only on its own chunk's
   cost field. When chunk C becomes dirty, drop every cache entry with
   `chunk_id == C`. Cost: O(portals in C). This is the common case, and
   it is nearly free.

2. **Portal-graph edge repair.** A terrain change inside C changes only
   C's intra-chunk portal-to-portal costs, and possibly C's portal set.
   Recompute by running one Dijkstra per portal inside C: 4-10 runs of
   1024 cells, so **50-200 us per changed chunk**. Neighbour chunks are
   unaffected unless the border cells changed.

3. **Plan repair.** Existing commands hold portal paths. A changed edge
   may invalidate some. Do not re-plan them all at once. Mark the command
   as "plan stale" and re-plan it when its units next reach a chunk
   boundary. This spreads the cost and it hides the latency.

**Failure mode to watch:** a player who spams wall construction can
dirty many chunks per tick. Bound the repair work. Process at most K
chunk repairs per tick from a queue. Units in a not-yet-repaired chunk
follow the stale field for one more tick. That is acceptable. A visible
one-tick lag beats a frame spike.

### 3.6 Local steering and collision avoidance at scale

You cannot afford RVO or ORCA at 1M units. ORCA solves a small linear
program per agent per frame against its k nearest neighbours. Cost is
roughly 1-5 us per agent. At 1M agents that is 1-5 seconds.

Use a cheap three-term blend instead:

```
desired = w_flow * flow_dir
        + w_sep * separation_from_occupancy_index
        + w_coh * formation_offset
```

The separation term reads the per-tile occupancy index that the brief
already specifies (CSR offsets plus packed unit array). Read the 6 hex
neighbours plus own tile. Push away from the crowded ones. This needs no
neighbour search at all, because the occupancy index *is* the search
result.

Cost: about 7 offset reads, a few multiplies, one normalize. Estimate
**20-40 ns per unit**. At 1M units: 20-40 ms core time, so **2-4 ms on
12 cores**. This fits the budget.

Add "unit density raises local cost" in the flow-tile build (§3.2). That
handles the macro congestion. The separation term then only has to stop
visual overlap.

**Trade-off:** this produces good-looking mass movement and poor
individual movement. Units will clip and jostle. For a strategy game
with 1M units that is the right trade. If a specific unit class needs
exact avoidance, run ORCA for that class only, and keep its count in the
hundreds.

### 3.7 Hex-specific concerns

Use axial (q, r) coordinates, as the brief proposes. Hex pathing differs
from square pathing in useful ways.

- **No diagonal problem.** All 6 neighbours sit at equal distance. Square
  grids need the sqrt(2) fudge, or they produce biased paths. Hex does
  not. This makes uniform-cost BFS a much better approximation of true
  distance, so the bucket queue is more often adequate.
- **Hex distance is cheap.** In axial coordinates,
  `dist = (|dq| + |dq + dr| + |dr|) / 2`. It is exact and it is an
  admissible A* heuristic. Three absolute values and a shift.
- **Direction fits in 3 bits.** Six directions plus "none" and "goal".
  You could pack two per byte. Do not. Keep one byte per cell. The
  unpack cost exceeds the bandwidth saving at 1 KB per tile.
- **Parallelogram chunks distort at the edges.** A 32x32 axial
  parallelogram is a rhombus in world space, not a compact blob. Its
  diameter is larger than a square chunk's. This makes intra-chunk
  portal distances slightly longer, and it makes the flow tile slightly
  less local. The effect is small. Exact nesting and shift/mask parent
  lookup are worth far more. Keep the brief's decision 3.

---

## 4. Spatial queries

### 4.1 The acceleration structure: use the pyramid

The brief's decision 6 already puts filterable fields in the L1/L2
summaries. Extend that to spatial queries.

**Do not add a separate uniform hash grid.** Reasons:

- The pyramid already covers the map with a uniform partition. A hash
  grid over the same map would hold the same information twice.
- Two structures means two invalidation paths, and the second one will
  fall out of sync in some rare case.
- The pyramid is already maintained by the dirty-bit walk. The marginal
  cost of using it for queries is zero.

The only case for a hash grid is a query whose natural cell size differs
sharply from 32x32. Handle that by choosing the level, not by adding a
structure.

**Range query** (all units within radius R of a point): compute the set
of L1 cells the disc touches. For each, test the summary. If the cell
holds no matching unit type, skip 1024 tiles. Otherwise descend to L0
and scan the occupancy index. Cost is O(cells touched + units found),
which is optimal to a constant.

**Nearest neighbour**: expand in hex rings from the origin cell. Stop
when the ring's minimum possible distance exceeds the best found. Use
the L1 summaries to skip empty rings wholesale. For batched nearest
neighbour, see §8.2. The batched form is much better.

**Area of effect**: this is a range query plus a write. Note the write
side: an AoE that hits 10,000 units must produce 10,000 events into a
thread-local buffer, not 10,000 direct mutations. That keeps it inside
the brief's event-apply model.

### 4.2 A concern about L2 size

With 32x32 blocks at both levels and a 4096x4096 map:

- L0: 4096 x 4096 = 16.7M
- L1: 128 x 128 = 16,384
- L2: 4 x 4 = 16

Sixteen L2 cells prune almost nothing. Each covers a quarter of the map
in each axis. A selector that touches any part of the map will match
most of them.

Two options:

- **Use 16x16 blocks.** Then L0 -> 256x256 (L1) -> 16x16 (L2). L2 holds
  256 cells, which prunes well. L1 cells hold 256 tiles, which is still
  a good scan granule.
- **Add an L3.** Keep 32x32 and accept that L2 is the "continent" tier,
  then add a third level. This costs more maintenance for little gain.

**Recommendation: 16x16 blocks, three levels.** It gives a better
branching factor for hierarchical descent. Also flag this as an open
question, because the right answer depends on the real map size, and the
brief lists map size as unresolved.

### 4.3 Line of sight and field of view on hex

Do not draw a Euclidean line to every hex. Lines overlap heavily, so you
examine most hexes several times.

Use the expanding-bubble form of shadowcasting. An obstacle casts an
angular shadow. Everything behind it in that angular interval is hidden.
Amit Patel's treatment of Clark Verbrugge's hex grids describes exactly
this framing.
([Hex LOS, Amit Patel](http://www-cs-students.stanford.edu/~amitp/Articles/HexLOS.html))

The clean adaptation: **recursive shadowcasting over 6 sextants instead
of 8 octants.** Classic recursive shadowcasting splits the plane into 8
octants and scans rows in ascending distance, tracking a shadow slope
interval.
([Bergström's recursive shadowcasting, and a survey of FOV algorithms](https://arxiv.org/pdf/2101.11002))

On hex, a sextant is the natural unit. Within a sextant, index cells by
`(ring, position_in_ring)`. Scan by increasing ring. Maintain a list of
open angular intervals in units of `position / ring`. When a cell blocks,
subtract its interval. This is O(cells in radius), which is O(3 R^2) for
radius R.

Concrete: R = 12 gives 469 hexes. Estimate **2-6 us per FOV**.

**This is the real scaling problem.** 1M units at 3 us each is 3 seconds.
You cannot compute FOV per unit per tick. Mitigations, in order:

1. **Only recompute for units that changed tile.** Most units do not move
   every tick. Typical churn is 5-20%.
2. **Share FOV across stacked units.** Units on the same tile with the
   same sight radius have identical FOV. Group by `(tile, radius)` before
   computing. In dense armies this collapses thousands of computations
   into dozens.
3. **Quantize sight radius.** Round to a small set of values, e.g. 4, 8,
   12, 16. This makes step 2 far more effective.
4. **Cache by (tile, radius) with terrain-version validation.** A tile's
   FOV changes only when nearby terrain changes.

With all four, expect 10,000-50,000 real FOV computations per tick, so
**30-300 ms core time**, which is 3-25 ms on 12 cores. Still heavy.
Consider running FOV at a lower rate than the sim tick, e.g. every third
tick.

### 4.4 Fog of war

Represent it per faction, with two bitsets and one counter array.

| Structure | Type | Size at 16.7M tiles |
|---|---|---|
| Explored (ever seen) | 1 bit/tile | 2.1 MB |
| Visible (seen now) | 1 bit/tile | 2.1 MB |
| Visibility count | u8/tile | 16.8 MB |

The counter array is what makes updates incremental. When a unit's FOV
gains a tile, increment. When it loses one, decrement. A tile is visible
when its count is above zero. Without the counter you would have to
rebuild the whole visible set each tick.

**Do not rebuild the visible bitset from scratch each tick.** That costs
2.1 MB of clear plus a full re-scatter per faction.

The u8 counter saturates at 255. That is a real risk with 1M units in a
small area. Use u16 (33.5 MB per faction) if you expect deep stacking, or
saturate and accept a small leak. **Recommendation: saturating u8, plus a
periodic full rebuild every N ticks to correct drift.** Choose N around
600. Amortize the rebuild across ticks by rebuilding one L1 row per tick.

Memory scales linearly with faction count. Eight factions at u8 cost
about 168 MB. If you need many factions, store fog only for factions with
an observer, or share fog within an alliance.

Deliver fog to Python as a NumPy view over the explored/visible bitsets.
Never as per-tile calls.

---

## 5. Parallel scheduling

### 5.1 Prior art

**Bevy** builds a `ScheduleGraph` from declared system parameters. Each
system reports a `FilteredAccess` set of component reads and writes. The
scheduler finds conflicting pairs, topologically sorts, and the
`MultiThreadedExecutor` dispatches systems whose access does not conflict
with the currently running set.
([bevy_ecs::schedule docs](https://docs.rs/bevy_ecs/latest/bevy_ecs/schedule/index.html))

Bevy also has an **ambiguity detection** pass. It reports pairs of
systems that could run in either order and that touch the same data. The
known limitation is that the check ignores query filters, so it reports
false conflicts.
([bevy#11796](https://github.com/bevyengine/bevy/issues/11796))

**Legion** took a similar declared-access approach, with a strong emphasis
on archetype-level access rather than component-level. That is a better
fit for an archetype-chunked store, because two systems that touch the
same component but disjoint archetypes do not actually conflict.

### 5.2 Recommendation: a static compiled schedule

Do this, and do not copy Bevy's dynamic executor.

1. Each system declares its access: component reads, component writes,
   resource reads, resource writes, and whether it makes structural
   changes.
2. At registration time, build the conflict graph. Two systems conflict
   when one writes what the other reads or writes, **and** their archetype
   sets intersect.
3. Greedily colour the graph into **stages**. All systems in a stage run
   at once. Stages run in a fixed order.
4. Freeze that stage list. Store it. Assert it does not change at runtime.

**Why static, not dynamic:**

- **Determinism.** In a dynamic executor, the order in which two
  non-conflicting systems complete depends on thread timing. If those
  systems have any hidden shared state, and eventually one will, results
  vary run to run. A static stage list removes the variable.
- **Overhead.** Dynamic dispatch costs an atomic access check per system
  start. With few, wide systems, that overhead is noise either way. With
  many narrow systems, static wins.
- **Debuggability.** You can print the schedule. You can diff it across
  builds. A user can see why two systems do not run at once.

**Cost:** you lose some parallelism when stages are uneven. Accept it.
§5.3 explains why it barely matters here.

Keep an ambiguity report as a *development tool*. Warn when two systems
in the same stage touch the same data with no declared ordering.

### 5.3 Get parallelism from data, not from systems

This is the most important scheduling point in the report.

At 1M entities, a single system over all units already saturates 12 cores
through `par_chunks`. Running two systems side by side adds nothing, and
it costs determinism risk.

So the schedule should be **narrow and deep**: a small number of stages,
each holding one or two very wide systems. Not **wide and shallow**: 40
small systems packed into 5 stages.

This also simplifies the conflict graph, because there is less to
conflict.

### 5.4 Rayon usage patterns

**Use `par_chunks_mut` with an explicit chunk size. Do not use the
default splitting.**

```rust
data.par_chunks_mut(CHUNK).for_each(|slice| { ... });
```

Guidance:

- **Chunk size.** Aim for 50-500 us of work per chunk. Too small and
  task overhead dominates. Too large and the tail straggles. For a 20 ns
  per-unit kernel, that means 4,000-25,000 units per chunk. Round to a
  multiple of the archetype chunk size so you never split a chunk.
- **Overhead threshold.** A rayon task costs roughly 1-5 us to spawn and
  join. Below about 10,000 total elements of light work, a serial loop
  wins. Add a guard: `if len < SERIAL_THRESHOLD { serial() } else { par() }`.
- **False sharing.** Two threads writing bytes in the same 64-byte line
  cause cache-line ping-pong, which can cost 10x. Align every parallel
  partition to 64 bytes. The brief already specifies 64B-aligned chunks.
  Keep that invariant for every parallel split, including bitsets: split
  bitsets on 64-*word* boundaries, not bit boundaries.
- **Work stealing and determinism.** Rayon steals work. The *order* of
  completion is not deterministic. The *result* must be.

**How to keep determinism under work stealing.** Follow three rules.

1. **Disjoint output.** Each task writes only to its own slice. No task
   reads another task's output. This makes order irrelevant.
2. **Deterministic reduction.** Never use a lock-protected shared
   accumulator, and never use `par_iter().sum()` on floats. Instead give
   each task an indexed output slot, then combine slots in index order on
   one thread. `fold` plus an ordered `reduce` over a fixed slot array is
   correct. A free-form `reduce` over a work-stealing tree is not, for
   floats.
3. **Deterministic concatenation.** For thread-local event buffers,
   allocate one buffer per *chunk index*, not per *thread*. Then
   concatenate in chunk-index order. The brief's decision 12 already says
   this. It is the right rule. Make it a hard invariant, and write a test
   that runs the same tick with 1, 2, and 12 threads and compares the
   event log byte for byte.

That three-thread-count test is the single highest-value test in the
project. Add it early.

### 5.5 Aggregates as the conflict domain for commands

The brief's decision 9 makes the aggregate boundary the parallelism
boundary. Make that concrete.

Define the aggregate as an **L1 cell**. Then:

- A command whose write set falls inside one L1 cell is local.
- Two commands with disjoint L1 write sets run at once.
- A command that spans cells takes all of them, in ascending cell ID
  order, to avoid deadlock.

Compute each command's write set during validation, before execution.
Then run a conflict-free scheduling pass: sort commands by issue order,
greedily assign each to the earliest batch whose L1 cell set it does not
touch. Execute batches in order, and each batch in parallel.

This preserves the brief's requirement, because a command's *effect*
still lands in issue order relative to any command it conflicts with,
and non-conflicting commands cannot observe each other.

**Failure mode:** a global command, e.g. "all units of faction X retreat",
has a map-wide write set. It serializes everything. Detect this case and
handle it differently: convert it into a data-parallel pass over the
selector result, rather than into a command with a write set.

**Open question for the ADR author:** does a command *read* outside its
write set? "Attack the nearest enemy" reads far and writes near. If reads
count in the conflict domain, parallelism collapses. The usual answer is
to snapshot: all commands read the state at the start of the barrier, and
writes land after. That makes reads conflict-free by construction. It
also costs a snapshot. Decide this explicitly.

---

## 6. The frame loop

### 6.1 Phase breakdown

The brief lists the phases. This is a concrete ordering with the barriers
named and justified.

```
  [ Python phase — GIL held by Python ]
 0. Python builds selectors and queues commands.
    No Rust state changes. Commands are inert data.

--- BARRIER 1: SEAL ------------------------------------------
    The command queue closes. Assign each command a sequence
    number. Release the GIL. Rust owns the world from here.
    Why here: it fixes the deterministic order once and for all.

  [ Rust phase — GIL released ]
 1. RESOLVE.   Evaluate every selector against the pyramid.
               Read-only. Fully parallel across commands.
               Output: an entity ID set per command.

 2. VALIDATE.  Check preconditions per command. Compute each
               command's L1 write set. Read-only. Parallel.
               Output: accepted set, rejection summaries.

 3. PLAN.      Batch commands into conflict-free groups (§5.5).
               Single-threaded, cheap: O(commands log commands).

--- BARRIER 2: PLAN COMPLETE ---------------------------------
    Why here: execution needs the full batching decision.

 4. EXECUTE.   For each batch in order, run its commands in
               parallel. Commands emit EVENTS. They do not
               mutate the world.
               Interleaved here: the wide data-parallel systems
               (movement integration, combat resolution, AI
               tick), each as a schedule stage (§5.2).
               All output goes to per-chunk-index event buffers.

--- BARRIER 3: EVENTS SEALED ---------------------------------
    Concatenate event buffers in chunk-index order. One
    deterministic event stream now exists.
    Why here: apply must see a fixed, ordered stream.

 5. APPLY.     Walk the event stream. Mutate component values.
               Mark tiles and L1 cells dirty as you go.
               Parallel only where events are pre-partitioned
               by target; otherwise single-threaded. Apply is
               pure and fast: it is a memory scatter.

 6. STRUCTURAL. Spawn, despawn, archetype moves, occupancy
               index rebuild. Single-threaded, or parallel per
               archetype. Kept separate because it invalidates
               every pointer and index.
               Why separate: no system may hold a reference
               across this phase.

--- BARRIER 4: STRUCTURE STABLE ------------------------------

 7. PYRAMID.   Walk the L1 dirty bitset. Recompute dirty cells
               from L0. Mark L2 parents dirty. Repeat upward.
               Parallel over disjoint dirty cells per level,
               with a barrier between levels.
               Why after structural: summaries must count the
               entities that now exist.

 8. FOG/FOV.   Recompute FOV for units that changed tile.
               Update visibility counters. Parallel.
               (Optionally every Nth tick.)

--- BARRIER 5: FRAME COMPLETE --------------------------------
    Reacquire the GIL.

  [ Python phase ]
 9. DELIVER.   Hand the event batch to Python as arrays.
               Hand command result summaries back.
```

Five barriers. Each one exists because the next phase needs a property
that the previous phase establishes. Do not add more.

**Note the read/write split.** Phases 1-4 read the world and write only
events. Phases 5-8 write the world and read only events. That split is
what makes phase 4 safely parallel, and it is what answers the open
question in §5.5. Snapshotting is unnecessary, because nobody writes
during the read phase.

Make this split an enforced type-level invariant, not a convention. Give
phase 1-4 systems a `&World` and an `&mut EventSink`. Give phase 5-6
systems a `&mut World` and an `&EventStream`.

### 6.2 Fixed timestep

Use a fixed timestep. Reasons:

- Determinism requires it. With a variable `dt`, floating-point results
  differ between machines and between runs.
- Replay requires it. Tick N must mean the same thing every time.
- The RL audience requires it. `step()` must be a well-defined unit.

Run the sim at 10-30 Hz. A strategy simulation does not need 60 Hz
logic. Render at display rate and interpolate positions between the last
two sim states. Store two position buffers and lerp.

Handle the slow-tick case by **dropping**, not by accumulating. If the
sim falls behind, run one tick and report the overrun. Never run a
catch-up loop of 10 ticks, because that produces a spiral.

For the RL audience, expose a mode with no wall clock at all: `step()`
runs exactly one tick and returns. That is the natural shape already.

### 6.3 Sub-tick systems

Not everything needs to run every tick. Give each system a period and a
phase offset.

- Movement: every tick.
- Combat: every tick.
- FOV: every 2-3 ticks.
- Economy and production: every 10 ticks.
- Long-range AI re-plan: every 30 ticks, staggered by faction.

Stagger the offsets so the work spreads. Keep the periods as constants,
because a data-driven period is another determinism hazard.

---

## 7. Simulation LOD

### 7.1 The choice

Two options for regions with no observer:

- **Freeze.** Stop simulating. Record the tick at which you froze.
- **Coarse simulate.** Run a cheap approximate model at the L1 level.

The brief defers this and leaves it as an open question. This section
gives evidence and a staged recommendation.

### 7.2 Prior art, and what each one actually did

**Factorio** is the cleanest engineering example. It maintains **active
chunk** lists. Only chunks with something to do get updated. Pollution
updates run per chunk every 64 ticks. A chunk starts diffusing once it
holds 15 units of pollution, and it sends 2% to each cardinal neighbour
per update.
([Factorio Wiki: Pollution](https://wiki.factorio.com/Pollution))

Two lessons from Factorio. First, **the coarse layer is a different model,
not a downsampled version of the fine model.** Pollution is a diffusion
field. It is not "biters, simulated cheaply". Second, **the coarse layer
runs at a lower rate**, 64 ticks, not every tick. Factorio's forums also
record that a full world sweep, even once per second, produces a visible
spike. So they spread chunk updates across ticks.
([Factorio forums: active chunks](https://forums.factorio.com/viewtopic.php?t=107797))

**Dwarf Fortress** keeps the fine simulation for the fortress only. Off
site, it runs a much simpler world model with abstract armies and
populations. Promotion back to fine detail happens rarely, and it happens
through a scripted site-generation step.

**Paradox grand strategy** never runs a fine layer at all. Everything is
provinces and pops. There is no promotion problem, because there is
nothing to promote to.

**RimWorld** and **Oxygen Not Included** avoid the problem by keeping the
map small enough to simulate fully. RimWorld abstracts only off-map
caravans, using a simple travel and event model.

**Kenshi** streams a large world but keeps only nearby zones active. It
is a well-known source of "the world only happens near the player"
complaints, which is the honest cost of freezing.

**MMO interest management** solves a different problem: which clients
need which updates. The server simulates everything. The relevance filter
is about network bandwidth, not about compute. The useful transferable
idea is the **area-of-interest** structure: a subscription set per
observer, updated on region crossing. That maps directly onto the L1 cell
as the subscription unit.

### 7.3 The promotion problem, stated honestly

Promotion is the hard part, and it is harder than it first sounds.

You froze a region with 12,000 units in a known arrangement. You ran a
coarse model for 4,000 ticks. The coarse model says the region now holds
"about 15,000 units, mostly infantry, morale low, 60% forested". Now the
player looks at it. You must produce 15,000 concrete units at concrete
tiles.

The problems:

1. **Underdetermination.** The L1 summary is a monoid reduction. It threw
   information away by construction. That is the point of a monoid. You
   cannot invert it. You must invent.

2. **Contradiction with memory.** The player saw a specific bridge, a
   named hero, a particular fort. Your generated detail must preserve
   every fact the player could remember. So you need a set of "pinned"
   facts that survive summarization, and those pinned facts break the
   monoid property.

3. **Non-commutation.** `summarize(simulate_fine(X))` does not equal
   `simulate_coarse(summarize(X))`. The two models drift. The longer the
   freeze, the worse the drift. There is no general fix. You can only
   bound the error by keeping the coarse model conservative.

4. **Re-summarize on exit.** After the player leaves, you summarize the
   materialized detail. But some of that detail was invented. If the
   player re-enters, you invent again from the summary of inventions.
   Errors compound.

**Determinism helps but does not solve it.** Seeding generation from
`hash(region_id, tick, history_digest)` makes the invention reproducible.
It does not make it consistent with what the player remembers.

### 7.4 Staged recommendation

**Stage 1 (v1). Freeze, with a resume model.** Do not simulate. Record
`frozen_at_tick`. On thaw, apply a deterministic catch-up function to the
L0 state: grow resource stockpiles by rate times elapsed, advance
production queues, age units. This is pure L0 arithmetic. It has no
promotion problem, because you never destroyed the L0 detail.

This is a much better first step than it sounds. The L0 tile data is
dense and small (about 134 MB for the whole map). You do not need to
discard it. You only need to stop *processing* it.

**Stage 2. Active-set simulation, Factorio style.** Keep the frozen L0
data. Maintain an active set of L1 cells. A cell is active when it holds
an observer, holds a unit with a pending command, borders an active cell,
or holds a queued event. Run the full fine simulation on the active set
only. Everything else stays frozen and gets the Stage 1 catch-up on
activation.

This gives most of the benefit. It has no promotion problem, because the
L0 data never went away.

**Stage 3 (optional). A coarse layer for genuinely global processes.**
Diffusion-like quantities only: influence, supply, pollution, migration
pressure, morale. These are exactly the things that a coarse model gets
right, because they are already averages. Run them over the whole L1
grid at a low rate, e.g. every 64 ticks. 16,384 L1 cells at a few
operations each costs microseconds.

**Stage 4 (probably never). Generative promotion from a summary.** Only
build this if the map genuinely cannot hold full L0 state. At 4096x4096
it can. Revisit if the target grows to 65536x65536.

**The key insight: you only need generative promotion when you discard
L0 data. At this map size you do not have to discard it.** Freezing the
*computation* and discarding the *data* are separate decisions. The brief
conflates them slightly. Separate them, and Stage 4 disappears.

---

## 8. Batched and set-valued algorithms

This is where the set-valued command model pays off. The pattern: an
algorithm that costs `O(k * f(N))` per item becomes `O(g(N) + k)` when
you know all `k` items at once.

### 8.1 Multi-source BFS and Dijkstra

**The single best example after flow fields.**

Question: "for every tile, which of my 200 cities is nearest, and how
far?" Naive: 200 Dijkstra runs, or 16.7M nearest-city searches.

Batched: seed one priority queue with all 200 sources at distance 0, each
tagged with its source ID. Run one Dijkstra. Each cell records the
distance and the winning source. This computes the full Voronoi partition
and the distance field in **one pass**, O(N log N) or O(N) with buckets.

Uses: territory control, supply range, "nearest depot", zone of control,
threat range. All of these are the same algorithm.

Cost on the full map: 50-200 ms. Too slow per tick. Run it at L1 instead:
16,384 cells, about 50,000 edges, so **under 1 ms**. Refine to L0 only
inside cells that a boundary crosses. That is a small fraction.

### 8.2 Batched nearest neighbour

Per-item nearest neighbour costs a search each. Batched, you sort.

Sort both sets by their L1 cell ID, which is already a spatial key. Then
sweep both sorted lists together. For each cell, compare only against the
cell and its 6 hex neighbours. This is O(n log n + m log m + pairs), and
the sort is the dominant term. With a radix sort on the cell ID it drops
to O(n + m + pairs).

At 1M units, a radix sort on a u32 key costs about 4-8 ms
single-threaded, and it parallelizes well. That beats 1M individual
searches by a large factor.

**Keep the units sorted.** If units are stored in L1-cell order and stay
roughly sorted between ticks, the per-tick sort becomes an almost-sorted
insertion pass. That is nearly free. This argues for periodically
re-ordering the ECS chunks by spatial key. It also improves cache
locality for every other pass.

### 8.3 Sort-merge joins instead of per-item lookups

Any "for each unit, look up something by key" pattern is a join. The
per-item form does a hash lookup per unit: a likely cache miss, about
100 ns. At 1M units that is 100 ms.

The batched form sorts both sides by key and merges. Both sides then
stream linearly. At 1M units, expect **5-15 ms**, and it parallelizes.

Apply this to: unit-to-tile terrain lookup, unit-to-type stat-table
lookup, event-to-target apply, and upgrade modifier resolution.

**Special case: the stat table.** `UnitType(u16)` gives at most 65,536
types, and in practice a few hundred. The stat table fits in L1 or L2
cache. Do not sort for this one. A direct index into a small hot table is
already fast. This is one of the strongest arguments for the brief's
decision 8.

### 8.4 Influence maps and diffusion

An influence map is a scalar field over the grid that answers "how
dangerous is here" or "how valuable is here". It is the standard tool for
strategic AI, and it is naturally batched.

Two forms.

**Diffusion (iterative).** Repeat: each cell becomes a weighted blend of
itself and its 6 neighbours, with a decay factor. One pass over the L1
grid (16,384 cells) costs about 8 KB of traffic and a few microseconds.
Ten iterations still cost microseconds. Run it every tick at L1. Run it
at L0 never.

This is a 7-point stencil. It vectorizes perfectly (§9).

**Falloff scatter (direct).** For each source, add `strength / (1 + d)`
to cells within radius R. Cost O(sources * R^2). Prefer diffusion when
sources are many, and scatter when sources are few and radii small.

**Scent and gradient maps.** A diffusion map with a decay gives a
gradient that agents can follow. This gives cheap emergent behaviour:
"walk uphill on the food map" needs no pathfinding at all. Combine with
flow fields: use the flow field for the commanded destination, and the
scent gradient for uncommanded idle behaviour.

**Recommendation:** maintain 4-8 influence maps at L1, one per faction
per concern (threat, supply, value). Each is 16,384 floats, so 64 KB.
Eight maps cost 512 KB, which fits in L2 cache. Update all of them in one
fused pass. Budget **under 1 ms**.

Expose them to Python as NumPy views. Researchers will want exactly this.

### 8.5 Bulk economic and resource propagation

Resource flow is a sparse linear system, or a flow problem on a small
graph. Do not simulate it per unit.

Build the graph at L1: nodes are cells with production or consumption,
edges are supply routes. That graph is small, typically thousands of
nodes. Solve it with a few Gauss-Seidel or Jacobi iterations per tick,
not to convergence. The error decays over ticks, and the simulation looks
smooth because of it.

Cost: thousands of nodes times a handful of iterations, so
**microseconds**. Compare with per-unit resource ticking at 1M units,
which would cost milliseconds for a worse answer.

### 8.6 Batched sorting is the common primitive

Notice how often "sort by spatial key" appeared above: nearest neighbour,
joins, event apply, chunk locality. **Invest in one very good parallel
radix sort on u32/u64 keys.** It will be the second most used primitive
after the flow tile. Consider `rdst` or a hand-written LSD radix sort
with per-thread histograms and a deterministic prefix-sum combine.

Make it a *stable* sort. Stability preserves issue order within equal
keys, which the brief's decision 9 requires.

---

## 9. SIMD kernels

### 9.1 Which kernels are worth it

Hand-vectorizing is expensive to write and to maintain. Do it only where
the kernel is hot, simple, and branch-free. Five candidates, ranked.

**1. Movement integration.** `pos += vel * dt` over 1M units, with a
flow-direction lookup. The lookup is a gather, which is the awkward part.
Split it: gather directions in one pass, integrate in a second. The
second pass is pure SoA arithmetic and vectorizes at 8 lanes with AVX2.
Expected speedup: 3-6x on the arithmetic. It becomes bandwidth bound
after that, which is the correct place to stop.

**2. Influence-map diffusion.** A 7-point hex stencil over 16,384 cells,
repeated. Perfectly vectorizable, entirely in cache. Expected speedup:
4-8x. Small absolute time, but it runs often.

**3. Bitset operations.** Fog of war, dirty bitsets, faction masks.
`and`, `or`, `andnot`, and `popcount` over multi-megabyte bitsets. These
already vectorize with plain `u64` chunks. AVX2 gives another 2-4x. Use
`u64` arrays and let LLVM auto-vectorize first. Measure before writing
intrinsics. Often you do not need to.

**4. The selector predicate scan.** "All units where faction == X and
health < Y". This is a filtered compaction over SoA arrays. Compare to
build a mask, then compact. AVX-512 has `compress`; AVX2 needs a shuffle
table. Expected speedup: 4-8x. This one is worth hand-writing, because
selectors are the core API.

**5. L0 to L1 reduction.** Summing and max-ing 256 or 1024 tiles into
one summary. Horizontal reductions vectorize well. Histogram building
does not, because it is a scatter. Split the summary: vectorize the
sum/min/max fields, and leave the histogram fields scalar.

**Do not vectorize:** pathfinding (branchy, pointer-chasing), the event
apply scatter (random writes), or anything with a priority queue.

### 9.2 The Rust SIMD landscape

**`std::simd` / `core::simd` is still nightly-only.** It sits behind the
`portable_simd` feature, tracking issue #86656. It is not stable, and the
API can change. Using it means pinning a nightly toolchain.
([std::simd docs](https://doc.rust-lang.org/std/simd/index.html),
[portable-simd stabilization blockers](https://github.com/rust-lang/portable-simd/issues/364))

For a library that ships abi3 wheels to three platforms, a nightly
requirement is a serious packaging cost. Avoid it on the default path.

**`wide`** is the mature stable-Rust option. It gives fixed-width types
like `f32x8`, and it supports SSE, AVX, NEON and WASM. Its gap is
**runtime multiversioning**: it does not select an implementation based
on detected CPU features. You compile for one target level.
([The State of SIMD in Rust in 2025](https://shnatsel.medium.com/the-state-of-simd-in-rust-in-2025-32c263e5f53d))

**`pulp`** does have built-in multiversioning. Its limitation is that it
exposes only the *native* SIMD width, so your kernel must handle a
variable lane count rather than a fixed `f32x8`.
([pulp on crates.io](https://crates.io/crates/pulp))

**`fearless_simd`** combines both: pulp's multiversioning with fixed-size
chunks. It is newer and less complete. Watch it, do not depend on it yet.
([Linebender: Towards fearless SIMD](https://linebender.org/blog/towards-fearless-simd/))

**`simba`** is a genericity layer, used by nalgebra. It lets you write
code generic over lane count. It is not a SIMD backend in itself.
([simba docs](https://docs.rs/simba))

### 9.3 Recommendation

1. **Write scalar SoA code first, and check the assembly.** LLVM
   auto-vectorizes clean SoA loops well. Confirm with `cargo asm` or
   Godbolt before writing anything by hand. Many of the five kernels
   above may need no manual work.

2. **Default target: x86-64-v2 (SSE4.2) baseline plus runtime dispatch to
   AVX2.** Do this with `is_x86_feature_detected!` plus
   `#[target_feature(enable = "avx2")]` functions. Detect once at startup
   and store a function pointer. Do not detect per call.

3. **Use `wide` for hand-written kernels on stable.** Pair it with the
   manual dispatch above. This gives you both fixed widths and
   multiversioning, at the cost of writing each kernel twice.

4. **Put `std::simd` behind an optional `nightly-simd` feature.** Let
   power users opt in. Never require it.

5. **Benchmark every SIMD kernel against its scalar version, on every
   platform, in CI.** SIMD kernels regress silently when a compiler
   version changes.

6. **Determinism caution.** Float SIMD reductions reorder additions.
   `a + b + c + d` in scalar order differs from lane-wise pairing. If you
   need bit-exact cross-platform results (the brief's open question),
   then **either use integer or fixed-point arithmetic for all
   simulation-critical values, or fix the reduction order explicitly**.
   Recommendation: use fixed-point for positions and health. Use floats
   only for presentation and for influence maps, whose exact values do
   not gate branches.

---

## 10. Summary table of budgets

Estimates for one tick at the stated scale, on 12 cores.

| Work | Scale | Core-ms | Wall-ms (12c) |
|---|---|---|---|
| Selector resolution, pyramid descent | 100 commands | 5-20 | 0.5-2 |
| Portal-graph A* | 100 plans | 5-50 | 0.5-4 |
| Flow tile builds | 100 tiles | 0.5-4 | 0.1-0.5 |
| Movement + steering | 1M units | 20-40 | 2-4 |
| Combat resolution | 100k engaged | 5-20 | 0.5-2 |
| Event concatenate + apply | 500k events | 10-30 | 2-6 |
| Structural changes | 10k spawns | 2-10 | 2-10 (mostly serial) |
| Pyramid dirty update | 5% dirty | 5-15 | 0.5-2 |
| FOV recompute (shared, quantized) | 30k real FOVs | 30-150 | 3-13 |
| Fog counter update | 30k deltas | 3-10 | 0.3-1 |
| Influence maps at L1 | 8 maps | 1-3 | 0.1-0.3 |
| Spatial radix sort | 1M units | 4-8 | 0.5-1 |
| **Total** | | **90-360** | **12-46** |

At 30 Hz (33 ms per tick) this is tight but reachable. At 10 Hz
(100 ms) it is comfortable.

**The two largest line items are FOV and event apply.** Attack those
first if the budget breaks. FOV responds well to the sharing and
quantization tricks in §4.3. Event apply responds to pre-partitioning
events by target region.

---

## 11. Failure modes to design against

1. **The full-map pass creeping in.** Someone adds "just one loop over
   all tiles" per tick. It costs 3.3 ms. Ten of them cost the frame. Add
   a debug counter that reports full-map passes per tick, and fail a test
   when it exceeds a threshold.

2. **Flow-tile cache thrash.** Many small groups with many distinct
   destinations defeat the cache. Detect a low hit rate. Fall back to
   a coarser destination (snap the goal to the nearest portal) to force
   sharing.

3. **The one-big-blob crowd.** 100,000 units into one chunk. The
   occupancy index for those tiles grows huge, and the separation term
   becomes a long scan. Cap the per-tile unit list length, or store a
   count plus a sample.

4. **A pathological dirty set.** A player action dirties every L1 cell.
   The pyramid update becomes a full rebuild. Bound the per-tick repair
   work and spread the rest.

5. **Nondeterminism from float reduction.** Silent, and it appears only
   under load, when work stealing changes the split. The 1/2/12 thread
   test in §5.4 catches it.

6. **Rayon oversubscription.** Nested `par_iter` inside a `par_iter`
   causes deep task trees and poor locality. Use one level of
   parallelism per stage. Configure one global rayon pool with a thread
   count fixed at startup, and record that count in the replay header.

7. **Straggler tasks.** One chunk holds all the work, and 11 cores idle.
   Balance by splitting on estimated work, not on element count. For
   units, element count is a good proxy. For chunk-level work it is not.

---

## 12. Open questions for the ADR author

1. **L2 block size.** 32x32 gives only 16 L2 cells at 4096x4096. Is 16x16
   with three levels better? (§4.2)

2. **Determinism target.** Bit-exact across platforms, or within a run?
   This decides fixed-point versus float, and it decides how much freedom
   the SIMD kernels get. (§9.3)

3. **Does a command's read set count as a conflict?** The phase split in
   §6.1 says no, because reads and writes are separated in time. Confirm
   that no verb needs to read its own writes.

4. **Sight radius quantization.** How many distinct values? Fewer means
   much cheaper FOV. Is 4 values acceptable to the game design?

5. **Fog per faction, or per alliance?** Per faction costs 19 MB each.
   How many factions must be supported at once?

6. **Does the ECS get re-sorted by spatial key?** It improves nearly
   every pass, and it costs one sort per tick. It also changes entity
   iteration order, which interacts with determinism.

7. **Is the sim tick 10 Hz or 30 Hz?** The whole budget follows from
   this. It should be an explicit decision, not an emergent one.

---

## Sources

- Emerson, *Crowd Pathfinding and Steering Using Flow Field Tiles*, Game
  AI Pro —
  https://www.gameaipro.com/GameAIPro/GameAIPro_Chapter23_Crowd_Pathfinding_and_Steering_Using_Flow_Field_Tiles.pdf
- How To RTS: Basic Flow Fields —
  https://howtorts.github.io/2014/01/04/basic-flow-fields.html
- How To RTS: Continuum Crowds —
  https://howtorts.github.io/2014/01/09/continuum-crowds.html
- Botea, Muller, Schaeffer, *Near Optimal Hierarchical Path-Finding* —
  https://webdocs.cs.ualberta.ca/~mmueller/ps/hpastar.pdf
- Amit Patel, *Clark Verbrugge's Hex Grids* (hex LOS) —
  http://www-cs-students.stanford.edu/~amitp/Articles/HexLOS.html
- *New Algorithms for Computing Field of Vision over 2D Grids* —
  https://arxiv.org/pdf/2101.11002
- bevy_ecs::schedule documentation —
  https://docs.rs/bevy_ecs/latest/bevy_ecs/schedule/index.html
- Bevy issue #11796, ambiguity detection and query filters —
  https://github.com/bevyengine/bevy/issues/11796
- Factorio Wiki, Pollution —
  https://wiki.factorio.com/Pollution
- Factorio forums, active chunks —
  https://forums.factorio.com/viewtopic.php?t=107797
- Davidoff, *The State of SIMD in Rust in 2025* —
  https://shnatsel.medium.com/the-state-of-simd-in-rust-in-2025-32c263e5f53d
- std::simd documentation (nightly) —
  https://doc.rust-lang.org/std/simd/index.html
- portable-simd, critical issues before stabilization —
  https://github.com/rust-lang/portable-simd/issues/364
- pulp crate —
  https://crates.io/crates/pulp
- Linebender, *Towards fearless SIMD, 7 years later* —
  https://linebender.org/blog/towards-fearless-simd/
- simba documentation —
  https://docs.rs/simba
