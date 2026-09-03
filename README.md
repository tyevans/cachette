# Cachette

**A large-scale, bit-for-bit deterministic world simulation engine. Rust core, Python control plane.**

Cachette simulates large hex-grid worlds of up to 16.7 million tiles and one million individual units.[^1]
A unit is an autonomous person, not a number in a formation.
A unit occupies a tile, gathers resources, experiences hunger, and works a trade.[^2]
The simulation core runs in Rust for predictable hardware performance.[^3]
You configure, drive, and inspect the world from Python.[^4]

## Key Capabilities

- **Massive scale**: The engine simulates up to 16.7 million tiles and one million active entities on a single server.[^1]
- **Exact determinism**: One binary yields the identical state hash and byte-for-byte event log across any thread count.[^5]
- **Zero floating-point drift**: All simulation calculations use integer and Q16.16 fixed-point arithmetic.[^6]
- **Emergent behaviour**: Famines, migration, and trade emerge naturally from individual needs and local production.[^7]
- **Spatial pyramid**: Three levels of detail provide instant answers for continental aggregate queries.[^8]
- **Native AI agent support**: An integrated Model Context Protocol server connects external agents directly to the engine.[^9]

## Who It Is For

### Game Developers

Build deep strategy games, colony simulations, or living world games.
You author the rules for needs, resource gathering, and production rates.
The engine simulates the world and emits structured events.
You build the user interface and the rendering pipeline in the client of your choice.
Set-valued commands let you order thousands of units with single flow-field operations.

### Researchers and Modellers

Run computational experiments that demand strict reproducibility.
Simulate macroeconomic systems with one million distinct households instead of aggregate approximations.
The counter-based random number generator isolates causal variables between runs.[^10]
Change one parameter, keep the seed, and know that all other conditions remain identical.
Export event and gathering logs directly into NumPy arrays for statistical analysis.[^11]

### Artificial Intelligence Researchers

Evaluate multi-agent coordination, governance, and planning at planetary scale.
The built-in Model Context Protocol server lets language agents inspect simulation state and issue commands.[^9]
Because the simulation is deterministic, you can benchmark agent strategies against reproducible scenarios.

## The Determinism Guarantee

Run a simulation tick on one thread.
Run the same tick on twelve threads.
The state hashes match, and the event logs match byte for byte.[^5]

Cachette enforces determinism through structural invariants:

1. **No floating-point numbers**: Floating-point addition is not associative.
   Cachette bans floating-point numbers from simulated and aggregated state.[^6]
   All simulation math uses integer and Q16.16 fixed-point representation.

2. **Counter-based pseudorandom generation**: No system uses thread-local random state.
   Every random draw derives from a counter keyed by system, frame, entity, and draw index.[^10]

3. **Deterministic parallel joins**: Parallel execution stages write to disjoint memory slots.[^12]
   Reductions sort by stable keys rather than thread completion order.[^13]

4. **Declared byte padding**: Every event structure uses plain data representation with explicit padding.[^14]
   No uninitialised memory enters the state hash.

The determinism guarantee holds for any thread count on a given binary.[^5]
Two continuous integration tests verify this property on every build.
One test compares event logs across 1, 2, and 12 threads.
The other test compares world state hashes against stored golden files.

## Architectural Foundation

### The Spatial Pyramid

The engine organises the world into three levels of detail:

- **Level 0**: Individual tiles and individual units.[^8] This level is the only source of truth.
- **Level 1**: City-scale summaries combining blocks of level 0 tiles.
- **Level 2**: Region-scale summaries combining blocks of level 1 cells.

Upper pyramid levels are derived projections, not approximations.[^8]
Totals are exact sums of their component tiles.[^15]
Extremes are exact minimums and maximums.
Averages store exact numerators and denominators to prevent rounding errors.

Continental queries evaluate upper levels first.
A query dismisses whole regions when their aggregate bounds do not match the filter.
The engine descends to individual tiles only where boundaries require inspection.

### Python Control Plane and Rust Data Plane

Cachette keeps a strict boundary between Python and Rust.
The core simulation crate has no dependency on the Python runtime.[^16]
Python authors never write per-entity loops for simulation tasks.

Instead, Python constructs set-valued selectors and commands.[^17]
The Rust engine receives the command and executes an efficient whole-set algorithm.
For example, moving forty thousand units toward a target uses a single flow field rather than forty thousand path searches.
The engine releases the Python global interpreter lock for the entire duration of the simulation step.[^18]
Simulation events return to Python in batched, contiguous NumPy arrays at the frame barrier.[^11]

## Python Example

Use the Python package to configure and advance the world:

```python
import cachette

# Create a new world with four factions
world = cachette.World(width=64, height=64, seed=1, faction_count=4)

# Spawn soldiers for faction 1 at axial coordinates
addresses = [(0, 0), (1, 0), (0, 1), (1, 1)]
units = world.spawn_soldiers(addresses, faction=1)

# Issue a set-valued order to gather resources
world.order_gather(units, kind=0)

# Advance the simulation across multiple worker threads
# The Python global interpreter lock is released during execution
world.step(threads=4)

# Validate structural invariants
assert world.check_invariants()

# Inspect deterministic state hash and extract columnar NumPy arrays
print(f"Tick {world.tick}: state_hash=0x{world.state_hash():016x}")
events = world.event_log_columns()
gather_log = world.gather_log_columns()
print(f"Recorded {world.gather_count} gather events")
```

To run the Model Context Protocol agent server:

```bash
uv run python -m cachette.agent
```

## Watching and Inspecting the World

Cachette includes tools to inspect the running simulation visually.[^19]
You can open an interactive window or export headless diagnostic images.

```bash
# Open the interactive visualizer window (requires a graphical display)
just watch

# Render the world map to an image file without a display
just map seed=1 extent=128 out=target/world.ppm soldiers=600

# Generate a diagnostic panel image of engine metrics
just inspect out=target/panel.ppm
```

## What Cachette Is Not

- **Not a renderer**: Cachette does not manage graphics, shaders, or 3D scene graphs.
  It computes simulation state and returns columnar buffers for your client to draw.
- **Not a distributed engine**: Cachette targets a single large server with many CPU cores.[^3]
  It avoids network serialization overhead to achieve high agent density.
- **Not a general physics engine**: Cachette simulates discrete hex grids and discrete entity interactions.
  It trades continuous physics for scale, determinism, and performance.

## Project Status and Verification

The core determinism engine and the Python bindings are implemented and verified.
Continuous integration gates test unit properties, slot reductions, and multi-threaded equivalence.

Hardware performance targets AWS Graviton server instances with 64-byte cache lines.[^3]
Development takes place on x86-64 and Apple Silicon architectures.
Local execution verifies logic and determinism.
Benchmark measurements on the target server hardware will be published as testing progresses.[^20]

## Getting Started

### Prerequisites

- Rust toolchain (2024 edition)
- Python 3.12 or newer
- `uv` package manager
- `just` command runner

### Setup and Verification

Clone the repository and prepare the local environment:

```bash
just setup        # sync Python virtual environment and build extension
just test         # run the Rust, Python, and determinism tests
just check        # execute all verification gates and integrity checks
```

Read the contribution guide for repository layout and test requirements.[^21]
Review the decision records for architectural rationale and design rules.[^22]

## References

[^1]: ADR-0017, world geometry. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^2]: ADR-0012, tiles and units. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^3]: ADR-0008, the primary target. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^4]: ADR-0085, entity crossing to Python. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^5]: ADR-0001, determinism across thread counts. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^6]: ADR-0002, float ban in simulated state. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0063, unit needs and thresholds. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^8]: ADR-0022, level 0 truth and derived levels. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^9]: PRD-0019, agent MCP interface. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^10]: ADR-0003, keyed counter-based random draws. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^11]: DEC-060, columnar event logs. `docs/DECISIONS.md`
[^12]: ADR-0009, disjoint parallel outputs. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^13]: ADR-0007, key vector sort. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^14]: ADR-0006, plain data events. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^15]: ADR-0023, exact aggregate combination. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^16]: ADR-0041, crate split. `docs/adrs/REGISTRY.md`
[^17]: ADR-0051, lazy expression tree selectors. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^18]: ADR-0042, release Python interpreter. `docs/adrs/REGISTRY.md`
[^19]: PRD-0002, watching the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
[^20]: Budgets and costs register. `docs/reference/budgets.md`
[^21]: Contribution guide. `CONTRIBUTING.md`
[^22]: ADR Registry. `docs/adrs/REGISTRY.md`
