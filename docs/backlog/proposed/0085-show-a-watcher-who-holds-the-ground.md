---
id: 0085
title: Show a watcher who holds the ground
status: proposed
created: 2026-08-31
---

The engine now says who holds each tile, and the holding changes while the
world runs. The viewer does not draw it, so a watcher cannot see a boundary
and cannot see one holding meet another.

The product record asks for exactly that, and it is the one statement in its
list of checkable statements that the engine alone cannot answer.[^1] A fact
that nobody can see is a fact nobody can check.

The work is a holder layer in the viewer. A tile that nobody holds draws as it
does today. A tile that a faction holds takes a colour from the faction
identifier, and the edge between two holdings is drawn, because the edge is
what the record asks a watcher to see.

The impact review must say where the colour for a faction comes from. The
engine holds no screen position and no palette, so a colour chosen in the
engine would put presentation into simulated state.

## References

[^1]: PRD-0006, a place belongs to somebody. `docs/product/shaped/prd-0006-a-place-belongs-to-somebody.md`
