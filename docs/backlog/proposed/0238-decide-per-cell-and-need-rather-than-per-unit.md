---
id: 0238
title: Decide per cell and need rather than per unit
status: proposed
created: 2026-09-02
implements: [ADR-0096]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The engine computes one answer many times.** The choice pass scores the fixed
option set for each live unit. The score is the unit's drive multiplied by a
weight and by the value the option reads from the unit's level 1 cell.

The engine holds one weight profile for every unit alive, and no unit carries a
type or a profile of its own.[^1] So the inputs to a choice are the unit's cell
and the unit's need, and nothing else. **Two units in one cell with the same need
always get the same answer, and the engine computes it twice.**

The record on cost binds this: the engine computes one answer once for every
reader that would compute the same answer, and a unit reads rather than
computes.[^2]

**The pass has two serial phases as well, and they are the part a thread count
cannot help.** It collects every live unit into one list before any thread
starts, and it applies the results afterwards by walking that list. Both grow
with the population. A finding holds the reading that found them.[^3]

**The interval may become unnecessary, and that is the behavioural prize.** The
choice runs at an interval today because scoring every unit every frame is
expensive. If the expensive part is computed per cell and per need, the reason
weakens. A unit acting on a reading as old as the interval is a behaviour nobody
chose, and removing it would be worth more than the cost saved.

## What the work does

Compute the decision over the lattice, and let a unit read it. Apply results in
the order the lattice produced them rather than by scattering into the columns.

## What is missing before this is refined

- The impact review. ADR-0096 governs it and ADR-0064 governs the choice.
- **How a need becomes a bucket, and what that costs in behaviour.** A need is a
  fixed-point value, so a per-cell answer needs the need quantised. A coarse
  bucket makes two units with different needs act alike; a fine one approaches
  one answer for each unit and saves nothing. This is the design question of the
  item and it is not a detail.
- Whether the bucketing changes any answer. If it does, the golden state hash
  changes and the change has to be stated rather than discovered.
- Whether the interval can be removed, and what that does to the frame under the
  new shape. ADR-0064 D4 decides the interval, so removing it is a record change
  and not a tuning change.
- What happens to the score floor and the mover count, which the register
  already records as a frame-budget parameter rather than a design knob.[^4]
- Whether the same treatment applies to the other unit passes, or only to the
  choice.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-251. `docs/FINDINGS.md`
[^2]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D2 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^3]: Findings register, FND-252. `docs/FINDINGS.md`
[^4]: Findings register, FND-014. `docs/FINDINGS.md`
