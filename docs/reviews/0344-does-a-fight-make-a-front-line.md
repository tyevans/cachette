# Does a Fight Make a Front Line?

This review reports a measurement taken on 3 September 2026 for backlog item
0344.[^1] It writes no combat pass into the engine, and the engine holds none.

Cachette is a world simulation engine. It holds a hex world at two levels of
detail. Level 0 holds individual tiles and units. Level 1 summarises a block of
tiles, and one layout constant fixes the block edge at 32 tiles. So one level 1
cell covers 1024 tiles.

A design sketch for combat resolves a fight for each level 1 cell, as a small
table over unit types. A fight resolved for a whole block kills units across the
whole block, so the casualties may not form a front line. Two factions had never
been run into contact in this engine, so there was no evidence either way. One
blocker held that gap, and this measurement closes it.[^2]

**The answer in one line.** The band that holds the middle 90 percent of the
casualties is 1 tile wide at the tile and up to 30 tiles wide at the level 1
cell. A tile resolution makes a front line. A cell resolution makes a smear.

## 1. The impact review

**Governed by.** Four accepted records govern this work.

- **One binary gives one answer at any thread count.**[^3] The unit-to-tile
  bridge rebuilds in parallel, and every figure here is read through it. A test
  runs the same measurement with the barrier at 1 thread and at 12 threads and
  compares the casualty count, both percentiles, the furthest distance and the
  count of casualties away from an enemy. The two agree.
- **Simulated and aggregated state holds no floating point number.**[^4] Every
  number in the harness is an integer. The percentile is an index into a sorted
  vector of integer distances. The tank model is two folds over `i64`.
- **The unit-to-tile bridge is derived, and it rebuilds at the barrier.**[^5]
  The harness rebuilds the bridge after every placement and reads it after the
  rebuild, never during one.
- **Iteration order is explicit.**[^6] The harness groups units by an integer
  key and walks a sorted vector. No hash map reaches a result.

**Changes.** No record. **Creates.** No record.

**Blockers.** BLK-052 governed the work and it is now resolved.[^2] The work
opened BLK-080, which asks whether a running world ever produces a tile that
holds two factions.[^7]

**Precedent.** The testing rule says a fixture that models the typical case
supplies no extreme, and the findings register held two instances of that shape
before this one.[^8] This review holds a third, in section 4.

## 2. What is the engine and what is a model

**The engine supplies** the world, the terrain and its passability, the unit
arena, the faction column, the tile column, the unit-to-tile bridge, the block
layout that fixes the block edge, and the counter-based generator that keys the
draw. Every geometric figure in section 3 comes from those structures.

**The harness supplies** three things the engine does not hold.

1. **An approach rule.** A unit takes its direction from a field over cells, and
   the control plane can name no destination, so no engine call sends one army
   at another.[^9] The harness moves each unit one tile toward the enemy, unless
   the ground refuses it, the tile is at the capacity of its ground, or the unit
   already stands with an enemy.
2. **A provisional casualty rule.** Nothing kills a unit in a fight. The harness
   removes up to one unit from each side of each contested resolution unit in
   each frame. **This rule is thrown away.** It exists to place casualties on
   tiles so that a band can be measured.
3. **The selection of which units die.** The harness copies the shape of the
   rule the project already holds for a ration: one keyed draw serves a whole
   group, and the served set is the ordinals of the group rotated by a keyed
   offset.[^10]

**The tank test in section 5 is entirely a model.** A unit carries no type and
no strength, so the engine cannot express either side of it.

**The harness placed units on shared tiles through the placement call, which
skips the admission rule and skips the movement pass.** So the arrangements
measured here are ones the engine has never produced for itself. That is the
limit of this measurement, and section 7 states what follows from it.

## 3. The band, measured

**The machine.** An x86-64 development machine. The engine targets AWS Graviton
servers, and a development machine misleads on a timing. **No figure here is a
timing.** A band is a shape, and the same arithmetic on the same integers gives
the same band on any machine.

**The method.** Seed two factions on opposite sides of a world of 128 by 96
tiles. March them into contact. Resolve 24 frames. For each casualty, take the
distance in tiles to the nearest tile that holds an enemy. Report the band that
holds the middle 90 percent of those distances.

**The world.** The seed gives 10825 tiles that admit a unit and 1463 that do
not, so the contact line is ragged rather than straight. The harness asserts
that shape, so a change to the terrain generator fails the test rather than
moving a number quietly.

**Every run resolves twice, and the defect is put back on purpose.** The two
runs of one arrangement differ in the granularity and in nothing else.

| Arrangement | Grain | Casualties | Band, tiles | Furthest, tiles | Away from an enemy |
|---|---|---|---|---|---|
| wall | tile | 1978 | 1 | 0 | 0 of 1978 |
| wall | cell | 144 | 6 | 10 | 99 of 144 |
| blob | tile | 1344 | 1 | 0 | 0 of 1344 |
| blob | cell | 96 | 11 | 16 | 69 of 96 |
| skirmish | tile | 768 | 1 | 0 | 0 of 768 |
| skirmish | cell | 144 | 1 | 0 | 0 of 144 |
| corner | tile | 96 | 1 | 0 | 0 of 96 |
| corner | cell | 48 | 30 | 36 | 32 of 48 |

The `wall` arrangement is a deep line across the whole world, with the contact
column inside a block rather than on its edge. The `blob` arrangement is a
compact square that meets another near the corner of a block. The `skirmish`
arrangement is two tiles deep. The `corner` arrangement is one army that fills a
level 1 cell against an enemy at one corner of it.

**What counts as narrow.** A band is narrow when a watcher can name the line. A
band of 1 tile is a line. A band of 30 tiles is not, and the reason is not
opinion: the world is 128 tiles wide, so a 30-tile band is nearly a quarter of
the whole world, and it is close to the 32-tile block edge that the cell rule
inherits. **The threshold this review proposes is the block edge.** A band that
approaches the block edge means the casualties are spread over the resolution
unit rather than over the contact, and the picture then carries no information
about where the armies met.

**The distance is measured against the enemy that stands there, not against the
ground the enemy holds.** The territorial spread rule takes many frames to
follow an army, and the question is where the killing happens rather than where
the flag is.

**Two thirds of the casualties of a cell resolution stand on a tile that holds
no enemy.** That is the smear stated as a share rather than as a width, and it
is the figure a watcher meets first: a unit dies with no enemy on its tile.

**One figure exceeds the block edge.** The furthest casualty of the `corner`
arrangement stood 36 tiles from the nearest enemy, and the block edge is 32. The
diagonal of a block is longer than its side, so a cell resolution can kill
further than the block edge suggests.

## 4. The fixture is the finding

**The `skirmish` arrangement reports no smear at either granularity.** Its cell
band is 1 tile and no casualty stands away from an enemy. Two armies two tiles
deep have nothing behind the front, so a rule that may kill anything in the
block kills the same units a tile rule would.

**A single typical fixture would have closed this blocker with the wrong
answer.** That is the third local instance of the shape the testing rule names,
and the findings register now holds it.[^11]

## 5. The tank test

**The project owner's acceptance test is that one tank still kills four
bowmen.** The sketch that meets it applies a penetration threshold for each
attacker type before anything is aggregated, so a type that cannot penetrate
contributes exactly zero and a sum of zeroes stays zero.

**This section is a model.** The numbers below are chosen to state the shape.
They are not a balance table and no record should take one from here.

Give a bowman an effect of 10 and a tank a threshold of 100. Give the tank an
effect of 60 and a bowman a threshold of 20.

**With the threshold before the sum, the tank wins at every count.** The term
for one bowman is zero, and zero times any count is zero. The model asserts it
at 4, 40, 400, 4000 and one million bowmen. The tank penetrates a bowman, so it
kills the four.

**With the threshold after the sum, the tank loses at eleven bowmen.** Four
bowmen sum to 40 and do not reach 100. Eleven sum to 110 and do. So the
acceptance test survives one order of two operations and fails the other, and it
fails at a count a player reaches in an ordinary game.

**The two runs differ in the order of one operation and in nothing else.** That
is what makes the first result mean something.

## 6. Where the model gives a result a player would call wrong

The research report asked for this, and it matters more than a confirmation.[^12]

**First, the cliff, stated as a number.** A crowd of 1000 attackers whose effect
is 99 against a threshold of 100 does nothing at all. The same crowd at an
effect of 101 does everything. One point of upgrade turns a harmless army into a
lethal one, and no intermediate outcome exists. The consumption module met the
same shape and removed it with a per-unit accumulator, so the project has both a
precedent and a stated reason to dislike it. A decision row holds whether the
threshold is hard, and the project owner owns it.[^13]

**Second, and this one is new here: a packed army cannot be attacked at all.**
Ordinary ground holds 8 units, and the admission rule reads the capacity of the
ground and not the faction. A tile that already holds 8 units of one faction
offers no room, so no enemy can ever stand on it. A rule that fires only when
one tile holds two factions therefore never fires against an army that fills its
tiles. Two armies can stand on adjacent tiles for ever and nothing happens. **A
player would call that wrong at the first battle, and the fault is not the
threshold.** The findings register holds the reading, and a new decision row
holds the question it raises: does a contest read one tile, or a tile and its
six neighbours?[^14] [^15]

**Third, the tile bound is small.** Because ordinary ground holds 8 units, a
table over the units of one tile reads at most 8 units across every faction and
every type. That is cheap, which is the good half. The other half is that a
tile-granular fight cannot express a hundred against one, and the sketch's
picture of a powerful army overpowering things is a picture of exactly that.

**Fourth, a cell resolution kills a unit that no enemy can see.** Two thirds of
the casualties in section 3 stood on a tile that held no enemy, and one stood 36
tiles away. A watcher of a cell resolution sees units fall in the rear while the
front stands. This is the smear stated as a player would meet it.

## 7. What item 0345 should do differently

Item 0345 resolves a meeting between two factions.[^16] Five things follow from
this measurement.

1. **Resolve at the tile.** The decision row is closed on that option, and the
   band is the evidence.[^17] The unit-to-tile bridge already lists the units
   that stand on one tile and it rebuilds at every barrier, so the input exists
   and the item needs no new structure for it.
2. **Do not size a table for a cell.** The sketch named a count for each faction
   and each type at whatever granularity the fight uses, and called it the
   largest new structure. At the tile it is at most 8 rows, because that is the
   capacity of ordinary ground.
3. **Settle what a contest reads before writing the pass.** A same-tile rule
   never fires against a packed army. That is a game rule the project owner has
   not stated, and a new decision row holds it.[^15]
4. **Do not take the reachability of the case for granted.** The measurement
   placed units on shared tiles directly, and no engine pass has ever produced
   one. A new blocker holds that gap, and the item should either close it or
   state that it builds against an arrangement nobody has seen.[^7]
5. **Keep the provisional rule out.** The casualty rule in the harness exists to
   place casualties on tiles. It states no game rule, and item 0345 owes it
   nothing except the shape of the keyed draw, which comes from an existing
   record rather than from the harness.[^10]

## 8. The gates

Every gate ran on the x86-64 development machine on 3 September 2026.

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | Passes, no output |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` | Passes, no warning |
| Tests | `cargo test --workspace` | Passes, no failing test |
| Records | `just records` | Passes |

The new suite adds eight tests. They report the table of section 3, the shape of
the ground, the thread-count equivalence of the band, the agreement between the
bridge and the harness, the capacity of ordinary ground, and the three results
of section 5.

## References

[^1]: Backlog item 0344, measure whether a fight makes a front line. `docs/backlog/complete/0344-measure-whether-a-fight-makes-a-front-line.md`
[^2]: Blockers register, BLK-052. `docs/BLOCKERS.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D1. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: Blockers register, BLK-080. `docs/BLOCKERS.md`
[^8]: Testing rules, section 2a. `.claude/rules/testing.md`
[^9]: Findings register, FND-363. `docs/FINDINGS.md`
[^10]: ADR-0106, a cohort serves whole rations to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^11]: Findings register, FND-391. `docs/FINDINGS.md`
[^12]: Research report 21, what a god needs from this engine, section 4.1. `docs/research/reports/21-what-a-god-needs.md`
[^13]: Decisions register, DEC-145. `docs/DECISIONS.md`
[^14]: Findings register, FND-392. `docs/FINDINGS.md`
[^15]: Decisions register, DEC-170. `docs/DECISIONS.md`
[^16]: Backlog item 0345, resolve a meeting between two factions. `docs/backlog/proposed/0345-resolve-a-meeting-between-two-factions.md`
[^17]: Decisions register, DEC-144. `docs/DECISIONS.md`
