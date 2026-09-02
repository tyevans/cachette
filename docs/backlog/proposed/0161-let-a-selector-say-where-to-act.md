---
id: 0161
title: Let a selector say where to act
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: [ADR-0040, ADR-0043, ADR-0051, ADR-0052]
serves: []
blocked-by: []
---

## Why

The control plane cannot say where to act. It can name a unit it already holds,
and it can name an address it chose, but it cannot ask for the places that
answer a description. A caller that needs ground holding a resource therefore
sweeps the world and lets the engine decide, and the findings register holds
that with its evidence.[^1]

The project reserved the answer and never wrote it. Four registry rows hold the
claim and no file exists for any of them: Python is a control plane, a declared
tier enforces the no-loop rule and the API refuses the loop, a selector is a
lazy expression tree that Rust evaluates, and a selector result may be a range
rather than an enumerated set.[^2]

Under a selector the control plane never asks where to act. It says where as
part of the command. One predicate tree crosses the boundary, the engine
evaluates it over the columns, and the answer may be a range or a mask rather
than sixteen million enumerated indices.[^3]

The rules exist and the mechanism does not. That gap is what the finding
records, and closing it is what turns a rule a caller cannot follow into one it
can.[^1]

## What the work does

Write the four records, then build what they state. The records come first,
because each governs every verb the boundary will ever carry.

**The author of these records is not their reviewer.** The registry says an
author may set `Draft` and only a reviewer may set anything beyond it.[^4]

## What it must not do

It must not build the expression tree before a caller needs it. Nothing needs
it today, and a capability that nothing invokes ships inert.[^5]

It must not answer the question with a per-tile read. A call that asks whether
one tile holds a resource moves the sweep from units to tiles, and the tile
population is the larger one.[^1]

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-147. `docs/FINDINGS.md`
[^2]: ADR Registry, rows 0040, 0043, 0051 and 0052. `docs/adrs/REGISTRY.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^5]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
