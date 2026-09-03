# Working Report — research report 20, the Python interface

This document reports the work of one research task. It is a record of one
moment, and nobody maintains it.

Date of the work: 3 September 2026.

Cachette is a world simulation engine. The core is Rust. The control plane is
Python. The task asked what a Python developer with an idea should be able to do
easily, and what the interface should look like. It asked for design research
and forbade implementation.

## 1. What the work produced

**One research report.**[^1] It ranks five jobs, writes the current code and the
proposed code for each, answers the five shape questions, answers the question
about Pydantic, argues for two tiers, states the cost at the target scale, and
names the product record and the seven decision records that should follow.

**Three findings.**[^2] The constructor's prose is written in Rust and reaches
no Python object. The engine reports neither the seed nor the faction count, and
the agent server already carries a second declaration site because of it. The
range check on a kind refuses two of the five wrong values and accepts three.

**Five open decisions.**[^3] Whether the package ships a friendly tier and where
the line falls. Whether a bare float reaches a fixed-point argument. Where the
prose of a pure Python member lives. Whether a kind enumeration is generated or
written. Whether the compiled module gets a public name.

**One blocker.**[^4] No Python developer outside this repository has used the
interface, so the ranking of the jobs is inferred from two graded reads and from
the code inside the repository.

**No code changed.** The task forbade it. Nothing under `crates/` or `python/`
was touched.

## 2. What was measured, and against what

Every measurement ran on an x86-64 development machine against the installed
extension module. The target platform is a 64-bit Arm server, so no figure here
is a target platform figure.

**The installed module is older than the source tree.** Its `World` class
provides no `faction_population` and no `panel_names`, and the source of this
worktree declares both. The author found this by comparing the members of the
imported class against the type stub. Every measurement in the report is
therefore a measurement of that older build.

**The author did not rebuild the extension.** A rebuild installs into a shared
environment that other work in this session uses, and the task did not need one.
The report states the provenance instead.

## 3. What was left undone

**The extension was not rebuilt, so the measurements are of an older build.**
The three measurements the report leans on are the cost of the singular read
loop, the acceptance of an overlapping kind number, and the absence of the
constructor's prose. None of them touches a member that changed, but the author
did not confirm that by reading the difference.

**The published reference page was not fetched.** The task supplied two graded
reads of it, and the report treats them as reports of one moment. The author did
not confirm what the page shows today, and did not confirm whether the class
signature reaches the page even though it reaches Python.

**Two claims about the Pydantic core are marked unverified in the report.** The
author did not read the source of that crate and did not confirm whether it is
published for a Rust consumer. The report's argument does not rest on either
claim, because the decisive argument is that this project's validation is a
question about live world state.

**No product record was written.** The task forbade it and asked for a
recommendation instead. The report names the title, the audience and the six
gate answers. Nobody has allocated the number.

**No decision record was written, and no registry row was added.** The report
names seven records by claim. The registry allocates each number, and the author
did not reserve any, because the task allocated no record numbers.

**No backlog item was written.** The report's section 7.3 ranks the work in four
stages. Nobody has turned any stage into an item, so no priority index changed.

**The selector design was not tested against the Rust side.** The report writes
the Python that a caller types and argues it against the accepted selector
record. It does not say how the tree is represented when it crosses, and it does
not say what the engine does with a node it cannot prune. Both belong in the
decision record that follows.

**The read side names no field vocabulary.** The report shows
`read("unit", "tile", "faction")` and does not say which fields a unit selector
offers, or how that list stays in step with the Rust columns. That is the same
redundant declaration question the report answers for the kind enumeration, and
it needs the same answer.

**Persistence was ranked and not designed.** The report names the job, ranks it
last, and says the friendly tier is the wrong place for it.

## 4. The gates

The record check command was run and every check passed. The record check
reported two notes about records that nothing cites, and neither is a record this
work touched. The conflict marker check reported one note about a fixture it does
not read. The footnote check reported no failure. The commit body holds the
output.

Three failures appeared on the first run and the work repaired all three. Each
register stated a next number below the row this work added, so each line was
raised. The footnote check then found three labels that gave one source a second
name, so each reuses the label the register already had.

Two register files each held the next number twice, from an earlier merge. The
two copies stated different numbers. This work made them agree and then removed
the second copy, because one value in two places is the defect shape this project
records.[^5]

No test was run, because no code changed.

## References

[^1]: Research report 20, what the Python interface should be. `docs/research/reports/20-the-python-interface.md`
[^2]: Findings register, FND-350, FND-351 and FND-352. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-130 to DEC-134. `docs/DECISIONS.md`
[^4]: Blockers register, BLK-045. `docs/BLOCKERS.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
