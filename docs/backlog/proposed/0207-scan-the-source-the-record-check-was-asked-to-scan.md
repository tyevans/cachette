---
id: 0207
title: Scan the source the record check was asked to scan
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The record check reports a record that no other record and no source file
cites. The rule asks a reviewer to weigh that signal: a record nothing cites
may be a description rather than a constraint.[^1]

**The source half of that check reports nothing, for every record, in every
worker's run.** The scan skips a path whose parts name a worktree, because a
worktree holds another checkout and its files change under the run that reads
them. The skip matches on the parts of an absolute path, so it also matches the
root of the scan when the caller is itself a worktree. Every worker on this
project runs in a worktree. The scan then collects no file at all.

Nothing fails, because the result is a note. The check keeps printing a
plausible list, and most records stay off it because the record half of the
check still works. The finding holds the evidence.[^2]

## What the work might do

The shape is a skip that is relative to the root rather than absolute. The
questions this item must answer before it is refined:

- Whether the skip should compare against the path relative to the root, or
  whether the check should refuse to run from a location it would skip.
- What the check should do when it collects no source file at all. A scan that
  finds nothing is a result no caller wants, and reporting it would have caught
  this on the first run.
- Whether any other check carries the same skip. The skip set appears where a
  script walks the tree, and a defect in one is a candidate in all of them.
- Whether a test can hold it. A check that reads the tree is hard to test
  against a fixture, and this one has no test today.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decision Record Scope, section 6. `.claude/rules/adr-scope.md`
[^2]: Findings register, FND-201. `docs/FINDINGS.md`
