# ADR-0132: Conversion changes the faction of a unit, and adds no second allegiance

## Context

A downstream game needs a god that converts people. A person who converts stops
belonging to one god and starts belonging to another. A product record states
the need.[^1]

There are two ways to hold that fact. A unit can carry its faction and nothing
else, and conversion writes the faction. Or a unit can carry a faction and a
separate allegiance, and conversion writes the allegiance while the faction
stays.

The project owner decided the first one. Conversion changes the faction
outright.

The choice is not free of consequence, and the consequences are the reason it
is worth recording. A unit carries more than a faction. It carries a type, a
carried load, a home site, a seat at that site, a gather order, a build order,
a destination, and sometimes a character. Each of those is a separate fact, and
each one either follows the unit into the new faction or does not.

**A faction is also a key of several totals.** The unit arena keeps one live
count for each faction and maintains it where a slot becomes live and where a
slot stops being live.[^2] A cohort is the units of one faction that belong to
one site.[^3] The presence relation is folded from the faction of every
unit.[^4] A conversion is a third place that moves a unit between two factions,
and it is neither a birth nor a death.

## Decision

### D1. Conversion writes the faction of the unit, and the project holds no allegiance value

**A unit carries exactly one statement of who it belongs to.** Conversion
writes that statement. The engine holds no second value that a rule could read
instead.

The unit keeps its identity, its slot and its generation, so nothing that names
the unit stops naming it.[^5]

The alternative is a separate allegiance value. It was rejected because every
rule that reads a faction would then have to choose which of the two values it
means, and nothing fails when two rules choose differently. That is the defect
shape this project meets most often.[^6] With one value, the presence relation,
the ground a faction holds and a meeting between two factions all read the
change with no further work.

The cost is that the change is not reversible by the engine. A unit that
converted carries no memory of where it came from. A game that wants that
memory reads the event log, which records both factions.[^7]

### D2. A convert keeps every physical fact about itself

**The type, the carried load, the tile, the home site and the seat at that site
all survive a conversion.**

The load must survive, because what leaves a tile has to arrive somewhere
exactly, and a load that vanished would break the conservation the engine
checks.[^8]

The type must survive, because a type is what a body can do rather than what a
body believes.

The home site must survive, because a unit draws what it consumes from the site
it belongs to. A convert that lost its home would stop eating on the frame it
converted, and conversion would then be a disguised death. The cohort table is
keyed on the site and the faction together, so the row of the unit moves by
itself when the table is derived again.[^3]

The alternative is to send a convert home to a site of its new faction. It was
rejected because the engine holds no rule that says which site, and inventing
one puts a placement rule inside a conversion.

### D3. A convert loses every order that the control plane gave it

**The gather order, the build order and the destination all end at the
conversion.**

An order is a standing instruction from the control plane that owned the unit.
A convert that kept one would let a god steer the units of another god without
that god agreeing, which is a hole in the boundary rather than a feature of
conversion.

The unit takes an option for itself on the next frame that its cell schedules a
choice, in the same way as a unit that nobody has sent anywhere.[^9]

### D4. A convert takes its character with it

**When a unit carries a character, the character changes faction with the
body.**

A character is the person that a unit became, and the unit is the body that
carries the person.[^10] A body of one faction carrying a person of another
would be one allegiance held in two places, with the copies in disagreement and
nothing that fails.[^6]

### D5. One place applies a conversion, and it moves every total that follows a faction

**The field pass and the control plane verb both apply through one function.**
That function writes the faction, clears the orders, moves the character,
emits the event and derives the cohort table again. The unit arena moves its
own live count for each faction, and its own check recounts and compares.[^2]

Two apply paths would be two declaration sites for what a conversion means, and
the second one would forget a total. The arena check would fail, and the repair
that makes a failing check pass is to remove the check.[^6]

## Consequences

- The project cannot express a unit that fights for one faction and believes in
  another. A game that wants that must model it above the engine.
- A conversion raises the arena revision, because the presence relation is
  derived from the faction column and refuses a read from an arena that has
  moved on.[^4] The derived unit structure does not read that column, so it
  rebuilds for nothing on a frame that converted somebody. That is the
  conservative side of one counter, and a second counter would be one fact in
  two places.[^6]
- A convert may hold a seat at a site of its old faction until the next
  rebalance of that site opens the seats again.[^11] The engine states no rule
  that removes it earlier, because a rule that did would be a placement rule
  inside a conversion.
- A game that wants to know where a unit came from reads the event log. The
  engine keeps no history on the unit.[^7]

## References

[^1]: Product record 0035, a god takes the people of another god. `docs/product/shaped/prd-0035-a-god-takes-the-people-of-another-god.md`
[^2]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^3]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^4]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D1 and D4. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^5]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^6]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^7]: ADR-0134, a god reads conversion as an event log and as the counts it already reads, decision D1. `docs/adrs/draft/adr-0134-a-god-reads-conversion-as-an-event-log.md`
[^8]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^9]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^10]: ADR-0104, a soldier is promoted from a level that never falls, decision D3. `docs/adrs/draft/adr-0104-a-soldier-is-promoted-from-a-level-that-never-falls.md`
[^11]: ADR-0099, a site fills its positions by one sort and one scan, decision D2. `docs/adrs/draft/adr-0099-a-site-fills-its-positions-by-one-sort-and-one-scan.md`
