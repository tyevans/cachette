---
id: 0342
title: Let the control plane name the seed set of a strategy field
status: complete
created: 2026-09-03
implements: [ADR-0091 D1, ADR-0091 D6, ADR-0095 D1, ADR-0095 D3, ADR-0110 D2, ADR-0125 D1, ADR-0125 D2, ADR-0125 D3, ADR-0125 D4]
changes: []
creates: [ADR-0125]
serves: [PRD-0020]
blocked-by: []
---

## Why

The engine derives a reach for each faction and each level 1 cell, seeded at
every live site of that faction, and a laden unit climbs it home. The core
answers that direction today, so the mechanism is built rather than
proposed.[^1]

**The seed set is fixed by the record.** So a unit goes to a site of its faction
and nowhere else. A developer cannot send a set of units to a mountain, to a
frontier, or to a place a player chose, because none of those is a site.

Naming a set of addresses that seeds a plane gives two of the six things a
downstream game asks for: move units somewhere, and gather units in a place. No
unit gains a search, and the derivation, the relaxation and the tie-break all
stay as they are. The decisions register held the options.[^2]

## Impact review

**Governed by.** Five records govern this work.

ADR-0091 D1 states that movement takes its direction from a per-cell field and
never from a per-unit search.[^3] The work honours it: a sent unit reads one
entry of one plane, indexed by its own cell. It reads no neighbouring cell and
it holds no address.

ADR-0091 D6 states that a direction the ground refuses falls back to a keyed
draw and never freezes a unit.[^4] The work honours it by routing every case
that gives a sent unit no direction into the draw that already exists.

ADR-0095 D1 states that a strategy that names a place arrives as a field over
cells.[^5] ADR-0095 D3 states that several destinations seed one field.[^6] The
work is that shape exactly.

ADR-0110 D2 states the relaxation: the reach in cells, the fixed pass count, the
strict comparison, the lowest direction index as the tie-break, and the refusal
of a cell whose open tile count is zero.[^7] The work shares that relaxation
rather than copying it.

ADR-0005 D1 states that a solver runs a fixed iteration count.[^8] The
derivation runs the same pass count that the return field runs.

ADR-0004 D1 states that iteration order is explicit.[^9] The seed set is sorted
and holds each cell once, and the derivation walks planes and cells in ascending
order.

**Does the work contradict a record?** No. ADR-0110 fixes the seeds of the
return field at the live sites, and the work does not change that field. It adds
a second field with a second seed set, and the record that holds the return
field stays true.

**Creates.** ADR-0125. The scope rule gives three conditions and all three
hold.[^10] A contributor could reasonably give a unit a destination it carries,
which is the first thing anybody writes. Choosing that costs more than changing
it later, because it puts a second source of directions in the engine and no
gate would refuse it. The reasoning is not visible in the code, because a
per-unit bearing looks cheap and correct at the call site.

**Changes.** No record changes.

**Blockers.** BLK-007 governs every cost figure, so no figure appears in the
record and the plane count is a parameter of the world.[^11]

**Precedent.** FND-315 records a unit against a shoreline that stood still for
ever, because a cell exit the ground refuses repeats exactly.[^12] Ordered
movement reproduces that shape four ways, and the work must route all four into
the keyed draw.

**Product record.** PRD-0020.

## What fails if somebody changes it back

Four defects. Each was put back and the suite was watched.

1. Remove the destination branch from the movement pass. A sent unit then takes
   the direction of its own option and never arrives.
2. Read the intent before the destination. A sent unit that has chosen nothing
   then stands still until its cell next chooses.
3. Let a sent unit with no direction stay put rather than draw. A unit beyond
   the reach, a unit the ground refuses, and a unit that arrived all freeze.
4. Stop sorting the seed cells. Two calls that name one set in two orders then
   derive two fields, and the state hash reports two worlds.

## Done when

- The control plane names a set of units and a set of tiles in one call.
- No unit holds an address, a route or a distance.
- Every case that leaves a sent unit without a direction takes the keyed draw.
- The record states the constraint and holds no cost figure.
- The whole check command runs green.

## Outcome

Built. The engine holds one relaxation, and two fields use it: the return field
seeded by the world at its own sites, and the destination field seeded by the
control plane. A unit column holds the plane a unit obeys and nothing else.

Two Python verbs and one Python read cross the boundary. A caller sends a set of
units to a set of tiles in one call, stops the order in one call, and reads the
units of a faction as columns in one call.[^13]

The four defects above were put back one at a time. Every one was caught. The
review holds the output.[^14]

## References

[^1]: Findings register, FND-363. `docs/FINDINGS.md`
[^2]: Decisions register, DEC-142. `docs/DECISIONS.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^5]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^6]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^7]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decision D2. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^8]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-315. `docs/FINDINGS.md`
[^13]: ADR-0125, the control plane names the seed set of a destination field. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^14]: Review of ordered movement and the set read. `docs/reviews/0342-ordered-movement-and-the-set-read.md`
