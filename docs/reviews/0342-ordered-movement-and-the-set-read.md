# Review: ordered movement and the set read

This document reports the work of two backlog items. The first lets the control
plane name the seed set of a strategy field. The second reads the units of a
faction as a set.[^1] [^2]

They are one piece of work from two sides. Every write verb at this boundary
takes a set, and no read produced one, so a caller who ordered a set somewhere
could not then ask where that set was without the loop the design rules
forbid.[^3]

## 1. The impact review

### 1.1 Which records govern the work

**ADR-0091 D1. Movement takes its direction from a per-cell field, never from a
per-unit search.**[^4] Honoured. A unit that the control plane sent somewhere
reads one entry of one plane, indexed by the level 1 cell it stands in. It reads
no neighbouring cell, it scores no neighbour, and it computes nothing from its
own address toward a destination. The unit column holds one small number, which
names the plane the unit obeys. It holds no address, no route and no distance.

**ADR-0091 D2. The field is a projection of level 0 and carries nothing between
frames.**[^5] Honoured. The destination field is derived again at every rebuild
of level 1, from the seed set and from the summaries the rebuild produced. It is
also derived when the seeds change, so no path leaves it stale.

**ADR-0091 D4 and D5. The tie-break, and a cell that admits nobody.**[^6]
Honoured without a second statement. The work made the relaxation shared, so the
destination field runs the same code that the return field runs.

**ADR-0091 D6. A direction the ground refuses falls back to a keyed draw, and it
never freezes a unit.**[^7] Honoured. Section 3 gives the four cases and the
evidence.

**ADR-0095 D1 and D3. A strategy arrives as a field over cells, and several
destinations seed one field.**[^8] Honoured. One call names a set of tiles, and
the engine seeds one plane at every one of them at once.

**ADR-0110 D2 and D5. The reach, and the fixed pass count.**[^9] Honoured and
shared. The destination field runs the same relaxation and the same pass count.

**ADR-0005 D1. A solver runs a fixed iteration count.**[^10] Honoured. The
derivation reads no residual and tests no convergence.

**ADR-0004 D1. Iteration order is explicit.**[^11] Honoured in three places. The
seed cells are sorted and each is held once. The derivation walks planes in
ascending order and cells in ascending order. The set read walks the arena in
slot order.

**ADR-0003 D1. Every random draw is keyed.**[^12] Honoured. The fall-back for a
sent unit is the draw the movement pass already takes, keyed on the system, the
frame, the entity and the draw index.

**ADR-0085 D3. The engine resolves an identity that Python hands back.**[^13]
Honoured, and it does not bind the set read. The set read takes a faction number
and hands back no identity, so every entry names a live soldier at the moment of
the call. It needs no validity mask and no sentinel. The singular read still
takes an identity and still refuses a dead one.

**ADR-0002 D1. No floating point in simulated state.**[^14] Honoured. Every
value the work added is an integer: the reach is a byte, the direction is a
byte, the plane number is a small integer, and a seed cell is an index.

**ADR-0022 D1. Level 0 is the only truth.**[^15] Honoured. The seed set enters
the state hash, because a later frame reads it. The field does not, because it
is derived from the seeds and from level 1.

### 1.2 Does the work contradict a record?

No. ADR-0110 fixes the seeds of the return field at the live sites of a faction,
and the work does not touch that field. It adds a second field beside it. The
record stays true.

### 1.3 Which records the work creates

**ADR-0125, the control plane names the seed set of a destination field.** It
is a draft and the registry holds the row. The scope rule gives three conditions
and all three hold.[^16] A contributor could reasonably give a unit a
destination it carries, which is the first thing anybody writes. Choosing that
costs more than changing it later, because it puts a second source of directions
in the engine and no gate would refuse it. The reasoning is not visible in the
code.

**The set read needed no record.** The third condition fails: the read takes no
identity, so the mask question that would need a record never arises, and the
records that exist decide the rest. The item states the judgement.

### 1.4 Blockers

**BLK-007 governs every cost figure.**[^17] The record holds no figure. The
number of destination planes is a parameter of the world, and the record says
so rather than choosing a value.

**No new blocker opened.** The allocation for this work reserved two numbers.
Neither was taken, because the work found no question that stopped it.

## 2. How the seed set reaches Rust

A caller passes two sequences and a number in one call.

```python
world.send_units_to(units, [(32, 32), (33, 32)], destination=0)
```

The units are the identities, or the NumPy array of `numpy.uint64` that the
spawn returned. The seeds are the tiles the caller wants the units at. The
destination is the number of the plane that carries the order.

The engine does four things and then returns.

1. It refuses a number that names no plane of this world.
2. It resolves every identity, and refuses the whole call if one is dead.
3. It takes the level 1 cell of each address, sorts the cells and holds each
   once. A seed outside the world refuses the whole call.
4. It writes the plane number into the column of every unit named, and derives
   the field.

**The set is all or nothing at every step.** Nothing changes until every check
has passed.

**The engine allocates no plane.** The caller names one, and a caller that names
it again replaces the seed set. That is how a caller moves a set on to a second
place, and it needs no second order. The register holds the reasoning.[^18]

The read side is one call.

```python
columns = world.faction_units(1)
columns["unit"]   # numpy.uint64, one entry for each live soldier
columns["tile"]   # numpy.uint32, the row-major tile index of each
```

## 3. What stops a unit freezing

**Four cases leave a sent unit with no direction.**

1. Its cell holds a seed. The unit arrived.
2. Its cell is further from every seed than the fixed pass count reaches.
3. Ground that admits nobody cuts its cell off from every seed.
4. The field gave a direction, and the ground under that one unit refuses it.

**All four take the keyed draw that the movement record already states.**[^7]
The draw is keyed on the frame, so a unit the draw refuses again takes a
different direction on the next frame.

Case 4 is the one that matters, because it is the shape that froze a unit
against a shoreline for ever.[^19] The cell, the plane and the direction all
hold from one frame to the next, so the refusal repeats exactly. A rule that
only stayed put would stop the unit for ever, not delay it by one frame.

**The order does not clear itself.** A unit that arrives keeps the order and
walks about inside the block it arrived in. The engine cannot tell that a unit
arrived, because the field is at the pitch of a block and the tile the caller
named is one tile of that block. The control plane reads where the set is and
stops the order.

## 4. Is exploration the same verb?

**The mechanism is the same. The verb is not, and this work does not build it.**
The register holds the row and the reasoning.[^20]

Three things separate them.

**The seed set of exploration is not a place a caller can name.** It is the
frontier between what a faction has observed and what it has not, and that
frontier moves on every frame as the units walk. A caller that named it would
read it back, derive the difference, and name it again on every frame. That is a
loop over the world in the control plane, which is the thing the design refuses.

**Exploration needs a model that does not exist.** The engine hides nothing
today. Every tile is readable by every caller and by every unit, so a god cannot
uncover what is not covered. A product record asks that a faction sees only what
its own units observe, and it is accepted and unbuilt.[^21]

**The seed set of exploration is per faction, and a destination plane is not.**
Two factions exploring at once need two frontiers. A destination plane is a
plane the caller names, and nothing in it is indexed by the faction.

**What follows.** When the observation store exists, exploration is a third
field over the same lattice, seeded by the engine at the frontier of each
faction and derived at the barrier beside the other two. It writes no relaxation
of its own, because this work made the relaxation shared.

## 5. The defects put back, and whether each was caught

Each defect was put back one at a time, the suite was run, and the defect was
then removed. A test with no proven failure mode is decoration.[^22]

### 5.1 Ordered movement

**Defect 1. The destination does not steer the step.** The branch that reads the
destination plane was replaced by the exit field of option zero, so a sent unit
took the direction its own option would have taken.

*Caught.* `a_sent_set_walks_to_the_place_the_caller_named` failed. No unit
reached the destination cell in four thousand frames. The other seven tests
passed, which is the point: only the arrival test can see this.

```
test a_sent_set_walks_to_the_place_the_caller_named ... FAILED
test result: FAILED. 7 passed; 1 failed
```

**Defect 2. The intent filter runs before the destination.** One line was added
that reads the intent and returns early, which is where the read sat before this
work.

*Caught, twice.*

```
test a_sent_unit_moves_before_it_has_chosen_anything ... FAILED
test a_sent_unit_beyond_the_reach_leaves_the_tile_it_started_on ... FAILED
test result: FAILED. 6 passed; 2 failed
```

**Defect 3. A sent unit with no direction stays put.** The fall-back to the
keyed draw was skipped when the cell held no direction.

*Caught.*

```
test a_sent_unit_beyond_the_reach_leaves_the_tile_it_started_on ... FAILED
test result: FAILED. 7 passed; 1 failed
```

**Defect 3b. A sent unit the ground refuses stays put.** The second fall-back
was skipped when the ground refused the direction the field gave. This is the
shape that froze a unit against a shoreline.

*Caught.*

```
test a_sent_unit_the_ground_refuses_leaves_the_tile_it_started_on ... FAILED
test result: FAILED. 7 passed; 1 failed
```

**Defect 4. The seed set keeps the order the caller named it in.** The sort and
the removal of a repeated cell were deleted.

*Caught.*

```
test the_order_the_caller_names_the_seeds_in_does_not_reach_the_field ... FAILED
test result: FAILED. 7 passed; 1 failed
```

### 5.2 The set read

**Defect 5. A dead slot enters the set.** The walk stopped testing whether a
slot is live.

*Caught.*

```
FAILED tests/test_public_api.py::test_a_dead_unit_leaves_the_set_read
assert 5 == (5 - 1)
1 failed, 102 passed
```

**Defect 6. The tile column stands for nothing.** Every entry of the tile column
was written as zero.

*Caught by the agreement test, which is the one that matters.*

```
FAILED tests/test_public_api.py::test_the_set_read_agrees_with_the_singular_read
assert 1 == 0
1 failed, 102 passed
```

### 5.3 What no defect proved

**The claim that the read is one crossing is structural, not measured.** The
test asserts that the result is two NumPy arrays of the declared element types,
and that neither holds Python objects. It does not count crossings, because
nothing in the module can count them. A reader who wants the cost should read
the research measurement of the loop this read replaces.


## 6. The gates

Every gate below ran on this branch, in this worktree.

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | passes, after one formatting pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | passes, no warning |
| `cargo test --workspace` | 82 test binaries, every one `ok`, no failure |
| Thread-count equivalence | `cargo test -p cachette-core --test thread_equivalence`: 14 passed |
| Golden state hash | `cargo test -p cachette-core --test golden_state_hash`: 2 passed |
| `just records` | 74 records, 26 product records, 237 backlog items, 412 register entries, 181 priority rows, 6545 citations, 709 files, 410 documents: 0 failures |
| `just lint-python` | ruff: all checks passed. mypy: no issues in 22 source files |
| `just test-python` | 103 passed |
| `just docs` | 64 members with prose, 0 without. Every summary reached the site |
| `just docs-probe` | both cases failed the job, as they must |

**The new tests.** Eight in the core, and eight in the Python suite.

```
test result: ok. 8 passed; 0 failed  (a_sent_set_walks_to_its_destination)
```

### 6.1 The golden files moved, and why

Every one of the eight golden files moved. **No behaviour changed in any of the
eight scenarios**, because no scenario sends a unit anywhere.

Two new bytes reach the hash. The unit arena writes the send column, which is
zero for every slot of every scenario. The world writes the length of the seed
set of each destination plane, which is zero for every plane. A hash covers the
bytes that decide a later frame, and both of these do, so both belong in it.

The files were recorded with the switch the test names, and the sequences were
then verified at three thread counts. The pass count of the test was moved to
one thread and to twelve threads in turn, and the stored files matched at both.
The recording thread count is unchanged in the tree.

```
CACHETTE_UPDATE_GOLDEN=1 cargo test -p cachette-core --test golden_state_hash
cargo test -p cachette-core --test golden_state_hash   # at 1, 4 and 12 threads
test result: ok. 2 passed; 0 failed
```


## 7. What is left undone

**A sent set is not promised to arrive.** A tile-level barrier stops it, and the
unit wanders beside the barrier rather than crossing it. This was found by the
work and it is not fixed by it. The finding holds the measurement, the record
states the consequence, the Python prose says it plainly, and a backlog row
holds the gap.[^23] [^24]

**Nothing reports that a set is stuck.** A caller sees movement and no progress,
and it has no read that says why. The same backlog row holds it.

**Two blocker numbers were reserved and neither was opened.** The work found no
question that stopped it.

**The record is a draft and it is not accepted.** An author may set `Draft`, and
only a reviewer may set anything beyond it. The decision record priority index
holds the row that says what a reviewer should test hardest.

**The demonstration does not use the verbs.** Nothing in the Python
demonstration sends a set anywhere or reads a faction as a set. The tests drive
both, and the tests are the only caller.


## References

[^1]: Backlog item 0342, let the control plane name the seed set of a strategy field. `docs/backlog/complete/0342-let-the-control-plane-name-the-seed-set-of-a-strategy-field.md`
[^2]: Backlog item 0346, read the units of a faction as a set. `docs/backlog/complete/0346-read-the-units-of-a-faction-as-a-set.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^5]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D2. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^6]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decisions D4 and D5. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^7]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^8]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decisions D1 and D3. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^9]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction, decisions D2 and D5. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^10]: ADR-0005, a solver runs a fixed iteration count, decision D1. `docs/adrs/accepted/adr-0005-a-solver-runs-a-fixed-iteration-count.md`
[^11]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^12]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^13]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^14]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^15]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^16]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^17]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^18]: Decisions register, DEC-190. `docs/DECISIONS.md`
[^19]: Findings register, FND-315. `docs/FINDINGS.md`
[^20]: Decisions register, DEC-191. `docs/DECISIONS.md`
[^21]: Research report 21, what a god needs from this engine, section 1.2. `docs/research/reports/21-what-a-god-needs.md`
[^22]: Testing rules, section 1. `.claude/rules/testing.md`
[^23]: Findings register, FND-411. `docs/FINDINGS.md`
[^24]: Backlog item 0401, decide how a sent unit gets around a barrier the field cannot see. `docs/backlog/proposed/0401-decide-how-a-sent-unit-gets-around-a-barrier.md`
