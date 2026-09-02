---
id: 0080
title: Give the world settings a constructor
status: proposed
created: 2026-08-31
serves: []
---

The settings struct that builds a world has public fields and no constructor.
Every caller therefore builds it with a struct literal. Adding one field
breaks every literal in the tree at once, in the core crate, in the viewer, in
the Python binding and in the Python type stub.

That already stopped a piece of work. The site rate work needed a schedule
period and a phase. It put both in the settings struct, the compiler refused
twenty-five files, and the work moved the schedule to a default on the world
with a setter beside it. The finding holds the reasoning.[^1]

**A second piece of work has since paid the price rather than avoided it, and
it was three times higher.** The unit reservation work added one field and the
compiler refused 82 struct literals across three crates. The register holds
the figure and the search command that produced it.[^3] Two measurements make
this a trend rather than an anecdote: the cost grows with the number of places
that build a world, and nothing bounds that number, so deferring this item
gets more expensive and never less.

The move was right for a cadence, because a cadence has a recommended value
and a caller may leave it. It is not right for every parameter. A parameter
that a caller must state belongs in the settings, and the settings must be
able to grow.

Two options, and this item chooses between them.

A constructor that takes the values a caller must state, with the rest behind
setters or behind a builder. This removes the struct literal from every
caller, so a later field costs nothing.

A `#[non_exhaustive]` attribute plus a `Default` implementation, so a caller
writes the fields it cares about and spreads the default over the rest. This
is smaller, and it still forces one edit at each literal.

Neither option is free, because both touch every caller once. The value is
that they touch every caller once and never again.

Refine this against the value type record and the Python boundary records. The
Python type stub states the same shape a second time, so whatever this item
chooses must leave something that fails when the two copies disagree.[^2]

## References

[^1]: Findings register, FND-064. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: Findings register, FND-145. `docs/FINDINGS.md`
