---
id: 0205
title: Stop a record path from changing when it is accepted
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: [DEC-083]
---

## Why

**A citation of a record names its path, and the path holds the directory.** A
record moves from one directory to another when a reviewer accepts it. Every
citation written while the record was a draft then names a path that does not
resolve, and the citation check fails on each one. That check reads source
comments as well as documents.[^1]

So accepting a record is a whole-tree sweep. The sweep is invisible until the
check runs, and it lands wherever the record is cited.

**The cost scales with how well the record is cited, which inverts the
incentive the project wants.** A record that nothing cites moves for free. A
record that reaches many call sites pays for every one of them. The project
already treats a citation from a source file as the strongest evidence that a
record is a constraint rather than a description.[^2] This makes that evidence
expensive to hold.

**Two records are sitting at verdict Accept right now, and a file move is the
only thing between them and binding.** A review read both against the code,
attempted its objections, and gave each an accept. It could not set either
status, because it was a documents-only worker and most of the citations sit in
source comments.[^3] A record that a reviewer has passed and that binds nothing
is the failure the priority index exists to prevent.[^4]

**The same defect makes a supersession expensive.** A superseded record keeps
its file and changes its status, so it moves too, and every citation of it
moves with it. A supersession is the moment a reader is most likely to follow
an old citation, and it is the moment the citation is most likely to be broken.

**Item 0198 is this defect seen from the other side.** There, the record check
reads any mention of a record number as a citation, so a retired number cannot
be named at all.[^5] A record that moves and a record that cannot be mentioned
are one problem: the project has no stable way to refer to a record whose
status is not the status it had when somebody wrote the reference.

## What is missing before this is refined

- The impact review.
- **The register row must close first.** It holds whether a citation should
  name the directory at all, and what a stable reference looks like. It
  recommends holding every record in one directory and letting the registry
  hold the status.[^6] The work follows the answer and must not invent it.
- What a directory listing loses. A reader who browses the tree can see today
  which records bind. If the directories merge, the registry and the priority
  index are the only places that answer, and both must be easy to reach.
- Which checks read a path, and what each of them must do instead. Two of them
  resolve a path today, and one of them also reads the status of the directory
  a footnote names.
- Whether the answer covers item 0198 as well. A stable reference to a retired
  number is the same question, and two repairs to one problem is the shape this
  project records as a recurring defect.[^7]
- What the sweep costs once, if the answer requires one. The move is a
  documents change and a source-comment change, and a source-comment change
  needs a worker who may edit that tree.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The citation check script. `scripts/check_citations.py`
[^2]: Decision Record Scope, section 6. `.claude/rules/adr-scope.md`
[^3]: Review 0204, the two corrected records. `docs/reviews/0204-the-two-corrected-records.md`
[^4]: Decision record priority index. `docs/adrs/PRIORITY.md`
[^5]: Backlog item 0198, tell a mention of a record number from a citation of it. `docs/backlog/proposed/0198-tell-a-mention-of-a-record-number-from-a-citation.md`
[^6]: Decisions register, DEC-083. `docs/DECISIONS.md`
[^7]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
