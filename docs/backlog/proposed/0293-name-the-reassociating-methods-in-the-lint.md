---
id: 0293
title: Name the reassociating methods in the lint
status: proposed
created: 2026-09-02
implements: [ADR-0002]
changes: []
creates: []
serves: []
blocked-by: [DEC-107]
---

## Why

**A door the compiler used to hold shut is now open, and only one of the two
mechanisms that guard the arithmetic boundary stands in it.** The project
forbids floating point in simulated and aggregated state, and it holds that
boundary with two mechanisms because one is not enough.[^1] The first is a
lint that bans the float types by name. The second is a script that catches
what the lint cannot see.

**The reassociating float methods were rejected by the compiler, not by either
mechanism.** On the stable release the project pinned, a call to one of those
methods was a hard error, because the library feature was gated and the gate
cannot be opened on a stable channel. On the dated nightly the project now
pins, the same call compiles with no attribute.[^2]

**The lint can now name them, and it does not.** The same measurement shows
that the lint resolves the method and rejects it once its list holds the path.
The list does not hold it. The script's name check is what stands there today,
alone.

**A lint entry that reaches nothing is silent, which is why this is worth a
check and not only a change.** The lint tool ignores a disallowed-method path
it cannot resolve, and it emits no warning and no note.[^2] So an entry added
here could be misspelt, or could stop resolving on a later date, and the file
would read as a live rule for as long as anybody left it there.

## What the work does

Add the reassociating methods to the disallowed-method list of the lint
configuration, for both float widths. Leave the script unchanged.

Add a check that fails when a disallowed-method entry resolves to nothing, so
that the silence of the tool stops being the project's problem. A fixture with
a call site for each banned path, compiled under the lint, is one way: an entry
that fires proves it resolves.

## What good looks like

A file that calls a reassociating method is rejected by the lint, and is
rejected by the script, and the two failures name different mechanisms.

**Put the defect back and watch the check stay green.** Misspell one path in
the lint configuration and confirm that the new check fails. An entry that
cannot be shown to fire has not been shown to exist.[^3]

## What it costs at the target scale

Nothing. This is a build-time check and no simulation code changes.

## What it does not do

It does not narrow the script, and it does not remove anything. The register
holds the open question of what each mechanism should cover, and a reviewer
owns it.[^4] This item implements the recommendation of that row and no more.

It does not decide whether a reassociating method may ever be used at the
interpreter boundary. Nothing in the core crate holds a float at all today.

## References

[^1]: ADR-0002, state holds no floating point number, decision D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: Findings register, FND-284. `docs/FINDINGS.md`
[^3]: Testing rules, section 2a. `.claude/rules/testing.md`
[^4]: Decisions register, DEC-107. `docs/DECISIONS.md`
