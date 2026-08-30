# Testing Rules

This rule states how to design a test in this project. The gates that a test
must pass are in the definition of done.[^1]

No code exists yet. Apply this rule when the first crate exists.

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

## 2. Do not assert on time

Never assert on wall clock time, on elapsed duration, or on a thread finishing
first. A timing assertion is flaky on a loaded machine, and it teaches everyone
to ignore a red pipeline.

Measure performance in a benchmark. A benchmark does not gate a merge.

Benchmark on the target platform. The development machines have a different
cache line size, so a local measurement misleads on false sharing.[^3]

## 3. Prefer a property to an example

The simulation rules are algebraic. State the property.

- An aggregate combines in any order and gives the same result.
- Applying an event is pure. Applying the same event twice from the same state
  gives the same state.
- A level 1 cell equals the exact sum of its level 0 tiles.

Record the seed of every failing property run in the test output, so that a
reader can repeat the failure.

## 4. Drive the real caller

A test that builds a mechanism and exercises it proves that the mechanism works.
It does not prove that anything reaches the mechanism.

When the engine is obligated to call something, drive the engine and then
inspect the mechanism. A capability that nothing invokes passes its own test and
ships inert.

Ask who must invoke this: the user of the library, or the engine. If the engine
must invoke it, the test starts at the engine.

## 5. Test at the public interface

New behaviour needs a test that goes through the public interface. A test that
reaches into a private field pins the implementation, not the behaviour.

## References

[^1]: Definition of Done. `.claude/rules/definition-of-done.md`
[^2]: ADR-0001, Determinism as the primary constraint. `docs/adrs/draft/adr-0001-determinism.md`
[^3]: Project orientation. `CLAUDE.md`
