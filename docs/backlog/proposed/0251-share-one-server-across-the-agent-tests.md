---
id: 0251
title: Share one server across the agent tests
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The Python test recipe now costs about as much as the whole gate budget.**
The budget for the suite is 190 seconds on the development machine the
register names.[^1] The Python test runner reported between 162 and 225
seconds in five gate runs on that machine on 2 September 2026.[^2]

The Python side ran one test module when the budget was measured. It now runs
four. The largest of the three new modules tests the agent-facing server, and
it holds 22 tests.

**Each of those 22 tests starts the server as a new subprocess.** The test
module opens a client, initialises a session, runs one conversation, and
throws the subprocess away. Nothing is shared between tests. Each start
imports the interpreter, the protocol library and the compiled extension.

A fixture that lives for the module, or for the session, would pay that cost
once instead of 22 times.

## What is missing before this is refined

- **The impact review.** No decision record is known to govern the lifetime of
  a test fixture. The work should say so with an empty list rather than by
  omission.
- **A measurement that names this module.** The figure above is the whole
  Python run, not this module. A per-module figure must exist before anyone
  claims the saving. The pre-registered prediction says this module holds
  above half of the Python test time, and that claim is untested.[^3]
- **Whether a shared session stays honest.** The tests build worlds and call
  verbs. A shared server carries state between tests, so a shared fixture may
  make one test depend on another. The testing rule asks a test to drive the
  real caller, and a client that reconnects is a real caller too.[^4] The work
  must say which tests may share a session and which must not, rather than
  sharing every one of them.
- **Whether the saving is worth the coupling.** If a shared fixture saves 20
  seconds and hides a state leak, it is not worth it. The number decides.

## Done means

- A per-module figure exists for the Python test run, on a named machine, with
  the architecture, the profile and the date beside it.
- The gate suite costs measurably less, and the register holds the new figure
  with the commit that caused it.
- No test passes because another test ran first. Putting a state leak back
  must turn the suite red.

## References

[^1]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^2]: Five gate run logs from 2 September 2026, on an Intel Core i7-1260P, x86-64. `/tmp/g26f.log`, `/tmp/g27.log`, `/tmp/g29.log`, `/tmp/g29b.log`, `/tmp/g22.log`
[^3]: A prediction, which gate grew. `docs/research/gate-cost-prediction.md`
[^4]: Testing Rules, section 5. `.claude/rules/testing.md`
