# Review: the slot reduction and the key vector sort

## What was reviewed

Commit `f52542e`, "Provide the slot reduction and the key vector sort", on the
branch `feat/sprint-2-entity-storage`.

| Path | Role |
|------|------|
| `crates/cachette-core/src/slots.rs` | New. The slot reduction. |
| `crates/cachette-core/src/sort.rs` | New. The key vector sort. |
| `crates/cachette-core/src/world.rs` | Changed. The step now calls the slot reduction. |
| `crates/cachette-core/src/lib.rs` | Changed. Two modules and four re-exports. |
| `crates/cachette-core/tests/slot_reduction.rs` | New. Eight tests. |
| `crates/cachette-core/tests/key_vector_sort.rs` | New. Twelve tests. |
| `crates/cachette-core/tests/determinism_probe.rs` | Changed. One test added. |
| Two `.proptest-regressions` files | New. |

The reviewer wrote none of this code. The reviewer ran the test suite, the
probe build, and the sort under the probe feature.

## Verdict

**ACCEPT WITH AMENDMENT.**

The two mechanisms are correct. The reviewer attempted eleven objections
against the correctness of the order and could not break any of them. Every
finding below is about coverage, cost, or a second declaration site, not about
a wrong answer.

Three findings must be fixed before this is done. Finding 1 makes a claim in
the commit message false. Finding 2 leaves the sort with no proven failure
mode. Finding 3 leaves two disagreeing ways to order a `Candidate`.

---

## A. Record compliance, decision by decision

### ADR-0004 D1, iteration order is explicit

**Honoured.** `Slots::in_combine_order` at `slots.rs:143` returns
`self.entries.iter()`, which is index order. `Slots::combine` at
`slots.rs:126` is the only reader of it. No result in either file reads a
hash map, and no result reads a thread handle. `std::thread::scope` at
`world.rs:326` and `sort.rs:190` joins every thread before the combine runs,
and neither call site inspects which thread returned first.

One nuance the code does not state. `Slots` fixes *index* order. That index is
the chunk ordinal, not the tile index. Index order equals data order only
because both call sites hand out chunks in ascending data order in the same
loop that spawns them (`world.rs:328`, `sort.rs:192`). Nothing enforces that
pairing. See finding 6.

### ADR-0004 D2, an order-free reduction combines in any order

**Honoured, and correctly not applied.** `World::tile_total` at `world.rs:247`
still folds an integer sum with no slots, which is right. Nothing in
`slots.rs` forces a slot onto an order-free reduction. The module comment at
`slots.rs:3` states the distinction.

### ADR-0004 D3, a reduction that is not order-free needs a slot

**Honoured.** The record names three reductions. All three exist and all three
route through `combine`: `first_wins` at `slots.rs:168`, `minimum` at
`slots.rs:183`, `maximum` at `slots.rs:196`, all via `best` at `slots.rs:204`.
Each writes one slot per unit of parallel work through `entries_mut` at
`slots.rs:114`, and each reads through `combine`. The strict test in `best`
(`candidate.rank < best.rank`, `slots.rs:184`) is the right choice: a
non-strict test would let a later slot displace an equal earlier one and
silently invert the tie rule.

The record says the slot is "indexed by a stable key". The code indexes by
chunk ordinal. That satisfies the intent, because the chunk ordinal is a
function of the data partition and not of a thread identifier.

### ADR-0004 D4, a sort uses a stable key

**Honoured.** `order_on` at `sort.rs:171` accepts `&[SortKey<N>]` and nothing
else. `sorted_run` at `sort.rs:256` sorts by `SortKey<N>`, whose `Ord` is
derived at `sort.rs:105` and is therefore plain lexicographic comparison of
`[u64; N]`. No comparison in the file can vary between two calls on the same
input.

### ADR-0007 D1, content supplies a key, never a comparison function

**Honoured.** No public function in `sort.rs` takes a closure, a function
pointer, or a generic bounded by `Fn`. `order`, `order_on` and `sorted` take
slices of values. Contrast `Slots::combine`, which does take an `FnMut` — that
is the engine's own fold, not a content ordering, and it is the right place
for the asymmetry.

### ADR-0007 D2, the last key field is a stable identifier

**Honoured, and enforced rather than documented.** See section C.
`SortKey::new` at `sort.rs:120` carries `const { assert!(N > 0) }`, so a key
with no identifier does not compile. `check_identifiers` at `sort.rs:238`
rejects a repeated identifier at run time, before any thread starts.

### ADR-0007 D3, the engine never calls content code from inside a sort

**Honoured.** See section B.

### ADR-0002 D1, no floating point

**Honoured.** `Candidate::rank` is `i64` (`slots.rs:50`). `SortKey` fields are
`u64` (`sort.rs:107`). `from_signed` at `sort.rs:89` uses an integer XOR, not
a cast through a float. Neither file mentions a float type or a float literal.

### ADR-0001 D4, one binary gives one answer at any thread count

**Honoured for the reductions, unproven for the sort.** See finding 2.

---

## B. The ADR-0007 D3 claim: is a comparison function unrepresentable?

The commit claims a comparison function is not discouraged but unrepresentable.
The reviewer attempted six ways to defeat that claim.

1. **Pass a closure.** Refused by the signature. `order`, `order_on` and
   `sorted` take no callable parameter. **Held.**
2. **Smuggle logic through `descending()` (`sort.rs:80`).** It is
   `pub const fn descending(field: u64) -> u64 { !field }`. It runs in the
   caller and returns a `u64`. Its output is a value in the key array by the
   time `order_on` sees it. **Held.**
3. **Smuggle logic through `from_signed()` (`sort.rs:89`).** Same shape. A
   pure `const fn` from `i64` to `u64`, evaluated by the caller. **Held.**
4. **Override `Ord` on the key.** `SortKey<N>` is defined in this crate and
   derives `Ord`. Content cannot write a conflicting `impl Ord for SortKey<N>`
   — the orphan rule forbids it, and a duplicate impl in this crate would not
   compile. `sorted_run` at `sort.rs:258` names that `Ord` through
   `sort_unstable_by_key`. **Held.**
5. **Run content code through the item type `T` in `sorted` (`sort.rs:218`).**
   `T: Copy`, so `T` has no `Drop` and no user `Clone`. Line 230 does
   `items[*index as usize]`, a bitwise copy. No user code runs. **Held.**
6. **Run content code through the payload `P` in `Candidate`
   (`slots.rs:173`).** `P: Copy`, same argument. `best` compares `rank` only
   and never touches `payload`. **Held.**

**No path exists where content code runs inside the sort.** The claim is
correct as stated. This is the strongest part of the commit.

One caveat worth recording, not a defect: the interface makes an ordering
*that needs pairwise comparison* impossible to express, exactly as the record's
consequences section promises. Reviewers should expect content authors to push
back on this, and the answer is the record, not the code.

---

## C. The tie claim: enforced or documented?

**Enforced, and the refusal is correct.**

`check_identifiers` at `sort.rs:238` collects every last field, sorts the copy,
and scans neighbouring pairs. A repeat returns
`SortError::RepeatedIdentifier`. The test at `key_vector_sort.rs:118`
exercises it. The error names the lowest repeated value, because the scan runs
over a sorted copy, so the error message itself does not depend on the input
order. That detail is easy to get wrong and this code gets it right.

Uniqueness of the last field implies no two whole keys tie, so checking one
field is sufficient. **Refusal, not a silent wrong answer.**

**Documenting would not have been sufficient here**, and the code is right to
check. The reason is at `sort.rs:258`: `sorted_run` uses
`sort_unstable_by_key`. An unstable sort given two equal keys may place them in
either order, and that order can depend on chunk length, which depends on the
thread count. Uniqueness is therefore not a nicety — it is the precondition
that makes `sort_unstable` safe to use. `check_identifiers` runs at line 181,
before the spawn at line 190, so the precondition is established before it is
relied on. That ordering is load-bearing and undocumented. See finding 5.

`merge_runs` at `sort.rs:271` is independently total on ties (strict `<` at
line 282 means a lower run index wins), so even if the check were removed the
merge would not deadlock. It would just return an order that depends on the
chunking.

---

## D. Determinism

What fixes the order of every result:

| Result | Order fixed by |
|--------|----------------|
| `Slots::combine` fold | `entries.iter()`, index order (`slots.rs:143`) |
| `first_wins`, `minimum`, `maximum` | `combine`, plus a strict win test (`slots.rs:204`) |
| `World::step` event log | `combine` over the chunk slots (`world.rs:340`) |
| `sorted_run` within a chunk | derived `Ord` on `SortKey`, over unique keys (`sort.rs:258`) |
| `merge_runs` across chunks | key comparison, strict `<`, lower run index on a tie (`sort.rs:282`) |
| `check_identifiers` error value | sorted copy, lowest repeat reported (`sort.rs:240`) |

**Hash iteration:** none. Neither file constructs a `HashMap` or `HashSet`.

**Thread completion order:** none. Both `thread::scope` blocks join before any
result is read.

**Time:** none. Neither file references `Instant`, `SystemTime`, or a
duration.

**`sort_unstable`:** two uses, both sound. `sort.rs:240` sorts a `Vec<u64>` of
plain values, where element identity does not exist. `sort.rs:258` sorts by a
key proven unique on line 181.

**Can `Slots::combine` be bypassed?** Yes. `entries()` at `slots.rs:102` is
`pub` and returns the raw slice. `merge_runs` already takes that route
(`sort.rs:201`). This is not a determinism defect — a slice is read in index
order — but it is the reason the probe misses the sort. See finding 2.

**Reviewer found no determinism defect in this commit.**

---

## E. The probe

The switch moved from `world::ordered_slots` to `Slots::in_combine_order`
(`slots.rs:142` and `slots.rs:152`). The reviewer verified the following by
running the builds.

**It still makes the thread-count test fail.** Confirmed indirectly: the probe
now sits under the step's log join, and `slot_reduction.rs` test
`the_world_step_joins_its_slots_through_the_reduction` fails under the feature.
The `justfile` `probe` recipe asserts the thread-count failure with `!`.

**It now covers more.** Under `--features probe-nondeterminism`, six of the
eight `slot_reduction` tests fail:

```
cargo test -p cachette-core --features probe-nondeterminism --test slot_reduction
-> test result: FAILED. 2 passed; 6 failed
```

That is a real improvement over the old switch, which only reversed the log
join.

**No shipped path depends on the feature.** The feature appears only in
`slots.rs` (two `cfg` attributes), `Cargo.toml`, the probe test's
`#![cfg(...)]`, the `justfile`, `.github/workflows/ci.yml`, and `docs/TESTING.md`.
No other crate in the workspace declares it, so feature unification cannot turn
it on.

**Every reduction in `slots.rs` is perturbed.** `first_wins`, `minimum` and
`maximum` all reach `combine`, and `combine` is the only reader of
`in_combine_order`. The reviewer looked for a reduction that reads
`self.entries` directly and found none.

**But the sort is not perturbed at all.** See finding 2.

---

## F. Test quality

Twenty new tests, plus one in the probe. The commit says twenty; counting the
probe test it is twenty-one. Trivial.

### Tests that carry weight

- `the_lowest_position_wins_a_tie_for_the_minimum` (`slot_reduction.rs:141`)
  and `..._for_the_maximum` (line 156). **The best tests in the commit.** They
  compute the expected answer with `iter().min()` and `iter().position()` —
  independently of the mechanism — so they pin the answer rather than compare
  the mechanism against itself. The test module comment at line 137 says
  exactly this, and it is right. Both fail under the probe.
- `first_wins_takes_the_lowest_position` (line 171). Same shape, independent
  expectation from `iter().position()`. Fails under the probe.
- `the_world_step_joins_its_slots_through_the_reduction` (line 213). Drives
  the engine, not the mechanism, which is what the testing rule asks for.
  Fails under the probe.
- `the_output_is_one_rising_permutation` (`key_vector_sort.rs:72`). Checks a
  real property — permutation plus strict rise — against an independent
  `is_a_permutation` helper.
- `the_order_does_not_depend_on_the_input_order` (line 90). Rotates the input
  and compares the emitted *keys*, not the indices. Correct: comparing indices
  would have been meaningless after a rotation.
- The four error tests (`a_reduction_over_zero_slots_is_an_error`,
  `a_repeated_identifier_is_an_error`, `a_sort_at_zero_threads_is_an_error`,
  `a_length_mismatch_is_an_error`) each pin a refusal that the code could
  plausibly drop.

### Tests that would still pass if the ordering rule were violated

- `the_minimum_does_not_depend_on_the_thread_count`
  (`slot_reduction.rs:128`). It compares the mechanism against itself at three
  thread counts. Under the probe it does fail, but only because reversal
  interacts with chunking; a reversal that were somehow thread-count-uniform
  would slip through. The paired `the_lowest_position_wins_a_tie` test covers
  the gap, so this is acceptable as written.
- **All twelve `key_vector_sort` tests pass under the probe.** Verified:
  `test result: ok. 12 passed`. Every one of them is blind to a slot-order
  defect. That is finding 2, not a defect in any individual test.
- `an_empty_reduction_holds_no_candidate` (line 189) and
  `a_key_reports_its_fields_and_its_identifier` (`key_vector_sort.rs:188`) are
  low-value accessor tests. They cannot fail from an ordering defect. They are
  cheap and they do pin `identifier()` to the last field, so keep them.

### Tests that pin an implementation detail

- `the_combine_step_reads_the_slots_in_index_order` (`slot_reduction.rs:199`)
  asserts on `slots.entries()` at line 209 as well as on the fold result. The
  `entries()` assertion pins a getter and cannot fail from an ordering defect,
  because `entries()` is not perturbed. The `combine` assertion above it is the
  real test. Minor.

### Tests that cannot fail

None. Every test has at least one assertion that a plausible defect would
break. The reviewer looked specifically for a test whose expected value is
produced by the code under test and found only the self-comparison shape noted
above, which is paired with an independent check.

---

## G. Recurring defect shapes

### Shape 1, redundant declaration sites — **two instances found**

**1a. Two ways to order a `Candidate` that disagree on a tie.** `Candidate`
derives `PartialOrd` and `Ord` at `slots.rs:47`. The derived order is
lexicographic on `(rank, payload)`. `Slots::maximum` at `slots.rs:196` orders
on `rank` alone and gives the tie to the *lowest* payload. So
`slots.entries().iter().flatten().max()` and `slots.maximum()` return
different candidates whenever two ranks tie — which is the exact case the type
exists for. Nothing fails when they disagree. See finding 3.

**1b. The zero-count check is declared twice.** `world.rs:310` returns
`StepError::ZeroThreads`, and then `world.rs:324` maps `SlotError::ZeroSlots`
to the same error on a branch that cannot be reached. The same pattern is at
`sort.rs:175` and `sort.rs:188`. Both copies agree today. This is the benign
form of the shape — the second site is unreachable, not wrong — but it is
still two statements of one rule. Low severity.

### Shape 2, documents that rot — **one instance found**

`Cargo.toml:15` says the feature "makes the step join its output slots in
reverse order", and `docs/TESTING.md:84` repeats it. That was true before this
commit. It is now narrower than the truth: the switch perturbs every
`Slots::combine`, not only the step. See finding 4.

### Shape 3, inert code — **one instance found**

`sort.rs` is invoked by nothing in the engine. `world.rs` does not call it,
and no other module does. The only callers are its own tests. `Slots` avoided
this trap — `world::step` is a real engine caller and
`the_world_step_joins_its_slots_through_the_reduction` drives it. The sort has
no equivalent.

The reviewer does **not** raise this as a blocking finding. The sort is a
foundation primitive that the selector engine will consume, and building it
before the caller is a defensible sequence. But the rule says "do not declare a
capability before something calls it", and this commit does. Record it, watch
it, and treat it as overdue if the next two commits do not use it.

### Shape 4, nondeterminism the tests cannot see — **one instance found**

Finding 2. The sort's thread-count independence is asserted by a test that
provably cannot fail from a slot-order defect.

### Shape 5, a record that no longer describes the code — **none found**

The reviewer read ADR-0004 and ADR-0007 against the code decision by decision
(section A). No record statement is contradicted. The footnote citations in
both new files name the correct decision numbers; the reviewer checked each one
against the record text.

---

## H. The world refactor

**The log is correctly cleared and rebuilt.** `world.rs:338` takes the vector
with `core::mem::take`, leaving `self.log` as an empty `Vec` that owns no
allocation. Line 339 clears the taken vector, which drops the length to zero
and keeps the capacity. `combine` at line 340 extends it slot by slot in index
order and returns it, and line 340 assigns it back. The previous frame's events
are gone and the new frame's events are complete.

**No allocation leaks across frames.** The capacity survives the round trip, so
after a few frames the step stops allocating. This is a genuine improvement
over the previous `self.log.clear()` shape only in that it makes the reuse
explicit; the old code reused the capacity too. Nothing is leaked and nothing
is allocated twice.

One narrow observation, not a defect: between line 338 and line 340 the world
holds an empty log. If `combine` panicked, the world would survive with an
empty log rather than the previous frame's. `combine` folds
`extend_from_slice`, which panics only on capacity overflow. Not worth
changing.

**The zero-thread error path still works.** `world.rs:310` returns before any
allocation. `thread_equivalence.rs:108` asserts `world.step(0).is_err()`. The
reviewer confirmed the test exists and covers the path.

**Chunking is sound when threads exceed tiles.** `chunks_mut(chunk_len)`
yields at most `threads` chunks, so `zip` against `threads` slots never drops a
chunk. Surplus slots stay empty and contribute nothing to the join. The same
argument holds for `sort.rs:192`.

---

## Findings, ranked

### Finding 1 — MEDIUM. The proptest regression files are never read

`crates/cachette-core/tests/key_vector_sort.proptest-regressions` and
`slot_reduction.proptest-regressions`

The commit message says: "Both proptest-regressions files are checked in, as
proptest recommends, so the seeds that caught the mutations run first on every
future run." That is false in this repository. Proptest emits this on every
failing run:

```
proptest: FileFailurePersistence::SourceParallel set, but failed to find lib.rs or main.rs
```

The reviewer counted four occurrences in one probe run of `slot_reduction`.
Persistence resolves the source root by walking up from the test file looking
for `lib.rs` or `main.rs`. An integration test in `crates/cachette-core/tests/`
has neither above it, so persistence is disabled: proptest neither writes these
files nor reads them. The seeds do not run first. They do not run at all.

The same applies to the pre-existing `hex_geometry.proptest-regressions`, so
this is not a defect introduced by this commit — but the commit is the first to
assert the behaviour.

**Amendment.** Either configure `FileFailurePersistence::Direct` with an
explicit path in a `proptest!` config block, or delete the files and stop
claiming they run. Record the outcome in the findings register, because the
project now believes something false about its own test suite.

### Finding 2 — MEDIUM. The sort has no proven failure mode

`crates/cachette-core/src/sort.rs:201`, `crates/cachette-core/tests/key_vector_sort.rs`

`order_on` reads its slots through `runs.entries()`, not through
`Slots::combine`. `entries()` is not perturbed by the probe. The reviewer
verified the consequence:

```
cargo test -p cachette-core --features probe-nondeterminism --test key_vector_sort
-> test result: ok. 12 passed; 0 failed
```

The testing rule says a determinism test with no proven failure mode is
decoration. `the_order_does_not_depend_on_the_thread_count`
(`key_vector_sort.rs:62`) is currently in that category with respect to slot
order.

The order is nonetheless correct today, because the merge compares keys and the
keys are unique, so run order genuinely cannot change the answer. The problem is
that the test does not demonstrate this — it would pass either way.

**Amendment, either of:**

- Route `merge_runs` through `Slots::combine` so the probe reaches it, and add
  a probe assertion for the sort. This is the stronger fix; it also makes
  `combine` the single reader of slot order.
- Or state in the `sort.rs` module comment that the sort's result is
  independent of run order by construction, name the reason (unique keys plus a
  strict comparison at `sort.rs:282`), and say that the probe therefore does not
  apply. Then the absence of a failure mode is a claim, not an omission.

Related: the `justfile` `probe` recipe (line 60) and `.github/workflows/ci.yml`
run only `thread_equivalence` and `determinism_probe` under the feature. The
six `slot_reduction` failures that now prove the reduction's failure mode are
not part of the probe gate. Add
`! cargo test -p cachette-core --features probe-nondeterminism --test slot_reduction`
to the recipe so the new coverage is enforced rather than incidental.

### Finding 3 — MEDIUM. `Candidate` derives an `Ord` that disagrees with `maximum`

`crates/cachette-core/src/slots.rs:47`

`#[derive(..., PartialOrd, Ord, ...)]` gives `Candidate` a lexicographic order
on `(rank, payload)`. On a tied rank that order prefers the *highest* payload.
`Slots::maximum` prefers the *lowest*. Both are reachable, both look
authoritative, and nothing fails when they disagree. This is exactly the
recurring shape the project rule names: one fact declared in two places with no
check.

The derived `Ord` is used by nothing in the crate. The tests use only
`PartialEq`.

**Amendment.** Remove `PartialOrd` and `Ord` from the derive list at
`slots.rs:47`. A `Candidate` should be ordered through `Slots`, or not at all.
If a future caller needs an order, it should get it from a documented method
whose tie rule matches `minimum` and `maximum`.

### Finding 4 — LOW. Two comments describe the old perturbation switch

`crates/cachette-core/Cargo.toml:15` and `docs/TESTING.md:84`

Both say the feature "makes the step join its output slots in reverse order".
The switch now lives in `Slots::in_combine_order` and perturbs every combine,
including reductions the step does not perform. This is the document-rot shape.

**Amendment.** In both places, replace the sentence with: "It makes every slot
reduction read its slots in reverse index order, so any result that depends on
slot order changes." Then name the two gated tests.

### Finding 5 — LOW. An undocumented precondition guards `sort_unstable_by_key`

`crates/cachette-core/src/sort.rs:258`

`sorted_run` uses `sort_unstable_by_key`. That is deterministic only because
every key is unique, which is true only because `check_identifiers` ran at line
181. The doc comment on `sorted_run` (lines 249 to 255) explains that the sort
never calls content code, but says nothing about why an unstable sort is
acceptable. A future contributor who makes the identifier check optional, or who
moves it, removes a determinism guarantee without touching the line that depends
on it.

**Amendment.** Add one sentence to the `sorted_run` doc comment: "The keys are
unique, because the caller checked the identifiers first, so an unstable sort
has exactly one correct output."

### Finding 6 — LOW. `Slots` does not state the caller's obligation

`crates/cachette-core/src/slots.rs:62`

`Slots` guarantees index order. It does not guarantee that index order is the
stable data order — that holds only because both call sites assign chunks in
ascending data order. A caller that assigns chunk 3 to slot 0 would get a
deterministic answer that is nonetheless the wrong one, and every existing test
would pass. The type doc says a slot is "indexed by a stable key", which
describes the record's intent rather than what the type enforces.

**Amendment.** Add to the `Slots` doc comment: "The caller must assign the
slots in the order of the stable key, so that a lower slot index always holds a
lower key. The type cannot check this."

### Finding 7 — LOW. `check_identifiers` is a serial cost on every sort

`crates/cachette-core/src/sort.rs:238`

The check allocates a `Vec<u64>` of the input length and sorts it on one
thread, before the parallel work starts. At the target scale of one million
units that serial `O(n log n)` pass plus its allocation is likely to dominate
the parallel sort it guards. This is not a defect — the check is required by
finding C — but it is a cost that the module comment's promise of "a radix sort
later" does not account for.

No figure belongs in a record. Record the concern in the backlog item, and
measure it on the target platform before the selector engine depends on it.

### Finding 8 — LOW. `order_on` trusts an unbounded thread count

`crates/cachette-core/src/sort.rs:186`

`Slots::filled(threads, Vec::new())` allocates one `Vec` per requested thread,
and `merge_runs` at `sort.rs:274` is `O(items × runs)` over all slots including
the empty ones. A caller that passes a large `threads` with a small key set
allocates and scans proportional to `threads`, not to the data. The same is
true of `World::step`. No caller does this today.

Note only. No amendment required at this stage.

### Finding 9 — INFORMATIONAL. `sort.rs` is inert

Nothing in the engine calls `order`, `order_on`, or `sorted`. Only the tests
do. See shape 3 above. Not blocking; flagged so it is not forgotten.

### Finding 10 — INFORMATIONAL. The re-exports are asymmetric

`crates/cachette-core/src/lib.rs`

`Slots`, `Candidate` and `SlotError` are re-exported at the crate root.
`SortError` and `SortKey` are too, but `order`, `order_on`, `sorted`,
`descending` and `from_signed` are not, so a caller reaches half of the sort
through `cachette_core::` and half through `cachette_core::sort::`. Harmless.
Choose one.

---

## Objections the reviewer attempted and that failed

Recorded so that a later reader does not repeat them.

1. **"A closure can be smuggled into the sort."** Failed. Section B, six
   attempts.
2. **"`descending()` composed with `from_signed()` breaks the order."** Failed.
   `descending(from_signed(v))` is `!(v as u64 ^ 1<<63)`, a bijection that
   reverses the signed order. Correct.
3. **"`from_signed(i64::MIN)` wraps."** Failed. `(i64::MIN as u64) ^ (1<<63)`
   is `0`, which is the correct least element.
4. **"The tie rule can be violated silently by two equal keys."** Failed.
   `check_identifiers` refuses. Section C.
5. **"`chunks` can yield more chunks than slots and drop work."** Failed.
   `ceil(n / ceil(n/t)) <= t` for all positive `n`, `t`. `zip` never truncates
   a chunk.
6. **"`base += chunk.len() as u32` overflows."** Failed. `keys.len()` is
   checked against `u32::MAX` at `sort.rs:178` before the loop.
7. **"`merge_runs` can panic on `winner.expect`."** Failed. The loop runs
   `keys.len()` times and the runs hold `keys.len()` items in total, so a
   winner always exists.
8. **"`mem::take` loses the log on the error path."** Failed. Every error
   return in `step` precedes line 338.
9. **"Feature unification turns the probe on in a shipped build."** Failed. No
   other manifest in the workspace names the feature.
10. **"The step's answer depends on the thread count through the chunk
    boundaries."** Failed. Concatenating contiguous ascending ranges in slot
    order reproduces full index order for any chunking.
11. **"`best` lets an equal rank displace the earlier candidate."** Failed. The
    test at `slots.rs:184` and `slots.rs:197` is strict.

## What the reviewer did not check

- The reviewer did not benchmark. Finding 7 is a reasoned concern, not a
  measurement, and the project rule forbids claiming a measurement that was not
  taken.
- The reviewer did not run `cargo mutants` over the two new modules. That gate
  is not a commit gate, but it is the gate that would settle whether the twenty
  tests have teeth.
- The reviewer did not run the full `just check`. The individual test binaries,
  the probe build, and the perturbed builds were run.
