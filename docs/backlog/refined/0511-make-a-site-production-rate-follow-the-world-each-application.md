---
id: 0511
title: Make a site production rate follow the world each application
status: refined
created: 2026-09-06
implements: [ADR-0062 D1, ADR-0062 D2, ADR-0062 D7, ADR-0143 D1, ADR-0164 D2]
changes: []
creates: [ADR-0055]
serves: [PRD-0007, PRD-0008]
blocked-by: []
---

## Why

A site sets its production rate exactly once. The founding path reads the food
the survey measured and writes the rate, and nothing reads the ground
again.[^1] A site founded on good land therefore produces the same amount on
tick nineteen thousand as on tick one, whatever happened to the land, the
weather, the upgrades or the people.

The project owner named the problem. There is not enough balance in the world,
and everything is saturated all the time. A quantity that reaches a value early
and holds it makes no interesting world, and the store is one of several.

The store climbs to its clamp in every seed, and the wealth path ends 32 of 32
games because of it.[^2] A source with no sink can only climb.

## What the work does

1. The stored rate stays the base rate and stays on the site.[^3] Nothing about
   the founding rule changes.
2. An ordered pipeline turns the base rate into an **effective rate** each time
   the rate pass applies. Four terms compose, each a signed share of one, added
   to a base of one and clamped.
3. The four terms read the ground the site reaches, the moisture over it, the
   upgrades standing on it, and the people who live in it.
4. The effective rate is **derived**, so it does not enter the state hash. Its
   four inputs are stored and are already in the hash.[^4]
5. No stage is added. The rate pass builds the derived table and hands it to
   the same apply function, at the same point in the frame.

## Impact review

**Governed by.**

- **ADR-0062 D1.** A rate belongs to a site and never to a unit. The pipeline
  reads the ground of the site's own disc and writes a rate for that site. The
  cost still follows the site count.
- **ADR-0062 D2.** Every rate is a Q16.16 value at or above zero. The clamp
  floor of the pipeline is above zero, so the effective rate never goes below
  it, and upkeep is untouched.
- **ADR-0062 D3, D4, D5, D6.** The apply function, the schedule, the position
  in the frame and the reduction are all unchanged. The work replaces the table
  the apply function reads and changes nothing else about it.
- **ADR-0062 D7.** This is the record's own condition. It says a second
  modifier source needs the pipeline record, and it names ADR-0055 as the
  reserved row. This work adds four sources, so the record is a deliverable.
- **ADR-0143 D1.** Wet ground yields more. The moisture term takes the same
  reader that the gather resolve takes, so the project gets one moisture reader
  and not two.
- **ADR-0164 D2.** A derived value stays out of the hash and its inputs enter
  instead. The weights are compile-time constants and not stored parameters, so
  D1 and D3 add no hash line.
- **ADR-0002 D1 and D2.** Every term is Q16.16 through the arithmetic module.
- **ADR-0004 D1.** The pipeline visits the disc in the fixed disc order and
  composes the four terms in a stated order.

**Changes.** No record changes. ADR-0062 keeps every decision it holds.

**Creates.** ADR-0055, an effective stat comes from an ordered modifier
pipeline. The registry reserves the row.[^5] The claim passes the three
condition test: a contributor could reasonably compose the sources by
multiplication instead of by addition, the composition rule is expensive to
change once other subsystems read an effective value, and the reason for
addition over multiplication is not visible in the code.

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-130 asks what weather is worth, and the moisture term is written against it
rather than against a measurement.

**Precedent.** FND-012 records that a fixed-point multiply carries a permanent
downward bias below zero. The pipeline composes by addition and multiplies
once, and both operands of that multiply are above zero.

**Not in this work.** **Temperature.** A separate worker is adding temperature
to the weather lattice now, and it is not merged. That worker states plainly
that the field is not built and its range is not measured, and asks that no
weight be written against a number nobody has taken. Writing one would invent a
value, which this project's registers forbid. Item 0512 holds the term.[^6]

**Conflict surface.** A new module, and one function in `world.rs`. `fn step`
is not touched, because the pass keeps its position and its stage.

## Done when

- The rate the apply function reads is recomputed each application from the
  ground, the moisture, the upgrades and the people.
- The stored base rate is unchanged by the pass, and the founding rule still
  holds: a full site on untouched dry ground with no upgrades produces its base.
- Four tests, each with a fixture at the extreme rather than a copy of the
  demonstration world: production falls as the ground is drawn down and
  recovers as the ground recovers; production rises when the ground is wet and
  falls when it dries; production rises when a terrace completes; a site on
  poor ground never matches a site on good ground.
- Each of the four defects is put back and the test goes red.
- The effective rate is derived and no new field enters the state hash.
- The thread-equivalence test passes at 1, 2 and 12 threads.
- The golden files are recorded again, because the store trajectory moves.
- ADR-0055 is written and the registry row reads `Draft`.
- The balance register holds one row for each weight, each with a derivation.

## References

[^1]: Backlog item 0136, provision a founded site from the ground it reaches. `docs/backlog/complete/0136-provision-a-founded-site-from-the-ground-it-reaches.md`
[^2]: Findings register, FND-543. `docs/FINDINGS.md`
[^3]: ADR-0062, production and upkeep are rates attached to a site, decision D1. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^4]: ADR-0164, every stored value the step reads enters the state hash, decision D2. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^5]: ADR Registry, row 0055. `docs/adrs/REGISTRY.md`
[^6]: Backlog item 0512, add the temperature term to the production pipeline. `docs/backlog/proposed/0512-add-the-temperature-term-to-the-production-pipeline.md`
