---
id: 0332
title: Give a reader an install command that works
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: [BLK-040]
---

## Why

**A reader of the published reference cannot get the software.** The page named
no install command, and a graded review scored that gap as the second worst
failure on the page.

**The obvious command installs a different project.** The public Python package
index answers on the name `cachette` with an unrelated package by another
author, at a version this project never published. A reader who runs the obvious
command gets that package, and the import then fails with a message that names
no cause.[^1]

The reference now says that no public index carries this engine and gives the
commands that build it from a checkout. That is prose about a service outside
this repository, and no test can hold it. It decays the moment the project
publishes.

A blocker holds the two missing facts: the distribution name, and whether the
project publishes at all.[^2] The project owner owns both.

## References

[^1]: Findings register, FND-341. `docs/FINDINGS.md`
[^2]: Blockers register, BLK-040. `docs/BLOCKERS.md`
