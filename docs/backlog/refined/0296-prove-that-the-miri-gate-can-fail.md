---
id: 0296
title: Prove that the Miri gate can fail
status: refined
created: 2026-09-02
implements: [ADR-0097 D4]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

**The project now runs Miri, and nobody has shown that the run can reject
anything.** The gate drives the engine, hashes the state, and passes.[^1] A
gate that has only ever passed is a gate whose reach is unmeasured, and this
project has a rule about that shape: a test with no proven failure mode is
decoration.[^2]

**The project already answers this question for its other invisible gate.**
The determinism tests carry a build switch that perturbs the join order, and
the gate asserts that each determinism test then fails and that the probe that
watches them passes.[^3] The Miri gate has no equivalent, so its coverage is
asserted rather than measured.

**The specific doubt is real, not theoretical.** The gate exists to catch an
undeclared padding byte reaching the state hash. Whether it does depends on
things nobody has checked: whether the fixture's extent reaches the structure
that would hold the padding, whether the hash reads that structure as bytes
rather than field by field, and whether the interpreter's check is on for that
read. Each of those could be wrong, and the gate would stay green through all
of them.

**A fixture that never produces the case measures the fixture.** The gate's
world is deliberately small, because a world at the target unit population does
not finish under interpretation.[^4] Small was chosen for cost. Nothing has
shown that small still reaches the defect.

## Impact review

**Governed by.** ADR-0097 D4 establishes the Miri gate over operations that read
the simulation state as raw bytes.[^5] ADR-0006 D1 and D3 require every event
and every hashed type to use a declared layout without undeclared padding.[^6]
Hard invariant 5 requires every event and hashed type to be `bytemuck::Pod` with
declared padding.[^7] Undeclared padding causes false nondeterminism in state
hashes.

**Changes.** None.

**Creates.** None.

**Blockers.** None.

**Serves.** PRD-0002.[^8]

## What the work does

Add a build switch that puts an undeclared padding byte into a type the state
hash reads as bytes, in the shape the invariant forbids. Then assert that the
Miri gate fails while the switch is on, and that the ordinary test suite still
passes while it is on.

The second assertion is the interesting one. It states what the gate is for: a
defect that every other gate in this project waves through.

## What good looks like

Two runs of one commit. With the switch on, the Miri gate rejects the read and
names the uninitialised byte. With the switch off, it passes. The ordinary
suite passes in both.

**Put the defect back and watch the gate stay green.** If the perturbed build
passes the Miri gate, the fixture does not reach the case, and widening the
fixture is then the work rather than an afterthought.

## Done when

- The `probe-undeclared-padding` feature compiles in `cachette-core`.
- Under the probe feature, `CarryLoad` contains an uninitialised padding byte.
- The ordinary test passes when the probe feature is on.
- The Miri test fails and detects an uninitialised byte read when the probe feature is on.
- The `miri-probe` recipe in the justfile runs both assertions and exits with zero.
- The continuous integration miri job runs `just miri-probe`.

## What it costs at the target scale

Nothing at run time. The switch is a build feature, off by default, in the same
shape as the existing perturbation switch.

The cost is gate time. The probe adds a second interpreted run, and
interpretation is the slow part. Whether that belongs in every continuous
integration run or in a slower job is a judgement this item should make with a
figure rather than without one.

## What it does not do

It does not widen the Miri gate to more tests. If the probe shows the fixture
does not reach the case, that is a finding and a separate item.

It does not measure what the gate costs. No figure for it exists in any
register, and the record that created the gate says a register owns any such
figure.[^5]

## References

[^1]: The state-byte gate. `crates/cachette-core/tests/state_bytes_are_initialised.rs`
[^2]: Testing rules, section 1. `.claude/rules/testing.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D5. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Findings register, FND-285. `docs/FINDINGS.md`
[^5]: ADR-0097, the toolchain is a dated nightly, decision D4. `docs/adrs/draft/adr-0097-the-toolchain-is-a-dated-nightly.md`
[^6]: ADR-0006, an event is plain data and applying it is pure, decisions D1 and D3. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^7]: Project orientation, the hard invariants, rule 5. `CLAUDE.md`
[^8]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
