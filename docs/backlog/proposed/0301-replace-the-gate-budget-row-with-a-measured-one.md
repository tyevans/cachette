---
id: 0301
title: Replace the gate budget row with a measured one
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The gate suite budget describes a tree that no longer exists.** The register
holds 190 seconds for a development machine it names, derived from a measured
153 seconds on 1 September 2026.[^1] The suite has since passed at 1563
seconds on that machine. Three things changed between the two figures, and
each alone breaks the comparison: the Rust test binaries went from 44 to 56,
the toolchain moved from a pinned stable release to a nightly one, and the
1563 second run carried another worker on the machine.

**A row that no longer describes the suite is worse than no row.** The cost
report prints a comparison against it on every run, so every contributor reads
a figure that means nothing, many times a day. The report is what a
contributor sees, and it is now telling them something false with the
authority of a register.

**The register forbids the easy repair.** It says not to raise a budget to
cover a growth, and that rule holds.[^1] Replacing a row whose subject is gone
is a different act from adjusting one to make a slow run pass, but the two look
alike in a diff, so the commit must say which it is.

**The instrument exists and the measurement does not.** A harness splits the
run by recipe and reports each against the whole.[^2] It reads the recipes from
the gate definitions rather than restating them, names every command line that
failed, and reports when the commit moved under the run. Nothing has run it on
a quiet machine against a still tree. One attempt was discarded because three
branches merged into the working tree while it ran.

## What is missing before this is refined

- The impact review.
- **The measurement.** It needs a machine with no other work on it and a tree
  that nobody merges into for the duration. The second condition is the one
  that has been hard to get, and the harness now reports when it was not met.
- **Which recipe holds the cost.** Five gate logs from 2 September 2026 show
  the Rust test binaries self-reporting 205 to 221 seconds and the Python test
  runner self-reporting 162 to 225 seconds, on a machine under load.[^3] Either
  figure alone exceeds the whole budget. A per-recipe table would say whether
  that still holds, and the ratio between those two recipes is the part that
  survives a changed compiler, because both are measured inside one run.
- **Whether one budget row is still the right shape.** A suite that costs a
  quarter of an hour may need a fast gate and a slow gate rather than one
  number. That is a decision and it needs a record if it is taken.
- **What the row says about the toolchain.** No row in the register carries a
  toolchain today, and two of them now come from different compilers. The
  format may need a column.

## Done means

- The register holds a gate suite figure and a budget taken on a named
  machine, with the architecture, the build profile, the toolchain and the
  date beside them.
- The commit says what changed the cost, and says plainly that the row was
  replaced rather than adjusted.
- The harness reported no failed command line and no commit move for the run
  the row comes from.
- A reader can tell which recipe holds the cost without running anything.

## References

[^1]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^2]: The per-recipe timing harness. `scripts/gate-times.sh`
[^3]: Five gate run logs from 2 September 2026, on an Intel Core i7-1260P, x86-64. `/tmp/g26f.log`, `/tmp/g27.log`, `/tmp/g29.log`, `/tmp/g29b.log`, `/tmp/g22.log`
