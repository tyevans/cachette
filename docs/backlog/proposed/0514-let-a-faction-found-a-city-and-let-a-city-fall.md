---
id: 0514
title: Let a faction found a city and let a city fall
status: proposed
created: 2026-09-06
implements: []
changes: [ADR-0148, ADR-0150]
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

This is the second of the five links that closed the domination path. The first
link is now open: a tile carries a lease, the lease follows the units that
stand on the tile, and a faction takes ground by using it.[^2] [^3] Ground
therefore changes hands by force. **The cities still do not.**

Three things follow from a fixed city set.

**A faction that loses every tile keeps its city and its reach.** The reach of
a city is a disc around its seat, and the disc returns the moment the lease of
the invader decays. A conquest is therefore never final, and the domination
reader asks for a state that no run holds for long.

**A faction cannot spend a win on more ground.** The only lever a faction has
over its own reach is the upgrades it finishes inside its own ground.[^4] It
cannot plant a second seat where the ground is good.

**The territory path reads a distribution that four discs fix.** The held count
of a faction is its disc, plus what its units lease. A second city would move
the count by far more than a lease does.

## What is missing before this can be refined

- **What destroys a city, and what it costs.** A cohort that stands on a seat
  now takes the ground under it, and the settlement stands on that ground
  unchanged. The work must say whether a settlement falls to occupation, to a
  work total the way an upgrade does, or to nothing at all. Each answer is a
  different mechanism and a different record.
- **Whether a faction founds a second city, and what it pays.** A settler verb
  exists and one record states it.[^5] The controller choice set has no found
  option, so nothing in a run calls it. The work must say which store the
  founding spends, and what stops a faction from founding on every tile.
- **What happens to the store, the queue and the residents of a city that
  falls.** A settlement carries a store, a production queue and a resident
  count. The work must say whether each is destroyed, transferred or left.
- **What the game end reads afterwards.** The seat of a faction is the tile of
  its first founding, and the domination reader reads the holder of each seat
  tile.[^6] The work must say what the seat of a faction with no city is, and
  what the reader answers for a faction whose city fell.
- **Which records change.** The game end record holds the seat clause. The
  holding record holds the reach of a city and the settler verb. Both hold a
  decision this item touches.

## Done when

Not written. Refine the item first.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-542. `docs/FINDINGS.md`
[^2]: ADR-0153, a tile's lease follows the units that stand on it. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^3]: Backlog item 0507, let a faction take the ground of another. `docs/backlog/complete/0507-let-a-faction-take-the-ground-of-another.md`
[^4]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D2. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^5]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^6]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
