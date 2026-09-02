# Review 0143: The housing, growth, founding and recovery records

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md` | Status `Draft` at review, `Accepted` after it |
| `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md` | Status `Draft` at review, `Accepted` after it |
| `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md` | Status `Draft` at review |
| `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md` | Status `Draft` at review |
| Commit | `910dec0` |
| Code read | the founding module, the resource module, the cohort module, the settlement module, the soldier module, the step of the world, and the tests of each |

The reviewer wrote none of these four records. The reviewer wrote no code in
this repository.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0076 | Accept, with three footnote labels reordered |
| ADR-0080 | Accept, with two sentences amended |
| ADR-0081 | Reject. The engine already holds the count that D3 asks the project to store |
| ADR-0082 | Reject, because it rests on ADR-0081 |

Two of the four hold against the code. One states a premise that the code
falsifies. The fourth depends on that one.

**The rejections are not a judgement of the subject.** Housing and growth are
needed, and most of what these two records argue survives. Section 3 says which
decisions stand and what the author must change.

## 1. ADR-0081: the count already exists

### The finding

The record opens with this claim: "nothing states how the engine answers how
many it holds now."

The engine answers it today. The cohort table holds one row for each faction at
each site, and each row holds a headcount.[^1] The residents of one site are the
sum of its rows. The table is rebuilt from the home column of the soldier arena,
which the record itself names as the residence.[^2]

Every unit that can hold a residence is a soldier today. The character arena
holds no home column. The cohort headcount is therefore the resident count of a
site, and not a part of it.

### What follows for D3

D3 states a prohibition: "**No pass over the units recomputes the count during a
running frame.** That is the prohibition, and it is what makes the count worth
storing."

The consumption pass rebuilds the cohort table twice inside one frame. It
rebuilds it once before the pooled draw, and once after the scan that ends a
starved unit.[^3] Each rebuild walks the whole home column and recomputes a
per-site headcount.

The record and the code therefore disagree, and one of two things is true.

**Either the prohibition reaches the cohort rebuild**, and the engine violates
ADR-0081 D3 on the day the record is accepted. A record the code contradicts is
worse than no record, because it lies.[^4]

**Or the prohibition reaches only the new count**, and D3 buys nothing. The
project would then pay the population for a per-site headcount inside the frame,
and also maintain a second per-site count by the change, and also write a check
that compares the second count against the column. The record prices this as one
extra declaration site. It is two.[^5]

Neither reading admits acceptance.

### What follows for the rest of the record

D3 also justifies the stored count by its callers: "A caller asks for it, and
asks often. A watcher reads how full a place is, and the admission of a birth
reads the same number."

The cohort table already serves both. It is settled at the end of the
consumption pass, and the growth pass of ADR-0082 D4 runs after that pass. The
number a birth admission needs is therefore current where the admission wants
it, without a new store. `World::cohorts` and `CohortTable::headcount` are
public, so a watcher reaches the number through the public interface today. What
is missing is a reader that sums the rows of one site, because the table splits
the count by faction.

**The check that D3 asks for also exists.** D3 says the price of the second
declaration site is a check that fails when the two disagree.
`World::cohorts_describe_the_units` derives the table again from the home column
and compares. Its own documentation gives the reason D3 gives.

The record's comparison against the tile case is sound and is worth keeping. A
residence has a lifetime and a tile occupancy does not. That reasoning survives
the finding. What does not survive is the conclusion that a new stored count is
the way to hold it.

### The implementation status is stated wrongly

The record says: "**No code implements this record.** The engine holds the
residence column. It holds neither a capacity nor an occupancy count."

The first sentence is false. The third is half false: no site holds a housing
capacity, and the engine does hold a per-site count of residents.

D2 is implemented. The soldier arena holds the home column, the founding writes
it, and the destroy path clears it.[^2] D4 is honoured, because the destroy path
reads every unit rather than a reverse index. D1 is not implemented.

The registry requires the acceptance of an unimplemented record to say plainly
that nothing implements it.[^6] A record that says so wrongly is worse than a
record that says nothing, because a reviewer reads it and stops looking.

### What must change

1. Correct the context. State that the cohort table answers the resident count
   today, and state what it does not answer.
2. Decide D3 again against that fact. Either derive the count from the cohort
   table, or state why a second maintained count beats it. The open row holds
   the choice.[^7]
3. Correct the implementation status, decision by decision.
4. Repair the footnotes. Section 4 holds the detail.

## 2. ADR-0082: it rests on ADR-0081

ADR-0082 D2 admits a birth against "the free places of a site". A free place is
the housing capacity of ADR-0081 D1 less the occupancy of ADR-0081 D3. Neither
exists in an accepted record and neither exists in the code.

A draft binds nothing.[^6] ADR-0082 therefore rests on nothing, and it cannot be
accepted while ADR-0081 is unsettled.

**This is not a verdict on its decisions.** D2, D3 and D4 are the strongest
prose in the four records, and the reviewer tried and failed to break them.
Section 5 records the attempts. D2's argument that a bound and a factor compose
differently is correct, and its argument that the admission key needs an ordinal
within the site is correct. D4's third key test, for two proposals of one site in
one frame, is the test that the project has already lost once elsewhere.[^8]

The record has one defect of its own, beyond its dependency. Footnote 5 and
footnote 20 name one source: ADR-0063 decision D4. The documentation rule
forbids a repeated footnote and asks the author to reuse the marker.[^9] Footnote
19 and footnote 20 also occur in the body before footnotes 12 to 18, so the
labels do not follow the order of occurrence.

**Do not renumber the whole file to repair this.** Wait for the ADR-0081 outcome
and repair both in one change, because D2 and D4 both change if the resident
count changes.

## 3. ADR-0076: the record against the code

The code exists. The reviewer read each decision against it.

**D1, the minimum distance.** `founding::survey_addresses` marks a candidate
`separated` when its distance from every taken place is at or above
`MINIMUM_FOUNDING_DISTANCE`, and a candidate is eligible only when it is
separated. A founding that finds no eligible candidate returns
`FoundingError::NoPlaceFound`, and it neither draws again nor widens the sample.
The record states no value for the distance, which is what the scope rule
asks.[^10]

The floor is checked rather than commented. A compile-time assertion fails when
the minimum distance is not greater than twice the survey radius. The record
claims exactly this, and the assertion is what makes the claim true.

**D2, the order and the outcomes.** `World::found_run_for_every_faction` loops
over the faction indices in ascending order. It pushes the place of each
successful founding into the taken list before the next founding runs. It
returns one `FoundingOutcome` for each faction, and each outcome carries the
faction and either the founding or the refusal. A test asserts the ascending
order. A second test asserts that a world too small for its factions seats some
and refuses the rest, and that the foundings before a refusal stand.

**D3, the faction in the frame slot.** `founding::founding_frame` returns the
faction as the frame slot of the draw key, and both candidate draws use it. A
test draws a sample for each faction and asserts that no faction draws the
sample of faction zero. That test is the only guard on D3, and the record says
so.

**The record holds no volatile material.** It states no count, no file table, no
figure and no version. It names no module.

**The title states a claim.** A violation is findable: two foundings closer than
the distance, or a run in another order, or a key without the faction.

### The one change

Footnote 8 first occurs in the consequences. Footnote 9 and footnote 10 first
occur in D3, which is earlier. The documentation rule asks the author to number
the footnotes in the order that they occur.[^9] Three labels therefore move, and
the definition list moves with them. No claim changes and no source changes.

**The change is applied.** A draft exists to be edited, and the edit is
mechanical.[^6]

## 4. ADR-0080: the record against the code

The reviewer read the cost claim first, as the priority index asks.

**D2, the cost of recovery.** The claim is that the pass reads the depleted set
and takes no tile count. `Depletion::recover` takes a tick and nothing else. It
walks its own entries. It reads no grid, no extent and no tile count, so the
forbidden pass is not expressible through that signature. The record states the
guarantee as a property of the signature rather than of the body, and that is
the stronger form.

The pass records how many entries it read, and two tests assert that count. The
cost claim is therefore checked without a timing assertion.[^11] **No figure was
measured, and the record claims none.** Every cost figure in this project is
derived.[^12]

**D1, ageing never grows an amount.** `aged` reduces the stored take by the
whole periods that were spent, and it takes the minimum of that number and the
take. The take cannot go below zero, so the stock cannot rise above what the
generator gave. The bound comes from the arithmetic, as the record claims.

**D3, exact whole numbers in key order.** The division is integer division. The
anchor advances by the whole periods spent, and not to the tick. An entry that
owes nothing restarts its clock at the tick. The pass walks the entries in the
order the ledger holds, and the ledger holds them sorted by a packed key. Each
of these matches the record sentence by sentence.

**D4, the order in the step.** `World::step` calls `self.depletion.recover(tick)`
immediately before `self.gather(threads)`. A comment at the call site cites this
record.

**D5, one declaration of the period.** `RecoveryRules` holds one period for each
kind, and a caller replaces the whole rule set. `RecoveryRules::from_ticks`
refuses a period of zero, because a period of zero and an absent period would be
two ways to say one thing. The periods are stated in simulated days and
converted in one place.

### The two amendments

**The first is a misattribution.** The alternatives section says: "the product
record refuses it by name: a world that steps every deposit pays the world for
the deposits that nothing is touching." The footnote on that sentence names
ADR-0072 decision D4. ADR-0072 D4 does not hold that sentence. PRD-0018 holds
it.[^13]

The sentence is therefore wrong whichever way it is read. It names a source that
its footnote does not hold, which the documentation rule forbids.[^9] If the
footnote were repaired to name PRD-0018, the record would cite a product record,
which the product rule forbids.[^14]

The repair states the reason directly and drops the attribution:

> **A pass that steps every deposit on every tick.** This is the obvious shape,
> and it pays the world for every deposit that nothing is touching. The engine
> already stores nothing for a tile that nobody gathered from, so such a pass
> would read millions of tiles to change none of them.[^2] D2 forbids it.

The existing footnote 2, which names ADR-0072 D4, supports that sentence
correctly.

**The second is an inventory of an uncalled function.** D4 says: "A pure
function of the stored take, the tick and the period would give the same answer
without depending on the order of the step, and the module holds one. Nothing
calls it."

The function is `Depletion::taken_at`. A whole-tree search finds no caller. The
sentence is therefore true today. It stops being true on the day an open backlog
item drives the reader or retires it, and nothing fails when it does.[^15] A
record must not hold a survivor list, and it must not record a capability that
nothing invokes.[^10]

The repair keeps the constraint and drops the inventory:

> This decision claims no more than that. The property rests on the order of the
> step, and a change to that order breaks it. A reader that took the stored take,
> the tick and the period as its arguments would not depend on the order, and no
> caller reads a stock that way.

**Both amendments are applied.** A draft exists to be edited, and neither
amendment changes a decision.[^6]

## 5. Objections attempted

A review that lists no attempted objection did not happen.[^6] These are the
objections the reviewer raised against the decisions, and what happened to each.

The reviewer attempted twenty-four objections across the four records, and nine
held. Sixteen are below. The other eight are in sections 1 to 4, because each
one produced a rejection or an amendment: three against ADR-0081, two against
ADR-0082, two against ADR-0080, and one against ADR-0076.

### Against ADR-0076

**"D2 states a loop, not a constraint."** It fails. The order of the foundings
fixes which faction gets the better place, so an unstated order is a determinism
defect. The scope rule names determinism in its counter-test and asks for a
record even when the answer looks obvious.[^10]

**"D3 repeats the key decision of ADR-0075."** It fails. ADR-0075 puts the
candidate ordinal in the entity slot and the axis in the draw slot. ADR-0076 adds
the faction to the frame slot, which ADR-0075 leaves as a constant. Each has a
violation the other permits.

**"D1's floor is a second declaration of the survey radius."** It fails. The
floor is a compile-time assertion, so the two copies cannot disagree without
breaking the build. That is what the defect rule asks for.[^5]

**"The consequences hold a survivor list: 'including the demonstration binary
and every test'."** It fails. The sentence names a class of caller, not a file
table. No count and no path appears.

**"The faction count is declared in six places."** This one holds, and it holds
against the code rather than against the record. `world.rs` writes
`faction_count.max(1)` at six sites. Each site states that a faction count of
zero behaves as one, and nothing fails when the copies disagree. The record is
right to state the rule once. A backlog item now carries the repair.[^16]

### Against ADR-0080

**"The consequence about the golden files is a statement about one moment."** It
fails, narrowly. The sentence says that the state hash now covers the anchor of
each entry. That is a permanent consequence of D1, and the golden files move
because of it.

**"D2 states a cost, and a cost belongs in a register."** It fails. D2 states a
shape and no number. It says what the pass may reach, which is a constraint a
reviewer can find a violation of, in a signature.

**"D5 names a module arrangement, because it says the period lives in one
place."** It fails. The constraint is that one declaration site exists, not where
it sits. The scope rule refuses a module arrangement and admits a constraint that
a single site enforces.[^10]

**"Recovery could run after the gather resolve and give the same answer."** It
fails. A gather that ran first would take against a stale amount, and the record
says so. The code orders the two calls in the way the record states.

### Against ADR-0081

**"D3 contradicts ADR-0074 D3, which rejects a maintained occupancy count."** It
fails. ADR-0074 D3 rejects a dense array over every tile. ADR-0081 D3 proposes a
sparse count over the sites. The two subjects differ, the lifetimes differ, and
the record argues the difference well. **The two records can stand together, and
neither must supersede the other.** This was the objection the task named, and it
does not hold.

**"The engine already answers the question."** It holds. Section 1 holds the
evidence, and this is the reason for the rejection.

**"D2 is not a decision, because the column already exists."** It fails. D2
forbids a second column, and a contributor could reasonably add one. The findings
register records that this item planned exactly that.[^2]

**"D4 records an absence, and an absence needs no record."** It fails. A reverse
index is the obvious structure, and D4 states why the project declines it and
what the decline costs. That is a constraint.

### Against ADR-0082

**"The two limits should multiply, so that housing slows growth."** It fails. The
record answers it directly. A product of two limits gives one answer in the easy
case and another at the boundary, and the easy case is where a test looks.

**"D3's ordering of the applied births is over-specified."** It fails, and this
was the strongest attempt. The slot a newborn takes comes from a free list, and
the slot is part of an identity. An order left to the collection would be stable
and would pass both determinism tests, and it would still be unrecorded. The
record states this and it is right.

**"D1 states no rate, so it decides nothing."** It fails. D1 decides that the
store sets the rate, and that a site with no surplus proposes no birth. The value
is content and an open row holds it.

## 6. Two rules that nothing checks

Both came out of reading these four records. Both are recorded as findings.

**A decision record must cite no product requirement record.**[^14] Four accepted
records cite one, and three of these four drafts cite one. No script checks it.
The reviewer did not reject any record on this ground alone, because the accepted
precedent runs the other way and a reviewer must not apply a rule to a draft that
the project does not apply to its accepted work. An open row now holds the
choice: enforce the rule and repair the accepted records, or drop the rule.[^17] [^18]

**The documentation rule states two things about footnotes that nothing
checks.** Number them in the order they occur, and do not repeat one.[^9] Three
of these four drafts break one or both.[^19] A backlog item now adds the check.[^20]

## 7. What the review did not check

**Nothing was measured.** No record here states a measured figure, and the
reviewer took no measurement. The blocker governs every cost figure in this
project.[^12]

**Nothing under `crates/` was read for anything but a citation.** The reviewer
edited eight source files, and every edit replaced the draft path of a record
with its accepted path. No code, no comment prose and no test changed. The
citation check derives the truth from the tree and is what proves the sweep.

**Nothing was checked against a second reader.** The delegation stands, and a
review by a second person supersedes this one.[^6]

**Four stale citations were repaired outside the reviewer's own work.** A
dispatcher commit moved items 0059 and 0060 out of the refined directory and
left four citations of the old paths behind, in the decisions register, in item
0136 and in item 0155. The same commit cited a finding that only this branch
holds. The citation check was red on the trunk for both reasons. The reviewer
repaired the four paths rather than hand over a red gate, and recorded the
shape.[^21]

**Backlog items 0059 and 0060 were not repaired.** Both were refined against
the decisions of ADR-0081, and item 0059 names all four of them. The rejection
invalidates that impact review, and the dispatcher moved both items back to
`proposed/`.

**Item 0059 was read, and the engine reaches further into it than the
rejection alone implies.** Two of its four numbered work steps are built and a
third is half built. The occupancy count exists, split by faction, and a caller
reads it through the public interface. The invariant check exists, and the test
that proves the check can fail exists. Only the housing capacity is wholly new.
A separate finding holds the evidence, because a re-refinement that starts from
a new record rather than from the code repeats the defect.[^22]

## 8. For the registers

- ADR-0076 moves from `Draft` to `Accepted`. The footnote labels moved first,
  and every citation of the draft path moved with the file.
- ADR-0080 moves from `Draft` to `Accepted`. The two amendments went in first,
  and its citations moved the same way.
- ADR-0081 stays a `Draft`. Section 1 states what must change.
- ADR-0082 stays a `Draft`. It moves when ADR-0081 moves.
- Five findings are recorded.[^1] [^17] [^19] [^22] [^21]
- Two open choices are recorded: whether a decision record may cite a product
  record, and how a site answers its resident count.[^7] [^18]
- **No blocker opens, and the number reserved for one is unused.** The housing
  items are stopped by a choice, not by missing information, and the blockers
  register says that a choice belongs in the decisions register.[^23] The open
  row holds it.[^7]
- No blocker closes.

## References

[^1]: Findings register, FND-128. `docs/FINDINGS.md`
[^2]: Findings register, FND-116. `docs/FINDINGS.md`
[^3]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^4]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^5]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^6]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^7]: Decisions register, DEC-057. `docs/DECISIONS.md`
[^8]: Findings register, FND-122. `docs/FINDINGS.md`
[^9]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^10]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^11]: Testing Rules, section 3. `.claude/rules/testing.md`
[^12]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^13]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^14]: Product requirement records, what does not belong here. `docs/product/README.md`
[^15]: Backlog item 0135. `docs/backlog/proposed/0135-drive-or-retire-the-deposit-amount-reader.md`
[^16]: Backlog item 0145. `docs/backlog/proposed/0145-give-the-faction-count-one-rule-for-zero.md`
[^17]: Findings register, FND-129. `docs/FINDINGS.md`
[^18]: Decisions register, DEC-056. `docs/DECISIONS.md`
[^19]: Findings register, FND-130. `docs/FINDINGS.md`
[^20]: Backlog item 0144. `docs/backlog/refined/0144-check-the-footnotes-of-a-record.md`
[^21]: Findings register, FND-132. `docs/FINDINGS.md`
[^22]: Findings register, FND-131. `docs/FINDINGS.md`
[^23]: Blockers register, what a blocker is. `docs/BLOCKERS.md`
