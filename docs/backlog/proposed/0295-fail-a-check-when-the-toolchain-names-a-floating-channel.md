---
id: 0295
title: Fail a check when the toolchain names a floating channel
status: proposed
created: 2026-09-02
implements: [ADR-0097]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The rule that the compiler is pinned to a date is prose, and nothing
enforces it.** The toolchain record states that the pin names a date and never
a channel, and it gives the reason: a bare channel makes the compiler an input
that changes without a commit, so two contributors on one commit build with two
compilers.[^1]

**One word is the whole distance between the rule and its violation.** A
contributor who wants a newer compiler edits the toolchain file, and the
shortest edit that works is to delete the date. Everything then builds, every
gate passes, and the project has silently lost the property the record
bought.[^2]

**The failure is invisible at the moment it is made and expensive later.** The
first sign would be a golden state hash that differs between two checkouts of
one commit, and the natural reading of that is a simulation defect. The
project's one unrecoverable property is that a binary gives one answer, and
this is the cheapest thing that keeps a channel out of the answer.[^3]

## What the work does

Add a check that reads the toolchain file and fails when the channel is not a
dated build. Put it with the other invariant checks, so that it runs in the
gate and in continuous integration.

The check is small: the channel must match a dated form, and a bare `stable`,
`beta`, `nightly` or a bare release name must fail. It is a check about one
file, so it needs no whole-tree search.

## What good looks like

The gate rejects a toolchain file whose channel carries no date, and the
message names the record and says why the date is there.

**Put the defect back and watch the check fail.** The project already runs its
record checks against broken fixtures for exactly this reason, and this check
gets a fixture in the same shape.[^4]

## What it costs at the target scale

Nothing. It reads one small file at gate time.

## What it does not do

It does not check that the date is recent, and it should not. How often the
date moves is an open decision with an owner, and a check that enforced a
freshness rule before that decision closed would enforce an interval nobody
chose.[^5]

It does not check that the installed compiler matches the file. The toolchain
manager already does that, and a second check of it would be a second
declaration site.

## References

[^1]: ADR-0097, the toolchain is a dated nightly, decision D2. `docs/adrs/draft/adr-0097-the-toolchain-is-a-dated-nightly.md`
[^2]: Recurring defect shapes, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Testing rules, section 1. `.claude/rules/testing.md`
[^5]: Decisions register, DEC-106. `docs/DECISIONS.md`
