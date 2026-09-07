# ADR-0181: A faction that holds no site and no unit leaves the game, and its ground returns to nobody

## Context

A faction is a bit index into a mask, and the engine allocates that index when
the faction takes its seat.[^1] The seat is the stored record of where the
faction founded. Until now nothing ever removed a faction from the game.

A faction that lost every unit and every person kept everything else. It kept
the ground its cities once reached, and it kept the leases its units once
raised.[^2] [^3] It kept its seat, and every reader of the world still walked
it.

That state is wrong in one visible way. One reader of the game end compares the
ground that each faction holds at the tick limit, and it names the faction with
the most ground the winner.[^4] A faction with nothing alive could therefore
win the game on ground it can no longer defend, act on or lose.

Conquest makes this reachable. A companion record lets a faction take the last
city of another faction, or destroy it.[^5] Before that record no rule could
empty a faction, so the question did not arise.

Three forces fix the shape.

**A faction with nothing alive has no way back.** The engine must say so once,
in one rule, rather than let each reader decide for itself.

**The ground must return by the rule that already decides every holder.** A
second rule for who inherits ground would be a second statement of one fact,
and nothing would fail when the two disagreed.[^6]

**Every pass in this project runs a fixed number of times.** A rule that
repeated until nothing more changed would be a convergence test, and a run must
give one answer at any thread count.[^7]

## Decision

**A faction that holds no site and no unit leaves the game. The engine removes
its characters, releases its held ground and its leases to nobody, and stops it
from winning. The faction keeps its identifier and its seat for ever.**

### D1. A faction leaves the game when it holds no site and no unit

The test reads two things. The faction owns no live site, and no live unit
names the faction. A faction that fails both tests leaves the game.

A site grows people and a unit acts.[^8] A faction with neither has no way back
into the world. It cannot found a city, because a founding needs a unit that
carries the settle capability.[^9] It cannot build, gather, fight or take
ground, because each of those acts starts from a unit or from a site.

A faction that has not yet founded has not entered the game, so the rule does
not remove it. The seat of a faction is what says that the faction entered. The
elimination therefore reads the seat first, and it walks only the factions that
hold one.

A reviewer finds a violation when the rule removes a faction that owns a live
site, when it removes a faction that a live unit names, or when it removes a
faction that has not taken a seat.

### D2. A character alone does not keep a faction in play, and the elimination removes its characters

A character carries no tile, so it stands nowhere. It holds no ground, it takes
no step and it fights nothing. A faction whose only remaining property is a
character is therefore out by D1.

The elimination removes the characters of the faction. Nothing then outlives
the faction, and no stored row names a faction that is out. A reader that
walked the characters would otherwise find a row for a faction that no other
reader sees.

This creates a coupling with a capture. A capture moves every resident of a
site to the taker, and the character that a resident carries moves with the
unit.[^10] A character that stayed with the previous faction would name a
faction that the elimination is about to remove. The two rules must agree,
because nothing fails when they disagree.

A reviewer finds a violation when a character keeps a faction in the game, when
a character survives the elimination of its faction, or when a character and
the unit that carries it name different factions.

### D3. The elimination releases the ground and the lease together, and the released ground returns to nobody

A faction claims ground in two ways. The holder column names the faction on
each tile that a city of the faction reaches.[^2] A lease on a tile names the
faction, and a lease at the claim threshold outranks the reach of every
city.[^3]

The elimination releases both. A release that cleared only the holder would
give the tile back on the next spread, from a lease that nobody can raise any
more. The lease of every tile the faction holds therefore returns to nobody,
and its count returns to zero.

The released ground goes to nobody. The spread of the next tick then gives each
released tile to the nearest city that reaches it, by the rule that already
decides the holder of every tile.[^2] **This is why no second rule for who
inherits the ground is written.** A tile that no city reaches stays with
nobody, which is the same answer the spread gives to any other unreached tile.

A reviewer finds a violation when the elimination clears the holder and leaves
the lease, when it names a faction as the inheritor of a released tile, or when
a released tile keeps a lease count above zero.

### D4. The pass runs once for each tick and an elimination does not cascade within a tick

The pass runs once for each tick. It reads the world as the passes before it
left it, and it records every faction that fails the test of D1.

Ground that one elimination releases reaches another faction on the next tick,
through the spread.[^2] The pass does not run again on the state it produced. A
pass that settled until quiet would run a variable number of times, and this
project fixes the count of every pass.[^7]

The pass walks the factions in ascending identifier order. Two factions that
leave on one tick are therefore recorded in that order, and no result depends
on the order in which a thread finishes.[^11]

A reviewer finds a violation when the pass repeats until nothing changes, when
it runs more than once for a tick, or when the order of two eliminations reads
anything other than the faction identifier.

### D5. An eliminated faction wins nothing, and every game end reader walks the factions that may still win rather than every faction

Every reader of the game end walks the factions that may still win. An
eliminated faction is not one of them, so no reader can name it the winner.[^4]

Be plain about what this changes today. The release of the ground and the
removal of the characters already leave an eliminated faction with nothing that
any reader now reads. The ground reader finds no ground, and no other reader
finds a live thing. **The filter therefore changes no answer that this project
can currently construct.**

The filter is written because the rule must stand where a future reader of the
records will see it. A later reader is added by later work, and that reader
must not have to derive the rule again. The record states the constraint, and
it does not claim that a current reader needs it.

A reviewer finds a violation when a reader of the game end walks every faction
with a seat, or when any path can name an eliminated faction the winner.

### D6. A faction keeps its identifier and its seat for ever

The identifier of a faction indexes the faction mask and the relation plane,
and the engine never reuses it.[^1] A stored record that names a faction
therefore always resolves to the faction that wrote it, and never to a later
one.

The seat stays as the record of where the faction founded. A watcher reads the
history of a finished run and finds where each faction started, including the
factions that left.

A reviewer finds a violation when the engine frees a faction identifier, when
it gives an identifier to a second faction, or when an elimination clears the
seat.

## The alternatives this rejects

**A character keeps a faction in play.** Rejected under D2. A character carries
no tile, so it holds no ground, takes no step and fights nothing. A faction
with only characters would sit in the game for ever with no act available to
it.

**Released ground falls directly to whoever holds the ground around it.**
Rejected because the spread already answers that question, by the rule that
decides the holder of every tile.[^2] A second rule would be a second
declaration of one fact, and nothing would fail when the two disagreed.[^6]

**Elimination cascades within one tick until nothing more changes.** Rejected
because that is a settle-until-quiet loop. The pass count would then depend on
the state, and a run must give one answer at any thread count.[^7]

**The faction identifier is freed for reuse.** Rejected because the identifier
indexes the faction mask and the relation plane.[^1] A reused identifier makes
a stored record resolve to the wrong faction, and nothing fails when it does.

**Elimination waits for a faction to lose its characters too.** Rejected
because a character cannot act. The faction would stay in the game while it had
no way to change anything, and the ground would stay held by a faction that
cannot defend it.

## Consequences

**A game can now end in domination by elimination.** One faction removes every
other faction, and the game end has a winner before the tick limit.

**A dead faction can no longer win on territory.** That is the defect this
record closes.[^4]

**A run that eliminates a faction is shorter.** That is a balance consequence.
The balance register and the project owner must judge it, and this record does
not.[^12]

**Every golden state file that holds an eliminated faction moves.** The
elimination writes the holder column, the lease and the characters, and the
state hash folds every stored value that the step reads.[^13]

**The engine now removes stored rows that a caller may hold a handle to.** A
caller that stored a handle to a character of an eliminated faction finds that
the handle no longer resolves. The caller must handle that failure, in the way
it handles any other identity that no longer resolves.[^14]

## References

[^1]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decisions D1 and D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^2]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D1 and D3. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^3]: ADR-0153, a tile's lease follows the units that stand on it, decisions D1 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^4]: ADR-0148, a game end is recorded once and stops the controllers, decision D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^5]: ADR-0180, a site changes hands or the taker destroys it, decisions D1 and D5. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^6]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^7]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^8]: ADR-0157, a site's free places are its built housing less the residents the engine counts, decisions D2 and D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^9]: ADR-0150, held ground is the ground within reach of a city its faction owns, decision D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^10]: ADR-0180, a site changes hands or the taker destroys it, decision D5. `docs/adrs/draft/adr-0180-a-site-changes-hands-or-the-taker-destroys-it.md`
[^11]: ADR-0004, iteration order is explicit, decisions D1 and D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^12]: Balance register. `docs/reference/balance.md`
[^13]: ADR-0164, every stored value the step reads enters the state hash, decision D1. `docs/adrs/draft/adr-0164-every-stored-value-the-step-reads-enters-the-state-hash.md`
[^14]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
