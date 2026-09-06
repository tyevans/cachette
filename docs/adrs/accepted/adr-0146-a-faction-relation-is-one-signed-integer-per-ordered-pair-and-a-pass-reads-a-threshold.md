# ADR-0146: A faction relation is one signed integer per ordered pair, and a pass reads a threshold

## Context

Two factions in this engine are at war whenever they touch. The contest pass
resolves a meeting wherever two factions stand beside each other, and nothing
gates it.[^1] A downstream game needs alliance, peace, tension and war between
players, and it needs a faction to move from one to another by what happens in
the world.[^2]

**The shortest path is a named state per pair.** An enumeration with four
variants, one per pair, and a pass that matches on it. A contributor who reads
the design sees four band names and reaches for four variants.

Two facts refuse that. **A named state cannot move by a step.** A contract
that delivers in full should warm a relation a little, and a unit that falls
should cool it a little. A four-state machine has no "a little". Every cause
would need a transition table, and every transition table is a rule nobody has
stated. **A named state puts the band edges in code.** Where tension becomes
war is a balance value, and a balance value that lives in an enumeration cannot
move without a compile.

The engine already holds one shape for a fact about an ordered pair of
factions. The trade plane holds a row per ordered pair, its size follows the
faction count of the world and never the population, and it enters the state
hash.[^3] An earlier record fixed the shape of every relation between
factions: one row for each faction, and never a field over the tiles.[^4] That
row holds bits, and a bit holds no distance from an edge.

## Decision

**The relation between two factions is one signed integer for each ordered
pair. A pass that needs a band compares the integer to an edge it reads from
the balance table. The engine holds no enumeration of bands.**

### D1. One signed integer per ordered pair, in a dense matrix that enters the hash

The matrix holds one signed integer for the pair (A, B), which is what A feels
toward B. The pair (B, A) is a separate entry. The matrix is dense, its size
follows the faction count of the world, and it enters the state hash because a
later frame reads it.[^5] No term of its size follows the population.

**This changes ADR-0053 D7.** That decision says a relation between two
factions is one mask row for each faction. A row of bits cannot move by a
step, so it cannot carry a graded relation. This record keeps the shape that
decision fixed, and widens the row: one signed integer for each ordered pair,
and never a field over the tiles.[^4]

The integer is a whole number and never a floating point value.[^6]

### D2. A band is a threshold in a table, and a pass reads the edge, never a name

The bands the game names today are alliance, peace, tension and war. Their
edges are balance values and live in the balance register.[^7] A pass that
asks whether A is at war with B compares the integer to the war edge.

Each edge carries the name of the band it opens, and that name is the name of
a parameter. **The engine holds no type whose values are the bands, and no
pass takes a branch on such a value.**

A reviewer finds a violation when the code holds a variant named for a band, or
when a pass compares the integer to a literal.

### D3. Every cause moves the relation by an integer step, and every step is a balance value

A contract that delivers in full moves the relation up. A contract that fails
moves it down. A unit that falls to the other side moves it down. A unit that
converts away moves it down. A drift moves it toward the peace band on a fixed
schedule, in the shape the economy and position schedules already use.

Each step is an integer in the balance register.[^7] The engine invents no
step.

**A storm on the ground of another faction is a cause that nothing supplies.**
A god inflicts weather only on ground its own faction holds, so no storm of
one faction reaches the ground of another.[^8] The register row for the step
is unset, and a blocker governs the value.[^9] The rules carry the step and no
pass reads it. When a later record lets a storm reach the ground of another
faction, that cause moves the relation by this step and by no other path.

### D4. The contest fires only when at least one of the pair is in the war band

This decision changes ADR-0121 D1. That decision says a tile is contested when
some unit within reach belongs to a faction that some unit on the tile does
not. This record adds one condition: the pass resolves the meeting only when at
least one of the two factions is in the war band toward the other.

The contest passes from always-on to gated. Two factions that touch in peace
stand beside each other and nothing happens.

Three other passes read the same integer. Conversion requires the leading
faction to be below a stated edge toward the faction of the unit.[^10] Trade
refuses an offer and a counter-offer when either side is in the war band
toward the other. Movement refuses entry to ground another faction holds when
the holder is below a stated edge toward the guest. Both edges are balance
values.[^7]

### D5. A unit speaks a relation move through one gated verb

One verb moves the entry for (speaker, other) by a step. The speaker is a
unit. The verb refuses when the type of the speaker unit has a command reach
of zero, and it refuses a step above a bound that is a balance value.[^11]
[^7] The faction controller reaches the matrix through this verb and through
no other, so the gate holds for a controller as it holds for a god.[^12]

A caller may also write an entry outright, with no speaker and no gate. That
write states the relation a scenario starts from. It names no unit, so no
command reach can gate it.

**One function inside the engine writes every entry.** The verb, the outright
write and each cause of D3 pass through it, so a crossing is judged once
however it arose.[^13]

A reviewer finds a violation when a second site writes an entry, or when the
verb reads a per-faction flag rather than the type of the speaker unit.

### D6. A crossing of the war edge writes one event

When the integer crosses the war edge in either direction, the pass writes one
event. The event holds the tick, the two factions, and the band on each side
of the move. A reader takes the direction from the two bands, so no field
states it twice.[^13] The event is plain data with a declared layout, declared
padding and no boolean.[^14]

### D7. A band reaches a caller as a number that counts the edges at or below the value

A caller outside the engine needs the band. The engine answers with a number:
how many edges lie at or below the integer. The number rises with the
relation, so the lowest number is the war band.

**The engine supplies no word for a band.** A caller that wants a word holds
its own list, and D2 is the reason the engine holds none.

The numbering is a decision because more than one reader carries it across the
boundary, and nothing fails when a caller reads it the other way round. Every
such reader then reports the opposite band, and reports it consistently.

The count of the edges decides the count of the bands. A later record that
adds an edge adds a band and moves every number above it, so a caller that
stores a band number across such a change stores a band that moved.

A reviewer finds a violation when the engine answers a word for a band, when a
reader numbers the bands downward, or when a second site derives the band from
the edges rather than calling the one that does.[^13]

## The alternatives this rejects

**An enumeration of named states.** Rejected because a state cannot move by a
step, and because the edges would live in code.

**A symmetric relation.** One integer per unordered pair. Rejected because
what A feels toward B is not what B feels toward A. A default hurts the
creditor and not the debtor, and a one-sided grievance is the most common
shape a game has.

**A relation derived from events every frame.** Rejected because a derivation
over the event log would cost the log length on every frame, and because the
relation is a fact a later frame reads, so it is simulated state and must be
hashed.[^5]

**Keeping the contest always-on and adding a truce flag.** Rejected because a
flag is a second copy of a fact the integer already holds, and nothing fails
when the two disagree.[^13]

**A named band on the event.** Rejected under D7. A word on an event is a
second declaration of the edges, and it goes stale the moment a register row
moves.

## Consequences

**The engine cannot express a relation finer than one integer.** Two factions
that are allied in trade and at war in territory hold one number. A game that
wants two axes writes a second matrix, and that needs a record.

**Peace makes the contest do nothing.** A world whose factions never cross the
war edge never fights, and a test that expected a fight on contact now fails
until it sets the relation. That is the correct failure.

**The relation enters the hash, so a golden hash moves when the matrix is
added.** The commit records the change.

**The matrix is small and fixed, so nothing about diplomacy is made slow by a
large world.** The cost follows the square of the faction count and no term
follows the population.

**A caller that stores a band number is exposed to a change of the edges.**
The number is derived, and the engine derives it again on every read. A caller
that keeps one across a change of the register reads a stale band, and nothing
tells it so.

**Every balance value this record depends on is a parameter.** The edges, the
steps, the drift period, the verb bound and the two stated edges live in the
balance register, and this record decides none of them.[^7]

## References

[^1]: ADR-0121, a meeting between two factions resolves at the tile, never at a level 1 cell, decision D1. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^2]: Design, the living world game layer, section 3. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^3]: ADR-0126, a trade negotiation is engine state, and the words are not, decision D1. `docs/adrs/draft/adr-0126-a-trade-negotiation-is-engine-state.md`
[^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^5]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: Balance register, the relation. `docs/reference/balance.md`
[^8]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
[^9]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^10]: ADR-0133, a unit converts to the faction that leads the influence field at its cell. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
[^11]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decision D3. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^12]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^13]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^14]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
