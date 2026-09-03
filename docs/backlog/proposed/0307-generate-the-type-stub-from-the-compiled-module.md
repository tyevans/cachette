---
id: 0307
title: Generate the type stub from the compiled module
status: proposed
created: 2026-09-03
implements: [ADR-0107]
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**The Python package declares its public interface in two places, and nothing
fails when they disagree.** The compiled extension module is one. The
hand-written type stub beside it is the other. A record now states which of the
two owns the prose: a member the compiled module provides carries its prose in
the Rust doc comment and nowhere else.[^1] Nothing enforces that.

**The stub already holds copies.** Nine exception docstrings are the same words
as the Rust strings, character for character. One class docstring is an
abridged copy that dropped two paragraphs, one of which is the paragraph that
warns against one value in two places. A finding holds the evidence.[^2]

**An agreeing copy is the worse case.** It reads as a maintained file, so a
contributor who changes the Rust doc comment has no reason to look for the
second site. The project rule on recurring defects asks for a check that fails
when two sites disagree, and it says plainly that a comment naming the winner is
evidence that the second copy should not exist.[^3]

**The stub also states a check that does not exist.** Its own docstring said
that the build regenerates it and that a job fails on a difference. That
sentence is repaired, and the repair replaced a false claim with a true
one.[^4] The claim was worth making true, and this item is what makes it true.

**A reader of the published reference pays for this.** The product record
requires that something derives the list of public names from the package
itself and fails when the two disagree.[^5] A hand-written stub is the opposite
of that.

## What the work does

Generate the type stub from the compiled module, so that the signatures have one
declaration site rather than two.

Add a check that fails when the generated stub differs from the file in the
tree, and run it in continuous integration. The job that runs it builds the
extension first, in the way the documentation job does.[^1]

Add a check that fails when a stub member the compiled module provides carries a
docstring. A typed dictionary or any other declaration that the module does not
provide is exempt, because it has no other home for its prose.

Remove the docstrings that copy the Rust source: the nine exception classes and
the two class docstrings the finding names.[^2]

## What good looks like

A contributor changes a signature in the bindings crate, does not touch the
stub, and the check fails and names the member.

A contributor adds a docstring to a stub method, and the check fails and says
where the prose belongs.

**Put the defect back and watch the check stay green.** Restore one removed
exception docstring and confirm the second check fails. Change one Rust
signature and confirm the first check fails. A check that has never been shown
to fire has not been shown to exist.[^6]

## What it costs at the target scale

Nothing at run time. This is a build-time check, and no simulation code changes.

The check needs the built extension, so the job that runs it costs what the
wheel job costs. That cost is the same one the record already accepts for the
documentation job.[^1]

## What it does not do

It does not build the documentation site, choose a builder, or publish
anything. Those are separate items under the same record.

It does not move any prose into the stub. The record forbids that, and this item
enforces the record rather than reopens it.[^1]

It does not decide which surface of the package is public. That line is a
decision the product record asks for and no record draws.[^5]

## References

[^1]: ADR-0107, the Python reference is generated from the compiled module, decisions D2, D3 and D4. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^2]: Findings register, FND-321. `docs/FINDINGS.md`
[^3]: Recurring Defect Shapes, shape 1, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^4]: Findings register, FND-320. `docs/FINDINGS.md`
[^5]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^6]: Testing Rules, section 2a. `.claude/rules/testing.md`
