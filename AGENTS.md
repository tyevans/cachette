# Cachette

Cachette is a world simulation engine. The core is Rust. The control
plane is Python.

The engine simulates a hex world at three levels of detail. Level 0 holds
individual tiles and units. Level 1 summarises blocks of tiles at city
scale. Level 2 summarises blocks of level 1 cells at region scale. Level 0
is the only source of truth. A summarised level is a derived projection.

**Level 2 does not exist. The pyramid holds one derived level.** Read the
three levels as the target, and read the code for what is built. A reader
who takes the paragraph above for the code plans against a level that
nothing writes.

The target scale is 16.7 million tiles and one million units.

## Status

The determinism core is accepted and the foundation crate exists. **Read the
registry for which records are binding, and read those records before you
write code.**[^1] The accepted records are in one directory.[^2]

Do not read a count here as the list. This section held one, and it was wrong
by seven records before anyone noticed. The findings register holds this shape
and now holds two instances of it in this file.[^3]

The rest of the design is open. Most registry rows reserve a number and have
no file yet. Research reports support them.[^4]

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
2. **Route all simulation arithmetic through the `sim_math` module.** Two
   mechanisms enforce this boundary, because one is not enough. A lint bans
   the float types by name. A script catches what the lint cannot see: a
   float literal whose type is inferred. It also names the reassociating
   methods, which the compiler rejected on the old stable pin and which now
   compile. The lint can name them too, and does not yet.
3. **Seed every random draw from a counter-based generator.** Key each
   draw on the tuple (system, frame, entity, draw). Do not use
   thread-local random state. Thread-local state destroys determinism.
4. **Keep the `cachette-core` crate free of any PyO3 dependency.** This
   makes a mid-step Python callback a compile error. It also lets Miri check the
   unsafe code, and `just miri` runs it.
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
   over 16.7 million tiles reaches 4,258,500,000. That is inside a `u32`
   by 0.85 percent, and it passes the ceiling above 16,843,009 tiles. An
   accumulator must not depend on that margin. Use `i64`.

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
written, in the same way the decision registry does.[^5]

A record moves through four directories: `idea/`, `shaped/`, `accepted/` and
`shipped/`. **A record is shaped when it answers six questions**: who it is
for, what that person cannot do today, what good looks like as a checkable
statement, what it does not do, what it costs at the target scale, and which
blockers govern it.

**A product record states a need. It never states a structure.** A record
that names a data structure holds an architectural decision, and that
decision belongs in a decision record. A check enforces this.

A decision record may cite a product record for the need that made a choice
hard. It must not take a figure from one. A refined backlog item cites both the
records that govern it and the need it serves.

## Task management

Work lives in `docs/backlog/`. One file is one item. An item moves between
three directories: `proposed/`, `refined/`, and `complete/`.

**An item is refined when its architectural impact review is done.** That
review names the decision records that govern the work, the records it will
change or create, and the blockers that hold it. An item that cannot answer
those stays in `proposed/`.

Take the highest item you can start from the priority index.[^6] Each of the
three systems has one, and each states the order between the things that are
open. A check fails when an open item is missing from its index.

Take work from `refined/`. Read the backlog guide before you add an item or
move one.[^7]

## Repository layout

| Path | Contents |
|------|----------|
| `docs/adrs/accepted/` | Binding decision records |
| `docs/adrs/draft/` | Decision records under review |
| `docs/research/reports/` | Research that supports each decision record |
| `crates/` | The Rust core and the Python bindings |
| `python/` | The Python control plane package |
| `docs/reference/` | Registers of figures that change |
| `scripts/` | Checks that run in continuous integration |
| `.agents/rules/` | Rules that apply to all work in this repository |
| `.claude/rules/` | Rules that apply to all work in this repository |
| `docs/backlog/` | The work queue |
| `docs/product/` | Product requirement records |

## Definition of done

Work is done when the impact review was made before starting, the
implementation was checked against each governing record decision by decision,
the registers were updated, and every gate passes. The full rule holds the
detail.[^8]

## Documentation rules

All prose in this repository follows Simplified Technical English
(ASD-STE100). Every document stands alone. Every reference to external
material appears in a footnote. All footnotes appear in one section at
the end of the document.

The full rule holds the details.[^9]

A separate rule states what belongs in a decision record, what does not, and
how to decide whether a decision needs a record at all.[^10] A script checks the
mechanical part of it.[^11] Run the script before you hand over work that
touches a record.

The documents under `docs/adrs/` predate this rule. They use inline
links. Convert them before you extend them.

## Open questions

**Read the blockers register for the list.**[^12] Do not read this section as
the list. A register does not decay; a summary does. This section held a
summary once, it went stale, and the finding records what that cost.[^3]

**One blocker the project owner owns is open, and it holds one question:
whether an upgrade changes hands when the ground does.** The register is the
current statement of it.[^12] The questions beside it are answered. A unit
builds only on ground that its own faction holds. Anyone may destroy an
upgrade. A unit-level destruction takes work, and a faction-level removal is
instant. The scale constants table holds the values.[^13]

A benchmark now runs on the target platform, and a register holds what it
measured.[^14] Most cost figures are still derived, and the blocker that
says which stays open.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: The accepted records. `docs/adrs/accepted/`
[^3]: Findings register, FND-039. `docs/FINDINGS.md`
[^4]: Research reports 01 to 07. `docs/research/reports/`
[^5]: Product requirement records. `docs/product/README.md`
[^6]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^7]: Backlog guide. `docs/backlog/README.md`
[^8]: Definition of Done. `.agents/rules/definition-of-done.md`
[^9]: Documentation Rules. `.agents/rules/documentation.md`
[^10]: Decision Record Scope. `.agents/rules/adr-scope.md`
[^11]: The record check. `scripts/check-adrs.sh`
[^12]: Blockers register. `docs/BLOCKERS.md`
[^13]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^14]: Target platform costs. `docs/reference/graviton-costs.md`
