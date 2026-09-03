# Review — PRD-0021, the control plane has no published documentation

This document reports the work that wrote one product requirement record. It
is a record of one moment, and it is not maintained.

## What was written

One product requirement record, numbered 0021 and titled "A developer can use
the control plane without reading its source".[^1] It moved from `shaped/` to
`accepted/` in a second commit, on an authorisation the project owner gave in
the dispatching session on 3 September 2026.

The registry row was added with status `Idea` and no file before the file was
written, as the product rule requires.[^2] It then took `Shaped` with the
path, and `Accepted` with the new path after the move.

The record was placed at the top of the `Later` group in the product priority
index, above 0018.[^3]

One finding was added to the findings register, numbered FND-319.[^4]

## The audience, and why

**A developer who builds a strategy game on this engine, and who has never
read the source of this project.**

Python is the only part of the engine that person writes, so the package is
the whole product as far as that person can see. The other two audiences of
the product directory, a modeller and a researcher, reach the engine through
the same package. Both gain from the answer. Neither states the need as
sharply, because each arrives with one narrow question. Only the game
developer must learn the whole interface before doing anything.

A fourth audience exists: an agent that works on this repository. Product
record 0019 holds it.[^5] That record asks the running engine to answer a
question. This one asks the package to explain itself before anything runs.
The record says plainly that neither answer replaces the other.

## The six gate answers in brief

**Who this is for.** The game developer who has not read the source.

**What the person cannot do today.** That person cannot learn the control
plane without reading its source. The package explains itself in a docstring
that a reader gets after installing, in one worked example in the orientation
document that nothing executes, and in Rust source comments on the compiled
module. Three costs follow: the developer cannot find the boundary rule that
shapes every program, cannot tell what exists from what is planned, and cannot
answer a question without opening the source.

**What good looks like.** Nine checkable statements. A reader who has never
seen the package builds a world and runs one tick from the documentation
alone. That reader installs the package from the documentation alone. That
reader reaches the documentation without cloning the repository. Every example
a reader can copy is executed by something that fails when the example stops
working. Every public name appears in the documentation, and something derives
the list of public names from the package. Every described call states what it
returns and which error it raises. The documentation states the rule that
Python sends one command over a set and does not walk a population, and states
whether anything enforces it. The documentation separates the surface a
program may depend on from the surface that serves this repository. The
documentation states what the package cannot do yet.

**What this does not do.** It does not document the Rust core, the decision
records, the research reports, or the tool surface that serves repository
agents. It does not choose how the documentation is made or where a reader
finds it. It does not promise that the interface stops changing. It does not
teach a person to design a strategy game. It does not replace a test.

**What it costs at the target scale.** The target scale of this record is not
the world. A document costs the same for a large world as for a small one. The
cost is maintenance, because every sentence is a copy of a fact whose original
lives in the code, and prose does not fail when the copy goes stale. The record
cites the project's own two measurements of that cost.[^6] [^7] Three
statements bound it: prefer a statement that something executes, because it
fails on the day it becomes false; a statement nothing executes costs the
reader's trust for the whole document the first time it is wrong; and the cost
follows the size of the interface, never the size of the world. The record
states no number.

**Which blockers govern this.** Two.[^8] [^9] The first governs every cost
figure in the project, and the record states none. The second asks whether an
upgrade changes hands when the ground does, and the record says the
documentation must not answer while the question is open.

## Gates run, and the real result

    ./scripts/check-prds.sh        21 product records, 0 failures
    ./scripts/check-priority.sh    151 priority rows, 0 failures
    just records                   0 failures across six checks, 2 pre-existing notes
    just records-probe             0 failures
    just merge-defects             0 failures

The two notes from the record check name two decision records that nothing
cites. Both predate this work.

**The Rust gates were not run.** This change touches no Rust and no Python
source, so formatting, lint, the test suite and the two determinism tests were
not exercised. That is a skipped step and not a green claim.

One command was run that is not a gate. The worked example in the orientation
document was run unchanged against the installed package. It printed a tick
and a gather count and raised nothing. That is the evidence for the record's
statement that the example runs today, and for the finding below.

## What was left undone

**The false docstring was not repaired.** The top-level docstring of the
Python package states that the selector interface, the verb interface and the
view scope are not written yet. The selector interface is not written. The
other two are: the compiled module carries set-valued verbs and columnar
reads, and the example calls both. FND-319 records the correction.[^4] The
sentence was left in place on purpose. The sentence and the documentation of
this package are one statement, and correcting the sentence alone would create
a second place that says what the package holds. That is defect shape 1.[^10]
**The worker who chooses how this package is documented owns both.**

**No decisions register row was added.** The audience choice is a judgement,
and a register row is the usual home for a judgement. The record itself states
the choice and justifies it in its first section. A row would be a second
declaration site for one fact, with nothing failing when the two disagree. The
choice therefore lives in the record alone.

**No blocker was opened or closed.**

**The findings next-number line was raised past the number this work took.**
The dispatcher reserved a range for this work above what the line held, and
the line answers from merged history, so it could not see the reservation. One
number below the range may go unused. A gap is safe, because a number is never
reused.

## What the record could not say, for the next worker

The next worker writes the technology decision record. Everything below was
considered and left out, because a product record states a need and never a
structure. None of it is a recommendation. Each is a question the decision
record must answer.

- **How the documentation is produced.** No generator, no framework and no
  build step is named. The record only states that something must derive the
  list of public names from the package, and that something must execute every
  example. Whether one mechanism does both is open.
- **Where a reader finds it.** No host and no address. The record states only
  that a reader reaches the documentation without cloning the repository.
- **What shape it takes.** No file format, no page layout and no directory
  arrangement. The record does not say whether the answer is one page or many,
  a tutorial or a reference.
- **Where the text lives.** The record does not say whether the documentation
  is written beside the code, in a separate tree, or generated from the Rust
  source comments on the compiled module. This is the sharpest open question,
  because the compiled module's documentation is written in Rust today and the
  reader who needs it writes Python.
- **How the two languages meet.** A method of the compiled module carries its
  documentation in a Rust file. A pure Python module carries a docstring.
  Nothing joins them. The record states the reader's need and not the join.
- **What enforces the example.** The record requires that something executes
  every published example and fails. It does not say what.
- **Which surface is public.** The record requires that the documentation
  separates the surface a program may depend on from the surface that serves
  this repository. It does not draw the line. Drawing it is a decision, and the
  window and the tool server both sit near it.
- **Whether the interface is stable.** The record explicitly declines to
  promise it. A decision record may make that promise, and the documentation
  changes shape if it does.

## Where this work disagreed with its instructions

**The instruction named the tool server that serves repository agents as
material a reader would need documented. The record bounds it out.** The
record names one audience, and the tool server serves a different one, which
product record 0019 already holds.[^5] Documenting it under this need would
give the need no bound, which is the failure the fourth gate question exists
to prevent. The package was read, and the reading informed the statement that
the package holds surfaces at different stages that nothing distinguishes.

**The instruction reserved two numbers in the decisions register. Neither was
used.** The reason is above: a register row for the audience choice would be a
second declaration site.

**The instruction reserved two numbers in the blockers register. Neither was
used.** This work found no missing information. Every open question it met is
a choice, and the next worker's decision record is where a choice is made.

## References

[^1]: Product requirement record 0021. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^2]: Product requirement records, naming and numbering. `docs/product/README.md`
[^3]: Product priority index. `docs/product/PRIORITY.md`
[^4]: Findings register, FND-319. `docs/FINDINGS.md`
[^5]: Product requirement record 0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^6]: Findings register, FND-223. `docs/FINDINGS.md`
[^7]: Findings register, FND-242. `docs/FINDINGS.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: Blockers register, BLK-034. `docs/BLOCKERS.md`
[^10]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
