---
id: 0321
title: Turn the documentation site publishing on
status: proposed
created: 2026-09-03
implements: [ADR-0107 D4]
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**The documentation site builds, it has an address, and it does not answer on
it.** The workflow builds the site on every run and publishes it on every push
to the main branch.[^1] The site configuration states the address.[^2] One
setting of the repository is missing, and the host reads that setting rather
than any file in this tree.

**Only the project owner can change it.** The setting says whether the host
serves the site from a workflow or from a branch. A contributor cannot read it
from the tree and cannot change it. This is why the work sits in a separate
item: it is an action in a browser, and the rest of the work is done.[^3]

**The publishing job fails until the setting changes.** It carries no switch of
its own, and the decisions register says why.[^4] The failure names the setting
and the person who can change it. That is the intended signal, and it stops as
soon as he acts.

**A developer who builds a game on this engine cannot read the reference until
this happens.** A product record states the need and names the audience.[^5]

## What the work does

**The project owner sets the hosting source of the repository to the workflow
rather than to a branch.** This is the whole of the manual part.

**Then somebody confirms that the site answers.** Open the address, read the
reference page, and confirm that the prose of a method is on it. The build
already checks that the prose reached the built site, and this checks that the
built site reached the reader.[^6]

**Then somebody reads the page the host serves for an unknown address.** Ask
for an address the site does not hold, and confirm that the page loads its own
style and that its links stay inside the site. A finding holds why this page is
the one to check.[^7]

## What good looks like

The address answers with the index page of the site.

The reference page holds a sentence that exists only in the Rust doc comment of
the bindings crate.

A request for an address the site does not hold returns the page the build
made, with its style and with links into the site.

The publishing job passes on the main branch, and it has passed at least once.

## What it costs at the target scale

Nothing at run time. No simulation code changes and no engine code changes.

The job costs what it costs today. This item adds no step to it.

## What it does not do

It does not write a tutorial, a how-to guide or an explanation page. Three
separate items hold those.[^8]

It does not decide whether the documentation build joins the whole check
command. A row in the decisions register holds that choice, and it waits for
the site to answer.[^9]

It does not add a check that the published site is reachable. Nothing in this
repository can reach the host, and a check that needs the network is a check
that fails for a reason that is not the project's.

## References

[^1]: The documentation site job. `.github/workflows/docs.yml`
[^2]: The configuration of the documentation site. `mkdocs.yml`
[^3]: Backlog item 0320, publish the documentation site to GitHub Pages. `docs/backlog/complete/0320-publish-the-documentation-site-to-github-pages.md`
[^4]: Decisions register, DEC-117. `docs/DECISIONS.md`
[^5]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^6]: The reference check. `scripts/check_reference.py`
[^7]: Findings register, FND-334. `docs/FINDINGS.md`
[^8]: Backlog item 0308, the documentation plan. `docs/backlog/refined/0308-the-documentation-plan.md`
[^9]: Decisions register, DEC-115. `docs/DECISIONS.md`
