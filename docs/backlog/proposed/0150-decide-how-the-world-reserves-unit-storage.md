---
id: 0150
title: Decide how the world reserves unit storage
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: [PRD-0012]
blocked-by: [DEC-059]
---

## Why

The accepted product record for a founding states that the storage the world
reserves is sized for the target population, that it does not change during a
run, and that a run does not stop to grow.[^1]

The engine does the opposite. The unit arena opens as many slots as the slot
index holds, its own comment says that the limit is the range of the index and
not a budget, and it reserves no memory. Each spawn appends one entry to each
of its columns, so the storage grows with the population under a running
simulation. A driver founded 120 people through the public interface and read
the capacity back as the range of the index.[^2]

A record the code contradicts is worse than no record, because it lies.[^3]
One of the two changes.

The choice is architectural, so a row in the decisions register holds it. The
row recommends that the world reserves the columns at construction and refuses
a spawn past the reservation.[^4]

**What refining this must answer.** Whether the reservation belongs to the
world settings or to a separate population setting. Where the refusal surfaces,
and whether the founding path already has a typed refusal that fits. What the
state hash does, if anything, when a reservation replaces a growth. Whether the
other three entity arenas hold the same question, or only the unit arena.

## What the work does

The work follows the option the decisions row closes on. Under the recommended
option:

1. The world takes the reservation from one place, and no second copy states
   it.[^5]
2. The unit columns reserve that many entries at construction.
3. A spawn past the reservation gets a typed refusal, and a test drives that
   path through the public interface.
4. The product record then describes the code, and no sentence in it changes.

Under the option that keeps the growth, the work removes the storage statement
from the product record instead, and the record says what the engine does.

## Done when

- The decisions row is closed, and the row states the outcome and the
  reasoning.
- The engine and the product record agree. A reader can check the statement
  against the code.
- A test goes through the public interface for whichever behaviour the row
  chose.
- The two determinism tests pass at 1, 2 and 12 threads.
- The whole gate command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^2]: Findings register, FND-135. `docs/FINDINGS.md`
[^3]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^4]: Decisions register, DEC-059. `docs/DECISIONS.md`
[^5]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
