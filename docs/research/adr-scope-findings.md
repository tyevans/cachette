# Findings: the scope of a decision record

This note records a measurement of two decision record corpora. The purpose was
to decide what belongs in a Cachette decision record, and how to tell whether a
decision needs a record at all.

The two corpora belong to sibling projects by the same author.[^1] [^2] One
holds 61 numbered records. The other holds 45. Both have full commit history.
Both are Python. Cachette is Rust with a Python control plane, so the subject
matter does not transfer. The failure modes do.

The note states the numbers so that a later reader can test the rule against
evidence instead of arguing it again.

## Method

Each measurement ran over the record files and the commit history. Churn means
the number of commits that touch a file, counted with rename following. Length
means the word count of the file at the time of measurement. A citation means
one record naming another, or a source file naming a record number.

The counts are of the corpora as they stood on 30 August 2026.

## Measurement 1: length predicts churn

Length and churn correlate in both corpora, and the two correlations agree.

| Corpus | Records | Pearson r, words against commits | Median words |
|---|---|---|---|
| Project A | 61 | 0.704 | 1247 |
| Project B | 45 | 0.715 | 1322 |

The effect is large at the extremes.

| Corpus | Band | Records | Mean commits |
|---|---|---|---|
| Project A | 2000 words or more | 17 | 3.76 |
| Project A | under 800 words | 12 | 1.67 |
| Project B | 4000 words or more | 10 | 6.10 |
| Project B | under 1200 words | 19 | 1.63 |

A long record is edited about 2.3 to 3.7 times as often as a short one. Two
independent corpora give the same coefficient. The most likely cause is that a
long record holds more than one decision, so any one of them can force an edit.

**This is a direct warning for Cachette.** The six draft records run from 1777
to 4570 words. The median is about 3400 words. Every one of them sits above
both reference medians. Four of the six sit in the band that churned most.

## Measurement 2: records that hold a value churn more

A record that pins a dependency version, or that quotes a measured figure,
churns more than the corpus mean.

| Corpus | Subset | Records | Mean commits | Corpus mean |
|---|---|---|---|---|
| Project A | Pins a version | 5 | 3.60 | 2.34 |
| Project B | Pins a version | 5 | 5.00 | 2.87 |
| Project B | Quotes a measured figure | 5 | 4.80 | 2.87 |

The version subset churns 1.5 to 1.7 times the mean. One record in project A
selects a testing tool by name and version. Its own text now says that the
selected release line is superseded. Nothing cites that record.

## Measurement 3: a record superseded within days held an unstable decision

Two supersessions in the corpora happened almost immediately.

| Corpus | Record | Created | Superseded by | Latency |
|---|---|---|---|---|
| Project B | 0018, a replay report carries its failures | 5 Aug 2026 | 0020 | 1 day |
| Project A | 0017, snapshot strategy pattern | 28 Jul 2026 | 0021 | 2 days |

Both recorded a component shape rather than a constraint. Project B record 0018
says in its own status block that every decision below it still holds, but that
none of them is held there any more. The decision survived. The record did not.
The record had bound the decision to a module that then moved.

Several amendments landed within three days of the record they amend. Project A
record 0019 was amended two days after creation. Records 0015 and 0023 were
amended three days after creation.

## Measurement 4: five records were deleted, not superseded

Project A skips numbers 0002 to 0006. The five files existed from 6 December
2025 to 14 February 2026. A single commit deleted all five. The commit message
calls them generated documentation that nobody maintains.

Their titles were: pydantic event models, optimistic locking, projection error
handling, API design patterns, and event registry serialization. Each carried
the status `Accepted`. Each ran between 1241 and 1846 words.

Every one of the five names a topic, not a choice. None of them states a
constraint that a reader could violate. They record what the project does. They
do not record what the project may not do. Nothing cited them, and the project
lost nothing when they went.

**This is the strongest evidence for the framing that a decision may be left
unstated.** Five records were written because a topic existed. All five were
deleted together. The project ran for 70 days with them and did not use them.

## Measurement 5: uncited records are short

A record that nothing cites averages about half the length of the corpus mean.

| Corpus | Group | Records | Mean words |
|---|---|---|---|
| Project A | Cited by no other record | 8 | 779 |
| Project A | All records | 61 | 1582 |
| Project B | Cited by neither a record nor code | 1 | 666 |

Five of the eight uncited records in project A predate the newest work by weeks,
so age does not explain them. Their mean length is 815 words.

Project B is much healthier. Only one of its 45 records is cited by nothing at
all, and it is the second shortest file in the corpus.

Project A also shows that a record can be cited by other records and never reach
code: 33 of its 61 records have no reference from any source file.

## Measurement 6: both projects moved from topic titles to claim titles

A title either names a subject or states a claim. "Snapshot strategy pattern"
names a subject. "An event cannot name another aggregate" states a claim.

The share of claim titles rises over time in both corpora, without the two
projects sharing a rule that says so.

| Corpus | First half | Second half |
|---|---|---|
| Project A | 1 of 33, 3 per cent | 6 of 28, 21 per cent |
| Project B | 13 of 23, 57 per cent | 18 of 22, 82 per cent |

In project B the form predicts churn.

| Corpus | Title form | Records | Mean commits | Mean words |
|---|---|---|---|---|
| Project B | Claim | 31 | 2.13 | 1887 |
| Project B | Topic | 14 | 4.50 | 4178 |

A topic title churns twice as much and runs more than twice as long. The
mechanism is plain. A topic title has no boundary, so any material about the
topic can be added to the file. A claim title has a boundary. Material that does
not support the claim has nowhere to go.

All six Cachette drafts except record 0001 carry a topic title.

## Measurement 7: the corpora record their own decay

Project A holds a rule about defect shapes. Its fifth shape is documents and
records that rot as soon as a sweep names specifics. It lists eleven correction
commits that only repair a previous correction. Its rule is that a record must
not contain counts of things or tables of files, because those belong in a
commit message, which is immutable.

Project A also holds a record whose subject is that a declared catalogue of
telemetry keys listed keys that nothing ever emitted. An earlier record had
claimed that the catalogue described what the library emits. That claim was a
wish, not a description. The later record repairs it.

Both projects record three record-number collisions caused by parallel branches
choosing their own numbers. Cachette already solves this. Its registry allocates
the number before the file exists.

## What the evidence supports

1. Length is the best single predictor of instability. One record, one claim.
2. A record that holds a value that a measurement or a release can change will
   be edited. Cite the value; do not hold it.
3. A record that binds a decision to a module shape dies when the module moves.
   Record the constraint, not the arrangement.
4. A record written because a topic exists gets deleted unread.
5. A claim title bounds a record. A topic title does not.

## What the evidence does not support

The evidence does not support deleting a record because nothing cites it. In
project B the uncited record is correct and small. Low citation is a signal to
check, not a verdict.

The evidence does not support a word limit. Project B record 0013 runs 10810
words, churned six times, and is cited by two records. It is long because its
subject is genuinely large. A limit would have split it arbitrarily.

The evidence does not support treating amendment as failure. Project A amends
often and its records stay usable. An amendment that adds a consequence is
healthy. An amendment that corrects a stale figure is the defect.

## References

[^1]: Project A, the `eventsource-py` repository. 61 records under `docs/adrs/`. Measured at commit history to 12 August 2026.
[^2]: Project B, the `redstring` repository. 45 records under `docs/adrs/`. Measured at commit history to 21 August 2026.
