---
id: 0105
title: Move a good over a network by solving a flow
status: proposed
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0004 D4]
changes: []
creates: [ADR-0061, ADR-0058, ADR-0049]
serves: [PRD-0010]
blocked-by: []
---

## Why

A good is taken where it is found and stays with the unit that took it.
Nothing carries a good from a place that has it to a place that wants it, so
every place stands alone. A shortage no relief can reach is a fact about the
map, not a situation, and a mountain pass that nothing travels is terrain and
not a route.

This item lets a quantity move between places over a network. It is the work
PRD-0010 asks for, and until now no item cited that record at all.

## What the work does

1. The engine holds a network of places and the links between them. A link
   carries a cost that the ground and the improvements on it set.
2. A solver moves a quantity from where it is plentiful toward where it is
   scarce, over the whole network at once, for a fixed number of iterations.
3. The quantity is conserved exactly. A flow that splits and rejoins arrives
   whole.
4. Blocking a place changes what arrives at the places behind it.
5. A property test asserts conservation over a run, and a determinism test
   asserts an identical result at 1, 2 and 12 threads.

## Impact review

**Governed by.** ADR-0002 D1 forbids a floating point quantity, so a flow and
a link cost are integer or Q16.16 values.[^1] ADR-0004 D4 requires a stable
sort key, so nothing in the solver is ordered by thread completion order.[^2]
ADR-0062 holds production and upkeep as rates attached to a site, and a site
is the place a flow starts from and arrives at.[^3]

**Changes.** None known. State any that the refinement finds.

**Creates.** Three rows are reserved and hold no file: ADR-0061 states that
trade solves a flow and never a path for each cart, ADR-0058 states that a
field update is a flux pair on an edge so quantity is conserved exactly, and
ADR-0049 states that a quantity is a rate, a constraint or a set.[^4] The
refinement decides how many of the three this work writes. Do not choose a
number; the registry allocates it and these three are already allocated.

**Blockers.** BLK-007 governs the cost figures this item would state, so it
states none.[^5]

**Precedent.** FND-049 records that the term which grows with the number of
things dominates the term that grows with the number of tiles.[^6] A solver
that searches a path for each thing that moves puts the moving quantity into
the dominant term, and PRD-0010 rejects that shape by name.

**Serves.** PRD-0010.[^7] It does not give a good a value, and it does not
show a flow to a watcher. Item 0106 holds the second of those.

**Conflict surface.** Unknown until the network representation is decided.

## What is missing before this is refined

**The network.** What a node is has no answer. A settlement, a level 1 cell
and a tile are all candidates, and the choice sets the cost of every tick that
solves the flow. Answer it in the impact review, with the storage figure that
follows, and not during the work.

**The value.** PRD-0010 asks that a good have a value that responds to what is
available and what is wanted. Whether that value is a product of this solver
or a separate quantity is undecided. Decide it before refining, and write a
third item if it is separate.

**The dependency order.** The reserved rows say that ADR-0061 depends on
ADR-0049 and ADR-0058, and neither of those has a file.[^4] Refining this item
means saying which of the three is written first.

## Done when

- A quantity moves between two places without a unit carrying it, and the
  direction follows from where the quantity is scarce.
- A conservation property test balances to zero over many solver passes.
- Blocking a place changes what arrives, and a test asserts the change.
- The solver runs a fixed iteration count. No convergence test and no time
  budget appears in the code.
- A determinism test asserts an identical result at 1, 2 and 12 threads.
- The fixture produces a real scarcity gradient rather than a uniform world,
  and the commit body says how that was checked.[^8]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR-0062, production and upkeep are rates attached to a site. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^4]: ADR Registry, rows 0049, 0058 and 0061. `docs/adrs/REGISTRY.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register, FND-049. `docs/FINDINGS.md`
[^7]: PRD-0010, a good moves to where it is wanted. `docs/product/accepted/prd-0010-a-good-moves-to-where-it-is-wanted.md`
[^8]: Findings register, FND-051. `docs/FINDINGS.md`
