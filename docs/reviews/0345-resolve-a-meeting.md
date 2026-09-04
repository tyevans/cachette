# Review: resolve a meeting between two factions

This document reports the work of backlog items 0343 and 0345. Item 0343 gives
a unit a type that indexes a shared table. Item 0345 resolves a meeting between
two factions.

Both items are complete. Four decision records were written. Two decisions in
the register closed, one opened, and three findings opened.

**One number needs reconciling at the merge.** A parallel worker measured the
front line and recorded the same defect this work met: admission reads the
capacity of a tile and never the faction, so a full enemy tile cannot be
entered. That worker holds it as FND-392 on its own branch, and this work holds
it as FND-402, because a citation of a number this register did not hold fails
the citation check. The merge must fold the two rows into one.

## 1. The architectural impact review

The review was made before any code was written. It names each governing record
by number and by decision.

### 1.1 The records that govern item 0343

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0066 | D1 | A unit type is a column of the mobile shape, not a fifth shape. |
| ADR-0012 | D3 | The unit arena is a struct of arrays, so the type is one more column. |
| ADR-0014 | D1 | The column is indexed by the slot, and the arena never compacts. |
| ADR-0014 | D2 | Every read of a type resolves the identity first, and a dead identity resolves to nothing. |
| ADR-0002 | D1 | The attack and the armour are fixed-point values. No field is a floating point number. |
| ADR-0011 | — | The type is a newtype of one byte, so no other byte substitutes for it. |
| ADR-0001 | D4 | The type column and the whole table reach the state hash. |

### 1.2 The records that govern item 0345

| Record | Decision | How the work honours it |
|---|---|---|
| ADR-0001 | D3 | The pass runs a fixed amount of work. It holds no convergence test and no time budget. |
| ADR-0001 | D4 | The table, the type column and the fallen log all reach the state hash. |
| ADR-0002 | D1 | The harm is a 64-bit accumulator of fixed-point terms. Nothing is a floating point value. |
| ADR-0002 | D2 | The combine and the scale of the arithmetic module are the only arithmetic on the harm. |
| ADR-0002 | D3 | The attack is scaled by a whole count through the module operation for that. |
| ADR-0003 | D1 | Every draw is keyed on the system, the frame, the tile and the draw index. The contest owns a system identifier alone. |
| ADR-0004 | D1 | The deaths apply in one ascending scan of the unit slots. |
| ADR-0004 | D2 | The union of the marks and the sum of the harm are both order-free. |
| ADR-0006 | D1 | The fallen event is plain data with an explicit layout, declared padding and no boolean field. |
| ADR-0009 | D1 | The threads do not own disjoint slot ranges, so each takes its own plane and the planes join by union. |
| ADR-0018 | D2 | The pass reads the units of one tile from the derived structure, in one contiguous run. |
| ADR-0018 | D3 | The pass reads the structure that the barrier rebuilt, and the step rebuilds it again after the deaths. |
| ADR-0023 | D1 | The harm combines exactly, in any order. |
| ADR-0053 | D2 | A faction is a bit index below the ceiling, and the draw index packs it. |
| ADR-0056 | D3 | Admission reads the capacity and not the faction. This is why contact is adjacency. |
| ADR-0074 | D1 | A spawn may over-fill a tile, and the fixtures use that to reach a crowd. |
| ADR-0091 | D1 | The neighbour read is a pass over tile pairs, not a search from a unit. |
| ADR-0106 | D1 | A group serves whole outcomes to as many members as the quantity covers. |
| ADR-0106 | D2 | The subset is the ordinals of the group, rotated by a keyed offset, drawn again each frame. |
| ADR-0120 | D1 to D4 | The pass reads the shared table and holds no copy of a row. |

No record is contradicted. No record is superseded.

### 1.3 The records this work created

| Number | Claim |
|---|---|
| ADR-0120 | A unit carries a type, and the type is an index into a table the world is built with |
| ADR-0121 | A meeting between two factions resolves at the tile, never at a level 1 cell |
| ADR-0122 | An attacker whose attack does not exceed the defender's armour contributes exactly zero |
| ADR-0123 | Casualties are whole units served to a keyed subset, never a fraction of everybody |

Each is a separate claim that a reviewer could accept alone, which is why they
are four records and not one.[^1] Record 0123 governs determinism, so it needs
a record even where the rule looks obvious.[^1]

## 2. The table shape

The table is a dense array, indexed by the type. Its length never changes while
a world runs. The code declares the width once, and no record states the
number, because a number a content choice can move does not belong in a
record.[^1]

Each row holds two fields.

| Field | Meaning | Scale |
|---|---|---|
| `attack` | The harm one unit of the type delivers in one resolution | Fixed-point, one whole casualty is the unit |
| `armour` | The attack an attacker must exceed to reach a unit of the type | The same scale as the attack |

Both fields are in one scale, so the comparison between them is exact and the
two never mean different things. The writer refuses a negative value in either
field: a negative armour would sit below every attack, including no attack at
all, and a negative attack would heal.

**The unit carries the index and never a copy of a row.** A copy would be the
table in a second place, with nothing that fails when the two disagree.[^2]

The table is square in the type count, because the resolution reads it for each
ordered pair. The cost of one tile therefore follows the square of a small
fixed number and never the population of the tile.

## 3. How the threshold is applied

**The threshold applies for each attacker type, before anything is
aggregated.**

The pass counts the units of each faction and each type within reach of the
tile it is resolving. For each defender group it walks the attacker groups. For
each attacker group it asks one question: does the attack of the attacker type
exceed the armour of the defender type? The comparison is strict, so an attack
that equals the armour does not reach.

A group that does not reach adds nothing at all. A group that reaches adds its
attack scaled by its count, into a 64-bit accumulator.

**Zero is the identity of integer addition, so a sum of zeroes is zero at any
count.** That is the whole of the tank test. It needs no rate, no cap and no
balance figure, so no later measurement can weaken it.

The order is the decision. A pass that multiplied first and tested the product
against the armour would satisfy the test at four bowmen and fail it at ten
thousand. Section 6 puts that defect back and reports which tests caught it.

## 4. How the remainder is distributed

The harm against one defender group is a fixed-point quantity of whole units.

**The whole part is certain.** It is the harm shifted down by the fractional
bits of the scale, and it is capped at the number of defenders present, because
a tile cannot lose more units than it holds.

**The remainder takes one keyed draw.** The remainder is the low bits of the
same value, so the floor and the remainder cannot disagree. The pass draws one
value uniformly below the scale and compares it against the remainder. The
number of casualties therefore has the harm as its expected value exactly, and
no rounding rule holds that up.

**The units that fall are named by a keyed rotation.** Each defender holds an
ordinal: its place among the defenders of its own faction and its own type on
that tile, in ascending identity order. The pass draws one offset and a unit
falls when its ordinal, advanced by that offset and wrapped at the group size,
is below the number of casualties. A rotation is a bijection, so exactly as
many units fall as the harm paid for.

**Two draws serve a whole group, and no draw is taken for one unit.** The draw
index packs the faction and the type of the defender group, and never its
position inside the tile. A position depends on who else stands there, so an
index taken from it would change the draw when an unrelated unit arrived. A
unit test asserts that no two groups share a draw index at the faction ceiling
and the table width.

## 5. What contact means, and why it changed

**The brief said co-occupation. The work ships adjacency.**

A parallel measurement found that admission enforces the tile capacity by
reading the capacity and the standing count, and never the faction.[^3] An army
that fills a tile therefore cannot be entered by anybody. A resolution that
fired only when two factions stood on one tile would never fire against exactly
the case a fight is about, and an army that packed itself to capacity would be
unattackable.

Two repairs were possible.

**Rejected: admission gains a rule that an enemy may enter a full tile.** That
supersedes an accepted record, and it makes the capacity mean nothing at the
one moment it matters. The capacity exists to bound the crowd on a tile, and a
rule that suspends it for an enemy leaves the crowd unbounded in a fight.

**Chosen: contact is adjacency.** A unit reaches every unit of another faction
on its own tile and on the six tiles beside it. Nobody has to enter anything.
Two armies that face each other stand on neighbouring ground, which is the
ordinary reading of a meeting, and the rule still covers co-occupation, so a
founding or a spawn that puts two factions on one tile still fights.

**Adjacency at the tile level means the six hexagonal neighbours, and nothing
further.** A neighbour outside the extent contributes nothing, because the
world is a rhombus and it does not wrap.

**The iteration stays a pass over tiles.** The pass walks the derived unit
structure block by block, and the units of one tile lie in one contiguous run
of it. For each such run it reads the six neighbours of that tile, in the fixed
direction order the hexagonal geometry declares. Six is a property of the
geometry, so the neighbour read is a pass over tile pairs and not the per-unit
search the movement record forbids.

**Nothing about determinism changed.** The neighbour reads are immutable, the
marks go to a plane owned by one thread, and the planes join by a bitwise
union. The defenders of one resolution are the units of one tile, so every unit
is a defender exactly once and nothing is resolved twice.

Two tests hold the rule. One puts each side on its own tile, beside the other,
and asserts that the tank reaches the bowmen. One puts them two steps apart in
a world of three tiles and asserts that neither reaches the other.

## 6. The defects put back, and which test caught each

Each defect was compiled into the engine, the suite was run, and the defect was
removed again. A test that stayed green would have been measuring its fixture.

| The defect | Caught by |
|---|---|
| The threshold applies to the aggregate rather than to each attacker type | `one_tank_kills_four_bowmen_and_takes_nothing`, `ten_thousand_bowmen_lose_to_one_tank`, `a_unit_reaches_the_tile_beside_it` |
| The frame leaves the draw key | `the_frame_is_in_the_draw_key` |
| The tile leaves the draw key | `the_tile_is_in_the_draw_key` |
| A draw for each unit replaces the keyed rotation | `ten_thousand_bowmen_lose_to_one_tank`, `a_pair_that_both_reach_takes_losses_on_both_sides`, `the_frame_is_in_the_draw_key` |
| Contact is co-occupation alone, so the neighbour read is dropped | `a_unit_reaches_the_tile_beside_it` |

**Every defect was caught, and each by a test that names the behaviour it
protects.** The two key-field defects are the ones that matter most, because
both are deterministic: the thread-count test and the golden hash pass while
either is present, and only a test that changes one field of the key sees them.

## 7. Why the golden file moved

**Every stored golden hash moved, and two causes are responsible.**

**First, the unit arena gained a column.** The state hash reads the unit
columns as raw bytes, so a new column changes the hash of every world that
holds a unit. It changes the hash of a world that holds none as well, because
the hash also folds the shared type table, which every world now carries.

**Second, the step gained a pass.** A world whose table is filled and whose
factions meet now loses units, so its later frames differ in more than the
hash.

The files were recorded from the source with the recording switch, and the
difference was read before it was committed. The recorded sequence was then
checked at one, two, four and twelve threads and found identical at all four,
and equal to the stored file.

A contested scenario joined the golden suite. Without it no golden file would
have covered the resolution at all, and a file that cannot move is a guard that
has already stopped working. The scenario asserts that it ended somebody, so a
fixture that stopped reaching the case fails rather than recording a file that
covers nothing. A contested scenario joined the thread-count suite as well, at
three extents, and it compares the fallen log byte for byte.

## 8. The registers

| Register | What moved |
|---|---|
| Decisions | DEC-144 closed on the tile. DEC-145 closed on the hard threshold. DEC-180 opened, on whether the resolution runs on a schedule. |
| Findings | FND-400, FND-401 and FND-402 opened. |
| Blockers | None opened. BLK-052 stays open, and neither item waits on it any more. The allocation of `BLK-090` and `BLK-091` was not used, because this work found nothing that needs information the project does not have. |
| Registry | Four rows added, each at status `Draft`. Four rows added to the record priority index, each naming what a reviewer should test hardest. |

**No cost figure was recorded.** The target platform register holds a table of
per-stage figures for each measurement run it names, and this work took no
measurement. The new stage therefore appears in no table there, and the tables
that exist stay true, because each records a tree as it was at one moment.

## 9. The gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass |
| `cargo test --workspace` | Pass |
| Thread-count equivalence | Pass at one, two and twelve threads |
| Golden state hash | Pass |
| `just probe` | Pass |
| `just records` | Pass |
| `just lint-python` | Pass |
| `just test-python` | Pass |
| `just docs` | Pass |
| `just docs-probe` | Pass |

The commit body holds the command output.

## 10. What is left undone

**No binding reads the fallen log.** The engine writes one event for each unit
that fell, and the control plane cannot see it. A caller watches its population
fall and cannot see where or to what. Item 0390 holds it, and it sits beside the
item that holds the same gap for three other logs.

**No posture lets a unit refuse a fight.** A unit beside an enemy fights. The
decision that asks whether attack is a verb or a destination and a posture is
open, and this work did not close it.

**The resolution runs on every frame.** Two other passes that change state run
on a schedule. A decision was opened for it, with a recommendation to leave it
as it is until a measurement says the cost matters.

**The blocker on the width of the killing stays open.** It measures how wide
the casualty band is, and this work does not close it.

**The new pass has no perturbation of its own in the probe build.** The
existing perturbations already make the contested scenarios of both determinism
tests fail, so the new scenarios have a proven failure mode. Nothing perturbs
the ordering of the contest itself.

**No measurement of the cost of the new stage exists.** It is a stage of the
step and the stage table names it, so a benchmark run on the target platform
will price it. None was run here.

**The unit type does not reach the choice pass.** Every unit alive still shares
one weight profile, so two units of different types in one cell with the same
need choose alike. Item 0343 named that as a question and this work did not
answer it.

## References

[^1]: Decision Record Scope, sections 1, 2 and 4.1. `.claude/rules/adr-scope.md`
[^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: Findings register, FND-402. `docs/FINDINGS.md`
