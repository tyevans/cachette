---
id: 0515
title: Write the five weather tests three closed items asked for
status: proposed
created: 2026-09-06
implements: [ADR-0160, ADR-0161, ADR-0162]
changes: []
creates: []
serves: [PRD-0004]
blocked-by: []
---

## Why

**Three items closed with the engine work done and the evidence they demanded
never written.** Each named a test in its own acceptance, each shipped without
it, and each outcome now says so plainly.[^1] [^2] [^3] This item is the debt.

The engine work is real and the records that govern it are written. What is
missing is the set of assertions that would fail if somebody undid the work.

**Five assertions are missing.**

1. The wind lags the pressure. A record states that the acceleration step is
   what makes the wind lag, and that the lag is the whole reason the field
   carries the wind at all.[^4] Nothing tests it.
2. Drag brings a wind to rest. The same record states that drag settles the
   wind where the step and the share balance.[^5] Nothing tests it.
3. The air total is unchanged over random winds and random air planes. The item
   that carried the water on the wind asked for this as a property test, and
   asked that the isotropic share be put back and the failure recorded.[^2] The
   tests present are example tests, and no such perturbation is recorded
   anywhere.
4. Some cells are wet and some are dry at one tick, with no storm raised. **This
   is the exact statement the original measurement said was false**, so it is
   the assertion the whole rain item existed to make true.[^3] A probe measured
   it once, and a probe is not a test.
5. The near side of high ground holds more water than the far side.[^3]

**The precedent this item exists under.** A test that passed under its own
defect and a fixture that measured itself both appeared in this same wave, and
the register holds both.[^6] [^7] The rule is to put the defect back and watch
the test stay green, and that is the only proof that an assertion reaches its
case.[^8]

## What is missing before this is refined

- Which of the five can be stated as a property rather than as an example. The
  air total is one. The other four may be.
- Whether assertions 1 and 2 can be made through the public interface. A wind
  that lags a pressure needs a world in which the pressure moves and the reader
  can see both, and no constructor today builds one.
- Whether assertion 5 needs a world built for it. High ground with open water
  on one side is a distribution, not a seed the demonstration gives.
- What each test costs in run time. Two of the five want a long run.

## Done when

- Each of the five assertions exists as a test that drives the world step.
- Each test is proved able to fail, by putting the defect back and recording in
  the commit body that the test then went red.
- The commit body names which of the five are property tests and which are
  example tests, and why.
- No test asserts that something moved. Each states a count, a share or an
  equality, so that a partial failure fails.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Backlog item 0499. `docs/backlog/complete/0499-give-every-level-1-cell-a-wind-that-carries-its-momentum.md`
[^2]: Backlog item 0500. `docs/backlog/complete/0500-carry-the-air-water-along-the-wind.md`
[^3]: Backlog item 0501. `docs/backlog/complete/0501-make-rain-arrive-rather-than-sit-everywhere.md`
[^4]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D2. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^5]: ADR-0160, the wind is carried state, and the pressure gradient accelerates it, decision D3. `docs/adrs/accepted/adr-0160-the-wind-is-carried-state-and-the-pressure-gradient-accelerates-it.md`
[^6]: Findings register, FND-557. `docs/FINDINGS.md`
[^7]: Findings register, FND-558. `docs/FINDINGS.md`
[^8]: Testing rules, sections 2 and 2a. `.agents/rules/testing.md`
