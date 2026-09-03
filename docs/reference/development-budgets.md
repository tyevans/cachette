# Development Budgets

This document is a **register**. It holds one kind of figure: how long the gate
suite takes on a development machine.

**Nothing in this document is evidence about the target platform.** The engine
targets AWS Graviton, and the primary target triple is
`aarch64-unknown-linux-gnu`. Development happens on x86-64 and on Apple
Silicon, and both are development targets only. Apple Silicon uses a 128-byte
cache line and Graviton uses a 64-byte cache line, so a local measurement
misleads on false sharing and on alignment.[^1]

A separate register holds every figure that belongs to the target.[^2] The two
registers are separate files so that a reader cannot take a row from one as
support for the other.

## Why this register exists

The project owner decided that the project keeps two performance paths with
different standing.[^3]

- **The target platform owns every claim about how the engine performs.** A
  third register holds every figure measured there.[^11] One blocker states
  which figures are still derived, and that blocker stays open.[^4]
- **The development machine owns one local budget: how long the gate suite
  takes.** A contributor runs the suite many times a day, and the cost is paid
  on that machine and nowhere else.

Before this decision, no rule owned the second cost, so it grew without
anything noticing.

## A development machine is not one machine

A budget row names the machine that produced it, or the row means nothing. The
project develops on two architectures that do not perform alike. The project
owner reports that the engine runs much faster on Apple Silicon.

Apple Silicon is the closer of the two to the target in one way: it is arm64,
so it exercises the same code generation. It is the further away in another:
the cache line size differs. Neither fact makes an Apple Silicon figure
evidence about the target.

## The gate suite budget

The gate suite is the command that a commit must pass. It runs formatting,
lint, tests and the record checks.[^5]

| Figure | Value | Machine | Architecture | Profile | Conditions | Date |
|---|---|---|---|---|---|---|
| Whole gate suite, wall clock | 153 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, opt-level 1 | An isolated run. Nothing to rebuild | 1 September 2026 |
| Whole gate suite, budget | 190 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, opt-level 1 | An isolated run. Nothing to rebuild | 1 September 2026 |
| Golden state hash test, wall clock | 16 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, opt-level 1 | An isolated run. Nothing to rebuild | 1 September 2026 |
| Whole gate suite, wall clock | 435 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, no optimisation | An isolated run. Nothing to rebuild. The profile the project used before | 1 September 2026 |
| Whole gate suite, wall clock | 441 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, opt-level 1 | An isolated run. An empty target directory | 1 September 2026 |
| Whole gate suite, wall clock | 746 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, no optimisation | An isolated run. An empty target directory. The profile the project used before | 1 September 2026 |
| Whole gate suite, wall clock | 1563 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | dev, opt-level 1, `nightly-2026-09-01` | A passing run. One other worker building throughout. Load average 7.35, falling from 21. 56 test binaries | 3 September 2026 |

The budget is the measured figure plus an allowance of one fifth, rounded up
to the nearest ten seconds. A run over the budget is a signal to look, not a
failure.

**The last row is not comparable to any row above it, and the budget row no
longer describes the suite.** Three things changed between them, and each alone
breaks the comparison. The Rust test binaries went from 44 to 56. The toolchain
moved from a pinned stable release to a nightly one, so the two figures come
from different compilers. The machine carried another worker throughout, and
the load average was 7.35. The row is here because it is the only real total
anyone has taken since the suite grew, not because it measures the same thing.

**Do not adjust the budget row to cover it.** The register's own rule forbids
raising a budget to cover a growth, and that rule holds. The budget row
describes a tree that no longer exists, so it needs replacing by a figure
taken under stated conditions rather than adjusting. Nobody has taken one. A
harness that splits the cost by recipe exists and waits for a quiet machine and
a still tree.[^12]

**The budget describes a run with nothing to rebuild, and only such a run.**
That is the run a contributor makes many times a day. A run that starts from
an empty target directory compiles the whole workspace first, and it goes over
this budget every time. The table holds a row for that run so that a reader
who sees the report can tell the two apart. Do not read a cold run as a gate
that grew, and do not raise the budget to cover one.

Compilation is now the larger half of a cold run, and optimisation is why. The
two cold rows and the two warm rows above give both halves for both profiles,
so a reader can see what the optimisation level bought and what it cost.

**A row states the conditions it holds under.** The cost of this suite depends
on what else runs on the machine. The suite compiles and tests in parallel, so
two suites on one machine do not cost twice one suite. A run of this suite was
killed after about forty minutes with four other runs present, on the machine
in the rows above.[^8] That figure is contended and reported, not measured
here, and it is in this paragraph rather than in the table for that reason.

**The rows above are one run each, on a machine with no other work on it.**
The last row holds the profile the project used before the optimisation level
was raised, and it is there so that a reader sees the step the register
recorded.[^9] A row that carries no profile carries no meaning, because the
profile is the largest thing that moves this figure.

**A row is a snapshot, and it does not support a comparison against a row
taken hours earlier.** This machine slows down under sustained load. The same
suite, unchanged, measured about four and a half minutes early in a session
and about seven minutes after two hours of continuous running, on an idle
machine that reported a low temperature throughout. The effect is large enough
to invert a result.[^10] To compare two versions of the code, alternate them
back to back and report the pair. Do not compare a run today against a row
from last week.

**Every row above describes one development machine.** No row is evidence
about the target platform, and the comparison a row supports is between two
versions of this code on that machine.[^6]

**A row belongs to one architecture.** The project develops on x86-64 and on
Apple Silicon, and the two do not perform alike. The suite compares a run
against a row of the same architecture, and reports without a comparison when
no such row exists. Add a row for a machine rather than reading another
machine's row.

### How the suite reports the cost

The gate suite times itself and prints the cost against the row for the
architecture that runs it.[^5] The script reads the value from the table
above. The figure has one home, so a change to the budget is a change to this
file and to nothing else.

### How to find the gate that grew

A whole-suite figure says that a gate grew. It does not say which one. A
second harness times each recipe of the suite separately and reports each
against the whole.[^12] It reads the recipes from the gate definitions rather
than restating them, so it cannot disagree with the suite. It reads the budget
from the table above through the same reader the cost report uses.

Run the harness on a machine with no other work on it. It prints the load
average before and after the run, and it counts the crate builds it saw, so a
reader can throw away a contended figure and can tell a cold run from a warm
one.

The report never fails the build. A wall clock figure on a loaded machine is
not a gate, and a timing assertion trains a reader to ignore a red
pipeline.[^7]

### The command that produces the figures

Run the whole suite and the single test on the machine you name in the row.

```
just check
cargo test --package cachette-core --test golden_state_hash
./scripts/gate-times.sh
```

State the machine, the architecture, the build profile and the date beside any
value you record. A figure without those four facts is not usable.

### Keeping a row true

Record a new value when the suite changes cost on purpose, and say in the
commit what changed. Do not edit a row to make a slow run pass. A row that
follows the suite records nothing.

## What belongs here

- The wall clock cost of a gate, on a named development machine.
- A budget for such a cost, and the allowance that goes with it.
- The command that produced a figure.

## What does not belong here

- Any figure about the target platform. Two registers hold those, one for a
  derivation and one for a measurement.[^2] [^11]
- A per-tick or per-frame simulation cost, on any machine. The engine's
  performance belongs to the target.
- A decision. A budget is an input to a decision, not a decision.

## Format for a row

Give the name, the value, the machine, the architecture, the build profile, the
command, and the date. Cite the source in a footnote.

## References

[^1]: Project orientation, the target platform. `CLAUDE.md`
[^2]: Budgets and costs, the target register. `docs/reference/budgets.md`
[^3]: Decisions register, DEC-033, option 2. `docs/DECISIONS.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: The gate suite. `justfile`
[^6]: ADR-0008, the primary target is `aarch64-unknown-linux-gnu`, decision D2. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^7]: Testing rules, section 3. `.claude/rules/testing.md`
[^8]: Findings register, FND-099. `docs/FINDINGS.md`
[^9]: ADR-0083, the gate build checks every integer overflow. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
[^10]: Findings register, FND-142. `docs/FINDINGS.md`
[^11]: Target platform costs, the measurement register. `docs/reference/graviton-costs.md`
[^12]: The per-recipe timing harness. `scripts/gate-times.sh`
