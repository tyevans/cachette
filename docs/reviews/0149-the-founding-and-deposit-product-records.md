# Review 0149 — The founding and deposit product records

## What was reviewed

Three product requirement records, at commit `910dec0`.

| Record | Path | Question asked |
|---|---|---|
| PRD-0018 | `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md` | Does it pass the six gate questions? |
| PRD-0012 | `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md` | Does a run meet each checkable statement? |
| PRD-0006 | `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md` | Is one statement the only one that fails? |

The reviewer wrote none of the three records.

## Verdicts

| Record | Verdict |
|---|---|
| PRD-0018 | REJECT. It stays at `Shaped`. Section 3 below gives the exact text to change. |
| PRD-0012 | Eight of nine statements are met. One is not. One cost statement is false of the engine. It stays at `Accepted` and does not move to `shipped/`. |
| PRD-0006 | One statement fails, and it is the one the priority index names. |

---

## 1. PRD-0018 against the six gate questions

| Gate question | Result |
|---|---|
| Who this is for | Passes. It names a developer, and a modeller second. |
| What the person cannot do today | Passes. It states three behaviours the developer cannot produce. |
| What good looks like | Passes. Eleven statements, and a reader can check each one. |
| What this does not do | Passes. Eight bounds, and one of them explains itself. |
| What it costs at the target scale | **Fails.** It states a mechanism in place of a cost. |
| Which blockers govern it | Passes. It cites the measurement blocker and states no figure. |

## 2. The objections, and what happened to each

### O1. The cost section states the mechanism of a decision record. HELD.

This is the blocking objection.

A product record states a need. It never states a structure. A record that
names a data structure holds an architectural decision.[^1]

The cost section states the storage the engine uses and the algorithm that
recovers a deposit. Three passages do this.

- It states that the world stores no amounts, that it stores only what units
  took, and that it stores that for a tile only when somebody took from it.
- It states that recovery is the removal of a fact, and that the world makes a
  stored record smaller until the record goes.
- It states that the amount follows from what was taken, when it was taken and
  the parameters of the kind, and that the world can therefore answer the
  question when it is asked.

Two decision records already hold all three claims. One says that a tile stock
is generated and that only what was taken is stored, and the project accepted
it.[^15] The other ages the stored take, makes its cost follow the depleted
set, and the project accepted it too.[^2] The product record therefore holds a second
copy of two decisions. One fact in two places, with nothing that fails when
the copies disagree, is the defect shape this project keeps meeting.[^3]

The mechanical check does not see this, because the record names no decision
record by number. The check looks for that name only.[^4] A separate finding
records the gap.[^5]

### O2. Two cost bullets name the store. HELD.

Two of the five bullets in the cost section state storage rather than cost.

- The bullet that says the depleted set shrinks, because a recovered deposit
  stops being a fact the world holds.
- The bullet that says recovery adds no storage for a tile that nobody
  gathered from.

Both state a property of a store that a decision record chose. The need behind
them is a cost claim, and the record can state that claim without the store.

### O3. The crop paragraph states what the engine carries today. HELD, minor.

The paragraph that separates a crop from recovery says that the engine already
builds an improvement over ticks and that a site already produces at a rate.
Both sentences describe the tree as it stands. A product record must not carry
material that the next change makes false.[^6] The argument does not need
them. A crop is an act on a chosen site and recovery is not, and that sentence
alone carries the distinction.

### O4. The record presents a figure as measured. FAILED.

The record states no cost figure. It says that every figure in the project is
derived, and it cites the measurement blocker.[^7] The objection does not hold.

### O5. The record names two audiences. FAILED.

The gate asks for one audience, and the record names a developer first and a
modeller second. The accepted record for a founding does the same, so this is
the settled house style and not a defect of this record.[^8]

### O6. The record reserves a number for the crop record. FAILED.

The record states that no number is reserved, and that reserving one would
state an intent as a fact. This is correct, and it is the opposite of the
defect the objection looked for.

## 3. The exact change PRD-0018 needs

Replace the cost section between the heading and the crop paragraph with a
statement of the cost, and delete every sentence that names a store.

The section must keep these claims, because each is a cost at the target
scale and none of them names a mechanism.

- The cost of recovery grows with the number of deposits that units have
  depleted. It does not grow with the number of tiles, and it does not grow
  with the extent of the world.
- The cost of recovery does not grow without bound over a long run. A deposit
  that has returned to its starting amount costs nothing again.
- A world where nobody gathered pays nothing for recovery.
- Every amount is an exact whole number, so a total over tiles combines the
  same in any order and at any thread count.

The section must lose these sentences, because a decision record holds each
one.

- Every sentence that says what the world stores, or does not store.
- Every sentence that says recovery removes or shrinks a record.
- The sentence that says the world answers the amount when it is asked.
- The two sentences of the crop paragraph that describe what the engine
  carries today.

The record is otherwise ready. A reviewer may accept it once the cost section
states cost alone.

Backlog item 0149 holds this work.[^9]

---

## 4. PRD-0012, statement by statement

The evidence comes from two runs.

The first is the demonstration binary, `cargo run --release --package
cachette-view`, closed after 25 seconds. It founds four groups of thirty in a
640 by 440 world at seed `0x0cac4e77e5eed001`, and it prints each founding
before it opens the window.

The second is a driver outside this repository. It links the core crate and
calls the same public interface that the demonstration binary calls:
`found_run_for_every_faction`, `found_group_at`, `survey_founding` and `step`.
It touched no file in this tree.

Both ran on a development machine, not on the target platform. No figure below
is a cost.

| No. | Statement | Result |
|---|---|---|
| S1 | A run begins with a small group, whose size is an input | Met |
| S2 | The engine chooses the place by reading the world, and a watcher can ask why | Met |
| S3 | A different seed gives a different place that answers the same test | Met |
| S4 | A watcher can compare the chosen place against the places not chosen | Met |
| S5 | The group changes something a watcher can see inside a hundred ticks | Met |
| S6 | A poor founding does worse than a good one, and a watcher sees it | Met |
| S7 | The end population follows from the run, not from the starting number | **Not met** |
| S8 | The same seed gives the same founding at every thread count | Met |
| S9 | A small group costs the small group, not the target population | **Not checkable** |

### The evidence

**S1. Met.** The demonstration founded four groups of thirty people each, at
(162, 396), (501, 379), (619, 56) and (285, 270). The group size is an
argument of the founding call, and the world was sized 640 by 440 rather than
for the group.

**S2. Met.** The demonstration printed the quantities the survey read at each
chosen place. The first founding reaches 44 food, 240 wood and 0 stone, over
37 open tiles, with no open water beside it. The head-up display carries the
same quantities in the window.

**S3. Met.** Three seeds gave three places: (162, 396), (489, 232) and
(325, 159). Each chosen place reported itself eligible and separated, so the
new place answers the same test as the old one.

**S4. Met.** The survey holds 63 places it did not choose, with the quantities
it read at each. The head-up display shows one of them beside the chosen
place, so a watcher compares without leaving the window.

**S5. Met.** Over the first hundred ticks, 118 of the 120 founded people
changed tile, 19331 tiles came under a holder, and both the state hash and the
whole-world tile total changed. The window draws the units and the holder
layer, so a watcher sees all three.

**S6. Met.** The driver founded the same group twice in one world, once at the
place the survey chose and once at the worst place the survey still admitted.
The good place reaches 44 food. The poor place reaches 8. The good founding
held thirty people alive at tick 1000, with no rationing. The poor founding
rationed its site 13 times and lost all thirty people by tick 130. The
head-up display reports the units that are short and the units that ended, so
a watcher sees the difference.

**S7. Not met.** Nothing is born. The good founding held exactly thirty people
at tick 1000, which is the number the run started with. The population can
fall from the run, because the poor founding reached zero, but it cannot rise.
The statement asks for both. Item 0060 grows the population from the store and
the housing, and this review adds PRD-0012 to what that item serves.[^10]

**S9. Not checkable.** No measurement exists on the target platform, so no run
on this machine answers a cost statement.[^7] One thing a run does show is not
a cost, and it contradicts the record. Section 5 states it.

**PRD-0012 does not move to `shipped/`.** One statement is not met and one
cost statement is false.

## 5. A cost statement of PRD-0012 that the engine contradicts

The cost section of PRD-0012 says that the storage the world reserves is sized
for the target population and does not change during a run, and that a run does
not stop to grow.

The engine does the opposite. The unit arena opens as many slots as the slot
index holds, reserves no memory for them, and appends one entry to each of its
ten columns at each spawn. After the demonstration founded 120 people, the
arena reported a capacity of 4294967295, which is the range of the index and
not a population.

The record and the code disagree, so one of them changes.[^11] The choice is
architectural, and this review holds no rights over a decision record. It opens
a row in the decisions register instead, and a backlog item to answer it.[^12]
[^13] A finding records what the project believed.[^14]

---

## 6. PRD-0006

The priority index says that one statement still fails, because two values name
the owner of a tile. The review confirms that, and confirms nothing else fails.

The driver founded four groups and ran 200 ticks at 4 threads.

| Statement | Result | Evidence |
|---|---|---|
| A tile is held by a faction, or by nobody | Met | 44415 tiles held, 237185 held by nobody, of 281600 |
| A watcher sees who holds a tile, and where holdings meet | Met | The holder layer draws the ground, and a tile whose neighbour holds otherwise takes a border |
| A holding changes by a rule during a run | Met | Nothing is held at the founding, 19331 tiles at tick 100, 44415 at tick 200 |
| A faction's holding is reportable without a walk | Met | The census gave 4682, 22056, 11241 and 6436, and the four sum to the held count |
| Holding is exclusive, and the invariant re-derives it | Met | The whole-world invariant check passed after 200 ticks |
| Terrain influences holding | Met | No tile of the first ground kind is ever held, out of 109884; the fifth kind holds 180 of 29799 |
| The same seed gives the same holdings at every thread count | Met | The holder column is identical at 1, 2 and 12 threads after 50 ticks |
| Exactly one value names the owner of a tile | **Not met** | Item 0084 removes the second value |

**Item 0084 ships PRD-0006.** It is the only outstanding statement. This
review changed nothing in that area.

## References

[^1]: Product requirement records, what a record must not state. `docs/product/README.md`
[^2]: ADR-0080, a depleted deposit recovers by ageing the stored take, decisions D1 and D2. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^3]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^4]: The product record check. `scripts/check_prds.py`
[^5]: Findings register, FND-134. `docs/FINDINGS.md`
[^6]: Decision Record Scope, section 4.3. `.claude/rules/adr-scope.md`
[^7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^8]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^9]: Backlog item 0149. `docs/backlog/proposed/0149-state-the-cost-of-recovery-without-the-mechanism.md`
[^10]: Backlog item 0060. `docs/backlog/proposed/0060-grow-the-population-from-the-store-and-the-housing.md`
[^11]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^12]: Decisions register, DEC-059. `docs/DECISIONS.md`
[^13]: Backlog item 0150. `docs/backlog/proposed/0150-decide-how-the-world-reserves-unit-storage.md`
[^14]: Findings register, FND-135. `docs/FINDINGS.md`
[^15]: ADR-0072, a tile stock is generated, and only what was taken is stored. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
