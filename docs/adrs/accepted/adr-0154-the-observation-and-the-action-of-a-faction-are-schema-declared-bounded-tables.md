# ADR-0154: The observation and the action of a faction are schema-declared bounded tables the engine owns

## Context

A product record asks for a learner that plays one faction against the
built-in controllers.[^1] The engine already holds most of what such a learner
needs. A caller marks a faction as externally controlled, and the built-in
controller then skips that faction.[^2] A caller acts on a faction through
verbs, and the built-in controller emits its commands through the same
verbs.[^3] The engine reads the standing of a faction on each win path.[^4] It
reads a faction's relation row, its trade board and its weight vector.[^5] [^6]

Nothing says what shape those readings and those actions take when they cross
to a learner. A learner is not an ordinary caller. It reads one array on every
decision, and it trains a function of that array's layout. A change to the
layout changes the meaning of a trained policy, and nothing in a weight file
says which layout produced it. A learner also needs the set of actions it may
take, in a form a network can index.

Design documents work this problem out, and they name this record.[^7]
[^8] The shortest implementation is a reader that returns a dictionary of
named values, and a Python file that lists the action rows. A contributor would
reach for both. A dictionary is readable, and a Python list is quick to write.

The facts below refuse that. **A learner needs one fixed shape.** Every backing
system builds its input space once, from a fixed length and a fixed bound for
each position. A dictionary states neither. **A Python list of offsets is a
second declaration site.** The layout would live in the core crate and in a
Python file, and nothing would fail when the two disagreed. That is the first
defect shape this project names, and the project has already met it.[^9] **An
action set of the learner's own doubles a declaration.** The built-in
controller already chooses from a closed set. A second set for the learner
states the same fact twice, and it destroys the imitation target that the
controller's own log gives for free.

**This record runs ahead of the code, and every decision below is a constraint
on work that nobody has written.** The engine holds no schema, no observation
reader and no action table. No reader takes a viewing faction, so every reader
answers with the truth about the whole world. The engine holds the inputs a
sight rule would need, and it holds no sight rule.[^10] [^11] The controller's
own log holds a kind and a one-byte argument, so a campaign's target tile lives
in a second log and no command carries a magnitude.

The owner has ruled that the harness shows a learner only what a player of that
faction could see.[^12] That ruling and the product record that states the same
rule govern the observation.[^13]

## Decision

**The engine owns one observation table and one action table for a faction. It
declares the layout of both in a schema it returns, and that schema is the only
declaration of the layout.**

### D1. The engine returns a schema, and no caller states an offset

The engine returns a schema for the observation and a schema for the action
table. The observation schema names each field, the position where the field
starts, the number of positions it holds, its integer width, and the lower and
upper bound of each of its positions. The action schema names each verb, the
kind of thing its candidate list names, the ceiling on that list, and the
number of magnitude buckets the verb takes.

The engine builds each schema from the same constants that build the array. The
start of a field is the sum of the lengths before it. No file outside the engine
states a position, a length or a bound. A caller that needs one asks for the
schema.

A reviewer finds a violation when a file outside the core crate states an
offset, a length, a bound or an action count as a literal. A test builds the
observation, reads the schema, and asserts that each field sits where its row
says.

### D2. An observation is one flat array of integers of fixed length

The observation of one faction is one flat array of signed integers. Its length
is a function of the world parameters and never of the population. It holds no
floating point number, because the simulation holds none and a learner reads
what the simulation holds.[^14] A fixed-point value crosses as its raw integer,
and the learner scales it in Python. Floating point is allowed on the learner's
side of the boundary, and only there.[^15]

The reader starts no pass over the tiles or the units. It reads the aggregates
the engine already keeps and the summary the pyramid already rebuilt.[^11]

A reviewer finds a violation when the length depends on the unit count, when a
position holds a float, or when the reader walks the world.

### D3. An observation holds only what its faction observes, and the engine enforces it

A caller passes a faction and receives what that faction sees. No argument asks
for the truth. The engine applies the rule inside the reader, so a learner
cannot see through fog. The interface offers no way to ask for more.[^13] [^12]

**This decision binds the reader, and it does not decide the storage.** How the
engine holds what a faction has observed is a separate claim, and the registry
reserves a number for it.[^16] This decision stays true whatever that record
decides.

A reviewer finds a violation when a reader that names a faction returns a value
that faction has not observed, when an argument lets a caller ask for more, or
when a count over the whole world enters an observation.

### D4. An action is one integer that indexes a bounded table the engine declares

An action is one integer. It indexes a table the engine declares, and the table
factorises into a verb, a target and a magnitude. The target is an index into a
bounded candidate list for that verb, and the world parameters bound every list.
The magnitude is a bucket from a fixed set, and the bucket edges are balance
rows.[^17] The engine exposes the factorisation in the schema, as a mixed radix
over the verb offsets, the candidate bounds and the bucket counts. A caller
decodes an integer by arithmetic over the schema, never by a table it holds.

The verb set of the table is the set that the built-in controller's choice
enumeration holds, plus one no-op row. That enumeration lives in the core crate,
and the schema derives its verb rows from it.

The engine resolves the candidate list at the tick the action applies, and it
orders the list by a stable key.[^18] It then applies the action through the
same verbs a caller and the controller use.[^3] A row the verb refuses is
dropped and counted, as a controller command is.[^19] A batch of actions applies
in an order the data fixes.[^20]

A reviewer finds a violation when a candidate list has no bound, when a bound
follows the population, when an action reaches a store without a verb, or when
the applied order depends on the order a caller wrote the rows.

### D5. The engine answers which actions are legal now

The engine returns one byte for each row of the action table. The byte says
whether the verb would refuse that row at this tick. The answer reads the same
candidate lists and the same refusal rules the verbs read, and it duplicates no
rule. A padding row of a list shorter than its bound reads as not legal.

The no-op row is always legal, so the answer is never empty. A learner therefore
never learns legality by trial.

A reviewer finds a violation when the answer restates a refusal rule that a verb
owns. A row the answer allows and the verb then refuses is a defect, and a test
compares the two over a seed set.

### D6. One log records the controller's choice and the learner's action in one encoding

The engine writes one row to one log when a verb applies an action, whatever
chose it. The row holds the tick, the faction, the action integer of D4, and
whether the verb took it. It is plain data with a declared layout, as every
event in this engine is.[^21] The engine writes it, so the log is the engine's
record and not a wrapper's.

**The row carries the whole action, so no field of it lives anywhere else.** A
target and a magnitude survive in that row, and never in a second log beside it.
A second log for the learner would be a second declaration of one fact.[^9]

A reviewer finds a violation when a learner's action and a controller's choice
reach two logs, or when a field of an action survives only outside the log.

## The alternatives this rejects

**A dictionary observation.** Rejected because a learner needs one fixed shape
and one fixed bound for each position, and a dictionary states neither. A
backing system would flatten it, and the flattening order would then be the
layout, declared in the flattener and nowhere else.

**A per-caller schema, held in Python.** Rejected because it is a second
declaration site for one layout. Nothing fails when the Python file and the core
crate disagree, and the wrong answer is a valid array of wrong numbers.[^9]

**A flat one-hot over every target.** Rejected because the width would follow
the population. A verb that names a unit or a tile would give an action space
that grows with the world, and the cost of one decision would then follow the
population rather than the lattice.[^22] A bounded candidate list keeps every
bound a world parameter.

**Legality by exception.** Rejected because a learner that finds legality by
trial spends most of its decisions on refused rows. The refusal is also
something the engine already computes, so hiding it costs the learner and saves
the engine nothing.

**The truth to a learner, with fog added later.** Rejected because a policy
trained on the truth is not the policy of a player. The owner ruled against
it.[^12]

## Consequences

**The layout binds a trained policy.** A field added, removed, relengthened or
rebounded changes the meaning of a stored weight file. The engine therefore
carries one version integer beside the schema, and a learner that loads a policy
under a different version stops with an error that names both. The version and
the test that holds it are code, and not a record.

**The full-information readers stay, and the learner's path does not call
them.** A modeller and a reproducer read the truth, and neither is the audience
of the product record. A check keeps the learner's path off those readers.

**A count over the whole world cannot serve an observation.** A reader that
serves the observation of a faction counts the events of that faction only, so
the readers that count over the whole world do not serve it.

**The map block waits for the observation plane, and the rest does not.** The
blocks that hold a faction's own standing, its own relation row, the public
board and its own weights pass without a mask. The schema, the action table, the
legality answer and the log can be written before the plane lands.

**The controller's log changes shape.** It gains the action integer, and a
campaign's target stops living in a second log. Every reader of that log changes
with it.

**A new verb moves every action integer above it.** The table is a mixed radix,
so a new verb or a new bound renumbers the rows above it. That is why the
version exists.

**This record holds no figure.** Every bound, every length and every bucket edge
must be a row in the reference tables, and every cost figure stays derived until the
target platform measures it.[^17] [^23]

**A learner stays a control plane.** It sends one integer for one faction and
reads one array back. It never loops over entities, so the boundary carries an
instruction and an answer, and never the population.[^24]

## References
[^1]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^2]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D6. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^3]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^4]: ADR-0148, a game end is recorded once and stops the controllers, decision D1. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^5]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold, decision D1. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^6]: ADR-0149, a faction's trade board is simulated state that any faction may read, decision D4. `docs/adrs/accepted/adr-0149-a-factions-trade-board-is-simulated-state-that-any-faction-may-read.md`
[^7]: Design, a learner plays one faction against the controllers, sections 4 and 5. `docs/superpowers/specs/2026-09-05-reinforcement-learning-design.md`
[^8]: Design, one environment core serves every learning stack, sections 2 and 3. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^9]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^10]: ADR-0111, the presence relation is derived at the end of the step, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^11]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^12]: Design, one environment core serves every learning stack, the owner's ruling of 5 September 2026. `docs/superpowers/specs/2026-09-05-reinforcement-learning-interfaces-design.md`
[^13]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^14]: ADR-0002, state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^15]: ADR-0002, state holds no floating point number, decision D4. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^16]: ADR Registry, the reserved row for fog storage. `docs/adrs/REGISTRY.md`
[^17]: Balance register. `docs/reference/balance.md`
[^18]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^19]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^20]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D5. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^21]: ADR-0006, an event is plain data and applying it is pure, decision D1. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^22]: ADR-0096, cost follows the lattice, not the population. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^23]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^24]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
