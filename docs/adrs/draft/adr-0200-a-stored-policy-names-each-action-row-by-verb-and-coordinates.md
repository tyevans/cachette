# ADR-0200: A stored policy names each row of the action table by its verb and its candidate coordinates

## Context

A faction acts by one integer. The integer indexes a bounded table the engine
declares, and the engine publishes the layout of that table in a schema.[^1] A
verb of the table declares its own argument positions, and the integer is a
mixed radix over the positions of the verb it selects.[^2]

A trained policy scores every row of that table. The readout therefore holds
one weight vector for each row, and a stored policy is a function of the
layout the schema published when the run wrote it.

**The engine publishes one version integer beside the action schema.** A
stored policy records that integer, and the reader refuses a file whose
recorded integer is not the integer the engine publishes now. An accepted
record states this as a consequence of the layout binding a trained
policy.[^3]

**One integer cannot say which rows changed meaning.** The engine moves the
version whenever it moves a row, and it must, because a caller decodes a row
by arithmetic over the schema. So a change to one verb invalidates every row
of every stored policy, including the rows that mean exactly what they meant.

That happened. A place argument was added to the campaign verb.[^4] The verb
gained rows, and every verb after it in the table moved to a higher index.
Every stored policy of the project was refused and deleted. Most of their rows
had not changed meaning at all: the gather verb sat where it sat and named the
resource kinds it had always named, and a verb above campaign moved without
changing what any of its rows means.

**A sketched change will do it again.** The place record leaves the crossing
verb and the settling verb with no place position, and it names the condition
under which a later contributor may give them one.[^5] A contributor who takes
that path moves the table again, and every policy trained under the present
table dies the same way.

### What makes the choice hard

**A row index is not an identity.** The index of a row is a position in a
layout, and the layout is a function of the verb set and the bounds. The
identity of a row is what the row means: a verb, and one coordinate for each
argument position that verb declares. The engine already publishes both, so
the identity is available and the reader throws it away.

**Refusing a file is the safe failure, and it is not free.** A refusal costs a
whole training run. Accepting a file whose rows moved under it costs a policy
that scores the wrong verb, and that failure is quiet: the run continues, the
scores drift, and nothing says why. Whatever this record decides must fail
loudly on the second case in order to earn the first.

**A new row has no stored weight, and every value for it is a claim.** A
change that adds rows adds rows the file says nothing about. Zero is the
obvious answer and it is not a neutral one. A trained readout scores the verbs
it learned to use above zero, so a row filled with zero loses to them at every
decision. A resumed run therefore starts biased against exactly the capability
the change added, and it stays biased until training moves those weights,
which it can only do by first choosing the row it will not choose.

**The observation has the same defect and it is not this record's.** The
observation also carries one version integer, and a field added to it also
retires every stored policy. A draft record owns the observation layout and
its revision.[^6] This record touches neither.

## Decision

**A stored policy names each row of the action table by its verb and its
candidate coordinates, and the loader rebuilds the readout by that identity.**

This record changes one consequence of ADR-0154, the consequence that a
learner which loads a policy under a different layout version stops with an
error naming both. It changes that consequence for the action table alone. The
observation half of it stands, and every numbered decision of ADR-0154 stands.

This record extends ADR-0176 D1. That decision makes the schema the only
declaration of the table and requires a caller to decode a row by arithmetic
over the schema. It states no rule for a caller that must decode a row against
a schema the engine no longer publishes. This record states it.

### D1. A checkpoint stores the action table the engine published, and derives it

A checkpoint records, for each verb of the table, the name of the verb, the
index of its first row, how many rows it holds, and for each argument position
the candidate kind, the bound and the stride. Every one of those values is
read from the schema the engine publishes at the moment the file is written.

**No file of this project states a verb, a bound or a stride as a literal.**
A hand-written verb list would be a second declaration of what the engine
publishes, and nothing fails when two declarations disagree.[^7] The stored
table is a copy of one publication and never a restatement of the rule that
produced it.

A reviewer finds a violation when a source file outside the engine names a
verb of the table, a candidate bound, or a stride.

### D2. A stored row is named by its verb and its candidate coordinates

The identity of a stored row is the name of its verb, and the coordinate of
each argument position that verb declared when the file was written. The
coordinates are the mixed-radix digits of the row within its verb block, and
the stored strides and bounds decode them.

An identity survives a change that moves a row. It does not survive a change
to what a coordinate names, and no record can make it survive one.

A reviewer finds a violation when a reader matches a stored verb to a current
verb by index rather than by name.

### D3. The loader rebuilds the readout by identity, and refuses a verb the current table does not extend

The loader walks the rows of the current table. For each row it takes the
verb, matches it by name against the stored table, and reads the coordinates
of the row. It then names the stored row of that verb whose coordinates are
the leading coordinates of the current row, and copies the weight vector of
that stored row.

Four cases follow, and each is decided.

A verb whose stored coordinates are unreachable in the current table loses
those rows. The file is accepted and the weights of the lost rows are dropped.

A verb the stored table does not name gets no weight from the file.

A verb the current table does not name is dropped with its weights.

A verb whose argument positions the current table does not extend is a
refusal. The loader accepts a current position sequence that begins with the
stored one and adds positions after it. It refuses any other, including a
reordering, a renamed candidate, and a removed position. Such a verb no longer
means what the file meant by it, and no mapping of a coordinate recovers it.

**The refusal names the verb and both position sequences.** A message that
said only that a file did not fit would leave a reader with the failure this
record exists to end.

A reviewer finds a violation when a verb whose candidate kinds changed loads
without an error, or when the loader accepts a stored position sequence that
the current sequence does not begin with.

### D4. The fit of a stored policy is per verb, and it does not read the version integer of the action table

A reader that holds both tables compares them verb by verb under D3. It does
not compare the version integer the engine publishes for the action table, and
it does not compare the row count of the table.

The version integer stays in the file, because the engine owns it and a reader
that reports what a file says must report it. It decides nothing.

A reader that holds no stored table falls back to the version integer and the
row count, and refuses a mismatch. A file written before this record states no
table, and a reader with nothing to match by has only the integer.

The observation version and the observation length are unchanged by this
record. A reader still refuses a mismatch of either, and it still refuses a
world of another extent or another faction count.

A reviewer finds a violation when the action version integer decides a refusal
for a file that states a table.

### D5. A new row takes the weight of the stored row it specialises, and the mean of its verb when there is none

A row that an added argument position created takes the weight of the stored
row that the position narrows. That stored row is the one whose coordinates
are the leading coordinates of the new row, and it is the row the verb held
before the position existed.

**Index zero of a place enumeration names the whole frame**, and that is the
answer the verb gave before the position existed.[^8] The stored row and the
new rows are therefore the same decision at two resolutions, and the coarser
one is the honest donor for the finer ones.

A row that a grown bound created has no such donor, because no stored
coordinate reaches it. It takes the mean of the stored rows of its verb, which
is the closest statement the file holds of how much that verb appeals.

A row of a verb the file does not name takes zero. The file says nothing about
that verb, and there is nothing to donate.

**This is not lossless, and the cost is the opposite of the cost of zero.**
Every row an added position created scores identically when the file loads, so
the policy is indifferent among them. The choice then falls to the tie rule of
the reader, which takes the lowest legal row, so the policy names one
coordinate arbitrarily. It keeps choosing the verb at the rate it learned, and
it names the argument by nothing at all, until training separates the rows.

A run resumed this way can therefore score below the checkpoint it resumed
from, because an arbitrary argument may be worse than the coarse answer the
verb used to give. Zero has the opposite defect: the policy stops choosing the
verb, so it never reaches the rows that would teach it. This record prefers a
policy that acts and must learn to aim over a policy that must first learn to
act again.

A reviewer finds a violation when a new row is filled with zero for a verb the
file names, or when a donor is chosen from another verb.

## Consequences

**A change to one verb no longer retires a policy.** A verb may gain rows, a
bound may grow, a verb may be added, and every other verb keeps its weights.
The change that added a place argument would have kept every stored policy
under this record.

**A resumed run reports that it rebuilt rather than loaded.** The two are
different states and an operator who cannot tell them apart cannot read a
score. The loader states what it moved, and the run prints it.

**A stored policy is no longer one matrix and a small header.** It carries a
copy of one publication of the action table, and a reader of a weight file
must decode that copy before it can place a row. The file is larger and the
loader is longer.

**The stored table can disagree with the row count the file states.** Both are
written from one schema at one moment, so a disagreement is a defect in the
writer. A check compares them and fails, in the way the existing check
compares the stored row count against the architecture the file holds.[^7]

**A file written before this record still loads, and it loads by the old
rule.** It states no table, so the reader falls back to the version integer.
Such a file is retired by the next change to any verb, as it always was.

**The reader now holds a rule the engine does not.** The engine states the
layout and the loader states how one layout maps onto another. Nothing in the
engine enforces that the loader read the schema correctly, so a test must, and
the test must construct a table the engine no longer publishes.

**A removed argument position still retires every policy.** D3 refuses it. A
contributor who removes a position pays the cost this record removed for the
other cases, and that is deliberate: a verb that stops naming a thing means
something else by every row that named it.

**The observation keeps the defect this record removes.** A field added to the
observation still retires every stored policy, and the record that owns the
observation layout is where that question belongs.[^6]

## References

[^1]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decisions D1 and D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^2]: ADR-0176, an action integer is a mixed radix over the argument positions each verb declares, decision D1. `docs/adrs/accepted/adr-0176-an-action-integer-is-a-mixed-radix-over-the-positions-a-verb-declares.md`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, the consequence that the layout binds a trained policy. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D1. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
[^5]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D5. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
[^6]: ADR-0195, the observation of a faction is a fixed-width scale-free table in an egocentric frame, decision D9. `docs/adrs/draft/adr-0195-the-observation-of-a-faction-is-a-fixed-width-scale-free-table.md`
[^7]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^8]: ADR-0199, a verb names a place by a cell of the egocentric frame the observation publishes, decision D2. `docs/adrs/draft/adr-0199-a-verb-names-a-place-by-a-cell-of-the-egocentric-frame.md`
