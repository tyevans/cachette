---
id: 0019
title: Implement the soldier column set
status: proposed
created: 2026-08-30
---

Give the core one of the four entity shapes that ADR-0066 fixes: the soldier.
A soldier is mobile, so it carries a tile position, a faction and a
generational handle.

The other three shapes wait. The record fixes four shapes; it does not
require that all four exist at once.

Refine this at sprint 2 planning. The storage rows 0012, 0014 and 0021 have
no file, so item 0007 writes the claims this item implements.
