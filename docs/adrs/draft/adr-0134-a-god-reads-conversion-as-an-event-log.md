# ADR-0134: A god reads conversion as an event log and as the counts it already reads

## Context

A unit that converts changes its faction outright.[^1] The field decides most
conversions, so a god does not choose them one at a time and cannot know in
advance which units it will take.[^2]

A mechanic that a player cannot observe is a mechanic that a player cannot
play. A god that gained people has to be able to see that it gained them, where
it gained them, and from whom. Otherwise a source term placed on the map has no
visible result until a headcount happens to move.

The engine already answers "how many units does each faction hold". The unit
arena keeps one live count for each faction and maintains it where a slot
becomes live and where it stops being live.[^3] The cohort table holds the same
count for each site and faction pair.[^4] Neither of those says where a change
happened, or who lost it.

The engine also has one shape for reporting what a frame did. It holds one
append-only array for each event type, the array covers the last step alone,
and it crosses to the control plane at the frame barrier.[^5] [^6]

## Decision

### D1. The engine emits one event for each unit that changed faction, and the event names both factions

**One event holds the frame, the identity of the unit, the tile it stood on,
the faction it left and the faction it joined.**

The tile is what makes the log answer "where". The two factions are what make
it answer "from whom", and they are also the only record that the unit was ever
somewhere else, because the unit itself carries no history.[^1]

The event is plain data with an explicit layout and no boolean field, like
every other event of this project.[^5]

The log covers the last step alone. It holds the conversions the field decided
and the conversions the control plane asked for, because both are the same
change and a reader that had to merge two logs would be reading one fact from
two places.[^7]

### D2. The engine adds no new aggregate for conversion

**A god that wants a count reads the counts that already exist.** The live
count for each faction moves with every conversion, because the arena moves
it.[^3] The cohort headcount for each site and faction moves with it, because
the table is derived again.[^4]

A separate running total of conversions was rejected. It would be a second
statement of something the population counts already say, and nothing would
fail when the two disagreed.[^7]

### D3. A god reads the field itself to see where a conversion will happen next

**The engine states no prediction and no explanation.** A caller that wants to
know why a unit converted, or where the next one will, reads what each faction
reaches at a cell. That read already exists, and it is one gather from the
level the caller already reads.[^8]

An explanation call was rejected. The engine holds one for a movement choice,
and it exists because a choice scores a fixed option set against weights that a
reader cannot see.[^9] A conversion has no such hidden score: the leader and
the margin are both readable at the cell.

## Consequences

- A control plane that does not read the log after a step loses the record of
  that step. The log is cleared at the start of the next one.
- A game that wants the history of one unit keeps it itself. The engine holds
  the last step and nothing before it.
- A watcher can draw where belief is moving from the log alone, because every
  event carries a tile.

## References

[^1]: ADR-0132, conversion changes the faction of a unit and adds no second allegiance, decisions D1 and D2. `docs/adrs/draft/adr-0132-conversion-changes-the-faction-of-a-unit.md`
[^2]: ADR-0133, a unit converts to the faction that leads the influence field at its cell, decisions D1 and D3. `docs/adrs/draft/adr-0133-a-unit-converts-to-the-faction-that-leads-the-field.md`
[^3]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^4]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^5]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^6]: ADR-0006, an event is plain data and applying it is pure, decision D2. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^7]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: Decisions register, DEC-040. `docs/DECISIONS.md`
[^9]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
