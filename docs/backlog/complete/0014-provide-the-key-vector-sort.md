---
id: 0014
title: Provide the key vector sort that content orders through
status: complete
created: 2026-08-30
implements: [ADR-0007 D1, ADR-0007 D2, ADR-0007 D3]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

ADR-0007 says content supplies an ordered vector of key fields and never a
comparison function, that the last field is a stable identifier so no two
items tie, and that the engine never calls content code from inside a
sort.[^1]

No sort exists in the crate. Every subsystem that orders anything needs one:
admission orders by a stable key, a selector result is ordered, and a
modifier pipeline applies in order.[^2] Writing the sort once, before those
subsystems exist, is what makes the record enforceable rather than advisory.

## Impact review

**Governed by.** ADR-0007 D1, D2 and D3 state the interface. ADR-0004 D4
requires a sort to use a stable key. ADR-0002 D1 makes every key field an
exact integer, which is what permits a radix sort.[^1] [^3] [^4]

**Changes.** None.

**Creates.** None. ADR-0007 holds the claim.

**Blockers.** None. BLK-007 governs whether a radix sort beats a comparison
sort at scale on the target platform, and that is a benchmark question, not a
correctness one. Write the interface so that either implementation satisfies
it, and record no throughput figure.[^5]

**Precedent.** FND-040 records what a citation costs when it decays; this item
cites the accepted record and its decision numbers, which are now fixed.[^6]

**Serves.** No product record. ADR-0056 D3 depends on this mechanism, and that
record stays in draft.[^2]

## Done when

- The core exposes a sort that takes an ordered vector of exact integer key
  fields and orders items by it.
- The interface cannot accept a comparison function from content.
- The sort rejects, at compile time or by construction, a key whose last field
  is not a unique stable identifier.
- A property test asserts that the output is one exact permutation, whatever
  the input order, for a generated input with many ties in every field but the
  last.
- A property test asserts that the result is identical at 1, 2 and 12 threads.
- The failing seed prints on a property failure, so a reader can repeat it.[^7]
- The record holds no measured figure, and neither does the code comment.
- `just check` runs green.

## Outcome

`crates/cachette-core/src/sort.rs` holds the sort. No entry point accepts a
function of any kind, not a comparator and not a key extractor, so ADR-0007
D3 holds by construction rather than by discipline. A key with no identifier
field fails to compile, and a repeated identifier is refused before any
sorting.

Two things changed from the plan. The plan asked for a proven failure mode
under the probe; the sort has none, and the module now states why instead of
pretending otherwise. The merge picks the lowest remaining key and every key
is unique, so the output is one exact permutation whatever order the runs are
read in. The runs are still read through `combine` rather than the raw slot
array, so a later algorithm that loses that property does not also have to
remember where to read.

The item said to record no throughput figure and none is recorded. The
comparison sort per chunk can be replaced by a radix sort without changing a
caller, which is what BLK-007 will decide.

## References

[^1]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^2]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/draft/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-040. `docs/FINDINGS.md`
[^7]: Testing Rules. `.claude/rules/testing.md`
