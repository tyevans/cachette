---
id: 0347
title: Read the tile holder as a column
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0031]
blocked-by: []
---

## Why

The control plane reads who holds a tile one address at a time, as one entry of
a report built for that address. The engine holds the holders as one dense
column over the tiles, and it holds one faction mask for each block.[^1]

So a caller who wants to draw a map of who holds what, or to ask about a
region, walks the world from Python. That is the loop the boundary rule
forbids, and the engine already returns tile values as one array, so the shape
of the answer exists.

This is smaller than the presence relation and it does not replace it. The
relation answers whether anybody is present. This answers where the ground is.

## What is missing before this can be refined

- Whether it returns the whole world or a window. The engine refuses a window
  census above a radius ceiling, and the same argument may apply.
- What the column holds where nobody holds the tile. A holder is a faction or
  nobody, and a report currently answers the second with a null value.
- Whether the item is absorbed by the selector. A description of the tiles one
  faction holds is a selector expression, and building this first may build the
  same thing twice.

## References

[^1]: Findings register, FND-361. `docs/FINDINGS.md`
