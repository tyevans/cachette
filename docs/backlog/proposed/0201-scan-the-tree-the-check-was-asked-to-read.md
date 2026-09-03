---
id: 0201
title: Scan the tree the check was asked to read
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## The skip is repaired. The dead corpus is not.

**The first half of this item landed and this item was not narrowed.** The
record check now names three paths rather than three path components, so a run
inside a worktree reads that worktree. Item 0225 states the same repair from
the other side, and the two items were one job stated twice.

**What is left is the second defect alone: the corpus the check builds and
never reads.** The note still re-reads every source file, once for each record
it tests. Take this item for that.

**Read this item against item 0225 before planning.** That item holds the two
guards that would stop the skip being undone in silence, and neither exists.

## Why

**The record check reported more notes in a worktree than on the trunk, and the
extra notes were false.** One note asks whether any other record or any source
file cites a record. The scan that gathers the source files drops every path
that holds a directory part named `worktrees`. A worktree of this project sits
under that directory, so its own root carries the part, and the filter drops
every file of the tree it was asked to read. No record can then be cited by a
source file. The finding holds the evidence.[^1]

Every worker on this project runs in a worktree, so every worker reads the
inflated count. The check reports the note rather than failing on it, because
the rule treats low citation as a question and not a verdict.[^2] A question
that many records raise in one run, most of them wrongly, teaches a reader to
skip the whole block.

**The filter states a real need and states it in the wrong terms.** A run must
not read a checkout that another run is changing, because a file deleted
mid-scan raises rather than reporting. The filter names a directory part, and
a part cannot tell the tree the check was asked to read from a tree it should
refuse. Both carry the same part.

**Three sibling checks solve the same problem correctly.** The footnote
check, the citation check and the conflict marker check each skip a full path
under the root of the run, rather than a directory part. In a worktree that
path names nothing, so each check reads the tree it was asked to read. The
priority check walks no tree at all and reads named directories. The repair
has a shape the project already holds in three places.

**The same function holds a second defect, and it is the one that is left.**
The check joins the records, the registry and every source file into one
corpus, and nothing reads it. The note re-reads each source file instead, once
for each record it tests. A second finding holds the evidence.[^3] The two
defects are independent, and the repair to the skip made the corpus full and
left it unread, exactly as this item predicted.

**The note is the whole cost.** The source corpus reaches one assertion of the
record check, and that assertion is the note. No failure depends on it, so no
gate was weakened and no pass was meaningless. The work repairs a report and
not a gate.

## What is missing before this is refined

- The impact review.
- **What the corpus was for.** It is dead in the commit that created the
  script, so no earlier reader states the intent. Decide whether the note
  reads it or whether it goes.
- Whether reading the corpus once is faster than re-reading each file for each
  record, and by how much. The answer decides whether the corpus is read or
  removed.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-194. `docs/FINDINGS.md`
[^2]: Decision Record Scope, section 6. `.claude/rules/adr-scope.md`
[^3]: Findings register, FND-195. `docs/FINDINGS.md`
