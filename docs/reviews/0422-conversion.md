# Review of item 0422 — a god takes the people of another god

This review states what was built for conversion, what it decides, what proves
it, and what is left undone. The item file holds the plan.[^1]

## The impact review

**Governing records, decision by decision.**

- **ADR-0001 D1**, one binary gives one answer at any thread count. The pass
  marks in parallel and applies in one ascending scan of the arena slots. A new
  thread-count scenario runs a converting world at 1, 2 and 12 threads and
  compares the conversion log byte for byte. **Honoured.**
- **ADR-0001 D3**, no convergence test. The pass walks every block of the
  derived unit structure once. It reads no clock and tests no residual.
  **Honoured.**
- **ADR-0002 D1**, no floating point. Every value is an integer. The margin is
  a 16-bit reach difference, the count is a 128-bit product divided by a
  reference, and the accumulator is 64 bits wide. **Honoured.**
- **ADR-0002 D2**, arithmetic goes through the arithmetic module. The count
  that converts uses the share of the module, and the remainder uses a new
  operation in the same module that pairs with it. **Honoured.**
- **ADR-0003 D1**, every draw is keyed. The pass owns a system identifier that
  no other pass shares, and it keys on the tuple of the system, the frame, the
  tile and a draw index that names the faction losing the unit. A test changes
  each field of the key and asserts the draw changes. **Honoured.**
- **ADR-0004 D1**, iteration order is explicit. The marks are sorted on the
  arena slot before anything is applied, and no two marks name one slot.
  **Honoured.**
- **ADR-0006 D1**, an event is plain data. The conversion event is `repr(C)`,
  `Pod`, holds no boolean, and its fields fill the type exactly, so it declares
  no padding. A layout test asserts the size against the sum of the field
  sizes. **Honoured.**
- **ADR-0006 D2**, events cross at the barrier. The log covers the last step
  and crosses in one call. **Honoured.**
- **ADR-0009 D1**, parallel stages write disjoint outputs. Each thread owns its
  own mark list and a contiguous run of blocks. **Honoured.**
- **ADR-0014 D1 and D2**, an identity is a slot and a generation. A convert
  keeps both, so every handle still names the same unit. A test converts a set
  twice using the same identities. **Honoured.**
- **ADR-0018 D2 and D3**, the derived unit structure and the barrier. The pass
  asserts that the structure describes the arena before it walks it, and it
  refreshes the structure after it applies, because the faction change raises
  the arena revision. **Honoured.**
- **ADR-0065 D1**, a cohort is the units of one faction at one site. The apply
  derives the cohort table again, and a test asserts that the cohorts still
  describe the units on every frame of a converting run. **Honoured.**
- **ADR-0070 D1**, the per-faction population is maintained rather than walked.
  The arena moves its own count inside the faction setter, and its own check
  recounts and compares. **Honoured.**
- **ADR-0072 D5**, conservation. A convert keeps its carried load, so nothing
  leaves the world. A test asserts the load is unchanged. **Honoured.**
- **ADR-0087 D1**, the influence solve runs a fixed pass count. The conversion
  pass reads the field and never solves it. **Honoured.**
- **ADR-0111 D2 and D4**, the presence relation. The pass runs before the fold,
  and the faction change raises the arena revision so a read taken between a
  conversion and a step is refused rather than answered. **Honoured.**

**Contradictions.** None. No record changes.

**Records created.** Three, and the item file argues each one against the three
conditions of the scope rule.[^1] [^2]

**Blockers.** Two opened, and neither stops work.[^3]

**Findings.** Two recorded.[^4]

## What triggers conversion

**Belief is the influence field, and a unit converts to the faction that leads
that field at its cell.**

The engine already holds a field that carries the reach of a faction across the
world. The control plane sets a source at a place, and a solve of a fixed pass
count spreads it. A second field for belief would be a second declaration of
one idea, and nothing would fail when the two disagreed.

The pass reads the level 1 cell that covers each occupied tile, finds the
faction that reaches it most strongly, and takes units from every other faction
in proportion to the margin. A tie leads to nothing, and the tie between two
equal leaders breaks on the lower faction number. A cell that no faction
reaches converts nobody, which is the ordinary case in a world whose control
plane set no source.

**A god causes it deliberately in two ways.** It sets an influence source,
which is slow, spatial and visible to everybody. Or it calls a set-valued verb
on named units, which is immediate and all or nothing. Both apply through one
function, so the two cannot disagree about what a conversion does.

**The randomness follows the contest.** The pass takes two draws for each group
on a tile and never one for each unit. One draw decides whether the remainder
of the exact count converts one more unit. The other rotates the ordinals of
the group, and a rotation is a bijection, so exactly as many units convert as
the margin paid for.

**Conversion is not gated on territory.** The field is already the gate,
because a faction reaches a cell only where the control plane put a source and
the ground let it spread. The presence gate that the trade verbs use was
rejected: a god that already holds the ground has less need to convert the
people on it, and a god that does not could never start.

**A convert keeps its type, its carried load, its tile, its home site and its
seat.** It loses its gather order, its build order and its destination, because
an order is an instruction from the faction that no longer holds it. A unit that
carries a character takes that character with it.

## How every per-faction total stays correct

The tree was searched for every derivation of a per-faction total. Four exist,
and each is accounted for.

**The maintained live count of the unit arena.** The arena keeps one count for
each faction. It had two write sites, where a slot becomes live and where it
stops being live. Conversion is a third, and it is neither a birth nor a death.
The faction setter moves the count itself, inside the same function that writes
the column, so no caller can write one without the other. The arena check
recounts the live column and compares, and it fails when the two disagree.

**The cohort table.** A cohort is the units of one faction that belong to one
site, so a faction change moves a unit from one row to another. The apply
derives the whole table again, in the same way the reap and the contest do. The
world holds a check that derives it again and compares, and a test calls that
check on every frame of a converting run.

**The presence relation.** It is folded from the faction column at the end of
every step, and the conversion pass runs before the fold. For a caller that
converts outside a step, the faction setter raises the arena revision, so the
relation refuses the read rather than answering from an arena that has moved
on. A test asserts that refusal.

**The variety score of a faction.** It folds over the luxuries on the tiles a
faction holds, and it reads the holder column rather than any unit. Conversion
moves people and never ground, so the score is unaffected.

The level 1 summary was checked and holds no per-faction term. Its unit count
is a total over all factions, which a conversion does not change.

## What stops a flip loop

**Strict dominance, and nothing else.** A unit converts only where another
faction leads its own strictly. After the change the leader is its own faction,
so the margin against it is zero and the unit cannot convert again while the
field stands still. A second conversion needs the field itself to move, and the
field moves only when the control plane moves a source or the ground changes.

The rule is antisymmetric by construction, so it needs no state on the unit. A
cooldown counter would be a column that every unit carries so that a rare case
behaves, and it would enter the state hash. A margin threshold would be a
tuning value that decides behaviour, so it would enter the state hash too, and
it buys nothing that strict dominance does not already give.

A test runs a converting world for forty-eight frames and asserts that no event
ever reports a unit leaving the leading faction, and that a settled world emits
an empty log.

**The engine does not police the control plane.** Two gods that call the verb
on one unit on alternate frames make that unit flip on alternate frames. That
is the control plane doing it on purpose.

## What a god reads to see it happen

**One event for each unit that changed hands.** The event names the frame, the
identity of the unit, the tile it stood on, the faction it left and the faction
it joined. The log covers the last step alone, it holds the conversions the
field decided and the conversions the verb asked for, and it crosses to Python
as columns in one call.

**The counts a god already reads.** The live count for each faction moves with
every conversion, and the cohort headcount for each site and faction moves with
it. No new aggregate was added, because a second statement of a count that
already exists is the defect shape this project meets most often.

**The field itself.** The reach of a faction at a place is now readable from
Python, so a god can see where the next conversion is likely before it happens.
The engine states no prediction and no explanation, because the leader and the
margin are both readable at the cell.

## The defects that were put back

Each defect was written back into the tree, the test that must catch it was
run, and the tree was restored. Two of them were not caught the first time, and
both gaps are recorded here rather than repaired in silence.

| The defect | Caught by | Outcome |
|---|---|---|
| The faction setter does not move the maintained per-faction count | `a_unit_changes_faction` | **Caught.** Five of the nine tests failed. |
| The apply does not derive the cohort table again | `a_unit_changes_faction` | **Caught.** One test failed. |
| The faction setter does not raise the arena revision | `a_unit_changes_faction` | **Not caught at first.** The step folds the presence relation unconditionally, so every assertion taken after a step passed. The test now reads the relation between a conversion and a step, and asserts the refusal. The defect then failed one test. |
| The remainder draw drops the frame from its key | `a_unit_changes_faction` | **Caught.** The key test failed. |
| The gather order survives the conversion | `a_unit_changes_faction` | **Caught.** One test failed. |
| The marks keep the order the threads produced them in, and the join reads the thread lists in reverse | `thread_equivalence` | **Not caught at first.** Every scenario of that test set no influence source, so it converted nobody and ran the pass over an empty result. A converting scenario was added, and it still did not catch the defect, because its units were seated on a contiguous patch that fell inside one block, so one thread produced every mark. The fixture now strides its tiles over the whole world, and the defect then failed the scenario. |

**One defect could not be made to fail.** Removing the sort alone, and leaving
the join reading the thread lists in order, changes nothing observable. Each
thread takes a contiguous ascending run of blocks and the join reads the
threads in order, so the unsorted list is already in block order and already
independent of the thread count. The sort is therefore a stronger guarantee
than the current partition needs, and it is what makes the join order
irrelevant. The paired experiment shows this: with the sort in place, reversing
the join changes nothing, and with the sort removed, reversing the join breaks
the scenario.

## The golden state hash

**It did not move.** The golden test passed unchanged.

Conversion adds no state. The pass writes the faction column, which the arena
already folds into the hash, and it adds no parameter, no schedule and no
counter. A world converts nobody unless its control plane set an influence
source, and no golden scenario sets one, so no golden world converts anybody
and every recorded hash still describes its world.

This is the reason no tuning threshold was added. A threshold that decides
behaviour would have entered the hash by the precedent of every other world
parameter, and every golden file would have had to be recorded again for a
value nobody has measured.

## The gates

| Gate | Outcome |
|---|---|
| `cargo fmt --all` | Clean. `cargo fmt --all -- --check` reports nothing. |
| `cargo clippy --workspace --all-targets -- -D warnings` | No warning and no error. |
| `cargo test --workspace` | Green. |
| Thread-count equivalence | Green, including a new converting scenario at 1, 2 and 12 threads. |
| Golden state hash | Green. No file changed. |
| `just invariants` | The float ban and the crate split both pass. |
| `just census` | Two tests pass. |
| `just records` | Every check passes. The record check reports two uncited records, which it reports without failing and which this work did not touch. |
| `just lint-python` | `ruff check` reports "All checks passed". `mypy` reports "Success: no issues found in 26 source files". |
| `uv run ruff format --check python tests` | "54 files already formatted". |
| `just test-python` | "169 passed". |
| `just docs` | "cachette._core: 99 members with prose, 0 without", and every summary reached the site. |
| `just docs-probe` | "both cases failed the job", which is what the probe requires. |

## What is left undone

**The cost of the pass is derived, not measured.** The pass opens a stage, so
the stage cost table names it, but no benchmark was run on the target platform.
Item 0423 holds that work, and it names the two terms worth measuring: the
per-cell read multiplied by the faction count, and the rebuild of the derived
unit structure that a converting frame forces.

**A convert may hold a seat at a site of its old faction** until that site
opens its seats again. The engine states no rule that removes it earlier,
because a rule that did would be a placement rule inside a conversion. BLK-123
holds the question.

**The engine charges nothing for belief.** BLK-122 holds what a god should pay,
and it names the one answer that would change the shape of the pass rather than
add a term to it.

**Six of the allocated numbers were not spent.** Findings 435 to 442, decisions
218 to 223, backlog items 0424 to 0431 and blocker 124 onward were not taken.
The registers state the next free number, so nothing is stranded.

**The viewer shows no conversion.** The panel reads the logs it already knows
about, and the conversion log was not added to it. That is a viewer item and
this work did not open one for it.

## References

[^1]: Backlog item 0422, let a god take the people of another god. `docs/backlog/complete/0422-let-a-god-take-the-people-of-another-god.md`
[^2]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^3]: Blockers register, BLK-122 and BLK-123. `docs/BLOCKERS.md`
[^4]: Findings register, FND-433 and FND-434. `docs/FINDINGS.md`
