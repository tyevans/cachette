---
id: 0314
title: Write the explanation pages and link out rather than copy
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**This repository already holds its own explanation, and copying it into a
documentation site would create a second declaration site.** Decision records
state why each constraint exists. Research reports hold the evidence. Registers
hold what the project believed and corrected. One fact stored in two places,
with nothing that fails when the copies disagree, is the defect shape this
project names first in its own rule.[^1]

**The product record refuses the copy for its own reasons.** It states that
documenting the decision records and the research reports is not part of the
need, because both bodies of work have their own homes and their own rules, and
pulling them in would give the need no bound.[^2]

**Three statements have no other home, and each is a translation rather than a
copy.** A record states a constraint for a contributor to this project. None of
the three below is stated anywhere for the reader who will break it.

## What the work does

Write three short pages. Each states a conclusion in the reader's terms and
cites its source in a footnote, which is what the documentation rule asks of any
document in this tree.[^3]

1. **The rule that governs every program on this engine.** Python builds a
   selector and sends one command, and Python does not walk a population. The
   product record requires the documentation to state this rule and to state
   plainly whether anything enforces it.[^2] Nothing does. The package docstring
   already says so, and a record says the same.[^4] [^5]
2. **What determinism gives the reader, and what it does not.** Two runs agree.
   That is not a statement that either run is right. A defect that is itself
   deterministic passes both determinism tests, and this project has recorded an
   instance.[^6] A reader who builds an experiment on the guarantee needs both
   halves.
3. **Which surface a program may depend on, and what is missing.** The package
   holds a compiled module, a demonstration window and a tool server that serves
   this repository. The product record requires the reader to tell them apart,
   and to learn what the package cannot do yet rather than conclude that they
   failed to find it.[^2] Two findings show what happens when nobody states the
   line: the orientation document claims a pyramid level that nothing writes,
   and it presents the tool server as a capability of the product.[^7] [^8]

Every other explanation is a footnote that names a record. No page restates a
record, and no page holds a figure, a count or a version.

## What it does not do

It does not explain the Rust core. A person who changes the engine has the
source in front of them, and the product record excludes that audience.[^2]

It does not answer whether an upgrade changes hands when the ground does. That
question is open and the page cites the blocker rather than answering.[^9]

It states no performance figure. A blocker governs every figure in this
project.[^10]

## Why this is not refined

**How a published page cites a repository document is an open choice, and this
item is the one that cannot proceed without it.** These pages are mostly
footnotes that name records under `docs/`, and a reader who reaches the site
without cloning cannot open one. The decisions register holds the options and a
recommendation.[^11] Refining this item waits on that choice.

The reading of the three earlier quadrants also decides how much of these pages
is needed at all. That is why this item sits last in the plan.[^12]

## References

[^1]: Recurring Defect Shapes, shape 1, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^2]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^3]: Documentation Rules, sections 2 and 3. `.claude/rules/documentation.md`
[^4]: The Python control plane package. `python/cachette/__init__.py`
[^5]: ADR-0043, a declared tier enforces the no-loop rule, decision D5. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^6]: Testing Rules, section 2. `.claude/rules/testing.md`
[^7]: Findings register, FND-322. `docs/FINDINGS.md`
[^8]: Findings register, FND-323. `docs/FINDINGS.md`
[^9]: Blockers register, BLK-034. `docs/BLOCKERS.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: Decisions register, DEC-114. `docs/DECISIONS.md`
[^12]: Backlog item 0308, the documentation plan. `docs/backlog/refined/0308-the-documentation-plan.md`
