# The Presence Relation, and the Holder Column

A working report on two backlog items. Written 3 September 2026.[^1] [^2]

Cachette is a world simulation engine. The core is Rust and the control plane is
Python. A downstream game gates a conversation between two players on presence:
one may speak to another only while one of its own units stands in the other's
territory. This report records what the work built, what it corrected, what it
proved by putting defects back, and what it left undone.

## 1. The architectural impact review

### 1.1 Which records govern the work

**ADR-0053, a faction is a bit in a mask, and a relation is a plane.** D7 fixes
the shape of every relation between factions: one mask row for each faction,
never a field over the world. It states that no code stored a relation yet, and
that the first one must take that shape. The presence relation is that first
one, and it takes that shape. D1 makes a faction one bit of a 64-bit word, so a
row is one word. D2 gives a tile one holder, so the fold reads one value for
each unit. D3 forbids a field of the world indexed by the faction, which the
relation is not. D4 keeps a running total for each faction, which is the cheap
answer to "how much ground", and is why the relation drops its own diagonal.

**ADR-0023, an aggregate combines exactly, in any order.** D1, D2 and D3 ask
for an associative, commutative and exact combine. A bitwise union of sets is
all three, with zero as its identity.

**ADR-0009, parallel stages write disjoint outputs.** D1: each thread of the
fold writes its own row array and no other. D2: the join reads the slots in
slot order and never in completion order. D3: the partition comes from the slot
count and the thread count.

**ADR-0004, iteration order is explicit.** D1: the fold walks the arena
columns in slot order.

**ADR-0001, one binary gives one answer at any thread count.** D1 is the claim
the fold must not break, and D4 names the two tests that protect it.

**ADR-0040, Python is a control plane, not a data plane.** D1 and D2: the
number of crossings must not grow with the number of entities. Both new reads
cross once.

**ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the
barrier.** D3 supplies the freshness pattern that the relation copies: record
the arena identity and the arena revision, and refuse a read against a changed
arena.

**ADR-0022, level 0 is the only truth.** D2: a derived thing is derived again
and never stored. The relation reaches no state hash.

**ADR-0107, the Python reference is generated from the compiled module.** D2:
the prose of a published member lives in the Rust doc comment. Every word a
Python reader sees for the three new calls is written in the bindings crate.

**ADR-0044, what copies and what does not is declared at the call site.** The
holder column read says that it copies.

### 1.2 Whether the work contradicts a record

It does not. ADR-0053 D7 anticipated it and fixed the shape. No record was
superseded and no record was changed.

### 1.3 Which record the work creates

**ADR-0111, the presence relation is derived at the end of the step and never
stored as a fact.**[^3] The registry row was added before the record was
written.

The three-condition test of the scope rule passes. A future contributor could
reasonably maintain a presence set incrementally, from every rule that moves a
unit or changes a holder. That choice costs determinism and creates one
declaration site for each such rule, and it is expensive to undo. The reasoning
is not visible in the code. The counter-test also applies: the decision governs
determinism, which is the property this project cannot recover.

Item 0347 creates no record. The first condition fails: a dense column already
exists, and returning it as one array is the only workable shape.

### 1.4 Whether the work is blocked

**BLK-050 is open and it does not stop this.** It holds the rules of the
downstream game, including whether the messaging rule is symmetric. The relation
is directed, so it answers either reading: a caller asks one direction, or reads
both rows and combines them. Nothing here invents a game rule.

**BLK-007 governs every cost figure in this project.** No figure in this work
was measured. The stage is named in the stage table, so a benchmark on the
target platform will price it beside every other stage.

**BLK-036 does not reach this.** It asks whether a built thing changes hands
with the ground. The relation reads the holder of the ground and never what
stands on it.

### 1.5 Whether this was settled before

FND-361 and FND-362 hold the reasoning that produced the items. DEC-141 held
the choice and recommended what the work built. Nothing in the findings register
contradicted the plan.

### 1.6 Are the two items one change or two?

**Two.** They share no code. The presence relation reads the holder column
inside Rust and never crosses it. The holder column read copies that column
across the boundary and knows nothing about units. They were built together
because they serve one product record, and each stands alone. Neither needs the
other.

## 2. The shape that was chosen, and why

### 2.1 A bitmask for each faction

**The relation is 63 rows of one 64-bit word.** Row `host` names every faction
that has a live unit standing on a tile that `host` holds.

The record already required this shape, and the reason it gives is the one that
holds here. A relation is the faction count squared in bits, which stays in the
data cache of one core. A relation expressed over the world would be the world
multiplied by the faction count squared, which is not storable. The size of the
answer follows the faction ceiling and nothing else, so a large population and a
large world cannot make the read slow.

### 2.2 The relation is exact, and not an over-approximation

The fold reads the holder of the exact tile that each live unit stands on. No
block mask, no summarised cell and no bounding shape reaches the answer. A set
bit names a unit that is genuinely there. A clear bit means that no unit is.

**This is stated in the module doc comment, in the record, and in the published
Python prose.** A god that cannot message when it should is a defect a player
can see, so an over-approximation in the permissive direction would be a game
defect and an over-approximation in the strict direction would be worse.

### 2.3 The relation has no diagonal

A unit that stands on ground its own faction holds sets no bit. The product
record asks whether the people of one side stand on the ground of **another**
side, and a side never asks the question of itself.[^4] A caller that wants to know
how much ground a faction holds reads the running total, which reads no tile.

The consequence is that the diagonal is always empty, and one of the tests
asserts exactly that.

### 2.4 The fold runs last in the step, and not at the barrier

**The research report said the derivation rides on the bridge rebuild. It
cannot.**[^5] The bridge rebuild runs at the frame barrier, which is above the
holding spread and above the starvation reap. A fold there would read the
holders of the previous tick, and would name a unit the same frame ends, which
would make every read for the rest of the frame refuse as stale.

The fold is therefore its own stage, last in the step. FND-370 records the
correction. The cost argument the report made survives: the fold reads three
unit columns and one tile column once, and allocates one row array for each
thread.

### 2.5 What fixes the order

**The combine is a bitwise union, so the answer does not depend on the
partition.** That alone is the determinism argument.

The fold also writes disjoint outputs and joins them in slot order, in the same
shape as the holding candidate pass. Each thread folds a contiguous run of arena
slots into its own row array. The join reads the slots in slot order. Nothing
reads which thread finished first. The partition is a function of the slot count
and the thread count.

### 2.6 The Python boundary

Three calls were added. Each crosses once.

`presence_masks()` returns one array of `numpy.uint64`, one entry for each
faction the world was built with. A caller answers "which gods may I message" by
reading that array and testing 63 bits in Python, with no crossing for each
unit.

`stands_in_territory(guest, host)` returns a `bool` for one pair. It raises
`ViewError` and names the number when the world holds no such faction, which is
the checkable statement the product record asked for.

`tile_holders()` returns one array of `numpy.uint16`, one entry for each tile in
row-major order, with 65535 for a tile nobody holds. That value can never name a
faction, because a world holds at most 63 of them.

**Every word of the published prose lives in the Rust doc comment**, as the
record on the provenance of the prose requires. The prose states the element
type, the shape, the meaning of each entry, the error each call raises, and the
two properties a reader must not guess: that the diagonal is empty, and that the
answer is exact.

**A stale read is refused, not answered.** A caller that spawns or despawns a
unit and then reads meets `ViewError`. The product record asked for that in so
many words.

## 3. The defects that were put back

A green suite is not evidence. Each defect below was written into the source,
the tests were run, and the source was restored.

| The defect | Caught by |
|---|---|
| The diagonal is set: a unit on its own ground sets a bit | Two Rust tests failed |
| Any unit anywhere sets a bit in every row, ignoring the holder | Three Rust tests failed |
| Liveness is ignored: a dead unit still counts | One Rust test failed |
| The freshness guard always passes | One Rust test failed |
| The join takes the last slot instead of the union | **Nothing failed at first** |

**The fifth defect was not caught, and that is the most useful result here.**
The thread-count test spawned the guest last, so the guest held the highest
arena slot and sat in the last chunk at every thread count. A join that took the
last chunk and dropped every other one gave the right answer, and the comparison
passed. The fixture now spawns a further batch of units after the guest, so the
guest sits in a middle chunk. The same defect then fails the test. FND-373
records it.

Two more defects were put into the bindings crate: the holder column returns
65535 for every tile, and the faction error message drops the number. Section 5
gives the result.

## 4. The determinism tests

Both determinism tests were run and both pass. The thread-count test runs every
scenario at 1, 2 and 12 threads and compares the event logs byte for byte.

**The golden state hash did not move.** That is the expected result and it is
also evidence: the relation is derived and reaches no state, so it reaches no
hash. A hash that had moved would have said that the relation was stored. FND-371
records the check.

The presence relation has its own thread-count test as well, at 1, 2 and 12
threads, because the two protected tests compare event logs and state hashes and
the relation appears in neither.

## 5. The gates

Every command below was run from the branch. The table gives what each one
reported.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `uv run ruff format --check python tests` | 51 files already formatted |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass, after one length comparison was rewritten |
| `cargo test --workspace` | 40 test binaries, every one ok, 0 failures |
| `just determinism` | thread equivalence 14 ok, golden state hash 2 ok |
| `just probe` | every determinism test failed under the perturbation, and the probe binary passed |
| `just census` | 2 ok |
| `just records` | 74 records 0 failures, 26 product records 0 failures, 236 backlog items 0 failures, 180 priority rows 0 failures, 6490 citations 0 failures, 715 files 0 conflict markers, 409 documents 0 footnote failures |
| `just records-probe` | every broken fixture rejected, 8 moved-path cases 0 failures |
| `just merge-defects` | 2 moved, 0 not searched, 15 documents written, 0 failures |
| `just invariants` | the float ban passed and the crate split passed |
| `just lint-python` | ruff all checks passed, mypy no issues in 23 source files |
| `just test-python` | 103 passed |
| `just smoke` | the installed package built a world and stepped it |
| `just docs` | 62 members with prose, 0 without, and every one of the 62 summaries reached the site |
| `just docs-probe` | both cases failed the job, as they must |

The reference carried 59 members before this work and it carries 62 now. The
three new members are the two presence reads and the holder column read.

`just miri` and `just test-slow` were not run. Neither is a commit gate, and
this work adds no unsafe code and no dependency.

## 6. The registers

**FND-370.** The presence relation cannot ride on the bridge rebuild, because
that rebuild runs before the holding spreads.

**FND-371.** A derived relation moves no golden state hash, and that is the
check that it is derived.

**FND-372.** One visiting unit takes the tile it visits, so a naive presence
fixture measures the spread rule rather than the relation.

**FND-373.** A fixture whose guest sits in the last chunk cannot catch a
combine that is not order-free.

**DEC-141 closed.** Option C, with option A built. The row records what the
build changed about option A: the fold does not run at the barrier.

**DEC-150 opened.** Should the presence fold run on one thread, as the bridge
rebuild does? The recommendation is to wait for a measurement on the target
platform.

**No blocker opened and none closed.** Two blocker numbers were allocated to
this work and neither was spent. The cost of the fold is unmeasured, and one
blocker already governs every cost figure in this project, so a second row would
be the same fact in two places.[^6]

**No backlog item created.** Numbers 0360 to 0364 were allocated and none was
spent. The set-valued form of the question already has an item, and the stage
declaration already has one.

## 7. What was left undone

**Nothing was measured.** No figure in this work is a measurement, and none is a
target platform figure. The stage is named, so a benchmark will price it.

**The relation reaches no panel and no viewer.** A watcher of a territorial game
cannot see it. That is outside the two items.

**The set-valued form is not built.** A caller cannot ask which units are
standing there. That waits for the selector, which does not exist.

**The relation is not in the state hash and must not be.** A reader who wants a
replay of the diplomatic state derives it again from the columns.

## References

[^1]: Backlog item 0340, answer whether one faction stands in the territory of another, in one call. `docs/backlog/complete/0340-answer-whether-one-faction-stands-in-the-territory-of-another.md`
[^2]: Backlog item 0347, read the tile holder as a column. `docs/backlog/complete/0347-read-the-tile-holder-as-a-column.md`
[^3]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^4]: PRD-0031, a god knows whose ground its people stand on. `docs/product/shaped/prd-0031-a-god-knows-whose-ground-its-people-stand-on.md`
[^5]: Research report 21, what a god needs from this engine. `docs/research/reports/21-what-a-god-needs.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
