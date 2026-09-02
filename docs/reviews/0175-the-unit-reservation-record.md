# Review 0175: The unit reservation record

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md` | Status `Draft` at review, and `Draft` after it |
| Commit | `506d3d1`, the head of `main` at the time of the review |
| Code read | the soldier arena, the world settings, the world, the founding path, the state hash function, the settlement arena, the character arena, and the reservation test |

The reviewer did not write ADR-0084 and did not write the change it
accompanies. The reviewer read the record, the rules, and the tree. The
reviewer did not read the reasoning of the author, who was not available.

**The reviewer compiled nothing.** Three other workers held the machine, so no
`cargo` command ran. Section 5 names every claim that only a run can settle.
Each of those is marked unverified and is not counted toward the verdict.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0084 | ACCEPT WITH AMENDMENT. The record stays in `draft/`. Section 3 holds the exact text to change. |

The constraint is sound. The title states a claim. All four decisions have code
behind them, and each is reachable from the public interface. The body holds no
version pin, no count, no file table and no measured figure. The record is 1453
words, below both reference medians for a foundational record.

Two sentences are false against the tree. One of the two states the opposite of
what the finding beside it records, and the test that exercises that path
corrects the phrasing in its own comment two lines below the same claim. A
record the code contradicts is worse than no record, because it lies.[^1]

Each of the two is a sentence, not a decision. The record needs an edit, not a
rewrite. The reviewer did not make the edit, because writing a record into
acceptability is authoring and not reviewing.

## 1. The record against the code, decision by decision

**D1 holds.** The arena constructor takes a capacity and calls
`Vec::with_capacity` for every unit column and `VecDeque::with_capacity` for the
free queue. The columns are the ten the arena declares, and the queue is
reserved alongside them, which is what D1 asks for by name.

The copy claim holds. The arena writes its own clone rather than deriving one,
through a helper that allocates the reserved capacity and then copies the
entries. The record says the derived copy would allocate for what a column
holds, and the code carries that argument in a comment above the helper.

The hash claim holds and was checked directly against the hash function. The
function writes the slot count, the live count, the retired count, each column,
each generation and each entry of the free queue. It writes no capacity.
A reservation therefore moves no hash, exactly as D1 states.

**D2 holds in its substance and fails in one sentence.** The reservation is a
field of the world settings. The arena has one constructor, it takes the value
it is given, and it declares no default. There is no `Default` implementation on
the arena, so there is no second site for a reader to consult. The failing
sentence is in section 3.

**D3 holds.** The arena refuses at the point where it would open a slot past the
capacity, and it returns the typed refusal it already carried rather than a
panic. The founding wraps that refusal in a variant of its own, so a caller sees
which member of the group was refused and why.

The single-undo claim holds. The founding has two paths that can stop after the
settlement stands, and both call the same function, which despawns the people it
seated and then destroys the settlement. There is no second undo written at a
second return.

**D4 holds.** The arena has one constructor and no growth path. The function
that opens a slot compares against the capacity and returns before it pushes.
Nothing in the arena reallocates a column after construction, so the engine
holds one answer rather than two.

## 2. What the record gets right that a reader should not have to rediscover

**The record names no population.** This was checked first, because it is the
category the scope rule bans by name. The record says "the target population the
project owner answered" and cites the scale constants table. It states no
figure, no byte budget and no percentage. The blocker that owns every cost
figure is cited where the record declines to say what the reservation costs.

**The record does not extend the open row.** Its consequences say the settlement
arena and the character arena still grow, that both hold the same shape, and
that an open row states the question they raise. It decides nothing about
either. The code agrees: both arenas build their columns with `Vec::new`. A
record that quietly extended the owner's decision would be a defect, and this
one does not.

**The record states the shape of the cost and no figure.** It says a
reallocation lands inside a step at a moment nobody chose, and that nobody can
say what the copy costs. That is the correct treatment of a value the blocker
governs.

## 3. What must change

### 3.1 A refused founding does leave something behind

The record states, in bold:

> **A founding that a refusal stops leaves nothing behind.**

This is false, and the project has already recorded that it is false. The undo
owes that nothing lives and nothing stands. It does not owe, and cannot give,
the state the world held before. The arena never compacts the slot index space
and a generation never rewinds, so the slots the founding opened stay open and
their generations stay advanced.[^2]

That is not a subtlety a reader may be left to find. The state hash writes the
slot count, every generation and the free queue, so a refused founding moves the
hash deterministically. A reader who takes the record at its word and then reads
a moved golden file will look for a defect that is not there.

The record's own next sentence is correct and precise: it says every refusal
after the settlement stands goes through one path that undoes both. The heading
sentence is the only thing that overreaches.

The test that exercises this path already states the true outcome in its own
comment, two lines below a test name that repeats the record's phrasing. The
record should say what the test says.

**Replace:**

> **A founding that a refusal stops leaves nothing behind.**

**With:**

> **A founding that a refusal stops leaves nothing alive and nothing standing.
> It does not restore the state the world held before.**

and add, after the sentence that names the one undo path:

> The undo owes that nothing lives and nothing stands. It does not owe the
> state hash, because the arena never compacts the slot index space and a
> generation never rewinds. A refused founding therefore moves the hash,
> deterministically, and that is the arena rule rather than a defect.

### 3.2 The code does state a population of its own

The record states:

> The reference table holds the value, and the settings cite it. The code
> states no population of its own.

The second sentence is false as a reader will use it. The settings hold a public
constant whose value is one million, written as a literal, and the scale
constants table holds the same number in a row of its own. Nothing fails when
the two disagree. That is the declaration-site shape the record raises by name
in its own forces section, one level above the arena that the record correctly
rules out.

The reviewer searched the check scripts for a rule that ties the constant to the
table and found none.

The record's substance survives. There is one site a caller changes, and the
arena is not a second one. What must go is the claim that the code states no
population, because a reader who believes it will change the table and expect
the engine to follow.

**Replace:**

> The code states no population of its own.

**With:**

> The settings hold that value and the reference table holds it, so the two
> are copies of one fact and no check compares them. The arena is not a third
> copy, which is the site that would have mattered, because a caller reads the
> reservation back from the settings it gave.

## 4. Every objection the reviewer attempted

**Objection 1: the record holds two claims and should be split.** The title
names a reservation and a refusal, and the scope rule says to split a record
that states two claims which could be accepted separately. **Failed.** D4 argues
that the two are one claim: a world that reserves and then grows past the
reservation holds both behaviours and therefore holds two answers to one
question. Under that argument the reservation is the bound, and the refusal is
not separable from it. The record makes this argument explicitly rather than
leaving it to the reader.

**Objection 2: the record should cite the resolved blocker rather than the
table.** **Failed.** The blocker that asked whether one million is the whole
population is resolved, and the scope rule's parametric requirement applies to
an unanswered question. The reference table is the register that owns a value a
measurement can change, and the table's own row names the blocker that answered
it. Citing the table is the stronger form.

**Objection 3: the record extends the open row that holds the settlement and
character arenas.** **Failed.** The record names both, says neither is in the
closed row it implements, and cites the open row. It decides nothing about
either, and it notes that the character arena raises a second question because
its ceiling is larger than its target. That is exactly the treatment the open
row asks for.

**Objection 4: "the reservation is a field of the world settings" records a
module arrangement, which section 4.4 bans.** **Failed.** The arrangement is the
constraint here. D2's claim is that one site states the reservation and nothing
else does, and a claim about where a value is declared cannot be made without
naming the place. The record states the constraint the arrangement serves in the
same decision.

**Objection 5: the hash claim is an assertion the record cannot support.**
**Failed.** It was checked against the hash function directly. The function
writes lengths, counts, columns, generations and the free queue, and it writes
no capacity. The claim is exact.

**Objection 6: the record is too long for a subsystem record.** **Failed.** It
is 1453 words, against reference medians near 1300 and against six sibling
drafts that run from 1777 to 4570. It is the shortest of them.

**Objection 7: the record claims a refusal leaves nothing behind.** **Held.**
Section 3.1.

**Objection 8: the record claims the code states no population.** **Held.**
Section 3.2.

## 5. What only a run can settle

The reviewer compiled nothing. The claims below rest on the source and on the
commit body of the merged change, and none is counted toward the verdict.

- **The reservation test passes.** The test file reads the address of the first
  entry of nine unit columns and asserts that it does not move, which is the
  assertion that a growing column would break. Whether it passes was not
  observed.
- **A world that reserves the target population builds at all.** The reservation
  is one million entries in ten columns and a queue. Whether that allocation
  succeeds on a development machine was not observed, and no measurement exists
  on the target platform in any case.[^3]
- **The two determinism tests still pass.** A reservation moves no hash by the
  argument in section 1, and that argument was read rather than run.
- **The founding refusal is reachable in a run.** The test builds a world whose
  reservation is half its group, which makes it reachable by construction. The
  run was not observed.

## 6. What the review found beyond the record

**The phrase that the finding corrected is still in the tree.** The doc comment
above the function that undoes a founding says that a refused founding changes
nothing that a caller can observe. A caller can observe the state hash, and the
test beside it says so in its own words. This is the same overreach as section
3.1, in the code rather than in the record. It is recorded as a
finding.[^4]

**The reservation sweep missed one call site and a later commit repaired it.**
The settings field reached 82 struct literals, and one test file was left
without it. The tree did not compile until the commit that heads `main` now.
This is the sweep shape the defect rule names, and the finding that priced the
field already holds the count.[^5] No new finding was opened, because the repair
is merged.

## 7. Checks run

Six document checks were run. All six pass. They compile nothing.

| Check | Result |
|---|---|
| `scripts/check_adrs.py` | 0 failures. One note, about an uncited record that is not ADR-0084 |
| `scripts/check_citations.py` | 0 failures |
| `scripts/check_registers.py` | 0 failures |
| `scripts/check_priority.py` | 0 failures |
| `scripts/check_footnotes.py` | 0 failures |
| `scripts/check_conflict_markers.py` | 0 failures |

The whole gate was not run, because three other workers held the machine.

## References

[^1]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^2]: Findings register, FND-144. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-168. `docs/FINDINGS.md`
[^5]: Findings register, FND-145. `docs/FINDINGS.md`
