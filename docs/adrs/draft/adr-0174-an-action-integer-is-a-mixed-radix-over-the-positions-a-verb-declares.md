# ADR-0174: An action integer is a mixed radix over the argument positions each verb declares

## Context

A learner plays one faction while the built-in controllers play the rest.[^1] An
accepted record says how that learner acts. It says an action is one integer,
that the integer indexes a bounded table the engine declares, and that the
engine exposes the table in a schema.[^2] Two accepted records rest on that
claim. One says a learner reaches the unit queue through the action table.[^3]
One says a learner acts through a bounded action table rather than through a
term the choice pass always applies.[^4]

That record also fixed the shape of the table. It said the table factorises
into a verb, a target and a magnitude, and it said the verb set is the set the
built-in controller's choice enumeration holds, plus one no-op row.[^2]

**The enumeration does not have that shape, and it never did.** The engine owns
the enumeration, and each of its choices was added under a record of its
own.[^5] [^6] [^7] [^8] [^9] Three shapes sit in it that a verb, a target and a
magnitude cannot hold.

- **One choice names two things.** A campaign carries an objective kind and an
  objective tile. One target position holds one of them.
- **Several choices name nothing.** A board rewrite, a negotiation step, a
  carrier assignment, a project send and a founding are each a command over the
  whole faction. The engine resolves what each one touches at the tick the
  command applies, so no target of theirs exists to encode.
- **No choice carries a quantity.** A magnitude is a position of the
  factorisation, and no choice supplies one. The register that would hold the
  bucket edges holds no row for them.[^10]

**The record is the thing that is wrong, and not the engine.** No source file
implements the record, so no code contradicts it. The record stated a property
of an enumeration that other records own, and it stated the property as a fact.
The rule on what a record may hold names that failure: write the constraint the
code must satisfy, and never write what you hope the code will do.[^11] To bend
the enumeration to fit the encoding would run the authority backwards. It would
split one choice into two verbs, and give a quantity to choices that take none,
so that a boundary encoding could keep its shape.

**The constraint underneath the shape is sound, and this record keeps it.** A
learner needs one integer, one bounded table, and a decoding it derives from a
schema rather than from a table of its own.[^2] Only the fixed triple fails.

A register holds the disagreement and its evidence.[^16]

The freeze on an accepted record governs the repair. The retcon window is shut,
because two accepted records cite the decision this record changes.[^12] [^3]
[^4] So this record changes that decision, and every other decision of that
record stands.

## Decision

**A verb declares its own argument positions. The action integer is a mixed
radix over the positions of the verb it selects, and a verb declares as many
positions as it needs, including none.**

This record changes ADR-0154 D4. It replaces the clause that fixes the
factorisation as one verb, one target and one magnitude. It keeps every other
clause of that decision, and it keeps every other decision of that record.

### D1. A verb declares its argument positions, and the schema states them

The action schema names each verb, and for each verb it names the argument
positions that verb takes, in order. Each position names its kind and its
bound. A position of the candidate kind names the kind of thing its list holds
and the ceiling on that list. A position of the bucket kind names how many
buckets it holds.

The world parameters bound every candidate list, and a bound never follows the
population. The engine builds the schema from the same constants that build the
table.[^2]

A caller decodes an integer by arithmetic over the schema, and never by a table
it holds.

A reviewer finds a violation when a position has no bound, when a bound follows
the population, when a file outside the core crate states a position or a bound
as a literal, or when a verb takes an argument the schema does not name.

### D2. A verb that resolves its own content declares no argument position

A verb whose content the engine resolves at the tick the action applies takes
no argument position, and it holds one row of the table. The learner selects
the verb, and the engine resolves what the verb touches, in the way it resolves
it for the built-in controller.

Such a verb is not an incomplete action. It is a whole action whose content is
a function of the state at the tick it applies, and the engine owns that
function. **Do not add a position to carry that content.** A position that
restated it would be a second declaration of one fact, and the engine's answer
would be the one that acted.[^13]

The no-op row is a verb of this kind, and it is always legal.[^14]

A reviewer finds a violation when the schema names an argument position that
the verb ignores, or when an action carries a field the verb resolves for
itself.

### D3. A magnitude is one kind of position, and never a position every verb holds

A magnitude is a bucket position. A verb that takes a quantity declares one,
and the edges of its buckets are rows of the balance register.[^10] A verb that
takes no quantity declares none, and its rows hold no bucket position at all.

A reviewer finds a violation when a verb declares a bucket position that
nothing reads, when a bucket edge sits in this record or in any record, or when
a caller states an edge.

### D4. The verb set still comes from the controller's enumeration

The verb set of the table is the set the built-in controller's choice
enumeration holds, plus one no-op row, and the schema derives its verb rows
from that enumeration. This restates the clause of ADR-0154 D4 that this record
keeps, so that a reader of this record alone does not read the enumeration as
open.

A new choice in that enumeration therefore adds a verb, and it renumbers every
row above the verb it joins. That renumbering is the reason the schema carries
a version.

A reviewer finds a violation when the table holds a verb the enumeration does
not, or when the learner reaches a store through anything but the verbs a
caller and the controller use.[^5]

## The alternatives this rejects

**Reshape the enumeration to fit the triple.** Rejected because a boundary
encoding may not decide what the simulation offers a faction. It would split
one choice into two verbs and invent a quantity for choices that take none, and
each of those choices sits under a record of its own.[^6] [^7] [^8] [^9]

**Keep the triple and give the campaign a flattened candidate list.** Rejected
because the flattening would be a second declaration of the pair, held in
whatever code flattened it. A reader of the schema would see one target where
the verb takes two, and the pairing rule would live nowhere the schema
states.[^13]

**Carry an escape payload beside the integer for the choices the triple cannot
hold.** Rejected because a field of an action would then live outside the
action, which the log decision of the same record refuses.[^15]

**Supersede the whole record.** Rejected because the other decisions hold, and a
record whose subject is one corrected clause is a record for a topic rather than
for a constraint.[^11]

## Consequences

**The schema grows one level.** A verb row now holds a list of positions, rather
than one target bound and one bucket count. A reader of the schema walks that
list.

**A verb with no position stays cheap, and the table stays small for it.** The
learner sees one row for such a verb, so the table does not widen to carry a
shape that no verb uses.

**The legality answer is unchanged in kind.** It still returns one byte for each
row of the table, and a padding row of a list shorter than its bound still reads
as not legal.[^14]

**The two records that cite the changed decision stand.** One reaches the queue
verb through the table, and one keeps a preference out of the choice pass
because the learner acts through the table.[^3] [^4] Neither rests on the
factorisation this record replaces.

**This record holds no figure.** Every bound and every bucket edge is a row of
the reference tables.[^10]

## References

[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^3]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, the context. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^4]: ADR-0156, a faction's option weights are policy, set through one verb, the context. `docs/adrs/accepted/adr-0156-a-factions-option-weights-are-policy-set-through-one-verb.md`
[^5]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^6]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D3. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
[^7]: ADR-0159, a project order names one category for each unit and one seed set for the faction, decision D1. `docs/adrs/accepted/adr-0159-a-project-order-names-one-category-and-one-seed-set.md`
[^8]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^9]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D2. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^10]: Balance register. `docs/reference/balance.md`
[^11]: Decision Record Scope, sections 3 and 4.6. `.agents/rules/adr-scope.md`
[^12]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`
[^13]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^14]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^15]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^16]: Findings register, FND-565. `docs/FINDINGS.md`
