# Working Report — item 0320, publish the documentation site to GitHub Pages

This document reports the work of one backlog item.[^1] It is a record of one
moment, and nobody maintains it.

Date of the work: 3 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The project builds a documentation site from this repository. The
Python reference on that site comes from an import of the compiled extension
module, and a record binds that.[^2] Before this work the site built on every
run and published nowhere, because no file in the tree could say where it
answers.

## 1. What was closed, and how

**The blocker on the documentation site is resolved.**[^3] The project owner
stated the host directly in the dispatching session on 3 September 2026. The
site publishes to GitHub Pages. This repository does not carry the name of its
owner, so the site is a project site and it answers on a path below the host
name of the owner. The site configuration now states the address, and it is the
one declaration site of it.[^4]

**The row asked for two things, and only one of them was information.** The
address is a fact the project did not have, and this register holds exactly
that. The hosting source setting of the repository is an action that one person
takes in a browser. A finding records the distinction, and one backlog item now
holds the action.[^5] [^6]

**The row moved to the resolved section with the outcome and the reasoning.**
It names the owner, the date and what closed it.

## 2. What changed in the tree

**The site configuration states the address.** The comment beside it says where
the address comes from and what the builder does with it. Nothing else in the
configuration was left parametric under the blocker. The address setting was
the only one, and the rest of the file names the source directory, the output
directory, the navigation, the extensions and one plugin.

**The publishing job lost its switch.** It runs on every push to the main
branch and carries no repository variable. The decisions register holds the
choice.[^7]

**The command runner comment no longer names the blocker.** It says that the
recipe builds and publishes nothing, and that the workflow publishes from the
main branch.

## 3. The sweep

The rule on a resolved blocker says to search the whole tree for its number and
to repair every place that calls it open.[^8] This is the step the definition
of done names as having gone wrong twice.

The command:

```
grep -rn "BLK-035" --exclude-dir=.git .
```

It found eleven lines in eleven files before the work. Every one is repaired.
The files:

| File | What it said | What it says now |
|---|---|---|
| `mkdocs.yml` | The address is not in this tree | The address, with a comment on what the builder does with it |
| `.github/workflows/docs.yml` | The row holds the address, and the job takes a repository variable | Nothing about the row. The job publishes from the main branch |
| `justfile` | The row holds the address the site publishes to | The workflow publishes, and the configuration holds the address |
| `docs/BLOCKERS.md` | The open row | The resolved row, with the outcome and the date |
| `docs/DECISIONS.md` | DEC-115 waits for the address | DEC-115 says the address exists and asks to be read again |
| `docs/backlog/PRIORITY.md` | The row governs item 0308 | The row governed it and is resolved |
| `docs/backlog/refined/0308-the-documentation-plan.md` | Blocked by the row, and every item states the address as a parameter | Blocked by nothing, and the configuration states the address |
| `docs/backlog/complete/0309-...` | Blocked by the row, the address is a parameter, the publishing step is off | The row is resolved, and item 0320 replaced the parameter |
| `docs/reviews/0309-the-generated-reference.md` | A section titled what stays blocked | The same section, with a dated note that says it is out of date |
| `docs/reviews/0308-the-documentation-plan.md` | The row is open | The row was open at the time of the review, and it closed |

The two review documents are records of one moment. The work added a dated note
to each rather than rewriting what a reviewer saw.

**A second search covered the switch that went away.**

```
grep -rn "CACHETTE_DOCS_PUBLISH" --exclude-dir=.git .
```

It returns nothing.

After the repair, the first search returns eleven lines and not one of them
calls the row open. Every remaining line is a footnote that names the row as
resolved, the row heading itself, or a priority row that says the row is
resolved.

## 4. The repository variable, and why it went

**The decision is that the publishing job keeps no switch of its own.** It
publishes on every push to the main branch. The decisions register holds the
row, the option it rejected and the reasoning.[^7]

**The variable was a second declaration site for one fact.** The hosting source
setting of the repository already says whether the host serves the site from a
workflow. The deploy step reads that setting. A repository variable beside it
can disagree with it, and nothing fails when it does. This project names that
shape as its first recurring defect, and it names it with local evidence.[^9]

**The variable also hid the state of the job.** With the variable unset, a job
that is turned off and a job that is broken look the same, and neither reports
anything. The project rule on testing says that a guard nobody has seen fire
has not been shown to exist.[^10] Under the variable, nobody ever takes the
publishing path.

**The price is honest and it is stated.** The publishing job fails on the main
branch until the project owner changes the hosting source setting. That failure
names the setting and the person who can change it, and it stops as soon as he
acts. A variable that is never set produces silence instead, and silence is
what let the site build for a day and publish nowhere.

## 5. What Ty must do in the GitHub interface

**Two steps. Nobody else can take them, and this work did not take them.**

1. **Open the repository settings, go to the Pages section, and set the build
   and deployment source to GitHub Actions.** It is set to a branch, or to
   nothing, today. This is the setting the publishing job needs. Nothing in
   this repository can read it or change it.
2. **Confirm that the environment named `github-pages` accepts a deployment
   from the main branch.** GitHub creates that environment when Pages turns on.
   If it carries a branch protection rule that excludes the main branch, the
   publishing job stops there.

**Then push to the main branch, or run the workflow by hand**, and read the
address the job reports. Item 0321 holds the checks that follow: open the
address, read the reference page, and ask for an address the site does not hold
to confirm that the page for an unknown address links inside the site.[^6]

**Until step 1 is done, the publishing job fails on the main branch.** That is
the intended signal, and section 4 says why.

## 6. The site is a project site, and the build does not assume the root

A project site serves from a path below the host name and not from the root of
the domain. The work checked what the build writes.

**Every ordinary page uses relative links.** The build writes no root-absolute
link into the home page or the reference page. The directory URL setting keeps
its default, which is what produces those relative links, so nothing needed to
change.

**One page is different, and it was wrong before this work.** The builder
generates a page for an address the host does not hold. That page cannot use a
relative link, because the host serves it for any address. The builder writes
an absolute path into every link and asset of it, and it derives that path from
the site address. With no address, it wrote the root of the domain, which is
outside a project site.

The evidence is two builds of one tree, one with the address and one without:

```
grep -o '\(href\|src\)="/[^"]*"' target/site/404.html | sort -u
```

With the address, each link starts with the repository name. Without it, each
link starts at the root. A finding holds this.[^5]

## 7. The gates

**`just docs`.** Green.

```
==> check that the build environment imports the compiled module
cachette._core: 59 members with prose, 0 without
==> build the site from mkdocs.yml
Build started
No issues found
Build finished in 3.13s
==> check that the built site carries the prose of the compiled module
cachette._core: 59 members with prose, 0 without
every one of the 59 summaries reached the site
```

The job still reports that every summary reaches the site.

**`just docs-probe`.** Green. Both cases still fail the job.

```
==> case 1: take the compiled module out of the build environment
FAIL import: the build cannot import cachette._core: No module named 'cachette._core'
ADR-0107 D4: the job builds the extension before it builds the site.
the job failed, as it must

==> case 2: turn module inspection off
FAIL prose: 46 members carry prose that no page holds
ADR-0107 D1: the reference comes from an import of the compiled module.
the job failed, as it must

both cases failed the job
```

**`just records`.** Green.

```
checked 71 records: 0 failures, 2 notes
checked 24 product records: 0 failures
checked 221 backlog items: 0 failures
checked 376 register entries: 0 failures
checked 164 priority rows: 0 failures
checked 6122 citations in 454 files outside the records: 0 failures
checked 683 files for a conflict marker: 0 failures
checked the footnotes of 380 documents: 0 failures, 101 baselined, 65 out of body order
```

The two record notes name ADR-0082 and ADR-0097, and both predate this work.
The conflict marker note about a fixture directory is the normal output of the
repository scan.

**The footnote check failed once, and the work repaired it.** Three new
footnotes gave one source a second label. The repair reuses the existing label
in each case, which is what the documentation rule asks for.[^11]

**No other gate ran.** This change touches no Rust and no Python source, so the
formatting checks, the lint, the test suites and the two determinism tests were
not exercised. That is a skipped step and not a green claim.

## 8. How the workflow was validated without a push

**The file parses, and the structure was read.** A parser read the file and
returned two jobs, the condition of the publish job, its permissions, its
environment and its one step. The condition holds one term and no variable.

**No linter for a workflow file is available in this environment.** The
`actionlint` tool is not installed, and it is not in the package index that
this project's tool runner reads. The work did not install one.

**The publishing path is not proved.** Nobody has seen this job publish. It
cannot run here, and it cannot run anywhere until the setting in section 5
changes. Section 4 states that plainly rather than claiming a guard that has
never fired.

## 9. The work against the record, decision by decision

**D1, the reference is generated by an import of the compiled module.
Honoured.** Nothing in the source of the reference changed. The build still
imports the module, and the check still derives its expectation from that
import on every run.

**D2, the prose lives in the Rust doc comment. Honoured.** This work wrote no
prose for any member of the compiled module.

**D3, the type stub declares types. Honoured.** This work did not touch the
stub.

**D4, the job builds the extension before the site and fails when the import
fails. Honoured, and not weakened.** The build job is unchanged. The probe
recipe is unchanged. Both still run before anything publishes, and the publish
job runs only after the build job passes. The probe was run, and it still fails
the job in both of its cases.

## 10. The registers

- **Blockers.** BLK-035 resolved. No new row opened. The one thing that is
  left is an action rather than missing information, and a register of blockers
  holds information.[^3]
- **Decisions.** DEC-118 opened and closed in one change: the publishing job
  keeps no switch of its own.[^7] DEC-115 gained a paragraph, because it says
  in its own text to read it again when the address exists, and the address now
  exists.
- **Findings.** FND-336 and FND-337.[^5]
- **Backlog.** Item 0320 is this work, and it is complete. Item 0321 holds the
  action in the browser and the checks that follow it, and it is proposed.[^6]
- **Priority index.** Item 0321 is placed directly under item 0308. Item 0320
  is complete, so it is not listed.
- **Registry.** Untouched. No decision record was written and no number was
  allocated.

## 11. What was left undone

- **The site does not publish yet.** Section 5 holds the two steps, and only
  the project owner can take them.
- **The publishing job has never run to completion.** Nobody has seen it
  publish, and this report does not claim that it works.
- **The main branch will fail this job** until the project owner changes the
  hosting source setting. That is a deliberate choice and section 4 gives the
  reasoning. It is also the one cost of that choice, and it is stated here
  rather than discovered.
- **No documentation page was written.** The tutorial, the how-to guides and
  the explanation pages belong to three other items, and they were out of
  scope.
- **DEC-115 is still open.** The work added the fact it waited for and did not
  choose between its three options.
- **The record is a draft.** ADR-0107 binds nothing until a reviewer accepts
  it, and this work is written against a draft.[^2]

## References

[^1]: Backlog item 0320, publish the documentation site to GitHub Pages. `docs/backlog/complete/0320-publish-the-documentation-site-to-github-pages.md`
[^2]: ADR-0107, the Python reference is generated from the compiled module, decisions D1, D2, D3 and D4. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^3]: Blockers register, BLK-035, resolved. `docs/BLOCKERS.md`
[^4]: The configuration of the documentation site. `mkdocs.yml`
[^5]: Findings register, FND-336 and FND-337. `docs/FINDINGS.md`
[^6]: Backlog item 0321, turn the documentation site publishing on. `docs/backlog/proposed/0321-turn-the-documentation-site-publishing-on.md`
[^7]: Decisions register, DEC-118 and DEC-115. `docs/DECISIONS.md`
[^8]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
[^9]: Recurring Defect Shapes, shape 1, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^10]: Testing Rules, section 1. `.claude/rules/testing.md`
[^11]: Documentation Rules, section 3. `.claude/rules/documentation.md`
