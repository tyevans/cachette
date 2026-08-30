# Testing policy

This document states how this project tests its code. It states which
tests may see internal code and which may not. It states how to add each
kind of test.

Cachette is a world simulation engine. A Rust core runs the simulation. A
Python control plane issues commands. The engine must produce identical
state for the same inputs at any thread count. That property is the
primary constraint of the project, and it decides the shape of the test
suite.[^1]

## 1. The rule: test through the front door

**A test uses the public interface.** It calls the same names that a user
calls. It does not reach into a private module and it does not read a
private field.

A test that goes through the public interface tests the product. A test
that reaches into internal code tests one implementation of the product.
The second kind fails when you refactor, and it passes when the public
interface is broken.

The project enforces this rule with structure, not with review.

**Rust.** The tests live in the `tests/` directory of each crate. A file
in that directory compiles as a separate crate. It links the library the
way a user links it, so it sees the public API and nothing else. A private
item is a compile error there.

**Python.** The tests live in the `tests/` directory at the top of the
repository. They import the installed package. The source directory is
never on the import path. A session-start check in `tests/conftest.py`
fails the run if a test imports the source tree instead of the install.

## 2. What may test internals

Three kinds of test may see internal code. Each has a reason.

**A unit test inside a Rust source file.** Write it in a `#[cfg(test)]`
module next to the code. Use it for a function that has a hard internal
contract, such as a rounding rule or a bit layout. Keep it small. The
public tests remain the main suite.

**A test of an invariant that the public interface cannot observe.** The
correct answer is usually to expose a checking method rather than to reach
inside. The engine already does this: `check_invariants` is a public
method that exists so a test can call it.[^2]

**A layout assertion.** A test may assert the size, the alignment and the
padding of a type. The layout is part of the contract, because an event
type must be plain data.[^3]

Everything else goes through the front door.

## 3. The kinds of test

### 3.1 The two determinism tests

These two tests protect the primary constraint. They exist before the
first solver.[^4] Both live in `crates/cachette-core/tests/`.

**Thread-count equivalence.** The test runs the same tick at 1, 2 and 12
threads. It compares the event log byte for byte. It also compares the
state hash.

To add a scenario, add a row to the `SCENARIOS` table in
`thread_equivalence.rs`. The test then runs the new row at every thread
count.

**The golden state hash.** The test hashes the whole state each frame and
compares the sequence against a stored file. The stored files live in
`crates/cachette-core/tests/golden/`.

To add a scenario, add a row to the `SCENARIOS` table in
`golden_state_hash.rs`, then record the file:

```
just golden
```

**A determinism test must have a proven failure mode.** A test that
compares a run against itself always passes. The core crate carries a
test-only feature, `probe-nondeterminism`. It makes the step join its
output slots in reverse order, which breaks the ordering rule on
purpose.[^5] Under that feature the thread-count test must fail and
`determinism_probe.rs` must pass.

```
just probe
```

Never build a shipped artefact with that feature.

**Read the difference before you commit a changed golden file.** A changed
sequence is a changed simulation. Agree that the change is correct first.

### 3.2 Property tests in Rust

The project uses `proptest`. The worked example is
`crates/cachette-core/tests/sim_math_properties.rs`.

Test the algebraic laws that determinism depends on. Integer addition and
bitwise OR are exactly commutative and associative, so a parallel
reduction over them gives one answer at any order.[^6] A property test is
how the project proves that a new operation holds the same law.

To add one, add a `proptest!` block to an existing file, or add a new file
in the `tests/` directory of the crate.

### 3.3 The property-based state machine in Python

The project uses `hypothesis`. The harness is
`tests/test_state_machine.py`.

The record names this the highest-value harness for a stateful engine, and
it puts the harness on the Python side, because the properties that matter
are properties of the boundary.[^7]

The machine generates a sequence of commands, applies them, and calls the
invariant check after every rule.

To add a command, add a `@rule` method. To add a property, add an
`@invariant` method. The file lists the properties that the record names
and that the engine cannot yet support.

### 3.4 Mutation testing

A mutation test changes the code and expects a test to fail. It measures
whether the suite has teeth.

**Rust.** The project uses `cargo-mutants`. The configuration is
`.cargo/mutants.toml`. The gate is slow, so it does not run on every
commit. It runs on a schedule and on a pull request that carries the
`mutants` label.

```
just mutants
```

The last run reported 105 mutants: 94 caught, 10 unviable, and 1 missed.
The missed one changes `>` into `>=` where the code tests the sign of a
value that is never zero, so both forms give the same answer. It is an
equivalent mutant and no test can kill it.

Read a missed mutant before you exclude it. A real gap looks the same as
an equivalent mutant in the report, and the first run over this code found
five real gaps in the value types.

**Python.** The tool is `mutmut`, and the gate is not switched on yet.

`mutmut` wins against `cosmic-ray` for this project. It runs in one
process and it reads its configuration from `pyproject.toml`.
`cosmic-ray` needs a session database and a distributed executor, and this
project has one small package to mutate. The extra machinery buys nothing
here.

The gate is off because there is nothing to mutate. The Python package
re-exports the compiled module and holds no logic of its own. `mutmut`
generates mutants, finds that no test reaches them, and exits with an
error. A gate that always fails trains everyone to ignore it.

**Switch the gate on when the package holds Python logic.** The selector
builder is the first such code. Add `mutmut` to the development group, add
a `[tool.mutmut]` section with `source_paths`, and add a job to
`.github/workflows/mutants.yml`.

### 3.5 Miri

Miri finds aliasing and provenance defects in unsafe code. No test
replaces it. Miri cannot run the Python interpreter, so it can only run
over a crate that does not link the interpreter. The crate split is what
makes this possible.[^8]

Storage will hold unsafe code. Add the Miri job when that code arrives.
There is no unsafe code today, so there is no Miri job today.

## 4. How to add a test

**A Rust test through the public API.** Add a file to
`crates/cachette-core/tests/`. Import the crate by name. Write one
statement in each test name.

**A Rust unit test.** Add a `#[cfg(test)] mod tests` block at the end of
the source file. Justify why the test cannot go through the public API.

**A Python test.** Add a file to `tests/` named `test_*.py`. Import
`cachette`. Never import from `python/cachette/`.

**A slow test.** Mark it with `@pytest.mark.slow` in Python. In Rust, put
it behind the release profile or give it its own file and name it in the
slow gate of the `justfile`.

## 5. Running the gates

| Command | What it runs |
|---|---|
| `just fmt` | Format the Rust code and the Python code |
| `just lint` | Clippy, ruff, mypy, and the invariant scripts |
| `just test` | The fast tests on both sides |
| `just determinism` | The two determinism tests on their own |
| `just probe` | Prove that the determinism tests can fail |
| `just test-slow` | The release tests, the licence audit, the target check |
| `just mutants` | Mutation testing on both sides |
| `just check` | Everything a commit must pass |
| `just ci` | What continuous integration runs |

## 6. What the tests do not cover

**No measurement exists on the target platform.** Every cost figure in
this project is derived, not measured. A performance conclusion taken on a
development machine does not transfer, because the cache-line size and the
memory model differ.[^9] There are no benchmarks in this repository yet.

**The wheel matrix is minimal.** Continuous integration builds a wheel for
Linux on x86-64 only. The record names five platforms. The rest are
deferred.[^10]

**The `scripts` directory is not under the Python gate.** It holds
standalone repository tooling. The gates run over `python` and `tests`.
Bringing the tooling under the same lint is a separate change.

**There is no engine.** The Rust crates hold stubs. The harnesses are
real. The subjects are not.

## References

[^1]: ADR-0001, Determinism as the primary constraint. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^2]: ADR-0006, The Python boundary, decision D11. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
[^3]: ADR-0001, Determinism as the primary constraint, decision D9. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^4]: ADR-0001, Determinism as the primary constraint, decision D11. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^5]: Testing rules, section 1. `.claude/rules/testing.md`
[^6]: ADR-0001, Determinism as the primary constraint, decision D7. `docs/adrs/draft/adr-0001-determinism-outranks-every-other-constraint.md`
[^7]: ADR-0006, The Python boundary, decision D11. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
[^8]: ADR-0006, The Python boundary, decision D2. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
[^9]: ADR-0002, Target platform and value types, decision D8. `docs/adrs/draft/adr-0002-value-types-are-exact-and-sized-for-one-target.md`
[^10]: ADR-0006, The Python boundary, decision D8. `docs/adrs/draft/adr-0006-python-is-a-control-plane.md`
