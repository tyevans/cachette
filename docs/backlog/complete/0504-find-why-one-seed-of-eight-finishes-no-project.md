---
id: 0504
title: Find why one seed of eight finishes no project
status: proposed
created: 2026-09-05
implements: [ADR-0152 D3, ADR-0159 D3]
changes: []
creates: []
serves: [PRD-0055]
blocked-by: []
---

## Why

**Seed `0x1` finishes no project in a run where seven other seeds finish
several.** Eight seeds were written down before any of them ran, and each ran
for 800 ticks of the demonstration world. Seven of the eight finished between
one and thirteen projects. The eighth finished none, both before and after the
repair that closed the road chain. The findings register holds the table and
the command that produced it.[^1]

**A behaviour that holds for seven cases and fails for the eighth is a defect,
not a distribution.** The seven that work are the reason nobody looks. The one
that fails is the input that a fixture built from the typical world never
supplies, and the register already records that shape twice.[^2] The world this
seed builds is cheap to inspect today, because the run is one command and the
reading takes minutes. It is expensive once a learner trains on the chain,
because a seat that never sees a finished project learns from a world that does
not close.

**This is a separate item, and not a question inside the plan repair.** Three
reasons. The plan repair touches `fn step`, so one worker holds it at a time,
while this reading writes no engine code and anyone may run it beside that
work.[^3] The two have different outputs: the plan repair ships a behaviour and
this ships a finding and, if a defect is found, an item that names it. Folding
the question into the plan repair would also hide it, because a repair that
makes the other seven better and leaves this one at zero would still read as a
success against every count the plan repair states.

## What is missing before this is refined

A refiner answers these before this item leaves `proposed/`.

- **What this seed's world holds that the other seven do not.** State the
  ground, the founding places and the distance between them. A road project
  joins two of the faction's places along one path, so a world whose places sit
  on ground that no path crosses zones a project that no unit reaches.[^4]
- **Whether the run zones anything at all.** A run that zones nothing and a run
  that zones and finishes nothing are two defects. Read the plan counts of this
  seed apart from the finished count.
- **Whether the cause is the same one the plan repair addresses.** The plan
  saturates in every seed. Say whether this seed fails only because of the
  saturation, or whether a second cause sits under it.[^3]
- **Whether the tick limit is the cause.** A slower world may finish a project
  after the bound of the run. Say what this seed does over a longer run, and
  whether the answer changes.
- **What the outcome is.** A defect gives a new backlog item and a finding. A
  world that is merely slow gives a finding alone, and the refiner says what
  the seeded test should then assert.
- **Which seeds the chain test keeps.** The test seeds the demonstration world
  at eight seeds and asserts a stated number of them finish a project. Say
  whether this seed stays in that set and what it then asserts.[^1]

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, the road chain finished nothing in a run. `docs/FINDINGS.md`
[^2]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^3]: Backlog item 0502. `docs/backlog/proposed/0502-let-a-faction-re-aim-its-project-order-and-keep-its-plan-live.md`
[^4]: ADR-0152, a faction plans its roads and zones with one solver, decision D3. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
