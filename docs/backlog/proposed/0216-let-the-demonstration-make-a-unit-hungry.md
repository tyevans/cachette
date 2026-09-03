---
id: 0216
title: Let the demonstration make a unit hungry
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: [PRD-0009]
blocked-by: []
---

## Why

**No unit in the demonstration ever forages, and no unit ever gathers.** The
engine steers a step by the option a unit chose, and it writes a gather order
for the unit that chooses to forage. Both work. Neither runs in the
demonstration, because the demonstration founds each group on a site whose
store feeds it. A unit that holds a whole need scores zero for the option that
is driven by what it lacks, whatever the ground carries.[^1]

The measurement is in the findings register. Over one thousand two hundred
ticks of the demonstration world, the mean need never fell below the full
need, every live unit held the `roam` option, no unit held a gather order, and
the depletion ledger held no entry.[^2]

**This costs the demonstration the loop and not one behaviour.** Food falls
where a crowd stands, the summary of the cell falls with it, and the exit field
then turns the crowd away. Items 0185 and 0186 built that chain in the
engine.[^3] [^4] A watcher sees none of it, and the product record asks that a
watcher changes the world and sees the behaviour change.[^5]

**What a watcher does see is a migration toward open ground.** The `roam` row
reads the share of a cell that admits a unit, which is a property of the ground
that no system writes, so the field never changes and the population walks to a
local maximum and settles. The step is directed rather than random: over three
hundred ticks the mean distance from the starting tile rose from 13 tiles under
the uniform draw to 36. That figure is one run on a development machine.[^6]

## What the work does

The work makes the demonstration produce a hungry unit, and it leaves the
engine alone. The shape is a parameter of the run and not a rule of the
simulation, so it belongs where the demonstration builds its world.

Three parameters could do it, and the item does not choose between them yet.

1. **Found a group larger than its store feeds.** The consumption pass rations
   a store that cannot serve every cohort, and the demonstration already
   reports a shortage.
2. **Give the site no production rate, or one below the ration.** The store
   then empties over a run rather than at the first frame.
3. **Raise the decay of the need rule.** This is the parameter the engine
   tests use, and it is the bluntest of the three.

**A run must not end the population.** The need rule holds a bound, and a unit
whose deficit reaches it is ended by the death scan. Whatever the work chooses,
it must leave a watcher a world that still holds people after a long run, and
it must say what it measured.

## Impact review

**To be done before the work starts.** The item is `proposed`, and it names no
governing record yet. The reviewer should read the record that states what a
need is, the record that states how production and upkeep attach to a site, and
the record that states that the viewer reads the world and never writes to
it.[^1] [^7] [^8]

**The file this touches is the demonstration binary, and it is not the
engine.** Nothing here changes a rule. A change that needed one would be a
different item.

## Done when

- A run of the demonstration produces a unit whose need falls below the
  threshold, and the number of ticks that takes is stated.
- A run produces gather events, and the depletion ledger holds entries.
- The population is still alive after a long run, and the run length is stated.
- A watcher who runs the window sees units walk toward food and work a deposit
  down. Whoever runs it says what they saw.
- The open choice about whether a cell that steps as a block reads as a crowd
  is answered against that run, or the row records why it still is not.[^9]

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^2]: Findings register, FND-209. `docs/FINDINGS.md`
[^3]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/complete/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^4]: Backlog item 0186, let the engine order a gather. `docs/backlog/complete/0186-let-the-engine-order-a-gather.md`
[^5]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: ADR-0062, production and upkeep are rates attached to a site. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^8]: ADR-0067, the viewer reads the world and never writes to it. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^9]: Decisions register, DEC-079. `docs/DECISIONS.md`
