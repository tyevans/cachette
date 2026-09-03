# Working Report — item 0309, the generated Python reference

This document reports the work of one backlog item.[^1] It is a record of one
moment, and nobody maintains it.

Date of the work: 3 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The Python package re-exports a compiled extension module, and the prose
that a Python developer wants is written as a Rust doc comment in the bindings
crate. A record binds where that prose lives and how it reaches a reader.[^2]

## 1. What the work built

**The dependency group.** The project manifest now holds a group for the
documentation build tools: the site builder and the handler that reads a Python
module.[^3] They are build tools of one job, so they are not runtime
dependencies of the package and they are not in the development group.

**The site configuration.** It is written in the portable format at the root of
the repository.[^4] It names the source directory, the output directory, the
navigation, three Markdown extensions and one plugin. It sets no site address,
and a comment beside the empty setting names the blocker that holds the
address.[^5] It also carries the comment that decision D1 of the record asks
for: the option that turns module inspection off is named, and the configuration
states that it never sets it.[^2]

**The site source.** Two pages.[^6] [^7] The reference page holds one directive
over the compiled module and no interface prose. The home page is marked as an
index, and it says what the site holds and what it does not hold yet. The
navigation holds the reference and the home page, and it carries the three prose
quadrants as commented entries that name the items that write them.

**The build job.** One script.[^8] It builds the extension and installs it, then
checks that the build environment imports the compiled module, then builds the
site, then checks the built site against the module. A recipe of the command
runner calls it.[^9]

**The guard.** One script.[^10] It imports the compiled module, walks the public
members, takes the first line of each docstring, and requires every one of them
on a page of the built site. It holds no copy of any docstring, so the prose has
one declaration site and the check derives its expectation on every run. It also
reports every public member that carries no prose, without failing, because the
record says that such a member publishes with no prose and that nothing fails on
it.[^2]

**The probe.** One script and one broken configuration.[^11] [^12] The probe
breaks the job in two ways and requires it to fail each time. A recipe calls
it.[^9]

**The workflow.** One job builds the site on every push, on every pull request
and on demand, and keeps the result as an artefact. A second job publishes, and
it is off.[^13]

## 2. How the work proved that the import failure is caught

The rule on testing says that a guard with no proven failure mode is
decoration.[^14] The work did not assert that the job fails. It broke the job,
ran it, and watched it fail.

**Case 1, the compiled module is not in the build environment.** The probe reads
the file name of the module from the module itself, moves that file aside, and
runs the job with the build step skipped. The job stopped before it built the
site:

```
FAIL import: the build cannot import cachette._core: No module named 'cachette._core'
ADR-0107 D4: the job builds the extension before it builds the site.
```

The probe then moved the file back.

**Case 2, the configuration turns module inspection off.** This is the case that
matters, because it is the one that reports nothing on its own. The probe copies
the broken configuration to the repository root and runs the job against it. The
builder ran in strict mode, reported `No issues found`, and exited zero. The
check then read the built site and failed:

```
FAIL prose: 46 members carry prose that no page holds
  cachette._core.Camera.clamp: Holds the view inside the world.
  ...
  cachette._core.version: Returns the version of the engine.
```

46 of the 57 summaries were gone from a site that reported no issue. The probe
then removed the copy.

**Both cases run in one command**, and the command restores what it breaks
whether it passes or not:

```
just docs-probe
```

**A finding holds the part of this that corrects a belief.** A reader of the
research report could conclude that the strict mode of the builder closes the
hole. It does not.[^15]

## 3. What the build produces

The build writes a static site to the output directory that the configuration
names. It holds the home page, the reference page, a search index, a site map,
an object inventory and the theme assets.

The reference page carries the prose of the compiled module. It holds the
sentence "Runs one frame and returns the number of events", which exists only as
a Rust doc comment in the bindings crate. It holds the footnote sections that
those doc comments carry, and the research report already measured that the
generated footnote identifiers do not collide.

The page also carries the typed dictionaries of the type stub with their own
prose, which is what decision D3 of the record asks for: the module does not
provide them, so the stub is their one home.[^2]

## 4. What stays blocked

**This section is out of date, and item 0320 is why.** It describes the tree on
3 September 2026, before the project owner named the host. The blocker is
resolved, the site configuration states the address, and the publishing job
carries no repository variable. One setting of the repository is what is left,
and one item holds it.[^21] Read the rest of this section as the state at the
time of the review.

**The address and the hosting switch.**[^5] Nobody in this tree can state
either, and the work invented neither.

The publishing job is written and it does nothing. It runs only on the trunk and
only when a repository variable says that publishing is on. The address comes
back from the host that answers on it, so no file in this repository states one.
The site configuration leaves the address setting empty for the same reason.

**What the project owner must do to publish.** Set the repository variable that
the job reads, and set the hosting source to the workflow. Until then the job
builds the site on every run and keeps it as an artefact that a person can
download.

## 5. The gates

Three gates ran. Each was green.

**`just records`.**

```
checked 71 records: 0 failures, 2 notes
checked 21 product records: 0 failures
checked 215 backlog items: 0 failures
checked 363 register entries: 0 failures
checked 161 priority rows: 0 failures
checked 5957 citations in 433 files outside the records: 0 failures
checked 675 files for a conflict marker: 0 failures
checked the footnotes of 370 documents: 0 failures, 101 baselined, 63 out of body order
```

The two record notes name ADR-0082 and ADR-0097, and both predate this work.

**`just lint-python`.**

```
uv run ruff check python tests
All checks passed!
uv run mypy
Success: no issues found in 18 source files
```

**`just test-python`.**

```
74 passed in 181.29s (0:03:01)
```

**Two recipes are new.** One builds the site and one proves that the build can
fail.[^9] Neither is in the gate command. A register row holds that choice with
its options and a recommendation, because the site job is not a merge gate for
anybody until somebody reads the site.[^16]

## 6. The work against the record, decision by decision

**D1, the reference is generated by an import of the compiled module. Honoured.**
The reference page holds one directive and no interface prose. The configuration
never sets the option that turns inspection off, and it names the option in a
comment beside the settings it does use. The check that reads the built site
fails when the prose does not reach the page, so the decision is enforced and
not only stated.

**D2, the prose lives in the Rust doc comment. Honoured, and untouched.** The
work wrote no prose for any member that the compiled module provides. It changed
no file under the crates directory.

**D3, the stub declares types and carries prose only where the module provides
none. Honoured, and untouched.** The work changed no file of the Python package.
The typed dictionaries reach the page with their stub prose, and every other
member reaches it with the prose of the module.

**D4, the job builds the extension first and fails when the import fails.
Honoured, and proven.** Section 2 holds the proof.

## 7. What the work left undone, and what it found

**Item 0307 has not closed.** It is still in the proposed directory. The plan and
the priority index both say that no documentation build starts before it,
because a reference built over two declaration sites publishes whichever site
the tool reached.[^17] [^18] The work ran anyway, because it was dispatched. The
observed behaviour is that the builder reads both sites and merges them: the
methods came from the module and the typed dictionaries came from the stub. A
member that the stub and the module both describe would publish one of the two,
and nothing would report it. Item 0307 closes that.

**No page states which citations a reader cannot follow.** One open choice asks
how a published page cites a repository document that a reader cannot open.[^19]
The two pages this work wrote cite no repository document, so the work did not
meet the choice and did not close it. The explanation quadrant meets it first.

**The blocker number that was allocated to this work is unused.** The work found
no missing information beyond the address, which already has a row.

**No backlog item was created.** The two numbers allocated for one are unused.

## 8. For the worker on item 0310

The work read the generated page and the collected docstrings. It did not change
any of them. Three observations follow, and each is about the audience of the
prose rather than its absence.

**Every public member already carries prose.** The import finds 57 public
members and every one has a docstring. The item's first half is closed and its
second half is not. A finding holds the count and the evidence.[^20]

**The constructor of a class carries no prose, and the reference cannot show
what a caller must pass.** The binding library does not copy the doc comment of
a constructor onto the Python object. The module carries the standard
interpreter sentence there instead, and the member filter removes it. A reader
of the page therefore learns that a world is a simulated world and never learns
how to build one. The repair is prose in the doc comment of the class, because
that is the comment the import does reach.

**Two properties of the camera share one summary.** The horizontal offset and
the vertical offset both read "The pixel offset of the tile at the origin." A
reader who lands on the page cannot tell them apart by their prose. Name the
axis in each.

## References

[^1]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/complete/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
[^2]: ADR-0107, the Python reference is generated from the compiled module, decisions D1, D2, D3 and D4. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^3]: The project manifest, the documentation dependency group. `pyproject.toml`
[^4]: The site configuration. `mkdocs.yml`
[^5]: Blockers register, BLK-035, resolved. `docs/BLOCKERS.md`
[^6]: The reference page. `docs/site/reference.md`
[^7]: The site index page. `docs/site/index.md`
[^8]: The documentation build script. `scripts/build-docs.sh`
[^9]: The command runner, the `docs` and `docs-probe` recipes. `justfile`
[^10]: The reference check. `scripts/check_reference.py`
[^11]: The documentation probe. `scripts/docs-probe.sh`
[^12]: The broken site configuration. `tests/fixtures/docs-inspection-off/mkdocs.yml`
[^13]: The documentation site job. `.github/workflows/docs.yml`
[^14]: Testing Rules, sections 1 and 2a. `.claude/rules/testing.md`
[^15]: Findings register, FND-324. `docs/FINDINGS.md`
[^16]: Decisions register, DEC-115. `docs/DECISIONS.md`
[^17]: Backlog item 0307, generate the type stub from the compiled module. `docs/backlog/proposed/0307-generate-the-type-stub-from-the-compiled-module.md`
[^18]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^19]: Decisions register, DEC-114. `docs/DECISIONS.md`
[^20]: Findings register, FND-325. `docs/FINDINGS.md`
[^21]: Backlog item 0320, publish the documentation site to GitHub Pages. `docs/backlog/complete/0320-publish-the-documentation-site-to-github-pages.md`
