# Target Platform and Value Types — Addendum

Written by the session lead after the six research reports landed. This
material was NOT available to the research agents. It comes from a later
decision by the project owner and from discussion in the lead session.

**Status of the claims below:** the hardware facts are stated at the
confidence level the lead holds them. Items marked VERIFY must be checked
against current AWS and ARM documentation before the ADR is finalised.

## New decision: target platform

The engine targets **server game backends on ARM cores**, specifically AWS
Graviton. Deployment is headless and multi-tenant. Development happens on
x86-64 and Apple Silicon; those are development-only targets.

Primary target triple: `aarch64-unknown-linux-gnu`.

This is a change in framing as well as hardware. A headless, multi-tenant
simulation server with a Python control plane is a more coherent product
than a "2D game engine". It also aligns with the third audience (RL and
agent-based research), which runs on the same cheap cloud cores.

## What ARM64 gives us

**NEON is baseline on aarch64.** 128-bit NEON is mandatory in the base ISA.
This removes runtime feature detection, function multiversioning, and
`target_feature` dispatch — the `wide` + `multiversion` machinery that
reports 01 and 06 recommended for x86. There is one code path, which is
also easier to keep deterministic.

**64-bit integer arithmetic runs at full rate.** No penalty against 32-bit.
This makes wide accumulators free (see the accumulator-width trap below).

**No SMT.** 64 vCPU means 64 real cores. This strengthens report 06's
"few very wide stages" recommendation. Single-thread performance is lower
than high-clock x86, so favour wide-and-simple over clever-and-serial.

**No x87.** One class of float-determinism problem does not exist.

## What ARM64 demands care about

**Weak memory model, not TSO.** Relaxed and acquire atomics emit real
barriers on ARM where they are near-free on x86. Two consequences:
- The dirty-bitset `fetch_or` pattern costs more. Keep it, but do not
  scatter atomics casually.
- False sharing hurts more, because recovery involves fence traffic.

This PROMOTES report 06's "disjoint outputs, indexed slot reductions"
recommendation from preferred to **required**.

**LSE must be enabled explicitly.** Rust's `aarch64-unknown-linux-gnu`
baselines at ARMv8.0. Large System Extensions (single-instruction atomics
such as `LDSET`) arrived in ARMv8.1. Without them, atomics compile to
LL/SC retry loops that degrade badly under contention on a 64-core box.
Graviton 2 and later support LSE. VERIFY the exact Graviton-to-Neoverse
mapping and the correct `target-cpu` value per generation.

Day-one `.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
rustflags = ["-C", "target-cpu=neoverse-n1"]   # neoverse-v2 for Graviton 4
```

**No scalar popcount instruction.** aarch64 routes `u64::count_ones()`
through the vector unit: move to a NEON register, `CNT` per byte, then
`ADDV` to reduce. Roughly 3-4 instructions against x86's single `POPCNT`.

Design consequence, and it matters because this design is bitplane-heavy:
**batch popcounts**. Counting one word at a time pays setup each time.
Counting 16 bytes at once with `CNT` and accumulating across a block is
where NEON wins. Structure aggregation kernels to popcount whole blocks,
never single words in a loop.

**SVE: skip it.** Vector-length-agnostic code is a real complexity cost
for little gain over NEON here. VERIFY SVE widths per Graviton generation
if you want to record them, but the recommendation stands regardless.

## Resolution of the block-size conflict

Reports 01, 02 and 06 disagreed. Report 01 wanted 32x32 tile blocks because
a 32x32 bitplane block is 16 u64 words = 2 cache lines, making parallel
bitset writes race-free by construction. Reports 02 and 06 independently
wanted 16x16, because 32x32 leaves L2 with only 4x4 = 16 cells, which
prunes essentially nothing.

Neoverse cores use 64-byte cache lines. That gives the numbers:
- 16x16 bitplane block = 256 bits = **32 bytes** = half a cache line.
  Two adjacent blocks share a line. Parallel writes false-share, which is
  worse under ARM's weak model.
- 32x32 bitplane block = 1024 bits = **128 bytes** = exactly 2 lines.

**These are two different granularities and the lead was wrong to treat
them as one.** Resolution: aggregate at 16x16 for a useful three-level
pyramid, and separately align and pad bitplane storage to 64-byte
boundaries. A 16x16 block padded to a full cache line wastes 32 bytes per
block — about 2 MB across the map, against a ~134 MB world. Buy the
alignment, keep the fanout.

The ADR should record this reasoning, not just the outcome.

## Value types

**Fixed-point over float.** Four reports independently converged on banning
floats from aggregated and simulated state (01 cache/determinism, 02
non-associativity of float fold, 03 exact-associativity requirement for
monoids, 04 non-associative modifier stacking). ARM makes the alternative
cheap: 64-bit ops are full rate, so `i64` fixed-point costs nothing against
`i32`.

**The accumulator-width trap.** A `u8` tile field summed over 2^24 tiles
reaches 2^32 exactly — it overflows `u32` at the top of the pyramid.
Pyramid accumulators must be wider than the tile fields they summarise,
and the widening must happen at L1, not L2. Free on a 64-bit target.

Proposed vocabulary. Newtype everything; they are zero-cost and they
prevent `TileIdx`/`ChunkIdx` confusion.

| Type | Repr | Notes |
|---|---|---|
| `TileIdx` | `u32` | Block-tiled odd-r offset index. 2^24 tiles leaves headroom. |
| `ChunkIdx` | `u32` | Derived by shift from `TileIdx`. |
| `Entity` | `NonZeroU64` | `NonMaxU32` index + `u32` generation. `Option<Entity>` stays 8 bytes. |
| `UnitTypeId` | `u16` | Index into the stat table. |
| `TerrainId` | `u8` | Keep under 32 variants for the terrain-cost matrix. |
| `FactionId` | `u16` | |
| `UpgradeSetId` | `u32` | Interned, unbounded. Per report 04. |
| `CapabilityMask` | `u64` | DERIVED from `UpgradeSetId`. Hot loops and selector predicates only. |
| `Fix32` | `i32`, Q16.16 | Positions, per-entity scalars. Report 04 proposes 1/1024 for modifiers — reconcile. |
| `Fix64` | `i64`, Q32.32 | Where range demands it. |
| `Accum` | `i64` | Pyramid accumulators. Always. |
| `Tick` | `u64` | |

**Two invariants on every type that crosses into an event:**
- Must be `bytemuck::Pod`: `repr(C)`, explicit padding, no `bool` (use `u8`).
- Report 05 flagged struct padding as a source of phantom nondeterminism in
  per-frame state hashing. This rule is where that is prevented.

**LDP/STP favour 8- and 16-byte alignment.** ARM loads and stores register
pairs. Component structs sized and aligned to 8 or 16 bytes move in single
instructions during archetype migration. Honour this in the layout.

## What server targeting changes beyond ARM

**Determinism gets easier — take the win.** The earlier framing assumed
arbitrary player hardware. On a fleet of identical instance types under our
control, "bit-exact for one binary on one architecture" is not a compromise,
it is the actual deployment. Report 03's recommendation to ship that and
architect for cross-platform is now clearly correct.

**The development-machine trap.** Development happens on x86-64 and Apple
Silicon; deployment is Graviton. Apple M-series uses 128-byte cache lines,
not 64. Local performance testing will therefore mislead on exactly the
false-sharing and alignment questions above. Make cache-line size a
compile-time constant and benchmark on the target.

## Required ADR sections arising from this addendum

1. A target-platform section naming `aarch64-unknown-linux-gnu` with
   `target-cpu` as primary, and x86-64 / Apple Silicon as development-only.
2. The value-type table, with the Pod rule and the accumulator-width rule
   stated as invariants.
3. The cache-line constant, with the Apple Silicon discrepancy noted.
4. The block-size resolution and its reasoning.
