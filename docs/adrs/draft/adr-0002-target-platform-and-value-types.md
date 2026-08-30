# ADR-0002: Target platform and value types

**Status:** Draft
**Date:** 2026-08-30
**Depends on:** ADR-0001, determinism as the primary constraint.[^1]

## Context

Cachette is a headless simulation engine. A Rust core runs the simulation. A
Python control plane issues commands. The engine runs on servers that the
project operates. It does not run on player hardware.

The root record makes determinism the primary constraint. It promises
bit-exact results for one binary on one architecture, at any thread count. It
does not promise cross-platform bit-exactness.[^1] That promise costs nothing
only if the project names one deployment architecture and holds to it.

This record names that architecture. It also fixes the value types that every
later record uses. Both must settle before storage, before the pyramid, and
before the event log, because each of those records spends the vocabulary
defined here.

An architecture is not a neutral choice. It gives some properties free and
charges for others. This record states what the target gives, what it demands,
and which rules follow.

## Decision

### D1 — The primary target is `aarch64-unknown-linux-gnu`

The engine targets 64-bit ARM cores on Linux. The deployment fleet is AWS
Graviton.[^2]

x86-64 and Apple Silicon are development targets. The project compiles and
tests on them. It does not tune for them, and it does not treat a measurement
taken on them as evidence about production.

One architecture makes the determinism promise of the root record a
description of the deployment rather than a compromise.[^1]

### D2 — NEON is a mandatory baseline, so there is no runtime dispatch

The 128-bit NEON vector instruction set is part of the base 64-bit ARM
instruction set. Every target core has it.

The engine therefore contains no runtime feature detection, no function
multiversioning, and no `target_feature` dispatch. There is one code path.

One code path is also the deterministic path. Runtime dispatch selects an
implementation from a property of the machine. Two machines that select
differently can produce different results. This record removes that risk by
construction rather than by discipline.

### D3 — Wide integers are free, and the x87 hazard does not exist

The target runs 64-bit integer arithmetic at the same rate as 32-bit
arithmetic. An `i64` accumulator costs nothing against an `i32` accumulator.

This is the reason the root record rejects a second fixed-point scale. The
only argument for the second scale was to keep a multiply inside 32 bits, and
the target removes that argument.[^1]

The target has no x87 unit. The x87 unit computes at extended precision and
then rounds, so an x86 result can depend on when a value spills to memory.
That class of float nondeterminism does not exist here.

### D4 — Prefer wide parallel stages over clever serial code

The target has no simultaneous multithreading. A 64-vCPU instance is 64 real
cores. Single-core performance is lower than a high-clock x86 core.

Structure the frame as a small number of very wide stages. Do not structure it
as many narrow stages that depend on single-core speed.

### D5 — The memory model is weak, so disjoint outputs are required

The target has a weak memory model. It is not total store order. A relaxed
atomic or an acquire atomic emits a real barrier where an x86 core emits
almost nothing.

The root record prefers disjoint outputs over atomics.[^1] On this target that
preference is a requirement. Two rules follow.

- Write a parallel reduction to indexed output slots. Do not scatter atomics
  across a shared array unless a measurement on the target justifies it.
- Treat false sharing as expensive. Recovery from a shared line costs coherence
  traffic that a strong memory model would absorb.

A dirty-bitset `fetch_or` remains legitimate, because bitwise OR is exactly
commutative.[^1] It is a permitted exception, not a pattern to copy.

### D6 — Count bits in blocks, never one word at a time

The target has no scalar population count instruction. A count of set bits in
one 64-bit word moves the word to a vector register, counts each byte, and
reduces the result. That is three or four instructions where an x86 core uses
one.

The design is bitplane-heavy, so this matters. Write every aggregation kernel
to count a whole block of bits in one pass. Never write a loop that counts one
word at each step.

Batching pays the vector setup once for each block instead of once for each
word. It is the case where the vector unit wins.

### D7 — Set `target-cpu`, but not for atomics

Set the `target-cpu` compiler flag to the Neoverse core of the deployment
generation. It is worth setting for instruction selection and for scheduling.

It is **not** needed to obtain single-instruction atomics. An earlier draft
said that Large System Extensions must be enabled explicitly, and that atomics
otherwise compile to load-linked retry loops. That claim is refuted. The
outline-atomics mechanism has been the default on this target since about Rust
1.57, so a suitable core gets the single-instruction form at run time without
a flag.[^3]

Record the exact core name for each fleet generation in the register, not
here. The name changes when the fleet changes.[^4]

### D8 — The cache-line size is a compile-time constant

Define one project-wide constant for the cache-line size. Derive every
alignment, every padding decision, and every block-size argument from it.

The value is 64 bytes on the target. Apple Silicon uses 128 bytes. A
development machine therefore reports false-sharing behaviour and alignment
behaviour that the deployment fleet does not have.

Two rules follow.

- Never take a performance conclusion about false sharing or alignment from a
  development machine. Measure on the target.
- Never write the number in a kernel. Write the constant.

No measurement on the target exists yet, so every cost figure in the project
is derived rather than measured.[^5]

### D9 — Every value type is a newtype

A raw integer carries no meaning, and two raw integers of the same width
substitute for each other silently. A tile index passed where a chunk index
belongs is a defect that the compiler must reject.

A newtype is zero cost at run time. Use one for every value below.

| Type | Representation | Purpose |
|---|---|---|
| `TileIdx` | `u32` | Index of a tile in the world grid |
| `ChunkIdx` | `u32` | Index of a storage chunk, derived from `TileIdx` by a shift |
| `Entity` | `NonZeroU64` | Index plus generation, so `Option<Entity>` stays 8 bytes |
| `UnitTypeId` | `u16` | Index into the shared unit stat table |
| `TerrainId` | `u8` | Index into the terrain cost matrix |
| `FactionId` | `u16` | Index of a faction |
| `UpgradeSetId` | `u32` | Interned identifier of a set of upgrades |
| `CapabilityMask` | `u64` | Capability bits, derived from `UpgradeSetId` |
| `Fix32` | `i32`, Q16.16 | Positions and per-entity scalars |
| `Fix64` | `i64`, Q32.32 | Values whose range exceeds `Fix32` |
| `Accum` | `i64` | Every summary accumulator |
| `Tick` | `u64` | Simulation time |

Four rules govern the table.

**`TileIdx` is 32 bits and the world fits inside it.** The world extent is not
yet decided, so the record states the width and not the count.[^5] A 32-bit
index holds any extent this project will choose.

**`Entity` is never a bare index.** The generation field is what makes a stale
reference detectable rather than silently wrong.

**`CapabilityMask` is derived, never authoritative.** The authoritative value
is `UpgradeSetId`. The mask is a cache for hot loops and for selector
predicates. Rebuild it from the identifier; never edit it in place.

**`Fix32` and `Fix64` share one fractional convention.** The root record fixes
the project-wide scale at Q16.16 and rejects a second scale.[^1] A proposal for
a modifier scale of one part in 1024 is closed by that decision.

### D10 — Two invariants hold on every type that enters an event

**Plain old data.** Every event type and every type inside an event is
`bytemuck::Pod`. That means `repr(C)`, declared padding, and no `bool`. Use
`u8` where a boolean is wanted. The root record derives this rule from state
hashing: undeclared padding holds uninitialised bytes, which enter the hash and
produce nondeterminism with no cause in the simulation.[^1]

**Accumulator width.** A summary accumulator is always wider than the field it
summarises, and the widening happens at the first summary level. The root
record derives the bound: a one-byte field summed over 2^24 tiles reaches 2^32
exactly, which a `u32` cannot hold.[^1] The `Accum` type in D9 exists so that
this rule has one name.

Neither rule is re-derived here. Both are cited because a later record must be
able to point at one place.

### D11 — Size and align migrating structures to 8 or 16 bytes

The target loads and stores register pairs in one instruction. A structure
sized and aligned to 8 bytes, or to 16 bytes, moves in a single instruction.

Apply this to any structure that the engine copies between storage locations.
Component data that migrates between chunks is the main case.

This rule is about structure size. It is separate from D8, which is about
cache-line alignment of whole arrays. Do not merge them.

### D12 — Use the `wide` crate for explicit vectors, and stay on stable Rust

The portable vector interface in the standard library is still available only
on a nightly compiler. It has no announced stabilisation date.[^6] The project
does not depend on a nightly compiler.

Where a kernel needs explicit vectors, use the `wide` crate. It compiles to
native vector instructions on the target and it works on a stable
compiler.[^7]

Do not use the Scalable Vector Extension. Vector-length-agnostic code costs
real complexity, and this workload gains little over the baseline vector unit.

Most kernels should need no explicit vectors at all. Write the scalar loop so
that the compiler vectorises it, then check the output.

## Consequences

### What this buys

One code path. No runtime dispatch, so no machine property can select a
different implementation and produce a different result.

Free wide accumulators. The accumulator-width rule of the root record costs
nothing on this target, so it is a rule with no ongoing price.

One class of float nondeterminism removed, because the target has no extended
precision unit.

A deployment fleet that matches the determinism promise exactly. The promise
is a description of production rather than a restriction on it.

Sixty-four real cores for wide parallel stages, with no shared execution
resources between hardware threads.

### What this costs

Every atomic is a real barrier. Parallel reductions must be written to indexed
slots, which needs more memory and more care than a scatter of atomics.

Every population count must be batched. A kernel that counts one word at a time
is a defect on this target, and reviewers must catch it.

Local performance testing does not transfer. A development machine reports
different false-sharing behaviour and different alignment behaviour. Every
performance conclusion needs a measurement on the target, and no such
measurement exists yet.[^5]

Lower single-core speed. Any stage that resists parallel execution is slower
here than on a high-clock development machine.

A newtype for every value is friction at every boundary conversion.

### What this forecloses

Any dependency that requires a nightly compiler, including the portable vector
interface in the standard library, until it stabilises.

Any dependency that dispatches on runtime processor features inside a path that
reaches simulated state.

Cross-platform bit-exactness stays open as an option, exactly as the root
record leaves it, but nothing in this record delivers it.[^1]

Deployment to player hardware. The engine is a server component. That follows
from this record and from the root record together, and reversing it means
revisiting both.

## References

[^1]: ADR-0001, Determinism as the primary constraint, decisions D1, D2, D4, D7 and D9. `docs/adrs/draft/adr-0001-determinism.md`
[^2]: Research report 07, target platform and value types. `docs/research/reports/07-target-platform-and-value-types.md`
[^3]: Findings register, entry FND-024. `docs/FINDINGS.md`
[^4]: ADR registry, the registers section. `docs/adrs/REGISTRY.md`
[^5]: Blockers register, entries BLK-001 and BLK-007. `docs/BLOCKERS.md`
[^6]: Rust portable SIMD project repository, stabilisation status, read 2026-08-30. https://github.com/rust-lang/portable-simd
[^7]: The `wide` crate, version 0.4.6, released 2026-06-05. https://crates.io/crates/wide
