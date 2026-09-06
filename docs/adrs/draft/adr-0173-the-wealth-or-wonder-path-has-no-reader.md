# ADR-0173: The wealth or wonder path has no reader

## Context

The engine ends a game when a reader fires. A game end record holds the
winning faction, the win path and the tick, and the step writes it once.[^1]
The design names four win paths: domination, territory at a tick limit,
wealth or wonder, and renown.[^2] The wealth-or-wonder path fired when the
stock total of a faction reached a target, or when an upgrade that carries a
victory claim stood on ground the faction held.

**That path decided every game, and it decided them early.** A sweep of 32
seeds, played to a horizon of 20000 ticks, ended by wealth or wonder in 32 of
32. Domination, territory and renown ended none. The end ticks ran from 1059
to 16850. The register holds the derivation and the findings register holds
the reading.[^3] [^4]

**The bar cannot be raised out of reach.** A faction founds one settlement,
the commodity count is one, and a store is a fixed-point value of a fixed
width, so the stock total of a faction clamps at a ceiling the type
states.[^5] A target at or above that ceiling fires never, so every target
the engine permits sits under it. The store rises and does not fall, so every
target the engine permits is crossed given enough ticks. A previous pass
raised the target sevenfold. That bought about fifteen percent more ticks and
still ended every game.[^4]

A previous record answered this by standing the bar above what one settlement
can hold, so that a wealth win asked a faction to expand.[^6] That record
addressed the reachability and not the question underneath it.

**The question underneath it is whether the path is a win condition at all.**
The project owner ruled that it is not: a stock total is not a claim to
victory, the path always triggers too early, and the moment it fires feels
arbitrary to a watcher, because nothing on the picture changed when it did.
Domination and territory are the two paths the project wants to decide a
game.

## Decision

### D1. The wealth-or-wonder path has no reader, and no game ends on it

The table of game end readers holds domination, territory and renown. It
holds no reader for the wealth-or-wonder path. No run of this engine ends on
that path.

The earlier record already states the shape this takes: a path that has no
reader is skipped, and the order of the others does not change.[^1] This
record states that the wealth-or-wonder path is such a path, and that it is
not waiting for a reader.

A reviewer finds a violation when the reader table holds a fourth entry, or
when any pass writes a game end record whose path is the wealth-or-wonder
path.

### D2. The path keeps its variant and its number

The win path type keeps the wealth-or-wonder variant, and the variant keeps
its number. An event carries that number, and a stored end record carries it.
A record written before this decision must still resolve to a name, and a
reader of an old log must still find one.

Removing the variant, or renumbering the paths after it, would silently
change the meaning of stored bytes. This decision costs one unused variant
and buys that.

A reviewer finds a violation when the variant is deleted, or when any path
after it takes a different number.

### D3. The quantity behind the path is still reported

The boundary reports the stock total of a faction and the work its furthest
wonder has reached, as running values of a faction. Nothing compares them
against a threshold.

A watcher may want to see a faction grow rich, and a caller may want to train
against that quantity. Neither asks for a win. The stock target and the
wonder work stay in the balance register, because the reported values are
stated in the same units, and each row records that no reader compares it.[^3]

A reviewer finds a violation when a report drops the quantity, and when a
document calls either quantity a way to win.

## The alternatives this rejects

**Raise the target again.** Rejected because every target the engine permits
sits under the ceiling of one store, and the store reaches the ceiling. The
measurement covers the whole distribution and not one value.[^4]

**Stand the bar above what one settlement holds.** Rejected as an answer to
this question, though it is a sound answer to the one it was written
for.[^6] It makes the stock clause quiet rather than absent, because nothing
in the engine founds a second settlement today.[^7] A clause that decides
nothing and looks as if it might is worse than one that is plainly gone: a
reader plans against it, and a later change that founds a second settlement
turns it back on without anyone deciding to.

**Keep the wonder clause and drop the stock clause.** Rejected because the
ruling is about the path and not about one of its two triggers. A wonder is a
thing a faction builds, and the engine should reward it, but the reward is
not the end of the game. The wonder clause ended three of the 32 seeds, so
keeping it would keep an arbitrary end for the same reason at a lower rate.

**Delete the variant.** Rejected under D2, because stored bytes carry its
number.

**Keep the reader and let the tick limit outrun it.** Rejected because there
is no such limit. The path fired as early as tick 1059, and a limit under
that ends every game on territory instead, which is a limit chosen to hide a
reader rather than to shape a game.

## Consequences

**Three paths decide a game, and one of them fires only at the tick limit.**
Domination needs a faction to remove its rivals or hold every seat. Renown
needs a column that no pass in the engine writes. So a run that reaches the
tick limit ends on territory, and a run that does not ends on domination.
The share each path takes is a reading the balance register holds, and this
change moves it.[^3]

**A run may now reach the tick limit where it did not before.** That is the
intended shape, because territory at a limit is one of the two paths the
project wants. It also means the tick limit now decides the length of most
runs, so the limit is load-bearing where it was not.

**A finished wonder ends no game.** The build pass, the census row and the
victory claim column are unchanged. The claim column is read by the reporting
function and by nothing that decides.

**A test that ends a game on stock or on a wonder no longer exists.** The
tests that held those cases now state the opposite rule, and each is proven
able to fail by returning the reader.

**The earlier record on the wealth bar is superseded.** Its decisions concern
how high a bar must stand for a reader that no longer exists.[^6]

## References

[^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^2]: Design, the living world game layer, section 5. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^3]: Balance register, the stock target and the wonder work. `docs/reference/balance.md`
[^4]: Findings register, FND-543 and FND-550. `docs/FINDINGS.md`
[^5]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: ADR-0165, the wealth bar stands above what one settlement can hold. `docs/adrs/draft/adr-0165-the-wealth-bar-stands-above-what-one-settlement-can-hold.md`
[^7]: Findings register, FND-542. `docs/FINDINGS.md`
