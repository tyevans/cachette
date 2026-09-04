---
id: 0341
title: Bind the build order and the upgrade removal to the control plane
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

The core orders a build for one unit and one kind, stops that order, reports it,
and removes a finished upgrade at an address. No line of the bindings crate
names any of the four, and no Python file names them either. The findings
register holds the search that measured it.[^1]

So a developer who wants to build anything can only found a settlement, which
creates an entity rather than marking the ground. A downstream game names
building as one of the six things its players must do.

This is a capability that nothing invokes, which is a shape this project already
lists.[^2] The mechanism is built and its own tests pass. Nothing reaches it.

## What is missing before this can be refined

- The shape of the order. Every write verb at this boundary takes a set and is
  all or nothing, and this one must match.
- Whether the kind crosses as a bare integer. Three integer scales in this
  interface already share the name `kind`, and a range check cannot separate
  two numberings that overlap.[^3]
- What the caller reads back. A build that a watcher cannot see is a build
  nobody can repair.

## References

[^1]: Findings register, FND-360. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^3]: Findings register, FND-352. `docs/FINDINGS.md`
