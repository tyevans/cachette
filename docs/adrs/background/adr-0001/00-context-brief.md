# Project Context Brief — Cachette

Shared context for all research agents working on ADR-0001.

## What we are building

A Rust-backed, Python-exposed high-performance 2D world simulation engine.
Target: millions of active simulated objects. Event-sourced, DDD-influenced
architecture on top of cache-aligned, tightly-optimized memory structures.

## The simulation domain

A multi-scale world simulation with three levels of detail:

- **L0** — a hex grid of individual tiles. Each tile has terrain type
  (forested, plains, ...), attributes, elevation, ownership, and optional
  upgrades. Individual units occupy tiles.
- **L1** — zoom out. Blocks of L0 tiles summarize into "city-scale" cells.
- **L2** — zoom out again. Blocks of L1 cells summarize into region-scale cells.

Target grid extent is on the order of 4096x4096 = ~16.7M tiles, plus
units in the hundreds of thousands to millions.

## Architectural decisions made so far (in discussion)

1. **CQRS framing.** L0 is the write model and single source of truth.
   L1 and L2 are read models / projections, derived and incrementally
   maintained, and disposable.

2. **Two storage regimes.**
   - Tiles: dense struct-of-arrays indexed by grid position. NOT in the ECS.
     Narrow types (u8/u16), bitplanes for booleans, sparse side tables for
     rare/optional per-tile data (upgrades, names).
   - Units and structures: archetype-chunked ECS (DOTS/Flecs style, not
     sparse-set). 16KB chunks, 64B aligned, pure SoA within a chunk.
     Entity ID = 32-bit index + 32-bit generation.
   - Bridge: per-tile occupancy index, CSR-style (offsets + packed unit array).

3. **Hex hierarchy.** Leaning toward axial (q,r) coordinates with
   power-of-two parallelogram chunks (e.g. 32x32) for exact nesting and
   shift/mask parent lookup. Alternative considered: H3-style aperture-7
   true-hex hierarchy (non-exact nesting, rotation per level). Display can
   render hexes at L1/L2 even if storage is parallelogram blocks.

4. **Aggregation must be a monoid.** An attribute may only appear at L1/L2
   if it is expressible as an associative combine with identity over L0.
   Safe: sum, count, min, max, bitwise-or, histogram. Mean = sum+count.
   "Dominant terrain" = histogram + argmax at read.

5. **Incremental update via a dirty pyramid.** One dirty bitset per level.
   Frame barrier walks set bits, recomputes only dirty cells, marks parents
   dirty, repeats upward. Parallel via rayon over disjoint blocks.

6. **The pyramid doubles as a query acceleration structure.** L1/L2 summaries
   carry the fields selectors filter on (faction bitmask, unit-type histogram,
   terrain histogram), so selector resolution is a hierarchical descent that
   prunes whole blocks.

7. **Python is a control plane, not a data plane.** Python must NEVER loop
   over entities. Interaction is via **set-valued commands**:
   `Command = (Selector, Verb, Params)`.
   - Selector is a lazy, composable expression tree built in Python,
     evaluated in Rust. Terminal ops (`.count()`, `.to_numpy()`, or feeding
     a verb) force evaluation.
   - Verbs are registered Rust implementations. ~30 expected.
   - Params are POD, or another selector.
   - Bulk data access is zero-copy NumPy views onto component arrays.
   - Events are delivered to Python batched, once per frame, as arrays.
     No Python callback fires mid-step.
   - GIL is released for the whole simulation step.

8. **Types and upgrades are data, not components.** `UnitType(u16)` indexes
   a shared immutable stat table (move_speed, attack, capabilities bitmask,
   terrain_cost matrix). Upgrades are a `u64` bitmask plus a sparse modifier
   table. This keeps all units in one archetype and prevents verb explosion.
   Types parameterize verbs; they do not multiply them.

9. **Command handling.** Commands queue during the Python phase and seal at
   the frame barrier. Applied in deterministic order (issue order / priority
   then issue order), never completion or thread order. Aggregate boundary
   = parallelism boundary; disjoint regions run concurrently.

10. **Partial failure returns summaries, not exceptions.** A command returns
    affected count, rejection reasons with counts, and a lazy selector over
    the rejected set so Python can chain without seeing IDs.

11. **Set-valued commands enable cheaper algorithms.** E.g. `move_to` for
    5000 units computes one hierarchical flow field, not 5000 A* searches.

12. **Event log starts transient** (per-frame command buffer, thread-local
    then deterministically concatenated) but events are POD and serializable
    and the apply step is pure, so retained/rollback is additive later.

## Users

Three audiences, in this priority order:
1. The author, building an actual strategy game on it (dogfooding).
2. Other developers building simulations.
3. Researchers running RL / agent-based models at scale (NumPy-native,
   deterministic, `step()`-shaped — this is the clearest differentiator).

## Known risks

- Two-language debugging across the FFI boundary.
- Packaging (abi3 wheels, maturin, cibuildwheel, three platforms).
- Users writing per-entity Python loops (API must make this hard).
- GIL contention if callbacks fire mid-step.

## Deferred deliberately

- Retained event log / rollback / time travel.
- Coarse background simulation of unobserved regions (freeze at first).
- Modding DSL or bytecode VM.
- Netcode.

## Open questions

- Exact grid extent and byte budget per tile.
- Hex nesting: exact-parallelogram vs true-hex.
- Determinism target: bit-exact cross-platform, or within-run only?
- Is background simulated or frozen?
