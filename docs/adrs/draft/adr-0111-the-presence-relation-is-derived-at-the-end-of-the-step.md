# ADR-0111: The presence relation is derived at the end of the step and never stored as a fact

## Context

A game asks one question of this engine more often than any other. Does any
unit of one faction stand on ground that another faction holds? A downstream
game gates a conversation between two players on that answer, and a product
record states the need.[^1]

The engine holds every part of the answer and joined none of it. A tile
carries one holder, and the holder is one faction or nobody.[^2] A unit carries
a faction and the tile it stands on. A running total says how much ground each
faction holds.[^3]

**The control plane could reach the answer only by walking the population.** It
reads the holder of one address at a time, and it cannot list the units of a
faction at all. So a caller had to keep every identity the engine handed back
and cross the boundary twice for each unit. The control plane rule forbids
that, and a research report measured what one of the two calls costs.[^4] [^5]

**A record already fixes the shape that a relation between factions takes.** It
says that such a relation is one mask row for each faction, never a field over
the world, and it says that the first relation built must take that shape.[^6]
This is that first relation.

Three things about the answer were still open, and each is a decision below.
Where the relation is derived, because the engine changes both the holders and
the population inside one step. Whether a faction standing on its own ground
counts. And what a caller meets when it asks after changing the population and
before stepping.

## Decision

### D1. The relation is derived, and the world stores no presence fact

**The world holds no column, no event and no accumulator that says a faction is
present.** The relation is folded from the unit columns and the holder column,
and the fold is the only writer.

The relation reaches no state hash, because it is not state. Level 0 is the
only truth and everything above it is derived, and this is derived.[^7]

The alternative is an incremental presence set that every rule which moves a
unit or changes a holder maintains. That is one fact declared in as many places
as there are such rules, and nothing fails when the copies disagree. It is the
defect shape this project meets most often.[^8]

### D2. The fold runs at the end of the step, after the last structural change
and after the holding spread

**A fold anywhere else answers for a world the step has already left.** The
step moves units, then spreads the holding, then ends starved units. A fold
before the spread reads the holders of the previous tick. A fold before the
reap names a unit the frame ended.

So the fold is the last stage of the step. A world that has never stepped
derives the relation once when it is built, so a caller that never stepped
meets an answer rather than a refusal.

### D3. A unit on ground its own faction holds sets no bit

**The relation holds no diagonal.** The question is whether the people of one
side stand on the ground of another side, and a side never asks it of
itself.[^1]

A caller that wants to know how much ground a faction holds reads the running
total, which reads no tile.[^3] Setting the diagonal would answer a different
question with the same word and would cost a reader the ability to test the
relation against a world in which everybody stands at home.

### D4. Every read that names a faction takes the arena and refuses a stale
answer

**A caller that spawns, despawns or moves a unit and then reads gets a refusal,
never an answer.** The relation records the arena it was derived from and the
revision of that arena. A read against a different arena, or against a changed
one, returns an error.

The identity matters as much as the revision. Two arenas of one extent, each
holding one unit, both sit at revision one, and a count alone would let a
relation answer about an arena it was never derived from.[^9]

The refusal reuses the error type of the unit-to-tile bridge. It is the same
question about the same arena, and a second error type stating the same three
failures would be one fact declared twice.[^8]

### D5. The fold combines by union, and the partition comes from the data

**The combine is a bitwise union of sets.** It is associative, commutative,
exact and has an identity, so the fold gives one answer whatever the partition
and whatever the thread count.[^10]

The fold still writes disjoint outputs and joins them in slot order, because
that is the parallel rule of this project and because a reviewer should not
have to prove commutativity again to accept a change.[^11] Each thread folds a
contiguous run of arena slots into its own row array. Nothing reads which
thread finished first. The partition is derived from the slot count and the
thread count, and never from the schedule.[^11]

### D6. The relation is exact, and never an over-approximation

**The fold reads the holder of the exact tile that each live unit stands on.**
No block mask, no summarised cell and no bounding shape reaches the answer. A
set bit therefore names a unit that is genuinely there, and a clear bit means
that no unit is.

The block masks make a query for the tiles of one faction cheap, and a fold
that used them instead would be an over-approximation.[^12] A game that gates
a conversation on this answer would then let a player speak when it should not,
which is a defect the player can see.

## Consequences

The engine cannot answer which units are present. The relation is one bit for
each ordered pair, so it names nobody. That answer is a set-valued read, and
the selector is what will hold it.[^13]

The engine cannot answer presence over a period. The relation states the world
at the end of the last step, and a game that wants a history keeps it.

The engine cannot express a relation between more factions than the mask is
wide, which is the consequence the faction record already carries.[^6]

The step gains one stage that follows the population. It reads three unit
columns and one tile column, and it allocates one row array for each thread.
Nothing here was measured, and one blocker governs every cost figure in this
project.[^14]

A caller that changes the population outside a step and then reads meets an
error. That is deliberate, and it is what stops a caller from taking a stale
answer for a fresh one.

The relation is a second statement of nothing. It derives from the columns each
time, so it cannot drift from them, and no check has to compare it against
them.

## References

[^1]: PRD-0031, a god knows whose ground its people stand on. `docs/product/shaped/prd-0031-a-god-knows-whose-ground-its-people-stand-on.md`
[^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^3]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D4. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^4]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^5]: Research report 21, what a god needs from this engine, section 2.2. `docs/research/reports/21-what-a-god-needs.md`
[^6]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^7]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^8]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^9]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^10]: ADR-0023, an aggregate combines exactly, in any order, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^11]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^12]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^13]: ADR-0051, a selector is a lazy expression tree that Rust evaluates. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^14]: Blockers register, BLK-007. `docs/BLOCKERS.md`
