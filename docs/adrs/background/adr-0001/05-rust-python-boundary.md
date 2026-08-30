# The Rust/Python Boundary — PyO3, the GIL, Zero-Copy, and Packaging

Research input for ADR-0001. Prepared 2026-08-30.
All version numbers come from crates.io and from the PyO3 guide on the date above.

---

## Executive summary

These are the recommendations. The detail sections give the evidence.

1. **Use PyO3 0.29 and the `Bound` API.** GIL-ref types are gone. The GIL
   vocabulary is also gone. `Python::with_gil` is now `Python::attach`.
   `Python::allow_threads` is now `Python::detach`. Write new code with the
   new names. Do not copy old tutorials.

2. **The per-call cost is not the risk.** One PyO3 call costs about 100 ns
   with simple arguments. 2000 commands per frame cost about 0.2 ms. That
   is 1.2% of a 16.7 ms frame. The risk is the per-entity loop. One million
   calls per frame costs about 100 ms. That is 6 frames per second. The
   design must make the second pattern difficult, not make the first pattern
   faster.

3. **Do not return raw NumPy views with unbounded lifetime.** Return views
   only inside an explicit scope. Use a Python context manager. Stamp each
   view with a generation counter. Invalidate every outstanding view at the
   frame barrier. This is the single most important safety decision in this
   area.

4. **Release the interpreter for the whole step with `Python::detach`.**
   Enforce this structurally. The step function must not hold a `Python`
   token. Then no Python callback can fire, because no code inside the step
   can call Python.

5. **Do not depend on free-threaded Python yet, but do not block it.**
   Free-threaded Python is supported but optional since 3.14 (PEP 779).
   PyO3 0.28 and later assume modules are thread-safe by default. Keep the
   default. Do not set `gil_used = true`.

6. **Do not plan for sub-interpreters.** PyO3 refuses to load in a second
   interpreter. Run several worlds as several `World` objects in one
   interpreter instead.

7. **Target `abi3-py311` from week one.** Set up maturin and cibuildwheel in
   week one. Add `abi3t` for Python 3.15 later, when 3.15 ships.

8. **Build a custom exception hierarchy with `create_exception!`.** Never
   return a bare `PyRuntimeError`. Attach the Rust error chain to the
   exception as data, not only as text.

9. **Keep Python.** Lua and Rhai are better for user scripting inside the
   frame. They are much worse for the research audience. NumPy is the
   differentiator. Do not give it up.

10. **Disagreement with the context brief:** the brief says "Bulk data access
    is zero-copy NumPy views onto component arrays". This is correct as an
    intent but incomplete as a decision. Archetype-chunked ECS storage is not
    one contiguous array per component. It is one array per component *per
    chunk*. A single flat NumPy view over all units does not exist. See
    section 3.3. The ADR must state which of the three options it takes.

---

## 1. PyO3 current state

### 1.1 Versions

| Crate | Latest stable | Note |
|---|---|---|
| `pyo3` | 0.29.2 | Released 2026-08-28 |
| `numpy` (rust-numpy) | 0.29.0 | Tracks the PyO3 minor version exactly |
| `maturin` | 1.15.0 | Build backend |
| `pyo3-stub-gen` | 0.23.0 | Third-party stub generator |
| `arrow` (arrow-rs) | 59.2.0 | Alternative data interchange |
| `thiserror` | 2.0.20 | Error definition |

The `numpy` crate version always matches the `pyo3` version. A mismatch does
not compile. Pin both crates together. Upgrade both together.

### 1.2 The API changes that matter

**GIL refs are gone.** Older PyO3 returned `&'py PyAny` and `&'py PyList`.
Version 0.21 added the `Bound<'py, T>` smart pointer. Version 0.23 removed
the old references. Use `Bound<'py, T>` for borrowed objects. Use
`Py<T>` for objects that outlive the current attachment.

**The GIL vocabulary is gone.** PyO3 renamed three items:

| Old name | New name |
|---|---|
| `Python::with_gil` | `Python::attach` |
| `Python::allow_threads` | `Python::detach` |
| `pyo3::prepare_freethreaded_python` | `Python::initialize` |

The rename is not cosmetic. On a free-threaded build there is no GIL. The
`Python<'py>` token now means "this thread is attached to the interpreter".
It no longer means "this thread holds the GIL".
(Source: <https://pyo3.rs/main/migration>)

**Free-threading is the default assumption.** Since 0.28, a `#[pymodule]`
declares that it does not need the GIL. Python therefore does not re-enable
the GIL when it imports the module. To opt out, write
`#[pymodule] #[pyo3(gil_used = true)]`. Do not opt out unless a real problem
appears.

**`#[pyclass]` types must be `Sync`.** This requirement came in with 0.23.
It applies on every build, not only the free-threaded build.

### 1.3 Recommended idioms today

- Use the `#[pymodule] mod` form, not the older function form. The stub
  tooling and the introspection feature only support the module form.
- Prefer `cast()` over `extract()` for native Python types. `extract()`
  builds a `PyErr` on failure. `cast()` does not.
- Pass Rust tuples as call arguments. PyO3 then uses the `vectorcall`
  protocol. A `PyTuple` argument forces the slower `tp_call` protocol.
- Get the token from `Bound::py()`. Do not call `Python::attach` again inside
  a function that already has a `Bound`.
- Consider the `pyo3_disable_reference_pool` cfg flag. It removes a global
  synchronization point. It also makes a wrong `Py<T>` drop panic.
  (Source: <https://pyo3.rs/main/performance>)

---

## 2. FFI call cost

### 2.1 Measured overhead

PyO3 issue #3827 tracks call overhead. Community measurements put a simple
PyO3 function call at about 20-40 ns above a plain C extension call. A plain
Python-level call into a C extension costs about 60-80 ns by itself. Add the
two. **Budget about 100 ns for a trivial PyO3 call with scalar arguments.**
(Sources: <https://github.com/PyO3/pyo3/issues/3827>,
<https://pythonspeed.com/articles/python-extension-performance/>)

Argument conversion dominates once arguments stop being scalars.
Approximate costs, from PyO3 discussion #2968 and from the performance guide:

| Argument shape | Approximate extra cost |
|---|---|
| `i64`, `f64`, `bool` | a few ns each |
| `&str` | ~20 ns, no copy on the UTF-8 fast path |
| `String` | ~40 ns plus the allocation |
| `Vec<u32>` of length N | ~10 ns per element; this is a real loop |
| `&PyAny` kept as an object | ~5 ns |
| Keyword arguments | ~50-100 ns extra, dict build plus parse |

The rule: a `Vec<T>` argument is O(N) work at the boundary. A NumPy array
argument is O(1) work at the boundary.

### 2.2 What this means for the stated budget

The brief targets about 2000 commands per frame at 60 Hz.

- Frame budget: 16.67 ms.
- 2000 calls at 100 ns: **0.2 ms, or 1.2% of the frame.**
- The FFI cost is not a problem at this scale.

Now consider the failure case the brief already names.

- One million entities, one Python call each: **100 ms per frame.**
- That is 6 frames per second, before any simulation work at all.

**The cliff sits at roughly 100,000 calls per frame.** At 100,000 calls the
boundary alone costs 10 ms, which is 60% of the frame. Below 10,000 calls the
boundary is noise.

A useful consequence: the design has about 50x headroom on the stated 2000.
Do not spend engineering effort on shaving nanoseconds off a single call.
Spend it on making the vectorized path the obvious path.

### 2.3 The hidden cost the brief does not mention

Selector construction happens in Python. A selector expression tree of ten
nodes costs ten Python object allocations. That is maybe 500 ns, which is
five times the FFI call it precedes. This is still small. It becomes large
if a user builds a selector inside a loop.

**Recommendation:** make selectors cheap to reuse. A selector should be an
immutable, hashable value. Cache the compiled Rust-side plan by selector
identity. Then a selector built once and used every frame costs one
compilation, not sixty per second.

---

## 3. Zero-copy data exchange

### 3.1 The mechanisms available

**rust-numpy (`numpy` crate 0.29).** This is the direct choice. It binds the
NumPy C API. Key types:

- `PyArray1<T>`, `PyArray2<T>` — owned handles to a NumPy array.
- `PyReadonlyArray1<'py, T>` — a checked read borrow.
- `PyReadwriteArray1<'py, T>` — a checked write borrow.
- `PyArray::borrow_from_array_bound` and the `from_slice` family — create a
  NumPy array that points at Rust memory.

rust-numpy has a borrow-checking module. It follows the NumPy base-object
chain and enforces one writer or many readers per allocation. It is
deliberately conservative. It rejects some cases that are in fact safe.
(Source: <https://docs.rs/numpy/latest/numpy/borrow/index.html>)

**Crucially, that borrow checker does not protect against the case we care
about.** The documentation says it does not defend against unsafe Rust, other
threads, or callbacks that mutate the array. Our risk is exactly that: Rust
reallocates a chunk while Python holds a view. rust-numpy will not catch it.

**The buffer protocol (PEP 3118).** PyO3 supports it through
`#[pyclass]` plus the buffer slots. It is more general than NumPy and needs no
NumPy dependency. It is slower to set up. Under `abi3` it needs Python 3.11 or
later. It gives you a release callback, which is useful: you learn when Python
drops the view.

**`__array_interface__`.** This is a pure-Python dict protocol. It needs no C
API. It is simple to emit. It gives no release notification at all. Reject it
for mutable views for that reason.

**DLPack.** This is the tensor exchange standard for machine-learning
frameworks. NumPy 1.22 and later support `np.from_dlpack`. It carries a
deleter callback, so lifetime is explicit. It is the right choice if the
research audience wants zero-copy handoff to PyTorch or JAX. It only handles
strided dense tensors of a numeric dtype. That fits component arrays well.

**Arrow / arrow-rs 59.** Arrow gives a language-neutral columnar format, a
C data interface with an explicit release callback, and good handling of
null masks and variable-length data. It is a heavier dependency. It is the
right answer if you ever want the event log to be readable by Polars or
DuckDB without a copy. It is over-engineered for per-frame component views.

### 3.2 The safety problem, stated plainly

A NumPy array that points at Rust memory is a raw pointer with a length. NumPy
does not know about Rust. If Rust does any of the following while Python holds
the array, the process may read freed memory:

- A chunk grows and reallocates.
- An entity is despawned and the ECS swap-removes it, moving another entity's
  data.
- A structural change moves an entity between archetypes.
- The world is dropped.

Rust's lifetimes do not help. The `Py<PyArray1<u32>>` handle carries no Rust
lifetime. Python can store it in a global and use it ten frames later.

**This is the highest-severity risk in the whole Rust/Python area.** It is a
use-after-free with no compile-time or run-time check by default.

### 3.3 Disagreement with the context brief

The brief, item 7, says "Bulk data access is zero-copy NumPy views onto
component arrays." Item 2 says units live in "archetype-chunked ECS
... 16KB chunks ... pure SoA within a chunk."

These two statements are in tension. Chunked SoA storage does not give one
contiguous array per component. It gives one array per component per chunk.
With 16 KB chunks and, say, 64 bytes per unit, a chunk holds about 256 units.
One million units is about 4000 chunks. A single flat `position_x` view over
all units does not exist in memory.

The ADR must choose one of three options. It should say which.

**Option A — a list of chunk views.** Return a Python list of 4000 small
NumPy arrays. Truly zero-copy. Forces the user to loop over chunks. 4000
Python-level iterations per access is acceptable, but it is not the clean
`world.units.position_x` the brief implies.

**Option B — gather into a scratch buffer.** Rust copies the requested
component for the selected entities into one contiguous scratch buffer, then
returns one flat view of that. This is *one copy*, not zero-copy. For one
million `f32` values the copy is 4 MB, which is about 0.3 ms at typical memory
bandwidth. It is entirely worth it for the ergonomics. The buffer is
Rust-owned, reused every frame, and invalidated at the barrier. This is the
recommended default.

**Option C — abandon chunking for hot components.** Keep a few very hot
components in one global contiguous array with a stable dense index. This
conflicts with the archetype design and should not be done for this reason
alone.

**Recommendation: Option B is the default, Option A is available for
power users under a clearly named method.** Call the fast path
`to_numpy()` (copies, always flat, always safe) and the sharp path
`iter_chunks()` (zero-copy, scoped, documented as advanced). Be honest in the
docs that `to_numpy()` copies. Do not claim zero-copy where you gather.

The brief's zero-copy claim survives for the tile grid. Tiles are "dense
struct-of-arrays indexed by grid position" and are not in the ECS. A tile
component *is* one flat array. Zero-copy views there are straightforward and
should be the flagship demonstration.

### 3.4 Recommended safety strategy

Use three layers together. Any one alone is not enough.

**Layer 1 — explicit view scope (the primary defence).**

Views only exist inside a context manager.

```python
with world.tiles.view() as v:
    elevation = v.elevation          # NumPy array, zero-copy
    mask = elevation > 100
# On exit, every array from v is set to length zero and marked read-only.
```

On exit, Rust must make the arrays harmless. Two ways to do this exist. The
robust one is to reset the NumPy array's shape to zero and clear its
`NPY_ARRAY_WRITEABLE` flag. A later read then returns an empty array rather
than freed memory. Keep a list of the `Py<PyArray>` handles issued inside the
scope so the exit can reach all of them.

**Layer 2 — generation stamps (the backstop).**

The world holds a `u64` structural generation counter. Every structural
change increments it. Every issued view records the generation at issue time.
Every Rust entry point that receives a view checks the stamp first. A stale
stamp raises `StaleViewError` immediately.

This does not stop a user who bypasses Rust and reads the NumPy array
directly. Layer 1 covers that case. Layer 2 covers the case where a stale view
is passed *back* into the engine.

**Layer 3 — a structural-change lock.**

While any view scope is open, the world refuses structural changes. `step()`,
`spawn()`, and `despawn()` raise `ViewsOpenError`. This is a counter, not a
real lock. It turns a use-after-free into a clear exception.

**Frame barrier invalidation.** At the barrier, bump the generation and close
any scope that the user forgot to close. Log a warning when this happens.

**What to avoid.** Do not rely on a Rust `Arc` keeping the allocation alive.
It keeps the *allocation* alive, but a swap-remove still moves another
entity's data into the slot the view points at. The reader then sees wrong
data with no crash. Silent wrong data is worse than a crash.

**Open question for the ADR author.** Should a view scope be allowed to span a
`step()` at all? The clean answer is no. The convenient answer is that
read-only views of tile data survive a step that only moves units. The clean
answer is far easier to make correct. Recommend starting with the clean
answer, and relaxing later if a real use case appears.

---

## 4. The GIL

### 4.1 Releasing it around the step

```rust
#[pymethods]
impl World {
    fn step(&mut self, py: Python<'_>) -> PyResult<StepReport> {
        let sealed = self.seal_command_queue()?;   // needs Python, cheap
        let report = py.detach(move || {
            self.sim.run_frame(sealed)             // no Python here at all
        });
        Ok(report)
    }
}
```

`Python::detach` releases the interpreter for the closure. `detach` requires
that the closure and its return type implement `Ungil`. `Ungil` is an
auto-trait that is not implemented for `Python<'py>` or for any GIL-bound
type. **This is the structural enforcement the brief asks for.** A `Python`
token cannot cross into the closure. Therefore no code inside the simulation
step can call Python. The compiler enforces it. No discipline or code review
is needed.

**Recommendation:** make this explicit in the crate structure. Put the
simulation in a `cachette-core` crate that has no PyO3 dependency at all.
Put the bindings in a `cachette-py` crate. Then it is not merely hard to call
Python from the step. It is impossible, because the core crate does not know
that Python exists. This is worth doing on day one. It is expensive to retrofit.

### 4.2 Pitfalls

- `detach` is not free. It costs roughly 30-60 ns each way. Never call it
  inside a loop. Call it once per step.
- `detach` on a free-threaded build still matters. The interpreter has
  "stop the world" pauses for garbage collection and for `os.fork()`. An
  attached thread that does long Rust work blocks every other thread during
  such a pause. Detach anyway.
- Do not hold a `Mutex` across a re-attach. Use `MutexExt::lock_py_attached`
  from PyO3 when a lock must be held while Python code runs. Otherwise a
  deadlock is possible against the stop-the-world pause.
- Panics inside `detach` unwind through the FFI boundary. See section 8.

### 4.3 Interaction with rayon

rayon inside `detach` is safe and correct. The closure runs with no
interpreter attachment. rayon worker threads are plain OS threads that never
touch Python. This composes cleanly.

Two cautions.

- rayon's global thread pool is process-wide. If the user also runs a
  Python thread pool, oversubscription is likely. Build a named rayon pool
  sized by an explicit parameter, and expose that parameter in the Python
  constructor. Default it to `available_parallelism() - 1`.
- Determinism. rayon's `par_iter` with `fold`/`reduce` on a non-associative
  operation gives non-deterministic results. The brief's monoid rule (item 4)
  already fixes this for aggregation, which is good design. Apply the same
  rule to command application: thread-local buffers, then deterministic
  concatenation in a fixed order, exactly as the brief's item 12 says.

### 4.4 Free-threaded Python

Status as of August 2026:

- PEP 779 was accepted on 2025-06-16. Free-threaded Python is officially
  supported, and no longer experimental, from Python 3.14. It is still not
  the default build. That is phase III, and it is not scheduled.
- The single-threaded performance gap closed to 5-10% on pyperformance,
  against a 15% ceiling set by the PEP.
- PyO3 has supported free-threading since 0.23. Since 0.28 it is the default
  assumption for modules.
- PEP 803 was accepted on 2026-03-30. It defines `abi3t`, a stable ABI that
  works on both free-threaded and GIL builds. It requires Python 3.15 or
  later. PyO3 exposes it as the `abi3t-py315` feature.

(Sources: <https://peps.python.org/pep-0779/>, <https://peps.python.org/pep-0803/>,
<https://pyo3.rs/v0.29.0/free-threading>)

**What free-threading changes for this design: very little, and that is the
point.** The design already releases the interpreter for the entire step. The
parallelism is already in Rust with rayon. Free-threading gives parallel
*Python*, and this design deliberately does not want parallel Python. The
benefit is real but small: several `World` objects can step in parallel
Python threads without contending on a GIL. That matters for the research
audience running many environments.

**Recommendations:**

- Do not set `gil_used = true`. Keep the default.
- Make every `#[pyclass]` `Sync`. This is required anyway since 0.23.
- Do not put a `RefCell` in a `#[pyclass]`. On a free-threaded build, two
  threads borrowing the same instance mutably produce a runtime exception,
  not a compile error. Use `Mutex` or `PyOnceLock` for shared state.
- Add a free-threaded job to CI from week one. A `3.14t` and a `3.14` job.
  This is cheap now and expensive to add after a year of drift.
- Do not ship `abi3t` wheels yet. Python 3.15 has not shipped. Revisit when
  it does.

---

## 5. Sub-interpreters and multiple worlds

**PyO3 does not support sub-interpreters.** A PyO3 module raises
`ImportError: "PyO3 modules do not yet support subinterpreters"` on a second
interpreter. The check compares `PyInterpreterState_GetID` against the value
stored at first import. PyO3 issues #576 and #3451 track the work. The
maintainers say sub-interpreter support needs a substantial API redesign. Do
not expect it soon.
(Source: <https://github.com/PyO3/pyo3/issues/3451>)

PEP 734 added `concurrent.interpreters` in Python 3.14. This makes
sub-interpreters easy to reach from Python. Users will try it. They will get
an `ImportError`. Document this clearly.

**Running several simulations in one process is still viable, and easy.**
Do it with several `World` objects, not several interpreters.

- A `World` is a `#[pyclass]`. Create as many as memory allows.
- Each holds its own arena, its own rayon pool handle, and its own RNG state.
- On a GIL build, `step()` on world A and `step()` on world B in two Python
  threads run in parallel, because both detach.
- On a free-threaded build, the setup and teardown around the step also run in
  parallel.

**Requirement:** no global mutable state in the Rust crate. No `static mut`.
No global registry keyed by nothing. The verb registry and the unit-type
stat table are immutable after construction; those may be shared behind an
`Arc`. This is a design rule worth writing into the ADR, because it is easy
to violate accidentally and hard to unwind.

For the research audience, a vectorized multi-world API is far better than
threads: `WorldBatch(n=64).step()` steps 64 worlds with rayon across worlds.
This gives one FFI call for 64 steps and perfect parallelism. Recommend it as
a first-class API rather than an afterthought.

---

## 6. Error mapping

### 6.1 The target

The brief, item 10, already says the right thing: partial failure returns
summaries, not exceptions. Keep that. Exceptions are then reserved for real
programming errors: a bad selector, a wrong dtype, a stale view, a missing
verb. There will be few of them, and each deserves a specific type.

### 6.2 The hierarchy

Define one root and specific leaves. A user can then catch broadly or
narrowly.

```
CachetteError(Exception)
├── SelectorError
│   ├── UnknownComponentError
│   └── SelectorTypeError
├── VerbError
│   ├── UnknownVerbError
│   └── VerbParamError
├── ViewError
│   ├── StaleViewError
│   └── ViewsOpenError
├── DeterminismError
└── SimulationPanic          # a Rust panic reached the boundary
```

In Rust:

```rust
pyo3::create_exception!(cachette, CachetteError, pyo3::exceptions::PyException);
pyo3::create_exception!(cachette, SelectorError, CachetteError);
pyo3::create_exception!(cachette, UnknownComponentError, SelectorError);
```

Export them with `#[pymodule_export]`.
(Source: <https://pyo3.rs/v0.29.0/exception>)

**Note on `abi3`:** subclassing a built-in exception with
`#[pyclass(extends = PyException)]` needs Python 3.12 or later under `abi3`.
`create_exception!` does not have this restriction. Prefer `create_exception!`
for that reason. If you need structured fields, attach them as attributes
after construction rather than by subclassing with `#[pyclass]`.

### 6.3 Preserving Rust context

Define the Rust side with `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum SelectorError {
    #[error("unknown component '{name}'; did you mean '{suggestion}'?")]
    UnknownComponent { name: String, suggestion: String },
    #[error("component '{name}' has dtype {actual}, not {expected}")]
    DtypeMismatch { name: String, actual: &'static str, expected: &'static str },
}
```

Then write one conversion that walks the `source()` chain and puts the whole
chain on the Python exception:

```rust
impl From<SelectorError> for PyErr {
    fn from(e: SelectorError) -> PyErr {
        let chain: Vec<String> = std::iter::successors(
            Some(&e as &dyn std::error::Error),
            |e| e.source(),
        ).map(|e| e.to_string()).collect();
        let err = match e {
            SelectorError::UnknownComponent { .. } => UnknownComponentError::new_err(chain[0].clone()),
            SelectorError::DtypeMismatch { .. }    => SelectorTypeError::new_err(chain[0].clone()),
        };
        Python::attach(|py| {
            let _ = err.value(py).setattr("rust_chain", chain);
        });
        err
    }
}
```

Three rules make the messages good.

1. **Name the thing that was wrong, and suggest the right thing.** A
   Levenshtein match against the component registry costs nothing and saves
   the user a doc lookup. This is the single highest-value error-message
   feature for a data API.
2. **Never write a `From<anyhow::Error> for PyErr` that produces
   `PyRuntimeError`.** It is tempting and it destroys the hierarchy. If you
   must have a catch-all, make it `CachetteError`, not `PyRuntimeError`.
3. **Put the Python-side context in.** The user's traceback shows the Python
   call site already. Add what Rust knows and Python cannot see: the world
   generation, the frame number, the entity count. Attach these as exception
   attributes so a test can assert on them.

---

## 7. Packaging

Solve all of this in week one. Every item below is cheap now and painful in
month six.

### 7.1 maturin and pyproject

Use maturin 1.15 as the build backend. It handles the Rust build, the wheel
metadata, the `.pyi` inclusion, and the CI generation.

`maturin generate-ci github` writes a working GitHub Actions matrix. Start
from it. It is not perfect but it is a correct skeleton.

Use a mixed Rust/Python layout: a `python/cachette/` directory with pure
Python code beside the compiled `cachette._core` module. This matters. Some
things belong in Python: the selector builder's operator overloading, the
context manager, `__repr__`, docstrings, and anything a user should be able
to read. Do not write these in Rust just because Rust is available.

### 7.2 abi3

`abi3` builds one wheel per platform that works on every Python from the
target version upward. Without it you build one wheel per Python version per
platform. With four Python versions and five platforms, that is 20 wheels
against 5.

**Recommend `abi3-py311`.** Rationale:

- Python 3.9 and 3.10 are at or near end of life. 3.11 is a reasonable floor
  in late 2026.
- The buffer protocol under `abi3` needs 3.11. If you want a buffer-protocol
  fallback beside NumPy, 3.11 is the floor anyway.
- 3.10 would gain a little reach and lose the buffer protocol. Not worth it.
- 3.12 would gain exception subclassing under `abi3`. `create_exception!`
  makes that unnecessary. Not worth the lost reach.

**What abi3 costs you:**

- Some per-version optimizations are unavailable. The measured cost is small,
  in the low single-digit percent on call-heavy code.
- Text signatures on classes need 3.10.
- The `dict`/`weakref` pyclass options need 3.9.
- **`abi3` wheels do not load on free-threaded Python at all.** The feature is
  ignored on a free-threaded build. Free-threaded builds need version-specific
  wheels until `abi3t` and Python 3.15 arrive.

(Source: <https://pyo3.rs/v0.29.0/building-and-distribution>)

So the real wheel matrix is: one `abi3-py311` wheel per platform for GIL
builds, plus one version-specific wheel per platform per free-threaded Python
version. That is 5 plus maybe 5. It is manageable.

### 7.3 Platform matrix

| Platform | Target | Note |
|---|---|---|
| Linux x86_64 | `manylinux_2_28` | Use maturin's container. `manylinux2014` is going out of support. |
| Linux aarch64 | `manylinux_2_28` | Cross-compile with `zig` via `maturin --zig`, or use native ARM runners. |
| macOS arm64 | 11.0 minimum | Apple Silicon is the common case now. |
| macOS x86_64 | 10.12 minimum | Still worth shipping. |
| Windows x86_64 | MSVC | The `x86` 32-bit target is not worth it. |

Do not build `universal2` macOS wheels. Ship two separate wheels instead.
`universal2` doubles the wheel size and the build time to serve a case that
`pip` already handles by selecting the right tag.

Use `cibuildwheel` if you also need to test the wheels in a clean environment,
which you should. Otherwise maturin alone is enough. `cibuildwheel` supports
maturin as a build frontend, so the two compose.

### 7.4 Size, time, and the things that bite

**Wheel size.** A Rust extension of this shape lands around 3-8 MB per wheel
before compression. Three actions cut this by half or better:

- `strip = "symbols"` in the release profile.
- `panic = "abort"` — but **do not do this.** See section 8. Panics must be
  caught at the boundary. `panic = "abort"` makes that impossible.
- `opt-level = 3` plus `lto = "thin"` and `codegen-units = 1`. This increases
  build time and reduces size and improves speed. Worth it for release.

**Build time.** A cold Rust release build with LTO of a project this size
takes 5-15 minutes per platform. Across 10 wheel jobs that is fine in
parallel. Use `sccache` or `Swatinem/rust-cache` in CI. Without a cache, CI
becomes the slowest part of the project inside two months.

**Week-one checklist.**

- [ ] `maturin generate-ci github`, committed and green.
- [ ] `abi3-py311` feature on, so the wheel count stays small.
- [ ] All five platforms building, even if the code is a stub.
- [ ] A `3.14t` free-threaded job, even if it only imports the module.
- [ ] `sccache` or `rust-cache` wired up.
- [ ] An sdist that builds. Test it: `pip install --no-binary :all:`.
- [ ] Trusted publishing to PyPI configured, so no API token lives in a secret.
- [ ] A minimum supported Rust version pinned in `Cargo.toml` and tested.
- [ ] `cargo-deny` for licence and advisory checks.

The one that always gets skipped and always hurts is the sdist. A broken sdist
is invisible until a user on an unusual platform tries to install.

---

## 8. Debugging across the boundary

### 8.1 Panic handling

A Rust panic that unwinds across the FFI boundary is undefined behaviour.
PyO3 handles the common cases for you. `#[pyfunction]` and `#[pymethods]`
wrap the body in `catch_unwind` and convert a panic into `pyo3::panic::PanicException`.

Two things this does not cover.

- A panic inside a rayon worker thread. rayon catches it and re-raises it on
  the thread that called `join`, which is inside `detach`, which is inside the
  wrapper. So this does work. But the payload is the worker's panic, and the
  backtrace points into rayon internals. Set a rayon `panic_handler` on the
  pool to log the real location before rayon re-raises.
- `panic = "abort"` in the release profile. This kills the process instead of
  raising a Python exception. **Do not set it.** The size saving is not worth
  losing every panic message.

Plumb `RUST_BACKTRACE`. The panic message reaches Python, but the Rust
backtrace does not, unless you capture it. Install a `panic::set_hook` at
module init that captures `std::backtrace::Backtrace` into a thread-local.
Read that thread-local in the `catch_unwind` handler and attach it to the
exception as a `rust_backtrace` attribute. Without this step, a panic in
production gives you a one-line message and no location.

### 8.2 Tools

| Tool | What it sees | Note |
|---|---|---|
| `py-spy` | Python frames, and Rust frames with `--native` | The best first tool. Attaches to a running process. No instrumentation. |
| `gdb` / `lldb` | Rust frames well, Python frames with the CPython gdb helper | Load `python-gdb.py` from the CPython source for `py-bt`. |
| `perf` + flamegraph | Everything, as symbols | Needs `debug = true` in the release profile. Keep it on; it does not slow the code. |
| `cargo flamegraph` | Rust only | Good for the step function in a pure-Rust harness. |
| `tracy` / `puffin` | Rust frame timing | Worth it for per-system frame timing later, not week one. |

**Set `debug = 1` in the release profile from the start.** It adds line-table
information, costs nothing at run time, and makes every profile and every
backtrace legible. It grows the wheel; strip the symbols into a separate
file if size matters.

The highest-value single investment here is a `RUST_BACKTRACE`-plumbed panic
hook plus `py-spy --native` documented in the contributor guide. Together
they cover most debugging needs without a debugger.

### 8.3 Mixed stack traces

The realistic outcome: a Python traceback that ends at the PyO3 call, plus a
Rust backtrace attached as an attribute. There is no interleaved single
stack. Do not promise one. Instead make the two halves easy to read together:
print the Rust backtrace in the exception `__str__` when a debug flag is set,
and hide it otherwise.

---

## 9. API design that discourages per-entity loops

This is a design problem, not a technology problem. The tactics that work:

**1. Do not provide the thing you do not want used.** The strongest tactic by
far. If there is no `world.get_entity(id)` that returns a Python object with
attributes, no one writes the loop. NumPy has no scalar element type that is
pleasant to use in a loop; that is why NumPy users vectorize. Polars removed
row iteration from the fast path for the same reason. Make the entity handle
opaque and nearly useless on its own.

**2. Make the fast path shorter to type than the slow path.**

```python
world.units.where(faction == 2).move_to(target)     # good, short
```

There should be no shorter way to express the same thing.

**3. Name the slow path so that it reads as a warning.** JAX does this well
with `jax.lax` versus Python control flow. Names like `slow_iter_entities()`
or `debug_each()` are honest and self-limiting. Do not name it `for_each`.

**4. Warn at run time on the pattern, not the call.** Count calls to the
per-entity path within one frame. Past a threshold, emit a `PerformanceWarning`
once with a link to the vectorized equivalent. NumPy does something similar
for slow paths. This catches the user who found the slow path anyway.

**5. Make selectors compose so the user never needs a loop to express a
condition.** The brief's design already does this. It is the right call. The
key detail: the selector must support the operations users actually reach for
loops to express — set difference, top-k, nearest, sort-by, random sample.
If a common need has no selector form, users will write the loop. Audit the
verb and selector list against the author's own game code, since the author is
audience number one.

**6. Return selectors, not IDs, from everything.** The brief's item 10 already
says this for rejections. Apply it everywhere. If no API ever hands a user a
list of IDs, no user can loop over one.

Prior art worth reading: Polars' expression API is the closest analogue to the
selector design and is worth copying in spirit. NumPy's fancy indexing is the
model for the mask-based selection. JAX's `vmap` shows how to let a user write
per-item logic that still executes vectorized — probably out of scope here,
but it is the answer if users demand custom verbs later.

---

## 10. Testing

**Determinism regression is the most important test, and the hardest.**

- Hash the whole world state at the end of every frame. Use a fast, stable
  hash such as `xxh3` over the raw component bytes and the tile arrays.
- A test runs a fixed scenario for N frames and asserts the hash sequence
  against a committed golden file.
- This catches almost everything: an ordering change, an uninitialized byte, a
  `HashMap` iteration leak, a floating-point reassociation.
- **Watch out for padding bytes.** Hashing a struct with padding gives
  non-deterministic results. Hash component arrays field by field, or make
  every struct `#[repr(C)]` with no padding and zero it on allocation.

**Open question the brief already raises:** bit-exact cross-platform, or
within-run only? This decides whether the golden hashes can be shared across
the CI matrix. Bit-exact across platforms requires no `f32` transcendentals
from `libm`, or a vendored `libm`. It requires no `fast-math`. It requires a
fixed rayon thread count or a fully order-independent reduction. It is
achievable but it constrains the whole codebase. **Recommend: choose
within-run determinism plus per-platform golden files first, and design so
that the stronger guarantee remains reachable.** Integer-only or fixed-point
arithmetic for anything gameplay-relevant is the escape hatch, and is worth
considering for movement and combat.

**Property-based testing across the boundary.** Use `hypothesis` on the
Python side. It is more valuable than `proptest` on the Rust side here,
because the properties that matter are boundary properties:

- A selector and its negation partition the world. Counts sum to the total.
- A command applied to an empty selector changes nothing. The state hash is
  unchanged.
- Selector composition is associative where it should be.
- `to_numpy()` of a selector has length equal to `count()`.
- A view scope that raises still closes and still invalidates.
- Any sequence of API calls leaves the world's internal invariants intact.
  Expose a `world._check_invariants()` method for tests, and call it after
  every hypothesis step. Use `hypothesis.stateful.RuleBasedStateMachine` for
  this. It is the single highest-value test harness for a stateful engine.

**Benchmarks.** Use both, for different jobs.

- `criterion` on the Rust side for the simulation systems. It gives good
  statistics and regression detection. Run it in CI on a dedicated runner or
  the noise will make it useless.
- `pytest-benchmark` on the Python side for boundary cost. This is what
  catches an accidental O(N) argument conversion.
- Add a specific benchmark for the thing you fear: N Python calls per frame,
  swept over N. It documents the cliff and detects regression in it.

**CI shape.**

| Job | Runs on | Purpose |
|---|---|---|
| `cargo test` | Linux only | Fast Rust unit tests |
| `cargo clippy -D warnings` | Linux | Lint |
| `cargo miri` on core | Linux | Catches unsafe-code errors in the ECS |
| `pytest` | all 5 platforms | Boundary correctness |
| `pytest` on 3.14t | Linux | Free-threading |
| determinism golden | all platforms | The cross-platform question above |
| `criterion` | one pinned runner | Performance regression |
| wheel build | all 5 platforms | Release readiness, on every merge to main |

Run `miri` on the core crate. The ECS has unsafe code by necessity: raw chunk
pointers, transmutes to component slices, manual layout. `miri` finds
aliasing and provenance errors that no test will. It cannot run PyO3 code, so
the split into `cachette-core` and `cachette-py` recommended in section 4.1
pays off here too.

---

## 11. Type stubs and IDE experience

Two options exist today.

**`pyo3-stub-gen` 0.23 (Jij-Inc).** Third-party, mature, widely used. You
annotate with `#[gen_stub_pyfunction]` beside the PyO3 attribute. A small
binary target runs at build time and writes the `.pyi`. maturin then picks it
up. It supports type aliases, re-exports, and `__init__.pyi` generation. The
minimum Python version is 3.10.
(Source: <https://github.com/Jij-Inc/pyo3-stub-gen>)

**PyO3's own `experimental-inspect`.** PyO3 embeds JSON introspection data in
the binary. The `pyo3-introspection` crate reads it. maturin can drive it.
It is explicitly still in active development. It only supports the
`#[pymodule] mod` form, and it cannot see inside `#[pymodule_init]`.
(Source: <https://pyo3.rs/v0.29.0/type-stub>)

**Recommendation: use `pyo3-stub-gen` now.** It works today. Track the PyO3
native feature and switch when it stabilizes, since the native one will need
no duplicate annotations.

**Keeping stubs in sync — this is the part that fails.** Generated stubs drift
silently. The fix is a CI check, not discipline:

```
maturin develop && cargo run --bin stub_gen && git diff --exit-code *.pyi
```

A pull request that changes the API and does not regenerate the stub then
fails. Add this in week one.

**Hand-write the stubs for the Python-side code.** The selector builder, the
context manager, and anything with interesting typing should live in Python
with real annotations, not be generated. Generated stubs are always worse than
hand-written ones for the parts users read most. The selector API in
particular wants overloads and `Literal` types that no generator will infer.

Run `mypy --strict` and `pyright` over the example code in CI. This tests the
stubs from the user's side, which is the only test that matters for them.

---

## 12. Is Python the right choice at all?

Compare against embedding Lua (via `mlua`) or Rhai.

| Dimension | Python + PyO3 | Lua (mlua) | Rhai |
|---|---|---|---|
| Call overhead | ~100 ns | ~10-50 ns | ~100-500 ns |
| Numeric/array ecosystem | NumPy, SciPy, PyTorch, Polars | almost none | none |
| Can run inside the frame | no, by design | yes | yes |
| Sandboxing for user mods | poor | good | very good |
| Embedding effort | high: packaging, ABI, wheels | low: one crate | very low: pure Rust |
| Distribution | wheels on 5 platforms | ships inside your binary | ships inside your binary |
| Debugging | hard, two runtimes | moderate | easy, one runtime |
| Hiring / familiarity | very high | moderate | very low |
| Determinism control | full, since scripts do not run in-frame | needs care with `pairs` order | good |

**By audience:**

*Audience 1, the author building a game.* Lua or Rhai is arguably better. The
scripting sits inside the frame, the packaging problem disappears, and
debugging is one runtime. But the author is one person who already knows
Python, and the control-plane design means game logic lives in Rust verbs
anyway. Python is adequate. Neutral to slightly negative.

*Audience 2, other simulation developers.* Python wins clearly. It is the
language they already use for tooling, analysis, and glue. A Lua-scripted Rust
engine has a much smaller addressable audience. Positive.

*Audience 3, RL and agent-based-model researchers.* Python wins overwhelmingly.
This audience does not merely prefer Python; it is defined by NumPy,
Gymnasium, PyTorch, and JAX. A `step()` returning NumPy arrays is the entire
value proposition. Lua would eliminate this audience completely. Strongly
positive, and this is the brief's stated differentiator.

**Verdict: keep Python.** The brief names audience 3 as "the clearest
differentiator". That judgement is correct and it settles the question.
Lua and Rhai cannot serve it at all.

**However — the two are not exclusive, and this matters later.** The brief
defers a "modding DSL or bytecode VM". If that returns, embedding Rhai for
in-frame user logic beside Python for out-of-frame control is a good
architecture, not a contradiction. Python is the control plane. Rhai would be
the extension plane. They occupy different slots. Keeping this door open costs
nothing today: it only requires that verbs be registered through a trait
rather than hard-coded in a match. Recommend designing the verb registry that
way from the start.

---

## 13. Open questions for the ADR author

1. **Chunked storage versus flat NumPy views.** Section 3.3. Which of options
   A, B, C? This must be resolved before the ECS layout is fixed. It is the
   most consequential open item in this area.

2. **May a view scope span a `step()`?** Section 3.4. Recommend no.

3. **Cross-platform bit-exact determinism, or within-run only?** Section 10.
   The brief already lists this. It has more consequences than it appears to:
   it decides the golden-file strategy, whether `f32` is usable for gameplay
   state, and whether rayon thread count must be fixed.

4. **Does the engine own a rayon pool, or use the global one?** Recommend a
   named pool with an explicit size, for multi-world and for determinism.

5. **What is the minimum Python version?** Recommend 3.11. Confirm this
   against the research audience's actual floor, which is sometimes older than
   expected in academic environments.

6. **Is `WorldBatch` in scope for v1?** Section 5. It is the highest-value
   feature for audience 3 and it constrains the `World` API, so decide early
   rather than retrofit.

7. **How are custom verbs added?** If Python may register a verb, it must run
   in-frame and everything in section 4.1 breaks. Recommend: no Python verbs.
   Custom verbs require a Rust plugin crate. State this explicitly in the ADR,
   because users will ask, and a "maybe later" answer will leak into the
   design.

---

## Sources

- PyO3 user guide, v0.29 — <https://pyo3.rs/v0.29.0/>
- PyO3 migration guide — <https://pyo3.rs/main/migration>
- PyO3 performance guide — <https://pyo3.rs/main/performance>
- PyO3 free-threading guide — <https://pyo3.rs/v0.29.0/free-threading>
- PyO3 building and distribution — <https://pyo3.rs/v0.29.0/building-and-distribution>
- PyO3 type stubs — <https://pyo3.rs/v0.29.0/type-stub>
- PyO3 issue #3827, call overhead — <https://github.com/PyO3/pyo3/issues/3827>
- PyO3 discussion #2968, `FromPyObject::extract` cost — <https://github.com/PyO3/pyo3/discussions/2968>
- PyO3 issue #3451, sub-interpreter tracking — <https://github.com/PyO3/pyo3/issues/3451>
- rust-numpy borrow module — <https://docs.rs/numpy/latest/numpy/borrow/index.html>
- rust-numpy repository — <https://github.com/PyO3/rust-numpy>
- PEP 779, free-threaded supported status — <https://peps.python.org/pep-0779/>
- PEP 803, stable ABI for free-threaded Python — <https://peps.python.org/pep-0803/>
- PEP 734, multiple interpreters in the stdlib — <https://peps.python.org/pep-0734/>
- pyo3-stub-gen — <https://github.com/Jij-Inc/pyo3-stub-gen>
- "The hidden performance overhead of Python C extensions" — <https://pythonspeed.com/articles/python-extension-performance/>
- Python free-threading guide — <https://py-free-threading.github.io/>
