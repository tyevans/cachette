---
id: 0271
title: Count the ground generations that one frame runs
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**Nothing can count how many times one frame generates a ground.** The ground
of a tile is generated from the seed and the address, and the engine holds no
map of it.[^1] The generation is the largest part of what a drawing costs, so
the count of generations is the number that says whether a change to the
drawing worked.

The drawing carries a count of the grounds it asked the core for. That count
is of one layer. A reader below the drawing that generated a ground of its own
does not appear in it, and that is exactly the defect that item 0210
removed.[^2] Two tests in two crates hold the two halves of the claim, and no
test states the whole of it. A finding records the limit.[^3]

**A contributor can therefore put the second generation back and no test will
fail.** The drawing would ask for the stock through the reader that starts
from the address alone, the picture would not change, and both counts would
stay right.

## What is missing before this is refined

- The impact review.
- **Where the counter lives.** A count of generations must sit in the
  generator, which is in the core crate. A counter in a hot path of the
  simulation is a cost the shipped engine must not pay, so the mechanism has
  to be absent from a normal build.
- **Whether a counter can be made determinism-safe.** A shared counter that
  more than one thread writes is a value that depends on the thread order, and
  the record forbids that in simulated state.[^4] A count that no simulated
  state reads may be outside that rule. The item must settle it rather than
  assume it.
- Whether the answer is a counter at all. A second option is a lint or a check
  that forbids the drawing from naming the reader that starts from the address
  alone. That option costs nothing at run time and proves less.
- What a test asserts. A test that asserts on elapsed time is forbidden.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^2]: Backlog item 0210, generate the ground of a drawn tile once. `docs/backlog/complete/0210-generate-the-ground-of-a-drawn-tile-once.md`
[^3]: Findings register, FND-261. `docs/FINDINGS.md`
[^4]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Testing Rules, section 3. `.claude/rules/testing.md`
