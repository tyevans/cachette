---
id: 0340
title: Answer whether one faction stands in the territory of another, in one call
status: complete
created: 2026-09-03
implements: [ADR-0053, ADR-0111]
changes: []
creates: [ADR-0111]
serves: [PRD-0031]
blocked-by: []
---

## Why

A downstream game gates a conversation between two players on presence. One may
speak to another only while one of its own units stands in the other's
territory. The engine holds every part of that answer and exposes none of it as
one call.

A tile carries one holder. A unit carries a faction and a tile. The control
plane reads the holder of one address at a time, and it cannot list the units of
a faction at all, so the only route today is a loop over the population.

**The answer for the whole world is one set of factions for each faction.** The
world admits at most 63 factions, so the whole relation is a small fixed number
of words. Deriving it rides on the passes that already visit every unit and the
tile it stands on. The findings register holds the reasoning.[^1] The decisions
register holds the options.[^2]

## Impact review

**Governed by.** ADR-0053 D7 fixes the shape of every relation between
factions: one mask row for each faction, never a field over the world. This is
the first relation built against it. ADR-0053 D1 makes a faction one bit of a
64-bit word. ADR-0053 D2 gives a tile one holder. ADR-0053 D4 keeps a running
total for each faction. ADR-0023 D1, D2 and D3 ask an aggregate to combine
exactly, associatively and commutatively, which a union of sets does. ADR-0009
D1, D2 and D3 govern the parallel fold. ADR-0004 D1 asks for an explicit
iteration order. ADR-0001 D1 asks for one answer at any thread count. ADR-0040
D1 and D2 forbid a boundary crossing for each entity. ADR-0018 D3 supplies the
freshness guard that this work copies. ADR-0022 D2 says that a derived thing is
derived again and never stored.

**Contradicts nothing.** ADR-0053 D7 states that no code stored a relation yet
and that this decision fixes the shape the first one must take. The work takes
that shape.

**Creates.** ADR-0111, the presence relation is derived at the end of the step
and never stored as a fact. The three-condition test passes. A contributor
could reasonably maintain a presence set incrementally from every rule that
moves a unit or changes a holder. Choosing that costs determinism and a
declaration site for each such rule, and it is expensive to undo. The reasoning
is not visible in the code. The registry row was allocated before the record
was written.

**Blockers.** BLK-050 holds the rules of the downstream game, and it stays
open. It governs whether the game's rule is symmetric. The engine answers
either way, because the relation is directed: a caller asks one direction, or
reads both rows and combines them. BLK-007 governs every cost figure in this
project, so this item states no measured figure.

**Registers.** DEC-141 closes with option A first, as it recommended. FND-370
records what the fold's placement in the step turned out to be, against what
the research report assumed. FND-371 records that the golden state hash did not
move.

**Serves.** PRD-0031.

## What the work does

Derive one faction mask for each faction, at the end of every step. Row `host`
names every faction with a live unit on a tile that `host` holds. Expose it to
the control plane as one array, and expose the one-pair question beside it.

## What good looks like

A caller answers "which gods may I message" in one crossing. The size of the
answer follows the faction ceiling and nothing else. A guest in foreign
territory sets a bit, and the bit clears when the guest leaves or dies. A unit
on its own ground sets no bit. The relation is the same at every thread count.
The golden state hash does not move, because the relation is derived and
reaches no state.

## What it does not do

It does not say which units are standing there. That is a set-valued read and
the selector will hold it.[^3]

It does not carry a message, and it does not define territory.

## References

[^1]: Findings register, FND-362. `docs/FINDINGS.md`
[^2]: Decisions register, DEC-141. `docs/DECISIONS.md`
[^3]: ADR-0051, a selector is a lazy expression tree that Rust evaluates. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
