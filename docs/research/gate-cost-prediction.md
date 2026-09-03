# A Prediction: Which Gate Grew

This document is a **pre-registered prediction**. It states, before anyone
times a gate recipe, where the cost of the gate suite now sits. A prediction
written after a measurement is a consistency check, and this project keeps the
two apart.

**Nothing here is a measurement of a gate recipe.** The author timed no
recipe. The figures below come from two sources: the git history of the gate
definitions, and five gate run logs that the project owner produced. Every
figure names the source that holds it.

## 1. What the project believes today

The local register holds one budget for the gate suite: 190 seconds on an
Intel Core i7-1260P with 16 hardware threads, on x86-64, in the development
profile at optimisation level 1.[^1] The budget is a measured 153 seconds plus
an allowance of one fifth. The register states that the budget describes a run
with nothing to rebuild, and it holds separate rows for a run that starts from
an empty target directory.

The register also states the rule this work follows: find the gate that grew,
and do not raise the budget to cover one.[^1]

## 2. What five gate runs reported

The project owner ran the suite five times on 2 September 2026, on the machine
the budget names.[^2] Each run reported its own cost.

| Run | Reported cost | Crate builds in the log |
|---|---|---|
| 1 | 708 s | 0 |
| 2 | 720 s | 0 |
| 3 | 573 s | 1 |
| 4 | 539 s | 0 |
| 5 | 570 s | 1 |

Three of the five runs compiled nothing at all. Those runs meet the condition
the budget was written for, so the cold-run row of the register does not
explain them.

The tools inside the suite report their own time, and the logs hold those
figures. Two figures matter.

| Run | Rust test binaries, summed | The Python test run |
|---|---|---|
| 1 | 220.7 s | 224.8 s |
| 2 | 214.9 s | 225.0 s |
| 3 | 218.8 s | 174.2 s |
| 4 | 205.0 s | 162.0 s |
| 5 | 205.3 s | 163.4 s |

Each figure is the tool's own report about itself. The Rust column sums the
figure that each test binary prints. The Python column is the figure that the
Python test runner prints. Neither includes the cost of starting a process,
checking a fingerprint, or linking.

**Either column alone is larger than the whole budget.** The two together
account for between 367 and 450 seconds of a run.

## 3. What changed since the budget was measured

The budget was measured on 1 September 2026. The git history since that day
shows what the gate suite gained.

- **The Rust tests grew by about 20,000 lines.** Twenty test files are new.
  Six of the largest existing files grew.
- **The Python tests grew from one module to four.** Three modules are new:
  the public interface tests, the site position tests, and the tests for the
  agent-facing server. The last is the largest, at about 700 lines and 22
  tests.
- **The gate suite gained four recipes or checks.** A census recipe. A merge
  conflict marker check. A footnote check. A merge defect check.
- **The perturbation gate gained two binaries.** One for the exit field and
  one for the influence field.

The three new Python modules matter more than their line count suggests. Each
of the 22 tests in the agent server module starts the server as a new
subprocess and initialises a session, then throws it away. The fixture is not
shared between tests.

## 4. The prediction

The figures below predict a run on a quiet machine, with nothing to rebuild,
on the machine the budget names. The author predicts a total between 430 and
560 seconds, and the point below sums to 518 seconds.

| Recipe | Predicted cost | Predicted share |
|---|---|---|
| fmt-check | 10 s | 2% |
| lint-rust | 8 s | 2% |
| lint-python | 10 s | 2% |
| invariants | 4 s | 1% |
| test-rust | 215 s | 42% |
| census | 3 s | 1% |
| probe | 40 s | 8% |
| test-python | 170 s | 33% |
| smoke | 10 s | 2% |
| records | 15 s | 3% |
| records-probe | 8 s | 2% |
| merge-defects | 25 s | 5% |
| Total | 518 s | 100% |

**The headline claim: two recipes hold the growth, and neither is new.** The
Rust tests and the Python tests together should hold above 70 percent of the
run. Every gate that landed after the budget was measured should hold under 60
seconds in total, which is under a sixth of the gap.

**The second claim: the Python tests are the larger growth.** On 1 September
the Python side ran one module. The whole suite cost 153 seconds then, so the
Python side cost well under that. It now costs between 162 and 225 seconds by
its own report. The author predicts the growth there is about 145 seconds.

**The third claim: the agent server module is the largest part of the Python
cost.** It should hold above half of the Python test time, because it starts
22 subprocesses that the other modules do not start.

**The fourth claim: the five logged runs were contended.** The Rust test time
varies by 8 percent across the five runs, while the reported total varies by
34 percent. Work that does not change cannot vary that much on its own. A run
on a quiet machine should land below the lowest of the five.

## 5. What would refute this

Each statement below refutes the prediction. The author wrote them before the
measurement, so that the result cannot be read to fit.

1. The Rust tests and the Python tests together hold under 60 percent of the
   run.
2. The Python test recipe costs under 120 seconds.
3. The merge defect check costs above 100 seconds.
4. The record checks, the record probe and the merge defect check together
   cost above 120 seconds.
5. The total on a quiet machine is above 600 seconds.
6. Any recipe this document predicts under 20 seconds costs above 60 seconds.

One condition voids the comparison rather than refuting it. If the run
compiles anything, it is a cold run, and the register holds separate rows for
that case.[^1] The timing harness reports how many crate builds it saw, so a
reader can tell the two apart.[^3]

## 6. How to test this

One harness times each recipe separately and reports each against the
whole.[^3] It reads the recipes from the gate definitions rather than
restating them, so it cannot disagree with the suite. It prints the load
average before and after the run, because a figure taken on a loaded machine
is not evidence.

The harness must run on a machine with no other work on it. The author did not
run it, and did not time any gate recipe, for that reason.

## References

[^1]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^2]: Five gate run logs from 2 September 2026, on an Intel Core i7-1260P, x86-64. `/tmp/g26f.log`, `/tmp/g27.log`, `/tmp/g29.log`, `/tmp/g29b.log`, `/tmp/g22.log`
[^3]: The per-recipe timing harness. `scripts/gate-times.sh`
