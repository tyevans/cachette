# The Documentation Toolchain for the Python Control Plane

Research report 19 for the decision on how this project publishes its Python
reference documentation to GitHub Pages. Prepared 3 September 2026.

Every version number, release date and measured figure in this report carries a
footnote that says what the author consulted and on what date. A decision
record must not hold this material, so this report is its only home.[^1]

## 0. Context

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The Python package holds a small amount of pure Python and re-exports a
compiled extension module. A build tool named maturin compiles that module from
a Rust crate through the PyO3 binding library.[^2]

All prose in this repository follows Simplified Technical English. Every
document stands alone. Every external reference sits in a footnote, and all
footnotes sit in one section at the end of the document.[^3] A documentation
tool for this project must therefore render Markdown footnotes correctly, and
it must render them inside a docstring as well as inside an authored page.

The project owner named one candidate: Zensical. This report tests that
candidate against the surface this project must document, and against three
alternatives. The report does not confirm the candidate by default.

### 0.1 The findings

1. **Zensical does extract Python docstrings, and it does so on this
   project's compiled module today.** The author built a site against the real
   `cachette._core` module and read the output. The Rust doc comments reached
   the page. Section 4 gives the measurement.

2. **The load-bearing fact is not the tool. It is where the docstring lives.**
   The Rust source carries the prose. The type stub carries almost none of it.
   A tool that reads the stub and refuses to import the module produces a page
   with signatures and no prose. The author measured both. The page shrank from
   105,348 bytes to 29,038 bytes. Section 4.2 gives the numbers.

3. **Every candidate must import the compiled module.** Sphinx autodoc imports
   it. pdoc imports it. mkdocstrings imports it when static parsing fails, and
   static parsing always fails on a compiled module. No candidate escapes this,
   so it is not a discriminator between the candidates. It is a constraint on
   the build environment, and it is the thing a naive comparison misses.

4. **Zensical is alpha software, and it says so.** The published package
   carries the classifier `Development Status :: 3 - Alpha`. The project
   roadmap states that Zensical is alpha software. Native API documentation and
   native versioning are on the roadmap and are not built. Section 3.1 and
   section 3.7 give the detail.

5. **The choice of builder is nearly free, because Zensical reads
   `mkdocs.yml`.** The same project builds with either tool. This makes the
   decision reversible in both directions, and it moves the risk from the tool
   to the configuration format. Section 3.2 gives the evidence.

6. **The recommendation is to bind the configuration format and the docstring
   source, not the builder.** Section 9 states it. A decision record that names
   Zensical would pin an alpha release, which the record scope rule
   forbids.[^1]

7. **This project's Rust doc comments already hold heading sections and
   footnotes, and both survive the build.** Sixty-five footnote identifiers on
   one generated page were unique, because the generator prefixes each one with
   the member path. Section 4.3 gives the numbers and the cost that comes with
   them.

8. **The repository has no documentation build of any kind today.** There is no
   Pages workflow, no site configuration, and no documentation dependency.
   Section 7 reports what is there.

9. **One belief in the repository is false, and this work found it.** The type
   stub states in its own docstring that the build regenerates it and that a
   job fails when the result differs. No generator exists in the tree and no
   job compares anything. The findings register now holds this.[^4]

## 1. Terms

**A docstring** is the prose attached to a Python module, class or function at
run time. A reader gets it from the `__doc__` attribute.

**A doc comment** is a Rust comment that starts with three slashes. PyO3 copies
a doc comment onto the Python object that it creates, so a doc comment becomes
a docstring.[^2]

**A type stub** is a file with the suffix `.pyi`. It declares the signatures of
a module without holding its implementation. A type checker reads it. A stub
may hold docstrings, and it may hold none.

**Static analysis** means that a tool parses source text and never runs it.
**Inspection** means that a tool imports a module and reads the objects that
the import creates. A compiled module has no Python source text, so static
analysis of the module itself is not possible. A tool that wants the compiled
module's docstrings must inspect it.

**A compiled extension module** is a shared object file. In this repository the
file is `_core.abi3.so`, and maturin builds it.

## 2. What this project must document

The author read the surface before judging any tool against it.

The Python package holds four parts. The first is the package entry point. It
re-exports twelve names from the compiled module and sets a version string. The
second is an agent-facing server. The third is a demonstration application and
its drawing surface. The fourth is the type stub for the compiled module.[^5]

The prose that a reader wants is not in the Python source. It is in the Rust
crate that PyO3 compiles. That crate holds 1,558 lines, and 543 of them are doc
comment lines.[^6] The doc comments are long, and many hold a `# References`
heading and Markdown footnote definitions. That style follows the project
documentation rule, which forbids a reference in body text.[^3]

The type stub holds 398 lines and 67 class or function definitions. It holds 36
docstrings. Every one of those 36 sits on a typed dictionary or on an exception
class. **No method of `World` and no method of `Camera` carries a docstring in
the stub.**[^7] Those are the classes a user calls.

This is the shape that decides the choice. One fact, the prose for a method,
has one authoritative home: the Rust doc comment. The stub is a second
declaration site that holds the signature and not the prose. The project rule
on recurring defects names this shape, and it asks for a check that fails when
two sites disagree.[^8]

## 3. Zensical, verified against primary sources

### 3.1 Publisher, licence and maturity

The Material for MkDocs team publishes Zensical. The licence is MIT. The
repository names Rust as its primary language, and the language byte counts are
1,138,712 for Rust and 248,569 for Python.[^9]

The current release is 0.0.58. The project published it on 2 September 2026.
Fifty-eight releases exist. The published package requires Python 3.10 or
later, and it declares the classifier `Development Status :: 3 - Alpha`.[^10]

The project roadmap states the same thing in words. It says that Zensical is
alpha software, that the team iterates rapidly, and that the team will remove
remaining defects from the initial implementation in the first months.[^11]

**Verified.** Publisher, licence, release number, release date, release count,
the alpha classifier and the roadmap statement all come from primary sources.

### 3.2 The relationship to MkDocs and to Material for MkDocs

Zensical is a rewrite, not a plugin and not a theme. It replaces both MkDocs and
Material for MkDocs with one stack that covers site generation, theming and
customisation.[^12]

Zensical reads `mkdocs.yml` natively. The migration guide states that the same
project builds with either command. It presents the existing MkDocs build as a
safety net that a project keeps while it moves across.[^13]

**The migration path runs in both directions, and that is the important
property.** A project keeps `mkdocs.yml` for as long as it wants. Moving to the
native `zensical.toml` format is optional. Some settings have no support yet:
`remote_branch`, `remote_name`, `exclude_docs`, `draft_docs`, `not_in_nav` and
`hooks`. Three command line flags also have no support: `--theme`,
`--use-directory-urls` and `--site-dir`. A project sets those in the
configuration file instead.[^13]

Zensical rewrites the MkDocs plugins rather than running them. The compatibility
page calls each one a behaviour-preserving rewrite that does not use the
original code.[^14]

**Verified.** All of the above comes from the Zensical documentation.

### 3.3 How it builds, and what it needs

The builder is Rust. The distribution is a Python package on the package index,
so a Python project installs it with `pip install zensical` or with
`uv add --dev zensical`. Container images also exist.[^15]

The package depends on eight Python packages, and three of them decide the
Markdown behaviour: `markdown`, `pygments` and `pymdown-extensions`.[^10] **This
matters.** Zensical keeps the Python Markdown pipeline. It does not replace it
with a Rust Markdown parser. A Markdown extension that works under MkDocs
therefore behaves the same way here.

Zensical does not support the symbolic link install mode of the `uv` tool. The
team names this as an open limitation. It also states that it cannot support
the conda-forge distribution, because it does not control it.[^15]

**Verified.** Install commands, dependency list and the two limitations come
from primary sources.

### 3.4 Docstring extraction, the load-bearing question

**Zensical does generate Python API documentation from docstrings, through
mkdocstrings.** It has done so since release 0.0.11. The Zensical
implementation is a rewrite of the mkdocstrings plugin. A user still installs
the real handler package, `mkdocstrings-python`, from the package index.[^14]

Two limitations are stated. Backlinks have no support. Zensical does not watch
sources outside the project directory during a preview build.[^14] One further
gap is open in the issue tracker: cross-references between mkdocstrings
identifiers.[^16]

The handler is the original `mkdocstrings-python`, and that package uses Griffe
to collect the API. Griffe parses source text where source text exists. Where a
module is built in or compiled, Griffe imports the module and reads the objects
instead. Griffe also reads a `.pyi` stub when one exists, and its documentation
recommends turning inspection off in that case, because inspection collects
many more members and the collected data can be low level.[^17]

Three handler options govern this. `allow_inspection` defaults to true and
permits the import. `force_inspection` defaults to false and forces the import
even when source text exists. `find_stubs_package` defaults to false and looks
for a separate stub package.[^18]

**Unverified.** The author did not confirm from the Zensical source that the
Zensical rewrite passes every handler option through to `mkdocstrings-python`.
The author did confirm that one option changes the output. Section 4.2 gives
that measurement.

### 3.5 The structural features this project needs

| Need | Zensical support | Evidence |
|---|---|---|
| Navigation tree | Yes, from `nav` in `mkdocs.yml` | measured, section 4 |
| Footnotes | Yes, through the Python Markdown `footnotes` extension | documented and measured |
| Admonitions | Yes, and collapsible ones through `pymdownx.details` | documented |
| Code highlighting | Yes, through `pymdownx.highlight` and Pygments | documented |
| Search | Yes, a native rewrite of the search plugin, since 0.0.3 | documented and measured |
| Versioned documentation | Only through a maintained fork of `mike` | documented |

Zensical documents footnote syntax and gives the configuration for both file
formats. It also offers a theme feature that renders a footnote as a
tooltip.[^19]

Zensical states that every extension that is part of the `pymdown-extensions`
package should work, and it names the ones with a native rewrite.[^20]

**Versioned documentation is the weak entry.** Zensical does not implement
versioning. It ships a fork of the `mike` tool, based on `mike` 2.2.0, which a
user installs from the source repository rather than from the package index.
The fork takes compatibility fixes and no new features. The Zensical
documentation calls the arrangement transitional and says that the team will
maintain the fork until Zensical provides native versioning.[^21] The roadmap
lists versioning as a feature that is not yet built.[^11]

### 3.6 Deployment to GitHub Pages

Zensical documents a GitHub Actions workflow. **There is no official Zensical
action.** The documented workflow installs Zensical with `pip`, runs
`zensical build --clean`, uploads the output with the standard Pages artefact
action, and deploys with the standard Pages deploy action. The workflow needs
the repository to publish Pages through GitHub Actions rather than from a
branch. The documentation advises against a build cache on a continuous
integration system for now, because the caching behaviour will change.[^22]

Zensical also documents GitLab Pages, Azure Static Web Apps and Read the
Docs.[^22]

**Verified.** The workflow shape comes from the Zensical documentation. The
author did not run this workflow.

### 3.7 What Zensical cannot do that the alternatives can

- **It has no stable public extension interface.** The module system is under
  test and the team has held the public interface back until it is confident
  that no breaking change is coming.[^16] Material for MkDocs runs on MkDocs,
  which has a documented plugin interface today. Sphinx has a large extension
  ecosystem.
- **It does not run an arbitrary MkDocs plugin.** It runs its own rewrite of a
  named list of plugins. A plugin outside that list has no support, because
  Zensical does not execute the original code.[^14]
- **It has no `hooks` support**, so a project cannot run its own Python code in
  the build.[^13]
- **It has no native versioned documentation.** A fork of another tool fills
  the gap.[^21]
- **It cannot yet cross-reference mkdocstrings identifiers**, and it does not
  render mkdocstrings backlinks.[^14] [^16]
- **It is alpha, and the release cadence is high.** Fifty-eight releases have
  shipped since the repository opened on 18 May 2025.[^9] [^10] A project that
  pins a version accepts frequent upgrade work. A project that does not pin
  accepts frequent behaviour changes.

## 4. The measurement

A comparison from documentation is weaker than a build. The author therefore
built a site against the real compiled module of this repository.

**Method.** The author created a virtual environment on Python 3.13.5 on x86-64
Linux. The author installed Zensical 0.0.58 and `mkdocstrings-python` 2.0.8.
That install brought MkDocs 1.6.1, mkdocstrings 1.0.6, `mkdocs-autorefs` 1.4.4
and `griffelib` 2.2.0. The author added the repository's `python` directory to
the environment, so that the environment imported the compiled module that this
worktree built. The site held one page. That page held one mkdocstrings
directive for `cachette._core.World`. The configuration file was `mkdocs.yml`,
and it named the `footnotes` and `admonition` Markdown extensions.[^23]

### 4.1 The default configuration works

`zensical build --clean` reported no issue and finished in 3.07 seconds. The
generated page was 105,348 bytes. The prose from the Rust doc comments was
present. The page held the text "Runs one frame and returns the number of
events", and it held the sentence about releasing the global interpreter lock.
Both come from a Rust doc comment in the bindings crate.[^6] [^23]

**The default configuration also produced unwanted members.** The page
documented `__doc__` and `__module__` as class attributes, and it printed the
CPython `str` docstring for each. This is the over-collection that the Griffe
documentation warns about for an inspected module.[^17] A member filter removes
it. With the filter `!^__` the build finished in 2.21 seconds, the page was
100,548 bytes, and no `__doc__` member remained.[^23]

### 4.2 The stub alone is not enough, and this is the decisive result

The author then set `allow_inspection` to false. Griffe could not import the
module, so it fell back to the type stub.

The build still succeeded in 1.42 seconds. **The page fell from 105,348 bytes to
29,038 bytes.** The class one-line summary survived, because the stub holds it.
Every method summary disappeared. The page no longer held "Runs one frame" and
no longer held the sentence about the global interpreter lock. In place of the
prose the page rendered the stub source text of the class.[^23]

**This is the single fact most likely to change the decision.** The published
reference is only useful when the build imports the compiled module. The stub
is not a substitute for the module, because the stub does not carry the prose.

Two consequences follow, and neither depends on which tool the project picks.

- The documentation job must build the extension before it builds the site. It
  is a compile step, not a text step. A workflow that installs a documentation
  tool and runs it against the source tree produces an empty reference.
- Either the prose stays in Rust and the build imports the module, or the prose
  moves into the stub and the project accepts a second declaration site for
  every docstring. The project rule on recurring defects argues against the
  second option, unless a check compares the two sites.[^8]

### 4.3 Footnotes inside a docstring survive, and each page pays for it

The project documentation rule puts every reference in a footnote, and the Rust
doc comments follow it.[^3] [^6] The author measured what happens when many such
docstrings land on one page.

The generated page held 34 footnote references and 34 matching back references.
It held 65 footnote element identifiers, and **all 65 were unique.** The
generator prefixes each identifier with the member path, for example
`cachette._core.World.draw--fn:1`. Identifiers therefore do not collide when
twenty methods each define a footnote numbered one.[^23]

The cost is the heading structure. Each doc comment carries its own
`# References` heading, and many carry a `# Errors` heading. The one generated
page held 16 headings that read "References" and 20 that read "Errors". Those
headings enter the page contents list. A reader of the page contents sees the
word "References" sixteen times.[^23]

This is a consequence of the project documentation rule meeting a generated
page. It is not a defect in any tool. A project fixes it by choosing a heading
level for a docstring section, or by accepting the repetition.

### 4.4 What the measurement does not prove

- The author built one page against one class. The author did not build the
  whole repository documentation set.
- The author did not run the GitHub Pages workflow.
- The author did not build the same site with Material for MkDocs and compare
  the output. The comparison in section 5.1 is therefore reasoned, not measured.
- The author did not test Sphinx or pdoc against the compiled module.

## 5. The alternatives

### 5.1 Material for MkDocs with mkdocstrings

**The property that matters: it is the same configuration and the same
handler.** Zensical reads `mkdocs.yml`, and Zensical's mkdocstrings support is a
rewrite of the same plugin driving the same handler package.[^13] [^14] A
project that writes `mkdocs.yml` can run either builder against it. The
alternative is therefore not a competitor. It is the fallback that makes the
Zensical choice safe.

MkDocs 1.6.1 shipped on 30 August 2024 and carries the classifier
`Development Status :: 5 - Production/Stable`. Material for MkDocs 9.7.7
shipped on 17 July 2026 and carries the same classifier. mkdocstrings 1.0.6
shipped on 11 July 2026 and `mkdocstrings-python` 2.0.8 shipped on 31 August
2026. Both carry `Development Status :: 4 - Beta`. Griffe 2.2.0 shipped on 16
August 2026 and carries the production classifier.[^24]

**The property that fails: build speed and the future of the theme.** The
Zensical team built Zensical because MkDocs has technical limits that ten years
of work exposed.[^12] A project that adopts Material for MkDocs today adopts the
stack its own authors are replacing. The measured Zensical build of one page
took 3.07 seconds from clean, which is not a useful comparison at this size. A
project with several hundred documents would feel the difference, and this
repository holds several hundred Markdown documents under its documentation
directory.[^23]

### 5.2 Sphinx with autodoc and a modern theme

Sphinx 9.1.0 shipped on 31 December 2025 and carries
`Development Status :: 5 - Production/Stable`. It requires Python 3.12 or
later. The Furo theme released 2025.12.19 on 19 December 2025.[^24]

**The property that matters: it is the most mature option, and autodoc imports
the module.** An import is exactly what a PyO3 module needs. Sphinx also has
the strongest cross-reference model of any candidate, and cross-references are
the one mkdocstrings feature that Zensical does not yet provide.[^16]

**The property that fails: it does not read this project's corpus.** This
repository holds its prose in Markdown with Python Markdown footnote syntax.
Sphinx reads reStructuredText natively and reads Markdown through MyST. MyST
footnote syntax is not the same syntax, so the corpus would need conversion.
The project documentation rule fixes the footnote format for every document in
the repository, so the conversion is not optional and it is not small.[^3] A
second cost: Sphinx 9 requires Python 3.12, and this project supports Python
3.11 as its floor.[^25] The documentation job would run on a different
interpreter from the lowest supported one.

**Unverified.** The author did not test whether MyST can be configured to
accept the Python Markdown footnote syntax that this repository uses.

### 5.3 The lightweight option: pdoc

pdoc 16.0.0 shipped on 27 October 2025 and carries
`Development Status :: 5 - Production/Stable`.[^24]

**The property that matters: pdoc imports the package and needs no
configuration.** It produces an API reference from one command. Because it
always imports, it gets the Rust doc comments by construction, and the trap in
section 4.2 cannot occur.

**The property that fails: it documents an API and nothing else.** It has no
navigation over authored Markdown, no site search over a prose corpus, and no
versioning. This repository's documentation is mostly prose: decision records,
research reports, registers and rules. An API reference is the smaller part of
the job. pdoc would need a second tool beside it, which is worse than one tool
that does both.

## 6. What each candidate needs in order to document this module

This section states the answer to the question that a naive comparison misses.

| Candidate | What it needs | Does a stub alone work? |
|---|---|---|
| Zensical with mkdocstrings | The built extension importable in the build environment | No, measured |
| Material for MkDocs with mkdocstrings | The same | No, same handler |
| Sphinx with autodoc | The built extension importable in the build environment | No, autodoc imports |
| pdoc | The built extension importable in the build environment | No, pdoc imports |

**No candidate reads the compiled module without importing it.** A stub exists
in this repository, and it does not close the gap, because it carries no prose
for the methods that matter.[^7]

The consequence for the build is the same in every case. The documentation job
compiles the Rust crate. It is therefore as slow as the wheel job and it needs
the Rust toolchain. A project that wants a fast documentation job must either
cache the build or accept the cost.

## 7. What the repository has today

The author looked, and reports honestly.

There is no documentation build. Three workflows exist: a continuous
integration workflow, a mutation testing workflow and a records workflow. None
of them builds or publishes documentation.[^26] The project manifest declares
no documentation dependency in any dependency group.[^25] No `mkdocs.yml`, no
`zensical.toml` and no Sphinx configuration exists anywhere in the tree.[^23]

The pieces a documentation build would use do exist. The package ships a
`py.typed` marker and a hand-written stub. The type checker runs in strict mode
over the package and the tests. The wheel job builds a wheel and a source
distribution and smoke tests both.[^25] [^26]

**One statement in the tree is false.** The stub's own docstring says that the
continuous integration system checks the stubs, that the build regenerates
them, and that the job fails when the result differs.[^7] No stub generator
exists in the tree. No job regenerates or compares anything. The contributing
guide's only use of the word "stub" describes the Rust crates as
unimplemented.[^27] This work recorded the correction in the findings
register.[^4]

## 8. What the findings register says that bears on this

The author searched the findings register. Three recorded findings bear on this
decision, and they point the same way.

**A list that a check was believed to protect had gone stale, because nothing
read it.** The register carried a table of records holding cost figures. The
work that cleared the records did not clear the table. The safeguard was not
what the register said it was.[^28] The lesson for this decision: a generated
page is safer than a hand-maintained page, because a generated page changes
when the code changes.

**A citation carried a status, and the status went stale.** Five citations
called a record a draft after the registry had accepted it. Nothing failed,
because prose is not compiled.[^29] The lesson: prose that repeats a fact from
another place decays. A docstring copied from Rust into a stub is that shape.

**A completed item's outcome decays like any other document.** An outcome
section was believed to be history, and history does not decay. It does,
because it sits in the tree and reads in the present tense.[^30] The lesson for
this decision: hand-written API prose in a stub is a document, and it decays
against the Rust source that it copies.

**All three support the same rule.** Generate the reference from the one place
that holds the prose. Do not create a second copy that nothing checks.

## 9. Recommendation

**Adopt Zensical as the builder, and bind the configuration format and the
docstring source rather than the tool.**

The reasoning, in order of weight.

1. **The load-bearing question came out in Zensical's favour, and the author
   measured it rather than assuming it.** Zensical builds this project's PyO3
   module reference today, with the Rust doc comments intact and with the
   project's footnote style working. Section 4 gives the evidence. The candidate
   is not disqualified.

2. **The choice is reversible at near zero cost, because the input is
   `mkdocs.yml`.** A project that writes the MkDocs configuration format can
   run Zensical or Material for MkDocs against the same file. The alpha status
   of Zensical therefore buys a small risk, not a large one. This is the single
   reason the recommendation is not "wait for a stable release".

3. **The Rust doc comment is the one source of the prose, and it must stay
   that way.** The stub must not become a second home for it. This is the
   constraint that a decision record should hold, because a future contributor
   could reasonably choose otherwise, because changing it later means rewriting
   every docstring, and because the code does not show the reason.[^1]

4. **The build must import the compiled module.** This is the second
   constraint worth recording. It decides the shape of the documentation job:
   the job compiles the crate. It also forbids the tempting configuration that
   turns inspection off.

5. **Versioned documentation is the weak point, and this project does not need
   it yet.** The package version is `0.0.0`.[^25] A project with no release has
   no versions to publish. When it does, the fork of `mike` exists, and native
   support is on the roadmap.

**What a decision record should state, and what it must not.** It should state
that the published Python reference is generated from the compiled extension
module, that the docstring lives in the Rust doc comment and not in the type
stub, and that the site configuration stays in the MkDocs format so that the
builder stays replaceable. It must not name a version, and it must not name a
release. The record scope rule forbids both in a record body, and this report
is where the version numbers live.[^1]

**What this work also asks for, separately from the decision.** The stub's
false claim about a regeneration check should be repaired or made true. The
findings register holds the case.[^4] A generator would remove the second
declaration site for every signature, which is the shape the recurring defect
rule warns about.[^8]

## 10. When this recommendation is wrong

Each condition below is checkable. If one holds, reopen the choice.

1. **The project decides that the docstring belongs in the stub.** Then the
   build no longer needs to import the module, the documentation job stops
   needing the Rust toolchain, and pdoc drops out as too heavy while Sphinx
   becomes viable on source text alone. The author judges this the wrong trade,
   because it creates a second declaration site for 543 lines of prose with no
   check between the copies.[^6] [^8] A project that adds such a check may
   reasonably choose otherwise.

2. **The reference documentation needs cross-references between API
   identifiers.** Zensical does not render mkdocstrings cross-references yet,
   and the issue is open.[^16] A reference for a large API leans on them. If
   that need is firm before the gap closes, Material for MkDocs is the correct
   builder, and the same `mkdocs.yml` runs it.

3. **The project needs a build hook or a plugin outside Zensical's list.**
   Zensical does not support `hooks`, and it runs only its own rewrites of named
   plugins.[^13] [^14] A project that needs to run its own Python code during a
   build must use MkDocs or Sphinx.

4. **Zensical's alpha status produces a real cost.** The measure is concrete:
   count the upgrades that break the build in the first three months. If a
   Zensical release breaks the build more than once, pin the version and
   reconsider at the next review. The fallback is one command away, because the
   configuration file does not change.

5. **The project adopts reStructuredText or MyST for its prose.** This will not
   happen while the documentation rule fixes the Markdown footnote format for
   every document.[^3] If it does happen, Sphinx becomes the strongest
   candidate, because it is the most mature and it has the best cross-reference
   model.

6. **The project needs versioned documentation before Zensical builds it.**
   The fork of `mike` is a transitional arrangement by its maintainer's own
   words.[^21] A firm versioning need makes Material for MkDocs with the real
   `mike` the safer choice.

7. **The published site turns out to need only an API reference.** Then pdoc is
   the right answer, because it needs no configuration at all. The author judges
   this unlikely, because this repository's documentation is mostly prose.

## References

[^1]: Decision Record Scope, sections 4.1 and 4.2. `.claude/rules/adr-scope.md`
[^2]: Research report 05, the Rust and Python boundary. `docs/research/reports/05-rust-python-boundary.md`
[^3]: Documentation Rules, sections 1 and 3. `.claude/rules/documentation.md`
[^4]: Findings register, FND-320. `docs/FINDINGS.md`
[^5]: The Python control plane package. `python/cachette/`
[^6]: The Python bindings crate. Line and doc comment counts taken with `wc -l` and `grep -cE '^\s*///'` on 3 September 2026. `crates/cachette-py/src/lib.rs`
[^7]: The type stub for the compiled module. Counts taken with `grep` on 3 September 2026. `python/cachette/_core.pyi`
[^8]: Recurring Defect Shapes, shape 1, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^9]: The Zensical repository metadata, read through the GitHub API on 3 September 2026. https://github.com/zensical/zensical
[^10]: The Zensical package metadata on the Python Package Index, read on 3 September 2026. https://pypi.org/pypi/zensical/json
[^11]: Zensical roadmap, read 3 September 2026. https://zensical.org/about/roadmap/
[^12]: Zensical announcement, Material for MkDocs blog, 5 November 2025, read 3 September 2026. https://squidfunk.github.io/mkdocs-material/blog/2025/11/05/zensical/
[^13]: Zensical documentation, migrating from MkDocs, read 3 September 2026. https://zensical.org/docs/compatibility/mkdocs/migration/
[^14]: Zensical documentation, MkDocs plugin support, read 3 September 2026. https://zensical.org/docs/compatibility/mkdocs/plugins/
[^15]: Zensical documentation, getting started, read 3 September 2026. https://zensical.org/docs/get-started/
[^16]: Zensical issue 237, mkdocstrings cross-reference support, read 3 September 2026. https://github.com/zensical/zensical/issues/237
[^17]: Griffe user guide, extending and the treatment of compiled modules, read 3 September 2026. https://mkdocstrings.github.io/griffe/guide/users/extending/
[^18]: mkdocstrings-python configuration, general options, read 3 September 2026. https://mkdocstrings.github.io/python/usage/configuration/general/
[^19]: Zensical documentation, footnotes, read 3 September 2026. https://zensical.org/docs/authoring/footnotes/
[^20]: Zensical documentation, Python Markdown Extensions support, read 3 September 2026. https://zensical.org/docs/compatibility/markdown/python-markdown-extensions/
[^21]: Zensical documentation, mike compatibility, read 3 September 2026. https://zensical.org/docs/compatibility/mkdocs/mike/
[^22]: Zensical documentation, publish your site, read 3 September 2026. https://zensical.org/docs/publish-your-site/
[^23]: The author's probe build, run on 3 September 2026 on x86-64 Linux with CPython 3.13.5. The commands, the three configurations and the byte counts are in the commit body of the change that adds this report.
[^24]: Package metadata for `mkdocs`, `mkdocs-material`, `mkdocstrings`, `mkdocstrings-python`, `griffe`, `sphinx`, `furo`, `pdoc` and `mike` on the Python Package Index, read on 3 September 2026. https://pypi.org/
[^25]: The project manifest. `pyproject.toml`
[^26]: The continuous integration workflows. `.github/workflows/`
[^27]: Contributing guide, the opening section. `CONTRIBUTING.md`
[^28]: Findings register, FND-242. `docs/FINDINGS.md`
[^29]: Findings register, FND-055. `docs/FINDINGS.md`
[^30]: Findings register, FND-059. `docs/FINDINGS.md`
