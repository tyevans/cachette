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

## Why

**The record check reports more notes in a worktree than on the trunk, and the
extra notes are false.** One note asks whether any other record or any source
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

**The note is the whole cost.** The source corpus reaches one assertion of the
record check, and that assertion is the note. No failure depends on it, so no
gate was weakened and no pass was meaningless. The work repairs a report and
not a gate.

## What is missing before this is refined

- The impact review.
- **Whether the repair adopts the sibling shape or a better one.** A full
  path under the root is what the other three checks use, and it works. It
  still states the boundary as a location. Ask whether the run should name the
  tree it owns instead.
- What the boundary is, if it is not a directory part. The scan must exclude a
  tree that the run does not own, and it must include the tree that the run
  does own, even when that tree sits inside another one.
- What a test asserts. A test that runs the check from inside a worktree and
  requires the same note count as a run on the trunk would show the two
  answers in one run.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-194. `docs/FINDINGS.md`
[^2]: Decision Record Scope, section 6. `.claude/rules/adr-scope.md`
