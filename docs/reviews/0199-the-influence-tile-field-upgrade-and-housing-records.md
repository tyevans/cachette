# Review 0199: The influence, tile field, upgrade and housing records

## What was reviewed

| Item | Value |
|---|---|
| `docs/adrs/draft/adr-0087-an-influence-solve-runs-a-fixed-iteration-count.md` | `Draft` at review, and `Draft` after it |
| `docs/adrs/draft/adr-0088-a-tile-field-is-a-generated-base-and-a-stored-change.md` | `Draft` at review, and `Draft` after it |
| `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md` | `Draft` at review, and `Draft` after it |
| `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md` | `Draft` at review, and `Draft` after it |
| Commit | `3773878`, the head of `feat-w23` at the time of the review |
| Code read | the influence field, the tile value field, the upgrade module, the terrain module, the position table, the cohort table, the household module, the settlement arena, the world step, the admission pass, and the tests of each |

The reviewer wrote none of the four records and none of the code behind
them. The reviewer read the records, the rules and the tree. The reviewer did
not read the reasoning of any author.

**The reviewer compiled nothing.** Other workers hold the machine, so no
`cargo` command ran. Section 6 names every claim that only a run can settle.
None of those is counted toward a verdict.

## Verdict

| Record | Verdict |
|---|---|
| ADR-0087 | Reject, and not for a defect. Every decision holds against the code. It cannot bind while DEC-067 is open, because it would then contradict an accepted record |
| ADR-0088 | Accept in substance, with one bullet of D1 amended. Section 2 holds the exact text. The record stays a draft until the author makes the change |
| ADR-0090 | Accept in substance, with one sentence of D3 amended. Section 3 holds the exact text. The record stays a draft until the author makes the change |
| ADR-0081 | Reject, for the reason a previous review already gave. DEC-057 must close first |

**No record was set to `Accepted`.** Two of the four hold in substance and each
holds one sentence that the tree falsifies. The reviewer could have accepted
those two only by writing the corrections in, and a reviewer that rewrites a
record into acceptability has authored it rather than reviewed it. The
corrections are stated exactly, so each is one edit.

The two rejections are not a judgement of their subjects. The influence solve
is the strongest of the four records and the reviewer failed to break any of
its decisions.

## 1. ADR-0087, the influence solve

### 1.1 The record against the code, decision by decision

**D1 holds, exactly.** The solve is a `for` loop over a constant, and it
returns. It reads no clock. In the ordinary build the residual is not merely
ignored: the function that would compute it is a constant function that takes
its arguments by reference and reads neither, so there is no residual for a
convergence test to read. The stop that follows it is a constant `false`. D1's
strong claim, that no branch anywhere in the solve reads what the field holds,
is true of the ordinary build.

The source injection carries the same property. Each pass adds the source term
to every cell with no test for whether a source is there, so the absence of a
source is the ordinary value rather than a case.

**D2 holds, and its test is the one the record describes.** The field reports
the passes it has run. A test drives four fields — one that holds nothing, one
at rest, one saturated at every cell, and one whose ground conducts nothing —
and asserts the constant times the number of solves for each. Those are the
four cases the record names, in the record's own order.

The test then drives the engine as well and asserts the same count over eight
steps, so the assertion covers the caller that is obligated to solve and not
only the solver.[^1]

The probe half is present. The perturbed build stops the solve when a pass
changed nothing, and a test asserts that the pass count then falls below the
constant. A second test asserts that the same perturbation does **not** move
the thread-count comparison, which is the record's argument for why D2 must
exist at all.

**D3 holds, exactly.** A pass fills one run of a scratch plane and reads the
input plane wherever the stencil points. The neighbour read goes through a
visibility function that is a constant `true` in the ordinary build and clips
to the run in the perturbed one. A test asserts that the clipped build makes
the field differ between one thread and twelve. The record's sentence about
the perturbed build is therefore a description of the tree and not an
intention.

**D4 holds as a description of the code.** The plane is not rebuilt by the
pyramid, it carries between solves, and its pass count and every cell enter
the state hash. No consumer reads it as a summary, because no consumer outside
the crate reads it at all.

### 1.2 Why it is returned

The ADR priority index states the condition: this record names the boundary it
draws against ADR-0022 D1, and a reviewer must settle that boundary before the
record moves.[^2]

The reviewer cannot settle it. ADR-0022 D1 is accepted and says that a value
which appears only at a level above level 0 is a defect. ADR-0087 D4 says this
plane holds such a value, and it says honestly that it does not claim the
level record permits it. Accepting the record would put two accepted records
in direct conflict, and the disclaimer in D4 documents the conflict rather
than resolving it. A contributor holding both would be entitled to refuse the
influence field, and the record exists to stop exactly that refusal.

DEC-067 holds the choice with three options and recommends the first: a record
that supersedes ADR-0022 D1 and names the case.[^3] Writing that record is
authoring, and it is not this review.

**Nothing in ADR-0087 needs to change.** It is held by a dependency and not by
a defect. When DEC-067 closes, D4's last paragraph becomes stale in one
direction or the other, and that is the only edit the closure will ask for.

### 1.3 Every objection the reviewer attempted

**Objection 1: the constant is a figure a measurement can change, which the
scope rule bans.** **Failed.** The record states no number at all. It states
that the constant is a reach rather than a budget, that no measurement chose
it, and it cites the blocker that owns every cost figure. The number lives in
the code as a named constant. That is the treatment the rule asks for.

**Objection 2: D2 is a testing requirement and not an architectural
constraint.** **Failed.** D2 constrains the public interface of the field: the
pass count must be observable. Without it D1 is unfalsifiable, and the record
argues that from a recorded case rather than from taste. A contributor who
removed the accessor as unused would remove the only guard D1 has.

**Objection 3: D1 is unenforceable, because a later contributor can add a
branch.** **Failed.** The perturbed build is the enforcement. It adds the
branch deliberately and a test asserts the failure, so the guard has a proven
failure mode.

**Objection 4: the record claims a capability nobody invokes.** **Failed for
the pass count, and it is worth stating where it nearly held.** The pass count
has two callers in the tests and one of them drives the engine. The field
itself has no consumer outside the crate, which the analysis note already
records, but ADR-0087 makes no claim about a consumer. The claim it does make
about a consumer is a prohibition, and a prohibition needs no caller.

**Objection 5: the record holds two claims and should be split, because D4 is
about storage and D1 to D3 are about the solve.** **Failed.** D4 is the
premise D1 rests on: the reach grows over ticks only because the plane carries,
and a plane that did not carry would need a pass count that no tick could
afford. The two cannot be accepted separately.

**Objection 6: the boundary against ADR-0022 D1 is unresolved.** **Held.**
Section 1.2.

## 2. ADR-0088, the tile field

### 2.1 The record against the code, decision by decision

**D1 holds in substance, and one of its three checkable properties is false.**
The field holds the seed, the extent, and one entry for each changed tile. Its
constructor is a `const fn` that allocates two empty vectors, so a field of any
extent costs the same to build. The stored part grows with what the frames
changed, and a public reader reports that count. Two worlds from one seed hold
one field, and a test asserts it by comparing whole copies.

The false property is the first one, and section 2.2 holds it.

**D2 holds.** The entries are sorted by tile index and a lookup is a binary
search. A merge takes a run in ascending order, asserts the order in a debug
assertion, and builds into a kept scratch buffer, so no entry is inserted into
the middle of a vector. The step sorts its joined run by tile index before it
merges, and the comment at that call site names the sort as the thing that
makes the field the same at any thread count.

**D3 holds.** The hash walks every tile in ascending index and writes the
combined value. It writes neither the seed nor the stored part alone. The
record's argument for why is the ground record's argument, and it is applied
correctly.

**D4 holds, and its test exists.** The generated part is a keyed draw whose
entity slot holds the tile index, whose frame slot holds a named constant, and
whose draw slot holds a named constant. Every arithmetic step goes through the
arithmetic module. The record warns that a key without the tile index gives one
value to every tile and that only a test of the key finds it. **That test is in
the tree**, it is named for the property, and it asserts that two tiles do not
hold one value. The reviewer went looking for its absence and did not find it.

### 2.2 What must change

D1 states, as the first of three properties a reviewer can check:

> - Building a world visits no tile of the field and allocates nothing for it.
>   The seed and the extent are the whole of a new field.

**The first half is false, and the record's own consequences say so.** Four
paragraphs later the record states that the first level of the pyramid sums
the value of every tile and therefore sweeps the field "at every barrier and
once when the world is built". Both sentences describe building a world, and
they disagree.

The tree agrees with the consequences and not with D1. The visit census counts
the tiles the field generates, and the build test asserts that building a world
of any extent makes one visit for each tile. Its own module comment says the
value field contributes no visit and that one visitor remains.

The register already holds this. A finding states plainly that a world is still
not built without a pass over every tile, and it records that the previous
repair removed the column and left two passes, of which one remains.[^4] A
record that contradicts a finding is the failure this project has recorded
twice, and here it does so in the one bullet that invites a reviewer to check
it.

**Replace:**

> - Building a world visits no tile of the field and allocates nothing for it.
>   The seed and the extent are the whole of a new field.

**With:**

> - Building the field visits no tile and allocates nothing for it. The seed
>   and the extent are the whole of a new field. Building a **world** still
>   visits each tile, because the first level of the pyramid sums the value of
>   every tile, and the consequences below name that reader and the item that
>   holds its removal.

### 2.3 Every objection the reviewer attempted

**Objection 1: D3 contradicts D1, because a hash of every tile is the pass the
product record forbids.** **Failed.** D3 answers it in its own text: D1 governs
what the engine stores and what a build costs, and it makes no promise that
every reader is cheap. The build does not hash. The distinction is stated
rather than left to the reader.

**Objection 2: the record supersedes ADR-0012 in substance without saying so.**
**Failed.** The record states what it leaves standing, names the exception
clause of the earlier record, distinguishes the case the clause delegates from
the case it does not, and cites the open row that holds whether the earlier
record should be superseded.[^5] That is more work than the objection asks
for.

**Objection 3: the record claims a capability nobody invokes.** **Failed for
every claim it makes.** The stored-change count has a test caller. The visit
census has a test binary. The whole-column copy is named for the copy and has
callers. The key test exists.

**Objection 4: the record holds a figure a measurement can change.** **Failed.**
It states no figure. It says the growth has a shape and that no measurement
exists, and it cites the blocker.

**Objection 5: D1's first checkable property is false.** **Held.** Section 2.2.

## 3. ADR-0090, the tile upgrade

### 3.1 The record against the code, decision by decision

**D1 holds.** The map holds one entry for each improved tile and nothing for
any other. Entries are sorted by tile index, a lookup is a binary search, and a
run is merged in ascending order. The advance pass takes the builders and the
map and takes neither a grid nor a tile count. The map reports the entries the
last advance read, and a test asserts that the count is the same in a small
world and a large one, and zero in a world where nobody built.

**D2 holds.** The progress accumulator is 64 bits wide, every contribution is
a whole number, and the accumulator is clamped at the work its kind asks for.
The clamp reads the catalogue rather than a second constant, in both the path
that opens an entry and the path that advances one. The kind and the progress
both enter the state hash.

**D3 holds in its substance, and its second sentence is false.** One function
composes the ground capacity with the finished upgrade, an upgrade never lowers
what a tile holds, ground that admits nobody stays closed, and a site under
construction changes nothing. The upgrade row reads the crossing capacity from
the terrain module rather than restating it. Section 3.2 holds the false
sentence.

**D4 holds.** Destroying an upgrade removes the entry, and nothing else stores
a property of an improved tile. The public verb that removes one exists and is
a control-plane verb, so its caller is the user of the library rather than the
engine.

### 3.2 What must change

D3 states:

> **One function reads both tables**, and every caller that asks how many units
> a tile holds calls it.

The first clause is true. **The second is false, at three call sites.**

The composition has two callers: the public reader that reports the capacity of
a tile, and the admission pass. Three other places ask how many units a tile
holds and read the ground alone.

- The position table bounds the positions of a site by a helper whose own
  comment says the answer comes from the terrain capacity table "and from
  nowhere else". Both the rebalance and the invariant check use it.
- The founding seats a group over the disc of a place and fills each tile up to
  the capacity of its ground.
- The founding survey adds the capacity of the ground of each tile when it
  estimates the room a place offers.

**The claim is false, and it is fortunate that it is false.** The width of the
position table is folded from the terrain capacity table, and a finished road
states a capacity above every value in that table. A position table that
followed the composition would be asked to hold more positions than its row can
carry. The code is self-consistent today precisely because the position table
does not call the composition.

That divergence is a code question and not a record question, and it is now in
the registers.[^6] [^7] What the record must do is stop claiming a universal
that the tree does not hold.

**Replace:**

> **One function reads both tables**, and every caller that asks how many units
> a tile holds calls it. A second rule beside it would be one fact in two
> places, and nothing would fail when the two disagreed.

**With:**

> **One function reads both tables**, and every caller that asks how many units
> may stand on a tile calls it. A second rule beside it would be one fact in
> two places, and nothing would fail when the two disagreed.
>
> A caller that asks a different question reads the table its question is
> about. The number of work positions a site opens is bounded by the ground,
> and a register row holds whether it should follow the composition
> instead.

### 3.3 Every objection the reviewer attempted

**Objection 1: the record names a fraction that no blocker answered.**
**Failed.** The blocker that asks what fraction of tiles carry an upgrade is
resolved, the record cites the blocker for the answer and the reference table
for the figure, and it states no figure of its own.

**Objection 2: the record states the target tile count in its body, which is a
figure.** **Failed.** The tile count is the extent the project chose, in the
same way the cache line size of the target is a property of the platform it
chose. The scope rule names that case and permits it. The record states no cost
against it.

**Objection 3: D1 claims a reported visit count that nothing reads.** **Failed.**
The count has a test caller, and the test asserts the property D1 exists for:
the same build costs the same in a small world and a large one.

**Objection 4: the upgrade catalogue restates a capacity, which the record
forbids.** **Failed.** The row returns the constant from the terrain module,
and both the constant and the row say in their own comments why it lives
there.

**Objection 5: D2's clamp is a second declaration of the work of a kind.**
**Failed.** Both clamp sites call the catalogue accessor. No literal appears.

**Objection 6: D3 claims every capacity caller composes.** **Held.** Section
3.2.

## 4. ADR-0081, the residence

### 4.1 The record against the code, decision by decision

**D1 has no code.** No settlement holds a housing capacity. The reviewer
searched the settlement arena, the site module and the world for a housing
field and found none. The decision is an argument about a shape that does not
exist yet, which the registry permits for a draft and which its acceptance
would have to state plainly.

**D2 holds.** One column names the site a unit belongs to, and it is the home
column of the unit arena. There is no second column. The household module reads
that column backwards and stores nothing, and its own comment gives the same
argument the record gives. A unit that lives nowhere carries the value that
means no home.

**D3 rests on a premise the tree falsifies.** The record argues that the count
is worth storing because a caller asks for it often and no cheap derived answer
exists. The engine already keeps the answer. The cohort table holds one row for
each faction at each site, each row holds a headcount derived from the home
column, and both the row reader and the total are public. A previous review
found this and the register records it.[^8]

The record has not been changed since. It does not name the cohort table
anywhere, so it does not argue against the answer that exists; it argues in a
world where the answer does not.

**D4 holds.** No structure stores the residents of a site. The reverse read
walks the live units in slot order.

### 4.2 Why it is returned

The open row states the outcome plainly: the housing draft states decision D3
as its second option, and it must be rewritten against whatever that row
decides.[^8] The row is still open and still recommends reading the count the
engine already keeps.

This review adds two things the previous one did not say.

**D3's second argument has no caller either.** The record says the admission of
a birth reads the same number. No birth exists in the tree, and the record that
would create one is itself a draft that rests on this one.[^9] So the frequent
caller D3 offers is a watcher, and the watcher can already read the cohort
rows.

**D1 has no code at all, and the priority index does not say so.** The index
states its own rule: a record that runs ahead of the code says so in its own
row, because a reviewer must know whether a claim was tested or only argued.
The row for this record does not. That is repaired in this change.

### 4.3 Every objection the reviewer attempted

**Objection 1: D2 is already false, because the engine holds a dwelling as well
as a home.** **Failed.** The dwelling is the home column read through the
settlement arena. There is one column, and the world reader says so and cites
the finding that made the two one fact.

**Objection 2: D4 is already false, because a household reader exists.**
**Failed.** The household reader is a pass over the units, which is exactly what
D4 says a caller must do. It stores no index. An open item holds the reverse
index and it waits for a measurement to ask for one.

**Objection 3: D1 is a module arrangement, which the scope rule bans.**
**Failed.** D1's claim is that the ground does not set the capacity. That is a
constraint on where the value comes from, not on where the code lives.

**Objection 4: the previous rejection may be stale, because the cohort table
could have moved.** **Failed.** It was checked again. The table still holds a
per-site, per-faction headcount, still derives it from the home column, and
still exposes both a row reader and a total.

**Objection 5: D3 states a premise the code falsifies.** **Held.** Section 4.1.

## 5. What the review found beyond the records

**The largest capacity the engine states is folded from one of two tables.**
The terrain module holds the ordinary ground capacity and the crossing capacity
of a made way, and both say in their own comments that they live together so
that no second declaration can disagree. The fold that reports the largest
capacity walks the terrain kinds only, so it does not see the crossing
capacity, which is the larger of the two. The width of the position table is
that fold, and its own guard comment says the clamp takes no effect today. On a
tile with a finished road it does. This is recorded as a finding, and the
choice it opens is a register row with a backlog item behind it.[^6] [^7] [^10]

**Nothing in the demonstration builds an upgrade.** The build order is a
control-plane verb and no engine rule issues one, so the divergence above is
reachable through the public interface and is not reached by a run today. An
open item holds the rule that would make a unit choose to build.[^11]

## 6. What only a run can settle

The reviewer compiled nothing. None of the claims below is counted toward a
verdict.

- **Every test named in this review passes.** Each was read, not run.
- **The visit census reports what the build test asserts.** The counter is a
  process-wide atomic behind a feature, and the recipe that runs it says the
  binary must run on one thread. Whether it does was not observed.
- **The perturbed builds fail the tests this review says they fail.** The
  probe functions were read and the assertions were read. Neither was run.
- **The position table would break if it followed the composed capacity.** The
  reviewer derived this from the constants and the array width. No run
  produced it.

## 7. Checks run

Five document checks were run. All five pass. They compile nothing.

| Check | Result |
|---|---|
| `scripts/check_adrs.py` | 0 failures |
| `scripts/check_footnotes.py` | 0 failures |
| `scripts/check_priority.py` | 0 failures |
| `scripts/check-citations.sh` | 0 failures |
| `scripts/check_conflict_markers.py` | 0 failures |

The whole gate was not run, because other workers hold the machine and this
change touches no source file.

## References

[^1]: Testing Rules, section 5. `.claude/rules/testing.md`
[^2]: Decision record priority index. `docs/adrs/PRIORITY.md`
[^3]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^4]: Findings register, FND-162. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-068. `docs/DECISIONS.md`
[^6]: Findings register, FND-193. `docs/FINDINGS.md`
[^7]: Decisions register, DEC-081. `docs/DECISIONS.md`
[^8]: Decisions register, DEC-057, and findings register, FND-128. `docs/DECISIONS.md`
[^9]: ADR-0082, the store sets the rate of a birth and the housing admits it. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^10]: Backlog item 0200, give one answer to how many units a tile holds. `docs/backlog/proposed/0200-give-one-answer-to-how-many-units-a-tile-holds.md`
[^11]: Backlog item 0180, let a unit choose to build. `docs/backlog/proposed/0180-let-a-unit-choose-to-build.md`
