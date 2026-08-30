---
id: 0022
title: Implement random behaviour through a keyed draw
status: proposed
created: 2026-08-30
---

A soldier chooses a neighbour tile to attempt each tick. The choice comes
from the counter-based generator, keyed on the system, the frame, the entity
and the draw index, as ADR-0003 requires.

Registry row 0064 says a unit chooses by scoring a small fixed option set. A
uniform draw over six neighbours is the degenerate case of that claim. Decide
at refinement whether this item implements row 0064 or defers it, and record
the answer.

Refine this at sprint 3 planning.
