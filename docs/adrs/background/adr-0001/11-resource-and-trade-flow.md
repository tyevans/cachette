# Resource and Trade Flow

Research report 11 for the foundational architecture decision record.

## 0. What this document decides

The engine simulates a hex world of 16,777,216 tiles and about one million
units.[^1] The world has three levels of detail. Level 0 holds 4096 x 4096
individual tiles. Level 1 holds 256 x 256 = 65,536 cells, each of which
summarises a 16 x 16 tile block. Level 2 holds 16 x 16 = 256 cells, each of
which summarises a 16 x 16 block of level 1 cells. Level 0 is the only source
of truth. Level 1 and level 2 are derived projections. The engine already
keeps a dirty bitset for each level.[^1]

This document answers one question: how does the engine move resources
between places, and between factions, without a global solve that costs more
than a frame.

The short answer: **hierarchical diffusion in flux form.** The engine solves
a plan at level 2 with an exact minimum cost flow, on an event schedule. It
then runs a fixed small number of conserving diffusion sweeps at level 1
every tick. It never solves a flow problem at level 0.

The report states the cost of each rejected option, so a later reader can
check the reasoning rather than repeat the survey.

## 1. Terms used in this document

**Commodity.** One kind of resource, for example grain or iron. A commodity
has its own stock value in each cell.

**Flow network.** A directed graph. Each node holds a supply value or a
demand value. Each arc holds a capacity and a unit cost.

**Minimum cost flow.** The problem of moving all supply to all demand at the
lowest total cost, without exceeding any arc capacity.

**Flux.** The signed quantity that crosses one arc in one step. A positive
flux leaves the lower-indexed node and enters the higher-indexed node.

**Conservation.** The property that the sum of all stocks does not change
when the engine moves resources. Only production and consumption change the
sum.

**Monoid.** A combine operation that is associative and has an identity
element. The pyramid requires a monoid, because it combines cells in an order
that the scheduler chooses.[^1]

**Relation plane.** A bit matrix over factions. Row `i` of the war plane
holds one bit for each faction that faction `i` fights. The fog of war design
already specifies three such planes at 1,536 bytes for 64 factions.[^2]

**Q16.16.** The fixed-point scale that the project uses everywhere. One unit
of the integer representation equals 1/65536 of one logical unit.[^1]

## 2. Executive summary

Ten recommendations. Section numbers give the supporting argument.

1. **Solve the plan at level 2. Move the goods at level 1. Never solve at
   level 0.** Level 2 has 256 nodes, so an exact solver runs in tens of
   microseconds. Level 1 has 65,536 nodes, so an exact solver costs hundreds
   of milliseconds and cannot run each tick. Level 0 has 16.7 million nodes,
   so an exact solver needs gigabytes of state. See section 5.
2. **Use diffusion in flux form at level 1, not a value average.** The engine
   computes a flux for each arc, then applies the negative value to the
   source cell and the positive value to the target cell. This conserves
   exactly in integer arithmetic. A value average does not. See section 6.
3. **Require exact conservation. Do not accept approximate conservation.**
   Exact conservation costs one extra pass and it removes a whole class of
   determinism bug. See section 7.
4. **Run the level 1 sweep every tick. Recompute the level 2 plan on an
   event.** The per-tick cost is fixed and small. The event cost is larger
   and rare. See section 8.
5. **Treat commodities as independent single-commodity problems. Do not
   build a coupled multi-commodity solver.** Integer multi-commodity flow is
   NP-hard.[^3] Split shared arc capacity between commodities in proportion
   to demand, and round the split with the largest remainder method. See
   section 9.
6. **Cap the commodity count at 16. Make 32 a hard build-time limit.** Cost
   and memory both scale linearly with the commodity count. See section 9.
7. **Partition the map into trade blocs, one bloc for each connected group
   of factions that may trade.** Then the total work stays proportional to
   the cell count. It does not scale with the faction count. See section 10.
8. **Store one shared arc geometry and one flux array for each commodity.**
   The level 1 network costs 4.7 MB once, plus 1.5 MB for each commodity. See
   section 11.
9. **Express the whole solver in the existing kernel vocabulary.** One
   stencil computes the potential. One map computes the arc fluxes. One
   gather sums the incident fluxes for each cell. One scan drives the
   largest remainder rounding. There are no atomic operations and no shared
   writes. See section 12.
10. **Drive carrier agents from the flow field. Do not let carrier agents
    drive the economy.** A visible cart reads the level 1 flux field and
    carries no authoritative state. See section 13.

## 3. Operations research: the exact methods

### 3.1 Maximum flow and minimum cost flow

The standard reference for this whole family is the network flows
textbook.[^4] Three implementation families matter here.

**Network simplex.** A specialisation of the linear programming simplex
method to a flow network. It keeps a spanning tree basis and pivots one arc
at a time. A polynomial time primal variant exists.[^5] The practical
behaviour is much better than the bound suggests. A large experimental
comparison found that network simplex is the fastest method on most instance
families, and that its advantage is largest on sparse graphs.[^6] A hex grid
is sparse, with six arcs for each node.

**Push-relabel.** The method keeps a node height function and a node excess
value. It pushes excess to lower neighbours and lifts nodes that cannot
push.[^7] The generic bound is O(V² E). The first-in first-out variant
reaches O(V³). Push-relabel solves maximum flow, not minimum cost flow.

**Cost scaling.** A push-relabel method extended to costs, with an epsilon
scaling loop over the cost magnitude. The bound is O(V² E log(V C)), where C
is the largest arc cost.[^8] The same experimental comparison found that cost
scaling beats network simplex on very large and dense instances, and loses on
sparse instances.[^6]

### 3.2 What each method costs at each pyramid level

The table gives single-core estimates on a Graviton core at about 2.6 GHz.
The arc count assumes six neighbours for each cell, counted once in each
direction. The solver state assumes 32 bytes for each arc, which is a
realistic figure for a network simplex tree basis plus a flow value plus a
reduced cost.

| Level | Nodes | Arcs | Solver state | Exact solve, one commodity | Verdict |
|---|---|---|---|---|---|
| L2 | 256 | 1,536 | 48 KB | 20 to 80 us | Feasible each tick |
| L1 | 65,536 | 393,216 | 12.6 MB | 0.3 to 2 s | Feasible as a background job only |
| L0 | 16,777,216 | 100,663,296 | 3.2 GB | minutes | Not feasible at all |

The level 1 figure follows from the published experimental results. Grid
instances near 10⁵ nodes and a few hundred thousand arcs take a fraction of a
second to a few seconds with the best available implementations.[^6] At a 10 Hz
tick the frame budget is 100 ms.[^1] A level 1 exact solve therefore needs 3
to 20 ticks. It is a background job, not a per-tick step.

The level 0 figure needs no benchmark. The solver state alone exceeds the
whole world state, which is about 160 MiB.[^1] Reject level 0 immediately.

### 3.3 The transportation problem and the assignment problem

The transportation problem is the special case of minimum cost flow on a
bipartite graph. Sources hold supply. Sinks hold demand. Every source
connects to every sink. The classical statement is old.[^9] The assignment
problem is the further special case where every supply and every demand
equals one. The Hungarian method solves it in O(n³).[^10]

These forms matter for one part of the design only. Matching a set of
caravans to a set of trade routes is an assignment problem. It is not the
main resource flow problem, because the main problem is spatial and the
graph is a grid, not a complete bipartite graph.

### 3.4 Auction algorithms

The auction algorithm solves the assignment problem by an economic analogy.
Each unassigned bidder raises the price of its best object by a small
increment, then takes it. An epsilon scaling loop reduces the increment. The
complexity with scaling is O(N A log(N C)), where A is the arc count and C is
the largest benefit value.[^11]

Two properties are relevant. The method is naturally parallel, because
bidders act independently inside one round. The method also works in integer
arithmetic, because the price increment is an integer.

One property disqualifies it as the main solver. The auction algorithm is
strongly polynomial only on the assignment problem. On a general grid
transportation problem it degrades, and the epsilon scaling loop makes the
iteration count data dependent. A data-dependent iteration count breaks the
fixed frame budget. Use the auction algorithm for caravan assignment, at a
few thousand bidders, and not for spatial flow.

### 3.5 Market equilibrium methods

Tatonnement is the price adjustment process from classical general
equilibrium theory.[^12] The auctioneer raises the price of a good with
excess demand and lowers the price of a good with excess supply. It repeats
until every excess demand reaches zero. Existence of the equilibrium is a
classical result.[^13]

Convergence is the problem. Tatonnement converges for markets that satisfy
the gross substitutes condition, and there are polynomial time results under
that condition.[^14] Faster converging variants exist for the same
restricted class.[^15] Outside that class it can cycle.

**Where tatonnement is overkill.** A world simulation does not need a proved
equilibrium. It needs prices that move in a believable direction and that
never diverge. The engine gets that from a single damped update each tick,
which is one tatonnement step and not a converged solve. Section 6.4 gives
the form.

**Where a full equilibrium solve would be wrong.** A converged price vector
each tick makes the economy react instantly to a distant event. That is both
expensive and worse as simulation. A single step per tick gives a lag that
matches the physical transport lag.

### 3.6 Multi-commodity flow

Multi-commodity flow sends K different commodities across one network. Each
commodity has its own supply and demand. All commodities share the arc
capacities. The sharing is the whole difficulty.

The fractional form is a linear program. It is solvable in polynomial time,
but the program has K times as many flow variables as the single-commodity
program. **The integer form is NP-hard, even for two commodities.**[^3] A
fully polynomial approximation scheme exists for the fractional form, and it
costs a factor near 1/epsilon² in running time.[^16]

The engine uses integers everywhere.[^1] So the exact integer multi-commodity
problem is out of reach by definition, not by budget.

**The practical consequence.** Do not couple the commodities in the solver.
Solve K independent single-commodity problems. Resolve the shared capacity
before the solves, by splitting each arc capacity between the commodities in
proportion to their demand across that arc in the previous tick. Round the
split with the largest remainder method, so the parts sum to the capacity
exactly. This is an approximation. Section 9 states its error.

## 4. Approximate and local methods

### 4.1 Diffusion and pressure equalisation

Diffusion replaces the global solve with a local rule. Each cell exchanges a
fraction of its stock with each neighbour, in proportion to the stock
difference. Repeat the rule. The stock distribution moves toward a uniform
distribution, at a rate set by the exchange fraction.

Two facts make this the right method for this engine.

**It is the stencil kernel that the engine already needs.** The influence map
design already specifies a 7-point hex stencil over the 65,536 level 1 cells,
run every tick, at a budget under 1 ms for eight maps.[^17] A commodity plane
is the same shape and the same size as an influence map. The code path
already exists.

**It runs in fixed-point without a convergence test.** The engine runs a
fixed number of sweeps each tick. It does not iterate to a tolerance. The
error decays across ticks instead of inside one tick, and the result looks
smooth because of it.[^17]

**Convergence rate.** Jacobi iteration on a lattice removes error at a rate
set by the lattice diameter. The iteration count needed to spread a
disturbance across a lattice of diameter D grows as D².[^18] Level 1 has a
diameter of 256 cells, so a global equalisation needs on the order of 65,536
sweeps. That is the honest statement of the fidelity loss: **a few sweeps
each tick move goods a few cells each tick.** They do not equalise the map.

This is why the design needs the level 2 plan. The plan carries goods across
the map. The diffusion carries goods across the neighbourhood.

**Fidelity loss against exact minimum cost flow.** Three differences.

| Property | Exact minimum cost flow | Bounded diffusion |
|---|---|---|
| Total cost of the flow | Optimal | Higher, because flow spreads instead of following one route |
| Respect for arc capacity | Exact | Exact, if the flux is clamped per arc |
| Time to reach a distant sink | Instant within one solve | One cell for each sweep |
| Response to a blocked route | Reroutes optimally | Backs up, then spills sideways |

The last row is a feature, not a defect. A blocked route that causes a
visible backlog is better simulation than a route that silently reoptimises.

### 4.2 Gradient following transport

Give each cell a potential value. Let goods move down the potential gradient.
This is the same field-based idea as the flow field movement design, which
already rejects per-unit search in favour of a shared field.[^17]

The potential for a commodity is a simple integer expression:

```
potential[c] = stock[c] * WEIGHT_STOCK
             - demand[c] * WEIGHT_DEMAND
             + plan_bias[c]
```

`plan_bias` comes from the level 2 plan. It is the term that makes local
diffusion serve a global goal. A cell that the plan marks as an exporter gets
a raised potential. A cell that the plan marks as an importer gets a lowered
potential. Goods then flow along the planned direction without any per-cell
route.

This is the key structural idea of the whole report. **The level 2 plan does
not move goods. It bends the field that moves goods.**

### 4.3 Hierarchical solving

The pyramid already exists, and it already maintains a dirty bitset for each
level.[^1] The hierarchy gives three properties that the design needs.

**A small exact problem at the top.** Level 2 has 256 cells. An exact minimum
cost flow at that size costs tens of microseconds, which is nothing.

**A cheap refinement below.** The level 2 solution gives a target net export
for each level 2 cell. Each level 2 cell contains 256 level 1 cells. The
refinement splits the target across those 256 cells in proportion to their
surplus, with a largest remainder rounding step. That is one scan and one map
over 65,536 values.

**Dirty-driven work.** A level 2 cell whose dirty bit is clear has no change
in production, in consumption, or in ownership. Its plan stays valid. The
engine reuses it. In a typical tick well under 5 percent of cells are
dirty.[^1]

The precedent is direct. The portal graph design already applies this exact
two-level pattern to movement: an abstract graph over chunks for the plan,
and a local field inside a chunk for the motion.[^17] The trade network reuses
that structure and, where a trade route follows a road, it can reuse the
portal graph itself.

## 5. Where trade should be solved

This section answers question 1 of the brief directly.

| Option | Where | Cost for each solve | Cadence possible | Verdict |
|---|---|---|---|---|
| A | Exact minimum cost flow at L0 | 3.2 GB state, minutes | never | Reject |
| B | Exact minimum cost flow at L1 | 0.3 to 2 s, 12.6 MB | every 30+ ticks, background | Reject as the main method |
| C | Exact minimum cost flow at L2 | 20 to 80 us, 48 KB | every tick, or on an event | **Accept for the plan** |
| D | Bounded diffusion at L1 | 0.05 ms for each sweep for each commodity | every tick | **Accept for the movement** |
| E | Bounded diffusion at L0 | 3.3 ms for each sweep, bandwidth bound | never | Reject |

Option E deserves its own note, because it looks cheap and is not. One full
pass over the hot level 0 tile data reads 134 MB. That costs 3.3 ms of pure
memory bandwidth.[^17] One sweep for one commodity therefore consumes 3 percent
of a 100 ms tick and 10 percent of a 33 ms tick, before any arithmetic. Eight
commodities at eight sweeps would cost 210 ms. **Level 0 resource flow is
bandwidth infeasible, not merely slow.**

**Recommendation: C plus D.** Solve the plan at level 2. Move the goods at
level 1. Level 0 holds the stock of individual buildings, and buildings draw
from the level 1 cell that contains them. Level 0 never runs a flow solve.

## 6. The recommended algorithm

### 6.1 The level 2 plan

Run once for each commodity, on the event schedule of section 8.

1. Build the level 2 trade graph. Nodes are the 256 level 2 cells. Arcs join
   adjacent cells. The arc capacity comes from the summed road capacity of
   the level 1 cells on the shared boundary. The arc cost comes from the
   summed terrain cost. Both values are already pyramid aggregates.
2. Set the node supply to `production - consumption`, summed over the level 2
   cell. This is an existing pyramid accumulator of type `Accum`, which is
   `i64`.[^1]
3. Remove every arc that the trade permission mask forbids. Section 10 gives
   the mask.
4. Balance the problem. Add one artificial sink that absorbs any net surplus,
   at a high cost. Add one artificial source that covers any net deficit, at
   a high cost. A balanced problem always has a feasible solution, so the
   solver never fails.
5. Solve with network simplex. Use integer arc costs and integer capacities.
   The result is an integer flow on each arc.
6. Convert the arc flows into a per-cell `plan_bias` value: a cell that
   exports gets a positive bias, and a cell that imports gets a negative
   bias, in proportion to its net planned flow.

Network simplex is the right choice here and not cost scaling, because the
graph is small and sparse, and the experimental evidence favours network
simplex on sparse graphs.[^6] The implementation must fix the pivot rule and
the tie-breaking order, so that the result is a pure function of the input.

### 6.2 The level 1 sweep, in flux form

Run every tick, for each commodity, for a fixed number of sweeps.

Phase A, potential. One **stencil** over the 65,536 level 1 cells. Compute
the integer potential of section 4.2. Read the stock plane, the demand plane
and the plan bias plane. Write one potential plane. Cost is 65,536 reads and
one write for each cell.

Phase B, flux. One **map** over the unique arcs. Level 1 has 65,536 cells and
about 196,608 unique arcs. For each arc compute

```
raw   = (potential[a] - potential[b]) >> DIFFUSION_SHIFT
flux  = clamp(raw, -capacity[arc], +capacity[arc])
flux  = clamp(flux, -stock[b], +stock[a])          // no negative stock
```

Write `flux` to the arc's own slot. **Each worker writes only its own arcs.
There is no shared write and there is no atomic operation.** This is the
disjoint output form that the target platform requires.[^19]

Phase C, apply. One **gather** over the cells. For each cell, read the six
incident arc slots, apply the correct sign, and sum them into an `i64`. Add
the sum to the stock. Each cell writes only its own stock.

Phase C is where conservation holds. Each arc appears exactly twice in the
gather, once with each sign. The sum over all cells of all applied deltas is
therefore zero, by construction, in exact integer arithmetic.

### 6.3 The rounding step

The shift in phase B discards a remainder. Discarding it is safe, because the
flux is antisymmetric and both endpoints see the same discarded value. The
sum still cancels.

The refinement from level 2 to level 1 is different. It divides one level 2
target across 256 level 1 cells, and the division has a remainder. Use the
**largest remainder method**, in the canonical cell order, exactly as the
`transfer` verb already specifies.[^20] The steps are the same four steps: sum
the demand, compute the scale once, write the floor for each cell, then
distribute the remainder in canonical order until the pool is exactly empty.

This is one **scan** followed by one **sort** of the remainders, or a partial
selection where a full sort is not needed.

### 6.4 Prices

Prices are optional. If the design wants them, use one damped tatonnement
step for each tick, not a converged solve.

```
excess      = demand - supply                        // i64
adjustment  = (excess * GAIN) / max(demand, supply)  // Q16.16
price      += clamp(adjustment, -MAX_STEP, +MAX_STEP)
price       = clamp(price, base_price / 4, base_price * 7 / 4)
```

The clamp to a band around a base price is the model that a shipped grand
strategy game uses. Its price band runs from 25 percent to 175 percent of the
base price, and its price response uses the normalised difference between the
buy orders and the sell orders.[^21] The band is what guarantees that the
price never diverges, and it removes the need for any convergence proof.

All arithmetic is `i64` in Q16.16. There is no floating point.

## 7. Conservation

This section answers question 2 of the brief.

**Exact conservation is required. Approximate conservation is not
acceptable.** Three reasons.

**Reason 1: the pyramid needs a monoid.** The pyramid combines cells in the
order that the scheduler picks.[^1] The combine must therefore be associative
and exact. Integer addition is. A conserving flow keeps the level 1 sum equal
to the level 0 sum, so the pyramid stays a valid projection. A leaking flow
makes the projection disagree with the truth, and no test can then tell a bug
from an accepted loss.

**Reason 2: the state hash.** The engine hashes world state every frame
against a golden file.[^1] A conservation error is a silent hash divergence
with no local cause. It is among the hardest classes of bug to find.

**Reason 3: integer arithmetic makes it nearly free.** This is the point the
brief asks the report to state, so state it plainly.

- Integer addition is associative and commutative. A parallel reduce of the
  world stock gives the same total for any thread count and any split. That
  makes conservation a testable invariant, not a hope.
- The flux form makes the two halves of every transfer come from one value.
  The engine computes `flux` once, subtracts it at one end, and adds it at
  the other end. There is no second rounding, so there is nothing to
  disagree.
- Clamping preserves conservation, because the clamp applies to the single
  shared flux value before either endpoint uses it.
- The largest remainder method distributes the remainder rather than
  discarding it. The parts sum to the whole exactly.[^20]

Contrast this with a float implementation. `a - f` and `b + f` do not
recombine exactly in binary floating point when the magnitudes differ. Every
transfer leaks. Over 16.7 million cells and 10 ticks each second, the leak
becomes visible in a day of simulated time. **The float version needs a
correction pass. The integer version needs no correction at all.**

**The one test.** Sum the whole stock plane before the trade stage and after
it. Add the production and subtract the consumption. Assert exact equality.
Run it at 1 thread, at 2 threads and at 12 threads, as the existing
determinism test already does.[^1]

**Where approximation is allowed.** The engine may be approximate about
*which route* goods take and about *how fast* goods arrive. It must be exact
about *how many* goods exist. Route optimality is a quality setting.
Conservation is an invariant.

## 8. Per-tick or event-driven

This section answers question 3 of the brief. The answer is both, at
different levels.

| Stage | Cadence | Trigger | Staleness | Core-ms |
|---|---|---|---|---|
| L1 diffusion sweep | Every tick | none | none | 0.4 to 3 |
| L1 potential rebuild | Every tick | none | none | 0.1 to 0.5 |
| L2 plan solve | Every 16 ticks | L2 dirty bit, or a diplomacy change, or a route change | up to 16 ticks, 1.6 s at 10 Hz | 0.2 to 0.6 |
| L2 to L1 bias refinement | With the plan | as above | as above | 0.1 to 0.3 |
| Price step | Every tick | none | none | under 0.1 |

**Why the movement must be per-tick.** Diffusion moves goods about one cell
for each sweep. If the engine skips ticks, goods stop moving. The visible
result is stutter in the supply of every settlement. The cost is small and
fixed, so there is no reason to skip.

**Why the plan must be event-driven.** The plan changes only when the
underlying facts change: a new road, a destroyed bridge, a captured province,
a declared war, or a large change in production. The dirty bitset already
reports exactly those changes.[^1] Recomputing an unchanged plan wastes the
whole solve.

**Bound the staleness.** Force a plan rebuild at least every 16 ticks even if
no dirty bit is set. This bounds the staleness at 1.6 seconds of simulated
time at the 10 Hz tick rate.[^1] It also spreads the work: rebuild one
sixteenth of the level 2 cells on each tick, keyed by `cell_index mod 16`.
That makes the per-tick cost flat instead of spiky.

**The precedent for the background job.** A shipped transport game runs its
cargo distribution solve on a separate thread with a deadline measured in
game days, not in ticks. The main thread joins the job when the deadline
arrives. The game exposes the recalculation interval, the deadline, and a
solver accuracy setting as user options, because the solve is expensive
enough to need tuning.[^22] That design is sound, and this report copies its
shape. The difference is size: that game solves over a few thousand stations,
while a level 1 solve here would cover 65,536 cells.

## 9. Single commodity or multi-commodity

This section answers question 4 of the brief.

**Recommendation: K independent single-commodity solves. Never a coupled
multi-commodity solve.**

The reason is not performance. It is complexity class. Integer multi-commodity
flow is NP-hard for two commodities already.[^3] The engine bans floating
point in simulated state,[^1] so the fractional relaxation is not available as
a result, only as an intermediate that would then need rounding. Rounding a
fractional multi-commodity solution back to integers reintroduces the hard
problem.

**The cost multiplier for K commodities.**

| Stage | Scaling in K | Reason |
|---|---|---|
| L1 potential stencil | linear | one plane for each commodity |
| L1 flux map | linear | one flux array for each commodity |
| L1 apply gather | linear | one stock plane for each commodity |
| L2 plan solve | linear | K independent solves |
| Arc capacity split | linear | one proportional split for each arc |
| Memory traffic | linear | this is the real limit |

So the multiplier is K, with no hidden term. That is only true because the
commodities are decoupled. A coupled solve would be superlinear and would
have no fixed cost.

**Wall time against K, at level 1, 8 sweeps for each tick, on 12 cores.**

| K | L1 plane bytes | Sweep traffic | Core-ms | Wall-ms |
|---|---|---|---|---|
| 1 | 512 KB | 1.3 MB | 0.4 | 0.05 |
| 4 | 2.0 MB | 5.2 MB | 1.6 | 0.15 |
| 8 | 4.2 MB | 10.5 MB | 3.2 | 0.3 |
| 16 | 8.4 MB | 21 MB | 6.4 | 0.6 |
| 32 | 16.8 MB | 42 MB | 12.8 | 1.2 |
| 64 | 33.6 MB | 84 MB | 25.6 | 2.4 |

**The recommended ceiling is 16 commodities. The hard build-time limit is
32.** Two reasons for the ceiling.

First, the working set. At K = 16 the level 1 planes hold 8.4 MB. That still
lives in the shared last level cache of a large Graviton part, so the sweep
stays cache resident. At K = 64 it does not, and the sweep becomes bandwidth
bound like a level 0 pass.

Second, the design does not need more. A shipped grand strategy title with an
unusually detailed market runs a few dozen goods across the entire world, and
its complexity comes from the demand model, not from the goods count.[^21] A
transport game keeps one separate graph for each cargo type, which is exactly
the decoupled form this report recommends.[^22]

**The approximation, stated honestly.** Splitting arc capacity between
commodities in proportion to previous-tick demand is a heuristic. It can
starve a commodity whose demand rises quickly, for one tick, until the split
updates. The error bound is one tick of lag on the split fraction. Two guards
make it acceptable: give every commodity a floor share of each arc, and
recompute the split each tick, which costs one map over the arcs.

## 10. Factions and diplomacy

This section answers question 6 of the brief.

**A note on the identifier width.** The brief describes a maskable 64-bit
faction identifier. The decision record currently specifies `FactionId` as
`u16`, with a `u64` bit mask over up to 64 factions in the fog of war
planes.[^1][^2] Those two statements agree in practice: the identifier is
narrow, and the *mask* is 64 bits wide. This report assumes at most 64
mask-addressable factions. If the ceiling rises above 64, every mask below
becomes an array of words and every `AND` becomes a loop. The report flags
that as an open question in section 15.

### 10.1 The naive design and why it fails

The naive design gives each faction its own commodity planes and solves once
for each faction. The cost multiplies by the faction count. At 16
commodities, 8 sweeps and 16 factions, the level 1 stage costs about 100
core-ms for each tick. That exceeds the whole per-tick budget, which the
decision record places between 90 and 360 core-ms for everything.[^17]
**Reject any design whose cost scales with the faction count.**

### 10.2 The recommended design: trade blocs

Each level 1 cell has exactly one owner faction. So the cells partition by
owner. Group the factions into **trade blocs**: a bloc is a connected group of
factions under the "may trade" relation. Compute the blocs once for each tick
by a union-find over the relation plane. At 64 factions that is 64 rows of one
`u64` word, so the whole computation costs under a microsecond.

Then assign one bloc identifier to each cell, by a map over the cell owner.

Then run **one** set of commodity planes for the whole map, and forbid every
arc whose two endpoints belong to different blocs:

```
allowed[arc] = (bloc[a] == bloc[b])
flux[arc]   &= -(allowed[arc] as i32)     // branchless zero when not allowed
```

Total work stays proportional to the cell count and the commodity count. **It
does not scale with the faction count at all.** That is the whole point of the
design.

### 10.3 Trade across a bloc boundary

Blocs are a coarse rule. Real diplomacy allows limited trade between
non-allied factions. Handle that with a per-arc permission and a per-arc
tariff, both derived from the relation planes in one pass:

| Relation | Arc capacity factor | Arc cost factor |
|---|---|---|
| Same faction | 1 | 1 |
| Allied | 1 | 1 |
| Trade agreement | 1 | 2 |
| Neutral | 1/4 | 4 |
| At war | 0 | not applicable |

Store the factors as Q16.16 integers in a 64 x 64 table, which is 4,096
entries of 4 bytes, so 16 KB. Derive the per-arc factor with one gather from
that table, keyed by the pair of owner factions. This is one map over the
arcs. The war row is already a `u64` bit row in the existing relation
plane.[^2] The blocs then become the connected groups of the "capacity factor
is non-zero" relation, rather than a strict alliance test.

### 10.4 Determinism

The union-find must produce a canonical bloc identifier. Use the lowest
faction index in the group as the bloc identifier. Then the labelling is a
pure function of the relation plane, and it does not depend on the union
order.

## 11. Storage

This section answers question 5 of the brief.

### 11.1 The transport network

Store one shared arc geometry, not one geometry for each commodity. Level 1
has 65,536 cells and about 196,608 unique arcs.

| Item | Type | Count | Bytes |
|---|---|---|---|
| Arc capacity | `i32` | 196,608 | 786 KB |
| Arc base cost | `i32` | 196,608 | 786 KB |
| Arc endpoint pair | packed `u32` x 2 | 196,608 | 1.57 MB |
| Arc permission factor | `i32` Q16.16 | 196,608 | 786 KB |
| Cell bloc identifier | `u8` | 65,536 | 64 KB |
| Diplomacy factor table | `i32` | 4,096 | 16 KB |
| **Shared geometry total** | | | **4.0 MB** |
| Flux array, for each commodity | `i32` | 196,608 | 786 KB |
| Potential plane, for each commodity | `i32` | 65,536 | 256 KB |
| Stock plane, for each commodity | `i64` | 65,536 | 512 KB |
| Demand plane, for each commodity | `i32` | 65,536 | 256 KB |
| Plan bias plane, for each commodity | `i32` | 65,536 | 256 KB |
| **Per commodity total** | | | **2.07 MB** |

The arc endpoint pair is optional on a regular grid, because the neighbour of
a cell follows from its index. Drop it and save 1.57 MB. Keep it only if the
design adds long-distance arcs such as sea routes.

**Totals.** 4.0 MB shared, plus 2.07 MB for each commodity.

| K | Total level 1 trade storage |
|---|---|
| 4 | 12.3 MB |
| 8 | 20.6 MB |
| 16 | 37.1 MB |
| 32 | 70.2 MB |

At the recommended ceiling of 16 commodities the whole trade subsystem costs
37.1 MB. The decision record puts the whole world at about 160 MiB for a rich
schema at 8 factions, and fog of war at 21.0 MB for each faction.[^1] Trade is
therefore a moderate cost, well below fog of war, and it does not grow with
the faction count.

The level 2 network is negligible: 256 cells and 1,536 arcs cost under 64 KB
in total for all commodities.

### 11.2 Per-settlement stock

A settlement is a level 0 entity. Its record for each commodity holds a
stock, a production rate, a consumption rate and a local price.

| Field | Type | Bytes |
|---|---|---|
| Stock | `i64` | 8 |
| Production rate | `i32` Q16.16 | 4 |
| Consumption rate | `i32` Q16.16 | 4 |
| Local price | `i32` Q16.16 | 4 |
| **For each commodity** | | **20** |

Store it as struct-of-arrays, one array for each field, indexed by
`settlement_index * K + commodity_index`. That keeps the per-tick production
step a flat contiguous map, which the compiler vectorises.

| Settlements | K = 4 | K = 8 | K = 16 | K = 32 |
|---|---|---|---|---|
| 1,000 | 80 KB | 160 KB | 320 KB | 640 KB |
| 10,000 | 800 KB | 1.6 MB | 3.2 MB | 6.4 MB |
| 65,536 | 5.2 MB | 10.5 MB | 21.0 MB | 41.9 MB |

Even one settlement in every level 1 cell, at 16 commodities, costs 21.0 MB.
**Per-settlement stock is not a storage problem at any plausible count.** The
network is the larger cost, and the network is still modest.

## 12. The algorithm in the engine kernel vocabulary

The engine names eight kernels: map, gather, scatter, reduce, scan, sort,
stencil and local join.[^1] The whole trade stage uses six of them, and it uses
no scatter at all. The absence of scatter is the reason there are no atomic
operations.

| Step | Kernel | Input | Output | Parallel over |
|---|---|---|---|---|
| Aggregate production to L1 | reduce | L0 settlements | L1 supply plane | L1 cells |
| Aggregate L1 to L2 | reduce | L1 planes | L2 supply | L2 cells |
| Solve L2 plan | serial | L2 graph | L2 arc flows | one thread |
| Refine plan to L1 bias | scan, then map | L2 flows | L1 bias plane | L2 cells |
| Compute permission factor | map | relation plane, owner plane | arc factor | arcs |
| Compute potential | stencil | stock, demand, bias | potential plane | L1 cells |
| Compute flux | map | potential, capacity, factor | flux array | arcs |
| Apply flux | gather | flux array | stock plane | L1 cells |
| Distribute to settlements | scan, then map | L1 stock | settlement stock | L1 cells |
| Verify conservation | reduce | stock plane | one `i64` | L1 cells |

Three notes on the target platform.

**Disjoint outputs.** The flux map writes one arc slot for each arc. The apply
gather writes one stock value for each cell. No two workers write the same
address in either step. The decision record raises disjoint outputs from
preferred to required, because the ARM memory model makes atomic operations
and false sharing more costly than on x86.[^19] This design meets that
requirement by construction.

**Split granularity.** Split the cell range on whole 16-cell groups, so that
no two workers share a 64-byte line. This is the same rule that the bitplane
design already applies.[^1]

**Vectorisation.** Every step above is a flat loop over a contiguous `i32` or
`i64` array with no branches. The clamp compiles to `SMIN` and `SMAX`. The
permission gate compiles to a mask and an `AND`. LLVM auto-vectorises all of
it on NEON with no explicit intrinsics, which matches the one-code-path
decision.[^1] Pin these loops in the assembly test, because auto-vectorisation
fails silently after a refactor.[^1]

**Phase placement in the frame loop.** The frame loop splits phases 1 to 4,
which read the world and write only events, from phases 5 to 8, which write
the world and read only events.[^1] The trade stage divides across that split:

- Phase 4, EXECUTE. Compute the potential, the permission factor and the
  flux. Emit one `TradeFlux` event for each arc with a non-zero flux. All
  reads are of the sealed world. All writes are events.
- Phase 5, APPLY. Walk the flux events. Apply the stock deltas. Mark the
  cells dirty.
- Phase 7, PYRAMID. The existing dirty walk carries the new stocks upward.

The `TradeFlux` event must be `bytemuck::Pod`: `repr(C)`, with declared
padding and no `bool`.[^1] A compact form is `{ arc: u32, commodity: u16,
_pad: u16, flux: i32 }`, which is 12 bytes with no implicit padding.

**A cost warning about the event form.** 196,608 arcs times 16 commodities
gives 3.1 million possible flux events for each tick, at 12 bytes, so 37.7 MB
of event traffic. That is larger than the design's 500,000 event budget.[^17]
Two answers. First, emit an event only where the flux is non-zero, which is
typically well under 10 percent of arcs. Second, and better, treat the whole
level 1 flux array as **one** bulk event that carries an array reference,
rather than as one event for each arc. The apply step then runs as a single
kernel. Record this as an explicit exception to the fine-grained event rule,
and record the reason.

## 13. What shipped games do

### 13.1 A transport game with a link graph

One long-running open transport simulation uses a link graph for cargo
distribution. It keeps one graph for each cargo type. Nodes are stations and
industries. Edges carry a supply value, a demand value, a capacity and a flow.
A demand calculator fills the demands, with three selectable models: no
distribution at all, a symmetric model that sends equal cargo in both
directions between a station pair, and an asymmetric model with no such
constraint. The distance decay is a tunable that controls how much more cargo
goes to a near station than to a far station. A multi-commodity flow solver
then computes the flows, and a mapper turns those flows into per-station
statistics. The solve runs on a background thread with a deadline measured in
game days, and the main thread joins it when the deadline arrives. The game
exposes the recalculation interval, the deadline and a solver accuracy setting
as user options.[^22]

**What to take.** The one-graph-per-commodity decoupling. The background job
with a deadline. The accuracy setting as an explicit quality dial.

**What to leave.** The station-level graph. This engine has a regular grid, so
it does not need an arbitrary graph, and a grid stencil is much faster than a
general graph solve.

### 13.2 A factory game with local transfer only

One widely studied factory game has no global resource solve at all.

Its belt system stores the **gaps between items** on a transport line, not the
absolute positions. Because unblocked items keep their relative spacing,
moving a whole line of many belt segments only changes the two terminal gaps.
The items themselves are untouched. The developers report an amortised
constant cost for each item on each traversed line, a 50 to 100 times gain on
item movement, and a 5 to 10 times overall gain.[^23]

Its fluid system was a local pressure equalisation between neighbouring pipe
entities. The developers document the resulting defects plainly: throughput
decayed with distance at an inconsistent rate, results depended on **build
order**, and junction splits were unpredictable. The 2.0 rewrite merges pipes,
underground pipes and storage tanks into **fluid segments**. A segment holds
one averaged fluid level, and fluid pushed into a segment is available
anywhere along it immediately. Machines extract at a rate proportional to how
full the segment is. Junction order no longer matters. The developers describe
it as the same kind of solution as their electric network.[^24]

**What to take.** This is the single most valuable lesson in the survey, and
it is a lesson learned the expensive way by a shipped title. **A local
equalisation over individual cells produces order dependence and unpredictable
behaviour. Equalisation over a larger aggregated unit does not.** The level 1
cell in this engine is exactly such an aggregated unit: it summarises 256
tiles into one stock value. A design that instead equalised between individual
level 0 tiles would reproduce the defect that the rewrite removed.

**What to leave.** The absence of any global plan. That game is a factory on
one screen. This engine is a world of 16.7 million tiles, where goods must
cross the map. The level 2 plan supplies what the local rule cannot.

### 13.3 City builders with discrete carrier agents

One city building series simulates cart pushers as part of the simulation, not
as decoration. A cart carries goods directly from a producer to a consumer, or
a carriage moves a surplus to a warehouse. The game decides whether to send a
partly loaded cart now or to wait for a fuller load.[^25]

No performance figures are published for this series, and none were found for
the other well known carrier-driven builder series. This report therefore
makes no numeric claim about them.

**What to take.** Nothing at the algorithm level. The carrier is a
presentation of the flow, and this engine should treat it that way. See
section 13.5.

### 13.4 Grand strategy: pooled resources and a market model

One grand strategy family pools resources at the nation level and applies a
long chain of percentage modifiers. There is no spatial flow at all. That is
the cheapest possible model, and it is a valid answer when the design does not
care where goods are.

A more recent title in the same family adds a genuine market. Every unit of
state-level production adds a sell order. Every unit of state-level
consumption adds a buy order. The orders pool into one market. The price
follows the normalised difference between the buy orders and the sell orders,
clamped to a band from 25 percent to 175 percent of a base price. Every good
has a base price equal to its value under ideal conditions. States connect to
the market capital with a variable market access. A blend factor mixes the
national market price with the local state price. The base blend is 75 percent
national. An isolated state has zero access and uses only its local price. Low
access also scales down the orders that the state contributes.[^21]

**What to take.** Three things, and this report adopts all three. First, the
clamped price band, which makes divergence impossible without a convergence
proof. Second, the base price as the anchor, which makes prices comparable
across commodities. Third, the market access blend, which is the same idea as
this report's plan bias: a place that is poorly connected trades at its own
local price, and a place that is well connected trades at the shared price.

**What to leave.** The absence of a physical route. That title abstracts
transport into a scalar access value. This engine has a real map, so it can
carry the physical flow and get the access value for free from the flux.

### 13.5 A colony simulator and the hauling problem

One deeply simulated colony game matches hauling jobs to workers. That
matching is O(jobs x workers) in the worst case. The game does not run it
every tick. Job assignment runs on roughly a hundred-tick cadence, and worker
job applications run on a separate staggered cadence of the same length. A
developer describes the result as an invisible auction over competing
priorities.

The published cost profile is instructive and is the opposite of what most
readers expect. Over 60 percent of processing time in a large fort goes to
units taking their turns, and **under 10 percent of that is pathfinding**. So
pathfinding is about 6 percent of the frame. The cost driver is the item
**stack count**, not the stack size: the game's own performance guidance
states that the quantity in a stack matters far less than the number of
stacks, because stacks drive hauling jobs, stockpile scans and paths. Totals
above about 10,000 items become seriously noticeable. The simulation is
effectively single-threaded.[^26][^27]

**What to take.** The hard lesson. The failure mode is not the flow
computation. It is the **number of discrete items that need an individual
decision**. This engine targets one million units.[^1] A per-item hauling
decision at that scale is two orders of magnitude beyond the budget, exactly
as a per-unit path search is.[^17] The staggered cadence is the correct
mitigation and this report adopts it for the level 2 plan.

### 13.6 A published economic strategy postmortem

One shipped real-time strategy game replaces combat with a live commodity
market as its core loop. Its postmortem covers the design path and the removal
of unit micromanagement in favour of macroeconomic play.[^28] No algorithm
level detail is published, so this report cites it as design rationale only,
and makes no performance claim from it.

## 14. What this report rejects, and why

| Rejected | Reason |
|---|---|
| Minimum cost flow at level 0 | 3.2 GB of solver state, which exceeds the whole world state. Minutes for each solve. |
| Diffusion at level 0 | 3.3 ms of memory bandwidth for each sweep for each commodity. 210 ms at 8 commodities and 8 sweeps. |
| Minimum cost flow at level 1 as the per-tick method | 0.3 to 2 s for each solve. It needs 3 to 20 ticks. Keep it as an optional background quality mode only. |
| A coupled multi-commodity solver | Integer multi-commodity flow is NP-hard for two commodities. |
| Converged tatonnement each tick | Convergence needs the gross substitutes condition. The iteration count is data dependent, which breaks the frame budget. A clamped single step gives the same visible result. |
| The auction algorithm for spatial flow | Its strong guarantee applies to the assignment problem. Its epsilon scaling gives a data-dependent iteration count. Keep it for caravan assignment only. |
| Cost scaling push-relabel at level 2 | The experimental evidence favours network simplex on small sparse graphs. |
| One solve for each faction | Cost would scale with the faction count. Trade blocs remove that scaling entirely. |
| Per-carrier simulation as the authority | The colony simulator evidence shows the cost driver is the discrete item count, not the flow computation. |
| A value-average diffusion rule | It does not conserve under integer rounding. The flux form does. |
| Floating-point prices or flows | Banned by an existing invariant, and it would make conservation need a correction pass. |

## 15. Open questions

**OQ-A. Is the faction mask 64 bits wide or wider?** This report assumes at
most 64 mask-addressable factions, which matches the existing relation plane
design.[^2] Above 64 factions the relation plane becomes an array of words, the
diplomacy factor table grows as the square of the faction count, and the bloc
union-find stops being free. At 256 factions the factor table alone reaches
256 KB, which is still acceptable. Confirm the ceiling.

**OQ-B. How many commodities does the design actually need?** This report caps
at 16 and hard-limits at 32. If the answer is 4, the whole subsystem costs
12.3 MB and under 0.2 wall-ms for each tick, and several of the tuning
concerns above disappear.

**OQ-C. Does trade follow roads, or does it diffuse across open terrain?** If
it follows roads only, the arc capacity is zero on most arcs and the flux map
becomes sparse. A sparse form would cut both the storage and the cost by a
large factor, but it adds an index. Decide after the road model exists.

**OQ-D. What is the settlement count?** Section 11.2 shows that the storage is
small at every plausible count. But the settlement count sets the cost of the
distribution scan from level 1 stock down to individual settlements. Confirm
the order of magnitude.

**OQ-E. Should the flux be one bulk event or one event for each arc?** Section
12 recommends a bulk event, and notes that this is an exception to the
fine-grained event rule. The exception needs the decision record author's
agreement, because it affects the event log format and therefore the replay
test.

**OQ-F. Does the diffusion shift value need to be per-commodity?** A heavy
good and a light good could reasonably move at different rates. One shift
value for each commodity costs 4 bytes and no time. Recommend allowing it.

---

## Ready-to-apply ADR decision block

The following text is ready to insert into the decision record. It uses the
existing decision numbering style. The decision record currently holds 50
numbered decisions, so these continue at D51. Adjust the numbers if other
reports also add decisions.

---

### Part H — Resource and trade flow

#### D51. Solve the trade plan at L2. Move the goods at L1. Never solve at L0

The trade stage has two parts at two levels.

**The plan.** An exact integer minimum cost flow over the 256 L2 cells, one
solve for each commodity. It produces a per-cell `plan_bias` value.

**The movement.** A fixed number of conserving diffusion sweeps over the
65,536 L1 cells, one set for each commodity, every tick.

L0 runs no flow solve of any kind. A building draws from the L1 cell that
contains it.

| Option | Solver state | Cost for each solve | Verdict |
|---|---|---|---|
| Minimum cost flow at L0 | 3.2 GB | minutes | Reject |
| Minimum cost flow at L1 | 12.6 MB | 0.3 to 2 s | Background quality mode only |
| Minimum cost flow at L2 | 48 KB | 20 to 80 us | **Accept for the plan** |
| Diffusion at L0 | — | 3.3 ms for each sweep, bandwidth bound | Reject |
| Diffusion at L1 | 2.07 MB for each commodity | 0.05 ms for each sweep | **Accept for the movement** |

The L0 rejection is bandwidth arithmetic, not a benchmark. One pass over the
hot L0 tile data reads 134 MB, which costs 3.3 ms of memory bandwidth. That
is 10 percent of a 33 ms tick for one sweep of one commodity.

Use network simplex at L2, not cost scaling. The graph is small and sparse,
and the published experimental comparison favours network simplex on sparse
graphs. Fix the pivot rule and the tie-breaking order, so the result is a pure
function of the input.

**The L2 plan does not move goods. It bends the potential field that moves
goods.** This is the same two-level pattern that D-series decisions already
apply to movement with the portal graph and flow tiles.

Source: report 11, sections 3, 4, 5 and 6.

#### D52. Diffuse in flux form. Conservation is exact and it is a tested invariant

Compute a flux for each arc once. Subtract it at one end. Add it at the other
end. Never compute a cell value as the average of its neighbours; that form
does not conserve under integer rounding.

```
raw   = (potential[a] - potential[b]) >> DIFFUSION_SHIFT
flux  = clamp(raw, -capacity[arc], +capacity[arc])
flux  = clamp(flux, -stock[b], +stock[a])
```

Each arc appears exactly twice in the apply step, once with each sign. The
total delta over all cells is therefore zero by construction, in exact integer
arithmetic.

Integer arithmetic is what makes this free. Integer addition is associative
and commutative, so a parallel reduce of the world stock gives the same total
for any thread count. The single shared flux value removes the second
rounding, so the two ends cannot disagree. The clamp applies before either end
uses the value, so clamping preserves conservation. In floating point neither
property holds, and every transfer would leak.

Where a division has a remainder — refining an L2 target across its 256 L1
cells, or splitting a cell's outflow across its arcs — use the **largest
remainder method** in canonical order, exactly as the `transfer` verb already
specifies. The parts then sum to the whole exactly.

**Add one test to the determinism suite.** Sum the whole stock plane before
the trade stage and after it. Add the production and subtract the consumption.
Assert exact equality. Run it at 1, 2 and 12 threads.

The engine may be approximate about which route goods take and about how fast
they arrive. It must be exact about how many goods exist.

Source: report 11, sections 6 and 7.

#### D53. Run the L1 sweep every tick. Rebuild the L2 plan on an event, with a 16-tick staleness bound

| Stage | Cadence | Trigger | Staleness | Core-ms |
|---|---|---|---|---|
| L1 potential and sweep | every tick | none | none | 0.5 to 3.5 |
| L2 plan solve | on an event | L2 dirty bit, diplomacy change, route change | 16 ticks maximum | 0.2 to 0.6 |
| L2 to L1 bias refinement | with the plan | as above | as above | 0.1 to 0.3 |
| Price step | every tick | none | none | under 0.1 |

Movement must run every tick. Diffusion moves goods about one cell for each
sweep, so a skipped tick stops the economy.

The plan must not run every tick. It changes only when a road, a border, a
diplomatic relation or a large production value changes. The existing dirty
bitset already reports exactly those events.

Force a rebuild at least every 16 ticks even with no dirty bit. That bounds
the staleness at 1.6 s of simulated time at the 10 Hz tick rate. Spread the
work by rebuilding the L2 cells whose index satisfies `index mod 16 ==
tick mod 16`. The per-tick cost is then flat instead of spiky.

A shipped transport game runs the same shape: a background solve with a
deadline in game days, joined by the main thread at the deadline, with the
interval, the deadline and a solver accuracy setting exposed as options.

Source: report 11, sections 8 and 13.1.

#### D54. Solve K independent single-commodity problems. Cap K at 16, hard-limit 32

Never build a coupled multi-commodity solver. Integer multi-commodity flow is
NP-hard for two commodities. The float relaxation is unavailable, because D4
bans floats from simulated state, and rounding a fractional solution back to
integers restores the hard problem.

Resolve the shared arc capacity **before** the solves. Split each arc capacity
between the commodities in proportion to their demand across that arc in the
previous tick, and round with the largest remainder method so the parts sum to
the capacity exactly. Give every commodity a floor share, so a rising demand
cannot starve for more than one tick.

Every stage then scales linearly in K, with no hidden term.

| K | L1 plane bytes | Sweep traffic | Core-ms | Wall-ms at 12 cores |
|---|---|---|---|---|
| 4 | 2.0 MB | 5.2 MB | 1.6 | 0.15 |
| 8 | 4.2 MB | 10.5 MB | 3.2 | 0.3 |
| **16** | **8.4 MB** | **21 MB** | **6.4** | **0.6** |
| 32 | 16.8 MB | 42 MB | 12.8 | 1.2 |
| 64 | 33.6 MB | 84 MB | 25.6 | 2.4 |

The ceiling is a cache argument. At K = 16 the L1 planes hold 8.4 MB, which
stays resident in the shared last level cache of a large Graviton part. At
K = 64 it does not, and the sweep becomes bandwidth bound like an L0 pass.

Enforce the limit of 32 with a `const_assert`, in the same style as the other
declaration limits.

Source: report 11, section 9.

#### D55. Partition by trade bloc. Trade cost must not scale with the faction count

Reject any design that runs one solve for each faction. At 16 commodities, 8
sweeps and 16 factions, that costs about 100 core-ms for each tick, which
exceeds the whole per-tick budget.

Every L1 cell has exactly one owner faction, so the cells partition by owner.

1. Derive a per-arc capacity factor and cost factor from the diplomacy
   relation planes, through a 64 x 64 table of `i32` Q16.16 pairs. The table
   is 16 KB. War gives a capacity factor of zero.
2. Compute **trade blocs** by a union-find over the "capacity factor is
   non-zero" relation. At 64 factions this is 64 rows of one `u64` word, so it
   costs under a microsecond. Label each bloc by the lowest faction index in
   it, so the labelling is a pure function of the relation plane and does not
   depend on the union order.
3. Assign a bloc identifier to each cell by one map over the cell owner.
4. Run **one** set of commodity planes over the whole map. Zero the flux on
   any arc whose endpoints are in different blocs, branchlessly.

Total work then scales with the cell count and the commodity count, and **not
with the faction count**.

The relation plane is already specified for fog of war at three `u64` rows for
each faction, so this decision adds a fourth concern to an existing structure
rather than a new structure. It assumes at most 64 mask-addressable factions,
which ties this decision to OQ5.

Source: report 11, section 10.

#### D56. The trade stage uses no scatter and no atomic operation

Express the whole stage in the existing kernel vocabulary.

| Step | Kernel | Parallel over |
|---|---|---|
| Aggregate production to L1, then to L2 | reduce | cells |
| Solve the L2 plan | serial | one thread |
| Refine the plan to the L1 bias | scan, then map | L2 cells |
| Compute the arc permission factor | map | arcs |
| Compute the potential | stencil | L1 cells |
| Compute the flux | map | arcs |
| Apply the flux | gather | L1 cells |
| Distribute to settlements | scan, then map | L1 cells |
| Verify conservation | reduce | L1 cells |

The flux map writes one slot for each arc. The apply gather writes one stock
value for each cell. No two workers write the same address in either step. D-
series decisions raise disjoint outputs from preferred to required on the ARM
target; this design meets that requirement by construction.

Split the cell range on whole 16-cell groups, so no two workers share a
64-byte line. This is the same rule the bitplane storage already uses.

Every loop is flat, contiguous and branchless. The clamp compiles to `SMIN`
and `SMAX`. The bloc gate compiles to a mask and an `AND`. LLVM
auto-vectorises all of it on NEON. Pin these loops in the assembly test.

Place the stage across the read and write phase split as follows.

- Phase 4, EXECUTE. Compute the potential, the permission factor and the
  flux. Read the sealed world. Write only events.
- Phase 5, APPLY. Apply the stock deltas. Mark the cells dirty.
- Phase 7, PYRAMID. The existing dirty walk carries the new stocks upward.

**Emit the L1 flux as one bulk event that carries an array, not as one event
for each arc.** One event for each arc would be up to 3.1 million events at 12
bytes, so 37.7 MB for each tick, against a 500,000 event budget. This is a
deliberate exception to the fine-grained event rule. Record the exception and
the reason, because it affects the replay test.

Source: report 11, sections 6 and 12.

#### D57. Carrier agents are a presentation of the flow. They are not the flow

A visible cart, caravan or porter reads the L1 flux field and follows it. It
holds no authoritative stock. Removing every carrier from the world must not
change the economy by one unit.

The evidence is direct. A deeply simulated colony game spends over 60 percent
of its frame on units taking turns, and its own performance guidance names the
**item stack count**, not the stack size, as the cost driver. It runs its
hauling match on a hundred-tick cadence, not every tick, because the match is
O(jobs x workers). This engine targets one million units, which is two orders
of magnitude past a per-item decision.

A factory game learned the matching lesson for flow. Its original per-pipe
local equalisation gave throughput that decayed inconsistently with distance
and results that depended on **build order**. Its rewrite merged pipes into
larger **segments**, each with a single averaged level. The L1 cell in this
engine is exactly such a segment: it summarises 256 tiles into one stock. A
design that equalised between individual L0 tiles would reproduce the defect
that the rewrite removed.

If a caravan must be a real entity carrying real goods, assign caravans to
routes with an auction algorithm at a few thousand bidders. Do not use an
auction for spatial flow, because its epsilon scaling gives a data-dependent
iteration count that breaks the frame budget.

Source: report 11, sections 3.4, 13.2 and 13.5.

#### D58. Prices are one clamped tatonnement step for each tick, never a converged solve

Prices are optional. If the design has them:

```
excess      = demand - supply                        // i64
adjustment  = (excess * GAIN) / max(demand, supply)  // Q16.16
price      += clamp(adjustment, -MAX_STEP, +MAX_STEP)
price       = clamp(price, base_price / 4, base_price * 7 / 4)
```

All arithmetic is `i64` in Q16.16. There is no floating point, per D4.

Do not run tatonnement to convergence. Convergence is only guaranteed under
the gross substitutes condition, and the iteration count is data dependent,
which breaks a fixed frame budget. The clamped single step needs no
convergence proof, because the band makes divergence impossible.

The band of 25 percent to 175 percent of a base price is the model that a
shipped grand strategy title uses, together with a price response driven by
the normalised difference between the buy orders and the sell orders. Adopt
its market access idea as well: a poorly connected place trades near its local
price, and a well connected place trades near the shared price. In this engine
the connection strength is already available as the arc capacity, so it costs
nothing to compute.

A converged price vector each tick would also be worse simulation. It makes
the economy react instantly to a distant event. A single step gives a lag that
matches the physical transport lag.

Source: report 11, sections 3.5, 6.4 and 13.4.

#### D59. Trade storage budget

Store one shared arc geometry and one set of planes for each commodity.

| Item | Bytes |
|---|---|
| L1 shared arc geometry, 196,608 arcs | 4.0 MB |
| L1 planes, for each commodity | 2.07 MB |
| L2 network, all commodities | under 64 KB |
| Diplomacy factor table, 64 x 64 | 16 KB |

| K | Total L1 trade storage |
|---|---|
| 4 | 12.3 MB |
| 8 | 20.6 MB |
| **16** | **37.1 MB** |
| 32 | 70.2 MB |

Per-settlement stock costs 20 bytes for each commodity: an `i64` stock, and
three `i32` Q16.16 values for the production rate, the consumption rate and
the local price. Store it as struct-of-arrays indexed by
`settlement_index * K + commodity_index`.

| Settlements | K = 8 | K = 16 |
|---|---|---|
| 1,000 | 160 KB | 320 KB |
| 10,000 | 1.6 MB | 3.2 MB |
| 65,536 | 10.5 MB | 21.0 MB |

At the recommended ceiling of 16 commodities the whole subsystem costs 37.1 MB
plus at most 21.0 MB of settlement stock. Compare with 21.0 MB of fog of war
for each faction. Trade is a moderate cost, and unlike fog of war it does not
grow with the faction count.

Drop the arc endpoint array and save 1.57 MB, because the neighbour of a cell
follows from its index on a regular grid. Keep it only if the design adds
long-distance arcs such as sea routes.

Source: report 11, section 11.

#### D60. Per-tick trade budget

Add these lines to the per-tick cost budget table. The figures assume 16
commodities, 8 sweeps, and 12 cores.

| Work | Scale | Core-ms | Wall-ms |
|---|---|---|---|
| L1 potential stencil | 65,536 cells x 16 | 0.8 | 0.1 |
| L1 flux map | 196,608 arcs x 16 | 4.0 | 0.4 |
| L1 apply gather | 65,536 cells x 16 | 1.6 | 0.2 |
| L2 plan solve, amortised over 16 ticks | 256 cells x 16 | 0.3 | 0.3 |
| Bloc union-find and factor map | 64 factions, 196,608 arcs | 0.2 | under 0.1 |
| Price step | 65,536 x 16 | 0.1 | under 0.1 |
| Conservation check | 65,536 x 16 | 0.3 | under 0.1 |
| **Trade total** | | **7.3** | **1.1** |

At 4 commodities the total falls to about 2.0 core-ms. Trade is therefore a
small line item beside fog of war recompute and event apply, which the
existing budget names as the two largest costs.

Source: report 11, sections 9 and 12.

---

## References

[^1]: ADR-0001, Foundational Architecture. Decisions D1 to D50, the byte budget section and the per-tick cost budget section. `docs/adrs/draft/adr-0001-foundational-architecture.md`
[^2]: Research report 08, Fog of War Representation. Section 6.3 and the storage table. `docs/adrs/background/adr-0001/08-fog-of-war-representation.md`
[^3]: Even, S., Itai, A. and Shamir, A., 1976. On the complexity of timetable and multicommodity flow problems. SIAM Journal on Computing, 5(4), pp. 691-703.
[^4]: Ahuja, R. K., Magnanti, T. L. and Orlin, J. B., 1993. Network Flows: Theory, Algorithms, and Applications. Prentice Hall.
[^5]: Orlin, J. B., 1997. A polynomial time primal network simplex algorithm for minimum cost flows. Mathematical Programming, 78(2), pp. 109-129.
[^6]: Kovács, P., 2015. Minimum-cost flow algorithms: an experimental evaluation. Optimization Methods and Software, 30(1), pp. 94-127.
[^7]: Goldberg, A. V. and Tarjan, R. E., 1988. A new approach to the maximum-flow problem. Journal of the ACM, 35(4), pp. 921-940.
[^8]: Goldberg, A. V., 1997. An efficient implementation of a scaling minimum-cost flow algorithm. Journal of Algorithms, 22(1), pp. 1-29.
[^9]: Hitchcock, F. L., 1941. The distribution of a product from several sources to numerous localities. Journal of Mathematics and Physics, 20(1-4), pp. 224-230.
[^10]: Kuhn, H. W., 1955. The Hungarian method for the assignment problem. Naval Research Logistics Quarterly, 2(1-2), pp. 83-97.
[^11]: Bertsekas, D. P., 1988. The auction algorithm: a distributed relaxation method for the assignment problem. Annals of Operations Research, 14(1), pp. 105-123.
[^12]: Walras, L., 1874. Elements d'economie politique pure. L. Corbaz, Lausanne.
[^13]: Arrow, K. J. and Debreu, G., 1954. Existence of an equilibrium for a competitive economy. Econometrica, 22(3), pp. 265-290.
[^14]: Codenotti, B., McCune, B. and Varadarajan, K., 2005. Market equilibrium via the excess demand function. Proceedings of the 37th ACM Symposium on Theory of Computing (STOC), pp. 74-83.
[^15]: Cole, R. and Fleischer, L., 2008. Fast-converging tatonnement algorithms for one-time and ongoing market problems. Proceedings of the 40th ACM Symposium on Theory of Computing (STOC), pp. 315-324.
[^16]: Garg, N. and Konemann, J., 2007. Faster and simpler algorithms for multicommodity flow and other fractional packing problems. SIAM Journal on Computing, 37(2), pp. 630-652.
[^17]: Research report 06, Simulation Algorithms and Parallel Scheduling. Sections 2, 3, 8.4, 8.5 and 10. `docs/adrs/background/adr-0001/06-algorithms-and-scheduling.md`
[^18]: Saad, Y., 2003. Iterative Methods for Sparse Linear Systems. 2nd edition. SIAM. Chapter 4, on the convergence rate of stationary iterative methods.
[^19]: Research report 07, Target Platform and Value Types. The section on the weak memory model. `docs/adrs/background/adr-0001/07-target-platform-and-value-types.md`
[^20]: Research report 04, Selector Engine and Verbs. Section 8.3, the `transfer` verb. `docs/adrs/background/adr-0001/04-selector-engine-and-verbs.md`
[^21]: Paradox Interactive, Victoria 3 developer diary 9, National Markets, and developer diary 37, Market Expansion. https://forum.paradoxplaza.com/forum/threads/victoria-3-dev-diary-9-national-markets.1484917/ and https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-37-market-expansion
[^22]: OpenTTD project, Link graph documentation and the cargo distribution manual. https://github.com/OpenTTD/OpenTTD/blob/master/docs/linkgraph.md and https://wiki.openttd.org/en/Manual/Passenger%20and%20cargo%20distribution
[^23]: Wube Software, Factorio Friday Facts 176, Belts optimization for 0.15. https://www.factorio.com/blog/post/fff-176
[^24]: Wube Software, Factorio Friday Facts 416, Fluids 2.0. https://www.factorio.com/blog/post/fff-416
[^25]: Ubisoft Blue Byte, Anno Union developer blog, Pushing carts, December 2017. https://www.anno-union.com/devblog-pushing-carts/
[^26]: Dwarf Fortress Wiki, Maximizing framerate. https://dwarffortresswiki.org/index.php/Maximizing_framerate
[^27]: Adams, T., interviewed by Game Developer, Q&A: Dissecting the development of Dwarf Fortress with creator Tarn Adams. https://www.gamedeveloper.com/design/q-a-dissecting-the-development-of-i-dwarf-fortress-i-with-creator-tarn-adams
[^28]: Johnson, S., 2017. Offworld Trading Company: An RTS Without Guns. Game Developers Conference. https://www.gdcvault.com/play/1024297/-Offworld-Trading-Company-An
