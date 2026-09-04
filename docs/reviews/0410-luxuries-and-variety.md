# Review 0410: Luxury resources and the variety score

## What was built

| Item | Value |
|---|---|
| Backlog item | 0410, seed luxury resources and score the variety over them |
| Product record | PRD-0032, a god knows what its ground is rich in, status `Shaped` |
| New module | `crates/cachette-core/src/luxury.rs` |
| New test files | `crates/cachette-core/tests/luxury.rs`, `tests/test_luxuries.py` |
| Registers | FND-420, FND-421, DEC-200, DEC-201, DEC-202, BLK-110, BLK-111 |
| New backlog items | 0411, 0412 |

The project owner asked for luxury resources that a caller seeds into a world
from Python, in any number, so that resource variety becomes a score. He said
that variety could change the worker policy of a faction, and he said that he
did not know.

## 1. The premise, checked

The dispatcher stated that the gatherable resource catalogue cannot be
extended at run time. **That is correct, and here is the evidence.**

The count of gatherable kinds is a constant that the compiler fixes. The
position module uses it as the length of two arrays: what one site wants of
each kind, and the table that maps a kind of work onto a commodity. The
resource module uses it as the length of four more, and the world uses it as
the length of two. Two compile-time assertions in the position module read it
as well.

The resource module also states, in its own prose, that the numbering is
stable because a state hash, an event and a sort key all read it. So
renumbering the kinds invalidates every golden file.

The premise is recorded as FND-420, with the evidence and what follows from
it.

## 2. The impact review

This section is section 1 of the definition of done, answered before the work
started.

**Which decision records govern this work.**

- **ADR-0001 D4.** The engine hashes the whole state each frame and compares
  the result against a stored file.
- **ADR-0002 D1.** No floating point in simulated or aggregated state.
- **ADR-0004 D1.** Iteration order is explicit.
- **ADR-0006 D1.** A stored value is plain data, and it declares its padding.
- **ADR-0022 D1.** Level 0 is the only truth. Every level above it is derived.
- **ADR-0023 D1, D2, D3 and D4.** An aggregate combines exactly, in any
  order, with an identity, a widened accumulator, and an inverse.
- **ADR-0024 D2.** Every summary field is extensive.
- **ADR-0040 D1.** Python is a control plane, not a data plane.
- **ADR-0053 D3.** A set of factions is one word.
- **ADR-0072 D3.** A kind is a row in a table, not a verb.
- **ADR-0090 D1.** A tile fact that no seed produces is stored sparsely.

**Does this work contradict a record?** One place needed care. ADR-0024 D2
says that every summary field is extensive, and the cell summary type carries
an inverse under ADR-0023 D4. A union of luxuries is idempotent and it has no
inverse, so it cannot join that type. **The work therefore does not touch the
cell summary at all.** Level 1 of the variety sits beside the summaries, in
the same way that the exit field and the return field do, and for the same
stated reason: two values that do not add do not belong in a type whose
combine is addition. No record changes.

**Does this work create a decision that no record holds?** Yes, one. Where a
luxury lives is a binding constraint, and it deserves a decision record. The
register holds the decision and the reasoning, and a backlog item holds the
record.[^1] [^2] **The number was not taken here on purpose.** The registry
allocates a record number, four other workers were writing at the same time,
and choosing a number without the registry is the collision this project has
already recorded three times.

**Is this blocked?** Two blockers were opened rather than answered. BLK-110
holds what the score should change. BLK-111 holds whether 64 luxuries is
enough. BLK-007 governs every cost figure, so this work states none.

**Has this been settled before?** FND-054 says that a fixture world below the
generator lattice spacing holds one ground everywhere, so the Rust fixture is
192 tiles on each side. FND-011 says that an unclamped accumulator banks
surplus that reaches the state hash; no accumulator here can bank anything,
because a population count is bounded by the word width.

## 3. Where a luxury lives, and why

**A luxury lives on a tile.**

**The ground carries it, and the ground does not move.** A luxury on a site
would move when the site moved. A settlement that was lost would take its
luxuries out of the world, and a region would then report a variety that
followed the building rather than the ground.

**Level 0 is the only truth, so the pyramid property needs a tile.** A level 1
cell equals the exact combination of the tiles it covers. That property is the
whole reason the score is a bit set and not a fraction, and it cannot be
stated at all if the fact lives on a site.

**Who holds a tile is already a level 0 fact.** The variety of a faction is
the union of the luxuries on the tiles that faction holds. That fold needs the
luxury on a tile.

**The storage argument for a site does not hold.** The engine stores one entry
for each tile that carries a luxury, and nothing else, in the same way it
stores an upgrade. A world in which nobody seeded a luxury holds no entry at
all. One dense word for every tile would cost 134 megabytes at the tile count
this project targets, and almost every word would be zero; the sparse table
costs 16 bytes for each seeded tile.

**A site holds no copy.** A site reads the luxuries of the tile it stands on.
Two homes for one fact, with nothing that fails when the two disagree, is the
defect shape this project meets most often.

The register holds this as DEC-201, with the two rejected options.

## 4. What consumes the variety

**Nothing in the engine consumes it. It is a read for the control plane.**

The doc comment of every read says so, in those words. So does the module doc
comment, and so does the Python prose that the reference publishes.

**Why nothing consumes it.** The rule on records forbids inventing a value
that an unanswered question governs. The project owner named no effect and
said that he did not know, so any rule written here would be invented, would
reach every world, and would carry a number that no measurement and no record
chose.

**This is not a capability that nothing invokes.** The rule on that defect
shape asks who is obligated to invoke the thing: the engine, or the user. For
a score, the user is obligated. The Rust test builds a world, seeds it, spawns
a garrison, steps the engine twelve times, and then reads the score through
the public interface. The Python test does the same through the published
package. The engine invokes the derivation of level 1 when the seed lands.

**The blocker names what would close it.**[^3] Item 0412 holds the work, and
it says plainly that it must not start while the blocker is open.

## 5. The limit, and the error at it

**The catalogue addresses 64 luxuries.** A set of luxuries is one 64-bit word,
so bit `n` stands for the luxury numbered `n`, and the word width is the
ceiling. The ceiling is a property of the word this project chose. It is not a
budget and no measurement moves it.

**A caller that names the luxury numbered 64 or higher gets a typed refusal.**
In Rust the seed returns `LuxuryError::IdAboveCeiling`, carrying the
identifier the caller gave. In Python the call raises `ConfigError`, and the
message names the ceiling and the identifier. **A refusal builds nothing**, so
the world is exactly where it was, and a test asserts that the state hash did
not move.

**The engine never folds an unaddressable luxury onto another bit.** A set of
factions does fold, onto an overflow bit, because the question a faction mask
answers is whether anybody holds the ground, and an overflow faction still
answers it. A luxury set answers how many *different* luxuries stand on the
ground. Two luxuries on one bit answer that question with the wrong number,
and nothing else would report the error. That is DEC-202.

**Whether 64 is enough is unanswered, and BLK-111 holds it.**[^4] A wider
catalogue needs a wider word or a second word. Both change what the engine
stores for each seeded tile, and both change the state hash, so the choice is
cheaper now than later.

## 6. The state hash, and why the golden files moved

**A luxury enters the state hash.** It is simulated state: two worlds that
carry different luxuries are different worlds, and no other input to the hash
reports the difference. The terrain and the resource stock are generated from
the seed, so the seed is their input; a luxury is authored, so nothing but the
field itself says what it is.

The field writes its entry count, then each entry as a tile index and a set,
in ascending tile order.

**Every golden file moved, from tick 0.** Eight scenario files changed, and
every line of each changed. The cause is the entry count: a world with no
luxury writes one zero into the hash where it previously wrote nothing, and
every later byte shifts.

**The count is in the hash on purpose.** A length that no hash reads is a
length that a defect changes in silence.

**The regeneration was verified.** The files were rebuilt from this source
with `just golden`. The determinism tests then ran at 1, 2 and 12 threads, and
the luxury test runs a seeded world four steps at each of those thread counts
and compares the whole state hash. All three agree.

## 7. The defects put back, and what caught each

Each defect below was put back into the source, one at a time. The test suite
ran after each one, and the source was restored. This is the exercise the
testing rule asks for.

| # | The defect | Caught | Tests that failed |
|---|---|---|---|
| D1 | The seed coalesces two placements on one tile by overwrite instead of union | Yes | 3, including the tile that carries several luxuries and the stepped world |
| D2 | Level 1 combines its tiles by overwrite instead of union | Yes | 3, including the faction read and the property test |
| D3 | The deposit total counts the union instead of the sum | Yes | 3, including one luxury on two tiles |
| D4 | The luxury field does not reach the state hash | Yes | 2, both hash tests |
| D5 | The seed does not sort the placements | Yes | 3, including the pyramid property |
| D6 | An identifier above the ceiling folds onto the top bit | Yes | 2, the ceiling test and the refused seed |
| D7 | Level 1 puts every tile into one cell | Yes | 5, including the pyramid property |
| D8 | The field invariant check is turned off | **No** | none |
| D9 | Level 1 drops the last entry of the field | Yes | 9, including the world invariant check |
| D10 | The deposit total truncates the per-tile count | **No** | none |

**D8 was not caught, and the reason is recorded as FND-421.** The luxury field
has one constructor. That constructor sorts, coalesces and refuses, so no
caller can build an unsorted field, a repeated tile or an empty entry. The
part of the invariant check that tests those three things is unreachable. It
stays as a guard for a future writer, and this report does not count it as
covered.

**D9 is the falsifiable half of the same check.** Dropping one entry from the
derivation made level 1 disagree with level 0, and the world invariant check
was one of the nine tests that failed. A check between two copies of one fact
is a test. A check on a value that no caller can build is not.

**D10 was not caught, and it could not be.** The deposit accumulator is 64
bits wide. The world this project targets holds 16,777,216 tiles and one tile
carries at most 64 luxuries, so the total reaches 1,073,741,824. A 32-bit
accumulator holds that, and it holds it only by a margin, which is exactly
what the hard invariant forbids depending on. **No test can trip the width**,
because a fixture would need about a billion deposits. The width is honoured
by construction, the reasoning is in the doc comment, and this report states
plainly that no test proves it.

## 8. The fixtures, and the extremes they supply

The testing rule says that a fixture which models the typical case supplies no
extreme. These are the extremes the tests build.

- **A world with no luxury.** Every read answers zero, the field is empty, and
  level 1 equals level 0 over an empty field.
- **A world that carries the whole catalogue on one tile.** The variety is the
  ceiling, the tile answers with every bit set, and one entry is stored.
- **The whole catalogue spread over 64 tiles.** The variety is the ceiling
  again, and the deposit count is the same, which separates the two questions.
- **A tile that carries several luxuries, including a repeat.** The repeat
  adds nothing, and one entry is stored.
- **One luxury on two tiles.** The variety is one and the deposits are two.
- **A tile outside the world, and a luxury above the ceiling.** Both refused,
  and the state hash does not move.

The property test draws up to 24 placements over 4096 tiles, seeds them
forward and backward, and asserts that the two fields are identical, that the
deposit total is the sum of the per-tile counts, that the world set is the
union of the per-tile sets, and that every cell of level 1 already holds the
luxuries of each tile under it.

## 9. Each governing decision, checked one at a time

**ADR-0001 D4, the whole state hashes each frame.** Honoured. The field enters
the hash. The derived level does not, because it holds no fact of its own, in
the same way that the pyramid does not.

**ADR-0002 D1, no floating point.** Honoured. Every value is a bit set, a
population count, a tile index or a 64-bit accumulator. The float ban script
passes over the module.

**ADR-0002 D2, arithmetic through the arithmetic module.** Honoured. Every
accumulator addition calls `sim_math::combine`. The union is a set operation
and not arithmetic on a simulated scalar, so it has no entry in that module
and needs none.

**ADR-0003 D1, every random draw is keyed.** Not applicable, and stated as
such in the module. Nothing here draws. A placement is authored content.

**ADR-0004 D1, iteration order is explicit.** Honoured. The seed sorts by tile
with a stable sort, the entries are held in tile order, the hash walks them in
that order, and the derivation of level 1 runs on one thread and writes each
cell once. Nothing here runs in parallel, so no thread completion order and no
work-stealing order can reach a result.

**ADR-0006 D1, plain data with declared padding.** Honoured. The stored entry
is `repr(C)`, it is `Pod`, it declares four padding bytes, and it uses no
`bool`. A test asserts the size, the alignment and that every padding byte is
zero.

**ADR-0022 D1, level 0 is the only truth.** Honoured. The derived level is a
pure function of the field and the block layout, it can be thrown away, and
the world invariant check compares it against the field on every rule.

**ADR-0023 D1 and D2, exact and order-free.** Honoured, and tested as a
property. The union has an identity, it is associative, it is commutative, and
it is idempotent.

**ADR-0023 D3, a widened accumulator.** Honoured by construction. Section 7
says that no test proves it.

**ADR-0023 D4, the combine has an inverse.** **Not honoured, and this is the
reason the union is not a summary field.** A union of luxuries cannot be
undone. The type that carries the inverse is the cell summary, and this work
adds nothing to it. The deposit count, which is extensive and does have an
inverse, is held beside the union.

**ADR-0024 D2, every summary field is extensive.** Honoured, by keeping the
union out of the summary type.

**ADR-0040 D1, Python is a control plane.** Honoured. The seed is one
set-valued call that takes every placement at once. Each read answers with one
fixed-width number. Nothing crosses the boundary once for each tile.

**ADR-0053 D3, a set is one word.** Honoured, with one deliberate departure
that DEC-202 records.

**ADR-0072 D3, a kind is a row and not a verb.** Honoured. A luxury is a
number. It adds no verb, it parameterises no verb, and the code holds no
branch on which luxury it is.

**ADR-0090 D1, stored sparsely.** Honoured. One entry for each tile that
carries a luxury, and nothing else.

## 10. The gates

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass, no warning |
| `cargo test --workspace` | Pass |
| `just determinism` | Pass, both tests |
| `just probe` | Pass |
| `just census` | Pass |
| `just records` | Pass |
| `just lint-python` | Pass |
| `just test-python` | Pass |
| `just docs` | Pass |
| `just docs-probe` | Pass |
| `just invariants` | Pass, the float ban and the crate split |

The Rust luxury tests run 24 cases, three of them properties. The Python
luxury tests run 12 cases.

## 11. What was left undone

**No decision record was written.** The registry allocates a record number,
and this work ran beside four other workers. Item 0411 holds the record, and
DEC-201 holds the reasoning until it is written.

**Nothing consumes the score.** That is a decision, not an omission, and
BLK-110 holds the question. Item 0412 holds the work and must not start while
the blocker is open.

**Two guards have no test.** The field invariant check cannot fail, and the
64-bit deposit accumulator cannot be tripped at a scale a test reaches.
Section 7 gives both, and FND-421 records the first.

**No cost figure was measured.** BLK-007 governs every cost figure in this
project. The storage claims in this report are derivations from the stored
shape, and the byte figures come from the size of the stored entry and the
tile count in the scale constants table. Nothing here was run on the target
platform.

**The Python seed is a method and not a constructor argument.** The project
owner asked for the luxuries to be seeded at construction. The constructor of
the Python world is a function that four other workers were editing at the
same time, and the instruction for this work forbade changing it. The method
refuses a second call, so the field is still fixed for the life of the world,
and a caller seeds it on the line after it builds the world. A later change
may fold it into the constructor.

**Trades, contracts, combat, movement orders, the presence relation and the
build verbs were not touched.** Other workers own them.

## References

[^1]: Decisions register, DEC-201. `docs/DECISIONS.md`
[^2]: Backlog item 0411, record where a luxury lives. `docs/backlog/proposed/0411-record-where-a-luxury-lives.md`
[^3]: Blockers register, BLK-110. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-111. `docs/BLOCKERS.md`
