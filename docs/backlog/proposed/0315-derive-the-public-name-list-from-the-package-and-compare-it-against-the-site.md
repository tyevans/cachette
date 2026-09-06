---
id: 0315
title: Derive the public name list from the package and compare it against the site
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**The product record asks for this in one sentence.** Every public name of the
package appears in the documentation, and something derives the list of public
names from the package itself and fails when the two disagree.[^1]

**A name that is added and never documented costs the reader the whole
document.** The reader cannot tell a current sentence from a stale one, because
both are prose. The product record states the cost, and it names the two times
this project paid it.[^1] [^2] [^3]

**The generated reference does not close this by itself.** It holds what the
directive names. A name added to the package and not to a directive publishes
nowhere, and the build reports nothing. The record that binds the reference says
the same shape about prose: a member with no doc comment publishes with no
prose, and nothing fails.[^4]

## What the tree already holds

**A derived check exists, and it compares prose rather than the name list.**
Read on 5 September 2026. It derives the members from an import of the compiled
module, holds no copy of a docstring, and fails when a member's first line of
prose reaches no page of the built site. The documentation build runs it twice,
and a probe proves the documentation job can fail.[^A1] [^A2]

**Four things are missing.** The name list itself: a public member with no doc
comment is printed as a note and does not fail. The reverse direction: nothing
fails when the site holds a name the package no longer exports. The derivation
this item asks about is unsettled, because the check reads the compiled module's
members rather than the package's exported names. And no proof exists that
either direction can fail.

**So this item is now the name list and its two directions**, not the check
itself.

## What the work does

Add a check that reads the exported names from the installed package, reads the
names the published site holds, and fails when the two disagree in either
direction.

Run it in the job that builds the site, so it fails with the build rather than
after it.

**Put the defect back and watch the check stay green.** Add a name to the
package and confirm the check fails and names it. Remove a name from the site
and confirm it fails again. A check that has never been shown to fire has not
been shown to exist.[^5]

## What it does not do

It does not check that a name has prose. That is a separate question, and one
item reads the page for it.[^6]

It does not check the signatures. Item 0307 holds the comparison between the
type stub and the module.[^7]

## Why this is not refined

The check compares against a published site, and no site exists yet.[^8] How the
check reads the site depends on what that item builds. Refining this item answers
that, and it also decides whether the package's exported name list or the
compiled module's members is the right derivation, because the two are not the
same set.

## References

[^1]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: Findings register, FND-223. `docs/FINDINGS.md`
[^3]: Findings register, FND-242. `docs/FINDINGS.md`
[^4]: ADR-0107, the Python reference is generated from the compiled module, the consequences. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^5]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^6]: Backlog item 0310, write the Rust doc comments for the Python reader. `docs/backlog/complete/0310-write-the-rust-doc-comments-for-the-python-reader.md`
[^7]: Backlog item 0307, generate the type stub from the compiled module. `docs/backlog/proposed/0307-generate-the-type-stub-from-the-compiled-module.md`
[^8]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/complete/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
[^A1]: The reference check script. `scripts/check_reference.py`
[^A2]: The documentation build and its probe. `scripts/build-docs.sh`
