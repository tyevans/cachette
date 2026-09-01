---
id: 0088
title: Promote a soldier into the character tier
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0011]
blocked-by: []
---

## Why

The living character column set exists, and the tier declaration exists.[^1]
Nothing creates a character from the world. A character today arrives only
because a caller asked for one.

A story needs somebody it is about. One million soldiers who each carry an
individual experience are the pool that a named person comes from. A soldier
who crosses an achievement bound must become a character.

## What the work does

1. Add an achievement column to the soldier row, and a rule that raises it.
2. Maintain an eligibility bit for each soldier, written by the same pass that
   raises the achievement value.
3. Scan the bits at the character barrier, not at every tick.
4. Rank the eligible soldiers by a key vector, never by a comparison function.
5. Allocate against a budget, so the population never passes the ceiling.
6. Link a character to the soldier who carries them, and clear the link when
   the soldier ends.

## What holds this back

**The achievement value must never fall.** The scan reads a level and not an
edge, so a lazy scan is correct only while the value rises. A rule that lowers
it breaks the scan in silence. This is a constraint on the content, not an
implementation detail. State it, and check it in a debug build.[^2]

**A promotion creates, it never mutates.** An entity never changes tier while
it lives, so the promoted soldier does not become a character. The engine
creates a character and links the two.[^3]

**A promoted soldier gets no invented ancestry.** He founds a new line, he has
a relation of zero to everybody, and he cannot inherit by blood. A title holder
may appoint him.[^4]

**The order must come from the sort.** The eligible set is collected in
ascending slot order, the rank is a key vector, and the identifiers are
allocated after the budget cut. Never during the scan.[^2] [^5]

## Impact review

Not done. This item is `proposed` and needs one before anyone takes it.

## References

[^1]: Backlog item 0066. `docs/backlog/complete/0066-provide-the-character-column-set-and-the-tier-declaration.md`
[^2]: The character graph and inheritance, section 9. `docs/research/reports/14-character-graph-and-inheritance.md`
[^3]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D4. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^4]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^5]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
