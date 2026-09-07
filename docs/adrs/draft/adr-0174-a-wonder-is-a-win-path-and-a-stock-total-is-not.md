# ADR-0174: A wonder is a win path and a stock total is not

## Context

The engine ends a game when a reader fires. A game end record holds the winning
faction, the win path and the tick, and the step writes it once.[^1] The design
named four win paths: domination, territory at a tick limit, wealth or wonder,
and renown.[^2]

**One path held two triggers.** The wealth-or-wonder reader fired when the stock
total of a faction reached a target, or when an upgrade that carries a victory
claim stood on ground the faction held. Two unrelated statements about a faction
shared one reader and one name.

A previous record retired the whole path, because the stock clause decided every
game and decided it early.[^3] That record considered keeping the wonder clause
alone and rejected it, on the ground that the ruling was about the path and not
about one trigger.

**The project owner has now separated the two.** The objection is to the stock
clause only: a store passing a number is not a way to win a war. A wonder costs
work, stands on ground the faction holds, and a watcher sees it appear. The
owner rules that the wonder path can be shaped by tuning what it costs, and that
the stock clause goes for good.

A separate record makes each win value a value the world holds, so a search can
shape a path without a rebuild.[^4]

## Decision

### D1. A finished wonder is a win path, with a reader of its own

The table of game end readers holds a reader for the wonder. It fires for the
first faction, in ascending faction order, that holds ground on which a standing
upgrade carries a victory claim above zero.

The engine has four win paths: domination, territory, wonder and renown. Each
has one reader, and each reader states one claim about a faction.

A reviewer finds a violation when the reader table holds no wonder entry, or
when the wonder reader compares a quantity other than a standing victory claim
on held ground.

### D2. No reader compares a stock total

The engine holds no stock bar and no reader that compares one. A stock total
wins no game, whatever its size.

The reason is not that a bar was set badly. A store is a fixed-point value of a
fixed width, so a stock total has a ceiling that the type states. Every bar the
engine permits sits under that ceiling, and a store rises and does not fall, so
every permitted bar is crossed given enough ticks. The previous record measured
this over a whole distribution rather than at one value.[^3]

A reviewer finds a violation when any reader compares a stock total, or when the
engine declares a stock bar again.

### D3. The wonder path keeps the number the wealth-or-wonder path held

The win path type names the wonder, and that variant keeps the number the
wealth-or-wonder variant held. An event carries that number, and a stored end
record carries it.

Renumbering would silently change the meaning of stored bytes. A record written
before this decision resolves to the wonder, which is one of the two things it
could have meant, and the more likely of the two to be worth reading.

A reviewer finds a violation when the wonder takes a different number, or when
any path after it moves.

### D4. The stock total of a faction is still reported

The boundary reports the stock total of a faction as a running value, beside the
values that feed the readers. Nothing compares it against a threshold.

A watcher may want to see a faction grow rich, and a caller may want to train
against that quantity. Neither asks for a win.

A reviewer finds a violation when a report drops the quantity, and when a
document calls it a way to win.

## The alternatives this rejects

**Keep the path whole, with both triggers.** Rejected because the two triggers
say unrelated things. One reader that fires on either cannot be tuned, because
raising the wonder cost does not quiet the stock clause and raising the stock bar
does not reach the wonder.

**Keep the path retired.** Rejected by the project owner. The earlier record
retired the wonder along with the stock clause on the ground that the ruling was
about the path.[^3] The owner has now stated that the ruling was about the stock
clause, and that a wonder is an achievement the game should end on.

**Give the wonder a work threshold rather than a claim.** Rejected because a
partly built wonder is not a wonder. The reader asks whether a finished one
stands. How much work finishing takes is a column of the upgrade table, and a
caller sets it.

**Delete the variant and renumber.** Rejected under D3, because stored bytes
carry the number.

**Keep the stock bar and stand it out of reach.** Rejected because no bar the
engine permits is out of reach. This was the subject of a record that the
previous one superseded.[^5]

## Consequences

**A wonder now ends a game, and the work it costs decides how often.** The work
is a column of the upgrade table, so a caller shapes the path without a rebuild.
A caller that sets the victory claim to zero takes the path out of the game
without touching the reader table.

**The order of the readers puts the wonder ahead of renown and behind
territory.** Two paths that become true on one tick record the earlier of the
fixed order. A run that reaches the tick limit with a finished wonder standing
records territory, not the wonder.

**A constant that stated the stock bar is gone, and so is the module function
that reported it.** A caller that read the bar as a scale for the reported stock
total reads the stock one settlement can hold instead. That value is a property
of the type and not a balance value.

**The tests that stated "a stock total ends no game" still state it.** They no
longer state it at a bar, because there is no bar. They state it at the ceiling
of one store and above the range of a 32-bit accumulator.

**The record that retired the whole path is superseded.** Its D1 gave the path
no reader, and the wonder now has one. Its D2 kept the variant and its number,
and this record keeps that. Its D3 kept both quantities reported, and this record
keeps the stock total reported while the wonder progress now feeds a path.[^3]

## References

[^1]: ADR-0148, a game end is recorded once and stops the controllers, decisions D2 and D3. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^2]: Design, the living world game layer, section 5. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^3]: ADR-0173, the wealth or wonder path has no reader. `docs/adrs/draft/adr-0173-the-wealth-or-wonder-path-has-no-reader.md`
[^4]: ADR-0175, a win threshold decides when a reader fires and never what the simulation does. `docs/adrs/draft/adr-0175-a-win-threshold-decides-when-a-reader-fires.md`
[^5]: ADR-0165, the wealth bar stands above what one settlement can hold. `docs/adrs/draft/adr-0165-the-wealth-bar-stands-above-what-one-settlement-can-hold.md`
