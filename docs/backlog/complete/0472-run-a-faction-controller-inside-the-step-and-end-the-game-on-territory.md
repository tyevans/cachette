---
id: 0472
title: Run a faction controller inside the step and end the game on territory
status: complete
created: 2026-09-05
implements: [ADR-0144, ADR-0148, ADR-0040 D1, ADR-0040 D2, ADR-0003 D1, ADR-0004 D4, ADR-0001 D4, ADR-0053 D4, ADR-0075 D2, ADR-0076 D2]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-007]
---

## Why

**Nothing in the demonstration world wants anything.** A unit gathers where the
choice pass sends it. No faction plans, and no run ends. A watcher sees motion
and sees no game. This item is pass 1 of the living world game layer, and it
is the tracer bullet. After it, a world built from a seed runs, a faction
controller orders gathering and building, the game ends on the territory path
at a tick limit, and the demonstration prints what every subsystem produced.[^1]

The pass holds five things, and each is small on its own.

1. **The seeding layer moves into the engine at construction.** Founding and
   luxury placement are Python verbs today. After this pass the world founds
   its factions and places its luxuries when it is built. The demonstration
   calls no seeding verb.[^1]
2. **A controller stage opens directly after the presence fold.** It is one
   new variant of the stage enumeration, and the step register gains one row.
   It takes no thread count. It reads the aggregates the engine already
   exposes. It emits commands only through the verbs a Python caller uses, and
   in this pass those are `order_gather` and `order_build`. The set forms of
   the gather, build and type verbs move from the Python binding into the core
   crate, so one loop serves both callers.
3. **A seeded weight vector per faction biases the controller.** The vector
   holds four weights: war, trade, build and renown. Only the build weight is
   read in this pass. A per-faction externally-controlled flag exists as one
   `u8`. It is off by default, and it stops evaluation when set. Nothing sets
   it.
4. **The territory reader and the game end record.** At the tick limit the
   faction with the most held tiles wins. A tie resolves by the lowest faction
   identifier. The step writes the record (winner, path, tick) once. After it,
   the controller emits nothing and the world keeps stepping.
5. **The subsystem census.** A reader `subsystem_census()` returns one count
   per subsystem. The list derives from one Rust table. The demonstration
   prints it at its end. A gate asserts that every count is nonzero after a
   tick count in the balance table. This closes the questions of item 0278.[^2]

**This pass touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** Passes 3, 4 and 5 also touch the step, so those three wait until this
one merges.

## Impact review

**Governed by.** ADR-0144 holds that a faction controller runs inside the step
and acts only through the caller's verbs. ADR-0148 holds that a game end is
recorded once and stops the controllers. Both records are written beside this
item, and the registry holds their status.[^3] ADR-0040 D1 and D2 hold that the
boundary carries an instruction and never the population, and that the number
of crossings does not grow with the entity count. The controller crosses no
boundary, and the set verbs move into the core crate so the binding loops in
Rust. ADR-0003 D1 holds that every draw is keyed. The controller keys each
evaluation on (controller system, tick, faction, draw). ADR-0004 D4 holds that
a sort uses a stable key. The command list sorts by (faction, sequence) before
any command applies. ADR-0001 D4 holds that two tests protect the claim.
ADR-0053 D4 holds that what a faction holds is a running total. The territory
reader reads that total and starts no pass. ADR-0075 D2 and ADR-0076 D2 hold
how a founding draws and in which order a run founds. Moving the founding to
construction changes when it runs and nothing about how.

**Changes.** None. The founding records state how a founding chooses. They do
not state that a verb starts it. If the worker finds a decision that ties the
founding to a verb, a new record supersedes that decision. Do not edit it.

**Creates.** None. The two records this pass implements are allocated and are
being written. The census needs no record. The defect rule already states the
constraint, and the check that fails on a stale list is the reasoning.[^4]

**Blockers.** BLK-007 governs the cost of the stage.[^5] The cost shape is
evaluations per faction, multiplied by factions, multiplied by the bounded cost
of one reading. The figure stays derived until the target platform measures it.
The evaluation count, the tick limit, the weight range and the census tick
count are rows in the balance register, and every row is unset.[^6] Write the
pass against the row. Do not invent the value.

**Precedent.** FND-269 records that three completed subsystems produced nothing
in the demonstration world and nothing said so. The census is the general form
of the guard that finding asked for.[^7] FND-048 records that a determinism
test cannot see a broken invariant, so the keyed draw needs a test for each
field of its key.[^7] FND-051 records that a fixture chosen for realism hides
the defect it should show, so the census gate must not be the only fixture.[^7]
FND-320 records that nothing regenerates the type stub, so a new reader edits
the stub by hand in the same commit.[^7]

**Serves.** PRD-0048, a developer watches factions play a game to an end.[^8]

## Done when

- A world built from a seed holds its founded factions and its luxuries with
  no seeding verb called, and the demonstration calls none.
- The stage enumeration holds the controller stage directly after the presence
  fold, declared as taking no thread count, and the step register holds its
  row.
- The controller emits only `order_gather` and `order_build`, through the same
  core function the Python binding calls. A whole-tree search shows no verb
  that the controller alone reaches, and the search command is in the commit
  body.
- The core crate holds the set forms of the gather, build and type verbs, and
  the Python binding calls them. No loop over entities remains in the binding.
- A faction with the externally-controlled flag set receives no evaluation.
  A test proves it by setting the flag through a test-only path.
- Each evaluation draws once, keyed on (controller system, tick, faction,
  draw). One test per key field proves that a change to that field changes the
  draw: the same faction on two ticks, two factions on one tick, and two draws
  in one evaluation.
- The command list sorts by (faction, sequence) before any command applies. A
  test visits the factions in reverse order behind a test-only switch and gets
  the same event log.
- The territory reader fires at the tick limit, and the record holds (winner,
  path, tick). A test builds a world where two factions hold the same count and
  asserts that the lower identifier wins. The record is written once, and a
  second tick past the limit does not change it.
- After the game end the controller emits nothing. A test steps ten more ticks
  and asserts that the command count is zero and the gather count is not.
- `subsystem_census()` returns one count per row of one Rust table. A test
  removes a row behind a test-only switch and asserts that the label list and
  the reader then disagree and the check fails.
- The demonstration prints the census at its end. A gate drives the
  demonstration world for the census tick count from the balance register and
  asserts that every count is nonzero.
- The fixture for the census gate reaches an extreme: one faction with no
  units. The census still reports every count, and the domination reader of a
  later pass has a case.
- Each new test is proven able to fail. Put the defect back behind the switch,
  run the test, and record in the commit body that it went red.
- The thread-count test and the golden state hash test pass at 1, 2 and 12
  threads.
- The type stub `_core.pyi` is edited by hand in the same commit as each new
  reader and verb, because nothing regenerates it.
- `score(faction)` and `game_end()` exist in the bindings and in the stub.
- The whole check command runs green.

## Outcome

Built. A world seeds itself in one engine call that takes nothing: it founds
every faction with the default group and places the luxuries by a keyed draw
on the deposit index. The demonstration calls that one call and no other
seeding verb. The founding verb and the luxury verb still serve a caller that
wants its own.

The controller is one stage, the last of the step, declared as taking no
thread count. It visits the factions in identifier order, skips a faction under
external control and a faction with no seat, and makes a fixed number of
evaluations, each one draw keyed on the controller system, the tick, the
faction and the draw index. The plan is sorted by faction and then by sequence
before any command applies. Every command applies through the set form of the
gather verb or the build verb, and the Python binding calls the same set forms,
so no loop over entities remains in the binding. A command the verb refuses
outright is dropped and counted.

The seat is a decision this item made and the records do not hold. A faction
plans around the tile of its first founding, and a faction that founded nothing
receives no evaluation. Without it the controller ordered every unit of every
laboratory fixture and thirty tests measured the controller instead of the pass
they were written for. A reviewer should decide whether the seat belongs in the
controller record.

The territory reader fires at the tick limit, the faction with the most held
tiles wins, a tie goes to the lowest identifier, and the record of winner, path
and tick is written once and enters the hash. After it the controller emits
nothing and the world keeps stepping. The two parameters the step reads on
every tick, the evaluation count and the tick limit, enter the hash too.

The subsystem census is one Rust table of name and reader, and the Python
reader walks it. The demonstration prints the census at its end in every mode,
prints the winner once when the record first appears, and under a flag runs
headless to the end.

Left undone. The gate that drives the demonstration world for the census tick
count and asserts every count nonzero is not written, because two rows,
contracts and storms, need verbs this pass does not emit, so the gate would
fail on the fixture and not on the engine. The balance register still marks
every value unset, and the provisional defaults this pass wrote are in its
derivation column.

Every new test was put back to red once. The commit body of each test commit
names the defect and the test that caught it.

## References

[^1]: Design: the living world game layer, sections 1, 5, 10, 11 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/proposed/0278-say-what-the-demonstration-world-never-produced.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Recurring Defect Shapes, shape 1. `.agents/rules/recurring-defects.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Balance register. `docs/reference/balance.md`
[^7]: Findings register, FND-269, FND-048, FND-051 and FND-320. `docs/FINDINGS.md`
[^8]: Product registry. `docs/product/REGISTRY.md`
