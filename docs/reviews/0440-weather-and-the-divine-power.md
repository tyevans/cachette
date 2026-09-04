# Review 0440 — Weather, and the power of a god to inflict it on a place

This document records one unit of work. It states what was decided, what was
built, what was proved, and what was left undone.

## The impact review

**Which records govern this work.** ADR-0001 D1 and D4 require one answer at any
thread count and a hash of the whole state each frame. ADR-0002 D1 and D2 forbid
a floating point number in simulated state and route the arithmetic through one
module. ADR-0003 D1 requires a keyed draw with a system identifier of its own.
ADR-0004 D1 fixes every iteration order. ADR-0009 D1 and D2 require disjoint
parallel writes and a fixed join. ADR-0022 D1 and D2 make level 0 the only truth
and every level above it derived. ADR-0023 D1 and D2 require an exact combine.
ADR-0072 D5 is the pattern for a conserved quantity with a running account.
ADR-0073 D3 is where the gather rate is read. ADR-0087 D1 requires a fixed
iteration count in a field solve. ADR-0121 D1 and D2 decide the opposite
granularity for a fight.[^1] [^2] [^3] [^4] [^5] [^6] [^7] [^8] [^9] [^10]

**Does the work contradict a record.** No. ADR-0121 decides the granularity of a
fight, not of a field, and the new record states the argument rather than citing
that one for it.

**Does the work create a decision no record holds.** Yes, four of them, and all
four are deliverables of this work. They are drafts.

**Is the work blocked.** BLK-007 governs every cost figure, so this work states
shapes and no numbers.[^11] It found one new blocker and opened it.

**Has this been settled before.** FND-402 records that a fight at level 1
granularity smears.[^12] FND-051 and FND-048 record that a uniform fixture hides
a defect.[^13]

## The granularity, and why

**Weather lives on the level 1 cell lattice. A tile holds none.**

The engine decided the opposite for a fight three days before this work.[^9] A
meeting between two factions resolves at the tile, because a cell covers a block
of tiles and a fight resolved there kills units spread over all of them. A
measurement found that smear directly.[^12]

That argument does not carry over, and the record says so rather than copying
it. A fight is an event between two units standing on one tile, so a cell is
coarser than the thing being resolved. Weather varies slowly over distance, so a
cell samples a field rather than smearing distinct events. Two units in one cell
genuinely stand in the same weather, and two units in one cell do not genuinely
fight one fight.

Two other forces point the same way. A field at tile pitch costs the whole world
every frame, and the product record rejects that shape by name.[^14] The project
already solves a field at this pitch, and a second machine for one shape would
be two ways to do one thing.[^15]

The finding register holds the correction, because the project believed the
lattice smeared whatever it carried.[^16]

## The quantities

Weather is water, and the field holds two quantities over each level 1 cell.

- **The air.** The water standing above a cell, in drops. A drop is a whole
  number. It moves between neighbouring cells and it falls onto the ground.
- **The ground.** The water that has fallen onto the cell, in drops. This is the
  one quantity a simulation pass reads.

Two running totals sit beside the planes. One counts every drop that has ever
entered the air. The other counts every drop that has ever left the ground.

**There is no wind.** A wind plane would be a third quantity that only the
spread reads, and nothing in the engine reads a direction. The decision register
holds the choice and what would reopen it.[^17]

## What weather changes

**Wet ground yields more to a gatherer.** A unit that gathers from a tile inside
a cell whose ground holds at least a stated quantity of water takes a fixed
whole number more in one tick. The read is one lookup for the whole run of units
that gather one resource from one tile, beside the deposit read that the resolve
already makes.[^7]

The effect is an addition and never a factor. A multiplier would need a second
scale beside the whole numbers the resource ledger counts in.

**Nothing else reads the weather field.** Movement, consumption, the choice pass
and the contest are unchanged. That is stated rather than left implicit. The
decision register holds the three passes that could read weather, with a
recommendation for each: movement next and with a record of its own, the choice
pass after that, and consumption not at all, because the product record says
plainly that weather does not damage a unit.[^18]

## How weather evolves on its own

The engine calls one stage at the end of every step, after level 1 rebuilds. No
caller reaches it.

1. **The sea lifts.** Each cell takes one keyed draw. The odds follow the number
   of its tiles that admit no unit, which is its water share, so a cell of open
   water lifts often and an inland cell never lifts.
2. **The air spreads.** A fixed number of passes moves water between neighbours.
   Each pass is a gather: a cell hands each neighbour a truncated integer share
   of what it holds, and the receiver adds the same integer.
3. **The rain falls.** Part of the air lands on the ground of the same cell. The
   share rises with the mean height of the cell, so high ground takes more out of
   the air than low ground.
4. **The ground dries.** Part of the water on the ground leaves, and the running
   total counts exactly what left.

**The account is exact.** The air, the ground and the evaporated total sum to the
raised total at every moment. The world invariant check reads it. Conservation
and decay hold together because what leaves is counted, which is the pattern the
resource account already uses for the load of a dead unit.[^8] [^19]

**A dry world costs one draw for each cell.** A world with no open water and no
god allocates no plane and runs no pass.

## What bounds the divine power

A god names a set of places and a strength, and the engine answers once.

- **The ground.** The cell of every place must hold at least one tile of the
  god's own faction. This is the gate the project puts on speaking to another
  faction and on delivering against a contract, and a divine power that ignored
  it would be the one act that escaped the central mechanic.[^20] [^21]
- **The strength.** The god names a small whole number with a ceiling, never a
  quantity of water.
- **The breadth.** One call names at most a stated number of places.
- **The wait.** A successful call sets the first tick at which that faction may
  act again. A refused call does not, because a caller that mistyped an address
  should not lose the power.

**The call is all or nothing.** Every place is resolved, every gate is checked,
and the wait is checked, before anything is written. The cells are then sorted
and repeated cells removed, so the result does not depend on the order the
caller named them in.

The gate reads the holder mask of the block rather than the presence relation.
The presence relation answers whether one faction stands on the ground of
another, and it is empty on its own diagonal by design, so it cannot answer this
question.[^22]

**Every number above is a content constant that no measurement chose.** A new
blocker holds them.[^23]

## What a watcher reads

The published interface carries six reads and one verb. Every quantity is a
whole number of drops, and the doc comment says so at each one, because a value
that is secretly sixty-five thousand times its apparent size is a trap this
project has already documented.[^24]

- The water in the air above a place, and the water on its ground.
- Whether the ground of a place is wet.
- The totals for the whole world: air, ground, evaporated, raised, and the
  number of wet cells.
- The whole ground plane as one array, in cell index order, so a watcher draws
  the field in one crossing rather than one for each cell.
- The number of cells across the world, so a watcher turns an index into a
  column and a row.
- The verb that inflicts weather, and four readable bounds beside it.

**The viewer does not draw weather.** A watcher can read the field and cannot
see it on the map. The product record asks for both, so that half of one gate
statement is not met, and it is listed under what was left undone.

## The defects put back, and whether each was caught

Each row below is one edit to the source, one named test, and the result. The
tree was restored after each.

| Defect | Test | Result |
|---|---|---|
| A spread pass keeps what it gave away | `the_water_account_balances_at_every_frame` | Caught |
| The lift draw drops the frame from its key | `the_lift_draw_is_keyed_on_the_frame` | Caught |
| The lift draw drops the cell from its key | `the_lift_draw_is_keyed_on_the_cell` | Caught |
| A spread pass loses its halo at the thread boundary | `the_field_gives_one_answer_at_any_thread_count` | Caught |
| The solve stops on a convergence test | `the_solve_runs_a_fixed_number_of_passes` | Caught |
| Wet ground gives no bonus to a gatherer | `a_gatherer_on_ground_a_god_wet_takes_more` | Caught |
| The divine power drops its ground gate | `a_god_may_not_strike_ground_its_faction_does_not_hold` | Caught |
| The divine power drops its ground gate | `one_refusal_leaves_the_world_unchanged` | Caught |
| The verb writes as it resolves each place | `one_refusal_leaves_the_world_unchanged` | Caught |

**Every defect put back was caught.** None was missed.

Two of the nine are the reason the fixture changed. The ground gate test passed
against a twenty-four by twenty-four world for the wrong reason, because the
block edge is thirty-two tiles and a world that narrow is one level 1 cell. The
fixture moved to a sixty-four by sixty-four world that holds four cells and
still holds no open water. The finding register records it.[^25]

## Why the golden file moved

**Weather is simulated state, so the field enters the whole-world hash.** Two
worlds that hold the same tiles and different weather must diverge on the next
frame, and only the hash says so.[^1]

Every scenario moved, including the ones whose ground holds no water at all. The
field folds the faction count, the pass count, the two running totals and the
readiness of each faction into the hash before it folds the planes, and those
values exist in a world that never rains.

The files were recorded again from this source at four threads, and the whole
field was then compared at one, two and twelve threads with the air plane, the
ground plane and the state hash all equal.

## The gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Passes |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passes, no warning |
| `cargo test --workspace` | Passes |
| Thread-count equivalence | Passes, 15 tests |
| Golden state hash | Passes, 2 tests |
| The nondeterminism probe | Passes, 19 cases, every one still fails as it must |
| `just census` | Passes |
| `just records` | Passes, 0 failures, 2 pre-existing notes |
| `just records-probe` | Passes |
| `just merge-defects` | Passes |
| `just lint-python` | Passes |
| `just test-python` | Passes |
| `just smoke` | Passes |
| `just docs` | Passes, 105 members with prose, 0 without |
| `just docs-probe` | Passes, both cases fail the job as they must |

**One gate failed for a reason outside this work, and it is not fixed here.**
The Python formatting check reports two files that would be reformatted. Neither
file is touched by this work, and the check reports the same two on the branch
point. It is reported rather than repaired, because another worker owns those
files.

## What was left undone

**The viewer does not draw weather.** The product record asks that a watcher see
the condition on the map and tell it apart from the terrain beneath it. The read
exists and the drawing does not, because the viewer crate is not owned by this
work.

**The storage bound is looser than the product record asks for.** The record
asks that storage grow with the area the condition occupies. One drop of water
anywhere allocates the whole lattice. The lattice is smaller than the world by
the square of the block edge, so the shape is right and the bound is not tight. A
sparse field over an active set would be tight, and nothing has priced either.

**Weather does not enter the pyramid.** The product record says that whether an
aggregate carries the condition upward is a separate question. The field sits on
the level 1 lattice already, so nothing needed it.

**No cost figure was measured.** The stage is named in the stage cost table and
can be priced on the target platform, and nobody has done it. One blocker
governs every cost figure this project holds.[^11]

**A god cannot take weather away.** The decision register holds the question and
recommends a second verb.[^26]

**Three of the four records are unreviewed, and so is the fourth.** All four are
drafts, and the author of a record must not review it. The record priority index
names what a reviewer should test hardest in each.

## References

[^1]: ADR-0001, one binary gives one answer at any thread count, decisions D1 and D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^2]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^3]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^6]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^7]: ADR-0073, gathering is admitted by sort-then-admit against the tile, decision D3. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^8]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D5. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^9]: ADR-0121, a meeting between two factions resolves at the tile, decisions D1 and D2. `docs/adrs/draft/adr-0121-a-meeting-between-two-factions-resolves-at-the-tile.md`
[^10]: ADR-0023, an aggregate combines exactly, in any order, decisions D1 and D2. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-402. `docs/FINDINGS.md`
[^13]: Findings register, FND-051 and FND-048. `docs/FINDINGS.md`
[^14]: PRD-0004, the world has weather that a watcher can read, what it costs at the target scale. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^15]: ADR-0087, an influence solve runs a fixed iteration count over the whole plane, decision D1. `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md`
[^16]: Findings register, FND-450. `docs/FINDINGS.md`
[^17]: Decisions register, DEC-236. `docs/DECISIONS.md`
[^18]: Decisions register, DEC-237. `docs/DECISIONS.md`
[^19]: Findings register, FND-452. `docs/FINDINGS.md`
[^20]: ADR-0142, a god inflicts weather only on ground its own faction holds, decision D1. `docs/adrs/draft/adr-0142-a-god-inflicts-weather-only-on-ground-it-holds.md`
[^21]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D1. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^22]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^23]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^24]: Findings register, FND-341. `docs/FINDINGS.md`
[^25]: Findings register, FND-453. `docs/FINDINGS.md`
[^26]: Decisions register, DEC-238. `docs/DECISIONS.md`
