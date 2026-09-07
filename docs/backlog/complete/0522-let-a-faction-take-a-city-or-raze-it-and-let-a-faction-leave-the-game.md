---
id: 0522
title: Let a faction take a city or raze it, and let a faction leave the game
status: complete
created: 2026-09-06
implements: [ADR-0004 D1, ADR-0006 D1, ADR-0014 D2, ADR-0014 D3, ADR-0153 D3, ADR-0153 D5, ADR-0164 D1]
changes: [ADR-0148, ADR-0150]
creates: [ADR-0180, ADR-0181]
serves: [PRD-0048]
blocked-by: []
---

## Why

**A settlement's faction was written at its founding and never again.** One
assignment wrote it, and the founding was its only caller. No verb and no stage
could move a city from one faction to another, and nothing destroyed a city
except the rollback of a founding that failed.

Two things followed, and both were measured.

**A war could take the ground and the people and end nothing.** Held ground is
decided by the reach of the city nearest a tile, so the ground of a faction
returned to it as soon as the lease of an invader decayed.[^1] A faction that
lost every unit kept its cities, kept its ground and kept its seat for ever,
and its cities kept producing.

**A faction was never removed from the game.** The territory reader compares
held ground at the tick limit, so a faction with nothing alive could win the
game on ground it could not defend.[^2]

The project owner asked for both a capture and a raze, and then asked for the
elimination that a raze makes reachable.

## What the work does

1. A faction that stands on a site tile with no defending unit takes the site.
   The step does this without an order, because the ground decides it.
2. A captured site keeps its identity, its store, its housing, its rates and
   its staff. Every upgrade on the ground goes with the ground.
3. Every resident of a captured site changes faction with the site, and so does
   the character each resident carries. The production queue is cleared.
4. A caller may order a raze instead. A raze destroys the site, its upgrades
   and its residents, and moves the store to the nearest live site of the
   razing faction.
5. A faction that holds no site and no unit leaves the game. The pass releases
   the holder and the lease of every tile it held, and it removes its
   characters.
6. Every game end reader walks the factions that may still win.
7. Two event types carry the two facts to a reader: one for a site that changed
   hands or fell, and one for a faction that left.

## Impact review

**Governed by.** The iteration record fixes the order of every walk this work
adds, and each walk is over the settlement slots or over the factions in
ascending order.[^3] The identity record says a captured site keeps its
identity while a razed one never resolves again.[^4] The lease record supplies
the one statement of who occupies a tile, and the capture reads the same list
the lease pass reads rather than counting again.[^5] The event record requires
plain data with declared padding and no boolean, and both new event types carry
a declared padding array.[^6] The state hash record requires every stored value
the step reads to enter the hash, and the column that says which factions have
left does.[^7]

**Changes.** The game end record states that every faction of a world reaches a
reader, and a faction that has left no longer does.[^2] The holding record
reads the faction of a city as the faction that founded it, and that is no
longer true.[^1]

**Creates.** Two records. One holds the capture and the raze. One holds the
elimination. They are separate because a reviewer can accept either without the
other.[^8] [^9]

**Blockers.** This work resolved the question of whether an upgrade changes
hands when the ground does. It states no cost figure and no balance value, so
the blocker that holds every cost figure of this project governs nothing
here.[^10]

**Serves.** A developer watches factions play a game to an end.[^11]

**Conflict surface.** The settlement column set, the holding, the capture stage
of the world and the game end readers. It touches the step, so one worker holds
it at a time.

## Is this worth building yet

Yes. It was the one thing that held the domination path back, and the priority
index placed it second.[^12]

## Done when

- A faction takes a city by standing on its tile, and the city answers with the
  new faction, its upgrades intact and its residents alive. A test drives the
  step.
- A garrison of one unit refuses the capture, whatever stands against it.
- A razed city is gone, its upgrades are gone, and its store has moved to the
  razer.
- No unit names a home site that is gone.
- A faction with no site and no unit leaves the game once, and the ground it
  held returns to nobody.
- An eliminated faction wins nothing under any reader.
- Each of those has a put-back experiment, and the report says which
  experiments failed and which stayed green.
- The thread-count determinism test passes.
- A sweep of the default seeds says what ends the games before the change and
  after it.

## Outcome

The work landed. Domination now ends every seed of the sweep, where it ended a
minority of them before, and every game ends well inside the tick limit. The
figures are in the commit body.

Three findings came out of it. One records that a fact nothing writes leaves
every reader of it unreachable.[^13] One records two places where a conquest
touched a fact stored in two columns with nothing that failed on
disagreement.[^14] One records two guards this work added that no fixture can
reach, and says so rather than letting a green test imply coverage.[^15]

**Two things are left undone, and neither belongs to this item.** Nothing
inside the engine orders a raze, so a seeded run captures and never razes. A
faction still cannot found a second city during a run, and a separate item
holds that half.[^16]

## References

[^1]: ADR-0150, held ground is the ground within reach of a city its faction owns. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^2]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^3]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0014, entity identity is an index plus a generation, decisions D2 and D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^5]: ADR-0153, a tile's lease follows the units that stand on it, decisions D3 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^6]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^7]: ADR-0164, every stored value the step reads enters the state hash. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^8]: ADR-0180, a site changes hands or the taker destroys it. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^9]: ADR-0181, a faction that holds no site and no unit leaves the game. `docs/adrs/draft/adr-0181-a-faction-that-holds-no-site-and-no-unit-leaves-the-game.md`
[^10]: Blockers register, BLK-036 and BLK-007. `docs/BLOCKERS.md`
[^11]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
[^12]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^13]: Findings register, FND-579. `docs/FINDINGS.md`
[^14]: Findings register, FND-580. `docs/FINDINGS.md`
[^15]: Findings register, FND-581. `docs/FINDINGS.md`
[^16]: Backlog item 0514. `docs/backlog/proposed/0514-let-a-faction-found-a-city-during-a-run.md`
