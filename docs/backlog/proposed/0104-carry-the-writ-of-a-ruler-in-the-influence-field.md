---
id: 0104
title: Carry the writ of a ruler in the influence field
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: [ADR-0060]
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

## What is missing before this is refined

- **The storage question, which is the whole record.** A registry row is
  allocated for the claim that an influence map is stored as a shared basis
  rather than one plane for each faction.[^5] The row is `Proposed`. Read the
  research that supports it and settle the claim before this item is
  refined.[^6]
- The impact review. The records that govern the level 1 pass and the solver
  have not been read against this work, so the item cannot name them by
  decision.
- The arithmetic. Every value in the field is fixed-point, and the scale and
  the saturation rule at the top of the range are not settled.
- The dependency. Whether this item needs the level 1 pyramid work that other
  items hold is not worked out.

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
- The fixture holds open ground, resistant ground, a source at an edge and a
  source at the centre. The commit body says how that was checked.[^7]
- ADR-0060 is written, the registry row moves to `Draft`, and the record holds
  no iteration count, no cell count and no cost figure.
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-040. `docs/DECISIONS.md`
[^2]: Decisions register, DEC-005. `docs/DECISIONS.md`
[^3]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^4]: Decisions register, DEC-041. `docs/DECISIONS.md`
[^5]: ADR Registry, row 0060. `docs/adrs/REGISTRY.md`
[^6]: Influence maps. `docs/research/reports/09-influence-maps.md`
[^7]: Testing Rules, section 2a. `.claude/rules/testing.md`
