# ADR-0197: A verb names a place by a cell of the egocentric frame the observation publishes

## Context

A faction acts by one integer. The integer indexes a bounded table the engine
declares, and the engine publishes the layout of that table in a schema.[^1] A
verb of the table declares its own argument positions, and the integer is a
mixed radix over the positions of the verb it selects.[^2]

**No row of that table names a place.** The engine resolves the place inside
the verb. A policy can therefore say "raise a campaign" and it cannot say
where. It can say "found a settlement" and it cannot say where. A research
report judges a spatial argument the largest single gain available in the
table, and also the most expensive change.[^3] The same report finds no
published macro action space whose rows name no place at all.[^4]

A draft record forbids the change today.[^5] It permits a verb to narrow the
set it acts on by a bounded categorical position, and its third decision
states that no such position names a place. It gives two costs. The legality
answer returns one byte for each row, so a table indexed by the map lattice
would make one decision walk the world. A weight file is a function of the
table length, so a table that followed the world extent would part every
policy from every world of another size.

**Both costs follow from one premise.** The premise is that a place is an
index into the extent of the world. That premise was true when the record was
written, because the observation held one position for each cell of the map
lattice and nothing else named a place.

**The premise no longer holds.** The observation now reads an egocentric frame
of rings and angular sectors. The frame has a centre that belongs to the
reading faction, rings at geometric radii, and sectors anchored to the world
axes. The cell count of the frame is a constant of the frame and it follows
nothing about the world.[^6] A place named in that frame is bounded by a
constant, so neither cost applies to it.

### What makes the choice hard

**Invariance is the requirement, not a preference.** A policy trains at one
map size and one faction count and runs at another. That property is the whole
reason the observation was rebuilt.[^6] A place argument that broke it would
be worse than no place argument, because it would retire every policy and
return nothing that transferred.

**The controller's choice must still land on one row.** The verb set of the
table is the choice enumeration of the built-in controller plus one no-op
row.[^7] A draft record reads a window of controller commands as one label
distribution over the table.[^8] The controller names no place. A verb that
gained a place position and offered no row for the controller's own choice
would break that reading.

**A place must not tell a faction what it has not seen.** The legality answer
already holds that rule for the two verbs that act on a place. Both resolve
their target through readers that answer for one faction, so a legality byte
states nothing about unseen ground.[^9] A place position brings the rule to
the argument as well.

**A narrowing resolved by a draw is already forbidden.** The narrowing record
finds a violation when the engine resolves a narrowing by a draw.[^5] Two of
the verbs that act on a place take their target from a keyed sample of the
world.[^10] That closes the question for those two, whatever this record
decides.

## Decision

**A verb names a place by a cell of the egocentric frame the observation
publishes, and by nothing else.**

This record changes ADR-0184 D3. It replaces the clause that forbids every
place. It keeps the constraint underneath that clause, which is that no bound
of the action schema follows the world extent. Every other decision of ADR-0184
stands.

### D1. A place is a cell of the egocentric frame, and the action reads the frame the observation reads

A place position of a verb carries a cell of the egocentric frame. The frame
has a centre that belongs to the acting faction, rings at geometric radii
about that centre, and angular sectors anchored to the world axes.

**The action and the observation read one frame.** The centre rule, the ring
rule and the sector rule have one statement in the engine, and both the
observation reader and the action table call it. A cell index therefore names
the same ground in the array a policy reads and in the integer that policy
emits.

A reviewer finds a violation when a place carries a tile index, a lattice cell
index, a coordinate, or any other index into the extent of the world. A
reviewer also finds one when the action table computes a centre, a ring or a
sector of its own, rather than calling the rule the observation calls.

### D2. Index zero of a place enumeration is the whole frame

The first value of a place enumeration names no cell. It says that the engine
resolves the place over the whole frame, which is the answer the verb gave
before the position existed.

This keeps four things true at once. The controller's own choice lands on one
row, so the verb set does not move and the record that fixes it stands.[^7]
[^8] A policy that learns nothing about places acts as a policy did before the
position existed. The whole-set rule of the narrowing record holds in the same
form it holds for every other narrowing.[^5] And the position carries no
content that the engine resolves, so the rule that forbids a second
declaration of the engine's own answer holds.[^11]

**A place position narrows the region the engine resolves within. It never
carries the target the engine resolves.** The engine still chooses the exact
target, by the rule it already owns, over the candidates that fall in the
named cell.

A reviewer finds a violation when the first value of a place enumeration names
a cell, when a controller choice maps onto a row that names a cell, or when a
place position carries a target rather than a region.

### D3. A place is legal only where the acting faction observes a candidate

The legality answer resolves the candidate set of the verb, restricted to the
named cell, through the readers that answer for one faction. A cell in which
the faction observes no candidate reads as not legal.

**The answer restates no refusal rule.** One reader answers which candidate
each cell holds, the legality answer reads it, and the verb reads the same
reader. A row the answer allows and the verb then refuses is therefore a
defect a test can find, in the way it already is for every other row.[^12]

This is a fact about ground the faction has already seen, so it leaks nothing.
It is the same rule the two place-resolving verbs already keep.[^9]

A reviewer finds a violation when the legality of a place row depends on
ground the faction has not observed, when the answer holds a candidate rule of
its own, or when the answer and the verb read two readers.

### D4. No bound of a place position follows the world

The bound of a place position is the cell count of the frame, plus the whole-
frame value of D2. The cell count is a constant of the frame.

**The legality answer is therefore a fixed length at every world size and at
every faction count**, and the length of the whole table stays a function of
the world parameters and of constants alone.[^1] A weight file fits a world of
any extent.

A reviewer finds a violation when a place bound reads the grid width, the grid
height, the tile count, the lattice cell count, the settlement count or the
unit count.

### D5. A verb takes a place only when it resolves its candidates without a draw

A verb declares a place position only when its candidate set is a reading of
the world and of what the faction observed. A verb whose target comes from a
keyed sample of the world declares no place position, because narrowing a draw
is a narrowing the engine resolves by a draw, and that is forbidden.[^5]

A verb that resolves without a draw and declares no place position is not in
violation. This decision states which verbs may take a place, and it does not
oblige one to.

A reviewer finds a violation when a verb declares a place position and
resolves its candidates by a draw.

### D6. A place the verb cannot honour is a refusal, and the refusal writes the log row

A verb that finds no candidate in the named cell changes nothing and reports
that it did not act. There is no second path and no fallback to another cell.

The engine writes one row of the command log for the action, with the action
integer and the byte that says the verb did not take it, in the way it does
for every refused row.[^13] The verb does not read the legality answer before
it acts.

A reviewer finds a violation when a verb acts on a place other than the one
named, when a refused place writes no log row, or when a verb consults the
legality answer before it runs.

## The alternatives this rejects

**A tile index or a lattice cell index.** Rejected under D4, and this is the
alternative ADR-0184 D3 refused. The bound follows the world extent, so the
legality answer walks the world and every stored policy parts from every world
of another size. Nothing in this record weakens that refusal.

**A frame that turns with the acting faction.** Rejected because the meaning of
a sector would drift between two decisions of one episode, and the observation
reads a frame anchored to the world axes.[^6] Two frames would name two
different places by one index, which is the defect shape this project names
first.[^11]

**A candidate list of the settlements the faction has observed, by slot.**
Rejected because a slot is an identity, and a narrowing position never carries
one.[^5] The slot bound also follows the site cap of the world rather than a
constant.

**A place carried beside the action integer.** Rejected because a field of an
action would then live outside the action, which the log decision refuses.[^13]

**Let a verb narrow the founding survey by cell.** Rejected under D5. The
survey is a keyed draw, so the legality of a cell would follow the sample
rather than the ground, and the narrowing record forbids a narrowing the engine
resolves by a draw.[^5] [^10] A record that admits it must first state what
governs a narrowed draw.

**Raise the resolution of the spatial observation instead, and leave the table
alone.** Rejected because no observation field lets a faction say where. The
report that ranked the two states this plainly, and it ranks the action
argument first.[^3]

**One verb for each cell.** Rejected because a parameter of a verb does not
multiply the verbs, and because the verb set is the controller's
enumeration.[^7]

**Supersede ADR-0184.** Rejected because five of its six decisions hold
unchanged, and a record whose subject is one corrected clause is a record for a
constraint rather than for a topic.[^14]

## Consequences

**Every stored policy parts from the table.** A place position moves every row
above the verb it joins, so the action version moves with it. A reader refuses
a weight file whose stated fit is not the table it plays, so the break is loud
rather than silent.

**The golden state hash moves.** The command log carries the action integer,
and the integers above the verb that gained the position are renumbered.

**The table grows by a constant factor on the verb that takes a place.** Every
factor is a constant, so the length stays independent of the population and of
the extent.

**The legality answer costs one more resolver pass for that verb.** The pass is
bounded by the candidate set the verb already reads, and not by the cell count,
because one pass answers every cell.

**A place is coarse far from the centre.** Resolution falls with distance in the
frame, so a policy names a direction and a distance band, and never a tile. A
player who wants a named tile still has no verb for it.

**The extension path is open and it needs no further record.** A verb that
resolves its candidates without a draw may declare a place position under D5.
The verbs whose target comes from a keyed founding survey may not, and a record
that admits them must govern a narrowed draw first.

**This record holds no figure.** The ring count, the sector rule and the cell
count of the frame are structural constants the engine derives, and the engine
publishes them in the observation schema.[^6]

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decisions D1, D4 and D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^3]: Report 40, what a well-trained policy needs, sections 5.3 and 6.1. `docs/research/reports/40-what-a-well-trained-policy-needs.md`
[^4]: Report 34, what would move the learner, sections 3.2 and 8. `docs/research/reports/34-what-would-move-the-learner.md`
[^5]: ADR-0184, a verb narrows its set by a bounded categorical position, decisions D1, D2, D3 and D4. `docs/adrs/draft/adr-0184-a-verb-narrows-its-set-by-a-bounded-categorical-position.md`
[^6]: ADR-0195, the observation of a faction is a fixed-width scale-free table in an egocentric frame. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^7]: ADR-0176, an action integer is a mixed radix over the positions a verb declares, decision D4. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^8]: ADR-0192, a window of controller commands is one label distribution over the action table. `docs/adrs/draft/adr-0192-a-window-of-controller-commands-is-one-label-distribution.md`
[^9]: PRD-0001, a faction sees only what it observes. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^10]: ADR-0075, the founding choice reads a bounded sample of the world, decision D1. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^11]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^12]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D5. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^13]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D6. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^14]: Decision Record Scope, sections 2 and 3. `.agents/rules/adr-scope.md`
