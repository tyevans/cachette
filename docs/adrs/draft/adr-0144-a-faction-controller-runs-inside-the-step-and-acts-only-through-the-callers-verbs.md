# ADR-0144: A faction controller runs inside the step and acts only through the caller's verbs

## Context

The demonstration steps a world and draws it. Nothing in that world wants
anything. A unit gathers where the choice pass sends it, no faction plans, and
no run ends. The design that adds a game layer puts a controller behind each
faction, so that a faction makes choices and the choices drive toward an
end.[^1]

The question this record answers is where that controller lives and how it
acts. Two shapes are available, and each has a shorter path than the one this
record takes.

**The controller could live in Python.** The control plane already holds the
verbs, and a Python loop that reads a few aggregates and calls a verb for each
faction is a small script. The project rejected a Python data plane for a
reason that a finding records: a rule with no mechanism lost to the first caller
who needed a read the engine did not have.[^2] A controller in Python would
read the world through the boundary on every tick, and the number of crossings
would follow the faction count and then the site count. The control plane rule
binds the direction of travel and the count of crossings, and a Python
controller breaks both.[^3] It would also run while the interpreter lock is
released for the whole step, which one record forbids.[^4]

**The controller could hold private verbs.** A controller inside the engine can
reach any field it likes. The shortest path to a working controller writes a
store, moves a unit or changes a holder directly, and never passes a verb. The
result is a capability that no caller invokes and no test starts from the
boundary, which is the defect shape this project meets most often.[^5] It also
splits the game into two rule sets. What a god may do and what the engine does
for a faction would differ, and nothing would fail when they differed.

The engine already fixes the shape of a step. The stages open in one fixed
order, each reads what the earlier ones settled, and every random draw takes
its key from the system, the tick, the entity and the draw index.[^6] [^7]

## Decision

**A faction controller is one system at one fixed stage of the step. It reads
aggregates the engine already exposes, and it acts only through the verbs a
Python caller can call.**

### D1. The controller is one system at one fixed stage, inside the core crate

The controller runs as a stage of the step, after the derived structures of
the frame are settled. It runs in the core crate, so no Python code runs while
it runs.[^4] It takes no thread count, because its cost follows the faction
count and never the population.

A reviewer finds a violation when a controller decision is taken in Python
between two steps, or when the controller runs at a place in the step where a
derived structure it reads is not yet rebuilt.

### D2. The controller emits commands only through verbs that a caller can also call

Every command the controller emits passes through the same code path a Python
caller takes. The controller writes no store, moves no unit and changes no
holder except through a verb.

**No verb exists for the controller alone.** A verb that only the controller
reached would be a capability that no caller invokes.[^5] Whatever the
controller can call, Python can call, and the same refusal rules apply to both.

A reviewer finds a violation when the controller calls a function that the
boundary does not expose, or when a verb checks who is calling it.

### D3. A command the verb refuses is dropped, and the refusal is counted

The controller does not retry a refused command in the same tick, and it does
not bypass the refusal. The refusal counts in the subsystem census, so a
controller that asks for the impossible is visible.

### D4. The controller runs a fixed evaluation count and draws from the keyed generator

For each faction the controller makes a fixed number of evaluations. Each
evaluation draws once, keyed on the controller system, the tick, the faction
and the draw index.[^7] The evaluation count is a balance value and lives in
the reference tables.[^8]

There is no convergence test and no time budget. A controller that ran until
it was satisfied would give a different answer under a different load.[^6]

### D5. Commands apply in an order the data fixes

The controller visits the factions in identifier order and pushes each command
to a list. The list is sorted by faction and then by draw index before any
command applies.[^9] The result never depends on the visit order or on any
thread.

### D6. A faction under external control receives no evaluation

Each faction carries one flag that says an external caller controls it. A
faction whose flag is set is skipped. Nothing in this work sets the flag. It
exists so that a later player hook has a place to stand, and so that a test
can prove the controller leaves such a faction alone.

## The alternatives this rejects

**A controller in Python.** Rejected because it breaks the control plane rule
in both of its checkable forms, and because it would run while the interpreter
is released.[^3] [^4]

**Private verbs for the controller.** Rejected because the result is a second
rule set that nothing tests from the boundary, and a capability that no caller
invokes.[^5]

**One controller per archetype.** Rejected because a named archetype is code
that multiplies with the vocabulary. One controller with a weight vector per
faction expresses the same variety as data.[^10]

**A controller that stops when the frame is out of time.** Rejected because a
time budget is a source of nondeterminism this project cannot recover
from.[^6]

## Consequences

**Every capability the controller needs is a capability a god has.** A game
that wants the engine to do something for a faction must first give a caller
the verb. That is a cost, and it is the cost that keeps one rule set.

**The controller cannot read the world.** It reads a bounded set of aggregates,
and a reading the engine does not expose is a reading the controller does not
have. Adding a reading is engine work.

**The controller cost follows the factions and the seats.** No term follows the
unit count or the tile count. The figure is derived and lives in the reference
tables until the target platform measures it.[^8] [^11]

**A refused command is a silent no.** The controller does not learn why. A
game that wants a smarter controller reads the census and changes the weights,
not the verbs.

## References

[^1]: Design, the living world game layer, section 1. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Findings register, FND-147. `docs/FINDINGS.md`
[^3]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^4]: ADR-0042, the interpreter is released for the whole step. `docs/adrs/draft/adr-0042-the-interpreter-is-released-for-the-whole-step.md`
[^5]: Recurring Defect Shapes, shape 3. `.agents/rules/recurring-defects.md`
[^6]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^8]: Budgets and costs. `docs/reference/budgets.md`
[^9]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^10]: Project orientation, the design principles. `CLAUDE.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
