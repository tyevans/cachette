# Testing Rules

This rule states how to design a test in this project. The gates that a test
must pass are in the definition of done.[^1]

The foundation crate exists. This rule applies to every test in it.

## 1. A determinism test must be able to fail

The two determinism tests compare a run against another run.[^2] A test that
compares a run against itself always passes and proves nothing.

- The thread-count test must run the same tick at more than one thread count,
  in one process, and compare the event logs byte for byte.
- The golden state test must read the hash from a stored file. It must not
  compute the expected value in the test.

Write a test that proves the test can fail. Perturb the seed or the iteration
order behind a test-only switch, and assert that the determinism test then
fails. A determinism test with no proven failure mode is decoration.

## 2. A determinism test cannot tell correct from consistently wrong

The two determinism tests prove that a run repeats. They say nothing about
whether the run was right.

A defect that is itself deterministic passes both of them. A random draw
keyed on the wrong field draws the same wrong value on every thread, on every
run, on every machine. The thread-count test compares two runs and finds them
identical, because they are.

This has happened once. A movement system keyed its draw on the slot index
rather than the entity identity, and on the frame count not at all. Both
defects survived every movement test. Only the golden state hash caught them,
and it caught them by accident: the hash changed because the behaviour
changed, not because anything checked the key.

**Test what the value depends on, not only that it repeats.** For a keyed
draw, that means a test for each field of the key: change the field, and the
draw must change. Two tests closed the case above. A soldier that takes the
same step twelve times proves the frame is in the key. A soldier respawned
into one slot at a later generation, drawing the same direction as the
soldier that died there, proves the identity is in the key.

A golden file is not this test. It notices that something changed. It cannot
say which input the output stopped depending on.

## 2a. A fixture supplies the input, and a uniform input hides a defect

Section 2 is about the assertion. This one is about the data.

A defect usually lives at an extreme of a distribution. A fixture that models
the typical case supplies no extreme, so the assertion never receives the
input that would fail it. The test then measures the fixture.

Do not build a fixture by copying the demonstration binary's world. That world
is chosen to look right, not to produce edge values. Ask instead what
distribution the test needs, and build the world that produces it.

This has happened twice, in two subsystems, in one session. The findings hold
the detail.[^4]

**Put the defect back and watch the test stay green.** That is the only proof
that a fixture reaches the case. Both instances were found that way, and
neither would have been found by reading the test.

## 3. Do not assert on time

Never assert on wall clock time, on elapsed duration, or on a thread finishing
first. A timing assertion is flaky on a loaded machine, and it teaches everyone
to ignore a red pipeline.

Measure performance in a benchmark. A benchmark does not gate a merge.

Benchmark on the target platform. The development machines have a different
cache line size, so a local measurement misleads on false sharing.[^3]

## 4. Prefer a property to an example

The simulation rules are algebraic. State the property.

- An aggregate combines in any order and gives the same result.
- Applying an event is pure. Applying the same event twice from the same state
  gives the same state.
- A level 1 cell equals the exact sum of its level 0 tiles.

Record the seed of every failing property run in the test output, so that a
reader can repeat the failure.

## 5. Drive the real caller

A test that builds a mechanism and exercises it proves that the mechanism works.
It does not prove that anything reaches the mechanism.

When the engine is obligated to call something, drive the engine and then
inspect the mechanism. A capability that nothing invokes passes its own test and
ships inert.

Ask who must invoke this: the user of the library, or the engine. If the engine
must invoke it, the test starts at the engine.

## 6. Test at the public interface

New behaviour needs a test that goes through the public interface. A test that
reaches into a private field pins the implementation, not the behaviour.

## References

[^1]: Definition of Done. `.agents/rules/definition-of-done.md`
[^2]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^3]: Project orientation. `AGENTS.md`
[^4]: Findings register, FND-051 and FND-048. `docs/FINDINGS.md`
