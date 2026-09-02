# ADR-0083: The gate build checks every integer overflow

## Context

The gate suite compiles the Rust tests in the development profile. That
profile checks every integer operation for overflow and panics when one
happens. The release profile wraps instead, and wrapping is silent.

The suite is slow, and the profile is why. An unoptimised build of a
simulation that steps hundreds of thousands of tiles spends most of its time
in code the optimiser would have removed. Raising the optimisation level of
the development profile makes the suite several times faster, and the
measurements are in the development budget register.[^1]

Raising the optimisation level does not remove the overflow check. Cargo
carries the two settings separately, and the development profile keeps the
check at every optimisation level. **The two are separate, so a contributor
can take one without the other, and nothing announces it.** A single line in
the workspace manifest turns the check off, and the suite gets faster again.
That is the trade this record refuses.

The check matters more here than the wrapping default suggests. The
arithmetic module saturates every operation it offers, so nothing inside that
module overflows and nothing inside it panics.[^2] The check therefore
protects the arithmetic outside that module: a count, an index, a capacity,
and the accumulator of a pyramid level. An accumulator narrower than the level
it sums is the defect the widening rule exists to stop, and an overflow panic
is the cheapest thing that finds one.[^3]

Nothing else in the project finds it. A wrapped sum is a plain number. It
passes a type check, it passes a lint, and it passes both determinism tests,
because a wrap is deterministic. The determinism tests compare a run against
another run, and a wrapped accumulator wraps the same way in both.[^4]

## Decision

### D1. The gate build checks every integer overflow

The build that the gate suite tests must check every integer operation for
overflow and must panic when one happens.

A profile setting that raises the optimisation level of that build is
allowed, and it is the reason this record exists. A profile setting that
turns the overflow check off is not allowed, at any optimisation level.

This record states no optimisation level. The level is a cost figure, it
follows a measurement on a development machine, and the register owns it.[^1]

### D2. A test asserts the check, and the test is the enforcement

The gate suite holds a test that overflows an integer on purpose and asserts
that the operation panics. A contributor who turns the check off gets a red
suite rather than a quiet loss.

The test reads the outcome by catching the panic. It does not read a compiler
switch. The switch that names this build is not stable on the pinned
toolchain, so a lint cannot see this and a test must.

The test asserts a second thing: that an accumulator too narrow for the level
it sums panics, and that the widened accumulator holds the same sum exactly.
That is the defect the check exists to catch, so the test drives it rather
than only the simplest overflow.

**The test compiles only into a build that has the debug assertions on.** The
release run of the slow gate compiles it out, because the release profile
wraps by design and asserting otherwise would make that gate red for a
correct reason. A contributor who turns the debug assertions off in the
development profile removes the test as well as the check, and nothing fails.
That residual is stated here rather than hidden, because the profile that
does it is not a profile anybody has a reason to write.

## Consequences

The project cannot buy suite speed by turning the overflow check off. The
speed that was available from the optimisation level is taken, and it is the
larger of the two.

A contributor who raises the optimisation level pays for it in compilation.
The first build after a profile change rebuilds every unit of the workspace.
An edit to one source file afterwards costs about what it cost before, so the
cost lands once and not on the loop a contributor runs all day. The figures
are in the register.[^1]

The overflow check is a development net and never a target guarantee. The
shipped build uses the release profile, which wraps. Code that relies on a
panic to stop a wrap is wrong in the shipped build, and the widening rule,
not this record, is what makes an accumulator correct.[^3]

A test that overflows on purpose prints nothing when it passes, because it
replaces the panic hook for the call. A reader of the suite output sees the
test name and the result, not a backtrace.

## Alternatives rejected

**Switch the gate to the release profile.** This is the obvious way to make
the suite fast, and it was rejected because it takes the overflow check with
it. The release profile also links with link-time optimisation and one
codegen unit, so it compiles far more slowly than the development profile at
a raised optimisation level, and it would have cost compilation to buy
execution.

**Raise the optimisation level of the dependencies only.** This is the cheap
version of the change, and it does nothing here. The simulation is first-party
code. The dependency graph of the core is one derive macro and, in the tests,
a property-testing library. Optimising it leaves the work where it was, and
the measurement says so.[^1]

**Reduce the number of property-testing cases.** This is a knob, not a fix. It
buys time by testing less, and the fixture rule already warns that a test
which measures its own fixture proves nothing.[^5] The optimisation level buys
the same time and tests the same thing.

**Enforce the check by reading the manifest in a script.** A script that greps
the workspace manifest would close the residual named in D2. It was rejected
because it states the same fact in a second place, and a second declaration
site is the shape this project sees most often.[^6] The test reads the
behaviour of the build, which is the fact that matters.

## References

[^1]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Testing rules, section 2a. `.claude/rules/testing.md`
[^6]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
