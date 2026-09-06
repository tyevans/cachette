---
id: 0508
title: Give a faction store a cost that scales with what it holds
status: complete
created: 2026-09-06
implements: []
changes: []
creates: []
serves: [PRD-0024, PRD-0053]
blocked-by: [BLK-050]
---

## Why

**The store of a faction rises from zero to its clamp and stays there, and the
rise is a straight line.** A sweep of 16 seeds of the demonstration world, each
played to 20000 ticks whatever the game end said, measured the store of every
faction on every twentieth tick. The findings register holds the table.[^1]

**The rise is straight because both ends of the flow are constants.** The
founding sets the production rate of a site from the food its survey read, and
no stage re-reads the ground afterward, so a site earns the same quantity on
every application for the life of the run. The people who consume are bounded
by the founding housing of 16 and no stage raises housing, so consumption stops
growing by tick 500.[^2] A constant source above a constant sink gives a
constant net, and a constant net fills any store.

**The sink that would answer this exists and nothing sets it.** A site holds an
upkeep rate beside its production rate, the rate pass subtracts it, and a
shortfall is a typed event with its own ledger. No stage of the engine writes a
non-zero upkeep rate. Only a Python caller can. The mechanism is complete,
tested and inert.[^3]

**Item 0506 does not answer this.** That item changes the reader that ends a
game on the stock. It leaves the store at its clamp, so a watcher still sees a
bar that fills early and never moves, and every value the store feeds is still
pinned. This item is about the flow and not about the win condition.

## What the work does

1. Give a faction a cost that rises with what it holds, so that a store settles
   at a level rather than reaching a bound. Any of these is a candidate and the
   refinement chooses one: a proportional upkeep, a cost that follows the units
   a faction fields, or a store that spoils a share of itself each tick.
2. Write the rate through the existing upkeep column, so the shortfall event
   and the ration path get their first engine caller.
3. Put the rate in the balance register with its derivation.

## What is missing before this is refined

- **Which of the three costs.** A proportional upkeep gives an equilibrium and
  no drama. A cost that follows fielded units couples the economy to the army
  and gives a faction a reason to disband. Spoilage punishes hoarding and gives
  a reason to trade. They are different games.
- **Whether the production rate should follow the ground.** A site whose rate
  is fixed at founding cannot answer a cost by working harder. If the answer to
  this item is a cost alone, a poor site drains to zero and starves with no
  move available to it.
- **Which records govern it.** ADR-0062 states that production and upkeep are
  rates attached to a site, and a rate that scales with a stock may not be a
  rate in that record's sense.

## Impact review

Not done. This item is in `proposed/` and refining it is the work.

## Outcome

Complete. A faction store now costs a share of itself to keep, so it settles
instead of climbing to its clamp.

**What was built.** The effective rate module derives an upkeep as well as a
production. Two terms add to the stored base upkeep: a holding term that is a
share of what the store holds now, and a resident term that is a share of the
ration each resident eats. The stored upkeep stays the base and stays settable
by a caller. Neither term is a negative production rate.

**Why the store settles.** With a production of P a tick and the holding share
h, the store settles at the net production divided by h. A site that earns more
settles higher, and a site whose ground is drawn down falls back. The recurrence
is monotone and it converges at the default period.

**The resident term reads the ration of the need rule** rather than restating
it, so the food a person costs keeps one declaration site.

**The record.** ADR-0055 states how the terms compose. It is the record that
ADR-0062 D7 reserved, and D7 named the exact condition that fired here.

**Left open.** Both terms apply to every commodity, and the commodity count is
one, so this states no rule about which good a person needs.

**Fallout the item created.** Every test that sets a production rate, steps and
asserts a store quantity now reads a different number.

## References

[^1]: Findings register, FND-549. `docs/FINDINGS.md`
[^2]: Balance register, the founding housing and the production queue.
`docs/reference/balance.md`
[^3]: ADR-0062, production and upkeep are rates attached to a site.
`docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
