# Review 0019: the soldier arena

> **The tile faction column is gone.** Backlog item 0084 removed it, with the
> construction loop that filled it and the clause of the world invariant check
> that read it.[^12] Finding 1 below names that clause as the third
> declaration site of the faction ceiling. Two sites remain, and they are the
> project ceiling constant and the faction count of the world settings. The
> review text is left as it was written, because it reports the commit it
> read.

## What was reviewed

Commit `2d119be`, "Implement the soldier arena, and give the golden test a
populated world", on branch `feat/sprint-3-entities-move`.

Files read at that commit:

| Path | State |
|---|---|
| `crates/cachette-core/src/soldier.rs` | New |
| `crates/cachette-core/src/types.rs` | Changed |
| `crates/cachette-core/src/world.rs` | Changed |
| `crates/cachette-core/src/lib.rs` | Changed |
| `crates/cachette-core/src/hex.rs` | Read for context |
| `crates/cachette-core/tests/soldier_arena.rs` | New |
| `crates/cachette-core/tests/value_types.rs` | Changed |
| `crates/cachette-core/tests/thread_equivalence.rs` | Changed |
| `crates/cachette-core/tests/golden_state_hash.rs` | Changed |
| `crates/cachette-core/tests/public_api.rs` | Changed |

The working tree has moved under `crates/cachette-core/`. Every line number
below is a line of the committed file, read through `git show`.

## Who reviewed it

A reviewer that wrote none of this code. The reviewer read the committed
state only. The reviewer ran no build and no test, so every statement about
test behaviour below is a statement about the source, not a measurement.

## Verdict

**ACCEPT WITH AMENDMENT.**

The code honours every decision it cites. The two repairs both hold. The
determinism story is sound. Four findings need an answer before this becomes
the foundation of the later entity systems, and one of them is a defect in
the world, not in the arena.

| # | Finding | Severity |
|---|---|---|
| 1 | Two faction ceilings govern one world, and the test suite already violates one of them while passing | High |
| 2 | `despawn` decrements `live_count` without a liveness guard | Medium |
| 3 | `check_invariants` does not reject a duplicate entry in the free queue | Medium |
| 4 | `hash_into` uses two byte-order conventions in one function | Low |
| 5 | The arena is inert with respect to `World::step`, and two test comments overclaim | Low |
| 6 | One assertion in the first-entity test cannot fail | Low |
| 7 | `place` validates the address before the identity | Low |
| 8 | `NO_GENERATION` means both "never used" and "retired" | Low |

---

## A. Record compliance, decision by decision

### ADR-0014 D1, an identity is a slot index and a generation, and the storage is the only thing that builds one

**Honoured.** `Entity` is `#[repr(transparent)]` over a private
`core::num::NonZeroU64` (`types.rs:65-67`). The two parts are packed into one
opaque value by `Entity::new` (`types.rs:90-96`), which is now `pub(crate)`.
Section B below reports the attempt to defeat this.

The arena is the only crate caller: `SoldierArena::spawn` (`soldier.rs:254`)
and `SoldierArena::iter` (`soldier.rs:390`). The only other call sites are the
three arena unit tests (`soldier.rs:507, 522, 540`).

`slot_count` never falls (`soldier.rs:208-210`); it returns
`self.generations.len()`, and no path shrinks that vector outside a test.
`open_slot` only pushes (`soldier.rs:264-267`). No compaction exists.

### ADR-0014 D2, resolving an identity can fail, and the caller handles the failure

**Honoured.** `slot_of` (`soldier.rs:314-322`) returns `Option<u32>` and
compares the stored generation against `entity.generation()`. Every reader
propagates the absence rather than reporting an error: `tile`
(`soldier.rs:332-335`), `address` (`soldier.rs:340-342`), `faction`
(`soldier.rs:347-350`), `contains` (`soldier.rs:326-328`). `despawn` returns
`false` (`soldier.rs:286-288`) and `place` returns `Ok(false)`
(`soldier.rs:369-371`). None of them is an error variant, which is what D2
asks for.

`slot_of` also handles an out-of-range slot: `self.generations.get(...)?`
(`soldier.rs:316`) rather than a subscript. That is the correct shape.

### ADR-0014 D3, the generation advances at the free, not at the allocation

**Honoured.** `despawn` advances it (`soldier.rs:299`). `spawn` never
advances it; it only lifts a never-used slot from `NO_GENERATION` to
`FIRST_GENERATION` (`soldier.rs:247-249`). The integration test
`a_stale_identity_fails_at_the_free_and_not_at_the_reuse`
(`soldier_arena.rs:73-89`) checks the distinction directly: it despawns and
then resolves, with no intervening spawn.

### ADR-0014 D4, a freed slot returns in first-in first-out order

**Honoured.** `free` is a `VecDeque` (`soldier.rs:138`). `despawn` pushes to
the back (`soldier.rs:300`) and `spawn` pops from the front
(`soldier.rs:242`). `a_freed_slot_returns_in_first_in_first_out_order`
(`soldier_arena.rs:120-150`) frees slots 0, 2, 1 in that order and asserts
the reuse order is `[0, 2, 1]`, which a last-in first-out queue would report
as `[1, 2, 0]`. The test discriminates.

### ADR-0014 D5, a slot whose generation cannot advance is retired

**Honoured in the code.** `despawn` checks `LAST_GENERATION` before the
increment (`soldier.rs:292-298`) and returns without pushing to `free`.
Section C below judges its reachability and the test that drives it.

### ADR-0014 D6, a generation starts at one, never at zero

**Honoured.** `FIRST_GENERATION` is 1 (`soldier.rs:52`) and `NO_GENERATION` is
0 (`soldier.rs:41`). Section B(ii) verifies representability.

### ADR-0014 D7, the location table is a dense array indexed by the slot

**Honoured.** `tiles`, `factions`, `generations` and `live` are all `Vec`
indexed by the slot (`soldier.rs:130-136`). No `HashMap` appears in the file.
The only associative structure is the `VecDeque` free queue, which is ordered
by insertion and not by a hash.

### ADR-0012 D3, an entity of any shape lives in the generational arena

**Honoured.** The soldier lives in `SoldierArena`, which `World` owns as a
field (`world.rs:133`). The tile columns `values` and `factions` hold no
entity. The split is clean.

### `ADR-0012 D4` does not exist

The record has no fourth decision; D3 is followed by "The alternative
this rejects". Nothing to check. If the task meant the D3 consequence that
tile data is the zero-copy path and unit data is not, the arena honours it:
`tile_column` and `faction_column` (`soldier.rs:404, 411`) return borrowed
slices, and the doc comment on `tile_column` declares what copies at the call
site.

### ADR-0066 D1, entity storage holds four fixed shapes, each with its own column set

**Honoured.** `SoldierArena` holds the soldier columns and nothing else. It
carries a tile address and a faction, which matches the record's description
of the mobile shape. The needs vector and the formation are absent, which is
correct for a first slice and is not a contradiction of D1.

### ADR-0066 D3, the shapes do not vary at run time

**Honoured.** `SoldierArena` is a Rust type with named fields. There is no
component registry, no run-time archetype table, and no way to add a column
without editing the struct. A fifth shape would be a new type.

### ADR-0017, the world does not wrap

**Honoured.** `spawn` and `place` both refuse an outside address through
`Grid::index_of` (`soldier.rs:236, 366`) with the typed variant
`TileOutsideWorld`. Neither wraps nor clamps.

### ADR-0002, no floating point

**Honoured.** The file declares `u32`, `u8`, `TileIdx(u32)` and
`FactionId(u16)`. No float type and no float literal appears.

### ADR-0004 and ADR-0006

Covered in section D.

---

## B. The two repairs

### (i) Can anything outside the arena mint an identity?

Every route the reviewer attempted, and the outcome of each:

| Attempt | Outcome |
|---|---|
| Call `Entity::new` from a test or a downstream crate | **Fails.** `pub(crate)` (`types.rs:90`) |
| Construct the tuple struct: `Entity(n)` | **Fails.** The field is private and the struct is not `pub`-fielded (`types.rs:67`) |
| `Entity::default()` | **Fails.** The derive list is `Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash` (`types.rs:66`). No `Default`, and `NonZeroU64` has none either |
| A `From` or `TryFrom` impl | **Fails.** `git grep "impl.*for Entity"` over the crate at this commit returns nothing |
| Deserialisation | **Fails.** `Cargo.toml` has one dependency, `bytemuck`. No `serde` anywhere |
| `bytemuck` transmute: `cast`, `cast_slice`, `from_bytes` | **Fails.** `Entity` derives neither `Pod` nor `Zeroable`, and it cannot derive `Zeroable`, because zero is not a valid `NonZeroU64` |
| Arithmetic on `to_bits` and a way back | **Fails.** `to_bits` (`types.rs:112-114`) has no inverse. No `from_bits`, no `from_raw`, no unsafe constructor exists in the tree |
| Clone or copy an identity the arena gave out, then mutate it | **Fails.** No mutator and no public field |

**D1 holds.** The reviewer could not defeat it.

One residual note, not a defect: `Entity` derives `Ord` over the packed value,
and the generation occupies the high 32 bits. Sorting a slice of identities
therefore orders by generation first and by slot second. Any later code that
sorts entities to fix an order under ADR-0004 will get generation-major
order. It is deterministic, so it is not a determinism defect, but it is not
the order a reader expects from a slot index, and `iter` (`soldier.rs:384`)
produces slot order instead. State the intended order at the first call site
that sorts.

### (ii) Is slot 0 at the first generation representable?

Yes. `Entity::new(0, 1)` computes `raw = (1u64 << 32) | 0 = 0x1_0000_0000`,
which is non-zero, so `NonZeroU64::new` returns `Some` (`types.rs:91-95`).

Nothing else can produce a zero-packed identity. `Entity::new` returns `None`
for it, and the two arena call sites both `.expect()` on a value the arena
guarantees is non-zero:

- `spawn` (`soldier.rs:254`) uses `self.generations[index]`, which is at least
  `FIRST_GENERATION` because line 247-249 lifts a `NO_GENERATION` slot before
  the call, and because `despawn` never leaves a slot in `free` at generation
  zero.
- `iter` (`soldier.rs:390`) uses the generation of a live slot, and
  `check_invariants` (`soldier.rs:462-464`) asserts that a live slot never
  holds `NO_GENERATION`.

The `.expect` in each is therefore unreachable while the invariant holds. It
is not a typed error path, and it does not need to be, because no caller
input reaches it. This is the correct use of `expect`.

`the_first_entity_the_arena_ever_allocates_has_an_identity`
(`soldier_arena.rs:42-59`) is the direct check, and it is right to exist. See
finding 6 for the one assertion in it that cannot fail.

---

## C. The retirement rule, D5

**`LAST_GENERATION` is not reachable in practice.** It is `u32::MAX`
(`soldier.rs:63`). Reaching it needs about 4.29 thousand million despawns of
one slot. Under first-in first-out reuse the increments spread across the
whole freed set, which is exactly the argument D4 makes, so the real workload
moves away from the case rather than toward it. A hostile workload that keeps
one slot alone in the queue could reach it, at a cost the record already
accepts.

**The retirement path is therefore driven only by its own unit tests.** Three
of them (`soldier.rs:501, 516, 534`) each write `arena.generations[0] =
LAST_GENERATION` directly and then mint the aged identity through
`Entity::new`. They construct the state; they do not reach it through the
public interface.

**This is acceptable, and the module says why.** The testing rule requires a
test to drive the real caller when the engine is obligated to invoke
something. Here the obligation is on the generation counter, not on a caller,
and the module doc comment (`soldier.rs:482-491`) states the exception
explicitly: the public interface would need four thousand million spawns. The
alternative designs are worse. Making `LAST_GENERATION` a configurable field
would put a second declaration site next to the identity layout, which is the
first recurring defect shape. Leaving the path untested would ship the one
rule this record exists to enforce with no coverage at all.

The three tests are also not degenerate. `a_retired_slot_never_returns_to_use`
(`soldier.rs:516-531`) asserts that the next spawn opens slot 1 and that
`slot_count` reaches 2, which the commit message reports as killing the
mutation that returns a retired slot to the queue.

**One gap.** No test asserts that `retired_count` is excluded from a later
reuse over more than one retirement, and no test retires a slot that is not
slot 0. Both are cheap to add. This is not a defect in the code.

---

## D. Determinism

### What fixes the order of each result

| Function | Order | Fixed by |
|---|---|---|
| `iter` (`soldier.rs:384-393`) | Slot order | `self.live.iter().enumerate()`, a `Vec` walk. No hash, no thread |
| `tile_column` (`soldier.rs:404`) | Slot order | Borrowed slice of a `Vec` |
| `faction_column` (`soldier.rs:411`) | Slot order | Borrowed slice of a `Vec` |
| `hash_into` (`soldier.rs:424-439`) | Slot order, then free-queue order | `Vec` walks and a `VecDeque` walk. `VecDeque` iteration is front to back by insertion, which `despawn` and `spawn` alone control |
| `check_invariants` (`soldier.rs:451-477`) | Slot order | An index range and `Vec` walks |

No `HashMap`, no `HashSet`, no `rayon`, no thread handle, and no completion
order appears anywhere in `soldier.rs`. The reviewer looked for all five and
found none. **ADR-0004 D1 holds.**

`hash_into` also covers the state that decides a later frame, which is the
part a hash usually misses: it writes the generation of every slot and the
whole free queue (`soldier.rs:432-437`), not only the live columns. That is
correct, and it is why the free-queue mutation in the commit message is
catchable at all.

### Does the thread-count test cover a populated arena?

Partly, and the test comment overclaims. See finding 5.

`a_world_that_holds_soldiers_is_identical_at_every_thread_count`
(`thread_equivalence.rs`) does build a populated arena through `populate`,
runs the frames at 1, 2 and 12 threads, and compares the event log bytes, the
state hash and the live count. The population is a fixed arithmetic pattern
with no randomness, so it is the same at every thread count. The assertion
`expected.2 > 0` guards against a silently empty arena, which is the right
guard to have written.

But `World::step` (`world.rs`) never reads or writes `self.soldiers`. It
steps `self.values` in chunks and joins the event log through `Slots`. The
soldier state at the end of the run is bit-identical to the soldier state
before the first step, at every thread count, because nothing touched it.

So the test proves two real things: the soldier columns reach the state hash,
and the parallel tile step does not corrupt the arena. It does not prove that
any parallel work over soldiers is deterministically ordered, because there
is no such work yet. That is fine for this commit. It stops being fine the
moment a system iterates soldiers, and the test as written will keep passing
green through that change without gaining coverage.

`the_soldier_columns_reach_the_state_hash` (`thread_equivalence.rs`) is the
better test of the two. It compares a bare world against a populated one,
then moves one soldier and shows the hash changes. It has a proven failure
mode.

---

## E. Test quality

### `soldier_arena.rs`

| Test | Judgement |
|---|---|
| `the_first_entity_the_arena_ever_allocates_has_an_identity` | See below |
| `a_soldier_carries_a_tile_and_a_faction` | Behaviour, public interface. Sound |
| `a_stale_identity_fails_at_the_free_and_not_at_the_reuse` | Behaviour. The strongest test in the file; it is the direct check of D3 |
| `a_second_despawn_of_one_identity_removes_nothing` | Behaviour. Sound |
| `a_reused_slot_gives_a_different_identity` | Behaviour. Asserts `slot_count() == 1`, which pins the reuse, not the layout. Sound |
| `a_freed_slot_returns_in_first_in_first_out_order` | Behaviour. Discriminates against last-in first-out. Sound |
| `the_arena_refuses_a_spawn_when_it_holds_no_free_slot` | Behaviour. Also checks recovery after a free. Sound |
| `the_arena_refuses_an_address_outside_the_world` | Behaviour. Checks four sides and asserts no slot opens. Sound |
| `the_arena_refuses_a_faction_at_or_above_the_ceiling` | Behaviour, and it checks that the ceiling is exclusive. Sound. But see finding 1: it checks the wrong ceiling for a soldier in a world |
| `a_stale_identity_moves_no_soldier` | Behaviour. Sound |
| `the_live_soldiers_come_back_in_slot_order` | Behaviour. Sound |
| `an_identity_round_trips_through_the_arena` | Property. Sound |
| `any_order_of_spawn_and_despawn_leaves_the_live_set` | Property, against a plain-list model that holds no generation, so it cannot repeat an arena defect. The best test in the commit. It checks `check_invariants` after every step |
| `a_stale_identity_never_reads_as_another_soldier` | Property. This is the test the record's consequences section asks for by name |

The proptest failure-persistence configuration (`soldier_arena.rs:266-279`)
names the regression file directly rather than relying on source-parallel
discovery, and cites the finding that recorded why. That is correct and it is
the kind of thing that usually gets omitted.

**No test in this file is unable to fail**, with one exception inside an
otherwise sound test.

### `the_first_entity_the_arena_ever_allocates_has_an_identity`

It does test what its name claims, but not through the assertion a reader
would expect.

The load-bearing line is `.expect("the first spawn must succeed")`
(`soldier_arena.rs:50-52`). Under the mutation "a generation starts at zero",
`spawn` computes `Entity::new(0, 0)`, which returns `None`, and the
`.expect` inside `spawn` (`soldier.rs:255`) panics. The test fails. So the
test kills the mutation, and the commit message's count of five killed tests
is plausible.

The two explicit assertions do less than they appear to:

- `assert_eq!(first.generation(), 1)` (line 54) is the real check of D6, and
  it is good.
- `assert!(first.to_bits() != 0)` (line 55) **cannot fail.** `Entity` wraps
  `NonZeroU64`, so `to_bits` is non-zero by construction, for every value the
  type can hold. The assertion restates the type. It reads as the check of the
  packing rule and is not one.

This is finding 6. The amendment is small.

### The arena unit tests

`a_short_column_fails_the_check` (`soldier.rs:547`) and
`a_live_count_that_disagrees_fails_the_check` (`soldier.rs:557`) both reach
into private fields. That is correct here: they exist to prove that
`check_invariants` can fail, and the only way to prove that is to break an
invariant that the public interface is designed to preserve. A check with no
proven failure mode is decoration, so these two earn their access.

### `value_types.rs`

The four rewritten call sites now mint through the arena, which is the point
of the repair. One loss came with it: the old
`an_entity_of_index_zero_and_generation_zero_does_not_exist` asserted
`Entity::new(0, 0) == None`, and the replacement
`an_absent_entity_costs_no_extra_space` only asserts the size of
`Option<Entity>`. The `None` case is now unreachable from a test, which
follows from making `Entity::new` `pub(crate)` and is the accepted cost. The
niche is still checked by the size assertion. No action needed; noted so that
nobody re-adds the old assertion and finds it will not compile.

---

## F. The golden scenario

**It exercises all three mechanisms.** `populate`
(`golden_state_hash.rs`) spawns 64 soldiers, frees every third one (22 of
them), then spawns 8 more. The 8 respawns pop from the front of the free
queue, so they take slots 0, 3, 6, 9, 12, 15, 18, 21 at generation 2, while
14 slots remain free. The scenario therefore covers the generation advance
(D3), a non-empty free queue at the end of the run, and reused slots (D4).
The `.expect("the respawn must reuse a freed slot")` is not itself a check
that reuse happened, but `hash_into` records `slot_count`, so a change that
opened new slots instead would change the hash.

**It would catch a change to the soldier state representation.** `hash_into`
absorbs `slot_count`, `live_count`, `retired_count`, the whole tile column,
the whole faction column, the live column, every generation, and the free
queue in order. A change to any field width, to the column set, or to the
reuse order changes the recorded sequence. The commit message reports the
last-in first-out mutation as changing this scenario and leaving the two
older ones green, which is the proven failure mode the golden file needed and
did not have.

**`populate` is deterministic.** Every value is derived from the loop counter
by integer arithmetic: `step * 7 % width`, `step * 5 % height`,
`step % 3`. No random draw, no clock, no iteration over an unordered
structure. `freed` is a `Vec` walked in push order. The second loop is the
same shape. It produces the same arena on every run and on every platform,
subject to finding 4.

**One weakness.** The scenario runs `populate` before frame zero and then
steps 32 frames, and `step` never touches the arena, so all 33 recorded
hashes carry the same soldier contribution. The golden file has 33 lines of
which one line's worth of soldier state is distinct. That is enough for now
and will need revisiting when a system moves soldiers.

---

## G. Recurring defect shapes

### Shape 1, redundant declaration sites with undocumented precedence

**Two instances. One is handled well; one is finding 1.**

Handled well: the arena holds a copy of the `Grid`, and
`World::check_invariants` (`world.rs`) compares the two copies and fails when
they disagree, with a comment that names the shape. That is exactly the
remedy the rule asks for.

**Finding 1, high.** The faction ceiling is declared in three places, and no
check fails when they disagree.

1. `FACTION_CEILING: u16 = 63` in `types.rs`, new in this commit.
2. `WorldConfig::faction_count: u16` in `world.rs`, whose doc comment says in
   prose "The ceiling is 63".
3. `World::check_invariants` uses `let ceiling = self.config.faction_count.max(1)`
   for the tile faction column.

`SoldierArena::spawn` (`soldier.rs:238-240`) validates against `FACTION_CEILING`,
the global 63. `World` validates its tile factions against
`config.faction_count`. So on a world built with `faction_count: 3`,
`World::spawn_soldier(address, FactionId(50))` succeeds, and
`World::check_invariants` returns `true`, because the world check only walks
the tile column and the arena check only compares against 63. The world now
holds a soldier of a faction the world does not have.

**This already happens in the test suite, and the suite is green.**
`thread_equivalence::populate` assigns `FactionId((index % 5) as u16)`. The
second scenario, "fewer tiles than threads", is `width: 7, height: 1,
faction_count: 2`, so `populate` walks indices 0 to 6 and spawns soldiers of
factions 0, 1, 2, 3, 4, 0, 1 into a world that has two factions. The fourth
scenario, "an uneven split", has `faction_count: 4` and takes a faction 4.
`run_with_soldiers` then calls `world.check_invariants()` and it returns
`true`, because the world check walks the tile column only and the arena
check compares against 63.

The tests do not fail, and they should. This is the shape the rule names: one
value in two places, read back correctly from both, with nothing that fails
when they disagree.

Further, `WorldConfig` validates nothing at construction. `World::new`
accepts `faction_count: 1000`, and then the tile column holds identifiers up
to 999 while the arena refuses anything at 63 or above. The two ceilings
disagree in both directions.

The amendment has two parts, and the second is the one that matters:

- `World::new` must refuse a `faction_count` at or above `FACTION_CEILING`,
  through a new `WorldError` variant. That makes the prose in the doc comment
  a check.
- `SoldierArena` must take the ceiling that governs the world it sits in,
  rather than reading the global constant, or `World::spawn_soldier` must
  validate against `self.config.faction_count` before it delegates. One of the
  two. Whichever is chosen, the other declaration site stops being a second
  authority.

Removing the prose "The ceiling is 63" from the `faction_count` doc comment
and citing `FACTION_CEILING` instead would remove the third site.

### Shape 2, documents that rot when a sweep names specifics

**Nothing found.** The doc comments in `soldier.rs` cite records and
decisions by number and state constraints. They name no count, no file table,
and no measured figure. The two constants that look like budgets,
`LAST_GENERATION` and `SLOT_INDEX_LIMIT`, both carry a comment saying they are
the range of a field and not a budget (`soldier.rs:56-58, 66-68`), which is
the right distinction and the right place to make it.

The commit message carries the counts, the mutation table and the correction
about running every test target. That is where the rule says they go.

### Shape 3, inert code that nothing invokes

**Finding 5, low.** `World::step` never reads `self.soldiers`. The arena is a
capability that the engine does not yet invoke. That is defensible for a
foundation commit: no system moves soldiers yet, and the commit message says
the Python bindings do not expose them because nothing needs them.

What is not defensible is the two comments that claim otherwise:

- `thread_equivalence.rs`, inside `run_with_soldiers`: "Drive the step, then
  inspect the arena. A column set that no test reaches through the engine is
  inert." The test does drive the step and does inspect the arena, but the
  step does not reach the arena, so the inspection proves only that the step
  leaves it alone.
- `public_api.rs`, inside
  `a_soldier_survives_a_step_and_the_world_holds_its_invariants`: "FND-041: a
  column set that no system reaches is inert. The test drives the step and
  then inspects the arena." Same overclaim. The test name is honest —
  "survives a step" is exactly what it checks — and the comment is not.

Amend both comments to say what is true: the step does not yet touch the
arena, and the test asserts that it leaves the arena unchanged. Then the
comment becomes a marker for the future work rather than a claim that the
work is done.

`retired_count()` (`soldier.rs:214`) is a public accessor that only the arena
unit tests read. It is a one-line getter over state the hash already covers,
so this is a note and not a finding.

### Shape 4, nondeterminism that the tests cannot see

**Nothing found in the arena.** Covered in section D. The five listed entry
paths are all absent: no hash iteration, no thread order, no convergence test,
no undeclared padding in this file, no thread-local random state.

One near-miss worth naming: `hash_into` writes `bytemuck::cast_slice(&self.tiles)`,
which produces native-endian bytes. That is finding 4. It is not
nondeterminism between runs on one platform and it is not nondeterminism
between thread counts, so neither determinism test can see it. It is a
divergence between platforms, and the project targets `aarch64` while
developing on `x86-64` and Apple Silicon. All three are little-endian, so the
golden files agree today. The defect is latent and the fix is cheap.

### Shape 5, a record that no longer describes the code

**Nothing found.** The reviewer checked ADR-0014 D1 to D7, ADR-0012 D3,
ADR-0066 D1 and D3, ADR-0017 and ADR-0002 against the implementation
individually, in section A. No record now disagrees with the code. The
footnote in `soldier.rs:402` cites ADR-0044 through the registry rather than
a file, which is correct, because that record has no file yet.

---

## H. Error paths

Every caller mistake in the public interface is a typed refusal.
`SoldierError` (`soldier.rs:76-87`) has three variants, one for each mistake a
caller can make, and it implements `Display` and `std::error::Error`
(`soldier.rs:89-107`). `spawn` returns them; `place` returns
`TileOutsideWorld`; `despawn`, `tile`, `address`, `faction` and `contains`
return an absence rather than an error, which D2 requires.

The reviewer walked every subscript, every arithmetic operation, every
`unwrap` and every `expect` in the non-test part of `soldier.rs`.

**`unwrap`: none.**

**`expect`: two, both unreachable.** `soldier.rs:255` and `soldier.rs:391`.
Both are analysed in section B(ii). Neither takes caller input; both assert an
invariant the arena maintains. They are correct.

**Subscripts.** `self.generations[index]`, `self.live[index]`,
`self.tiles[index]`, `self.factions[index]` in `spawn` (`soldier.rs:247-252`)
and `despawn` (`soldier.rs:290-299`), and `self.tiles[slot as usize]` in
`tile` and `place`. Every one of them takes an index that came from
`slot_of`, which bounds-checks through `.get()`, or from `open_slot`, which
pushes the row before returning the index. The `free` queue only ever holds a
slot that `despawn` put there, and `despawn` only reaches that line after
`slot_of` succeeded. None is reachable out of range.

`check_invariants` subscripts `self.live[*slot as usize]` and
`self.generations[*slot as usize]` over the free queue (`soldier.rs:474-476`),
after it has already returned `false` for a length mismatch
(`soldier.rs:453`). A free-queue entry beyond the column length would still
panic, because the length check compares the four columns against each other
and not against the queue. That is unreachable today by the same argument. It
is worth one line, because `check_invariants` is the function that is
supposed to survive a broken arena.

**`live_count -= 1` (`soldier.rs:291`). Finding 2, medium.**

This runs on the strength of `slot_of` returning `Some`, and `slot_of`
(`soldier.rs:314-322`) **does not check `live`.** It compares generations
only. So the guard against underflow is not a liveness test; it is the chain
of reasoning that no identity can match the generation of a non-live slot:

- A free slot's stored generation is one above the identity that was freed,
  and `spawn` mints the identity for that generation only at the moment it
  sets `live[index] = 1`.
- A retired slot's stored generation is `NO_GENERATION`, and no minted
  identity holds generation zero.
- A never-used slot holds `NO_GENERATION`, same argument.

The reasoning is sound today, and it holds for every path the reviewer could
construct. But it is three separate facts in three functions holding up one
subtraction, and nothing states it. If a later change lets `spawn` mint an
identity before it marks the slot live, or adds a second free path, the
subtraction underflows. In a debug build that panics; in a release build with
overflow checks off it wraps `live_count` to `u32::MAX`, and the arena then
reports four thousand million live soldiers, `is_empty` returns `false`
forever, and `check_invariants` starts failing far from the cause.

Amend `despawn` to guard on liveness rather than on generation alone:

```rust
let index = slot as usize;
if self.live[index] != 1 {
    return false;
}
self.live[index] = 0;
self.live_count -= 1;
```

This costs one branch on a path that is not the hot path, it makes the
invariant local, and it makes the subtraction correct by inspection rather
than by a three-step argument in a review.

**`self.generations[index] += 1` (`soldier.rs:299`).** Correct. The
`LAST_GENERATION` branch at `soldier.rs:292` returns before it, so the operand
is at most `u32::MAX - 1`. Cannot overflow. This is the one arithmetic site in
the file that is guarded properly and visibly.

**`self.live_count += 1` (`soldier.rs:253`).** Bounded by `capacity`, which is
at most `u32::MAX`, and by `slot_count`, which `open_slot` refuses to grow
past `capacity`. Cannot overflow.

**`self.retired_count += 1` (`soldier.rs:296`).** Bounded by the slot count.
Cannot overflow.

**`self.generations.len() as u32` (`soldier.rs:209`).** A narrowing cast. It
would truncate if the vector held more than `u32::MAX` entries. `open_slot`
compares the result against `capacity` before it pushes, and `capacity` is at
most `SLOT_INDEX_LIMIT`, which is `u32::MAX`, so the length stops at
`u32::MAX` and the cast is exact. Safe, but it is safe by a chain that runs
through the very function whose result it is. A `debug_assert` or a comment at
`soldier.rs:209` would make the reader's job shorter.

**`index as u32` in `iter` (`soldier.rs:390`).** Same bound, same argument.

**Finding 3, medium. `check_invariants` does not reject a duplicate in the
free queue.** The check (`soldier.rs:451-477`) verifies the column lengths
agree, the live count agrees, each live slot is well-formed, and each free
slot is not live and not retired. It does not check that `free` holds no slot
twice.

A duplicated entry is the worst failure this structure can have: two spawns
would take the same slot, the second would overwrite the first, `live_count`
would count two soldiers in one slot, and two live identities would name the
same row. The property test
`any_order_of_spawn_and_despawn_leaves_the_live_set` calls `check_invariants`
after every step, so a check that saw duplicates would catch a whole class of
future mistake for the cost of a sort or a bitset. No path produces a
duplicate today. Add the check while it is cheap, because this arena is the
foundation the other three shapes will copy.

The same function could also assert that `free.len() + live_count +
retired_count == slot_count`, which is the accounting identity of the whole
structure and which nothing currently states.

**Finding 7, low. `place` validates the address before the identity**
(`soldier.rs:364-371`). A dead identity paired with an outside address returns
`Err(TileOutsideWorld)`, not `Ok(false)`. `spawn` orders its two checks the
same way, so the file is internally consistent, but the result is that a
caller sweeping stale identities gets an error for a soldier that does not
exist. Under D2 the absent entity is meant to be skipped, not reported.
Resolving the identity first would return `Ok(false)` and let the caller skip.
This is a judgement call, and either order is defensible; state the choice in
the doc comment.

**Finding 8, low. `NO_GENERATION` carries two meanings.** It marks a slot the
arena has never used and a slot the arena has retired
(`soldier.rs:34-36, 295`). The two are distinguished only by `retired_count`,
which is a count and not a per-slot fact. `check_invariants` therefore cannot
tell a retired slot from a never-used one, and `slot_of` would resolve an
identity of generation zero against either. No such identity can be minted, so
this is latent. It costs one sentinel value to separate them, and it would let
`check_invariants` verify that the number of slots marked retired equals
`retired_count`, which is the second copy the rule asks to be checked.

---

## Amendments

The verdict is ACCEPT WITH AMENDMENT. These are the changes.

**Required.**

1. Finding 1. Make one authority govern the faction ceiling. Either
   `World::spawn_soldier` validates against `self.config.faction_count`, or
   `SoldierArena` is constructed with the world's ceiling. Add a `WorldError`
   variant so that `World::new` refuses a `faction_count` at or above
   `FACTION_CEILING`. Replace the prose "The ceiling is 63" in the
   `faction_count` doc comment with a citation of `FACTION_CEILING`. Add a
   test that spawns a soldier of a faction above the world's count and expects
   a refusal. Fix `thread_equivalence::populate`, which currently produces the
   violation; bound its faction by the scenario's `faction_count`.

2. Finding 2. Add the liveness guard in `despawn`, as written in section H.

3. Finding 5. Correct the two test comments that claim the step reaches the
   arena. Replace with: "The step does not yet reach the arena. The test
   asserts that the step leaves it unchanged. When a system moves soldiers,
   this assertion must become a check of what that system produced."

**Recommended.**

4. Finding 3. Add a duplicate check and the accounting identity to
   `check_invariants`, and a unit test that breaks each of them.

5. Finding 4. Write the tile and faction columns through explicit
   little-endian bytes in `hash_into`, as the generation loop already does, or
   state in the doc comment that the state hash is defined on a
   little-endian target. One function should not hold two conventions.

6. Finding 6. Replace `assert!(first.to_bits() != 0)` in
   `the_first_entity_the_arena_ever_allocates_has_an_identity` with an
   assertion that can fail, for example
   `assert_eq!(first.to_bits(), 1u64 << 32)`, which checks the packing and the
   starting generation together.

7. Findings 7 and 8. State the check order in the `place` doc comment. Give a
   retired slot a sentinel distinct from `NO_GENERATION`, and check
   `retired_count` against it.

## References

[^1]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^2]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^3]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^4]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^5]: ADR-0002, state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^6]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^7]: ADR-0006, an event is plain data and applying it is pure. `docs/adrs/accepted/adr-0006-an-event-is-plain-data-and-applying-it-is-pure.md`
[^8]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^9]: Recurring defect shapes. `.claude/rules/recurring-defects.md`
[^10]: Testing rules. `.claude/rules/testing.md`
[^11]: Reviews index. `docs/reviews/README.md`
[^12]: Backlog item 0084, give a tile one faction column. `docs/backlog/complete/0084-give-a-tile-one-faction-column.md`
