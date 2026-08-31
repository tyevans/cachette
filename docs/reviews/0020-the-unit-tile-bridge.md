# Review 0020 — The unit-to-tile bridge

## What was reviewed

Commit `7fe45d6`, on branch `feat/sprint-3-entities-move`.

| Path | State |
|---|---|
| `crates/cachette-core/src/bridge.rs` | New |
| `crates/cachette-core/src/world.rs` | Changed |
| `crates/cachette-core/src/soldier.rs` | Changed |
| `crates/cachette-core/src/lib.rs` | Changed |
| `crates/cachette-core/tests/unit_tile_bridge.rs` | New |
| `crates/cachette-core/tests/thread_equivalence.rs` | Changed |

The reviewer wrote none of this code. The reviewer ran `cargo test -p
cachette-core`. It passes: 17 tests in the bridge suite, and every other suite
green.

The line numbers below are the line numbers of the files at this commit.

## Verdict

**ACCEPT WITH AMENDMENT.**

The implementation is sound. The key derivation has one site, the sort is the
shared key vector sort, the tie-break is the whole identity, and the property
tests are strong. Three amendments hold: one hole in the staleness guard, one
class of reads that the guard does not cover, and one ordering claim that a
comment carries alone. None of the three is a defect in the answer the bridge
gives today. All three are defects in the mechanism that is supposed to stop a
later contributor from getting a wrong answer.

## Findings, ranked

### F1. The revision guard is defeated by a second arena. MEDIUM

`bridge.rs:526` `check_fresh` compares two things: the grid, and the revision
number. Neither names *which* arena the bridge was built from.

`on_tile`, `in_block`, `count_on_tile` and `check_invariants` all take
`arena: &SoldierArena` as a parameter, and all of them are public. Build a
bridge from arena A, then call `bridge.on_tile(&arena_b, address)` where B has
the same grid and has taken the same number of structural changes. The grid
matches. `built == arena_b.revision()` is true. The guard passes and the bridge
returns arena A's occupancy as if it were arena B's.

Two arenas of the same extent, each holding one soldier at a different tile,
both sit at revision 1. That is the whole reproduction. It needs no unsafe
code, no private field, and no unusual call order.

This is exactly the read the module doc at `bridge.rs:24-37` says is
"impossible to perform by accident". The second mechanism does not close it
either: the lifetime tie is `&'a self, arena: &'a SoldierArena`, which stops a
mutation of B while the range lives. It says nothing about whether B is the
arena the answer describes.

`World` is not exposed to this, because `World::soldiers_on` (`world.rs:308`)
always passes its own arena. The hole is in the crate interface, and the crate
interface is public.

**Amendment.** The revision counter must identify the arena, not only count its
changes. Give the arena an identifier assigned at construction and compare it
in `check_fresh`, or move the guard so that only `World` can reach a read.

### F2. Not every read is guarded. MEDIUM

The commit message says "every read returns a typed error when they disagree".
The module doc at `bridge.rs:30-33` says the same. Both are false.

These public methods take no arena and perform no freshness check:

| Method | Line |
|---|---|
| `UnitTileBridge::block_is_occupied` | `bridge.rs:423` |
| `UnitTileBridge::block_range` | `bridge.rs:434` |
| `UnitTileBridge::len` | `bridge.rs:309` |
| `UnitTileBridge::is_empty` | `bridge.rs:315` |
| `UnitTileBridge::check_structure` | `bridge.rs:547` |

`World::bridge()` (`world.rs:286`) hands out `&UnitTileBridge`, so any caller
reaches all five without touching the arena. A caller that spawns a soldier and
then asks `world.bridge().len()` gets the count from before the spawn, with no
error and no signal.

The sharpest case is D5 itself. D5 exists so that "a query that descends the
level of detail pyramid tests the bitplane and skips an empty block". That
query is `block_is_occupied`, and it is the unguarded one. The decision whose
whole purpose is to let a caller skip work is the decision whose read cannot
tell the caller the answer is stale.

**Amendment.** Either give these methods the arena argument and the freshness
check that the other reads have, or amend the module doc and the record so that
they claim what is true: the per-tile and per-block *unit* reads are guarded,
and the shape reads are not.

### F3. The ordering inside the barrier is enforced by a comment. MEDIUM

ADR-0018's consequence states that the rebuild runs after the structural apply,
and calls the ordering "a decision and not an implementation detail".

`world.rs:549-557` places the rebuild last in `step`, with a comment saying so.
The comment is honest — it says the stub applies no structural change yet, so
this call is the last thing in the step and a later apply goes above it.

But there is no structural apply in `step` at this commit. So the ordering the
record calls a decision is, in the code, an ordering between one operation and
nothing. Nothing fails if a later contributor adds a despawn apply below line
557. The record's central consequence — that every identity in the unit array
is live for the whole frame — is true today only because no entity dies during
a step at all.

This is defect shape 5: a record that describes an arrangement the code does
not yet enforce. It is also shape 1, one fact (the ordering) declared in a
record and in a comment, with no check that fails when they disagree.

**Amendment.** State in the review record, or in a backlog item, that the
ordering has no enforcement yet. When the structural apply lands, a test must
drive `step` through a despawn and assert that no dead identity reaches the
unit array. A comment is not the mechanism this project accepts for this class
of fact.

### F4. Rebuilding on every step is a decision no record holds. MEDIUM-LOW

`world.rs:557` rebuilds the whole bridge at the end of every step, whether or
not any soldier moved. `rebuild` re-reads the whole live population, allocates
two vectors, sorts, and clears and refills the ranges and the bitplane
(`bridge.rs:339-376`, `bridge.rs:386-410`).

ADR-0018 D3 says "the engine rebuilds the whole bridge once for each frame, at
the barrier". Read strictly, that sanctions the unconditional rebuild. But D3's
stated reason is the merge order of incremental writes, not the frequency. It
does not consider the case where the arena revision has not moved since the
last rebuild, which the bridge already tracks and could test in one comparison.

No figure appears here. BLK-007 governs cost figures and none was measured.
The finding is not that the rebuild is expensive. It is that "rebuild
unconditionally" and "rebuild when the revision moved" are two options, the
code chose one, and nothing records the choice. A future contributor could
reasonably choose otherwise, which is the first of the three tests for whether
a decision needs a record.

**Amendment.** Either add the revision comparison, or record the choice — a
line in the backlog item is enough if the project judges it below the threshold
for a record.

### F5. A move to the tile a soldier already stands on invalidates the bridge. LOW

`soldier.rs:400-414` `place` raises the revision on every successful call,
including one where `tile == self.tiles[slot]`. The bridge then reports `Stale`
for a frame in which nothing about the occupancy changed.

The failure is safe — a false `Stale` never returns a wrong answer. It is a
false alarm, and a caller that hits it has no way to distinguish it from a real
one.

### F6. The tie-break test relies on a population pattern that nothing asserts. LOW

`unit_tile_bridge.rs:90-112` `populate` despawns inside its loop so that a later
spawn takes a freed slot at generation two. The comment explains that this makes
two soldiers on one tile rank differently by slot index and by whole identity,
so that only the whole identity is a correct tie-break.

`the_bridge_is_identical_at_every_thread_count` asserts at line 422 that the
population shares a tile. Nothing asserts that the divergence between slot order
and identity order actually occurred. If a later edit to `populate` removed the
despawn, the test would still pass, and the D3 tie-break claim would no longer
be under test.

`a_per_tile_query_gives_what_a_scan_gives` (line 391) is the test that would
catch a slot-index tie-break, because `by_scan` sorts by `to_bits`. It carries
the same dependency on the pattern and the same missing assertion.

**Amendment.** Add an assertion in the same shape as line 422: that at least one
tile holds two soldiers whose slot order and identity order disagree.

### F7. Small items. LOW

- `bridge.rs:401` `if slot < self.ranges.len()` silently skips a block index the
  layout does not hold. The index is derived from an address the grid accepted,
  so the branch is unreachable. A silent skip is the wrong failure for an
  unreachable case; `check_structure`'s `covered == self.keys.len()` at line 589
  would catch the consequence one call later, which is a strange place to
  discover it.
- `bridge.rs:367-371` indexes `keys[item]` and `units[item]` with values from
  `sort::order_on` without a bound check. It is correct as long as `order_on`
  returns a permutation. That trust is not stated and not tested here.
- `world.rs:193` rebuilds the new world's bridge at a hardcoded one thread. The
  arena is empty so the result cannot differ, but it is a thread count declared
  in a second place.
- `bridge.rs:216` `block_count` multiplies `blocks_wide * blocks_high` as `u32`.
  The product is bounded above by the tile count, which is a `u32`, so it cannot
  overflow. Stated because the review looked for it.
- `soldier.rs` `revision` is `wrapping_add`. A collision needs 2^64 structural
  changes. Not reachable. Stated because section D asked.

## A. Record compliance, decision by decision

### ADR-0018

**D1 — the bridge is its own arrays and owns no unit. HELD.**
`UnitTileBridge` (`bridge.rs:269-281`) holds `keys`, `units`, `ranges` and
`occupancy`, and no unit. `rebuild` (`bridge.rs:339`) takes `arena:
&SoldierArena` by shared reference, so it cannot reorder the arena. The parallel
arrays are filled in one loop (`bridge.rs:367-371`), so nothing reorders one
without the other. `check_structure` (`bridge.rs:548`) fails when their lengths
disagree, and `a_short_unit_array_fails_the_structure_check` (`bridge.rs:683`)
proves the check fires.

**D2 — one derivation site for the key. HELD.** See section C.

**D3 — rebuild at the barrier by a sort on the key. HELD, with F3 against the
ordering consequence.** `rebuild` builds `SortKey::new([key, unit.to_bits()])`
(`bridge.rs:361`) and calls `sort::order_on` (`bridge.rs:364`). There is no
second sort and no comparator. The record says "radix sort"; the shared sort is
a chunked merge, not a radix sort. That is a discrepancy in the record's wording
rather than a violation — D3's binding content is that the sort is total, on the
integer key, with the identity as the tie-break, and it is. Nothing updates the
bridge incrementally: `rebuild` is the only method taking `&mut self`.

**D4 — read the block range, then search inside it. HELD.** `on_tile`
(`bridge.rs:456-482`) reads the range at line 475 and runs two
`partition_point` calls over `self.keys[start..end]` — the window is the block,
not the population. `in_block` (`bridge.rs:499`) serves the "many per-tile
answers within one block" case, and it returns a borrow rather than a stored
index, so nothing persists between frames.

**D5 — a bitplane marks each occupied block. HELD as data, weakened as an
interface.** `occupancy` is one bit per block (`bridge.rs:297`), set in
`rebuild_ranges` (`bridge.rs:406`), read by `block_is_occupied`
(`bridge.rs:423`), and `on_tile` short-circuits on it at line 472.
`the_bitplane_marks_the_occupied_blocks` (`unit_tile_bridge.rs:513`) checks it
against the population. F2 is the weakness: the pyramid-descent caller that D5
exists for reads it without a freshness guard.

### ADR-0007

**D1 — content supplies a key, never a comparison function. HELD.**
`bridge.rs:361` passes `SortKey<2>` of two `u64` fields. No closure, no
`sort_by`, no `Ord` impl on a domain type reaches the sort.

**D2 — the last key field is a stable identifier. HELD.** The last field is
`unit.to_bits()`, the whole identity as one integer (`types.rs:112`). The sort's
own `check_identifiers` rejects a duplicate last field, so a repeated identity
is an error rather than a tie. `check_structure` (`bridge.rs:561-567`) also
proves the order rises strictly on the pair.

**D3 — the engine never calls content code from inside a sort. HELD.** Every key
is materialised in the loop at `bridge.rs:352-362` before `order_on` is called.
The sort reaches back into nothing.

### ADR-0004

**D1 — iteration order is explicit. HELD.** The arena is read through
`arena.iter()` (`bridge.rs:352`), which walks the live column by slot index
(`soldier.rs:424-433`). No map, no set, no hash iteration anywhere in
`bridge.rs`. `rebuild_ranges` walks `self.keys` by position.

**D4 — a sort uses a stable key. HELD.** The key is `[block-major tile key,
identity]`. Both fields are exact integers. Neither depends on a pointer, an
address, or an allocation order.

### ADR-0014

**D1 — an identity is a slot index and a generation, held as one value. HELD.**
`bridge.rs:361` uses `to_bits()`, the whole value, never `index()` alone as a
sort field. `bridge.rs:353` uses `index()` only to subscript the tile column,
which is D7's dense array, and that is the correct use.

**D2 — resolving an identity can fail, and the caller must handle the failure.
HELD.** `check_invariants` calls `arena.contains(*unit)` at `bridge.rs:618` and
returns `Ok(false)` on a dead identity rather than assuming. The bridge stores
identities and never a raw slot, so no reader can skip the resolution.

### ADR-0002 and ADR-0001

No floating point appears in `bridge.rs`. No `f32`, `f64`, or float literal. The
key arithmetic is `u32` and `u64` shifts, masks and one multiply. ADR-0001 is
addressed under section E.

## B. The ordering consequence

Checked. The rebuild is the last statement of `step` (`world.rs:557`), so it is
after everything the step does. There is no structural apply in `step` at this
commit, so "after the structural apply" is vacuously true rather than enforced.
The record's consequence is not falsified. It is unenforced. See F3.

## C. The single key declaration site

Searched the whole `crates/` tree for `key_of`, `block_of_key`, `block_bits`,
`blocks_wide`, and for any shift-and-mask that reconstructs a block-major
ordering.

`BlockLayout::key_of` (`bridge.rs:234-242`) is the only derivation in non-test
code. Nothing stores the key on a unit or on a tile: `keys` holds the derived
value transiently for the frame, which is what D2 permits — it is not a second
declaration, because the rebuild recomputes it from the tile column every frame
and `check_invariants` (`bridge.rs:622`) compares the stored key against a fresh
`key_of` call.

`unit_tile_bridge.rs:494-495` recomputes a block column and row from `q` and `r`
by shifting. This is a second derivation, and it is the right kind: it is an
independent oracle in a test, deliberately not calling `key_of`, so that a
defect in `key_of` cannot hide behind itself. It would be a defect only if the
implementation depended on it. It does not.

D2 holds. This is the strongest part of the implementation as well as of the
record.

## D. The stale read

Both mechanisms were attacked.

**The lifetime tie holds.** `on_tile` and `in_block` return `&'a [Entity]` tied
to both `&'a self` and `arena: &'a SoldierArena` (`bridge.rs:456-460`,
`bridge.rs:499-503`). `spawn`, `despawn` and `place` all take `&mut self` on the
arena (`soldier.rs:261`, `soldier.rs:314`, `soldier.rs:400`), so the borrow
checker refuses any of them while a range is alive. The reviewer found no way
around this short of cloning the arena, and a clone is a different arena rather
than an invalidation.

**Every mutator raises the revision.** The three `&mut self` methods on
`SoldierArena` that change observable state are `spawn`, `despawn` and `place`,
and all three call `wrapping_add(1)` (`soldier.rs:282`, `soldier.rs:326`,
`soldier.rs:409`). `open_slot` is private and reached only from `spawn`. No
mutator escapes the counter. `place` raises it even on a no-op, which is F5.

**The counter cannot wrap in practice.** `u64` at one increment per structural
change. Not reachable.

**The guard is defeated by a second arena.** This is F1, and it is a real
sequence of public calls that returns a wrong answer with no error.

**Not every read is guarded.** This is F2.

## E. Determinism

**What fixes the order.** One sort, on `[block-major key, whole identity]`, with
the identity unique across live entities. The order is total by construction and
`check_structure` re-proves it strictly at `bridge.rs:561-567`.

**It is the shared sort and not a second one.** `sort::order_on`
(`bridge.rs:364`) is the crate's one sort. `bridge.rs` contains no `sort_by`,
`sort_unstable`, or comparator. `rebuild_ranges` does no sorting: it scans the
already-ordered keys.

**The tie-break is the whole identity, not the slot index.** `unit.to_bits()`
at `bridge.rs:361`, which carries the generation in the high 32 bits
(`types.rs:100-112`). Not `index()`.

**No hash iteration.** No `HashMap` or `HashSet` in `bridge.rs`.

**No thread completion order.** `sort::order_on` gathers its runs through
`Slots::combine`, which is the crate's one place that fixes slot order, and the
sort module documents that reading `entries()` directly would put it outside the
reach of the failure probe. `rebuild` itself starts no thread.

**Is thread-count equivalence tested, or asserted?** Tested.
`the_bridge_is_identical_at_every_thread_count` (`unit_tile_bridge.rs:416-450`)
rebuilds at 1, 2 and 12 threads and compares every tile's answer, every block
range, and every occupancy bit against the one-thread result. It runs and it
passes.

Two limits on that proof, both worth stating:

- The comparison is not literally byte-for-byte on the arrays. There is no
  public accessor for `keys` or `units`, so the test compares through `on_tile`,
  `block_range` and `block_is_occupied`. Together those cover the ordered unit
  slice for every tile and the whole range and bitplane arrays, so the coverage
  is complete in substance. The claim "byte-identical structure" is proven
  indirectly.
- The comparison drives `rebuild` directly, not `World::step`. `step` calls the
  same function, and the thread-count test in `thread_equivalence.rs` drives
  `step` at several thread counts — but the golden state hash does not cover the
  bridge, because the bridge is derived and `hash_into` reads the arena columns
  only. So the engine-level determinism test cannot see a bridge divergence.
  That is defensible: the bridge is a pure function of the columns the hash does
  cover, and `World::check_invariants` (`world.rs:492`) compares the bridge
  against those columns. Stated so that nobody later assumes the golden file
  guards the bridge. It does not.

## F. Test quality

Every test in `unit_tile_bridge.rs` uses only the public crate interface. It
reaches no private field. The unit tests inside `bridge.rs` do reach private
fields, and they say why at `bridge.rs:635-644`: the bridge is derived, so no
public call can make it disagree with the tile column, and a test of the
disagreement must write the arrays. That reasoning is correct and it is the same
route the arena tests took.

**Proptest persistence.** Present and correct. `unit_tile_bridge.rs:349-361`
carries the `proptest_config` block with
`FileFailurePersistence::Direct(concat!(env!("CARGO_MANIFEST_DIR"), ...))`, and
`crates/cachette-core/tests/unit_tile_bridge.proptest-regressions` exists and
holds five saved seeds — four of them shrunk to a non-zero count, so the file is
being written and read. FND-044 does not recur here.

**Tests that cannot fail.** The reviewer looked for a test whose assertions hold
regardless of the implementation, and found none outright. Three are weaker than
they look:

- `the_bridge_holds_exactly_the_live_soldiers` at `count = 0` asserts on an
  empty bridge and proves nothing. The range is `0..120`, so the shrink target
  is a vacuous case. Not a defect — the interesting cases are in the range — but
  a reader should not read a `shrinks to count = 0` line in the regressions file
  as evidence of a real minimal failure.
- `a_per_tile_query_gives_what_a_scan_gives` and
  `the_bridge_is_identical_at_every_thread_count` depend on `populate` producing
  a slot-order and identity-order divergence, and nothing asserts it did. This
  is F6.
- `a_bridge_that_was_never_built_refuses_every_read`
  (`unit_tile_bridge.rs:115`) checks `on_tile` and `check_invariants` only. It
  does not check `in_block`, and it cannot check `block_is_occupied` or `len`,
  because those have no guard to check. That gap is F2 seen from the test side.

**Tests that earn their place.** `a_read_after_a_move_is_refused_rather_than
_answered` (line 129) is the strongest: it proves the stale read is refused, and
then proves the rebuild makes both the old tile and the new tile answer
correctly, so it cannot pass on a bridge that simply refuses everything.
`the_keys_of_one_block_hold_one_run_of_the_key_space` (line 224) proves the
block-major property directly, over every tile of a world whose width is
deliberately not a multiple of the block edge (line 34-37). `the_world_answers_a
_tile_after_the_step_rebuilds_the_bridge` (line 287) drives the engine and then
inspects, which is the rule that FND-041 exists for, and it asserts the bridge
is stale before the step — so it proves the step is what makes the bridge
readable, not merely that a fresh bridge answers.

The `bridge.rs` unit tests each kill a specific mutation:
`a_key_that_disagrees_with_the_tile_column_fails_the_check` (line 672) is
carefully built so that `check_structure` still passes and only the column
comparison catches it, which is the right way to prove that the column
comparison is load-bearing.

## G. The five recurring defect shapes

**1. Redundant declaration sites with undocumented precedence.** The bridge is a
second declaration of where a soldier stands. Something does fail when it
disagrees with the arena tile column: `UnitTileBridge::check_invariants`
(`bridge.rs:608-630`) compares `key_of(column[unit.index()])` against the stored
key for every unit, and compares the counts. `World::check_invariants` calls it
(`world.rs:492`). It is tested three ways: the unit test at `bridge.rs:672`
(a key that disagrees), the unit test at `bridge.rs:700` (a lost unit), and the
property `the_bridge_holds_exactly_the_live_soldiers`. This is done properly and
it is the best answer to shape 1 in the commit.

One qualification. `World::check_invariants` swallows `Err(Stale)` at
`world.rs:498`. That is the right call — a stale bridge cannot be compared
against columns it was not derived from — but it means the cross-check is
silently skipped whenever the bridge is stale, which is the whole time between a
spawn and the next barrier. Nothing warns that the strongest check did not run.

The other instance of shape 1 is F1 and F2: the revision is a second declaration
of arena freshness that does not identify the arena, and five reads bypass it.

**2. Documents that rot when a sweep names specifics.** `bridge.rs` holds no
count, no file table, and no measured figure. `BLOCK_BITS_DEFAULT`
(`bridge.rs:81`) is a structural parameter with the reason and the open record
named, not a budget. `BLOCK_BITS_CEILING` (`bridge.rs:66`) documents itself as
the range of the index and not a budget. Clean.

**3. Inert code that nothing invokes.** `in_block` (`bridge.rs:499`) has no
caller in non-test code. D4 sanctions it — it is the interface for the "many
per-tile answers within one block" case — but no engine system calls it yet, and
`World` does not expose it. This is the mildest form of shape 3: a public
library capability rather than an engine obligation. It has a property test at
`unit_tile_bridge.rs:480`. Noted, not raised as a finding.

The bridge itself is not inert: `World::step` calls `rebuild` and
`the_world_answers_a_tile_after_the_step_rebuilds_the_bridge` drives the engine.

**4. Nondeterminism the tests cannot see.** Section E. The one gap is that the
golden state hash does not cover the bridge, which is correct for derived state
but should not be mistaken for coverage.

**5. A record that no longer describes the code.** Two small divergences.
ADR-0018 D3 says the sort is "a radix sort on the integer key"; the crate's sort
is a chunked merge. The record's binding content survives, but the wording names
an algorithm the code does not use. And the ordering consequence is F3.

## H. Error paths and panics

The reviewer searched `bridge.rs` for every panic route in non-test code.

- No `unwrap`. No `panic!`. No `expect` outside `#[cfg(test)]`.
- `key_of` (`bridge.rs:234`) returns `Option` and propagates `address_of`'s
  `None`. The `address.q as u32` and `address.r as u32` casts at lines 236-237
  cannot truncate: `Grid::contains` (`hex.rs:177-182`) rejects a negative
  coordinate, and `address_of` only produces addresses inside the grid.
- The shift-and-mask at `bridge.rs:239-241` cannot overflow. `block_bits` is at
  most 15, so `2 * block_bits` is at most 30, and the shift is on a `u64` from a
  `u32` block index. The largest key is under 2^62.
- `block_is_occupied` (`bridge.rs:423`) bounds-checks against `ranges.len()`
  before subscripting `occupancy`, and the two vectors are sized together in
  `new` (`bridge.rs:296-297`).
- `block_range` uses `get`, not a subscript.
- `on_tile`'s subscript `self.ranges[block as usize]` at line 475 is guarded by
  the `block_is_occupied` early return at line 472, which does the bounds check.
- `rebuild_ranges` casts `position as u32` and `(end - position) as u32`
  (lines 403-404). Neither can truncate, because `order_on` returns
  `SortError::TooManyItems` above `u32::MAX`.
- `rebuild`'s subscript `column[unit.index() as usize]` (line 353) is safe:
  `index()` comes from the arena's own live column and the tile column has one
  entry per slot.
- The remaining subscripts, `keys[item]` and `units[item]` at lines 369-370, are
  the one place that trusts an external result without a check. See F7.

`SoldierArena` changes: `despawn`'s new liveness guard (`soldier.rs:319-321`)
closes the `live_count -= 1` underflow, and its unit test at `soldier.rs:573`
constructs the state the public interface cannot reach and asserts the count
does not wrap. `place`'s reordering (`soldier.rs:401-411`) is correct and does
not change any error the caller can already handle.

No reachable panic found in non-test code.

## I. The interaction with sprint 3's other work

`World::step` rebuilds the bridge on every frame regardless of whether any
soldier moved (`world.rs:557`). No cost figure appears in this review; BLK-007
governs those and none was measured.

There is a correctness reason the rebuild must happen at the barrier when the
population changed: D3's whole argument. There is no correctness reason it must
happen when the population did not change. The arena already tracks that in
`revision()`, and `self.built == Some(self.soldiers.revision())` is the test.

So the unconditional rebuild is an unexamined default rather than a derived
consequence. That is F4. It is a decision no record holds.

Two smaller interactions:

- `World::new` (`world.rs:193`) builds and rebuilds the bridge before returning,
  so a freshly built world answers rather than returning `NeverBuilt`. Good, and
  `a_world_that_rebuilds_outside_a_step_answers_again` covers the manual path.
- `World::spawn_soldier` (`world.rs:240-252`) now rejects a faction the world
  does not hold. That repair is correct and the `thread_equivalence.rs` change
  at line 127 was necessary to make the existing test honest. The commit is
  right that the suite had been demonstrating the defect and passing.

## Objections attempted that did not hold

Stated so that a later reader knows they were tried.

- **Can two blocks produce one key range that overlaps?** No.
  `the_keys_of_one_block_hold_one_run_of_the_key_space` proves it over every
  tile of a world with a short last block column, and the derivation makes it
  structural: the block index occupies the high bits above `2 * block_bits`.
- **Can `rebuild_ranges` write two ranges for one block?** No. The keys are
  sorted, so the block index is monotone along `keys`, so each block appears in
  at most one run. If it ever did, `check_structure`'s `covered == keys.len()`
  would fail.
- **Can `on_tile` return units from an adjacent tile of the same block?** No.
  The two `partition_point` calls bracket the exact key, and the block-major key
  is unique per tile — proven at `unit_tile_bridge.rs:243`.
- **Does the bridge reorder the arena?** No. `rebuild` takes `&SoldierArena`.
  The compiler enforces it.
- **Does the revision counter reach the state hash and break determinism?** No.
  `hash_into` covers the tiles, factions, generations, live column and free
  queue. The revision is excluded, and `soldier.rs:140-153` documents why: it
  decides no later frame. Correct.
- **Can a caller hold a range across a rebuild?** No. `rebuild` takes
  `&mut self` on the bridge, and the range borrows `&self`.
- **Is `BLOCK_BITS_DEFAULT` a budget in disguise?** No. It is a parameter with
  the governing open record cited, which is what section 4.5 of the record scope
  rule requires.

## Amendments required

1. `bridge.rs:526` — make `check_fresh` identify the arena, not only its
   revision. (F1)
2. `bridge.rs:24-37` and the commit's claim — either guard the five unguarded
   reads, or correct the claim that every read is guarded. (F2)
3. `world.rs:549-557` — record that the barrier ordering has no enforcement
   until the structural apply exists, and state the test that must land with it.
   (F3)
4. Record the rebuild-always choice, or make it conditional on the revision.
   (F4)
5. `unit_tile_bridge.rs:90-112` — assert that the population produced a
   slot-order and identity-order divergence. (F6)

## References

[^1]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^2]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^5]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^6]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^7]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^8]: Recurring defect shapes. `.claude/rules/recurring-defects.md`
[^9]: Testing Rules. `.claude/rules/testing.md`
[^10]: Reviews index. `docs/reviews/README.md`
[^11]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^12]: Findings register, FND-041 and FND-044. `docs/FINDINGS.md`
[^13]: Decision Record Scope. `.claude/rules/adr-scope.md`
