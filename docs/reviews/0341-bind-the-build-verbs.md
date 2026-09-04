# Review of item 0341, bind the build verbs

This document reports the work of one backlog item. The item bound the build
order and the upgrade removal of the simulation core to the Python control
plane.[^1] The core is a Rust crate. The control plane is a Python package that
imports a compiled extension module.

The core held these methods before this work, and no binding and no Python line
called one. The findings register holds the search that measured it.[^2] That is
the shape of a capability that nothing invokes, which the project rules already
name.[^3]

## 1. The impact review

The review was made before any code was written, and the item was moved to
`refined/` with the review in it before the work started.

### Records that govern the work

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0040 | D1 | Each write verb takes a set and answers once. No caller loops. |
| ADR-0043 | D1 | A soldier is the mass tier, so both write verbs are set-valued. |
| ADR-0046 | D1 | Every refusal raises a typed error under the one root class. |
| ADR-0085 | D3 | Every identity resolves against the generation before any write. |
| ADR-0090 | D1, D2, D4 | The store stays sparse, progress is read back, and a removal is the removal of the entry. |
| ADR-0107 | D2 | Every word of prose lives in the Rust doc comment. |
| ADR-0110 | D1 | The direction read answers for a faction and a block of ground. |

### Decision by decision, against the implementation

**ADR-0040 D1. The boundary carries an instruction and an answer, never the
population.** The build order, the stop order and the removal each take one
sequence and answer once. The answer of the removal is one integer whatever the
size of the set. Honoured.

**ADR-0043 D1. The tier of a shape decides the shape of its interface.** A
soldier is the mass tier. Both write verbs over soldiers take a set. The two
reads are singular, which follows the same rule as the existing read of the tile
of one soldier: a set-valued read would have to answer with a value that stands
for nothing when one identity is stale. Honoured.

**ADR-0046 D1. One root type holds every error.** The build order raises the
verb error for a number that names no upgrade kind, and the view error for a
stale identity. The removal raises the view error for an address outside the
world. The direction read raises the view error for an address outside the world
and for a faction the world does not hold. Honoured.

**ADR-0085 D3. An entity crosses as one opaque identity that the engine
resolves.** Both write verbs resolve every identity into a list before they
write anything. A stale identity leaves the world unchanged. A test proves it,
and a defect put back into that path fails the test. Honoured.

**ADR-0090 D1. One entry for each improved tile.** The work adds no storage. It
reads the store the core already keeps. Honoured.

**ADR-0090 D2. An unfinished build is stored, and its progress is a clamped
whole number.** The report of one tile now carries that progress as an integer.
No floating point number crosses. Honoured.

**ADR-0090 D4. Destroying an upgrade is removing the entry.** The removal calls
the core method and adds nothing. A test asserts that the capacity of the tile
returns to the generated value. Honoured.

**ADR-0107 D2. The prose lives in the Rust doc comment.** Every new member
carries its prose in the bindings crate. The type stub gained signatures and no
prose. The published reference is generated from an import of the compiled
module, and the documentation job checks that every summary the import found
reached a page. Honoured.

**ADR-0110 D1. The return direction is a field over cells.** The doc comment
says that the field answers for a block of ground and not for a tile, so two
addresses in one block give one answer. Honoured.

**ADR-0001 D4. One binary gives one answer at any thread count.** The work adds
no simulation code and no parallel section. A test runs the same build at 1, 2
and 12 threads and compares the state hash and the work done. Honoured.

### Records changed or created

None. The work states no new constraint, so it writes no decision record. The
two shape choices it made went to the decisions register instead, because each
is a judgement with options rather than a constraint on the project.[^4]

### Blockers

BLK-036 governs whether an upgrade changes hands when the ground does. The work
states no answer to it. BLK-007 governs every cost figure in this project, and
the work states no figure.

A blocker number was allocated to this work and stays unused. The work opened no
blocker.

## 2. What the core methods actually do

Each statement below is a read of the core source, not a repeat of what another
document said about it.

**The build order is singular in the core.** It takes one identity and one
upgrade kind, and it writes that kind into the build order column of the soldier
arena. It returns false when the identity is dead. It reads no tile, no holder
and no terrain. The order alone builds nothing.

**A step is what builds.** The pass that collects the build intents walks the
live soldiers, takes the build order and the tile of each one, and produces one
intent for each soldier that holds an order. It reads no holder and no faction.
The intents merge into the upgrade store in tile order, so several soldiers on
one tile add to one total and the total does not depend on the thread count.

**The stop order is the same write with an empty value.** The work already done
stays on the tile, because the work lives in the store and not on the soldier.

**The read of a build order answers twice.** The outer answer says whether the
identity is live. The inner answer says whether the soldier builds.

**The removal takes one address and answers with a flag.** It returns false for
an address outside the world, and false for a tile that carries no upgrade. It
cannot tell those two apart. The binding checks the address itself for that
reason, so it can refuse the first case and count the second.

**The direction read answers twice as well.** The outer answer says whether the
address and the faction name an entry of the field. The inner answer says
whether the block of ground holds a direction at all. A block that holds a
settlement of that faction holds no direction, and so does a block that reached
no settlement.

**The direction is an index.** It indexes the fixed table of six neighbour
offsets in the core. A caller that cannot read that table cannot use the answer.

## 3. The shape chosen for each binding

| Member | Shape | Why |
|---|---|---|
| `order_build(units, kind)` | Set of identities, one kind, answers with nothing | The core method is singular. The wrapper loops in Rust, as the existing spawn and gather verbs do. |
| `stop_build(units)` | Set of identities, answers with nothing | The same rule. |
| `build_order(unit)` | One identity, answers with an integer or nothing | A read, and a set form would have to invent a value for a stale identity. |
| `destroy_upgrades(addresses)` | Set of addresses, answers with a count | An address with no upgrade is not a refusal, and a count is one value whatever the size of the set. |
| `return_direction(faction, q, r)` | One faction and one address, answers with an integer or nothing | A read of one entry of a derived field. |
| `direction_offsets()` | Static, answers with the six offsets | The answer is derived from the engine table, so no second declaration site of the order exists. |

**Every write verb is all or nothing.** The build order and the stop order
resolve every identity into a list before they write. The build order checks the
kind first, so an empty set reads the check alone. The removal checks every
address against the world before it removes anything.

**The core method is singular and the binding is not.** The wrapper calls the
core method once for each member of the set, inside Rust. A per-unit verb that a
caller repeated would be the loop the control plane rule forbids. Building has
no cheaper whole-set algorithm today, which is the same position the existing
spawn verb records.

**One read gained three keys rather than becoming a new member.** The report of
one tile now carries the upgrade, the work done and whether the work is
finished. A build was invisible before that: a road showed as a larger capacity
with no stated cause, and a terrace showed as nothing at all. A finding records
it.[^5]

## 4. The defects put back, and what caught each one

Each defect was written into the bindings source, the extension was rebuilt and
installed, the tests were run, and the source was restored. Every defect was
caught. No defect passed.

| Defect | Test that failed |
|---|---|
| The build order resolves each identity as it writes, rather than resolving the whole set first | `test_a_build_order_that_names_a_dead_identity_writes_nothing` |
| The stop order resolves each identity as it writes | `test_stopping_a_build_that_names_a_dead_identity_writes_nothing` |
| The removal checks no address before it removes | `test_destroying_at_an_address_outside_the_world_removes_nothing` |
| The removal answers with the size of the set rather than the count removed | `test_destroying_an_upgrade_returns_the_tile_to_the_generated_world` |
| The direction read ignores the faction argument | `test_the_return_field_leads_a_faction_to_its_own_site`, `test_the_return_direction_depends_on_the_faction`, `test_a_return_direction_outside_the_world_is_refused` |
| The tile report hides an unfinished build | `test_a_soldier_told_to_build_marks_the_ground`, `test_a_terrace_is_a_different_upgrade_from_a_road`, `test_stopping_a_build_keeps_the_work_already_done`, `test_destroying_an_upgrade_leaves_the_build_order_standing`, `test_the_road_and_the_terrace_carry_the_documented_numbers` |
| An unknown upgrade kind becomes a road rather than a refusal | `test_an_upgrade_kind_the_engine_does_not_hold_is_refused`, `test_the_build_order_takes_two_upgrade_kinds_and_refuses_the_third`, `test_a_resource_kind_that_overlaps_an_upgrade_kind_is_not_refused` |
| The read of a build order always reports nothing | `test_a_soldier_told_to_build_marks_the_ground`, `test_a_terrace_is_a_different_upgrade_from_a_road`, `test_stopping_a_build_that_names_a_dead_identity_writes_nothing`, `test_destroying_an_upgrade_leaves_the_build_order_standing`, `test_the_road_and_the_terrace_carry_the_documented_numbers` |

**The first defect is the one the task named.** A build order that names an
identity the world does not hold raises, and it leaves the world unchanged. With
the defect in place the call still raised, and the live unit in the same set
carried the order. The test caught the second half, which is the half that
matters.

**Every test starts at the Python boundary.** No test in this work constructs
the mechanism. The three methods shipped inert while their own Rust tests
passed, so a test that drove them again would have proved the same thing again.

**Two tests check what a value depends on rather than that it repeats.** One
gives two factions sites at opposite corners of one world and asserts that the
direction at the middle differs between them, so the faction is in the key. One
builds a terrace and asserts that the tile reports a terrace, so the kind
reaches the tile.

## 5. The gates

Every gate below was run on this branch. The results are the output of the run.

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Passes. No output. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passes. |
| `cargo test --workspace` | Passes. |
| `just determinism` | Passes. Both tests. |
| `just records` | Passes. 0 failures across every record check. Two pre-existing notes about records that nothing cites. |
| `just lint-python` | Passes. Ruff and mypy. |
| `just test-python` | Passes. |
| `just docs` | Passes. The site builds and every summary the import found reached a page. |
| `just docs-probe` | Passes. The documentation job fails when the import fails. |

The golden state hash did not move. The work adds no simulation code, and no
test needed the hash regenerated.

## 6. What is left undone

**The engine does not check who holds the ground.** The project orientation
states that a unit builds only on ground its own faction holds. Nothing checks
it, and a unit of one faction finishes a road on ground another faction holds. A
finding records the measurement, a decision records that the rule belongs in the
core rather than in a binding, and a backlog item holds the work.[^6] [^7] [^8]
The doc comment of the verb says what the engine does today rather than what the
project intends.

**The catalogue is still two kinds in a Rust enumeration.** A game cannot add a
third from Python. A separate backlog item owns that, and this work gives a game
the two kinds that exist.

**The upgrade kind is a fourth integer scale in this interface that carries the
name `kind`.** It overlaps the resource kinds and the ground kinds, and a range
check cannot separate two numberings that overlap. A caller that passes the
resource kind of food or wood gets a legal, wrong order. The doc comment says
so, and a test pins that sentence. A separate item owns the repair, and this
work did not attempt it.

**No core file changed.** No core method needed a change to be bindable. The
removal is the one place where the core answers two cases with one value, and
the binding works around it by checking the address itself. That is a small
duplication of the bounds rule, and a core method that told the two cases apart
would remove it.

**Nothing was skipped.** Every gate named above was run.

## References

[^1]: Backlog item 0341, bind the build order and the upgrade removal to the control plane. `docs/backlog/complete/0341-bind-the-build-order-and-the-upgrade-removal-to-the-control-plane.md`
[^2]: Findings register, FND-360. `docs/FINDINGS.md`
[^3]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^4]: Decisions register, DEC-160 and DEC-161. `docs/DECISIONS.md`
[^5]: Findings register, FND-381. `docs/FINDINGS.md`
[^6]: Findings register, FND-380. `docs/FINDINGS.md`
[^7]: Decisions register, DEC-161. `docs/DECISIONS.md`
[^8]: Backlog item 0370, refuse a build on ground another faction holds. `docs/backlog/proposed/0370-refuse-a-build-on-ground-another-faction-holds.md`
