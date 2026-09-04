# Review of item 0470, expose the economy tuning knobs to Python

This document reports the work of one backlog item.[^1] The item bound the
economic parameters of the simulation core to the Python control plane. The
core is a Rust crate. The control plane is a Python package that imports a
compiled extension module.

The engine held every one of these values before this work. A developer who
wanted a settlement to earn more, owe more, hold more, or return a worked
deposit faster had to fork the engine. That is the outcome the design
principle "unit types and upgrades are data, not code" exists to prevent.[^2]

The work is a boundary. It adds no engine behaviour, no storage and no pass.

## 1. The impact review

The review was made before any code was written. The item was refined, the
product record was written, and both priority indexes gained a row, in one
commit before the first line of Rust.

### Records that govern the work

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0040 | D1 | Each write that names places takes the whole set and answers once. |
| ADR-0043 | D1 | A settlement and a unit are written in sets. A world-wide value is one write for the world. |
| ADR-0046 | D1 | Every refusal the engine makes raises a typed class under the one root class. |
| ADR-0085 | D3 | Every identity resolves against its generation before anything is written. |
| ADR-0107 | D2 | Every word of prose lives in the Rust doc comment. |
| ADR-0002 | D1 | Every rate and every quantity crosses as an integer. |
| ADR-0062 | D1, D4 | A rate belongs to a site, and the stored rate is what one tick earns. |
| ADR-0001 | D4 | A value that a later frame reads reaches the state hash, or the work says why not. |
| ADR-0060 | D2 | The influence value uses its own narrow scale, and the doc comment states it. |
| ADR-0087 | D1 | The influence solve runs at the end of a step, over the whole plane. |

### Decision by decision, against the implementation

**ADR-0040 D1. The boundary carries an instruction and an answer, never the
population.** Five of the eight writes take a set: the production rate, the
upkeep rate, the settlement store, the home site, and the influence source.
Each takes one sequence and answers once. The other three are world-wide
values, and each is one write for the world. No caller loops. Honoured.

**ADR-0043 D1. The tier of a shape decides the shape of its interface.** A
soldier is the mass tier and a settlement is the group tier, and both write
verbs over them take a set. The three reads this work adds are singular or
world-wide, which follows the existing rule: a set-valued read would have to
answer for a stale identity with a value that stands for nothing. Honoured.

**ADR-0046 D1. One root type holds every error.** A stale or unknown identity
raises the view error. A value the engine refuses raises the verb error. A
sequence of the wrong shape raises the built-in `ValueError`, which the module
doc comment already names as the class for that case. Honoured.

**ADR-0085 D3. An entity crosses as one opaque identity that the engine
resolves.** Every set-valued write resolves the whole set into a list before
it writes anything. A stale identity leaves the world unchanged. A test proves
it, and a defect put back into that path fails the test. Honoured.

**ADR-0002 D1. No floating point in simulated state.** Every rate, every
quantity and every influence value crosses as a Python integer and becomes an
integer inside the engine. No float appears in any signature this work adds.
Honoured.

**ADR-0062 D1 and D4. A rate belongs to a site, and the stored rate is what
one tick earns.** The doc comment of each rate write states this in bold,
because a caller who reads the rate as the amount of one application writes a
rate that is too large by the period, and nothing catches that. A test proves
the property: a run at a period of one and a run at a period of four earn the
same amount over the same span of ticks. Honoured.

**ADR-0001 D4. One binary gives one answer at any thread count.** The work
adds no simulation code and no parallel section. A test sets eight knobs, runs
six steps at 1, 2 and 12 threads, and compares the state hash and the store
against the same run at one thread. Honoured, with one exception that section
3 states: the recovery rules do not reach the hash, and that is a defect the
work found rather than one it made.

**ADR-0060 D2. The influence value has its own scale.** The doc comment of the
influence read and of the influence write states that 65535 means one
reference unit, and states that this is not the Q16.16 scale the rates use.
Honoured.

**ADR-0087 D1. The solve runs a fixed iteration count.** The doc comment of
the influence write says the next step spreads a source, and that the solve
runs last in a step over the whole plane. A test writes a source, steps once,
and reads a value above zero at the place. Honoured.

### Records changed or created

None. Every value already had a decision behind it, and the work contradicts
no record.

## 2. Each knob, when it may be set, and whether it is hashed state

**Every knob in the table may be set at any time.** None of them is
construction-time configuration. No write can land inside a step, because the
engine releases the interpreter for the whole step, so no Python line runs
while a step runs.[^3] Work already in progress is therefore never half
affected: a write lands between two steps, and the next step reads it.

| Binding | Scope | When it may be set | Hashed state | What it does to work in progress |
|---|---|---|---|---|
| `set_production_rate` | A set of sites | Any time | Yes | The next application uses the new rate. An application already made is not revisited. |
| `set_upkeep_rate` | A set of sites | Any time | Yes | The next application uses the new rate. |
| `set_settlement_store` | A set of sites | Any time | Yes | The write is absolute. The next application starts from the value written. |
| `set_economy_schedule` | The world | Any time | Yes | The new period and phase decide the next due tick. A rate is per tick, so what a site earns over a span does not change. |
| `set_recovery_rules` | The world | Any time | **No. See section 3** | The next tick that ages a depleted deposit uses the new period. A deposit already returned stays returned. |
| `set_deed_threshold` | The world | Any time | Yes | The next promotion pass reads the new threshold. A unit already promoted stays promoted. |
| `set_home_site` | A set of units | Any time | Yes | The next consumption draw takes from the new home. A ration already taken is not returned. |
| `set_influence_source` | A set of places | Any time | Yes | The next solve starts from the new source. The field of the last solve stays until then. |
| `recovery_rules` | The world | A read | — | — |
| `deed_threshold` | The world | A read | — | — |
| `influence` | One place | A read | — | — |

The hashed column was read from the engine, not assumed. The rate table, the
economy schedule, the settlement stores, the unit arena and the influence
sources are each folded into the whole-world hash. The depletion rules are
not.

### Three names that gained no binding

The item scope named thirteen engine members. Three of them already cross to
Python inside a report that a binding builds.

- The production rate of a site crosses as `production` in `site_economy`.
- The upkeep rate of a site crosses as `upkeep` in `site_economy`.
- The stock a tile started with crosses as `generated` in `tile_report`.

A second binding for any of the three would be a second declaration site for
one value, which is the defect shape this project names first in its own
rule.[^4] The doc comment of each write points at the report that already
answers the read. FND-481 holds the search that measured this.[^5]

## 3. The defect the work found and did not repair

**The recovery rules govern the step on every tick and stand outside the state
hash.** FND-480 holds the reading and its evidence.[^6]

The fold of the depletion ledger writes the entry count, and then the key, the
taken amount and the anchor tick of each entry. It never reads the rule field
beside them. Two worlds that hold the same tiles, the same takes and different
recovery periods therefore hash the same, and they diverge on the next tick
that ages an entry. The golden state test reports the effect of such a change
one or more ticks late, and never the change itself.

The work did not repair it. The repair is one line in the fold, and it changes
the hash chain, so it moves every golden file. A binding must not move a
golden file. Backlog item 0471 holds the repair and sits at the top of the
priority index.[^7]

The doc comment of `set_recovery_rules` states the defect, so a caller is not
misled. One test asserts it: it writes a rule set and asserts the state hash
does not move. **That test fails when the defect is repaired**, which is the
signal that the repair landed.

## 4. The units and the scales

This is where a caller writes a silent defect, so each number is stated once
in the doc comment of the call that takes it.

| Value | Unit and scale |
|---|---|
| A production rate | Q16.16 as its raw integer. 65536 means one unit of the commodity in one tick. |
| An upkeep rate | Q16.16 as its raw integer, at or above zero. 65536 means one unit owed in one tick. |
| A settlement store | Q16.16 as its raw integer. A quantity, not a rate. |
| An economy period | A count of ticks, at least 1 and at most 32767. Not fixed point. |
| An economy phase | A count of ticks inside the period. A phase at or above the period wraps into it. |
| A recovery period | A count of ticks in which one depleted deposit regains one unit of stock. Not fixed point. `None` means the kind does not recover. |
| A deed threshold | A count of whole units of resource a unit has ever gathered. Not fixed point. |
| An influence source | Unsigned fixed point against a fixed reference. 65535 means one reference unit. **Not the Q16.16 scale.** |
| A commodity number | An index. The world holds one commodity, numbered zero. |
| A resource kind | An index. Food is zero, wood is one, stone is two. |

The two scales are the trap. A caller that passes 65536 as an influence source
gets the ceiling, and a caller that passes 1 as a production rate gets one part
in 65536 of a unit. Both doc comments state the scale in bold, and the test
file pins both numbers as named constants, in the way the documented values
test pins the constructor defaults.

## 5. What was proved, and what was not

**Eleven bindings reach the engine, and a test proves each one. Six of the
eight writes were proved to change what a later step does.** The other two
change hashed state, and a test proves that much; no read at this boundary can
show their effect on behaviour. Every test starts at the Python boundary and
drives the installed package.[^8]

| Knob | Witness | What the test asserts |
|---|---|---|
| `set_production_rate` | Behavioural | The store rises by the rate after one step, at a period of one. |
| `set_upkeep_rate` | Behavioural | The store falls by the rate after one step. |
| `set_settlement_store` | Behavioural | The site reports the value written, and a second write replaces the first. |
| `set_economy_schedule` | Behavioural | A far period moves no store over three steps, and a period of one moves it on the next step. |
| `set_recovery_rules` | Behavioural | A world at a period of one returns a depleted deposit further than a world where the kind does not recover, over the same 40 steps, with nothing gathering. |
| `set_influence_source` | Behavioural | A world with a source reports influence above zero after one step, and a world without one reports zero. |
| `set_deed_threshold` | Read-back and hash | The value reads back, and the state hash moves. |
| `set_home_site` | Hash | The state hash moves when a home is given, and again when it is taken away. |
| `recovery_rules` | Read-back | The three periods read back what was written. |
| `deed_threshold` | Read-back | The threshold reads back what was written. |
| `influence` | Behavioural | The read answers zero before a source and above zero after the solve. |

**Two knobs could not be proved to change the simulation from Python, and the
report says so plainly.**

- `set_deed_threshold`. The boundary publishes no read of the character
  population, so a test cannot watch a promotion happen or fail to happen. The
  hash witness proves that the write reached the engine and that the value is
  hashed state. It does not prove that the promotion pass reads it.
- `set_home_site`. The boundary publishes no read of the home of a unit and no
  read of the ration a unit received, so a test cannot watch a unit draw from
  a store. The hash witness proves the write reached the engine.

Both gaps are the same shape: a write with no matching read at the boundary.
Neither is a defect in this work, and neither is repaired here.

## 6. The defects put back, and whether each was caught

Each defect below was put into the bindings crate, the extension was rebuilt,
and the test file was run. The rule is that a fixture proves nothing until the
defect goes back and the test fails.[^9]

### Round A. A write that accepts a value and changes nothing

| Defect | Caught |
|---|---|
| The production write passes zero to the engine instead of the rate | Yes. Four tests failed. |
| The upkeep write passes zero instead of the rate | Yes. |
| The store write passes zero instead of the quantity | Yes. Three tests failed. |

### Round B. A write that reaches nothing

| Defect | Caught |
|---|---|
| The economy schedule write ignores its arguments | Yes. Four tests failed. |
| The deed threshold write ignores its argument | Yes. Two tests failed. |
| The home site write skips its loop | Yes. Two tests failed. |
| The influence source write skips its loop | Yes. Three tests failed. |
| The recovery rules write ignores its argument | Yes. Three tests failed. |

### Round C. A refusal that is not all or nothing

| Defect | Caught |
|---|---|
| The production write resolves each identity inside the loop, so it writes as it goes | Yes. The stale-identity test failed. |
| The influence write runs before it checks the addresses | Yes. The outside-the-world test failed. |
| The boundary check on a rate below zero is removed | **No.** |

**The third defect was not caught, and the work removed the check rather than
the test.** The engine refuses a rate below zero itself and writes nothing, so
the boundary check restated a rule that already had a home. That is the
redundant declaration shape, and nothing failed when the two copies disagreed
because they could not disagree in a way any test saw. The same argument holds
for the boundary check on the commodity number.

Both checks are gone. The engine is the one declaration site for both rules,
and the doc comments now say that the engine refuses the value before it
writes the first site, and that neither refusal depends on the site. A test
sets a rate for three sites, then attempts a rate below zero and a commodity
the world does not hold, and asserts that all three sites keep the rate they
had.

**One test in this file has no proven failure mode.** The three-site refusal
test above pins a property that lives in the engine, so no defect at the
boundary makes it fail. It is stated here as unproven rather than left to look
proven.

## 7. The gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass. No file changed. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass. No warning. |
| `cargo test --workspace` | Pass. 891 tests passed, 0 failed. |
| `cargo test --test thread_equivalence` | Pass. 15 tests passed, 0 failed. Five scenarios run the same tick at 1, 2 and 12 threads. |
| `cargo test --test golden_state_hash` | Pass. 2 tests passed, 0 failed. No golden file moved. |
| `just records` | Pass. 0 failures and 2 notes, across every check the recipe runs. |
| `just lint-python` | Pass. Ruff reports no error. Mypy reports no issue in 27 source files. |
| `just test-python` | Pass. 200 tests passed, 0 failed. Thirty-three test functions are new, and one of them runs at three thread counts, so the file contributes thirty-five cases. |
| `just docs` | Pass. 105 members with prose and 0 without. Every summary reached the site. |
| `just docs-probe` | Pass. Both cases failed the job, as the probe requires. |

**No golden file moved.** The work adds no simulation code, changes no engine
value and changes no fold. The golden state test compares against the stored
files and they are the files that were there before.

The thread-count test and the golden state test both run inside
`cargo test --workspace` as well, and they are named separately above because
the definition of done names them separately.

## 8. What is left undone

- **The recovery rules stay outside the state hash.** Item 0471 holds the
  repair. This is the most important line of this document.
- **No read of the home of a unit, and no read of the character population.**
  Two writes therefore have a hash witness and no behavioural one. Neither has
  a backlog item, because neither is in scope here.
- **The type stub is still hand-written and nothing compares it against the
  module.** The eleven new signatures were added by hand, in the way every
  other signature in that file was. Item 0307 holds the generator.
- **The world still holds one commodity.** Every rate write takes a commodity
  number and the only valid one is zero. FND-430 records why.

## References

[^1]: Backlog item 0470, expose the economy tuning knobs to Python. `docs/backlog/complete/0470-expose-the-economy-tuning-knobs-to-python.md`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/REGISTRY.md`
[^4]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Findings register, FND-481. `docs/FINDINGS.md`
[^6]: Findings register, FND-480. `docs/FINDINGS.md`
[^7]: Backlog item 0471, fold the recovery rules into the state hash. `docs/backlog/proposed/0471-fold-the-recovery-rules-into-the-state-hash.md`
[^8]: The economy knob tests. `tests/test_economy_knobs.py`
[^9]: Testing rules, section 2a. `.claude/rules/testing.md`
