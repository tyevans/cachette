# Decision Record Scope

This rule says what belongs in an architecture decision record, what does not,
and how to decide whether a decision needs a record at all.

An architecture decision record is a numbered document that states one binding
constraint on this project, and gives the reasoning that makes it binding. The
registry allocates the number and holds the status.[^1]

The rule comes from a measurement of two mature record corpora, 106 records with
full commit history.[^2] The measurements are in the findings note.[^3] Change
this rule against new measurements, not against argument.

## 1. The test for whether a decision needs a record

**Do not write a record because a topic exists. Write a record because a
constraint exists.**

A decision needs a record when all three statements below are true.

1. **A future contributor could reasonably choose otherwise.** If there is only
   one workable option, there is no decision. Describe the mechanism in the code
   or in a design document instead.
2. **Choosing otherwise costs more than changing it later.** A record buys the
   right to refuse a change. If the change is cheap, the record only adds
   friction.
3. **The reasoning is not visible in the artefact.** If a reader of the code can
   see why, the code is the record. A record exists to carry reasoning that the
   artefact cannot hold.

If any statement is false, leave the decision unstated. An unnecessary record is
not free. It binds work that should be free, someone must maintain it, and when
it goes stale it states something false with the authority of a record.

The evidence is direct. One reference project wrote five records that each named
a topic and stated no constraint. All five carried the status `Accepted`. All
five were deleted in one commit 70 days later, without supersession, because
nobody maintained them and nothing cited them.[^3]

### The counter-test

Do not use this rule to avoid work. A decision that governs determinism always
needs a record, even when it looks obvious now. Determinism is the one property
this project cannot recover after it is lost.[^4] An undocumented determinism
decision becomes an assumption, and a later contributor will trade it away for
speed.

When you are unsure, ask what a reviewer would need in order to reject a change.
If the answer is a written constraint, write the record.

## 2. One record holds one claim

Length predicts instability. In both reference corpora the correlation between
word count and the number of commits to a file is about 0.71. A record of 4000
words or more was edited about 3.7 times as often as a record under 1200
words.[^3]

The cause is not length itself. A long record usually holds several decisions,
so any one of them can force an edit to the whole file.

Split a record when it states two claims that could be accepted separately.

**The Cachette drafts are all above both reference medians.** The six drafts run
from 1777 to 4570 words against reference medians near 1300. Treat this as a
warning, not as a defect. A foundational record earns extra length. A subsystem
record does not.

There is no word limit. One reference record runs 10810 words and is healthy,
because its subject is genuinely one large thing.

## 3. Title the record with the claim, not the subject

A title states what the record decides. Write "Aggregates use integer
accumulators", not "Aggregation". Write "A verb never calls Python", not "The
Python boundary".

A topic title has no boundary, so any material about the topic can be added to
the file. A claim title has a boundary. Material that does not support the claim
has nowhere to go.

Both reference projects moved toward claim titles as they matured, without a
shared rule. In one corpus a topic title churned 4.50 times on average against
2.13 for a claim title, and ran 4178 words against 1887.[^3]

The number in the file name never changes when a title changes.[^1]

## 4. What must not go in a record

Each category below comes from an observed failure in the reference corpora.

### 4.1 A value that a measurement can change

Do not put a cost budget, a byte budget, a throughput figure, a latency figure,
or a percentage in a record. Put it in the reference tables and cite it.

Records that quote a measured figure churned 4.80 times on average against a
corpus mean of 2.87.[^3]

Every cost figure in this project is derived, not measured. A derived figure
changes when a better derivation arrives, which is more often than a decision
changes.

A structural constant is not a budget. The cache line size of the target is 64
bytes, and that is a property of the platform the project chose. State it.

### 4.2 A version number or a named release

Do not pin a dependency version, a crate release, or a tool version in the body
of a record. Name the dependency and state the property you need from it. Put
the version you read in the footnote, where it records what you consulted.

Records that pin a version churned 1.5 to 1.7 times the corpus mean.[^3] One
reference record selects a testing tool by name and version. That record now
says in its own text that the chosen release line is superseded. Nothing cites
it.

### 4.3 A count, a file table, or a survivor list

Do not write "thirteen systems moved" or "83 of 86 call sites". Do not put a
table of source files in a record.

The commit message holds this material. A commit message is immutable and is
correctly scoped to one moment.[^5] A count in a record decays the next time
anyone touches the tree.

One reference project traced eleven correction commits whose only purpose was to
repair the specifics named by a previous correction.[^3]

### 4.4 A module arrangement

Do not record where code lives, unless the location is itself the constraint.
Record the constraint that the arrangement serves.

Two reference records were superseded within one and two days of creation. Both
recorded a component shape. One says in its own status block that every decision
in it still holds, but that none of them is held there any more. The decisions
survived. The records died because the modules moved.[^3]

The crate boundary is a legitimate exception. A record may state that the core
crate holds no Python binding, because that boundary is the constraint and a
compiler enforces it.[^4]

### 4.5 A value that an unanswered question governs

If a blocker governs a value, do not invent the value. Express the decision with
the value as a parameter and cite the blocker.[^6]

### 4.6 A capability nobody invokes

Do not record an intent as if it were a fact. One reference record claimed that
a declared list of telemetry keys described what the library emits. Several keys
had no write site anywhere. A later record had to repair the claim.[^3]

Write what the code does, or write the constraint the code must satisfy. Do not
write what you hope the code will do.

## 5. What must go in a record

- **The constraint.** One claim, stated so that a reviewer can find a violation.
- **The forces.** What made the choice hard. This is the part that stays true.
- **The alternatives rejected, and why.** A rejected option that returns is the
  most common reason someone reads an old record.
- **The consequences.** What the project now cannot do.
- **The evidence, in footnotes.** Cite the research report or the source. Do not
  repeat the research.[^7]

Number the decisions inside a record and cite them with the record number, for
example `ADR-0004 D3`. Flat decision numbers collide.[^1]

## 6. Low citation is a signal, not a verdict

A record that no other record and no source file cites may not have been worth
writing. In one corpus, the eight records that nothing cites average 779 words
against a corpus mean of 1582.[^3]

Do not delete a record because it has no citations. In the other corpus the one
uncited record is correct and small. Treat a zero citation count as a question
for review: is this a constraint, or is it a description?

A record that reaches no source file is weaker evidence. In one corpus 33 of 61
records have no reference from any source file, and many of those are sound.

## 7. Amendment is not failure

A record that gains a consequence as the project learns is working. A record
that must be edited to correct a stale figure has the defect this rule prevents.

An accepted record does not change, except in status. To change a decision,
write a new record that supersedes it.[^1]

## 8. The check

One script checks the mechanical part of this rule.[^8] It fails when a record
misses a required section, when a record holds a version pin or a volatile
figure in its body, when a record cites a number that no record has, or when the
registry and the files disagree. It reports records that nothing cites, without
failing.

The check cannot tell whether a decision needed recording. Section 1 is a human
judgement. The check only stops the failures that a regular expression can see.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: The `eventsource-py` and `redstring` repositories, measured 30 August 2026.
[^3]: Findings, the scope of a decision record. `docs/research/adr-scope-findings.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Commit Message Rules. `.claude/rules/commits.md`
[^6]: Blockers register. `docs/BLOCKERS.md`
[^7]: Documentation Rules. `.claude/rules/documentation.md`
[^8]: The record check script. `scripts/check_adrs.py`
