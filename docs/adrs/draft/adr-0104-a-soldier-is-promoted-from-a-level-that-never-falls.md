# ADR-0104: A soldier is promoted from a level that never falls

## Context

The engine holds a character tier with its own storage, its own record of
descent, and a relation between any two members of it.[^1] Nothing in the world
created a member. A character arrived only because a caller asked for one, so a
world left to run held no character at any tick, and every capability built on
the tier was unreachable.[^2]

A story needs somebody it is about. The pool a named person comes from is the
soldier population, and the question this record answers is how one of them
becomes a person.

**The obvious method is a scan for an event.** A rule watches for a soldier
doing something notable and promotes it there and then. That method makes the
promotion depend on the moment the engine happened to look, and it puts the
decision inside whatever pass noticed. A contributor could reasonably write it.

The alternative is a level. Each soldier carries a value that records what it
has done, a separate pass reads the value at its own barrier, and the promotion
follows from the value rather than from the moment. **A level can be scanned
lazily and an edge cannot**, and that difference is what makes the pass
affordable at a population of one million.

The choice is not free. A lazy scan over a level is correct only while the
level rises. A rule that lowered the value would break the scan in silence, and
nothing in the type system says so. That is the constraint this record exists
to carry.

## Decision

### D1. A soldier carries one level that records what it has done

The unit row holds a running total of what the soldier gathered, summed over
every kind. The total rises where a soldier is given what it gathered, and
nowhere else, so the rule has one write site.

The value is an integer count of an integer amount. No part of it is
fixed-point and no part of it rounds.[^3]

The alternative of deriving the level from the state a soldier holds now is
rejected. What a soldier carries falls when it delivers, and a level that falls
cannot be scanned lazily. **The level records the deed, not the goods.**

### D2. The level never falls, and a check states it

Nothing may lower the level while the soldier lives. The add saturates, because
a total that wrapped would fall.

**This is a constraint on the content and not an implementation detail.** A
later rule that spent the level, decayed it, or reset it on any event would
leave the scan reading a stale answer, and no test of that rule would fail.

A second column states whether a soldier reached the threshold, so the scan
reads one byte for each soldier rather than eight. That column is a second
statement of a comparison the level already answers, and the invariant check
fails when the two disagree.[^4] A comment naming which copy wins would not be
enough, because the failure is silent.

### D3. A promotion creates a character and links the soldier to it. It changes neither row's tier

An entity declares its tier when it is created and never changes tier while it
lives.[^5] The soldier therefore does not become a character. The pass creates a
character and stores its identity on the soldier.

**The link runs one way, from the soldier to the character.** One direction
cannot disagree with the other. The other direction would also be wrong more
often than right: a character outlives the body that earned it, so a link held
by the character would name a dead soldier for the rest of that character's
life.

The stored value is a whole identity and never a bare slot index, so a
character that was removed does not resolve to the character created next in
its slot.[^6]

**The link is a reference and never a controller.** The soldier keeps the
decision it already makes, and the character tier decides at its own barrier. A
promotion adds no second decision site, which is the failure this project meets
most often.[^4] [^7]

A promoted soldier gets no ancestry. The character founds a new line and cannot
inherit by blood. A title holder may appoint them.[^8]

### D4. The rank is a key vector, and the identities are allocated after the cut

The eligible set is collected in ascending slot order and ranked by a key
vector whose last field is the whole identity, so no two candidates tie and the
order does not depend on the order the set was read in.[^9] [^10]

The budget cuts the ranked set. **The identities are allocated after the cut and
never during the scan**, because a pass that minted an identity while scanning
would spend a character slot on a candidate the budget then rejected.

The budget is the cut at the rank. It is not a second statement of the
population ceiling: the character storage is built at the ceiling of its
declared tier and refuses to exceed it, so the ceiling has one enforcement
site.[^11]

### D5. The scan runs at its own barrier, on a schedule, after the deaths of the frame

The pass does not run on every tick. A pass over every soldier on every frame
spends the frame for an answer that changes rarely, and the level it reads only
rises, so a later scan sees everything an earlier one would have.

The interval is a parameter that the world carries. No kernel holds it as a
constant.

The scan runs after the pass that ends a soldier, so it never promotes a
soldier that died in the same frame.

## Consequences

**The character tier is reachable from a running world.** A world left to run
now produces characters, so a reader, a watcher and a test can all meet one.
Before this, every capability built on the tier was unreachable and a test over
the tier measured its own fixture.[^2]

**The project cannot add a rule that lowers the level without superseding D2.**
That includes spending it, decaying it over time, and resetting it on any event.

**A promoted soldier is embodied, and D3 is what keeps that from costing a
defect.** One character row and one unit row name one person. If a later change
lets the character row decide where the person goes, one fact gains two
authorities and nothing fails when they disagree.

**What makes a soldier notable is one quantity, and it is a narrow one.**
Gathering is the only deed the engine counts, because gathering is the only
thing a unit does that the engine records per unit. A world whose units fight,
build or travel would want those too, and adding one is adding a term to D1
rather than changing this record.

**Nothing reads a character yet beyond the reader that names one.** The tier
now has members and no rule consults them. That is the next item and not this
one.[^12]

## References

[^1]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion. `docs/adrs/REGISTRY.md`
[^2]: Findings register, FND-269. `docs/FINDINGS.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D4. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^6]: ADR-0014, entity identity is an index plus a generation, decision D2. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^7]: Decisions register, DEC-002. `docs/DECISIONS.md`
[^8]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^9]: ADR-0007, content supplies a key vector, never a comparator, decisions D1 and D2. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^10]: ADR-0004, iteration order is explicit, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^12]: Backlog item 0068, give a faction a ruler and a succession. `docs/backlog/refined/0068-give-a-faction-a-ruler-and-a-succession.md`
