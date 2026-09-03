---
id: 0223
title: Review seven drafts for acceptance
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**Seven records are written and none binds anything.** A draft is not binding,
so work built on one is built on sand, and review is the bottleneck the record
priority index names first.[^1]

The seven are the control plane rule, the tier rule, the selector tree, the
selector range, the influence basis, the group membership rule, and the descent
walk. Each waits for a reader who did not write it.[^2]

Two of the seven govern the work that other items are waiting on. The selector
tree is the destination the control plane verbs are written toward, and the
tier rule is the enforcement the control plane rule has none of. Neither can
govern anything while it is a draft.

## What this item is

One review file for each record, under `docs/reviews/`, named for this item.
Each states what was read, every objection the reviewer attempted, and a
verdict.

A verdict is not a status change. Accepting a record moves its file, and a move
breaks every citation of the old path, which is a separate cost that a separate
item carries.[^3] A review of a well-cited record records the verdict and says
what the move would cost.

## Done when

- Each of the seven has a review file with a verdict and at least one attempted
  objection.
- The registry status of each record is set, or the review says why it was not.
- The record priority index reflects what moved.

## Outcome

**Seven reviewed. Two accepted, one of those moved, four returned.**

One review file for each record sits under `docs/reviews/`, named for this item.
Every one states what was read, what the reviewer tried to reject, and why each
attempt failed or held.

**ADR-0051, the selector tree. Accepted and moved.** Nothing implements it, and
both the record and the review say so, which is the condition the registry sets
for accepting ahead of the code. Five objections were attempted and four failed
outright. The move cost five path citations, all in documents, so the reviewer
performed it.

**ADR-0078, the descent record. Accepted, and the status left.** Its exactness
argument is held by a compile-time assertion rather than by a comment. Six path
citations sit in two source modules that document work may not touch.

**ADR-0060, the influence storage record. Accepted, and the status left.** Its
read-only clause is held by the compiler rather than by discipline. Eleven path
citations, five in the influence module.

**ADR-0040 and ADR-0043 were returned together.** Both describe a boundary rule
that the interface already breaks in four places, and no refusal would have
caught three of them because the engine documents and answers them. ADR-0043
also says the core crate checks the tier when it builds a shape's storage, which
is true of one shape and false of both shapes the record is about.

**ADR-0052 was returned** because its cost case rests on the tiles being stored
in blocks, and they are stored row by row.

**ADR-0065 was returned** because two of its statements were already refuted in
the registers before it was written.

**What the reviews produced besides verdicts.** Three findings, two decision
rows, one backlog item, and one registry row that now says what a layout is
today rather than only what a record would claim.[^4] [^5] [^6] [^7]

**What this did not do.** It did not move three accepted or accept-verdict
records whose citations reach source files. That work is one commit for whoever
may edit `crates/`, and an item already carries the cost.[^3]

## References

[^1]: Decision record priority index. `docs/adrs/PRIORITY.md`
[^2]: Reviews index, who may review. `docs/reviews/README.md`
[^3]: Backlog item 0205. `docs/backlog/proposed/0205-stop-a-record-path-from-changing-when-it-is-accepted.md`
[^4]: Findings register, FND-215, FND-216 and FND-217. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-091 and DEC-092. `docs/DECISIONS.md`
[^6]: Backlog item 0224. `docs/backlog/proposed/0224-answer-and-command-a-set-of-mass-tier-entities.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
