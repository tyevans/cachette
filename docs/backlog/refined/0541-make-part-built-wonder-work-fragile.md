---
id: 0541
title: Make part-built wonder work fragile
status: refined
created: 2026-09-10
implements: [ADR-0001 D4, ADR-0004 D1, ADR-0023 D1, ADR-0151 D4, ADR-0164 D1, ADR-0174 D1]
changes: [ADR-0150 D4]
creates: [ADR-0206, DEC-284, FND-765]
serves: [PRD-0057]
blocked-by: []
---

## Why

In a game of three factions, a trained policy wins about a third of its games
with one tactic. It puts every unit on one wonder from the first tick, and it
does nothing else. The built-in controllers grow large and let it finish.

Nothing a rival does undoes a part-built wonder today. A rival that kills the
builders stops the work, and the work stays. A capture gives the part-built
wonder to the taker. A raze removes only the upgrade on the tile of the
settlement. Work that is not finished never wears.

The product record asks that a player who chooses one way to win can lose to a
player who chooses another.[^1] A wonder that no rival can undo is a way to win
that nothing contests. The project owner approved the change on 10 September
2026: part-built wonder work becomes fragile.

## Impact review

**Governed by.** ADR-0174 D1 makes a finished wonder a win path, and its reader
reads a standing claim.[^2] This item does not change a finished wonder.
ADR-0151 D4 forbids a pass that names a category, so the rule reads the victory
claim column of the row above the entry.[^3] ADR-0150 D4 stops a build whose
ground changes hands, and it keeps the progress in the entry.[^4] ADR-0090 D4
returns a tile to the generated world when its entry goes.[^5]

The determinism records govern the pass. ADR-0001 and ADR-0164 put every stored
value the step reads into the state hash.[^6] [^7] ADR-0004 D1 fixes the order
of the walk, and ADR-0023 D1 and ADR-0002 D1 keep every term a whole
number.[^8] [^9] [^10]

ADR-0180 and ADR-0181 hold the capture, the raze and the elimination, which are
three of the paths that change the holder of a tile.[^11] [^12] ADR-0204 D4 and
D5 hold the reading of each win path and the bar of each path.[^13]

**Changes.** The last paragraph of ADR-0150 D4 stops being true for wonder
work. ADR-0206 states the change. ADR-0150 is not edited by this item.

**Creates.** ADR-0206, whose registry row is allocated. DEC-284 holds the rate
and the reset rule. FND-765 records the belief that a rival could contest a
wonder.

**Blockers.** BLK-007 governs the decay rate, because the rate is a cost in
work.[^14] BLK-050 governs every rule of the downstream game. The rate is a
balance row with a provisional value, and the record states none.[^15]

**Precedent.** FND-011 records that an accumulator that nothing clamps banks a
surplus that reaches the state hash.[^16] BLK-036 resolved that an upgrade
changes hands with the ground.[^17] That answer still holds for a standing
upgrade and for work toward a row that carries no claim.

## Done when

- Wonder work that no builder works loses the stated decay on each tick.
- Wonder work that a builder works on a tick loses nothing on that tick.
- A capture, a raze and ground that falls to nobody each reset the work.
- A finished wonder keeps its level whatever happens to its ground.
- A part-built upgrade of any other category keeps its progress.
- The event log and the state hash of a world in which a wonder decays are
  identical at one, two and twelve threads.
- Each defect put back turns a named test red.
- The golden state hash is regenerated from the merged source, because the
  change moves the world state.
- The whole check command runs green.

## Outcome

Part-built wonder work is fragile. The item stays in `refined/` until the
integrator runs the gate and regenerates the golden state hash. Then it moves to
`complete/`.

**What was done.** The rule reads the victory claim column of the row above an
entry, so in the default table it touches the wonder alone.[^3] Each entry
stores the holder that its work belongs to. A new serial pass runs after the
last stage that writes the holder column. It resets wonder work whose holder
changed, and it takes the decay from wonder work that the build did not advance
on that tick. An entry at no level that reaches nothing is removed.

**What changed from the plan.** The decay is a value of the upgrade table and
not a column of a row, because only a row with a claim reads it. A world setter
writes it, and the table hash covers it.

**What was left undone.** No binding exposes the decay setter to Python. No
event marks a reset or a removal of wonder work. ADR-0150 is not edited, because
another owner holds it. ADR-0206 states the clause it changes.

**Registers.** DEC-284 is closed. FND-765 is recorded. The balance register holds
the wonder decay row. No blocker opened or closed.

## References

[^1]: PRD-0057, more than one way to win decides a game. `docs/product/shaped/prd-0057-more-than-one-way-to-win-decides-a-game.md`
[^2]: ADR-0174, a wonder is a win path and a stock total is not, decision D1. `docs/adrs/draft/adr-0174-a-wonder-is-a-win-path-and-a-stock-total-is-not.md`
[^3]: ADR-0151, an upgrade is a category with a ground fit and a level, decision D4. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^4]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D4. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^5]: ADR-0090, a tile upgrade is stored sparsely, decision D4. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^8]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: ADR-0180, a site changes hands or the taker destroys it. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^12]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
[^13]: ADR-0204, every win path holds a bar of its own, decisions D4 and D5. `docs/adrs/draft/adr-0204-every-win-path-holds-a-bar-of-its-own.md`
[^14]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^15]: Balance register, the wonder decay. `docs/reference/balance.md`
[^16]: Findings register, FND-011. `docs/FINDINGS.md`
[^17]: Blockers register, BLK-036. `docs/BLOCKERS.md`
