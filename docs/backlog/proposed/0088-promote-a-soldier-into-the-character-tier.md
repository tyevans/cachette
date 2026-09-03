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

## What DEC-002 means for this item

**Both tiers decide. An individual chooses where to go, and a cohort chooses
what to buy.**[^6] A promoted soldier is embodied: one character row and one
unit row name the same person through the link this item adds.[^2]

That creates one hazard and one requirement.

**The hazard is a second decision site.** If the character row decides where
the person goes, and the unit row decides it too, one fact has two authorities
and nothing fails when they disagree. That is the shape this project meets most
often.[^7]

**The requirement is that the promotion adds no decision site.** A promotion
writes one character row and one link. The unit keeps the individual decision
it already makes, and the character tier decides at its own barrier. The link
is a reference, not a controller. State that in the impact review and assert it
with a test: a promoted soldier moves by the same pass as an unpromoted one.

The two-tier answer changes nothing else in the list above. It does not add a
column, and it does not move the barrier.

## Impact review

Not done. This item is `proposed` and needs one before anyone takes it. Two
questions are open and this item cannot invent the answers.

- Which decision records govern the promotion pass, by number and decision.
- Whether the promotion needs a record of its own, or whether the tier record
  and the key vector record already bind it.[^3] [^5]

**The third question is answered.** BLK-004 gives the ceiling, at 50,000 living
characters, and the register carries it as a resolved row.[^8] **The budget in
point five is not new work.** The character arena is built at the ceiling of
its declared tier and refuses a capacity above it, so the allocation already
cuts against the bound. What point five must add is the cut at the rank, not
the bound itself.

**BLK-011 is also resolved**, and the item already states its content in the
lineage paragraph above: a promoted soldier founds a new house, his kinship to
everyone is zero, and he cannot inherit by blood.[^4] The paragraph and the
register agree, so nothing here needs a decision.

## What the audit found, and why it did not take this item

An audit measured the demonstration world and found no character at any tick,
which is what moved this item up the index.[^9] It did not take the item, for
one reason that a later worker should read before planning.

**Points one, two and six each add a column to the unit row.** An achievement
value, an eligibility bit and a link to the character are three columns, and
the unit arena is the storage that a separate item is reordering for
locality.[^10] The two changes touch one file for opposite reasons. Sequence
them rather than merging them.

**Do not close the zero with a character that nothing reads.** A founding could
name a leader in one call and the count would stop being zero. That would
declare a capability nobody invokes, which the rules forbid, and it would
remove the signal that says the tier is unused.[^11] The zero is honest until a
character decides something, and item 0068 is what makes one decide.

## References

[^1]: Backlog item 0066. `docs/backlog/complete/0066-provide-the-character-column-set-and-the-tier-declaration.md`
[^2]: The character graph and inheritance, section 9. `docs/research/reports/14-character-graph-and-inheritance.md`
[^3]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D4. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^4]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^5]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^6]: Decisions register, DEC-002. `docs/DECISIONS.md`
[^7]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^9]: Findings register, FND-269. `docs/FINDINGS.md`
[^10]: Backlog item 0266, order the unit arena by cell. `docs/backlog/proposed/0266-order-the-unit-arena-by-cell.md`
[^11]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
