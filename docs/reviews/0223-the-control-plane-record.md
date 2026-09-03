# Review 0223: The control plane record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md` | `Draft` at review, and `Draft` after it |
| Commit | `c043acc`, the head of the review branch |
| Code read | the bindings, the type stubs, the tier declarations of the shapes, the public interface tests, the site position tests, and the agent server |

The reviewer did not write this record. The reviewer wrote seven other records
in the same session, and none of them is this one. The reviewer did write
ADR-0042 and ADR-0044, which cite this record, so the reviewer had a reason to
want it accepted. Section 4 states what the reviewer did about that.

**The reviewer compiled nothing.** Other workers hold the machine. Every claim
below is read from the source.

## Verdict

**Accept with amendment.** The four decisions are right and the record is well
argued. One consequence enumerates the reads that exist today and calls them
compliant, and the current surface holds three reads and one verb form that
this record's own D2 test fails. A reader of that paragraph concludes the
boundary complies. It does not.

Section 5 gives the exact text. The status stays `Draft` until the author
amends it or rejects the amendment with a reason.

**Nothing else held.** Four other objections were attempted and all four
failed. They are in section 3, because an objection that fails is evidence that
the record is sound in that place.

**Which questions this review put explicitly, and which it answered by
inference.** Three questions decide whether a record should exist: could a
contributor reasonably choose otherwise, does choosing otherwise cost more than
changing it later, and is the reasoning invisible in the artefact. This review
put the first and the third to the record, and it put the separate question of
whether the record contradicts the code, which is what section 2 answers. **It
did not put the second explicitly.** The reviewer judged that a boundary shape
every verb is written against is expensive to reverse, and did not argue it on
the page. A reader should treat that condition as unexamined rather than
answered.

## 1. What the record claims, and what the code does

**D1, the boundary carries an instruction and an answer.** The engine returns
tile values, event fields and position columns. Each is one call that returns
one column for each field. No method returns a handle that Python walks to
reach a value. This holds.

**D3, a verb accepts what the verb before it returns.** This holds, and it is
tested. The spawn verb returns a column of identities, the stub declares that
the verbs taking units accept a column or a sequence, and the public interface
test passes the column from the spawn straight into the gather order with no
conversion. The second instance of FND-147 is closed.[^1]

**D4, a question the control plane cannot ask is a defect of the boundary.**
This binds the reviewer, and section 2 is the reviewer obeying it.

**D2, the number of crossings does not grow with the number of entities.** This
is where the record and the code part company.

## 2. The objection that held: three per-entity reads and one verb that cannot vary

D2 gives the test in the record's own words: a reviewer counts the crossings a
caller needs and asks whether that count is a function of the population. Its
last paragraph says a verb over a mass-tier shape has no per-entity form, and
the tier record states the same rule for reads.[^2]

Both the soldier shape and the settlement shape declare the mass tier. The
declaration on the soldier carries the reason in its own comment: a soldier is
one of a million, so no caller walks the population. The settlement declares the
mass tier deliberately, because a shape declares the stricter tier its
population admits.

Against those two declarations the boundary offers three reads that each name
one entity: the tile of one soldier, the positions of one site, and the
preference of one site. The agent server wraps the first as a tool that reads
one unit. A caller that wants the answer for a set calls once for each member,
so the crossing count is a function of the population, which is the case D2
names as wrong whatever the caller wrote.

**The type stub does not merely permit the pattern. It directs the reader into
it.** The documentation of the gather event columns tells a reader to take a
value from the unit column and hand it back to the per-unit read. That is the
boundary instructing a caller to loop, which is the shape FND-147 records.[^1]

**A test already loops, and it loops four times for each site.** The thread
count test for the positions sets a preference one site at a time, because the
set-valued verb takes one target for the whole set and the test wants a
different target for each site. It then reads the positions of each site twice,
once for each column it compares. Eight sites cost eight command crossings and
sixteen read crossings, and a hundred sites would cost a hundred and two
hundred.

That test is not badly written. It is the only way to express what it needs
through the surface that exists, which is exactly what D4 says a sweep is
evidence of. **The missing capabilities are a set-valued read and a set-valued
command that carries one value for each member.**

This is filed, with the evidence, as a finding and as a backlog item.[^3] [^4]

## 3. The objections that failed

**Does a Python loop over a returned column break the record?** Two tests build
a list from the identities a spawn returned. The objection fails. D1 and D2 bind
the traffic across the boundary, not what Python does with an answer it already
holds, and the tier record states plainly that a caller who loops over an array
of numbers has left the set and is permitted to.[^2] One crossing produced the
array, and the loop adds none.

**Does the spawn verb break D2 by taking a list of addresses that a caller
built?** The objection fails. Building the list is Python work on the Python
side. The crossing count is one whatever the length of the list, which is the
test D2 states.

**Is D4 unfalsifiable, since any missing feature can be called a missing read?**
The objection fails, and it fails on the record's own second paragraph. D4
bounds the repair: the read that gets added must answer for a set, and a
per-tile read is named as the worse repair because the tile population is
larger. A decision that names the wrong answer as well as the right one is
falsifiable.

**Does the record contradict ADR-0085, which lets one identity cross?** The
objection fails. ADR-0085 governs the representation of an identity that
crosses, and this record governs how many times anything crosses. An identity
crossing whole inside one column is consistent with both.

## 4. What the reviewer did about wanting this record accepted

The reviewer wrote two records in this session that cite this one, and a
reviewer that needs a record to be accepted is not a neutral reader.

The reviewer therefore read the record against the bindings and the tests first
and against its own two records not at all. The objection in section 2 was found
by listing every public method of the bindings and asking of each whether one
call answers for a set, before reading the record's consequences. The
consequences were read afterwards, to see whether the record had already named
what the list found. It had not.

## 5. The amendment

Add one consequence, after the paragraph beginning "A column read that exists
today is not a breach". The record must not be rewritten by the reviewer, so the
text below is a proposal for the author.

> **Three reads that exist today are a breach, and the record names them rather
> than leaving them to be found.** The engine answers the tile of one soldier,
> the positions of one site and the preference of one site, and both shapes
> declare the mass tier. A caller that wants any of those answers for a set
> calls once for each member, which is the case D2 refuses. The set-valued
> command also carries one value for the whole set, so a caller that wants a
> different value for each member loops instead. Both gaps are missing
> capabilities of the kind D4 describes, and an item holds them.

The reference for the item goes in the reference list as a footnote.

If the author judges that a per-entity read over a mass-tier shape is
acceptable, then D2's last paragraph and the tier record's D1 are the things
that are wrong, and the correction belongs there instead. Either way the two
must agree before this record binds anything.

## 6. What this record does well, and why the amendment is small

The record already refuses to overstate itself in two places. It says plainly
that nothing enforces it and names the two records that would. It says plainly
that it states no cost, because no measurement exists on the target platform.
A record that already declines to claim an enforcement it does not have is a
record that will accept a correction naming a compliance it does not have.

The amendment adds a paragraph. It changes no decision.

## References

[^1]: Findings register, FND-147. `docs/FINDINGS.md`
[^2]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop, decisions D1 and D4. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^3]: Findings register, FND-215. `docs/FINDINGS.md`
[^4]: Backlog item 0224. `docs/backlog/proposed/0224-answer-and-command-a-set-of-mass-tier-entities.md`
