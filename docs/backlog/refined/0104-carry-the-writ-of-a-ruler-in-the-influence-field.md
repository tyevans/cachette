---
id: 0104
title: Carry the writ of a ruler in the influence field
status: refined
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0002 D2, ADR-0004 D1, ADR-0009 D1, ADR-0009 D3, ADR-0023 D1, ADR-0023 D2, ADR-0053 D3]
changes: []
creates: [ADR-0060, ADR-0087]
serves: [PRD-0016]
blocked-by: []
---

## Why

The world holds no influence field. A faction marks a unit and it marks ground,
and nothing carries a faction's reach across the map.

PRD-0016 needs a faction to be somebody, and it rejects the shape where a unit
follows a link to its faction and then to that faction's ruler. The replacement
is the field: a ruler contributes a source term, and a unit reads the level 1
cell it already gathers.[^1] Item 0068 writes that source term. This item
builds the thing it writes into.

Separating them is deliberate. A field is a structure with a storage question
and a solver. A ruler is a rule that writes into it. Building both in one
change would make one pull request that touches the level 1 pass and the
character tier together.

## What the work does

1. A faction has an influence field over the level 1 cells.
2. A source term raises the field at a cell, and the solver relaxes it outward
   over a fixed iteration count.
3. The relaxation carries terrain conductance, so influence flows around
   ground that resists it rather than through it.
4. A unit reads its cell as one more gather. It runs no extra indirection.
5. A field with no source term relaxes toward nothing over the same fixed
   iteration count. No pass branches on the absence of a source.

## The answers this item takes, stated plainly

**Terrain conductance is in.** The project already chose the plane with
conductance, and it chose it against a cost that is derived and local rather
than measured on the target.[^2]

**The solver runs a fixed iteration count.** It runs no convergence test and no
time budget, because both make the result depend on the machine.[^3]

**No branch for an absent source.** An absent ruler is an absent source term,
and the engine holds no branch for it.[^4] The field relaxes from the edge
inward, so a far province loses its hold before the seat does. That behaviour
is the solver, not a rule, and this item must not add a rule that produces it.

## Impact review

**Governed by.**

- **ADR-0022 D1, D2 and D3.** Level 0 is the only truth, no level above it
  holds a fact of its own, and no system writes above level 0.[^5] **This
  work stands against D1 as that decision is worded, and the conflict is
  named rather than hidden.** An influence plane carries the state of a
  solver from one tick to the next, so it holds a value that appears nowhere
  at level 0. The plane is not a summary: it does not claim that a cell
  equals the combination of the tiles it covers, and it therefore makes no
  claim D2 governs. ADR-0087 states the boundary it draws, and an open
  choice asks a reviewer to settle whether D1 needs a clarifying
  amendment.[^6]
- **ADR-0002 D1 and D2.** No floating point in simulated or aggregated
  state, and simulation arithmetic goes through the arithmetic module. Every
  cell of the field is an exact unsigned integer, and the kernel divides by
  a truncating shift.
- **ADR-0004 D1.** Iteration order is explicit. The solve visits the cells
  in ascending cell index, and every parallel result is named by its cell
  rather than by the thread that wrote it.
- **ADR-0009.** Parallel stages write disjoint outputs. A pass reads the
  whole of one plane and writes a contiguous run of another, so no two
  threads write one cell and no atomic operation appears.
- **ADR-0023 D1 and D2.** An aggregate combines exactly, in any order.
  Saturating unsigned addition at the cell ceiling is exactly associative
  and commutative, and it is what combines a source with a relaxed value.
- **ADR-0053 D2.** A faction is a bit in a mask. The field is indexed by the
  faction identifier and holds no second name for a faction.

**Changes.** No record changes. ADR-0022 is accepted, and this work does not
edit it. The conflict above is carried to a reviewer as an open choice.

**Creates.** ADR-0060, the storage shape, and ADR-0087, the fixed iteration
count. Both rows were moved to `Draft` before the records were written.[^7] The
author is not the reviewer of either.

**Blockers.** BLK-007 governs every cost figure in this work, because no
measurement exists on the target platform. The solve therefore states no
budget, carries no cadence chosen against a figure, and holds no cost
comment in the code. The cadence that the research recommends is a separate
item.[^8] [^10]

**Precedent.** FND-141 records that a one-byte tile field over the target
scale does not overflow a 32-bit accumulator, and that the margin is small
rather than absent. Nothing here depends on that margin: every accumulator in
the solve is wider than the field it sums.

## Done when

- A faction holds an influence field over the level 1 cells, and a watcher
  reads a cell through the public interface.
- A source term at a cell raises the field there and, after the solve, at the
  cells around it. A test asserts the falloff.
- Ground that resists influence obstructs it. A test places a source on one
  side of resistant ground and asserts that the far side holds less than an
  equally distant cell on open ground.
- The solver runs a fixed iteration count. No convergence test and no time
  budget appears anywhere in it, and a test asserts the iteration count is
  reached whatever the input.
- A faction with no source term produces a field that relaxes toward nothing,
  and no pass tests whether a source exists. A test asserts that removing the
  source makes the far cells fall before the near ones.
- Every value in the field is an exact integer or a fixed-point value, and the
  arithmetic goes through the arithmetic module.
- A property test asserts that the field is identical, cell for cell, at 1, 2
  and 12 threads.
- The perturbed build makes the iteration count depend on a convergence test,
  and each test that the perturbation defeats fails under it. The probe
  binary asserts that every perturbation is visible.[^9]
- The fixture holds open ground, resistant ground, a source at an edge and a
  source at the centre. The commit body says how that was checked.[^9]
- ADR-0060 and ADR-0087 are written, both registry rows read `Draft`, and
  neither record holds an iteration count, a cell count or a cost figure.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-040. `docs/DECISIONS.md`
[^2]: Decisions register, DEC-005. `docs/DECISIONS.md`
[^3]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^4]: Decisions register, DEC-041. `docs/DECISIONS.md`
[^5]: ADR-0022, level 0 is the only truth, and every level above it is derived. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^6]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^7]: ADR Registry, rows 0060 and 0087. `docs/adrs/REGISTRY.md`
[^8]: Backlog item 0169. `docs/backlog/proposed/0169-choose-the-cadence-of-the-influence-solve.md`
[^9]: Testing Rules, sections 1 and 2a. `.claude/rules/testing.md`
[^10]: Influence maps, section 7. `docs/research/reports/09-influence-maps.md`
