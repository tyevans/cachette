# ADR-0006: Python is a control plane, and a crate split enforces it

**Status:** Draft
**Date:** 2026-08-30
**Depends on:** ADR-0001, determinism as the primary constraint. ADR-0003,
storage.

## Context

Cachette simulates about 16.7 million tiles and about one million units. The
core is Rust. The control plane is Python. This record fixes the shape of the
boundary between the two.

The engine serves three audiences. The first is the project owner, who builds
a game. The second is other simulation developers. The third is researchers in
reinforcement learning and in agent-based modelling. The third audience is the
clearest differentiator, and it is defined by NumPy and by the machine
learning frameworks that read NumPy.

The cost of one call across the boundary is not the risk. One call with scalar
arguments costs about 100 nanoseconds.[^1] Two thousand commands in one frame
cost about 0.2 milliseconds, which is close to noise. One million calls in one
frame cost about 100 milliseconds, which is six frames each second before the
simulation does any work.[^1] The design must make the second pattern
impossible. It must not make the first pattern faster.

The boundary carries three further risks. A NumPy array that points at Rust
memory is a raw pointer with a length, and Rust can free that memory while
Python holds the array. A Python callback that runs inside a simulation step
destroys the determinism that ADR-0001 makes the primary constraint. A
packaging problem that is cheap in week one is expensive in month six.

This record answers all four risks with structure rather than with discipline.

## Decision

### ADR-0006 D1 — Python is a control plane, not a data plane

Python issues set-valued commands. Rust resolves each command over a set and
runs it.

A command names a selector and a verb. A selector is a lazy expression tree
that Python builds and Rust evaluates. It holds no data and it names no
entity. A verb acts on the whole set that the selector resolves to.

Bulk data moves as whole arrays, in one call for each array. It never moves
one entity at a time.

A set-valued command permits a cheaper algorithm. One flow field serves a
whole set, where a batched loop would run one path search for each member.
The set-valued form is therefore not a wrapper over the loop. It is a
different algorithm, and this is the reason the command layer takes sets.

A selector is an immutable value with a stable identity. Rust caches the
compiled plan against that identity. A selector that a user builds once and
sends every frame therefore compiles once.[^1]

### ADR-0006 D2 — A crate split enforces the boundary at compile time

The simulation lives in a core crate. That crate has no PyO3 dependency at
all. The bindings live in a separate crate that depends on the core crate.

This is the strongest decision in this record. It converts a convention into a
compile error.

**Why a convention is not enough.** The rule "do not call Python from inside a
system" is a rule that a reviewer must enforce on every pull request forever.
One violation is enough. A Python callback inside a step reorders work,
allocates, releases and re-acquires the interpreter, and can raise. Each of
those breaks the byte-exact reproduction that ADR-0001 requires. The defect
appears as a hash mismatch a thousand frames later, and it costs days to
locate.

**Why the split is enough.** The core crate does not know that Python exists.
No type in it can name a Python object. No function in it can take an
interpreter token. A mid-step callback is therefore not hard to write. It is
impossible to write, and the compiler says so at the call site.

A second mechanism agrees with the first. The function that releases the
interpreter takes a closure, and that closure may not capture an interpreter
token.[^1] The token is not thread-safe, so the compiler rejects the capture.
The crate split and the closure bound catch the same mistake twice, from two
directions.

**The split also permits Miri.** Storage uses unsafe code by necessity: raw
pointers into the arena, manual layout, and casts to component slices.[^2]
Miri finds aliasing and provenance defects that no test finds. Miri cannot run
the interpreter, so it cannot run any crate that links PyO3. A crate that
mixes the simulation with the bindings therefore gives up Miri on the only
code that needs it. This alone justifies the split.

**Custom verbs are Rust, not Python.** A user cannot register a verb written
in Python. A Python verb would have to run inside the step, which D2 forbids
and the crate split prevents. A new verb is a Rust plugin crate that
implements the verb trait. The verb registry takes a trait rather than a fixed
match, so the plugin path stays open.

The split is cheap on the first day. It is expensive to retrofit, because
every retrofit means auditing every function signature in the simulation.

### ADR-0006 D3 — The interpreter is released for the whole step

The step function releases the Python global interpreter lock before it runs
any simulation work, and re-acquires it after the work finishes. No Python
code runs while the simulation runs.

The release happens once for each step. It costs 30 to 60 nanoseconds each
way, so it must never happen inside a loop.[^1]

The release still matters on a build that has no global interpreter lock. Such
a build stops all threads for garbage collection. An attached thread that runs
long Rust work delays every other thread during such a pause.[^1] Release the
interpreter in both cases.

Parallel Rust work inside the released region is safe. The worker threads
never touch Python. The engine owns a named worker pool with an explicit size
rather than using the process-wide default pool, because two worlds in one
process must not contend for one pool, and because the thread count is an
input to ADR-0001 D1.

**Events reach Python in batches at the frame barrier.** A system never calls
Python. The step collects events, orders them by the stable key that ADR-0001
D6 requires, and hands the whole batch to Python after the barrier. Batched
delivery is what makes D2 and D3 consistent with a Python program that still
observes what happened.

### ADR-0006 D4 — The no-loop rule is a function of scale, and a declared tier enforces it

Per-entity access from Python is forbidden for the mass tiers and permitted
for the character tier. The threshold is 262,144 entities, which is two to the
power of eighteen.[^3]

The threshold is derived, not chosen. One character decision costs about 20
microseconds.[^3] A simulated year is 1,200 ticks. At 262,144 characters the
Python pass costs about 4.4 milliseconds for each tick, which is the largest
single line in the budget. At one million it costs about 17 milliseconds for
each tick, which exceeds the whole remaining budget.[^3]

**The cost is not mostly the boundary.** About 30 percent of a decision is the
boundary and about 70 percent is the interpreter.[^3] Removing the boundary
entirely would not change the order of magnitude. The budget is therefore wall
time for each simulated period, not a count of calls. State it that way, and
measure it that way.

**A declared tier on the class enforces the rule.** Each entity class declares
its tier at registration. The mass tier holds tiles and units. The character
tier holds characters, offices and titles. A singleton tier holds the world
and the factions. The tier is a static property of the class. It is not a
property of the current count.

A runtime cardinality check is rejected. Such a check makes the same script
pass against 500 entities in development and fail against 50,000 in
production. The failure then appears far from its cause, and only at scale.
That is the worst available failure mode. With a declared tier the mistake
fails on the first call, in development, with a message that names the tier
and the correct method.

A load-time guard is the backstop. When the world loads, it compares the
declared character capacity against the ceiling and refuses to build the tier
above it. The guard runs once, at load, so it never surprises a running
script.

### ADR-0006 D5 — State plainly what copies and what does not

A whole component column is a zero-copy view. A subset is never a zero-copy
view.

Storage is a dense arena and is not archetype-chunked, so one component is one
flat array.[^2] Chunked storage would give one array for each component for
each chunk, and one million units would be about 4,000 chunks. No flat column
would exist. Dropping chunking is what restores the whole-column view.[^4]

A selected subset is not contiguous. Rust gathers the selected values into a
scratch buffer that it owns, reuses each frame, and invalidates at the
barrier. That gather is one copy. Four megabytes of gathered values cost about
0.3 milliseconds, and the ergonomics are worth it.[^1]

The method that returns a flat array copies. The documentation says so in the
first line of the docstring. The project does not claim zero copy where it
gathers. A separate, clearly named method returns the underlying arrays
without a copy, and its documentation marks it as advanced.

**Tile data is the flagship demonstration.** Tiles are a dense
struct-of-arrays indexed by grid position and are not in the entity
arena.[^2] One tile field is genuinely one flat array. The zero-copy claim is
true there without qualification, so tiles are the honest example to lead
with.

### ADR-0006 D6 — View safety needs three layers

An array that points into engine memory is unsafe by default. Rust may free
that memory, and it may move another entity's data into the slot the array
reads. Silent wrong data is worse than a crash. Three layers guard against
this, and no one of them is enough alone.

**Layer one is an explicit scope.** A view exists only inside a context
manager. The engine records every array it issues inside the scope. On exit it
sets each of those arrays to length zero and clears the writable flag. A later
read then returns an empty array rather than freed memory.

**Layer two is a generation stamp.** The world holds a counter that every
structural change increments. Every issued view records the counter value at
issue time. Every entry point that receives a view compares the stamp before
it reads anything, and raises a stale-view error when the stamp does not
match. Layer one covers a user who reads the array directly. Layer two covers
a user who passes a stale array back into the engine.

**Layer three is a structural-change lock.** While any scope is open, the
world refuses a step, a spawn and a despawn. The refusal raises a named
exception. The lock is a counter, not an operating-system lock. It turns a
use-after-free into a clear error message.

At the frame barrier the engine increments the generation and closes any scope
that the program left open. It logs a warning when it does so.

**A view scope may not span a step.** The clean rule is far easier to make
correct than the convenient rule, and a relaxation stays available later.

**The NumPy binding crate does not cover this.** That crate has its own borrow
checking, and its own documentation states that it does not defend against
unsafe Rust, against other threads, or against a callback that mutates the
array.[^5] The failure this record guards against is exactly that case. Do not
treat the crate's borrow checking as the defence.

Reference counting is not a defence either. Keeping the allocation alive does
not stop a swap-remove from moving a different entity's data into the slot the
view reads.

### ADR-0006 D7 — The API refuses the loop rather than discouraging it

A mass-tier selector raises a type error for the truth test, the length, the
iterator and the index operations. Each message names the method that the user
wanted.[^6] The truth test raises because a lazy value that pretends to be
eager will be used eagerly.

The strongest guardrail is to ship nothing. There is no method that returns a
mass-tier entity as a Python object with attributes. There is no method that
returns a list of mass-tier identifiers. A user cannot loop over a list that
no method hands out. Every method that returns a set of mass-tier entities
returns a selector.

The ergonomic path is the fast path. The vectorized expression is the shortest
way to write the intent, and no shorter way exists. A guardrail that costs the
user typing is a guardrail that the user works around.

The selector vocabulary must cover what users reach for a loop to express:
set difference, top-k, nearest, sort by key, and random sample. A common need
with no selector form is a loop that the design forced.

The character tier is the exception that ADR-0006 D4 defines. Its per-entity
handle is a generational handle into the arena. It is not a copy of the row
and it is not a mutable view.

### ADR-0006 D8 — Packaging is solved in week one, on a stub extension

The wheel pipeline is built before the engine exists. Every item below is
cheap now and painful later.

**Pin the interpreter binding crate and the array binding crate together.**
The array crate tracks the binding crate minor version exactly, and a mismatch
does not compile. Upgrade the pair in one change, never one alone. The exact
versions are living reference and do not belong in this record.[^1]

**Target a stable ABI with a stated minimum of Python 3.11.** One wheel for
each platform then serves every later Python version. Without it, the project
builds one wheel for each Python version for each platform. The 3.11 floor is
the right one: 3.9 has reached end of life, 3.10 reaches it within months, and
the buffer protocol under the stable ABI needs 3.11 in any case.[^7] The
minimum is a decision. Raise it only in a record that supersedes this one.

**A stable-ABI wheel does not load on a free-threaded interpreter.** The
feature is ignored on such a build.[^8] Free-threaded interpreters therefore
need a wheel for each version until a stable ABI that serves both builds is
usable. That ABI is specified and the binding crate already exposes it, but it
requires an interpreter version that has not shipped.[^9] Do not ship against
it yet. Revisit when the interpreter ships.

**Do not depend on free-threaded Python, and do not block it.** Free-threaded
Python is officially supported and is still not the default build.[^10] The
module declares that it does not need the global interpreter lock, which is
the current default, and the project keeps that default. Every class the
module exposes is thread-safe. No class holds a single-threaded interior
mutability type, because two threads that borrow such a type on a free-threaded
build raise at runtime rather than fail to compile. Free-threading changes
little here, and that is the point: this design already runs its parallel work
in Rust and deliberately does not want parallel Python.

**The platform matrix is five targets:** Linux on x86-64, Linux on
64-bit ARM, macOS on Apple Silicon, macOS on x86-64, and Windows on x86-64.
The Linux baseline is the current manylinux tag, not the superseded one.[^11]
Ship two macOS wheels rather than one combined wheel. Add a free-threaded job
from the first week, even when it only imports the module.

**Four release settings are decisions, not defaults.** Strip symbols. Keep
line-table debug information, because it costs nothing at run time and makes
every profile legible. Enable link-time optimisation. Never abort on panic,
because that gives up every panic message and breaks ADR-0006 D9.

**Test the source distribution.** It is the item that is always skipped, and a
broken source distribution is invisible until a user on an unusual platform
tries to install.

**Type stubs are checked in continuous integration, not maintained by
discipline.** The build regenerates the stubs and the job fails when the
result differs from the committed file. Hand-write the stubs for the parts
written in Python, because a generator infers neither the overloads nor the
literal types that the selector API needs.

### ADR-0006 D9 — Every error is typed, and no error is opaque

One root exception type holds the whole hierarchy. Under it sit specific types
for a selector error, a verb error, a view error, a determinism error, and a
panic that reached the boundary. A user can then catch broadly or narrowly.

The engine never raises a bare runtime error. A catch-all conversion that
produces one destroys the hierarchy, and it is the easiest mistake to make.
The catch-all is the root type of this hierarchy.

Rust defines its errors with a chained error type. One conversion walks the
chain and attaches the whole chain to the Python exception as an attribute.
The user then sees the Python call site from the traceback and the Rust
context from the attribute.

Three rules make a message useful. Name the thing that was wrong. Suggest the
closest valid name, which an edit-distance match against the registry supplies
at no cost. Attach what Rust knows and Python cannot see: the generation, the
frame number, and the entity count. A test can then assert on those
attributes.

Build the exception types with the macro that creates a new exception rather
than by subclassing an exception from a Python class, because subclassing
under the stable ABI needs a later Python version and the macro does not.[^12]

**Panics are caught at the boundary.** A panic that unwinds across the foreign
function boundary is undefined behaviour. The binding macros wrap each method
body and convert a panic into an exception, which is why the release profile
must not abort on panic. A panic in a worker thread re-raises on the thread
that joined it, which is inside the released region and therefore inside the
wrapper. Install a handler on the worker pool so the log records the real
location before the re-raise.

A panic message reaches Python but a Rust backtrace does not. Install a panic
hook at module initialisation that captures the backtrace into thread-local
storage, and attach it to the exception. Without this step a panic in
production gives one line and no location.

There is no single interleaved stack across the two runtimes. Do not promise
one.

**Partial failure is not an exception.** A command that could not apply to
every member of its set returns a summary and a selector of the rejected
members. Exceptions are for programming errors: an unknown component, a wrong
element type, a stale view, an unknown verb.

### ADR-0006 D10 — Many worlds in one interpreter, never many interpreters

The binding library does not support sub-interpreters. A module raises an
import error on a second interpreter, and support needs an API redesign that
is not close.[^13] The Python standard library now makes sub-interpreters easy
to reach, so users will try this and will meet that error. Say so in the
documentation.

Several simulations in one process are still easy. Run several world objects
instead. Each holds its own arena, its own worker pool handle and its own
random number state. Two worlds step in parallel in two Python threads,
because each releases the interpreter for its whole step.

**This requires that the Rust crates hold no global mutable state.** No mutable
static. No global registry. The verb registry and the unit type table are
immutable after construction and may be shared behind a reference count. This
rule is easy to violate by accident and hard to unwind later, which is why it
is a decision and not a guideline.

A batch object that steps many worlds in one call is a first-class part of the
API, not an afterthought. It gives one boundary call for many steps and
parallelism across worlds. It is the highest-value feature for the research
audience, and it constrains the world API, so it is decided now rather than
retrofitted.

### ADR-0006 D11 — Two test harnesses cover the boundary

**A property-based state machine drives the API.** The test generates
sequences of commands, applies them, and calls an invariant check after every
step. The engine exposes that check as a method for this purpose. The
properties that matter are boundary properties: a selector and its negation
partition the world and their counts sum to the total; a command on an empty
selector leaves the state hash unchanged; the length of a returned array
equals the reported count; a scope that raises still closes and still
invalidates its views. This is the highest-value harness for a stateful
engine, and it belongs on the Python side because the properties are Python
properties.

**Per-frame state hashing runs against a golden file.** ADR-0001 D11 already
requires this test and ADR-0001 D9 already gives the padding rule that stops
it from failing falsely. This record adds one requirement: the boundary must
not perturb the hash. A test runs a scenario with no Python interaction and
the same scenario with heavy read-only Python interaction, and asserts the
same hash sequence.

**One benchmark documents the cliff.** It sweeps the number of Python calls in
one frame and records the cost. It exists so that a regression in boundary
cost is visible, and so that the cliff is a measured number rather than a
claim.

Continuous integration runs Miri over the core crate. ADR-0006 D2 is what
makes that possible.

### ADR-0006 D12 — Python wins against an embedded scripting language

The alternative is an embedded language such as Lua or Rhai. Such a language
calls faster, ships inside the binary, needs no wheel, sandboxes well, and
debugs in one runtime. It can also run inside the frame, which this design
does not want.

For the project owner the two are close. The scripting would sit inside the
frame, the packaging problem would disappear, and debugging would be one
runtime. Against that, the game logic lives in Rust verbs under ADR-0006 D1,
so little script runs at all, and the owner already knows Python.

For other simulation developers Python wins clearly. It is the language they
already use for tooling, for analysis and for glue.

For the research audience Python wins absolutely. That audience is defined by
NumPy and by the frameworks that read NumPy. A step method that returns NumPy
arrays is the whole value proposition. An embedded language removes this
audience completely.

The third audience decides the question, and it decides it in one direction.

The two are not exclusive. If an in-frame modding language returns later, an
embedded language is the extension plane and Python remains the control plane.
They occupy different slots. Keeping that door open costs nothing today. It
requires only that the verb registry take a trait, which ADR-0006 D2 already
requires.

## Consequences

### What this buys

A boundary that the compiler enforces. A mid-step Python callback is a compile
error, so the largest determinism hazard outside Rust cannot occur.

Miri over the unsafe storage code, which no test replaces.

An honest zero-copy story. Whole columns and tile fields are genuinely free.
Everything else states its copy.

A use-after-free that becomes a named exception with a clear message.

A research-grade interface: NumPy arrays, many worlds in one process, and a
batch step in one call.

An error that always names its cause and carries the Rust context.

### What this costs

Two crates instead of one. Every type that both sides need is declared in the
core crate and wrapped in the binding crate. The wrapper is real work and it
is ongoing.

The scratch-buffer gather for every subset. It is one copy that a chunk-free
column layout cannot avoid, and it is a real cost at one million entities.

Three safety layers for views. Each has a failure mode of its own, and the
scope discipline is visible to the user in every program that reads data.

A packaging pipeline that must be built before the engine, and a wheel matrix
that grows a second row for free-threaded interpreters until a stable ABI
serves both builds.

Per-entity access is available for characters and unavailable for units. Two
rules exist where a user would prefer one, and the difference must be
documented at every method.

### What this forecloses

Python code inside a simulation step, permanently. Any feature that wants a
per-entity Python callback in the frame is out of scope. A custom verb is a
Rust plugin.

Sub-interpreters, for as long as the binding library refuses them.

Archetype-chunked storage for units, as far as this record is concerned. The
whole-column view depends on the flat layout that ADR-0003 chooses.

A view that spans a step. This is a rule that a later record may relax; it is
not a structural bar.

## Notes

The threshold in ADR-0006 D4 is the one number in this record that is likely
to move, because it is derived from an interpreter cost that no one has
measured on the target platform. The rule that a declared tier enforces the
threshold is stable. The number is not. If the measured decision cost differs,
the ceiling changes and the mechanism does not.

Version numbers are deliberately absent from this record. The pin policy, the
stable ABI minimum and the platform matrix are decisions and appear above. The
exact crate versions change every few weeks and belong in the reference
register.[^14] The source report records the values that were current when it
was written, with the date.[^1]

## References

[^1]: Research report 05, the Rust and Python boundary. `docs/research/reports/05-rust-python-boundary.md`
[^2]: ADR-0003, storage: dense tiles and a generational arena. `docs/adrs/draft/`
[^3]: Research report 14, the character graph and inheritance, section 8. `docs/research/reports/14-character-graph-and-inheritance.md`
[^4]: Findings register, entry FND-003. `docs/FINDINGS.md`
[^5]: rust-numpy borrow module documentation. https://docs.rs/numpy/latest/numpy/borrow/index.html
[^6]: Research report 04, the selector engine and verbs, section 1.4. `docs/research/reports/04-selector-engine-and-verbs.md`
[^7]: Python release cycle and end-of-life dates. https://devguide.python.org/versions/
[^8]: PyO3 guide, building and distribution. https://pyo3.rs/v0.29.2/building-and-distribution
[^9]: PEP 803, a stable ABI for free-threaded Python. https://peps.python.org/pep-0803/
[^10]: PEP 779, criteria for supported status for free-threaded Python. https://peps.python.org/pep-0779/
[^11]: PyPA manylinux specification and platform tags. https://github.com/pypa/manylinux
[^12]: PyO3 guide, exceptions. https://pyo3.rs/v0.29.2/exception
[^13]: PyO3 issue 3451, sub-interpreter support. https://github.com/PyO3/pyo3/issues/3451
[^14]: Reference register, cost and dependency tables. `docs/reference/budgets.md`
