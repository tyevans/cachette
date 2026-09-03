---
id: 0294
title: Reconcile the three declarations of the toolchain
status: proposed
created: 2026-09-02
implements: [ADR-0097]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**Three files state which compiler this project needs, and they now disagree
in kind.** The toolchain file names a dated nightly build.[^1] The workspace
manifest names a minimum stable release. The lint configuration names a
minimum stable release of its own, for the lints.

**Only one of the three decides anything.** The toolchain file is what the
toolchain manager reads, and it is what every contributor and every continuous
integration job installs from. The other two are read, they are believed, and
they change nothing about which compiler runs.

**Nothing fails when they disagree.** This is the defect shape the project
records most often: one value declared in more than one place, with no check
that fails when the copies part company.[^2] The record that pinned the
nightly names this consequence and leaves it open.[^3]

**The disagreement is not yet a lie, which is why this is worth doing now.**
The manifest's claim is still true today, because nothing in the tree uses an
unstable feature, so the crates would compile on the stable release it names.
The first pass that uses the portable vector library makes it false, and
nothing will announce that. The cheap moment to settle this is before the
claim breaks, not after.

## What the work does

Decide what each of the three sites means, and make the tree say it.

The question is whether the minimum-release claim is still a claim this
project wants to make. A project pinned to one dated compiler has one supported
toolchain, not a floor and a ceiling. If the claim stays, something must check
it, and checking it means building on the named release, which is a second
compiler in the gate. If it goes, say what the lint configuration should hold
instead, because that field changes which lints fire.

Then add a check that fails when the sites disagree, whatever the answer is.
The rule is explicit that a comment naming the winner is not the remedy, and
that a comment explaining which copy loses is evidence the copy should not
exist.[^2]

## What good looks like

One place states which compiler this project needs. Any other place that must
repeat it is derived from that place, or is compared against it by a check that
fails.

**Put the defect back and watch the check fail.** Change one site to disagree
with the others and confirm that the gate goes red.

## What it costs at the target scale

Nothing. No simulation code changes and no data structure changes.

## What it does not do

It does not decide how often the pinned date moves. A decision row holds that,
and engineering owns it.[^4]

It does not add a second compiler to the gate. If the answer is that the
minimum-release claim stays, this item states what checking it would cost and
stops there, so that the cost is decided rather than absorbed.

## References

[^1]: The pinned toolchain. `rust-toolchain.toml`
[^2]: Recurring defect shapes, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^3]: ADR-0097, the toolchain is a dated nightly. `docs/adrs/draft/adr-0097-the-toolchain-is-a-dated-nightly.md`
[^4]: Decisions register, DEC-106. `docs/DECISIONS.md`
