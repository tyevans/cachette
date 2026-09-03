---
id: 0305
title: Give a laden unit a reason to go home
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0007]
blocked-by: []
---

## Why

**The delivery of a carried load works and never runs.** A unit gives its load
to the store of its site only while it stands on the tile of that site, and no
rule in the engine ever puts it there on purpose. A unit gathers where it
stands and then steps wherever the exit field of its cell points.

The four options are `roam`, `forage`, `climb` and `join`. Each ranks a
neighbouring cell on a summary field. **None of them is "go home".**

A run of the demonstration world for 4000 ticks delivered nothing of any kind.
The findings register holds the measurement.[^1]

Until this exists, what a settlement holds is the rate the founding set from
the survey and nothing else, and the whole economy is decided before the first
frame.

## The question this item has to answer

**A site is not a summary field.** The exit field ranks a neighbour on a value
that a cell carries, and one unit's own site is not a fact that a cell
carries.[^2] So a fifth option that reads a cell field cannot express "my
home", and the shape that would express it is a per-unit search, which the
movement record refuses.[^3]

At least three shapes could answer it, and choosing between them is an
architectural decision that needs a record.

1. **A cell field that says how much of the ground here belongs to the unit's
   faction.** The influence field already solves over the same lattice, so a
   laden unit could climb it. It steers a unit toward its faction rather than
   toward its own site, which may be near enough.
2. **A carry threshold that changes the option.** A unit whose load is full
   switches to an option that ranks on that field. This keeps the option set as
   the one place behaviour is declared.
3. **A per-unit direction toward a stored home address.** It answers exactly
   and it is the shape ADR-0091 D1 refuses on cost.

## Refine this by answering

- Which of the three shapes, and what the record that holds it says.
- What a unit with no home does, since a unit that belongs to no site draws
  from nothing and can deliver to nothing.
- Whether a laden unit stops gathering, and where that rule lives.
- What it costs at the target scale, given that the movement pass runs for
  every unit of the population.

## References

[^1]: Findings register, FND-317. `docs/FINDINGS.md`
[^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
