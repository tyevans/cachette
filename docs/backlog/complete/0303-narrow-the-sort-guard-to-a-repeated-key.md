---
id: 0303
title: Narrow the sort guard to a repeated key
status: complete
created: 2026-09-03
implements: [ADR-0105]
changes: []
creates: [ADR-0105]
serves: [PRD-0002]
blocked-by: []
---

## Why

**Every ordering pass in the engine sorted its whole key set twice.** Before the
radix ran, the sort collected the identifier of every key into a new vector and
comparison-sorted that vector to find a repeated value. Both ordering passes of
a frame paid it, on every frame.[^1]

**The property the guard proved is wider than the one determinism needs.** An
order is safe when no two keys are indistinguishable. Two keys that share an
identifier and differ in an ordering field are separated by the field they
differ in, so nothing is left for the identifier to decide.

The decisions register held the question with three options and a
recommendation, and it said the answer needed a record because narrowing a
shared sort's guarantee is a determinism decision.[^2]

## Impact review

**Governed by.** ADR-0001 binds every change to the step: one binary gives one
answer at any thread count.[^3] ADR-0007 D1 and D2 state that content supplies a
key vector and that the last field is a stable identifier.[^4] ADR-0004 D1 binds
the iteration order to something explicit and stable.[^5]

**One record was created rather than contradicted.** ADR-0105 states the narrow
property, why it is sufficient, and what it stops refusing. The registry row was
allocated before the file was written, and the record is `Draft`. Its author
must not review it.[^6]

**No blocker governs a value here.**

## Done when

- The sort refuses a repeated key and accepts a repeated identifier that ties
  nothing.
- A record states the property, and the registry row exists before the file.
- Breaking the narrow check fails a test. If it fails none, the check is
  decoration and that is the finding.
- A measurement on the target platform, taken so that a relayout cannot be
  mistaken for the change.

## Outcome

**The property had no test, and that was worth more than the speedup.** The two
tests that asserted the refusal both used keys sharing an identifier and
differing in an ordering field: a pair that ties nothing. A repeated key appeared
in no test in the repository. Narrowing the guard and running the suite failed
exactly those two tests and nothing else, so nothing in the project depended on
the wider refusal. Removing the narrowed guard altogether then failed nothing at
all, until three tests were written for it.[^7]

**The check was put back twice.** Once against the narrowed guard where it first
sat, and again after it moved into the pass that orders ties, because a check
proved in one place is not proved in another.

**The first implementation made the frame worse and the measurement caught it.**
It walked the sorted order and read two keys through the permutation for each
neighbouring pair, which is a random gather and not the cheap linear pass its own
comment claimed. The repair puts the check where the ties already are: a run
holding one entry cannot hold a repeated key, and nearly every run holds
one.[^8]

**The frame is unchanged, and the reason is instructive.** Admission's sort is
measurably cheaper and its three runs agree closely. Four stages that share no
code with the sort rose in all three runs, which is the cost of relaying a binary
built with full link-time optimisation and one code generation unit. The cost
register holds the five runs.[^9] [^10]

**One run in five was a 25 percent outlier on one stage.** A single pair taken at
that run would have reported a 9 percent worse frame; a single pair taken at the
next would have reported a 1 percent better one. This is what put a stated bound
under every figure the project takes from one pair.[^10]

**Two red gates were repaired on the way.** An example binary merged earlier used
`f64` to print a measurement, which failed both mechanisms that hold the float
ban: the lint and the script. The arithmetic is now integer. The state hash the
example prints is unchanged.

## References

[^1]: Findings register, FND-302. `docs/FINDINGS.md`
[^2]: Decisions register, DEC-111. `docs/DECISIONS.md`
[^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^5]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^7]: Findings register, FND-305. `docs/FINDINGS.md`
[^8]: Findings register, FND-306. `docs/FINDINGS.md`
[^9]: Target platform costs, the narrowed sort guard measured across five runs. `docs/reference/graviton-costs.md`
[^10]: Findings register, FND-308. `docs/FINDINGS.md`
