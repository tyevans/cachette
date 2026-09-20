---
id: 0514
title: Let a faction found a city during a run
status: refined
created: 2026-09-06
implements: [ADR-0150 D2, ADR-0150 D5, ADR-0144 D2, ADR-0148 D3]
changes: [ADR-0150]
creates: []
serves: [PRD-0053, PRD-0012, PRD-0054]
blocked-by: [BLK-050]
---

## Why

The set of cities is fixed at seeding for a whole run. No stage of the step
founds a settlement, and no stage destroys one. A city can fall or be razed,
but no faction can found a second city during a run. The city set therefore
only shrinks over time.[^1] [^7] [^8]

This leaves factions unable to expand or invest accumulated resources into new
territory. A faction's held ground remains bounded by its initial city reach
plus unit leases.[^4] Allowing a faction to build a settler unit, send it to an
unheld location, and found a secondary settlement expands the living world,
unlocks economic growth, and drives emergent territorial competition across
factions.[^2]

## Impact review

**Governed by.** ADR-0150 D2 provides that held ground is the ground within
reach of an owned city. ADR-0150 D5 specifies the settler verb and confirms that
founding consumes the settler.[^5] ADR-0144 D2 requires that the controller act
only through caller verbs. ADR-0148 D3 defines the domination win condition
based on the faction's primary seat tile.[^6]

**Changes.** ADR-0150 is amended to specify secondary city founding rules:
1. The primary seat of a faction (the founding tile recorded at tick zero) is
   immutable. A secondary founding creates a new settlement entity and opens
   site rows, but does not move the faction's seat.
2. Sponsoring a settler unit or founding consumes store food or goods from the
   parent site according to the balance parameter.
3. The new city projects an additional reach disc, uniting or expanding the
   faction's held ground.

**Creates.** None.

**Blockers.** BLK-050 governs downstream game rules and parameters. Founding
cost and distance thresholds are parameterized in the balance register rather
than hard-coded.

**Precedent.** FND-542 showed that thirty-two seeds ended with at most four
cities and never increased. FND-483 showed the necessity of visible census
counters when positions or sites open.

**Conflict surface.** `crates/cachette-core/src/controller.rs`,
`crates/cachette-core/src/world/controller.rs`, and
`crates/cachette-core/src/world/founding.rs`.

## Done when

- The faction controller queues a settler unit when a site holds required store
  surplus and unheld eligible land exists within survey reach.
- A settler unit moves toward the survey target and executes `Verb::Settle`.
- The settle verb founds a new settlement entity in the settlement arena and
  consumes the settler unit.
- The faction's primary seat remains unchanged on secondary founding,
  preserving domination win path invariants.
- The new settlement opens position, rate, and queue slots, and expands the
  faction's reach disc.
- A test verifies that a faction successfully founds a second city during a run
  and that the world state updates deterministically.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-542. `docs/FINDINGS.md`
[^2]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^4]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^5]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^6]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^7]: ADR-0180, a site changes hands or the taker destroys it. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^8]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
