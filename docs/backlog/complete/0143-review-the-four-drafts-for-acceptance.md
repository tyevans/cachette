---
id: 0143
title: Review the four drafts for acceptance
status: complete
created: 2026-09-01
implements: []
changes: [ADR-0076, ADR-0080]
creates: []
serves: []
blocked-by: []
---

## Why

Four records are drafts. A draft binds nothing, so work built on one is built
on sand.[^1]

Three of the four hold work that waits. Backlog items 0059, 0060 and 0103 all
cite the housing draft. The growth draft depends on the housing draft. The
founding draft and the recovery draft both describe code that exists, so both
can be read against the code rather than against the intent.

## Impact review

**Governed by.** The registry states who reviews and what a delegated review
must do that a second reader would do for free: read the record against the
code, be an agent that did not write it, and state what it tried to
reject.[^1]

**Changes.** ADR-0076 and ADR-0080 move from `Draft` to `Accepted`, or the
review returns them with objections. The registry row and the file location
change with each one, and every citation of a draft path moves with it.

**Creates.** No record.

**Blockers.** None. BLK-007 governs every cost figure, and the review claims
none.

**Precedent.** Review 0047 holds the shape of a record review.[^2] FND-116
records that the housing work already exists in part, which the housing draft
must be read against.[^3]

## Done when

- An agent that did not write the four records has read every decision of each
  one against the code, or has stated that nothing implements it.
- The review states what it tried to reject, and what happened to each
  objection.
- The cost claim of the recovery draft is checked against the signature of the
  pass, and no figure is claimed.
- The housing draft is read against ADR-0074 D3, which answers the tile case
  the other way.
- The registry holds the outcome of each record.
- The review is a file under `docs/reviews/`.

## Outcome

Two records are accepted and two are rejected.[^4]

**ADR-0076 and ADR-0080 hold against the code.** Every decision of each was
found in the source and in a test. The founding keeps its distance, founds in
ascending faction index, and puts the faction in the frame slot of the draw
key. Recovery ages the stored take, takes no tile count, uses whole numbers in
key order, runs before the gather resolve, and holds one period for each kind.
Each takes a small amendment that changes no claim.

**ADR-0081 is rejected because the engine already answers its question.** The
record's context says that nothing states how many units a site holds. The
cohort table holds a per-site headcount, derived from the home column the
record itself calls the residence, and the consumption pass rebuilds it twice
in one frame. The prohibition of decision D3 therefore either forbids code
that exists, or buys nothing. FND-128 holds the evidence and DEC-057 holds the
choice that follows.[^5] [^6]

**ADR-0082 is rejected because it rests on ADR-0081.** Its own decisions are
sound and four objections against them failed.

**Two rules were found that nothing checks.** A decision record must cite no
product record, and four accepted records cite one.[^7] The documentation rule
orders footnotes and forbids repeating one, and three of the four drafts break
it.[^8] Items 0144 and 0145 carry the repairs that follow.

**No measurement was taken.** Every cost figure in this project is derived.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: Review 0047, the viewer boundary record. `docs/reviews/0047-the-viewer-boundary-record.md`
[^3]: Findings register, FND-116. `docs/FINDINGS.md`
[^4]: Review 0143, the housing, growth, founding and recovery records. `docs/reviews/0143-the-housing-growth-founding-and-recovery-records.md`
[^5]: Findings register, FND-128. `docs/FINDINGS.md`
[^6]: Decisions register, DEC-057. `docs/DECISIONS.md`
[^7]: Findings register, FND-129. `docs/FINDINGS.md`
[^8]: Findings register, FND-130. `docs/FINDINGS.md`
