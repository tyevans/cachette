---
id: 0112
title: Build a world without a pass over every tile
status: complete
created: 2026-08-31
implements: [ADR-0088 D1, ADR-0088 D2, ADR-0088 D3, ADR-0088 D4]
changes: []
creates: [ADR-0088]
serves: [PRD-0003]
blocked-by: []
---

## Why

The product record for the ground states a cost shape: building a world must
not cost a pass over every tile before the first frame, so a developer who
changes a seed sees the new world at once.[^1]

The ground meets this. It allocates nothing and computes a tile only when a
reader asks for it.

The world did not. `World::new` looped once for each tile, drew a random
value for each one, and pushed into a vector sized to the tile count. At the
target count of 16.7 million tiles that is a pass over the whole world and an
allocation proportional to it, paid before anything is drawn.

The column belongs to the tile stub, not to the ground. The record's
statement is nevertheless false of the engine, and a record the code
contradicts is worse than no record.[^2] The record stayed in `accepted/`
because of it, and a finding holds the case.[^3]

The loop filled two columns when this item was written. Item 0084 removed the
second one, which named a faction and was not a holder.[^4] This item covers
the remaining stub value column and the pass that filled it.

## Impact review

**Who reads the column.** A whole-tree search found eleven call sites, in the
core, in the viewer, in the bindings and in the tests. The search command is
in the commit body of the change. Four kinds of reader exist. The drawing pass
reads one tile for each tile the window covers. The state hash reads every
tile. The first pyramid level sums the tiles of each block. The bindings copy
the whole column into a new array for Python.

**Governed by.** ADR-0012 D2 states that a tile field is one dense column.
This work leaves that shape, so the work needed a record of its own before it
could proceed. ADR-0001 D4 governs the state hash, and the field must reach
it. ADR-0002 D1 and D2 govern the arithmetic. ADR-0003 D1 governs the keyed
draw that generates a value. ADR-0004 D1 and D2 govern the order of the
merged run and of the sum. ADR-0006 D1 governs the purity of the frame
update. ADR-0009 governs what a parallel worker may write. ADR-0022 D1 states
that level 0 is the only truth, and the field is level 0.

**Changes.** No record changes. ADR-0012 D2 carries an exception clause, and
the review checked it: the clause delegates the width of a column, the
encoding of a boolean field and the form of a rare field, and it names the
record that holds a narrow column with bitplanes and sparse side tables. It
delegates the shape a column takes, not the case where no column exists. So
the clause does not cover this work. D2 is nevertheless left standing rather
than superseded, because two accepted records already sit outside it and
neither superseded it.[^5] [^6] Whether it should be superseded is a
reviewer's choice, and a register row holds it with the argument on both
sides.[^7]

**Creates.** ADR-0088, a tile field is a generated base and a stored
change.[^8] The registry row was allocated before the record was written. The
author set `Draft`. The author is not the reviewer.

**Blockers.** BLK-007 governs every cost figure, because no measurement
exists on the target platform.[^9] The work therefore states the shape of a
cost and never a number, in the code, in the comments and in the record.

**Precedent.** FND-086 records that this product record stated a cost the
engine never met, and that a checkable statement is not checked until
somebody runs it.[^3] The work therefore delivers a test that fails when the
pass returns, not a claim that the pass is gone.

**Serves.** PRD-0003. The work does not close it. Two passes remain, and item
0171 holds them.[^10]

## Done when

- Building a world stores nothing for the tile value field, at every extent.
- Building a world makes no visit to a tile of the tile value field. A
  test-only switch counts the visits, and the same test proves the counter
  counts.
- A tile reads one value, whether a frame has changed it or not.
- Two worlds built from one seed hold one field. Two seeds hold two.
- The tile index reaches the key of the generated value.
- The state hash covers the value of every tile.
- The thread-count test is byte-identical at 1, 2 and 12 threads.
- The eager pass, put back, makes the new tests fail.
- The whole check command runs green.

## Outcome

**Done.** The tile value field holds the seed, the extent, and one entry for
each tile that a frame has changed. A read generates the base and adds the
stored change. ADR-0088 records the shape as a claim over any tile field, not
as a fact about this one.

**The golden files did not move.** The generated value is the same expression
the eager loop used, keyed the same way, so every tile reads what it read
before. The plan for this item expected every golden file to move. A hash
that had moved would have been a behaviour change to explain.

**The experiment.** The eager pass was put back and both new test files
failed: the visit census reported 512 visits for a 256-tile world against the
256 the pyramid alone makes, and three storage assertions failed. The pass was
then removed and both files passed. The two runs were minutes apart on one
machine. No timing figure was taken, because two other workers shared the
machine and an unpaired wall clock number is not evidence.[^11]

**What changed from the plan.** The plan believed the tile value column was
the whole of the build cost. It is not. The build still passes over every tile
twice, through the first pyramid level, and it still allocates one holder for
each tile. PRD-0003 therefore stays false of the engine and stays `Accepted`.
FND-162 records the correction and item 0171 holds the remaining work.[^12]
[^10]

**Registers.** FND-162 and FND-164 opened. DEC-068 opened. ADR-0088 added to
the registry as `Draft`. Item 0171 added and placed in the priority index.

## References

[^1]: PRD-0003, a developer sees a world worth looking at, what it costs at the target scale. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^2]: Definition of Done. `.claude/rules/definition-of-done.md`
[^3]: Findings register, FND-086. `docs/FINDINGS.md`
[^4]: Backlog item 0084. `docs/backlog/complete/0084-give-a-tile-one-faction-column.md`
[^5]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^6]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^7]: Decisions register, DEC-068. `docs/DECISIONS.md`
[^8]: ADR-0088, a tile field is a generated base and a stored change. `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
[^11]: Findings register, FND-142. `docs/FINDINGS.md`
[^12]: Findings register, FND-162. `docs/FINDINGS.md`
