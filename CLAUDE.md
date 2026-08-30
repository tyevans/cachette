# Cachette

Cachette is a world simulation engine. The core is Rust. The control
plane is Python.

The engine simulates a hex world at three levels of detail. Level 0 holds
individual tiles and units. Level 1 summarises blocks of tiles at city
scale. Level 2 summarises blocks of level 1 cells at region scale. Level 0
is the only source of truth. Level 1 and level 2 are derived projections.

The target scale is 16.7 million tiles and one million units.

## Status

The project is in design. No code exists yet.

The foundational architecture decision record is a draft.[^1] It holds 50
numbered decisions. Read it before you write code. Six research reports
support it.[^2]

## Target platform

The engine targets AWS Graviton servers. The primary target triple is
`aarch64-unknown-linux-gnu`.

Development happens on x86-64 and on Apple Silicon. Both are development
targets only.

Apple Silicon uses a 128-byte cache line. Graviton uses a 64-byte cache
line. Local performance tests therefore mislead on false sharing and on
alignment. Benchmark on the target platform.

## Hard invariants

These rules are structural. You cannot add them to the project later.
Do not violate them for convenience.

1. **No floating point in simulated or aggregated state.** Float addition
   is not associative. An aggregate must combine exactly, in any order.
   Use integer or fixed-point arithmetic. The fixed-point scale is Q16.16.
2. **Route all simulation arithmetic through the `sim_math` module.** A
   lint enforces this boundary. The lint bans `f32::algebraic_add` and the
   other reassociating operations.
3. **Seed every random draw from a counter-based generator.** Key each
   draw on the tuple (system, frame, entity, draw). Do not use
   thread-local random state. Thread-local state destroys determinism.
4. **Keep the `cachette-core` crate free of any PyO3 dependency.** This
   makes a mid-step Python callback a compile error. It also allows Miri
   to check the unsafe code.
5. **Make every event type `bytemuck::Pod`.** Use `repr(C)`. Declare the
   padding. Do not use `bool`. Use `u8` instead. Undeclared padding
   creates false nondeterminism in state hashes.
6. **Release the Python global interpreter lock for the whole simulation
   step.** No Python code runs while the simulation runs.
7. **Deliver events to Python in batches at the frame barrier.** Never
   call Python from inside a system.
8. **Order every parallel result deterministically.** Sort by a stable
   key. Never use thread completion order. Never use work-stealing order.
9. **Widen pyramid accumulators at level 1.** A `u8` tile field summed
   over 16.7 million tiles overflows a `u32`. Use `i64`.

Two tests protect these rules. The first runs one tick at 1 thread, at
2 threads, and at 12 threads, then compares the event log byte for byte.
The second hashes the world state each frame against a golden file.

## Design principles

**Python is a control plane. Python is not a data plane.** Python must
never loop over entities. Python builds a selector, then sends one command.
Rust resolves the selector and runs the verb.

**A selector is a lazy expression tree.** Python builds the tree. Rust
evaluates it. The Python API raises an error for `__bool__`, `__len__`,
`__iter__`, and `__getitem__`. The error names the correct method.

**Unit types and upgrades are data. They are not code.** A unit type is
an index into a shared table. An upgrade set is an interned identifier.
Types parameterise the verbs. Types do not multiply the verbs.

**A set-valued command permits a cheaper algorithm.** Do not batch a
per-entity loop. Choose an algorithm that uses the whole set. One example
is a flow field instead of many path searches.

## Product

Work answers to a need. A need lives in `docs/product/` as a numbered product
requirement record. A registry allocates the number before the record is
written, in the same way the decision registry does.[^8]

A record moves through four directories: `idea/`, `shaped/`, `accepted/` and
`shipped/`. **A record is shaped when it answers six questions**: who it is
for, what that person cannot do today, what good looks like as a checkable
statement, what it does not do, what it costs at the target scale, and which
blockers govern it.

**A product record states a need. It never states a structure.** A record
that names a data structure holds an architectural decision, and that
decision belongs in a decision record. A check enforces this.

A decision record cites no product record. A refined backlog item cites both.

## Task management

Work lives in `docs/backlog/`. One file is one item. An item moves between
three directories: `proposed/`, `refined/`, and `complete/`.

**An item is refined when its architectural impact review is done.** That
review names the decision records that govern the work, the records it will
change or create, and the blockers that hold it. An item that cannot answer
those stays in `proposed/`.

Take work from `refined/`. Read the backlog guide before you add an item or
move one.[^6]

## Repository layout

| Path | Contents |
|------|----------|
| `docs/adrs/draft/` | Decision records under review |
| `docs/adrs/background/` | Research that supports each decision record |
| `docs/reference/` | Registers of figures that change |
| `scripts/` | Checks that run in continuous integration |
| `.claude/rules/` | Rules that apply to all work in this repository |
| `docs/backlog/` | The work queue |
| `docs/product/` | Product requirement records |

## Definition of done

Work is done when the impact review was made before starting, the
implementation was checked against each governing record decision by decision,
the registers were updated, and every gate passes. The full rule holds the
detail.[^7]

## Documentation rules

All prose in this repository follows Simplified Technical English
(ASD-STE100). Every document stands alone. Every reference to external
material appears in a footnote. All footnotes appear in one section at
the end of the document.

The full rule holds the details.[^3]

A separate rule states what belongs in a decision record, what does not, and
how to decide whether a decision needs a record at all.[^4] A script checks the
mechanical part of it.[^5] Run the script before you hand over work that
touches a record.

The documents under `docs/adrs/` predate this rule. They use inline
links. Convert them before you extend them.

## Open questions

Three questions block design work. The project owner must answer them.

1. Name three archetypes that you expect to exist. The answer decides
   whether the engine needs archetype machinery at all.
2. Confirm the grid extent and the world shape. A rhombus world removes
   a coordinate conversion.
3. State the maximum faction count. Fog of war costs 21.0 MB for each
   faction at the target scale.

The decision record lists 16 open questions in total.[^1]

## References

[^1]: ADR-0001, Determinism. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^2]: Research reports 01 to 07. `docs/research/reports/`
[^3]: Documentation Rules. `.claude/rules/documentation.md`
[^4]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^5]: The record check. `scripts/check-adrs.sh`
[^6]: Backlog guide. `docs/backlog/README.md`
[^7]: Definition of Done. `.claude/rules/definition-of-done.md`
[^8]: Product requirement records. `docs/product/README.md`
