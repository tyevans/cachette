# Review of item 0460, bind the character and lineage subsystem

This document reports the work of one backlog item. The item bound the
character tier, the record of descent and the deeds of a unit to the Python
control plane.[^1] The core is a Rust crate. The control plane is a Python
package that imports a compiled extension module.

The core held thirteen methods over this subsystem before this work. No line of
the bindings crate named one, and no Python line named one. The findings
register holds the search that measured it.[^2] That is the shape of a
capability that nothing invokes, which the project rules already name.[^3] It
is the largest instance of that shape found in this engine so far.

The consumer is a game in which a player, often a language model, directs a
group of simulated people. A language model reasons about a named person with
parents, a reputation and a history. It cannot reason about a count.

## 1. The impact review

The review was made before any code was written. The item was written into
`refined/` with the review in it, and the registry rows for the two product
records were allocated, before the work started.

### Records that govern the work

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0040 | D1 | Every read answers about a set or about one person in one crossing. Every write takes a set. No caller loops. |
| ADR-0043 | D1 | A character is not the mass tier, so a read about one person is allowed. A read that would be walked is not. |
| ADR-0046 | D1 | Every refusal raises a typed error under the one root class. |
| ADR-0085 | D1, D2, D3 | Every identity is a number the engine gave, that Python cannot build, and the engine resolves it against the generation before any write. |
| ADR-0014 | D2, D3 | A removed character never answers again, and the person made next in that slot does not answer to their identity. |
| ADR-0002 | D1 | The renown and the relation cross as raw Q16.16 integers. No float crosses. |
| ADR-0004 | D1 | Every answer states its order, and each order comes from the engine. |
| ADR-0054 | D1, D4 | A raised unit is not turned into a character. A character is created beside it. |
| ADR-0066 | D1 | The living character is one of the four fixed shapes, and the binding adds no fifth. |
| ADR-0078 | D1 | The record of descent outlives a character, and the answer says which people are alive. |
| ADR-0104 | D4 | The engine chooses who it raises. The caller sets the level and the schedule. |
| ADR-0107 | D2 | Every word of prose lives in the Rust doc comment. |

### Decision by decision, against the implementation

**ADR-0040 D1. The boundary carries an instruction and an answer, never the
population.** The population read answers once for the world or for one
faction. The lineage read answers the parents, every ancestor and every
descendant together. The relation read takes one subject and a set of others.
The deeds read and the character link read each take a set of units. Every
write takes a set. Honoured.

**ADR-0043 D1. The tier of a shape decides the shape of its interface.** A
character is not the mass tier, so the lineage read takes one person. That is
the same rule that allows the existing read of the tile of one soldier. The
relation read is set-valued anyway, because the number of candidates grows with
the population and the number of ancestors does not. Honoured.

**ADR-0046 D1. One root type holds every error.** A stale identity raises the
view error. A faction the world does not hold, a pair whose two parents are one
person, a set with no room, and a period of zero each raise the verb error.
Honoured.

**ADR-0085 D1, D2 and D3. An entity crosses as one opaque identity that the
engine resolves.** The core held no method that resolved a character identity,
so the work added one beside the two that already existed for a soldier and a
settlement. Every write resolves every identity into a list before it changes
anything. Honoured, with one qualification that section 5 states: an identity
does not name the arena that minted it, so the resolution refuses a stale
number and cannot refuse a number from another arena.

**ADR-0014 D2 and D3. An identity is a slot and a generation.** A removed
character is refused by every call. A test removes a parent and asserts that
the lineage of the child still names them, with a zero in the live column, and
that the identity itself is refused. Honoured.

**ADR-0002 D1. No floating point in simulated state.** The renown crosses as a
raw Q16.16 integer, and so does the relation. Both doc comments say the scale
and say which way to convert. A test pins one half as 32768. Honoured.

**ADR-0004 D1. Iteration order is explicit.** The population read walks the
arena in slot order. Each lineage group is in ascending birth order, and the
parent group is in role order. A test asserts that the ancestor birth orders
ascend. The determinism test runs the same fixture at 1, 2 and 12 threads and
compares the answers. Honoured.

**ADR-0054 D1 and D4. An entity declares its tier at creation and never
changes.** The character link read makes this visible: after a promotion the
unit still answers a tile read and the character answers a lineage read. A test
drives both. The doc comment says the unit is not turned into a character.
Honoured.

**ADR-0066 D1. Four fixed shapes.** The work adds no shape and no column. It
reads the columns the arena already keeps. Honoured.

**ADR-0078 D1. Descent is a bounded, append-only record.** The lineage answer
carries a live column for exactly this reason. The removal doc comment says the
record keeps the person. The creation checks the room in the record as well as
the room in the arena, because the record never releases a row and the arena
does. Honoured.

**ADR-0104 D4. The engine cuts the promotion at a rank.** The binding exposes
the level of deeds and the schedule, and it exposes nothing that names a unit
to raise. The doc comment says so in those words. Honoured.

**ADR-0107 D2. The prose lives in the Rust doc comment.** Every new member
carries its prose there. The type stub gained signatures and two typed
dictionaries, and the two dictionaries carry prose because the compiled module
does not provide them. Honoured.

**ADR-0001 D4. One binary gives one answer at any thread count.** The work adds
no simulation code and no parallel section. A test runs one fixture at 1, 2 and
12 threads and compares the population, the lineage and the state hash.
Honoured.

### Records changed or created

None. The work states no new constraint. Every claim it needs is already in the
records above, and a record that only described the binding would be a
description rather than a constraint.[^4] The shape choices went to the
decisions register instead.

### Product records

Two accepted records state the need behind the engine work, and neither was
ever reachable from a caller.[^5] [^6] Two shaped records were written for this
work: one for what a caller reads, one for what a caller writes.[^7] [^8] The
registry rows were allocated before the files were written.

### Blockers

BLK-007 governs every cost figure, and the work states none. BLK-004 answers
the ceiling on the living character population, so the work invents no number.
BLK-011 answers what a person raised from the ranks inherits, and the answer is
nothing, which is why a lineage answer with no ancestor is a real answer.

**The work opened one blocker.** BLK-150 asks what raises and lowers renown.
Two blocker numbers were allocated to this work and stay unused.

## 2. What the core actually does

Each statement below is a read of the core source, not a repeat of what another
document said about it.

**A character is a slot in its own arena.** It carries a faction, a birth tick,
a renown, a sex, and a pointer into the record of descent. It carries no tile
and no position. A removal releases every one of those columns.

**The record of descent is separate and append-only.** It holds one row for
every character the world has ever made. It never releases a row. It holds the
two parent edges, the child lists, the house, and the labels that make a
patrilineal test two integer comparisons.

**A row of the record names a person twice.** It holds the row number, which is
the birth order, and it holds the entity identity that the arena minted at the
birth. The second one survives the death of the person, because the engine
never reissues an identity.

**The relation is exact and it is bounded.** It is the coefficient of
relationship. Every step of the recursion halves a value, so no step rounds. A
parent and a child give one half, two children of one pair give one half, and a
person against themselves gives one.

**The relation only answers for two living characters.** It reads the descent
row that the arena slot points at, and only a living character has a slot. A
caller cannot ask how a living person is related to a dead ancestor.

**The birth verb reads no sex.** The two arguments are the mother role and the
father role. The engine puts the first identity in one and the second in the
other, and it tests neither against the sex column.

**No pass bears a child.** The engine raises people from the ranks and it never
states a birth. Every child in a run comes from a caller.

**No pass writes the renown and no pass reads it.** The creation sets it to
zero. One core method writes it, and before this work only tests called that
method.

**The promotion creates and never mutates.** It makes a character beside the
unit and links the two. The unit keeps its tile and keeps moving.

## 3. The shape chosen for each binding

| Member | Shape | Why |
|---|---|---|
| `characters(faction=None)` | Seven parallel columns over the living | The whole population in one crossing. The faction argument is a filter, not a second call. |
| `character_lineage(character)` | Three plain values and three groups of four parallel columns | One call answers the parents, every ancestor and every descendant. A caller cannot walk it, because there is nothing left to ask for. |
| `character_relations(subject, others)` | One subject, a set of others, one value for each | The number of candidates grows with the population. A pairwise call would cross once for each. |
| `unit_deeds(units)` | Set of unit identities, one value for each | The value is what makes a unit eligible, and a caller reads it over a set. |
| `unit_characters(units)` | Set of unit identities, one identity or zero for each | Zero means nobody, because the engine never issues zero. |
| `deed_threshold()` / `set_deed_threshold(threshold)` | One whole count | A content parameter that a game tunes. |
| `set_character_schedule(period, phase)` | Two counts of ticks | The same shape as the existing position schedule. |
| `create_characters(faction, count)` | One faction, a count, an identity for each | Everybody it makes founds a line, so no argument names a parent. |
| `bear_children(births)` | A set of parent pairs, an identity for each child | The set form is the only form. A pair is a set of one. |
| `remove_characters(characters)` | Set of identities, answers with nothing | The same shape as the existing soldier removal. |
| `set_character_renown(characters, renown)` | Set of identities and one value | Every set-valued write in this interface already takes one value for the set. |

**The lineage read is the design risk, and this is the answer to it.** A
relation query and an ancestor walk invite a caller to follow one edge at a
time across the boundary, which is the loop the control plane rule forbids. The
read answers with a whole structure and gives a caller nothing to follow. Each
of the three groups has the same four columns, so a reader learns the shape once
and applies it three times. A decision row holds the choice and what was
rejected.[^9]

**A person inside a lineage answer is named by the identity the arena minted,
never by the row number.** The minted identity names one person for ever, and
Python cannot build one. The row number is an integer that Python can build, so
publishing it as a handle would add a second kind of identity with weaker
properties than the first. The row number still crosses, as data that no call
accepts: it is what a caller sorts on and what names a house. A decision row
holds this.[^10]

**A live column comes with every group.** The record of descent outlives the
person, so most ancestors are gone. A caller must be able to tell a living
ancestor from a dead one without asking once for each, and asking once for each
is the loop the rule forbids.

**Every write is all or nothing.** The birth verb resolves every identity and
checks every pair before it bears anybody. The removal and the renown write
resolve the whole set before they touch anything. The creation checks the
faction and the room in both stores before it makes anybody. Four tests assert
that the state hash does not move across a refused write.

## 4. What a caller may not do

Each statement below is what the engine actually enforces, not what a reader
might hope.

**A caller cannot choose who the engine raises.** It sets the level of deeds
and it sets how often the engine looks. The engine collects the eligible units,
ranks them by a key vector of its own, and cuts the list at a budget. Nothing
in this interface names a unit to raise.

**A caller cannot set the budget.** The core holds it and the binding does not
expose it. A decision row says why.[^11]

**A caller cannot decide who bears a child by a rule.** It states each birth
itself. No pass in the engine states one, so a run with no caller has no
births at all.

**A caller cannot make the engine test the sex of a parent.** The two arguments
of a birth are roles. A game that wants a rule about sex reads the sex column
and applies the rule itself.

**A caller cannot set the sex of a person.** The engine draws it.

**A caller cannot ask how a living person is related to a dead ancestor.** The
engine cannot compute it, because the relation reads a row that only a living
character points at.

**A caller cannot ask whether a line has ended.** The engine answers that for a
person who is gone, and every read at the boundary takes a living identity. A
finding records the gap and a decision row holds the options.[^12] [^13]

**A caller cannot undo a birth.** Removing a child releases its slot. The record
of descent keeps its row, exactly as it keeps the row of anybody who dies.

**A caller cannot give somebody an ancestor they were not born with.** There is
no verb that adds an edge.

**Nothing in the engine reads the renown.** A caller writes it and reads it
back, and no simulation pass consumes it. The doc comment says this rather than
implying a mechanism that does not exist.

## 5. The defects put back, and what caught each one

Each defect was written into the bindings source, the extension was rebuilt and
installed, the tests were run, and the source was restored. The table states
the result of each run.

| Defect | Test that failed |
|---|---|
| The lineage read swaps the ancestors and the descendants | `test_a_descendant_read_answers_with_the_descendants_and_not_the_ancestors` |
| The birth verb bears each child as it resolves the pair | `test_bearing_a_child_of_a_dead_parent_bears_nobody` |
| The removal removes each character as it resolves it | `test_removing_a_set_that_names_a_dead_identity_removes_nobody` |
| The renown write writes each value as it resolves the identity | `test_writing_renown_to_a_set_that_names_a_dead_identity_writes_nothing` |
| The relation read answers in a different order from the set it was given | `test_the_relation_answer_follows_the_order_of_the_set` |
| The population read ignores the faction argument | `test_the_character_read_scopes_to_one_faction`, `test_a_child_takes_the_faction_of_its_mother` |
| The character link read always reports nobody | `test_a_raised_unit_stays_a_unit_and_names_its_character`, `test_a_unit_that_was_never_raised_names_no_character` |
| The creation swallows a refusal and answers with what it made | `test_making_people_in_a_faction_the_world_does_not_hold_makes_nobody` |
| The deed threshold write changes nothing | `test_the_deed_threshold_decides_whether_anybody_is_raised`, `test_the_threshold_reads_back_what_a_caller_wrote` |
| The schedule write changes nothing | `test_the_schedule_decides_which_frames_raise_somebody` |
| The relation read answers for a dead identity rather than raising | `test_a_relation_that_names_a_dead_identity_answers_nothing` |
| The parent group names every parent as the mother | `test_a_child_reads_both_parents_in_one_call`, `test_the_documented_numbers_hold` |
| The deeds read skips a dead identity rather than raising | `test_a_deeds_read_that_names_a_dead_identity_answers_nothing` |

**Every defect in the table was caught.** None passed.

**The first defect is the one the task named.** A descendant read that answers
with the ancestors gives a list of the right type and the wrong content, and no
type check sees it. The test builds two founders, a child and a grandchild, and
asserts both directions: the founder has the child and the grandchild below her
and nobody above, and the grandchild has all three above him and nobody below.
A swapped walk fails both halves.

**A fourteenth defect was put back and nothing failed.** The creation carried
its own check that the world holds the faction, before it made anybody. With
that check removed every test still passed, because the arena refuses the
faction itself and the undo removes what the call made, so the world is
unchanged either way. **The check was a second declaration site of one rule,
and the test proved it changed nothing.** It was removed rather than kept, which
is what this project's own rule says to do.[^15] The defect in the table above
replaces it: the creation now swallows the arena's refusal instead, and that is
caught.

**Two defects were reported as uncaught by an automated sweep and were caught
when run by hand.** The sweep restored the source and rewrote it inside one
second, and two of its cycles ran against a stale extension. Each of the two
was run again on its own, with the rebuild watched, and both failed the tests
named above. **A sweep that rebuilds is only evidence when the rebuild is
observed.** The two runs are the evidence here, not the sweep.

**One defect could not be put back cheaply.** The creation refuses a set that
the storage has no room for. Reaching that path needs a quarter of a million
characters, and no test in this work builds one. The path is therefore
unproven.

**Every test starts at the Python boundary.** No test in this work constructs
the mechanism. The whole subsystem shipped inert while its Rust tests passed, so
a test that drove it again would have proved the same thing again.

**Six tests check what a value depends on rather than that it repeats.** One
runs the same fixture with a low threshold and with a high one and asserts that
one raises somebody and the other raises nobody. One does the same with the
schedule. One asserts that the faction argument changes which people come back.
One asserts that the relation answer follows the order of the set it was given.
One asserts that a child takes the faction of its mother and not of its father.
One asserts that a removed parent still appears in the lineage of its child with
a zero in the live column.


## 6. The gates

Every gate below was run on this branch. The results are the output of the run.

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Passes. No output. |
| `uv run ruff format --check` | **Fails on two files this work did not touch.** Both are unchanged from the branch point, so the failure came in before this work started. Every file this work wrote or changed is formatted. |
| `just invariants` | Passes. The float ban and the crate split. |
| `just census` | Passes. 2 tests. |
| `just probe` | Passes. Every determinism test fails under the perturbed build, and the probe binary passes. |
| `just smoke` | Passes. The installed package builds a world and steps it. |
| `just records-probe` | Passes. Every record check rejects its broken fixture. |
| `just merge-defects` | Passes. 0 failures over the change on this branch. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passes. No warning. |
| `cargo test --workspace` | Passes. No test result line reported a failure. |
| `just determinism` | Passes. The thread-count test at 15 cases and the golden state test at 2. |
| `just records` | Passes. 0 failures across all eight record checks. Two standing notes about records that nothing cites. |
| `just lint-python` | Passes. Ruff reports no error and mypy checks 27 source files with no issue. |
| `just test-python` | Passes. 202 tests, 37 of them new. |
| `just docs` | Passes. The import found 106 members with prose and none without, and every one of the 106 summaries reached the site. |
| `just docs-probe` | Passes. The documentation job failed in both cases, as it must. |

**The golden state hash did not move.** The work adds no simulation code, and no
golden file was regenerated. The determinism tests read the stored hash and
passed against it.

**One gate is red, and this work did not make it red.** The Python formatter
reports two test files that need a blank line. Neither file is changed on this
branch, so both were red at the branch point. They were left alone rather than
reformatted, because other work is in them and a formatting change would
collide with it. Every file this work wrote or changed passes the formatter.

**The Python suite grew by 37 tests and the whole suite passes.** Two type stub
signatures were widened to the alias that the interface already uses for a set
of identities, because the new calls take a NumPy array exactly as the existing
ones do.


## 7. What is left undone

**The engine still bears no child on its own.** A run with no caller produces
no birth, so a population of characters never grows by itself. The two accepted
product records assume births happen. That is engine work and not boundary
work, and this item did not attempt it.

**Nothing reads the renown.** The column is now writable from the control
plane and no pass consumes it. BLK-150 holds the open question and the doc
comment of the write says plainly that nothing reads it.

**A caller cannot ask whether a line has ended.** Section 4 states it and a
finding records it. A backlog item holds the work.[^14]

**A character identity and a unit identity are the same number.** Section 4
does not state this as a limit of the caller, because it is a defect of the
interface rather than a rule. Neither call refuses the other's number and no
check reports it. The doc comments name the hazard and a test pins the
sentence. The same backlog item holds the repair, and the recommendation is to
put the arena into the identity.[^14]

**No promotion budget crosses.** A game that wants exactly one person raised in
a generation cannot say so.

**The house is a number and not a group read.** A caller can group people by
the house column itself. There is no call that answers who is in a house.

**Two core methods were added, and both are small.** The core held no way to
resolve a character identity, and a boundary that refuses a stale identity
cannot be written without one. The arena held no accessor for the generation of
a slot, which that resolution needs, and the same accessor already exists on
the soldier arena. One method was made public that was already written. No core
behaviour changed, and the golden state hash did not move.

**Nothing was skipped.** Every gate named above was run.

## References

[^1]: Backlog item 0460, bind the character and lineage subsystem to the control plane. `docs/backlog/complete/0460-bind-the-character-and-lineage-subsystem-to-the-control-plane.md`
[^2]: Findings register, FND-470. `docs/FINDINGS.md`
[^3]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^4]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^5]: PRD-0015, a unit has parents and children. `docs/product/accepted/prd-0015-a-unit-has-parents-and-children.md`
[^6]: PRD-0016, somebody is in charge. `docs/product/accepted/prd-0016-somebody-is-in-charge.md`
[^7]: PRD-0045, a god knows its congregation by name. `docs/product/shaped/prd-0045-a-god-knows-its-congregation-by-name.md`
[^8]: PRD-0046, a god raises somebody up. `docs/product/shaped/prd-0046-a-god-raises-somebody-up.md`
[^9]: Decisions register, DEC-260. `docs/DECISIONS.md`
[^10]: Decisions register, DEC-261. `docs/DECISIONS.md`
[^11]: Decisions register, DEC-264. `docs/DECISIONS.md`
[^12]: Findings register, FND-471. `docs/FINDINGS.md`
[^13]: Decisions register, DEC-265. `docs/DECISIONS.md`
[^14]: Backlog item 0461, tell a caller which arena an identity belongs to. `docs/backlog/proposed/0461-tell-a-caller-which-arena-an-identity-belongs-to.md`
[^15]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
