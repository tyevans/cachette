# Review 0021: The movement records


> **`ADR-0057` was retired while this audit ran.** The audit recommended
> leaving it a draft with its number retained. The project owner decided to
> retire it instead, on the ground that it did not serve the project, and the
> registry holds the retired row and the reason.
>
> The audit's reading of it survives the decision and is why this file keeps
> the section. It found that two of the five decisions state real
> constraints, and that the other three describe a portal graph, a flow tile
> and a cache that nothing had built. A future record that needs a long path
> should start from those two constraints and from the research report, not
> from the retired file.
>
> Every mention of the retired number below is a code span rather than a
> citation. A retired number names nothing to follow.

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md` | Status `Draft` |
| Retired. `docs/adrs/REGISTRY.md` | Status `Draft` |
| Branch | `feat/sprint-5-admission` |
| Commit | `7411bd4` |

The reviewer wrote neither record and wrote none of the code that the review
reads. The reviewer read the records, the rule, the accepted records that the
two drafts cite, and the foundation crate.[^1] [^2]

## Verdicts

| Record | Verdict |
|---|---|
| ADR-0056 | ACCEPT WITH AMENDMENT. Amendments 56-1, 56-2, 56-3 and 56-4 are mandatory. 56-5 and 56-6 are recommended. |
| `ADR-0057` | LEAVE AS DRAFT. The number stays allocated. Section 6 states what must be true first. |

The strongest finding is amendment 56-2. ADR-0056 D3 sub-step 3 does not hold
the capacity invariant that the whole record exists to hold. Item 0023
implements D3, so this defect becomes a defect in the engine.[^3]

---

## 1. ADR-0056, the three-condition test

The rule admits a record when three statements are all true.[^4]

**1. Could a contributor reasonably choose otherwise? Yes.** A continuous
position with a velocity is the ordinary choice for this problem, and the
research report costs the negotiation that it needs.[^5] A contributor who
has built a crowd system before will reach for it first.

**2. Does choosing otherwise cost more than changing it later? Yes.** The
position representation reaches every system that reads a position, the event
layout, the state hash and the Python boundary. A later reversal is not a
refactor.

**3. Is the reasoning invisible in the artefact? Yes.** A reader of the
movement kernel sees a tile index. Nothing in the code says that a sub-tile
coordinate was considered and rejected on two separate grounds.

The counter-test also applies and is decisive on its own. D3 governs
determinism, and a contributor who optimises admission with a first-come rule
writes code that passes a single-threaded test.[^6] The record is the only
thing that rejection can cite.

**The record is warranted.** No decision in it should be dropped, and the
number stays.

## 2. ADR-0056 against its dependencies, decision by decision

The bridge changed under this record. ADR-0018 is now accepted, it states
that the bridge is derived and rebuilt at the barrier, and it states that the
arena is not sorted.[^7] ADR-0014 fixes the slot index as half of the
identity, so nothing may reorder the arena.[^8]

### D1. A unit occupies exactly one tile

**Holds.** No dependency contradicts it. A tile index is a raw axial address
on a rhombus world, and the soldier column set already holds one.[^9] [^10]

Two observations, neither of them a contradiction.

The progress accumulator that D1 decides is a column, and no column set holds
it today. Entity storage holds four fixed shapes, and a new column on the
soldier shape is a change that record governs.[^11] D1 does not cite it. This
is amendment 56-5, and it is recommended rather than mandatory, because the
storage record does not forbid the column. It only owns it.

The clamp claim is correct and its evidence is correct. The findings register
holds the overflow.[^12] D1 states no width and no value, so it complies with
section 4.1 of the rule.

### D2. A move is an intent, and a separate admission step grants it

**Holds as a constraint. Over-specifies as a description.** See section 4,
where the code is read against it.

Nothing in ADR-0018 or ADR-0014 contradicts the two-phase shape. ADR-0018
strengthens it: the bridge is the occupancy as it stood at the barrier, so a
move applied during the frame does not change what a later system reads.[^7]
D2's claim that no unit sees a half-applied world is therefore supported by an
accepted record, and D2 does not cite it. That is worth adding.

### D3. Admission sorts by a stable key, then admits in that order

**Does not hold as written.** Two defects, one of them fatal to the invariant.
Section 3 gives them in full.

Against the dependencies, the parts that do hold are these. The sort is on a
key vector whose last field is a stable identifier, which is what ADR-0007
requires.[^13] Sorting the intents, rather than the units, is the correct
reading of ADR-0014 and ADR-0018: the intent set is a scratch array that
admission owns, so sorting it reorders nothing that the arena owns.[^7] [^8]
The record's context paragraph already says this.

**D3 never says where the occupancy of a target tile comes from.** This is the
gap that a previous review left open and named for this one to close.[^14]
ADR-0018 now states, as an accepted consequence, that admission resolves
contention for a target tile from the bridge. D3 reads a count array instead,
which only D4 mentions. An accepted record and a draft describe the same step
two ways. Amendment 56-3 settles it.

### D4. Capacity is a data-driven property of the terrain

**Holds as a constraint. One sentence must go.** Section 5 gives the reasoning.

Blocker hygiene is now clean here. Footnote 9 names BLK-001 and BLK-009 as
resolved, and both are resolved. The values live in the scale constants
table.[^15] [^16] This is the correction that a previous audit already applied,
and it survived.

### D5. A rejected unit is not stuck

**Holds.** The keyed draw is exactly what ADR-0003 requires, and the movement
system in the code already draws that way.[^17]

Two gaps, both recommended rather than mandatory.

The rejection count is a per-unit value, so it is a second new column, and the
same storage record governs it.[^11]

The threshold has no home. D5 correctly states no value, but it does not say
where the value lives. The rule requires a volatile value to sit in a
reference table with a citation from the record.[^4] Without that sentence a
contributor writes the literal in the movement kernel, which is the failure
that D4's last consequence forbids for capacity and does not forbid here.

## 3. ADR-0056 D3, is it implementable as written?

The primitives that exist are the key vector sort, the slot reduction, the
soldier arena and the unit-to-tile bridge.[^1]

### Sub-step 1, reduce intents by source tile

**Buildable.** A count for each source tile is a sum, and integer addition
combines in any order, so the reduction is order-free and needs no slot
array.[^18] It needs an array indexed by tile to accumulate into. See section 5
for the fact that no record decides that array.

### Sub-step 2, sort by target tile then by stable key

**Buildable.** The sort takes a key vector of two unsigned fields, the target
tile index and the identity as one integer. The identity is unique across live
entities, so the sort's uniqueness check passes.[^8] [^13] The result is one
exact permutation at any thread count, and the module states why the
perturbation probe cannot make it fail.[^1]

### Sub-step 3, admit until the target tile reaches its capacity

**Not buildable as written, and not correct as written.** Two separate
problems.

**Problem A: the departure credit is unsound.** Sub-step 1 counts intents, not
admitted departures. An intent is not a departure. A unit that intends to
leave a tile and is then rejected at its own target does not leave, so the
room it released was never released.

Take three tiles in a line, A, B and C. Tile B is at capacity, and tile C is
at capacity with no departure. One unit in B intends to move to C, and it is
rejected. One unit in A intends to move to B, and sub-step 3 admits it,
because B's departure count is one. Tile B now holds one unit above its
capacity.

The failure is not a determinism failure. The result is still a function of
the intent set, so the thread-count test and the golden state test both pass.
It is a correctness failure that those two tests cannot see, and the capacity
invariant is the only thing D3 exists to enforce.

The problem is not solvable by ordering the target tiles, because the
dependency between tiles is cyclic. Two units that swap adjacent tiles each
release room for the other, and a ring of units around a closed path does the
same. A cycle has no admissible order.

Three repairs are available. Admit only the departures that are themselves
admitted, which needs a fixpoint and therefore a convergence test that the
determinism rule forbids.[^6] Run a fixed number of admission passes, which is
the form this project already requires of a solver.[^6] Or accept the
overshoot and bound it. The overshoot is bounded: a tile can exceed its
capacity by at most the number of its own units that intended to leave and
were rejected, so the occupancy never exceeds twice the capacity and stays
inside the one-byte count.

Amendment 56-2 takes the fixed-pass repair, because it holds the invariant and
uses a mechanism the project already has.

**Problem B: the parallel claim is too strong.** D3 says the segments are
disjoint, so the admission scan runs in parallel without an atomic operation.
That is true of every write whose address is the target tile, because one
target tile owns one segment. It is false of every write whose address is the
source tile. The units leaving one source tile are scattered across many
segments, because they choose different targets. A departure applied inside
the segment scan is therefore a contended write.

The repair is small: apply the departures as a second reduction after the
scan, not inside it. Sub-step 4 already puts the departures after the
positions, so the record's own ordering supports the repair. It just does not
say why the ordering is needed. Amendment 56-2 covers this.

### Sub-step 4, write the accepted positions, then the departures, then the arrivals

**Buildable, and under-specified.** The three writes name three different
targets, and D3 never says what the second and third write into. Only D4
mentions a count array. As written, sub-step 4 is a step order for code, which
is close to the module arrangement that the rule keeps out of a record.[^4]
The ordering does carry a constraint, and amendment 56-2 states the
constraint rather than the sequence.

### What no primitive provides

Three things. None of them sinks the record; each is work that item 0023 must
do, and one of them is a decision that no record holds.

1. **A per-tile occupancy count.** The bridge answers `count_on_tile`, and
   that read is a search inside a block, guarded against a stale arena.[^1]
   Admission needs the count for every target tile in the intent set, once a
   frame, which the guarded read can serve. A dense count array is faster and
   is what D4 assumes. No record decides which.
2. **A per-tile departure count.** The bridge offers nothing here. Sub-step 1
   builds it.
3. **A capacity lookup.** No terrain table exists in the code, and the registry
   allocates no record for one. Section 5 treats this.

## 4. The half already built

`World::soldier_moves` is the intent half.[^19] It reads each live soldier in
slot order, draws one direction from the keyed generator on the tuple of
system, frame, entity and draw, and returns the chosen address. It writes
nothing into the world. The step then applies every move and rebuilds the
bridge last.

**The code honours D2's constraint and does not match D2's description.**

The constraint is that a unit does not move itself, that the choice is a pure
read, and that the two halves never interleave. The code holds all three, and
its own comment says so.

The description is that the intent record names the source tile, the target
tile and the unit's stable key. The code returns a pair of the identity and
the target address. It carries no source tile, because the arena already holds
it in the tile column and the identity is the subscript. It carries no
separate stable key, because the identity is the stable key.

**D2 over-specifies.** A three-field record is a struct layout, and a record
should hold the constraint rather than the arrangement.[^4] Worse, the
three-field form invites a second copy of the source tile, which is one fact in
two places with nothing that fails when the copies disagree.[^20] The arena
already holds the source tile, and the arena is the source of truth.

Admission does need the source tile, for sub-step 1. It reads it from the
arena by the identity. Amendment 56-1 states that, and drops the field list.

One further note on the code, not on the record. `soldier_moves` names D2 in
a footnote and says plainly that capacity and admission are not built yet. That
is the honest form, and it is what the registry asks of a record that no code
implements.[^2]

## 5. Capacity, D4, and what item 0023 can honestly build

**D4's constraint is sound and should be kept.** A contributor would
reasonably write a capacity constant in the movement kernel, the reversal is
expensive because every calibration depends on it, and the reasoning is not
visible in the kernel. All three conditions hold. The findings register gives
the evidence that capacity is a calibration lever and not a bound.[^21]

**D4's last sentence must go.** It reads:

> The count array that stores the occupancy of a tile bounds the capacity,
> because the count is one byte for each tile.

Two objections, and the second one holds.

The first objection was that this contradicts ADR-0018, which rejects a
structure that grows with the tile count. That objection fails now. A previous
review already settled it, and the accepted text of ADR-0018 says the project
rejects the offset array and not the per-tile array.[^7] [^14]

The second objection holds. **No record decides that the count array exists,
and no code holds one.** ADR-0018 says it already exists, in a paragraph whose
job is to reject something else. ADR-0056 D4 assumes it and derives a bound
from it. Neither record decides it, and the soldier arena holds no such array.
This is a capability that two records describe and nothing invokes, which is a
shape this project has recorded before.[^20] A record must state what the code
does or the constraint the code must satisfy, never what the author hopes the
code will do.[^4]

The repair is a deliverable, not an edit. **The tile occupancy array needs its
own record, and writing it is work for item 0023.** Amendment 56-4 removes the
assumption from D4 so that the two records stop describing an array that
neither of them decides.

**The smallest honest thing item 0023 can build.** D4 does not require a
terrain system. It requires that the engine holds no capacity constant of its
own, and that it reads the capacity through a lookup. Item 0023 can supply a
capacity source that the caller builds and hands to the engine, indexed by
tile or by a terrain identifier, with the test fixture filled from the scale
constants table.[^16] The engine then holds no literal, the lookup is the seam
that a terrain record later fills, and nothing inert ships.

**D4 does not need amending to permit that.** It needs amending only to drop
the count-array sentence. Item 0023's own text says the item respects the
capacity that BLK-009 fixed at eight; that is the correct reading of the
blocker and it does not conflict with D4, because the value arrives as data.

## 6. `ADR-0057`

### The three-condition test, applied one decision at a time

The record does not pass or fail as one thing.

**D2 passes.** The prohibition on a per-unit search over a long path is a
constraint. A contributor would reasonably give each unit its own search,
because it is the obvious answer and it works at a small unit count. The
reversal is expensive, because a per-unit search spreads into behaviour code
and into content. The reasoning is invisible in any one call site: the cost
appears only at the target unit count.

**D5 passes.** A coarse field does not answer connectivity, because a coarse
cell hides a one-tile gap and a one-tile wall. That is a constraint a reviewer
can find a violation of, and it is the kind of error that looks correct in a
test world.

**D1, D3 and D4 fail condition 3, and D1 and D3 fail condition 1.** They
describe a portal graph, a flow tile, and a cache key. No portal graph exists.
No flow tile exists. No cache exists. Nothing in the project asks for a long
path yet, and the first renderable example does not need one. These three
decisions record an arrangement rather than a constraint, and an arrangement
recorded before it is built is the shape the scope rule warns about: two
reference records that recorded a component shape were superseded within one
and two days.[^4]

The check already reports that nothing cites `ADR-0057`. The rule says to read
that as a question rather than a verdict, and the question here has an
answer: three of its five decisions are a description.[^4]

### Blocker hygiene

**`ADR-0057` names an unanswered question and opens no row for it.** Its last
consequence says the cache hit rate depends on how many distinct routes are
live at once, and that nothing in the project answers that number. That is a
blocker. The blockers register holds one open row, BLK-007, and it is about
measurement on the target platform, not about this.[^15]

This is the inverse of a defect the project has already recorded. That finding
was about a record citing a blocker that had closed; this is a record naming
an open question that no register holds.[^22] Both fail the same way, because
nothing revisits the claim.

The record states no value that BLK-007 governs. It states no measured figure
at all, which is correct.

### Compatibility with ADR-0056 and ADR-0018

No contradiction. `ADR-0057` D4 has a unit score its neighbours from the cost
field and the current occupancy. The occupancy it reads is the barrier
snapshot, because the bridge rebuilds at the barrier and does not change while
systems run.[^7] The read is therefore pure, which is what ADR-0056 D2
requires of intent writing. Worth stating in the record; not a defect.

### Verdict for `ADR-0057`

**LEAVE AS DRAFT. Do not retire the number.** D2 and D5 are real constraints
and retiring the number would lose them.

Three statements must be true before it can be accepted.

1. **Something needs a long path.** A backlog item, a product record, or code.
   Until then D1, D3 and D4 record a design that the project may not build.
2. **The route count has a blocker row.** Open one, and cite it from the
   consequence instead of citing the research report for a number the report
   does not hold.
3. **D1, D3 and D4 are either split out or defended.** The cleanest form is a
   short record holding D2 and D5, which are constraints today, with the
   portal graph and the flow tile deferred to the record that the
   implementation writes. If instead the whole record is accepted, the
   acceptance must say plainly that nothing implements it, which the registry
   permits.[^2]

Accepting it now is permitted by the letter of the registry and is the wrong
call. A record that describes an unbuilt subsystem gets edited the moment
someone builds it, and an edited accepted record needs a supersession.

## 7. The amendments to ADR-0056

### Amendment 56-1 (mandatory). D2, replace the second paragraph

Current:

> A unit does not move itself. It writes an intent record. The intent names
> the source tile, the target tile, and the unit's stable key. Writing the
> intent is a pure read of the world.

Replacement:

> A unit does not move itself. It writes an intent, which names the target
> tile and the unit's identity. The identity is the stable key, and the source
> tile stays in the unit column where it already lives. An intent that carried
> a second copy of the source tile would be one fact in two places, with
> nothing that fails when the copies disagree.[^A] Writing the intent is a
> pure read of the world.

Footnote A is the recurring defect rule, `.claude/rules/recurring-defects.md`,
section 1.

### Amendment 56-2 (mandatory). D3, replace sub-steps 3 and 4 and the closing paragraph

Current:

> 3. Admit the intents of a segment in their sorted order, until the target
>    tile reaches its capacity. The departure count of the target tile
>    releases room in the same tick. Reject the remaining intents.
> 4. Write the accepted positions, then the departures, then the arrivals.
>
> The sort is the engine's stable integer sort. The key ends in a unique
> identifier, so no two intents tie.[^7] The segments are disjoint, so the
> admission scan runs in parallel without an atomic operation.
>
> Sub-step 1 is not an optimisation. Without it a column of units in a
> corridor blocks itself, because the tile ahead still looks full.

Replacement:

> 3. Admit the intents of a segment in their sorted order, until the target
>    tile reaches its capacity. The departure count of the target tile
>    releases room in the same tick. Reject the remaining intents.
> 4. Repeat sub-step 3 for a fixed number of passes. Each pass recomputes the
>    departure count from the intents that the previous pass admitted, and it
>    admits only intents that no earlier pass rejected. The pass count is
>    fixed. Admission never runs to convergence and never takes a time
>    budget.[^B]
> 5. Apply the admitted moves. Write the positions first, then the departures,
>    then the arrivals.
>
> The sort is the engine's stable integer sort. The key ends in a unique
> identifier, so no two intents tie.[^7]
>
> **A single pass does not hold the capacity.** A departure count counts
> intents, and an intent is not a departure. A unit that intends to leave a
> tile and is then rejected at its own target stays where it is, so the room
> it released was never released, and the tile it stands on can exceed its
> capacity. The dependency between tiles is cyclic, because two units that
> swap tiles each release room for the other, so no order over the tiles
> repairs it. The fixed pass count is what repairs it, and the last pass
> admits a unit only when its own departure is admitted.
>
> The segments are disjoint by target tile, so the admission scan writes each
> target tile from one segment only and needs no atomic operation. The
> departures are not disjoint, because the units leaving one source tile
> choose different targets and land in different segments. Sub-step 5
> therefore applies the departures as a separate reduction after the scan, and
> never inside it.
>
> Sub-step 1 is not an optimisation. Without it a column of units in a
> corridor blocks itself, because the tile ahead still looks full.

Footnote B is ADR-0001, `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`.

Add one consequence:

> **The pass count is a cost, and it bounds how far a queue moves in one
> tick.** A corridor longer than the pass count does not clear in one tick.
> The value is a calibration parameter and the scale constants table holds
> it.[^10]

The pass count must not appear in the record as a number.

### Amendment 56-3 (mandatory). D3, add one paragraph after sub-step 1

> Admission reads the occupancy of a target tile as it stood at the frame
> barrier. The unit-to-tile bridge is that occupancy, and it does not change
> while systems run, so every intent in one frame is admitted against one
> consistent world.[^C] Admission never writes the bridge.

Footnote C is ADR-0018,
`docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`.

This closes the gap that a previous review left open, and it makes ADR-0018's
consequence true.[^14] If the project instead decides that admission reads a
dense count array and not the bridge, then ADR-0018's consequence is the text
that must change, and it is accepted, so that needs a supersession. Reading
the bridge is the cheaper and the truer answer, because the bridge exists.

### Amendment 56-4 (mandatory). D4, delete the last sentence

Delete:

> The count array that stores the occupancy of a tile bounds the capacity,
> because the count is one byte for each tile.

Replace with nothing. The bound belongs to whichever record decides the
occupancy array, and no record decides it yet. D4's constraint stands without
it.

Add one consequence to the record:

> **The storage of the tile occupancy needs its own record.** Two records
> describe a per-tile count array and neither decides it. The work that builds
> admission writes that record.[^A]

### Amendment 56-5 (recommended). D1 and D5, cite the storage record

D1 decides a progress accumulator and D5 decides a rejection count. Both are
columns on the soldier shape, and entity storage holds four fixed shapes with
one column set for each. Cite that record from both decisions, at
`docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`.

Add to D5:

> The threshold is a calibration value. This record states no value, and the
> scale constants table holds it.[^10]

### Amendment 56-6 (recommended). The context, attribute the non-compaction claim

The context says the unit arena is never sorted, because the slot index is
half of the entity identity. That claim now appears in three records: the
identity record's consequences, ADR-0018 D1, and here.[^7] [^8] Three
declaration sites for one fact is the first recurring defect shape.[^20]

Keep the sentence, because the record needs it to explain why admission sorts
the intents and not the units. Make it a citation rather than a restatement:

> The intent set is a scratch array that admission owns, so sorting it
> reorders nothing that the arena owns. The arena itself is never sorted.[^11]

## 8. Objections attempted

A verdict with no attempted objection is not a verdict.[^23]

### Against ADR-0056

1. *The record is not needed, because a hex tile world has no other choice.*
   **Failed.** A tile world can still hold a sub-tile offset for the renderer
   and for collision, and several shipped games do. The record forbids that,
   and the prohibition is the value.
2. *D1 belongs in the code, because the tile field is visible in the soldier
   column set.* **Failed.** The field is visible; the two rejected
   alternatives are not, and neither is the reason that a velocity is
   forbidden rather than merely unused.
3. *D3 records an algorithm, and an algorithm is an arrangement that section
   4.4 keeps out.* **Failed.** D3 states which answer the engine must produce,
   not where the code lives. The four sub-steps are the definition of the
   answer, and a reviewer needs them to find a violation.
4. *D3 is nondeterministic, because sub-step 3 reads a count that sub-step 3
   also writes.* **Failed.** The read and the write are both inside one
   segment, one segment has one target tile, and the segments are disjoint by
   target tile. The order inside a segment is the sort order.
5. *D3's departure credit breaks the capacity invariant.* **HELD.** This is
   amendment 56-2, problem A. The failure is invisible to both determinism
   tests, because the wrong answer is the same wrong answer at every thread
   count. A determinism test cannot tell correct from consistently wrong.
6. *D3's parallel claim is false for the departure write.* **HELD.** This is
   amendment 56-2, problem B.
7. *D3 cannot be built, because the sort cannot take two fields.* **Failed.**
   The key vector takes any field count, and the identity supplies the unique
   last field.[^1] [^13]
8. *D4 contradicts ADR-0018's rejection of a per-tile structure.* **Failed.**
   ADR-0018's accepted text already draws the distinction between the offset
   array and the per-tile array.
9. *D4 assumes an array that nothing decides and nothing holds.* **HELD.**
   This is amendment 56-4.
10. *D4 states a value that a resolved blocker governs.* **Failed.** The
    record states no capacity value and points at the scale constants table.
    The stale blocker reference that an earlier audit found has been
    repaired.[^22]
11. *D5's lateral step is a random draw in simulated state, so it breaks the
    determinism rule.* **Failed.** The draw is keyed on the tuple of system,
    frame, entity and draw, which is the required form, and the existing
    movement code already draws that way.[^17] [^19]
12. *D2 is falsified by the code, which writes no intent record.* **Failed as
    a rejection, held as an amendment.** The code honours D2's constraint. It
    is D2's field list that the code does not need, so the repair is
    amendment 56-1 rather than a rejection.
13. *The record is too long and holds several claims that could be accepted
    separately.* **Failed.** The record is about a thousand words, which is
    below both reference medians, and D1 through D5 are one claim about one
    mechanism.[^4]

### Against `ADR-0057`

1. *The record should be accepted, because the registry permits accepting a
   record that nothing implements, provided the acceptance says so.*
   **Failed.** The permission exists for a record that states a constraint
   before the code arrives. D1, D3 and D4 state a design, and a design gets
   edited when it is built.[^2] [^4]
2. *The record should be rejected and its number retired, because nothing
   cites it and nothing needs a long path.* **Failed.** D2 and D5 are
   constraints that survive whatever the implementation turns out to be, and
   D5 records an error that is expensive to rediscover. Retiring the number
   loses both.
3. *D2 duplicates the control-plane rule that a set-valued command permits a
   cheaper algorithm, so it needs no record.* **Failed.** The project rule is
   general guidance. D2 is a specific prohibition that a reviewer can find a
   violation of, and the general rule cannot reject a per-unit search on its
   own.
4. *D3's cache key is a constraint, because it decides what two commands
   share.* **Failed, narrowly.** It is the strongest of the three design
   decisions and it does carry a claim about sharing. It still names a cache
   that does not exist, and its key is the sort of thing an implementation
   changes on the first measurement.
5. *D4 contradicts ADR-0056 D2, because reading the occupancy at the moment of
   choice is not a pure read.* **Failed.** The occupancy is the barrier
   snapshot and it does not move while systems run.[^7]
6. *The record states a value that BLK-007 governs.* **Failed.** It states no
   figure of any kind.
7. *The record names an unanswered question and opens no blocker row.*
   **HELD.** Section 6 states it, and it blocks acceptance.

## 9. For the registers

Not verdicts. For whoever maintains them.

- **A finding.** A determinism test cannot see a capacity violation, because
  the wrong answer is the same at every thread count. The admission tests need
  a property that asserts the invariant directly: no tile holds more units
  than its capacity, after every tick. Item 0023 must carry that property, and
  the thread-count test alone is not enough. The testing rule already holds
  the general form of this: a determinism test proves that a run repeats, not
  that it was right.[^24]
- **A blocker.** `ADR-0057` needs a row for the number of live routes.
- **A record that nothing decides.** The tile occupancy count array. Two
  records assume it. Allocate a number before the work starts.[^2]

## References

[^1]: The foundation crate. `crates/cachette-core/src/`
[^2]: ADR Registry, who reviews, and the retcon window. `docs/adrs/REGISTRY.md`
[^3]: Backlog item 0023, implement sort-then-admit movement. `docs/backlog/proposed/0023-implement-sort-then-admit-movement.md`
[^4]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^5]: Report 10, crowd simulation and unit movement. `docs/research/reports/10-crowd-and-movement.md`
[^6]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^7]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^8]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^9]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^10]: The soldier column set. `crates/cachette-core/src/soldier.rs`
[^11]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^12]: Findings register, FND-011. `docs/FINDINGS.md`
[^13]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^14]: Review 0029, the storage and tile index records, errors 1 and 4. `docs/reviews/0029-storage-and-tile-index-records.md`
[^15]: Blockers register. `docs/BLOCKERS.md`
[^16]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^17]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^18]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^19]: The world step and the movement intents. `crates/cachette-core/src/world.rs`
[^20]: Recurring defect shapes. `.claude/rules/recurring-defects.md`
[^21]: Findings register, FND-037. `docs/FINDINGS.md`
[^22]: Findings register, FND-042. `docs/FINDINGS.md`
[^23]: Reviews index, what a review must contain. `docs/reviews/README.md`
[^24]: Testing Rules. `.claude/rules/testing.md`
