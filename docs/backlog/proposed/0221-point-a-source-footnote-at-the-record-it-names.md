---
id: 0221
title: Point a source footnote at the record it names
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**Source files cite records that had no file, so their footnotes name the
registry instead.** A crate root, both build manifests, a gate script, the
world module, three shape modules, the bindings and a test each cite a record
number of the log or the Python boundary. Each footnote gives the registry as
the location, because that was the only true location when the line was
written.

Six of those numbers now have files.[^1] A reader who follows one of those
footnotes arrives at an index and has to search it, rather than arriving at the
claim. The footnote is not wrong, so nothing fails and nothing will.

This is the shape where one fact sits in two places and nothing compares
them.[^2] The record path is the fact, the footnote is the copy, and the copy
went stale the moment the file appeared.

The same thing will happen again for every reserved number a source file cites
before its record exists, so a check is worth more than a sweep.

## What is missing before this is refined

- The impact review.
- Whether a check can derive the correct path from the registry and compare it
  against every footnote in the tree, and whether the citation check is the
  right place for it.
- What the rule should be for a number that has no file. Naming the registry is
  correct then, so the check must permit it and must stop permitting it when
  the file appears.
- Whether the same footnotes should carry a decision number. A footnote that
  names a record without a decision is harder to keep true, and easier.
- The whole-tree search that finds every one of them, for the commit body.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR Registry. `docs/adrs/REGISTRY.md`
[^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
