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

- **The target platform owns every claim about how the engine performs.** One
  open blocker states that no measurement exists there, and that blocker stays
  open.[^4]
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

| Figure | Value | Machine | Architecture | Profile | Date |
|---|---|---|---|---|---|
| Whole gate suite, wall clock | 544 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | debug | 31 August 2026 |
| Whole gate suite, budget | 660 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | debug | 31 August 2026 |
| Golden state hash test, wall clock | 36 s | Intel Core i7-1260P, 16 hardware threads | x86_64 | debug | 31 August 2026 |

The budget is the measured figure plus an allowance of one fifth, rounded up
to the nearest ten seconds. The allowance covers the load of the machine that
runs the suite. A run over the budget is a signal to look, not a failure.

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

The report never fails the build. A wall clock figure on a loaded machine is
not a gate, and a timing assertion trains a reader to ignore a red
pipeline.[^7]

### The command that produces the figures

Run the whole suite and the single test on the machine you name in the row.

```
just check
cargo test --package cachette-core --test golden_state_hash
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

- Any figure about the target platform. That register is separate.[^2]
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
