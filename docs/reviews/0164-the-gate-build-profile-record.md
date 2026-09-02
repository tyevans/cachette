# Review 0164: The gate build profile record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md` | Status `Draft` at review, and `Draft` after it |
| Commit | `1cf903f`, the head of `main` at the time of the review |
| Code read | the workspace manifest, the gate suite recipes, the overflow test, the fixed-point module, and the pyramid module |

The reviewer did not write ADR-0083 and did not write the change it accompanies.
The reviewer read the record, the rules, and the tree. The reviewer did not read
the reasoning of the author, who was not available.

**The reviewer compiled nothing.** Another worker held the machine, so no
`cargo` command ran. Section 6 names every claim that only a run can settle.
Each of those is marked unverified and is not counted toward the verdict.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0083 | Reject for now. The record stays in `draft/`. Section 3 lists what must change. |

The constraint is sound, the title states a claim, the body holds no volatile
material, and both decisions have code behind them. Three sentences in the
record are false against the code or against the record's own reasoning, and
one of the three contradicts the manifest that the record exists to protect. A
record the code contradicts is worse than no record, because it lies.[^1]

Each of the three is a sentence, not a decision. The record needs an edit, not
a rewrite. The reviewer did not make the edit, because writing the record into
acceptability is authoring and not reviewing.

## 1. The record against the code, decision by decision

**D1 holds.** The workspace manifest declares `[profile.dev]` with
`opt-level = 1`, `overflow-checks = true` and `debug-assertions = true`. The
test profile inherits from the development profile, and the gate suite runs
`cargo test --workspace`, so the build that the gate tests is the build D1
constrains. A comment above the profile names the record and gives its reason.

D1 states no optimisation level, which is what the scope rule asks of a value
that a measurement can change.[^2] The record says so in its own text and cites
the register that owns the figure.

**D2 holds in its substance and fails in two sentences.** The test file exists
and holds three tests. One adds one to the ceiling of a `u32` and asserts the
panic. One sums a two-byte field over the target tile count into a `u32`
accumulator and asserts the panic. One sums the same field into an `i64` and
asserts the exact total. Both operands of every overflow pass through
`black_box`, so the compiler cannot fold the sum and refuse the build instead.
The panic hook is replaced for the call, so a passing run prints no backtrace,
which is the consequence the record states.

The two sentences that fail are in section 3.

## 2. What the record gets right that a reader should not have to rediscover

**The record does not repeat the false example that FND-141 corrected.** This
was checked directly, because the false example is in the tree. FND-141 records
that a one-byte tile field at 255, summed over 16.7 million tiles, reaches
4,258,500,000 and stays inside a `u32` by 0.85 percent.[^3] ADR-0083 states the
rule and gives no arithmetic: it says "an accumulator narrower than the level it
sums", and it names no field width and no tile count. The test that supports it
sums a two-byte field, which does overflow. The record and its test are both
correct on this point.

**The record leans on a magnitude, and the magnitude is paired.** The context
says the change makes the suite several times faster. FND-142 warns that a
figure taken hours from its comparison is a figure about the machine.[^4] A
paired measurement exists: the commit that carried the change alternated the two
profiles back to back and reports 429 s and 430 s against 84 s and 79 s for
`cargo test --workspace` with nothing to rebuild. The Rust tests are nearly the
whole suite.[^5] The claim survives.

**The register the record cites does not hold that pair, and one of its rows is
contaminated.** This is a defect in the register, not in the record, and
section 5 files it.

## 3. What must change before the record is accepted

Each item gives the text to replace and the reason. No decision changes.

### 3.1 One third of the test does not compile out of the release build

The record says:

> **The test compiles only into a build that has the debug assertions on.** The
> release run of the slow gate compiles it out, because the release profile
> wraps by design and asserting otherwise would make that gate red for a correct
> reason.

The test file holds three tests. Two carry `#[cfg(debug_assertions)]`. The
third, which asserts that the widened accumulator holds the exact sum, carries
no attribute and its own comment says it holds in every profile. The slow gate
runs `cargo test --workspace --release`, and continuous integration runs the
slow gate, so that third test runs in the release build and passes there.

The paragraph before this one calls all three assertions "the test", so a reader
takes the sentence to cover the file. Read that way it is false, and it tells a
reader that no assertion from this file reaches the release gate.

**Replace with:** a sentence that names the two panic assertions as the part
that compiles only into a build with the debug assertions on, and states that
the assertion about the widened accumulator holds in every profile and runs in
the release gate.

### 3.2 The debug assertions do not carry the check, and the manifest says so

The record says:

> A contributor who turns the debug assertions off in the development profile
> removes the test as well as the check, and nothing fails.

The manifest sets `overflow-checks = true` explicitly. Cargo carries the two
keys separately, which the record's own context states two paragraphs earlier.
A contributor who sets `debug-assertions = false` and leaves the rest alone
therefore removes the test and keeps the check.

This makes the record contradict itself, and it makes the residual sound like
the wrong thing. The residual is worse than the record describes in one way and
better in another: the enforcement disappears silently while the check stays, so
the next contributor removes the check against a suite that is already blind.

**Replace with:** a statement that turning the debug assertions off removes the
two panic assertions and leaves the check in place, so the enforcement goes
before the check does. Say that the manifest sets the two keys explicitly, which
is why they do not fall together.

### 3.3 The consequences claim a comparison that nobody measured

The record says:

> The speed that was available from the optimisation level is taken, and it is
> the larger of the two.

Nobody measured the suite with the overflow check off. The commit that carried
the change ran the check off twice, and both runs were the overflow test alone,
to prove that the test can fail. Neither run was timed. The definition of done
forbids a claim of a measurement that was not taken.[^6]

The claim is very likely true by reasoning, and reasoning is an acceptable
support. It is not acceptable stated as a result.

**Replace with:** the reasoning. The optimisation level cut the execution of the
tests by a factor near five, and an overflow check is a comparison and a branch
on each arithmetic operation, so the check cannot hold a saving of that size.
State it as a derivation and not as a measurement.

## 4. Objections attempted

**"The record states a measured figure in its body, which section 4.1 of the
scope rule forbids."** The context says the change makes the suite several times
faster.

It fails. The rule names a cost budget, a byte budget, a throughput figure, a
latency figure and a percentage.[^2] "Several times faster" is none of these. It
is a magnitude that a better measurement narrows and does not overturn, and it
is the force that makes the record necessary. The record refuses the numbers
themselves, states that the register owns the level, and cites the register. A
record that could not say why the change was made would fail section 5 of the
same rule, which requires the forces.

**"The record leans on the register, and the register's rows are the unpaired
comparison FND-142 forbids."** This holds against the register and fails against
the record. Section 5 files it.

**"D1 records a build configuration, and section 4.4 forbids a module
arrangement."** It fails. D1 states a property of the build: it must check every
integer operation and must panic. It does not state which file holds which key,
and it explicitly refuses to state the optimisation level. A reviewer can find a
violation of D1 by running the suite, which is the form the rule asks for.

**"D2 is a testing requirement and belongs in the testing rule, not in a
record."** It fails. ADR-0001 D5 sets the precedent that a record may require a
test to be able to fail, and the reason is the same here: the enforcement is the
only thing that makes D1 more than a wish, so a record that stated D1 without D2
would state a constraint nothing holds. The three-condition test also passes. A
contributor could reasonably drop the test to save the suite. The cost of
dropping it lands later, as a silent wrap. The reasoning for a test rather than
a lint is not visible in the artefact.

**"The record rejects the script that would close its own residual, and the
residual is real."** The record rejects a script that greps the manifest,
because it would state the same fact in a second place.

It fails, and it is the strongest objection attempted. The rejection is argued
from the recurring defect rule, which names a second declaration site as the
shape this project sees most often.[^7] A grep of the manifest would also miss a
build that a `RUSTFLAGS` value changes, which the test catches. The residual is
narrower than the record says, and item 3.2 corrects the description rather than
the decision.

**"Footnote 2 cites ADR-0002 D2 for saturation, and D2 does not state
saturation."** It holds, weakly, and it is not a reason to refuse the record.
ADR-0002 D2 states that one module defines every simulation operation. The
saturation is a property of that module's code: every operation in the
fixed-point module uses a saturating form. The claim in ADR-0083 is true against
the code, and the footnote points at a decision that supports half of it. A
better footnote names the module. This is worth fixing in the same edit and does
not stand on its own.

**"The record is too long, or it holds two claims."** It fails on both counts.
The record runs about 1100 words, which is under both reference medians named by
the scope rule. D1 and D2 are separable: a build that wraps violates D1 while D2
passes, and a suite with the test deleted violates D2 while the build still
checks.

**"The record is uncited."** It fails. Three files cite it: the test, the
findings register and the development budget register. The record check reports
no note against it.

## 5. Two register defects the review found

Neither holds the record. Both are filed.

**The development budget register holds the comparison its own text
forbids.**[^8] The register says a row is a snapshot and does not support a
comparison against a row taken hours earlier, and it cites FND-142 for the
reason. Its own warm rows are 153 s at `opt-level 1` against 435 s with no
optimisation. The commit that recorded them says the fair pair is "the earlier
warm baseline of 435 s", so the two runs are not next to each other in time.
FND-142 lists 429 s, 432 s, 435 s and 435 s as third-hour figures for the
unchanged suite and 263 s, 283 s and 296 s as first-hour figures for the same
suite. The 435 s row is therefore a sample of a quantity that measured 263 s
earlier in the same session, and the ratio a reader takes from the table is
inflated. The paired evidence exists and is in a commit body, where the register
cannot reach it. FND-149 holds this.

**The sweep that corrected the false accumulator example reached one document
and left six.** FND-141 corrected the example, and the project owner's document
now states the arithmetic correctly. A whole-tree search finds the old example
in an accepted record, in a second accepted record, in two source comments, in
one further source comment, and in a complete backlog item. Two of the six are
accepted records, so the repair is a decision the reviewer does not own.
FND-150 holds this, and item 0165 carries the work.

## 6. What the review could not check

**Nothing was compiled and nothing was run.** Another worker held the machine.
The following claims rest on the commit body of the change and are unverified by
this review:

- That the two panic assertions fail when `overflow-checks` is off, by either
  route. The commit reports "1 passed; 2 failed" for both routes.
- That all three tests pass under the current profile.
- That the two determinism tests pass under the current profile.
- That the third test compiles and passes in the release build. This follows
  from reading the attributes and the gate recipes, and no run confirmed it.

The five record and register checks were run, because they compile nothing.
`check-adrs.sh` reports 37 records, 0 failures and 1 note, and the note is
against ADR-0082 and not against this record. `check-citations.sh` reports 3433
citations and 0 failures. `check-registers.sh` reports 182 entries and 0
failures. `check-priority.sh` reports 77 rows and 0 failures.

**Nothing was measured.** Every figure named in this review was taken from a
register or from a commit body, and every one of them describes a development
machine. No figure here is evidence about the target platform, and BLK-007 stays
open and untouched.[^9]

## 7. For the registers

- ADR-0083 stays at `Draft`. The registry row does not change, and no citation
  of the draft path moves.
- FND-149 opens: a register can hold the comparison its own text forbids.
- FND-150 opens: the sweep that corrected the accumulator example reached one
  document and left six.
- Item 0165 opens in `proposed/`, and it carries both repairs.
- No blocker opened or closed.
- No decision row was needed. The record makes the choice and the record holds
  it.

## References

[^1]: Definition of Done, the rule behind the rule. `.claude/rules/definition-of-done.md`
[^2]: Decision Record Scope, section 4. `.claude/rules/adr-scope.md`
[^3]: Findings register, FND-141. `docs/FINDINGS.md`
[^4]: Findings register, FND-142. `docs/FINDINGS.md`
[^5]: Findings register, FND-140. `docs/FINDINGS.md`
[^6]: Definition of Done, section 6. `.claude/rules/definition-of-done.md`
[^7]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
