# Review: bind the fallen log to the control plane

This document reports the work of backlog item 0390. The engine ends a unit in
a meeting between two factions and writes one event for each unit that fell.
Before this work no binding read that event, so a caller in the control plane
watched its faction population fall and could not see where or to what.

The item is complete. No decision record was written. One finding and one
decision opened, and one backlog item was added.

## 1. The architectural impact review

The review was made before any code was written. It names each governing record
by number and by decision.

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0040 | D1 | The read gives the whole log as columns in one crossing. Python never loops over the dead. |
| ADR-0044 | D1 | The read copies each column, and the doc comment declares the copy at the call site. |
| ADR-0085 | D1 | The unit column carries the whole identity. No column carries a slot index. |
| ADR-0085 | D3 | Every identity in the column is dead, and the engine refuses a dead identity rather than answer for the next occupant of the slot. |
| ADR-0121 | D4 | The log holds the units of one frame, in ascending slot order, and the pass empties it at its start. The doc comment states both. |
| ADR-0004 | D1 | The order of the entries is the ascending slot order the step ended the units in, so it follows no thread. |
| ADR-0001 | D4 | The read moves no state, so no stored golden hash moves. |
| ADR-0002 | D1 | Every column is an integer array. No floating point value crosses the boundary. |
| ADR-0006 | D1 | The event declares its padding, and the padding is not a field, so no column carries it. |
| ADR-0046 | D1 | The read raises a typed error. |
| ADR-0107 | D1 | The doc comment is the published reference, so it states every column, its element type, its order, its lifetime and its error. |
| ADR-0041 | D1 | The core crate gains no line and keeps no binding dependency. |
| DEC-060 | — | The read gives one column for each field, keyed by the field name, so no caller holds a byte offset or a field order. |

No record is contradicted. No record is superseded. No record was created: the
four logs that the bindings already expose set the shape, and a fifth of the
same shape states no new constraint.[^1]

**The item asked whether this is a row of item 0319. It is not.** Item 0319
holds three logs that no binding reads, and each one needs its own columns and
its own fixture. This work is a fourth log of the same shape, and it finished
on its own. Item 0319 stays open and unchanged.

## 2. What the binding exposes

Two names, which follow the gather log field for field.

`fell_log_columns()` returns a dictionary of five NumPy arrays. Every array
holds one entry for each unit that fell, and all five are the same length.

| Column | Element type | What it holds |
|---|---|---|
| `tick` | `numpy.uint64` | The step at which the unit fell |
| `unit` | `numpy.uint64` | The whole identity of the unit that fell |
| `tile` | `numpy.uint32` | The tile it stood on, as a row-major index |
| `faction` | `numpy.uint16` | The faction it belonged to |
| `unit_type` | `numpy.uint8` | The row of the shared type table it carried |

`fell_count` is the number of entries, as an integer.

No column carries a fixed-point value, so no column needs a scale. The tick is
a count of steps, the tile is an index, the faction is a faction number, and
the type is the row number that `define_unit_type` writes.

**A caller learns who fell, which faction it belonged to, where it stood, which
type it carried, and at which step. It does not learn what killed the unit.**
The pass records no killer, because it resolves a meeting for a whole group at
one tile and no single attacker owns one death.[^2] The doc comment says so,
and it tells the caller to read the enemy from the tile and the step.

**Every identity in the unit column is dead**, because the step ended the unit
that it names. The doc comment says so, and it says why the tile column earns
its place: a caller cannot place the death with a second read, because the
engine refuses a dead identity.

## 3. The last-step lifetime

**The log covers the last step alone, and the next step destroys it. The doc
comment states this in bold, at the call site.**

The engine empties the log before it resolves a meeting, so the hazard is
narrower than a reader might fear. A step with no fight gives five empty arrays
and never the entries of an earlier step, so a stale read is not possible. What
is possible is loss: a caller that steps twice before it reads has lost the
first step, and nothing fails.

**No queue was built.** A queue inside the engine needs a bound, a rule for
what happens when the bound is reached, and a decision record, because a queue
that drops its oldest entry silently is the same failure one layer on. It also
buys a caller nothing that a recorder in the control plane cannot buy outside
the engine, and the engine would then hold state that only one caller wants.

**The rule is uniform across four logs, and that uniformity is worth more than
the convenience of one log.** A caller who has read the gather log reads this
one with nothing new to learn. Making this log the exception would put one rule
in two shapes, with nothing that fails when the two disagree.[^3]

The open decision holds the three options and recommends leaving the engine as
it is until a caller states a need.[^4] A backlog item holds the work that
follows it.[^5]

## 4. The tests

**Every test starts at the Python boundary.** Each one builds a world through
the package, defines the two unit types through the package, spawns two
factions on neighbouring tiles through the package, steps the world, and reads
the log through the package. No fixture reaches into the engine.

The fixture is one tank of one faction beside four bowmen of another. The tank
delivers four whole casualties and carries an armour above the attack of a
bowman, so it ends all four bowmen and takes nothing back. That produces a log
of exactly four entries, of one faction and one type, which is what the
assertions read. The helper that steps the world raises when the fixture
reaches no fight, so a fixture that stopped producing a death fails rather than
asserting over an empty log.

| Test | What it holds |
|---|---|
| `test_the_fallen_columns_carry_the_fields_by_name` | The five keys, the length of each column against `fell_count`, and the element type of each column |
| `test_a_fallen_event_says_who_fell_and_where` | The set of fallen identities equals the set the spawn verb returned, the tank is not in it, and the tick, the tile, the faction and the type of each entry |
| `test_an_identity_in_the_fallen_log_is_dead` | The engine refuses every identity in the column |
| `test_a_step_with_no_fight_gives_an_empty_log` | A step that ends nobody gives an empty log, not the entries of the step before it |
| `test_a_new_world_gives_an_empty_fallen_log` | A world that has never stepped gives five empty arrays and the five keys |
| `test_the_thread_count_does_not_change_the_fallen_log` | The same fixture at 1, 2 and 12 threads gives the same log |

The existing test that bans a floating point column now reads the fallen
columns as well.

## 5. The defects put back, and which test caught each

Each defect was compiled into the tree, the extension was rebuilt and
installed, the tests were run, and the defect was then removed.

| The defect | Caught by |
|---|---|
| The step does not empty the log, so the log accumulates over the run | `test_a_step_with_no_fight_gives_an_empty_log` |
| The faction column carries the unit type | `test_a_fallen_event_says_who_fell_and_where` |
| The unit column carries the slot index rather than the identity | `test_a_fallen_event_says_who_fell_and_where` |
| `fell_count` reads the gather log | Every fallen test, through the helper that steps and through the length assertion |

**Every defect was caught. One test that looked like coverage was not.**

The third defect is the finding of this work. The test that hands each identity
back to the engine and asserts a refusal **stayed green** while the column
carried slot indices, because the engine refuses a slot index for the same
reason it refuses a dead identity: neither is an identity it gave out. Only the
comparison against the identities that the spawn verb returned saw the
difference. FND-443 records it.[^6]

## 6. The registers

| Register | What moved |
|---|---|
| Findings | FND-443 opened: a test that watches a dead identity refuse does not cover the column that carries the identity. |
| Decisions | DEC-224 opened: does a log the bindings expose hold more than the last step? Three options, with a recommendation. |
| Blockers | None opened and none closed. The blocker number this work was given stayed unused, because it found nothing that needs information the project does not have. |
| Registry | Unchanged. No record was written. |
| Backlog | Item 0390 moved to `complete/`. Item 0432 was added to `proposed/` and to the priority index. The allocation of items 0433 to 0436 was not used. |

**The next-number line of each register now points above the block that another
worker holds.** The register check demands a pointer one above the highest
entry, and the allocation for this work starts ten numbers above the merged
pointer. Each register therefore states one above the row this work added, and
the merge must lower each pointer to one above whatever the merged file then
holds. The numbers this work took collide with no other worker's.

**No cost figure was recorded.** This work took no measurement of a frame, and
the read is not a stage of the step.

## 7. The gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass |
| Thread-count equivalence, at 1, 2 and 12 threads | Pass |
| Golden state hash | Pass, and no stored file moved |
| `just records` | Pass |
| `just lint-python` | Pass |
| `just test-python` | Pass |
| `just docs` | Pass |
| `just docs-probe` | Pass |

**No golden state hash moved, and that is the result the work expected.** The
read copies columns out of a log that the step already wrote. It writes
nothing, so the state that the hash covers is the state it covered before.

**The formatter repaired one file this work did not write.** A merge left a
test file two blank lines short of the style the formatter demands, and the
format gate refuses it. The repair is two blank lines and no other change.

The commit body holds the command output.

## 8. What is left undone

**Item 0319 stays open.** Three logs still have no binding: the log of a unit a
shortage ended, the log of a unit a step promoted, and the log of a site that
fell short. This work gives them a fourth worked example to follow.

**The lifetime of a log is still a discipline that the caller holds.** DEC-224
is open with a recommendation, and item 0432 holds the work. Nothing in the
package helps a caller keep more than one step.

**No panel and no agent tool reads the fallen log.** The demonstration window
and the agent server both read other logs, and this work changed neither.

**The log still names no killer.** That is a property of the contest pass,
which this work did not touch. A caller that wants the faction that did the
killing reads the tile and the step, and the engine offers no direct answer.

## References

[^1]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^2]: ADR-0123, casualties are whole units served to a keyed subset, decision D1. `docs/adrs/draft/adr-0123-casualties-are-whole-units-served-to-a-keyed-subset.md`
[^3]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: Decisions register, DEC-224. `docs/DECISIONS.md`
[^5]: Backlog item 0432, decide the lifetime of every log the bindings expose. `docs/backlog/proposed/0432-decide-the-lifetime-of-every-log-the-bindings-expose.md`
[^6]: Findings register, FND-443. `docs/FINDINGS.md`
