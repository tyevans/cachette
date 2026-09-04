---
id: 0346
title: Read the units of a faction as a set
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

**Nothing lists the units of a faction.** The control plane reads a population
count for each faction and no identities. So a caller must keep every identity
that a spawn ever returned, in a Python list, for the life of the run.

A developer who cannot list their own people cannot use any verb on a described
set. Every write verb at this boundary already takes a set, so the gap is that
nothing produces one.

This is the read side of the selector. A research report ranks it and says the
read side must be built with the selector rather than after it, because a
selector that only feeds verbs leaves the loop exactly where it is.[^1] One
item already holds the general gap.[^2]

## What is missing before this can be refined

- The relationship to the existing item, which states the same gap from the
  write side.[^2] Refining one should decide whether they are one item.
- Whether this waits for the whole selector. A read of the units of one faction
  is narrower than a selector, and it may be the first thing the selector is
  proved against rather than the last.
- What a column holds for an identity that named nobody. A research report
  recommends a mask and never an in-band sentinel.[^3]

## References

[^1]: Research report 20, what the Python interface should be, section 7.3. `docs/research/reports/20-the-python-interface.md`
[^2]: Backlog priority index, item 0161. `docs/backlog/PRIORITY.md`
[^3]: Research report 20, what the Python interface should be, section 2.3. `docs/research/reports/20-the-python-interface.md`
