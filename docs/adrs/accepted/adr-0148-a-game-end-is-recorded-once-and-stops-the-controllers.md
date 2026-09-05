# ADR-0148: A game end is recorded once and stops the controllers

## Context

Before this record no run of this engine ended. A watcher saw motion and saw
no game, because nothing in the world could win.[^1] The design names the win
paths: domination, territory at a tick limit, wealth or wonder, and renown.
Each path is a reader over aggregates the engine already keeps or that the
game layer adds.[^1] The design names every path at once. The code adds one
reader at a time, so at any moment some paths have a reader and some do not.

The question this record answers is what happens at the tick a reader fires.

**The shortest path stops the step.** The step returns a value that says the
game is over, and the caller stops calling it. This is what a game loop
usually does, and a contributor would reach for it first.

Three facts refuse that. **A step that refuses to run breaks the golden
hash.** The determinism tests step a world a fixed number of times and compare
the hash to a stored file.[^2] A step that stops early gives a hash of a
different frame, and a test that ran past a game end would compare frames that
do not exist. **A watcher wants to keep watching.** The demonstration draws a
world and the panels read it, and a world that froze at the game end shows a
still picture. **The end is a fact a later frame reads.** The controller reads
it to know whether to act. A fact a later frame reads is simulated state and
enters the hash.[^3]

**The second shortest path records the end on every tick that a reader fires.**
A faction that holds every seat holds them on the next tick too, so the record
would rewrite itself each frame, and a second reader that fired later would
overwrite the first winner.

## Decision

**The step writes one game end record, at the first tick a reader fires. After
that tick the controllers emit nothing and every other pass continues.**

### D1. The game end is one record of the winner, the path and the tick

The record holds the winning faction, the win path and the tick. It is
simulated state and enters the state hash.[^3] A world with no game end holds
an empty record, and the hash covers the empty record the same way.

The boundary exposes the record, or nothing while the record is empty. The
boundary also exposes the running value of a faction on each path that has a
reader, so a caller can watch a path approach its end.

### D2. The record is written once, and nothing rewrites it

The step checks the readers only while the record is empty. At the first tick a
reader fires, the step writes the record. After that tick no reader runs and
nothing writes the record again.

A reviewer finds a violation when a later tick can change the winner, the path
or the tick.

### D3. Every reader satisfies one constraint, and the readers run in a fixed order

Every reader, present or later, satisfies this constraint. It runs at the
controller stage, directly before the controller evaluates, and only while the
record is empty. It reads aggregates the engine already keeps, and it starts no
pass over the units or the tiles. A reader that totals a value totals it in a
64-bit accumulator.[^7] A tie between two factions on one reader resolves by
the lowest faction identifier.[^4] Every threshold a reader compares against is
a balance value and lives in the reference tables.[^5]

The readers run in the order domination, territory, wealth or wonder, renown.
Two readers that fire on one tick resolve by that order. A path that has no
reader yet is skipped, and the order of the others does not change.

**One reader exists today, and it is the territory reader.** It fires at the
tick limit, and the faction with the most held tiles wins. The held tile count
is a running total the engine already keeps.[^6] A path that gains a reader
later takes its place in the order above and changes nothing in this record.

A reviewer finds a violation when a reader runs at another stage, when a
reader starts a pass, or when a reader breaks a tie by anything other than the
lowest identifier.

### D4. After the game end the controllers emit nothing, and every other pass continues

The controller stage reads the record at its start and returns when the record
is not empty. Weather, wear, gathering, movement, the contest and every other
pass run as before. The world keeps stepping and the picture keeps moving.

A reviewer finds a violation when the step returns early, when a pass other
than the controller reads the record to decide whether to run, or when the
controller emits a command after the record is written.

## The alternatives this rejects

**Stopping the step.** Rejected because it breaks the golden hash test, because
it freezes the picture, and because the end would then be a return value and
not a fact the hash covers.[^2]

**Rewriting the record on every firing tick.** Rejected because a second
reader would overwrite the first winner, and because a fact that rewrites
itself each frame is a fact that a reader cannot trust at any frame.

**Stopping every pass and keeping the step.** Rejected because a world that
does nothing for the rest of a run costs a step and shows nothing. The
controllers are the one thing that drives toward an end, so they are the one
thing the end stops.

**A game end as an event only.** Rejected because an event is a fact of one
frame, and the controller needs the fact on every later frame. An event may
accompany the record, and the record is the thing that stops the controllers.

## Consequences

**A run has an end that a watcher can read, and the world outlives it.** The
demonstration names the winner and the path, and under a flag it stops
stepping. Without the flag it keeps drawing.

**The golden hash covers the record.** The record is simulated state, so a
change to its shape moves every stored hash. The commit that changes the shape
records the new hash.

**A second win is not recorded.** A faction that would have won on a later tick
by a different path wins nothing. That is the meaning of once.

**The order of the readers is a rule of the game and not a balance value.** A
game that wants renown to beat domination changes this record, not a table.

**A test that steps past the game end sees a world that still moves.** The
thread-count test and the golden hash test therefore run unchanged.[^2]

## References

[^1]: Design, the living world game layer, section 5. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Testing Rules, section 1. `.agents/rules/testing.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: Budgets and costs. `docs/reference/budgets.md`
[^6]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^7]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
