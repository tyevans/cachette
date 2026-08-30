# Event Sourcing, CQRS, Aggregates and Determinism at Game-Engine Speed

Research note for ADR-0001. Area: event sourcing without allocation, DDD
aggregate boundaries, command/event separation, deterministic ordering,
floating-point determinism, deterministic RNG, snapshotting, and rollback.

---

## 1. Executive summary

These are the recommendations. Section numbers give the detail.

1. **Do not use classic event sourcing.** Classic event sourcing puts one
   heap-allocated polymorphic event on the heap for each fact. At 100k events
   per frame that costs 10-15% of a 16.6 ms frame in allocation alone, and
   more in cache misses. Use type-segregated append-only arenas instead.
   See §2.

2. **Segregate events by type.** Give each event type its own `Vec<T>` of plain
   old data. Do not use `Box<dyn Event>`. Do not use one enum with a large
   variant. A per-type arena gives a sequential write, no dispatch, and a
   contiguous apply loop. See §2.2.

3. **Use thread-local event buffers with a deterministic concatenation.**
   Each worker writes to its own buffer. At the frame barrier, concatenate the
   buffers by a fixed worker index, never by completion order. See §5.

4. **Keep "aggregate" at region scale, not entity scale.** An entity is a row
   in an array. A row cannot own an invariant. The aggregate is the region or
   the world. See §3.

5. **Be blunt about DDD.** Aggregate roots, repositories, domain objects, and
   per-entity encapsulation are ceremony here. They cost pointer chasing and
   they break the SoA layout. Keep only three DDD ideas: the ubiquitous
   language, the command/event split, and the explicit invariant boundary.
   See §3.3.

6. **Decide the determinism target now.** The context brief lists this as an
   open question. That is a mistake. The *target* can stay open, but the
   *architecture* cannot. See §6 and §12.

7. **Recommended target: bit-exact for one build on one platform, day one.
   Keep a clear path to bit-exact across platforms.** Do not promise
   cross-platform bit-exactness in version 1. To keep the path open, route
   all simulation math through one `sim_math` module day one. See §6.5.

8. **Floating-point sum is not associative.** The context brief says
   aggregation must be a monoid. A float sum is not a monoid. Parallel tree
   reduction over floats gives a different result for a different thread
   count. Use integer or fixed-point accumulators for all pyramid sums, or fix
   the reduction tree shape. This is a real defect in decision 4. See §6.6.

9. **Use counter-based RNG.** Give each random draw a value from a keyed hash
   of `(system_id, frame, entity_id, draw_index)`. Never use a thread-local
   RNG. Never use a shared mutable RNG. See §7.

10. **Snapshot at chunk granularity with copy-on-write.** A full memcpy of
    16.7M tiles costs about 27 ms. That is more than one frame. Snapshot only
    dirty chunks. See §8.

11. **Serialization: use `bytemuck` plus a raw byte copy for snapshots.
    Use `rkyv` or `bitcode` only at the file and network edge.** Your chunks
    are already pointer-free. A serializer adds no value inside the process.
    See §10.

12. **Build these day one, because you cannot retrofit them:** a stable sort
    key on every command and event, a frame sequence counter, the `sim_math`
    boundary, the counter-based RNG, a ban on ambient non-determinism, and a
    determinism test in CI. Everything else is additive. See §12.

---

## 2. Event sourcing without per-event allocation

### 2.1 Why classic event sourcing fails here

Classic event sourcing stores one event object for each fact. In Rust that
usually means `Box<dyn DomainEvent>` in a `Vec`, or an enum with heap fields.
Both fail at this scale. Here is the arithmetic.

Assume 1M active entities at 60 Hz. Assume only 10% of entities emit one event
per frame. That is 100,000 events per frame and 6M events per second.

**Allocation cost.** A `malloc` plus a later `free` costs about 20-50 ns per
pair on a general allocator. At 100,000 events per frame that is 2.0-5.0 ms.
The whole frame budget at 60 Hz is 16.6 ms. So allocation alone takes 12-30%
of the frame. The allocator is also a shared resource, so it becomes a
contention point when 16 threads emit events at the same time.

**Cache cost.** A `Box<dyn DomainEvent>` is a fat pointer of 16 bytes. To read
the event you must follow the pointer. The target is somewhere in the heap.
Each read is a probable cache miss of about 80-100 ns. At 100,000 events that
is 8-10 ms per frame, on top of the allocation cost. The two costs together
exceed the frame budget.

**Dispatch cost.** Each `dyn` call is an indirect branch through a vtable.
The branch predictor cannot predict it when event types are mixed. That blocks
inlining and blocks auto-vectorization of the apply loop.

**The failure is not marginal.** If every entity emits one event per frame,
the load is 1M events per frame and 60M events per second. Classic event
sourcing is then about 100x over budget. There is no tuning that recovers it.

**Memory cost of retention.** 100,000 events per frame at 32 bytes each is
3.2 MB per frame. That is 192 MB per second and 11.5 GB per minute. This
number, not CPU, is the strongest argument for the context brief's decision 12
(the log starts transient). Keep that decision.

### 2.2 Type-segregated append-only arenas

Give each event type its own buffer.

```rust
// One arena per event type. T is Pod: no pointers, no Drop.
pub struct EventArena<T: bytemuck::Pod> {
    data: Vec<T>,
}
```

The cost of one push is an amortized bounds check, a store, and a length
increment. That is 1-2 ns. For 100,000 events that is 0.1-0.2 ms. This is
25-50x cheaper than the classic method, and the apply loop reads a contiguous
array.

The apply step becomes one tight loop for each event type:

```rust
for ev in damage_events.iter() {
    health[ev.target as usize] -= ev.amount;
}
```

This loop has no dispatch, a predictable access pattern, and it can vectorize.

**Preallocate the arenas.** Reserve capacity at startup from a measured high
water mark. Then a frame does zero allocation. Add a hard capacity limit and
report an overflow as a rejected command, not as a panic.

**Clear, do not drop.** Because `T: Pod` has no `Drop`, `Vec::clear` is one
store to the length field. A transient log costs nothing to reset.

### 2.3 Structure of arrays for events

If an event type is wide, split it into parallel arrays.

```rust
pub struct DamageEvents {
    target: Vec<EntityId>,
    amount: Vec<i32>,
    source: Vec<EntityId>,
}
```

Do this only when a consumer reads a subset of the fields. If every consumer
reads all fields, an array of structs is better, because it needs one stream
instead of three. Measure before you split. Most event types are 8-32 bytes
and fit in half a cache line, so an array of structs is usually correct.

### 2.4 Comparison table

| Property | Classic (`Box<dyn Event>`) | Type-segregated arena |
|---|---|---|
| Cost per event | 20-50 ns | 1-2 ns |
| Allocation per frame | 100,000 | 0 |
| Apply loop | indirect call per event | one loop per type |
| Vectorizable | no | yes |
| Thread contention | allocator lock | none |
| Cache behaviour | random | sequential |
| Adding a new event type | no code change | new arena, new apply loop |

The last row is the true trade-off. The arena method costs you open
extensibility. You must name each event type at compile time, or you must
register it in a table at startup. For an engine with about 30 verbs, this is
an acceptable cost. Say so in the ADR.

### 2.5 Practical structure

```
Frame N:
  1. Python phase. Commands queue. GIL held by Python. Simulation is idle.
  2. BARRIER. Seal the command queue. Sort by the stable key.
  3. Validate phase. Each command produces accepted events or a rejection.
  4. Apply phase. Events mutate L0. Dirty bits set.
  5. System phase. Systems run. They emit more events to thread-local arenas.
  6. BARRIER. Concatenate the arenas by worker index. Apply.
  7. Projection phase. The dirty pyramid recomputes L1 and L2.
  8. Export phase. Events go to Python as NumPy arrays. Arenas clear.
```

Steps 3 and 4 must stay separate. A command handler must not mutate state. It
must only read state and emit events. This gives you three things at once:
a pure apply step, safe parallel validation, and a replayable log later.

---

## 3. DDD when an entity is a row in an array

### 3.1 What an aggregate means here

In classic DDD an aggregate is an object graph with one root. The root
enforces the invariants. All writes go through the root. That model assumes
each entity is an object with methods and identity.

Your entities are rows. A row has no methods. A row cannot enforce anything.
So the classic definition does not apply.

The useful definition is different. **An aggregate is the smallest unit of
state that a single transaction can lock and keep consistent.** By that
definition your aggregate is a region, not an entity.

Recommended levels:

| Aggregate | Contents | Invariants it can enforce |
|---|---|---|
| Region (a block of L0 chunks) | tiles, units in those tiles, occupancy index | tile occupancy limits, movement inside the region, local resource caps |
| World | all regions, global counters | faction totals, global unit caps, cross-region movement |

The context brief's decision 9 says "aggregate boundary = parallelism
boundary". That is correct for the region level and it is the right instinct.

### 3.2 Where that rule breaks

**Global invariants do not fit region aggregates.** Consider "a faction may
not exceed 500 units". Two regions can each spawn a unit at the same time.
Each region sees a valid local state. The global total then exceeds the cap.
The region aggregate cannot see this.

You need a second class of invariant. Classify every invariant when you write
it:

- **Region-local.** Check inside the parallel region pass. Cheap.
- **Global scalar.** Check in a serial reduce phase after the parallel pass,
  or reserve from an atomic budget before the parallel pass. The reserve
  method keeps the check parallel, but the *order* of reservation must be
  deterministic, so it must follow the sorted command order, not the thread
  order.
- **Cross-region.** A unit that moves from region A to region B. Handle these
  in a separate serial or two-phase pass after the parallel pass. Do not let
  them run inside the parallel region pass.

This classification is missing from the context brief. Add it. It is cheap to
add now and expensive to add later, because it changes how command handlers
are written.

### 3.3 Where DDD is ceremony — be blunt

Reject these, and say in the ADR why:

- **Aggregate root objects.** A root object means a pointer to a graph. That
  destroys the SoA layout and forces pointer chasing. Cost: everything.
- **Repositories.** A repository hides the storage. Your storage layout *is*
  the design. Hiding it removes the only thing that makes the engine fast.
- **Value objects with encapsulation.** A `Health` newtype that hides an
  `i32` is fine if it compiles away. A `Health` with an invariant check on
  every write is not fine, because it blocks vectorization.
- **Domain services as traits with dynamic dispatch.** Same vtable problem as
  the events. Use static dispatch or a function-pointer table indexed by a
  small integer.
- **One aggregate per entity.** This is the biggest trap. It converts a
  1M-row array pass into 1M transactions.

Keep these three DDD ideas, because they cost nothing and they help a lot:

1. **Ubiquitous language.** Name the verbs and events after the domain. This
   is free and it makes the Python API clear.
2. **Command and event separation.** A command is a request and it can fail.
   An event is a fact and it cannot fail. This split is what makes the apply
   step pure, and a pure apply step is what makes replay possible.
3. **Explicit invariant boundaries.** Write down what each region guarantees.
   This is your parallel-safety proof.

Summary: take the vocabulary and the command/event split. Leave the object
model.

---

## 4. Commands, validation, and partial failure

### 4.1 Shape of a command

The context brief says `Command = (Selector, Verb, Params)`. That is correct.
Add three fields for determinism:

```rust
pub struct QueuedCommand {
    priority: u16,     // caller-set ordering class
    issuer: u16,       // which source queued it
    sequence: u64,     // monotonic counter, assigned on queue
    verb: VerbId,      // u16 index into the verb table
    selector: SelectorHandle,
    params: ParamBlob, // POD bytes, or a second SelectorHandle
}
```

The sort key is `(priority, issuer, sequence)`. That key is total and stable.
It never depends on a thread, a clock, or a pointer address.

**Why `issuer` matters.** The context brief says commands sort by "issue
order". Issue order is well defined only when one thread issues them. If a
Rust system also queues commands, or if a future release allows more than one
Python thread, plain issue order is ambiguous. The `issuer` field removes the
ambiguity now, at a cost of two bytes.

**Recommendation: use two queues.** One for external commands from Python. One
for internal commands that systems generate. Apply the external queue first,
then the internal queue. Do not interleave them. If a system-generated command
can itself generate a command, cap the depth and report the cap as a
rejection. An uncapped cascade is a source of frame-time spikes.

### 4.2 Validation

Validation runs on the whole selected set, in parallel over regions, before
any mutation. Each command handler follows this shape:

```
fn validate(world: &World, sel: &ResolvedSet, params: &P)
    -> (AcceptedMask, RejectionCounts)
```

The handler reads. It never writes. This is what makes parallel validation
safe without locks, and it is what makes the apply step replayable.

### 4.3 Reporting partial failure

The context brief's decision 10 is correct. Here is a concrete form.

```rust
pub struct CommandResult {
    affected: u32,
    rejected: u32,
    reason_counts: [u32; MAX_REASONS], // reason code -> count
    rejected_set: SelectorHandle,      // lazy, for chaining
}
```

Points to settle in the ADR:

- **Reason codes must be a closed `u16` enum.** A string reason means an
  allocation for each rejection. Do not do that. Map the code to a message in
  Python.
- **The rejected set should be a bitset over the selected set**, not a list of
  entity IDs. A bitset for 1M entities is 128 KB. A list can be larger and it
  needs an allocation.
- **The rejected set must stay valid for exactly one frame.** Entities can
  die. Give the handle a frame stamp and reject a stale handle with a clear
  error. Do not let a stale handle read freed rows.
- **A command must be all-or-nothing per entity, never per field.** If a
  handler partly applies to one entity and then fails, the state is not
  replayable. Validate fully, then apply fully.

---

## 5. Deterministic ordering

### 5.1 The rules

Four rules give you determinism in a parallel engine. Follow all four.

1. **Seal the input at a barrier.** No command enters the frame after the
   seal. This makes the frame a pure function of (state, sealed commands).
2. **Sort by a stable total key.** Never rely on the order that a hash map
   iterates, or on the order that a work-stealing scheduler completes tasks.
3. **Partition the work by data, not by time.** Assign region R to worker
   index `R % num_workers`, or use a fixed static schedule. Do not let the
   result depend on how many workers exist.
4. **Merge by a fixed index.** When you concatenate thread-local buffers,
   loop over worker indices 0..N in order. Never append as workers finish.

### 5.2 Why thread completion order is fatal

Rayon uses work stealing. The order that tasks complete depends on cache
state, on other processes on the machine, on core count, and on timing. It is
not reproducible even on the same machine in the same process. If any output
depends on that order, your simulation is not reproducible and no test will
catch it every time. It will fail once in a thousand runs, which is the worst
failure mode.

### 5.3 The parallel reduction trap

Rayon's `par_iter().sum()` splits the range based on the *runtime* work-steal
pattern. For integers this is safe, because integer addition is associative.
For floats it is **not** safe. A different split gives a different rounding
and a different result.

Two safe options:

- **Fixed tree shape.** Reduce over a fixed chunk size, for example 1024
  elements per chunk, then combine chunk results in index order. The result
  is then independent of the thread count.
- **Integer accumulator.** Sum in `i64` fixed-point. This is exactly
  associative and it is faster.

Prefer the integer accumulator for the L1/L2 pyramid. See §6.6.

### 5.4 Other sources of non-determinism to ban day one

Ban all of these in a lint or a code review rule:

- `std::time::Instant` or `SystemTime` inside the simulation.
- `HashMap` and `HashSet` iteration order. `std::collections::HashMap` uses a
  random seed for each process. Use `BTreeMap`, or a `HashMap` with a fixed
  hasher, or an index-sorted `Vec`.
- Pointer or address values used as sort keys or hash inputs.
- Any thread ID or worker count in a computed result.
- Uninitialized memory read as data.
- `f32`/`f64` NaN bit patterns. Rust RFC 3514 says NaN payloads and the NaN
  sign bit are not deterministic. Never store a NaN and never compare bits.
- Iteration over a set built from a parallel `collect` into an unordered
  container.

---

## 6. Floating-point determinism in depth

### 6.1 What IEEE-754 does guarantee

IEEE-754 fully specifies the result of `+`, `-`, `*`, `/`, `%`, `sqrt`,
comparisons, and conversions. For the same inputs, the same rounding mode, and
the same operation, every conformant machine gives the same bits. This part is
solid.

Rust makes this a language guarantee. RFC 3514 (Float Semantics) states that
primitive float operations produce results that exactly match IEEE 754-2008,
with round-ties-to-even, no traps, and no flush-to-zero. The RFC is accepted
and rustc already matches it (tracking issue rust-lang/rust#128288).

### 6.2 What breaks it

The guarantee covers single operations. It does not cover what a compiler does
to a *sequence* of operations, and it does not cover the standard library's
transcendental functions.

- **FMA contraction.** `a * b + c` can compile to one fused instruction. The
  fused form does not round the product. The result differs in the last bit.
  A compiler may contract on one target and not another. Rust does **not**
  contract by default, and RFC 3514 forbids it. This is a real advantage over
  C and C++, where the default is often contraction.
- **x87 excess precision.** 32-bit x86 without SSE computes in 80-bit
  registers and rounds late. Results differ from SSE. Modern Rust targets
  x86-64 with SSE2, so this is mostly historic. Do not ship a 32-bit x86
  build, and the problem disappears.
- **Fast math.** `-ffast-math` allows reassociation, and reassociation breaks
  determinism. Rust has **no** stable fast-math flag. The unstable
  `core::intrinsics::fadd_fast` family exists but is unstable and unsafe. Do
  not use it. This absence is a feature for you.
- **Transcendental functions.** `sin`, `cos`, `exp`, `ln`, `pow`, `atan2` are
  the biggest risk. Rust's `f32::sin` calls the platform libm. glibc, musl,
  macOS libSystem, and MSVC each give different last bits. A glibc upgrade can
  change the result on the same machine. IEEE-754 does not require correct
  rounding for these, so nothing is being violated. This alone stops
  cross-platform bit-exactness.
- **SIMD reassociation.** Manual SIMD changes the grouping of a sum. Four
  lanes summed in pairs differ from a linear sum. Auto-vectorization of a float
  reduction requires reassociation, so LLVM will not do it without fast-math.
  Your own SIMD will, if you write it. Fix the lane count and the reduction
  shape if you hand-write SIMD.
- **Denormals and flush-to-zero.** Some environments set the FTZ and DAZ bits
  in MXCSR. Then very small values become zero. A third-party library, an
  audio driver, or a plugin can set these bits for the whole thread and the
  setting persists. This is a genuine hazard in a Python extension, because
  you do not control what else is loaded in the process. Consider reading
  MXCSR at the start of each step and rejecting or restoring an unexpected
  value.
- **libm version drift.** Even one platform is not stable across time if you
  call the system libm. Pin your own implementation.

### 6.3 The Rust picture, in short

| Risk | Status in Rust |
|---|---|
| Fast math by default | Absent. Good. |
| FMA contraction by default | Absent. RFC 3514 forbids it. Good. |
| `f32::mul_add` | Explicit and deterministic. It is one fused operation with one rounding, defined by IEEE-754. Use it when you want fusion. It may be slow on a target without an FMA unit, because it then calls a software routine. |
| `strict_fp` / rounding-mode control | Not available on stable. There is no stable way to change the rounding mode. This is fine, because the default mode is the one you want. |
| Transcendentals | Platform libm. Not portable. This is the main gap. |
| NaN payloads | Explicitly non-deterministic per RFC 3514. |

### 6.4 The three options

**Option A: floats, one build, one platform.**
You get bit-exact replay for a given binary on a given machine class. You do
not get bit-exact results between Linux and macOS. Cost: none. This is the
default behaviour today.

**Option B: floats plus a pinned libm.**
Route every transcendental through the `libm` crate (a pure Rust port of musl
libm, version 0.2.16, very widely used). Ban `f32::sin` and friends with a
clippy lint. Combine this with the existing no-contraction and no-fast-math
guarantees. You then get bit-exact results across x86-64 and aarch64 for the
same source, because every step is IEEE-specified or is your own code. Cost:
`libm` is slower than a tuned platform libm, perhaps 1.5-3x on the affected
calls. Risk: a `libm` version bump can change results, so pin the version and
treat a bump as a save-format break.

**Option C: fixed-point integers.**
Represent positions and other quantities as `i32` or `i64` with a fixed number
of fractional bits, for example Q16.16 or Q32.32. Integer arithmetic is exactly
defined everywhere. Addition is associative, so parallel reduction is safe.
The `fixed` crate (1.31.0, mature) gives the types.

Costs of Option C, stated honestly:
- Every division needs a shift and care about overflow.
- `sqrt`, `sin`, and `atan2` need your own table or polynomial. That is real
  work, but it is a fixed one-time cost and about 300 lines.
- Range and precision must be designed. Q16.16 gives a range of ±32768 with a
  step of 1/65536. For a 4096x4096 grid that is fine for tile-space positions.
- Multiplication needs a widening intermediate (`i64` for Q16.16), so it is
  slower than a float multiply on a modern core.
- Anything the user sees as a float must be converted at the boundary.

**Option D: software float.** Berkeley SoftFloat via `softfloat-sys`. Reject
this. It is 10-50x slower and the crate has almost no adoption (11.5k
downloads total, last updated 2024). It is only for reference testing.

### 6.5 Recommendation

**Ship Option A. Architect for Option B. Reserve Option C for the parts that
need it.**

Concretely, day one:

1. Create one module, `sim_math`. Every simulation float operation goes
   through it. Add a clippy deny rule for direct use of `f32::sin`, `f32::cos`,
   `f32::exp`, `f32::ln`, `f32::powf`, `f32::atan2` outside that module. The
   module is thin today and calls `std`. Swapping it to `libm` later is a
   one-file change. Retrofitting the module after 200 call sites exist is not.
2. Use **integers or fixed-point for anything that accumulates or aggregates**:
   pyramid sums, resource totals, and health. See §6.6.
3. Use floats for anything transient and non-accumulating: rendering, a
   per-frame flow-field value that is recomputed and never stored.
4. Store no float in a snapshot if an integer will do. A float in a snapshot
   turns a rounding difference into a permanent state divergence.
5. Add a CI test that runs 10,000 frames twice, with different thread counts,
   and compares a hash of the world state. Run it on every commit. Add a
   second test that compares against a checked-in golden hash, so a change in
   behaviour is visible in the diff.

State the contract in the documentation exactly: *"Identical results for the
same binary, the same input, and any thread count. Results may differ between
platforms and between versions."* This is what researchers actually need for
reproducible experiments, and it is honest.

### 6.6 The monoid problem — a defect in the context brief

Decision 4 in the brief says an attribute may appear at L1/L2 only if it is an
associative combine with an identity. That rule is correct in principle. But
**float addition is not associative**. So a float `sum` is not a monoid, and
the rule as written does not hold for float attributes.

The effect is concrete. The dirty pyramid recomputes only some cells. The
order of combination therefore changes between frames and between thread
counts. A float sum then drifts. Over many frames the drift grows and L1
disagrees with a full recomputation of L0. That is a silent correctness bug in
a read model, and it is very hard to debug.

Fix: **require an exact monoid.** Allow only these accumulator types in the
pyramid:

- Integer sum and count (exact and associative).
- Fixed-point sum in `i64` (exact and associative).
- `min`, `max`, bitwise OR, bitwise AND (exact and associative for both
  integers and floats).
- Histogram bins (integer counts).

Ban a raw float sum in the pyramid. If an attribute is conceptually a float,
store it as fixed-point in the pyramid and convert on read. Amend decision 4
to say "exactly associative", not just "associative".

---

## 7. Deterministic random numbers

The context brief does not mention RNG anywhere. That is a gap. Fix it before
any system uses randomness, because a retrofit means changing every call site
and it invalidates every saved replay.

### 7.1 Why the usual approaches fail

- **A thread-local RNG** (`rand::rng()`, or `thread_rng` in older versions)
  is seeded from the OS and its state advances in the order that work lands on
  a thread. Both facts destroy determinism. Ban it in the simulation.
- **A single shared seeded RNG** is deterministic only if draws happen in a
  fixed order. In a parallel pass they do not. It is also a contention point.
- **One RNG per thread, seeded from the frame** is deterministic only if the
  work split is fixed. It breaks when the thread count changes. That is a
  trap, because it passes a same-machine test and fails on a user's machine.
- **One RNG per entity, stored as state** works, but it costs 8-32 bytes per
  entity, so 8-32 MB at 1M entities, and it must go into every snapshot.

### 7.2 Counter-based RNG is the correct answer

A counter-based RNG has no state to advance. It is a keyed hash of a counter.
You compute the value directly:

```
value = hash(key = system_id, counter = (frame, entity_id, draw_index))
```

Properties that matter here:

- **Order independent.** Entity 500 gets the same value whether it is
  processed first or last, and whatever the thread count.
- **Stateless.** Nothing goes in the snapshot except the frame number.
- **Parallel safe.** No sharing, no atomics, no contention.
- **Random access.** You can compute the draw for one entity at one frame
  without replaying anything. This is exactly what rollback needs.

Philox and Threefry are the standard designs (Salmon et al., "Parallel Random
Numbers: As Easy as 1, 2, 3", SC11). NumPy ships Philox as a bit generator,
which is a useful point of comparison for your research users.

### 7.3 Crate choice, with a caution

- `rand_philox` 0.1.0 gives Philox4x32-10 with a stated reproducibility
  contract and known-answer tests. **But it has about 286 downloads and one
  release.** It is not mature. Treat it as a reference implementation to read,
  not as a dependency to trust.
- **Recommendation: write your own.** A counter-based mixer is about 40 lines.
  `splitmix64` is the simplest good choice, and `rand_xoshiro` 0.8.1 contains
  a tested `SplitMix64`. A `splitmix64` of a packed `(system, frame, entity,
  draw)` key gives good statistical quality for game use. Pin it with
  known-answer tests so a refactor cannot silently change it.
- If you later need higher statistical quality, for example for a Monte Carlo
  research user, add Philox behind the same interface.
- `rand` 0.10.2 is the ecosystem standard and its book has a page on
  reproducibility. Use its distribution code if you like, but be careful:
  distribution algorithms are not guaranteed stable across `rand` versions,
  and a float distribution uses transcendental functions. Implement your own
  uniform-integer and uniform-float mapping so a dependency bump cannot change
  your simulation.

### 7.4 Rule to write down

*Every random draw in the simulation must be a pure function of the frame
number, a compile-time system identifier, an entity or tile identifier, and a
draw index. No other source of randomness is allowed.*

---

## 8. Snapshotting

### 8.1 Why memcpy snapshots work

Your chunks contain no pointers. So a snapshot is a byte copy, and a restore
is a byte copy back. No serializer, no traversal, no allocation for each
entity. This is the single biggest benefit of the chosen memory layout, and it
should be stated in the ADR as a reason for the layout, not only as a
consequence.

### 8.2 The cost, measured

16.7M tiles at 16 bytes each is 268 MB. A memcpy runs at about 10 GB/s on one
core, so a full copy costs about 27 ms. That is more than one frame at 60 Hz.
A naive full snapshot each frame is not possible.

Even at 8 bytes per tile the copy is about 13 ms. Still too slow. So plan for
partial snapshots from the start.

### 8.3 Strategies, in order of preference

**1. Dirty-chunk copy-on-write.** You already maintain a dirty bitset for the
pyramid. Reuse it. Copy only the chunks that changed since the last snapshot.
In a typical frame far fewer than 1% of tiles change, so the copy is well under
1 ms. This is the right default.

**2. Ring buffer of recent snapshots.** Keep the last N frames as a base
snapshot plus N chunk deltas. This gives cheap rollback to any of the last N
frames, which is what GGPO needs.

**3. Full snapshot on a slow cadence.** Take a full snapshot every few hundred
frames, on a background thread, from a copy-on-write base. Use it as the
restore point for a long replay.

**4. Snapshot plus log.** Store one full snapshot and then all events since.
To restore, load the snapshot and replay the events. This is cheap in storage
and expensive in time. It is the correct format on disk. It is the wrong
format for per-frame rollback.

### 8.4 Compaction, if you retain the log

At 3.2 MB per frame you must compact. Options:

- **Drop by age.** Keep the last N frames only. Simplest and it fits the
  rollback use.
- **Snapshot and truncate.** Take a full snapshot, then delete every event
  before it. This is the standard event-sourcing compaction.
- **Semantic collapse.** Replace 100 `Move` events for one entity with one
  `Move` to the final position. This only works for events that are
  idempotent-in-the-last-write. It breaks any consumer that wants the path.
  Use it only for an export format, never for the authoritative log.
- **Column compression.** Because events are already in SoA arrays, a
  delta-plus-varint encoding on each column compresses very well. Entity IDs
  in a frame are often near-sorted, so a delta is small. Expect 3-10x.

### 8.5 What snapshot plus log buys

- **Replay.** Load the snapshot and replay. This is how you reproduce a bug
  report without shipping 268 MB.
- **Rollback.** Restore a snapshot and re-simulate with corrected input.
- **Time travel debugging.** Step backward. This alone will save many days on
  the two-language debugging risk named in the brief.
- **Audit.** Answer "why did this unit die" from the log.

None of these need to exist in version 1. All of them need the apply step to
stay pure and the events to stay POD. Keep that discipline day one.

---

## 9. Rollback netcode as evidence

Netcode is deferred, and that is correct. But the rollback literature is the
best available evidence about what determinism costs and buys, so use it.

GGPO-style rollback works like this. Each client predicts remote input,
simulates ahead, and re-simulates when real input arrives. It needs three
things:

1. A fixed tick rate.
2. A fully deterministic step function.
3. Fast save and restore of state.

The reported experience across the field is consistent:

- **Determinism is the hard part, not the networking.** The networking is a
  few hundred lines. Making a simulation reproducible after the fact is a
  months-long effort.
- **Floats are the usual cause of a desync.** Transcendentals and reordering
  are named repeatedly. Fixed-point or tightly controlled float is the standard
  fix.
- **Save and restore speed sets the rollback window.** If a save costs 27 ms
  you cannot roll back at all. This is the same constraint as §8.2, so your
  chunk-level copy-on-write design serves both uses.

The lesson for this ADR: the three requirements above are exactly the three
things §12 says to build day one. You get rollback almost free later if you
build them now, and you get it never if you do not.

---

## 10. Serialization crates

You need serialization at two places only: the disk save file, and a future
network boundary. Inside the process, a byte copy is enough.

Current state, checked on crates.io:

| Crate | Version | Notes |
|---|---|---|
| `bytemuck` | 1.25.2 | Safe casting between POD types and byte slices. Very mature. **This is your main tool.** |
| `zerocopy` | 0.8.56 | Same problem, stricter and more capable. Also mature. Either is fine. |
| `rkyv` | 0.8.18 | Zero-copy framework. Fast, and it handles pointers and collections. Overkill if your data is already POD. Note that 0.7 to 0.8 was a large break, so it still moves. |
| `bitcode` | 0.6.9 | Smallest output and the top of most benchmark rows. Not self-describing. Fewer users. |
| `postcard` | 1.1.3 | `no_std`, stable format, serde-based, well specified. Good for a network wire format. |
| `bincode` | 3.0.0 | Very widely used. Note that 3.0 is a recent major version, so check the format stability policy before you depend on the byte layout. |

Recommendations:

- **Snapshots on disk: raw chunk bytes plus a small header, via `bytemuck`.**
  A serializer buys you nothing over a `memcpy` of a POD array, and it costs a
  pass over the data. Add a format version, an endianness marker, and a
  checksum in the header. Reject a mismatch clearly.
- **Metadata and the header: `postcard`.** Small, stable, and it handles
  version fields well.
- **Do not depend on any serializer's byte format for the authoritative save
  format.** Define your own layout and write the reader by hand. Then a
  dependency bump can never break a user's save file. This is 200 lines and it
  removes a whole class of future pain.
- Use the `djkoloski/rust_serialization_benchmark` repository if you want
  current numbers, but treat all such benchmarks with care. Several are
  disputed, and none use your data shape.

Determinism aids:

- `libm` 0.2.16 — pure Rust libm. The key crate for Option B in §6.4.
- `fixed` 1.31.0 — fixed-point types for Option C.
- `rand_xoshiro` 0.8.1 — contains a tested `SplitMix64`.
- Avoid `simba` and `nalgebra` inside the deterministic core unless you audit
  them. They are general math libraries and they do not promise bit-exact
  behaviour.

---

## 11. Trade-offs and failure modes

| Decision | Failure mode if wrong | Detection |
|---|---|---|
| Type-segregated arenas | You must name event types at compile time. A plugin cannot add one. | Design review now. |
| Arena capacity | Overflow at a load spike. | Track the high-water mark. Report the overflow as a rejection. |
| Region aggregate | A cross-region operation deadlocks or gives an inconsistent result. | Write the two-phase pass now. |
| Sort key | A missing `issuer` field makes multi-source ordering ambiguous. | Add the field now. It is two bytes. |
| Float in the pyramid | Silent drift between L0 and L1 over hours. | A periodic full recomputation compared against the incremental value. Add this test. |
| Thread-local RNG | Non-reproducible results, seen only sometimes. | The varying-thread-count CI test in §6.5. |
| Full snapshot | 27 ms frame spike. | Measure it before you write the code. |
| Stale rejected-set handle | Reads a freed row. | Frame-stamp the handle. |
| FTZ set by another library in the process | Denormals become zero, results change. | Read MXCSR at the start of each step. |

---

## 12. What you must build day one

**Impossible to retrofit. Build now.**

1. **A stable total sort key on every command and event**:
   `(priority, issuer, sequence)`. Adding an ordering field later invalidates
   every replay and forces a change at every call site.
2. **A frame sequence counter** in the world state, and every event stamped
   with it. This is the key for the RNG and for the log.
3. **The `sim_math` boundary**, with a lint that bans direct transcendental
   calls in simulation code. It costs one file now. It costs a full audit
   later.
4. **The counter-based RNG interface**, with the `(system, frame, entity,
   draw)` key. Every randomness call site must use it from the first commit.
5. **A ban on ambient non-determinism**: no clock, no unordered hash map
   iteration, no addresses, no thread count in a result. Enforce in review and
   in a lint where possible.
6. **`#[derive(Pod)]` on every event and every component**, checked at compile
   time. This keeps the memcpy snapshot possible.
7. **The determinism test in CI**, from the first week. A determinism bug found
   at month six is very expensive. Found at week one it is trivial.
8. **The command-handler shape: validate reads, apply writes.** If handlers
   mutate during validation, the apply step is not pure, and neither replay
   nor rollback is possible. This is a shape you cannot change later without
   rewriting all 30 verbs.
9. **Exact-monoid accumulators in the pyramid** (§6.6).

**Additive later. Do not build now.**

- Retained event log and compaction.
- Rollback and time travel.
- Cross-platform bit-exactness (needs only the `sim_math` swap plus a
  fixed-point pass over the parts that need it).
- Delta and compressed snapshots.
- Netcode.
- A plugin event type registry.

The pattern is clear. Build the *shapes* now and the *features* later. A shape
is a sort key, a purity rule, a module boundary, or a type bound. Each is
cheap now and impossible later.

---

## 13. Where this disagrees with the context brief

1. **Decision 4 is not quite right.** "Associative" must be "exactly
   associative". Float addition is not. Ban float sums in the pyramid. See
   §6.6. This is the strongest disagreement in this note.

2. **Open question 3 should not stay open.** The *target* may stay open. The
   *architecture* must be fixed now. Choose "bit-exact within a build" as the
   shipped contract, and put the `sim_math` boundary in day one so the
   cross-platform target stays reachable.

3. **Decision 9 needs more detail.** "Issue order" is not a total order when
   more than one source queues commands. Add an `issuer` field. Add a separate
   internal command queue with a bounded cascade depth.

4. **Decision 9 needs an invariant classification.** "Aggregate boundary =
   parallelism boundary" holds for region-local invariants only. Add the
   global-scalar and cross-region classes. See §3.2.

5. **RNG is missing from the brief entirely.** Add it as a decision, not as an
   implementation detail. It is a day-one item.

6. **The brief does not state the snapshot cost.** A full snapshot is about
   27 ms. Record this, because it forces chunk-level copy-on-write, and that
   in turn is the same mechanism a future rollback needs.

7. **The brief does not say that selector resolution must be ordered.**
   Decision 6 describes a hierarchical descent. State that the descent visits
   children in a fixed index order, so the resolved set is always in the same
   order. If a verb's effect depends on the order within the set, an unordered
   set is a determinism bug.

---

## 14. Open questions for the ADR author

1. Do research users need bit-exact results across platforms? A reinforcement
   learning user usually needs reproducibility on one machine, not across
   machines. If so, Option A plus the `sim_math` boundary is enough, and you
   save months.
2. What is the target rollback window, if any? This sets the snapshot ring
   size and the memory budget.
3. Can a verb read another entity's state during validation? If yes, the
   validation pass needs a consistent read view, and that is a much harder
   design than a purely local check.
4. Is a command allowed to affect entities in more than one region? If yes,
   the two-phase cross-region pass is required in version 1, not later.
5. What is the event budget per frame? This sets the arena preallocation and
   the overflow policy.
6. Do you need a `sqrt` in the simulation? `sqrt` is IEEE-exact, so it is
   free. `atan2` and `sin` are not. Knowing which functions you need decides
   how much of §6.4 Option C you must write.
7. Should the engine detect and reject a changed MXCSR (FTZ/DAZ) set by
   another library in the Python process? A warning is cheap. Silence is a
   very confusing bug.

---

## 15. Sources

- [RFC 3514: Float Semantics — The Rust RFC Book](https://rust-lang.github.io/rfcs/3514-float-semantics.html)
- [Determinism for floating point operations in Rust — Rust Users Forum](https://users.rust-lang.org/t/determinism-for-floating-point-operations-in-rust/4426)
- [Deterministic Physics in C++: Fixed-Point Math and Reproducible Simulation](https://cppcat.com/deterministic-physics-engine/)
- [/fp — Specify floating-point behavior (MSVC)](https://learn.microsoft.com/en-us/cpp/build/reference/fp-specify-floating-point-behavior)
- [fp_contract pragma (MSVC)](https://learn.microsoft.com/en-us/cpp/preprocessor/fp-contract)
- [Reproducibility — The Rust Rand Book](https://rust-random.github.io/book/crate-reprod.html)
- [libm — docs.rs](https://docs.rs/libm)
- [Salmon et al., "Parallel Random Numbers: As Easy as 1, 2, 3" (SC11)](https://www.thesalmons.org/john/random123/papers/random123sc11.pdf)
- [Philox counter-based RNG — NumPy manual](https://numpy.org/doc/stable/reference/random/bit_generators/philox.html)
- [Counter-based pseudorandom number generators for CORSIKA 8](https://www.epj-conferences.org/articles/epjconf/pdf/2021/05/epjconf_chep2021_03039.pdf)
- [Randompack: Cross-Platform Reproducible Random Number Generation](https://arxiv.org/pdf/2605.05099)
- [Preparing your game for deterministic netcode — yal.cc](https://yal.cc/preparing-your-game-for-deterministic-netcode/)
- [Netcode Architectures Part 2: Rollback — SnapNet](https://www.snapnet.dev/blog/netcode-architectures-part-2-rollback/)
- [Determinism, Prediction and Rollback — coherence docs](https://docs.coherence.io/manual/advanced-topics/competitive-games/determinism-prediction-rollback)
- [Making a GGPO-style rollback networking multiplayer game](https://outof.pizza/posts/rollback/)
- [rust_serialization_benchmark — djkoloski](https://github.com/djkoloski/rust_serialization_benchmark)
- [rkyv is faster than {bincode, capnp, cbor, flatbuffers, postcard, prost, serde_json}](https://david.kolo.ski/blog/rkyv-is-faster-than/)
- [Determinism — Rapier physics engine](https://rapier.rs/docs/user_guides/javascript/determinism/)

Crate versions checked on crates.io on 2026-08-30: `rkyv` 0.8.18,
`bitcode` 0.6.9, `postcard` 1.1.3, `bincode` 3.0.0, `rand` 0.10.2,
`rand_xoshiro` 0.8.1, `rand_philox` 0.1.0, `libm` 0.2.16, `fixed` 1.31.0,
`bytemuck` 1.25.2, `zerocopy` 0.8.56, `rayon` 1.12.0.
