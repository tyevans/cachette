# Contributing to Cachette

Cachette is a world simulation engine. A Rust core runs the simulation. A
Python control plane issues commands. The project is in design, and the
Rust crates hold stubs.

Read this document before your first change. It is short on purpose.

## 1. Set up

You need three tools: the Rust toolchain, the `uv` Python tool, and the
`just` command runner. The file `rust-toolchain.toml` pins the compiler,
so `rustup` installs the right version for you.

```
just setup     # build the extension into a local environment
just           # list every target
just check     # everything a commit must pass
```

## 1a. Where things live

| Path | Contents |
|---|---|
| `crates/cachette-core/` | The simulation. It has no PyO3 dependency. |
| `crates/cachette-py/` | The PyO3 bindings. They depend on the core. |
| `python/cachette/` | The Python package. |
| `tests/` | The Python tests. They import the installed package. |
| `scripts/` | The checks that the lint cannot express. |
| `docs/product/` | The product requirement records. What is needed, and for whom. |
| `docs/adrs/` | The decision records. How the engine is built. |
| `docs/backlog/` | The work queue. |
| `docs/research/` | The research behind the records. |

The crate split is an architectural decision and not a layout preference.
The core crate has no PyO3 dependency at all, so a Python callback inside a
simulation step is a compile error.

## 2. Before you write code

Answer four questions. The definition-of-done rule holds the full
list.[^1]

- Which decision records govern this work? Read them. The registry lists
  them.[^2]
- Does this work contradict a record? If it does, stop. Change the work,
  or write a record that supersedes the old one.
- Does this work create a decision that no record holds? If it does, that
  record is a deliverable of the work.
- Is a value you need behind a blocker? Express the work parametrically
  and cite the blocker. Do not invent the value.

## 3. The rules you cannot break

These rules are structural. Nobody can add them to the project later.

**No floating point in simulated or aggregated state.** Float addition is
not associative, so a float sum drifts with the fold order.[^3] Use
`Fix32`, `Fix64` or `Accum`. Two checks enforce this. Clippy rejects a
named float type. A script rejects a float literal whose type the compiler
infers, and rejects the reassociating operations by name.

**All simulation arithmetic goes through `sim_math`.**[^4]

**The `cachette-core` crate never depends on PyO3.** That makes a Python
callback inside a simulation step a compile error rather than a review
comment. It also lets Miri run over the unsafe code.[^5] A script reads
the dependency tree and fails when PyO3 appears.

**Every random draw comes from a counter.** Key it on the tuple of system,
frame, entity and draw index. Thread-local generator state is
forbidden.[^6]

**Every parallel result has a declared order.** Write to indexed output
slots. Never use thread completion order. Never use work-stealing
order.[^7]

**Every event type is plain data.** Use `repr(C)`. Declare the padding. Do
not use `bool`; use `u8`. Undeclared padding holds uninitialised bytes,
and those bytes enter the state hash.[^8]

## 4. Write the tests through the front door

A test uses the public interface. The testing policy states the rule, what
may test internals, and how to add each kind of test.[^9] Read it before
you add a test.

Two tests protect the primary constraint: thread-count equivalence, and
the golden state hash. Both must stay green. A changed golden file is a
changed simulation, so read the difference before you commit it.

## 5. Write the prose to the documentation rule

All prose follows Simplified Technical English. Every document stands
alone. Every external reference sits in a numbered footnote, in one
`## References` section at the end. The full rule holds the details.[^10]

Prose means documentation, decision records, code comments, docstrings and
commit messages.

## 6. Before you open a pull request

```
just check
```

Do not hand over a red pipeline. If a gate cannot pass, remove it and say
why, rather than leaving it broken.

Report the work honestly. Say what you did and what you left undone. If a
test fails, give the output. Do not claim a measurement that nobody
took.[^1]

## References

[^1]: Definition of done. `.claude/rules/definition-of-done.md`
[^2]: ADR registry. `docs/adrs/REGISTRY.md`
[^3]: ADR-0001, Determinism as the primary constraint, decision D2. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^4]: ADR-0001, Determinism as the primary constraint, decision D3. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^5]: ADR-0006, The Python boundary, decision D2. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
[^6]: ADR-0001, Determinism as the primary constraint, decision D5. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^7]: ADR-0001, Determinism as the primary constraint, decision D6. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^8]: ADR-0001, Determinism as the primary constraint, decision D9. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^9]: Testing policy. `docs/TESTING.md`
[^10]: Documentation rules. `.claude/rules/documentation.md`
