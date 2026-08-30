# Cachette

Cachette is a world simulation engine. The core is Rust. The control plane
is Python.

The engine simulates a hex world at three levels of detail. Level 0 holds
individual tiles and units. Level 1 summarises blocks of tiles at city
scale. Level 2 summarises blocks of level 1 cells at region scale. Level 0
is the only source of truth. The other two levels are derived projections.

The target scale is 16.7 million tiles and one million units.

## Status

The project is in design. This repository holds the decision records, the
research that supports them, and the development scaffolding.

**There is no engine yet.** The Rust crates hold stubs. The stubs exist to
prove that the quality gates and the test harnesses run. Replace the body
of each stub. Do not replace its shape.

## Layout

| Path | Contents |
|---|---|
| `crates/cachette-core/` | The simulation. It has no PyO3 dependency. |
| `crates/cachette-py/` | The PyO3 bindings. They depend on the core. |
| `python/cachette/` | The Python package. |
| `tests/` | The Python tests. They import the installed package. |
| `scripts/` | The checks that the lint cannot express. |
| `docs/adrs/` | The decision records. |
| `docs/research/` | The research that supports the records. |
| `docs/backlog/` | The work queue. |

The crate split is an architectural decision, not a layout preference. The
core crate has no PyO3 dependency at all, so a Python callback inside a
simulation step is a compile error.[^1]

## Build and test

The project needs the Rust toolchain, the `uv` Python tool, and the `just`
command runner.

```
just              # list the targets
just setup        # build the extension into a local environment
just check        # everything a commit must pass
```

The testing policy states which tests may see internals and which may
not.[^2] The contribution guide states how to add each kind of test.[^3]

## Hard invariants

These rules are structural. You cannot add them to the project later.

1. No floating point in simulated or aggregated state. The fixed-point
   scale is Q16.16.
2. All simulation arithmetic goes through the `sim_math` module.
3. Every random draw comes from a counter-based generator, keyed on the
   tuple of system, frame, entity and draw.
4. The `cachette-core` crate holds no PyO3 dependency.
5. Every event type is plain data, with declared padding and no boolean
   field.
6. The step releases the Python global interpreter lock for its whole run.
7. Events reach Python in batches at the frame barrier.
8. Every parallel result has a stable order. Thread completion order is
   never used.
9. Pyramid accumulators widen at level 1.

Two tests protect these rules. The first runs one tick at 1, 2 and 12
threads, then compares the event log byte for byte. The second hashes the
whole state each frame against a stored file. Both live in
`crates/cachette-core/tests/`.

## Target platform

The engine targets AWS Graviton servers. The primary target triple is
`aarch64-unknown-linux-gnu`. Development happens on x86-64 and on Apple
Silicon. Both are development targets only.

Apple Silicon uses a 128-byte cache line. Graviton uses a 64-byte cache
line. A local performance test therefore misleads on false sharing and on
alignment. Measure on the target.

## References

[^1]: ADR-0006, The Python boundary, decision D2. `docs/adrs/draft/adr-0006-python-boundary.md`
[^2]: Testing policy. `docs/TESTING.md`
[^3]: Contribution guide. `CONTRIBUTING.md`
