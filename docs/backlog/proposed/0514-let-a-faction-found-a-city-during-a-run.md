---
id: 0514
title: Let a faction found a city during a run
status: proposed
created: 2026-09-06
implements: []
changes: [ADR-0150]
creates: []
serves: [PRD-0053]
blocked-by: [BLK-050]
---

## Why

**The set of cities is fixed at seeding for a whole run.** No stage of the step
founds a settlement, and no stage destroys one. The destroy verb has one caller
inside the engine, and that caller is the rollback of a founding that failed.
The controller choice set has no found option. A sweep of thirty-two seeds read
four settlements at tick zero and four at the end of every seed.[^1]

**A city now falls, and this item is the half that is left.** A faction takes
a city that stands undefended under it, and a caller may raze one instead.[^7]
A faction that holds no site and no unit leaves the game.[^8] What no faction
can do is found a second city during a run.

Two things follow from a city set that only shrinks.

**A faction cannot spend a win on more ground.** The only lever a faction has
over its own reach is the upgrades it finishes inside its own ground.[^4] It
cannot plant a second seat where the ground is good.

**The territory path reads a distribution that four discs fix.** The held count
of a faction is its disc, plus what its units lease. A second city would move
the count by far more than a lease does.

## What is missing before this can be refined

- **Whether a faction founds a second city, and what it pays.** A settler verb
  exists and one record states it.[^5] The controller choice set has no found
  option, so nothing in a run calls it. The work must say which store the
  founding spends, and what stops a faction from founding on every tile.
- **What the seat of a second city is.** The seat of a faction is the tile of
  its first founding, and the domination reader reads the holder of each seat
  tile.[^6] The work must say whether a second founding moves the seat. It must
  not move it, because a stored seat that moves makes a stored record resolve
  to the wrong tile.
- **Which records change.** The holding record holds the reach of a city and
  the settler verb, and it holds a decision this item touches.

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-542. `docs/FINDINGS.md`
[^4]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^5]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^6]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^7]: ADR-0180, a site changes hands or the taker destroys it. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^8]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
