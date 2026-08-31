# Findings (Register)

This document is a **register**. It records things the project believed and
then had to correct.

Its purpose is precedent. When a conflict arises, look here first. A finding
may already settle it.

Numbers are permanent. Never reuse one. A finding is never deleted, because a
superseded finding still explains why the project once believed otherwise.

Each entry gives what the project believed, what is true, the evidence, and
what follows.


## Allocating a number

**Claim the next number below before you write the row.** Increment it in the
same change that adds the row.

A writer that numbers a row by reading the last row collides with any other
writer working at the same time. That happened, and it is recorded as
precedent.[^1]

**Next number: FND-053**

## A. Corrections to stated rules

### FND-001 — A monoid needs EXACT associativity

**Believed:** an aggregate needs an associative combine operation.

**True:** it needs an **exactly** associative one. Float addition is not
associative, so a float sum is not a monoid. A pyramid built on float sums
drifts away from level 0 as the recombination order varies with which blocks
are dirty.

**Evidence:** the event sourcing report. Three other reports reached the same
conclusion from different directions.

**Follows:** no floating point in aggregated state. Integer or fixed-point
accumulators only. This is one of the four independent justifications for the
no-float rule.

### FND-002 — Incremental update needs a group, not a monoid

**Believed:** a monoid is enough to maintain the pyramid.

**True:** a monoid is enough to **build** it. Incremental update needs an
inverse, which is a group. Minimum and maximum have no inverse.

**Evidence:** the hex grid report. The research agenda later found this is the
standard maintenance condition from the incremental view maintenance
literature in databases. **The project derived a known theorem by hand.**

**Follows:** store minimum and maximum with a count of children at the
extremum, and rescan only when the count reaches zero. Store a popcount for
each bit rather than a bare OR mask, which yields OR, AND and counts, all
delta-updatable.

### FND-003 — Zero-copy NumPy is incompatible with chunked storage

**Believed:** component arrays can be exposed to Python as zero-copy views.

**True:** a chunked layout has no flat per-component array. One million units
is about 4,000 chunks.

**Evidence:** the Python boundary report.

**Follows:** dropping archetype chunking would restore whole-column zero
copy. Subsets never are. `to_numpy()` copies; document that plainly. Tile
data is genuinely flat, so it is the honest flagship demonstration.

**Later:** the project did not drop chunking. BLK-002 closed with four fixed
entity shapes, and the record that holds that claim accepts this cost
explicitly.[^2] A caller now reads one view for each shape, or takes a
copy. This finding stands as the reason the cost was known in advance.

### FND-004 — Opinion storage is linear, not quadratic

**Believed:** sparse opinion edges scale with the square of the character
population.

**True:** they are quadratic only if out-degree grows with population. A hard
out-degree cap makes storage exactly `N x K x edge_bytes`, which is linear.

**Evidence:** the character report and the vector report, independently.

**Follows:** the cap is the mechanism. It must be enforced, not assumed.

### FND-005 — Speed and throughput at a chokepoint

**Believed:** unit speed does not help at a chokepoint, because throughput is
capacity divided by dwell.

**True:** the formula is right; the conclusion does not follow. Dwell is
itself a function of speed.

**Evidence:** the group spatial report.

**Follows:** speed and throughput are the **same** knob below one tile per
tick, and independent above it. Cavalry gain a real advantage at a crossing,
with a hard ceiling at the dwell floor.

### FND-006 — Hawkins-Simon does not test runtime insolvency

**Believed:** an insolvent institution would fall out of a failed
input-output solve.

**True:** the coefficient matrix comes from the immutable recipe table, so it
is content, not state. Every faction sharing a recipe table shares one
spectral radius. A divergent series also does not fail visibly under
saturating integer addition; it saturates.

**Evidence:** the needs and economy report.

**Follows:** Hawkins-Simon is a **bake-time content validator**, which catches
an unproductive recipe table before release. Runtime insolvency is an explicit
ledger comparison. One piece survives: comparing solved gross output against
productive capacity gives **structural** insolvency, distinct from financial.

## B. Claims refuted

### FND-007 — The promotion and demotion problem does not exist

**Believed:** materialising plausible level-0 detail from a level-1 summary,
when a player zooms in, is the hardest part of the design.

**True:** the design conflated freezing computation with discarding data.
Level 0 is only about 134 MB, so nothing is ever discarded.

**Evidence:** the algorithms report.

**Follows:** freeze the processing, keep the data. Coarse background
simulation becomes active-set simulation. The largest scope risk in the
original design was imaginary.

### FND-008 — Aperture-7 hex hierarchies aggregate exactly

**Believed:** H3-style aperture-7 nesting is inexact, so it cannot aggregate
correctly.

**True:** it aggregates exactly over the logical index set. The boundary is
geometrically jagged, but no tile is double-counted or missed.

**Evidence:** the hex grid report. It also warns that H3's pentagons come from
the icosahedral projection and do not exist in a flat world, so that is not a
valid argument either.

**Follows:** the conclusion stands but the reason changes. Reject aperture-7
for non-power-of-two index maths, absent cache alignment, and non-contiguous
children.

### FND-009 — Absent literature is not impossibility

**Believed:** integer and fixed-point eikonal solvers cannot be relied on,
because no published literature exists.

**True:** no literature exists, and the solver works exactly anyway. The only
non-linearity is integer square root, which is exact. The update is monotone,
so it terminates. Sweep order is a compile-time constant. The fixed-point form
is **more** reproducible than the float form.

**Evidence:** a citation check found the absence; the crowd and movement report
derived the result.

**Follows:** **this is the most important finding in the register.** The
project's hardest constraint sits at right angles to most published numerical
work, because that work assumes floating point. When a method has no integer
literature, derive it rather than abandoning it.

### FND-010 — A region is not stable under movement

**Believed:** a formation could be a place rather than a membership list, so
that command is a selector over an aggregate.

**True:** a region is not stable under movement, so a move order changes its
own recipient set across frames.

**Evidence:** the character report, which rejected the idea on five functional
grounds.

**Follows:** formation membership is an ownership column with a reverse index.

## C. Defects found in specified rules

### FND-011 — The progress accumulator overflows

An unclamped accumulator lets a unit whose speed exceeds the local step cost
bank unspendable surplus, overflowing `u16` in about 341 ticks. The
accumulator is simulated state, so an overflow enters the frame state hash and
breaks both the golden-file test and the thread-count equivalence test.

**A movement bug that presents as a determinism failure.**

### FND-012 — Integer decay has a permanent negative bias

The obvious decay, `(x * k) >> 16`, sends positive values to exactly zero but
sticks negative values at minus one forever. Fixed with a sign-symmetric
ceiling decrement.

Left unfixed, every entity drifts slightly negative on every dimension.

### FND-013 — Opinion converges without an anchor

Without an anchor term every entity converges to the same vector. This is
proven, not merely a risk. The fix is an immutable birth anchor and it is
**not optional**.

### FND-014 — A flat field makes everyone a mover

Unemployment needs no special case, but a job field with no gradient makes
every entity a mover and triples movement cost. Needs a score floor.

### FND-015 — A cap defined against a world maximum inverts

`MAX_CAMP_TILES` was defined against the world's maximum capacity. Raising
bridge capacity tightens rather than loosens it. Define against ordinary
capacity.

**General lesson:** a constant derived from another constant rots silently.
Prefer removing the dependency. The straggle threshold was fixed this way — by
not counting queued or gated ticks, it stopped depending on capacity, dwell,
tick rate and width all at once.

### FND-016 — A capacity cap is not a negative rate

The likeliest defect in the field framework is writing a capacity cap as a
sink. Caps are idempotent; sinks add twice.

### FND-037 — A crossing time needs the terrain multiplier

**Believed:** a dwell of 2 ticks and a capacity of 8 units give a crossing of
12.5 seconds for a formation of 1,000 units.

**True:** that arithmetic used the ordinary ground step cost for a mountain
exit tile. It omitted the terrain multiplier. With the multiplier applied, the
same combination gives a mountain crossing of about 50 seconds. The
combination that meets the target is a dwell-2 baseline with a capacity-16
crossing.

**Evidence:** the movement timing note.[^3] It measured 12.9 seconds for the
corrected combination. The closed-form throughput law gives 12.5 seconds for
the same parameters. The 4-tick difference is unresolved and is recorded in
the note.

**Follows:** a crossing time is a function of three quantities, not two.
Capacity, dwell and the terrain multiplier all enter it. **Check that a rate
derivation names every multiplier on its path.** No record states the mountain
multiplier, so the 50-second figure implies its value rather than citing it.

### FND-035 — The float ban lint does not catch an inferred literal

**Believed.** A lint enforces the boundary that keeps floating point out of
simulated state. The project invariant says so plainly.

**True.** The lint catches a named type. It does not catch `let x = 1.5;`,
where the type is inferred and never written. It also cannot name the
reassociating methods, because those do not resolve on the pinned toolchain,
so a banned-method entry cannot refer to them.

**Evidence.** The scaffolding work proved both gaps by injecting a float, and
closed them with a script that reads the source directly.

**Follows.** State the enforcement as two mechanisms, not one. A lint plus a
script. An invariant that names one mechanism invites a reader to trust it
alone. The word "lint" in the project instructions is now wrong on its own.

## D. Cost estimates that were wrong

### FND-017 — A decision costs 4.1 nanoseconds, not 400

The needs report assumed random gathers. They are sequential, because units
are sorted by tile index and the fields are level-1 planes that stay in cache
with about fifteen times reuse for each cell.

Correcting the needs report's own cohort decision line drops it from 16.00
core-ms to under 0.05. That line was 92 percent of its subsystem.

**Follows:** individual decisions became a design choice rather than a budget
one.

### FND-018 — Needs decay is not the cheapest kernel

**Believed:** needs decay over one million individuals is the cheapest thing
in the engine.

**True:** 4.8 core-ms with four needs, which is more than the whole influence
map layer. Affordable, but overstated.

### FND-019 — State sharing saves ten times less than estimated

**Believed:** sharing needs state across formation members saves about 2 ms
per tick.

**True:** 0.03 to 0.16 ms, because the needs pass already runs every ten
ticks. Storage is 16 MB, not 6 MB.

**Decisive argument:** apportionment exists specifically to give members
different integers, so members diverge after their first meal.

**Follows:** **share what is configured, never what is accumulated.**

### FND-020 — Full snapshots are not viable

Copying 16.7 million tiles takes about 27 milliseconds, longer than a frame.
Chunk-level copy-on-write driven off the existing dirty bitset is mandatory,
not an optimisation. It is also the mechanism a future rollback needs.

### FND-021 — The old fog design broke its own budget

A dense `u8` visibility counter costs about 868,000 cache-missing writes for
each faction each tick at late-game scale, roughly 87 core-ms. The mechanism
that existed to make fog cheap was the most expensive thing in the frame.

## E. Layout and platform corrections

### FND-022 — Array-of-structs for characters, struct-of-arrays for cohorts

**Believed:** struct-of-arrays vectorises well, so use it everywhere.

**True:** the character pass is a random graph gather, so struct-of-arrays
touches twelve cache lines for each candidate and array-of-structs touches
one. A twelve times difference.

**Follows:** layout follows access pattern, not habit. Sequential passes want
struct-of-arrays; random gathers want array-of-structs.

### FND-023 — Stagger by cell index, not entity id

Staggering periodic work by entity id scatters the active fraction through a
16 MB array and costs three to four times more than staggering by a mix of the
level-1 cell index, which keeps runs contiguous.

### FND-024 — LSE atomics are already enabled

**Believed:** Large System Extensions must be enabled explicitly on aarch64,
or atomics compile to load-linked retry loops.

**True:** outline-atomics has been the default on Linux aarch64 since about
Rust 1.57.

**Follows:** the `target-cpu` flag is still worth setting, but not for this
reason.

### FND-025 — Hex geometry cuts both ways

Hex diffusion is **better** than square: directional error at a six-cell
feature is 0.035 percent for the hex seven-point stencil against 0.14 percent
for the best square nine-point, with two fewer taps and no timestep penalty.

Hex path metric is **worse**: a six-connected lattice has 15.5 percent
worst-case path error against 8.2 percent for an eight-connected square grid.

Both are true. Diffusion likes hex; distance does not.

## F. Sourcing

### FND-026 — Games do not document their implementations

Eight subsystems across seven games are community-wiki only, with no developer
documentation: Victoria 2 formulas, Dwarf Fortress needs, The Sims decay
rates, Anno tiers, Planetary Annihilation pathfinding, Crusader Kings
succession, RimWorld work priorities, and the Nemesis system.

Every verification pass found the same result.

**Follows:** cite a game only for observed behaviour, never for
implementation. Citations must come from operations research, numerical
analysis, and academic simulation.

Verified exceptions worth knowing: a primary source **does** exist for The
Sims object model, in Northwestern University course notes by Forbus and
Wright, 2001, hosted on the author's institutional page. No Will Wright
conference talk on the subject exists. The Nemesis system has a granted patent
whose claims can be read directly, which is the one citable implementation
description in the survey.

### FND-027 — Citation errors corrected

- Mike Lewis, not Mark Lewis. "Escaping the Grid" is influence mapping, not
  utility. The session lead put the wrong name in a research brief.
- There is no needs-based AI chapter by Dill. Cite Zubek, *Game Programming
  Gems 8*, 2010.
- *Assignment Problems* is Burkard, Dell'Amico and Martello, SIAM 2009. Toth
  and Martello wrote *Knapsack Problems*.
- Leontief's *Structure of American Economy* is 1941 first edition; 1951 is
  the revised second.
- Versu is 2014, volume 6 number 2.
- Tarn Adams edited *Procedural Generation in Game Design*; he did not write a
  chapter in it.

## G. Process

### FND-028 — Concurrent agents collide on shared numbering

Three decision-number collisions occurred because agents chose their own
numbers or were told to continue from the end of a document they could not
see being edited.

**Follows:** a registry allocates numbers, and an entry is made before the
work starts. Decision numbers are local to their record. One agent asked
before taking numbers outside its range, and that is what caught the third
collision.

### FND-029 — A stale read produces a confident wrong answer

The vector report computed against a copy of the character report taken five
minutes before that report was revised. Its conclusion about storage was
therefore wrong, and it rated that argument its weakest on the strength of the
stale figure.

**Follows:** when two documents are written concurrently, verify the read
rather than trusting the citation. The same failure was caught twice by
searching the file rather than believing the summary.

### FND-030 — A scoped guarantee stated without its scope reads as the wider one

**Believed.** The public description said the engine reproduces a run bit for
bit at any thread count. That sentence is true.

**True.** A reader takes it to mean more than it says. The guarantee holds for
one binary. A different processor or a different compiler may produce a
different hash, and the record says so.[^4] Five readers saw the wider claim,
and the fifth named it as the one thing it did not believe.

**Evidence.** Five independent cold reads of the public description. Each
reader had that document and nothing else.

**Follows.** State the bound of a guarantee in the same sentence as the
guarantee. A true statement that is read as a larger statement is a defect in
the writing, not in the reader. The public description now says that
reproducing a study means keeping the binary, not only the seed.

### FND-031 — Opinion state belongs to the character tier, not to every unit

**Believed.** A draft of the public description said each of the one million
units holds an opinion of the people near it.

**True.** Opinion is a character property. The character tier is planned in
the tens of thousands, not the millions. The entity tiers make units cheap by
not giving them a relation graph.

**Evidence.** The entity tier design, and the storage argument that keeps
opinion linear by capping out-degree.[^5]

**Follows.** A description of the product must not promise a property that
the entity model does not carry. Check any per-unit claim against the tier
that owns the field.

### FND-032 — A cold reader finds what an informed reviewer cannot

**Believed.** Reviewing a document against the project's own records is
enough to find its defects.

**True.** It is not. Five review cycles, each by a reader that could read the
one document and nothing else, found three factual errors that internal
review had passed: an overclaimed per-unit property, a guarantee stated
without its scope, and a disclaimer that defended a figure the document did
not contain.

**Evidence.** The five review cycles. The reviewers were forbidden to read
another file, search the repository, or look anything up.

**Follows.** For any document that faces outward, review it at least once
with a reader that has only that document. An informed reviewer repairs a gap
from memory and never notices the gap.

### FND-033 — Record length predicts how often a record is edited

**Believed.** A long record is thorough. Length was treated as a cost in
reading time only.

**True.** Length predicts churn. Across 106 records in two other projects the
correlation was 0.704 and 0.715, derived independently. Records of 4000 words
or more were edited 3.7 times as often as records under 1200 words.

**Evidence.** The record scope research.[^6]

**Follows.** A long record holds material that changes. Length is a signal to
look for that material, not a sign of care.

### FND-034 — A claim title churns less than a topic title

**Believed.** A record title names its subject.

**True.** A title that states the decision predicts a more stable record than
a title that names a topic. In one corpus the churn was 2.13 against 4.50.
Both reference projects drifted from topic titles to claim titles on their
own, from 3 to 21 per cent and from 57 to 82 per cent.

**Evidence.** The record scope research.[^6]

**Follows.** Title a record with the claim it makes. Five of the six current
drafts carry topic titles and are queued for retitling.

### FND-036 — Mutation testing found ten gaps that a passing suite did not

**Believed.** A test suite that passes, with the gates green, covers the code
it tests.

**True.** The first mutation run surfaced ten real gaps: untested entity
accessors, untested fixed-point conversions, untested saturation limits, and
an invariant check that no test could falsify. The suite was green
throughout.

**Evidence.** The first `cargo mutants` run on the scaffolding: 94 caught, 10
unviable, 1 equivalent, after the gaps were closed.

**Follows.** A green suite is evidence that the tests pass, not that they
test. This matters most for the determinism tests, whose failure mode is
invisible. That is why they now have a proven failure mode of their own.

### FND-038 — The registers had no allocator, and two writers collided

**Believed.** The registry allocates record numbers, so the numbering problem
was solved. FND-028 recorded the lesson after three collisions in the research
phase.

**True.** The lesson was applied to the records and not to the registers. The
findings, decisions and blockers registers number their rows by reading the
last row, which is the same failure that FND-028 describes. Two writers
working at the same time on the same day both wrote FND-035 and DEC-013.

**Evidence.** The collision itself. One writer recorded the float ban lint
gap as FND-035; the other recorded a crossing time correction as FND-035. The
same happened for DEC-013.

**Follows.** A correct lesson applied to one system does not transfer to a
sibling system on its own. When a rule fixes a class of defect, look for
every place the class occurs, not only the place it was found. Each register
now carries an explicit next number, and a writer claims it before writing.

### FND-039 — The orientation held open questions that no register held

**Believed.** The blockers register lists every question that stops work. A
reader who wants the open list reads the register.

**True.** The project orientation carried its own list of three open
questions. One of them was BLK-002. The other two, the maximum faction count
and the world shape, had no row in any register. They were declared in one
place and tracked in none.

**Evidence.** Answering BLK-001 to BLK-006 closed one of the three questions
in the orientation and left the other two with nowhere to live. They are now
BLK-013 and BLK-014.

**Follows.** This is the redundant declaration shape again, in the form the
project has already met twice. A summary of a register is a second
declaration site, and it decays silently because nothing fails when it
disagrees. The orientation now points at the register instead of repeating
it. When a document summarises a register, make it name the register and stop
there.

### FND-040 — A record split left every citation in the code dangling

**Believed.** The crate cites the records that govern it, so a reader who
follows a citation reaches the reasoning.

**True.** Commit 4937cd2 deleted six omnibus drafts and re-derived the
registry from claims. The crate, the Python package, the gate scripts, the
justfile, the continuous integration workflow and four rules still cited the
deleted records by their old decision numbers. 81 citations were dangling.
Nothing failed, because the record check reads only the records, and a
comment is not compiled.

**Evidence.** Accepting the determinism core required a whole-tree search for
the old paths and the old decision numbers. Every source file in the crate
carried at least one. Worse than a dangling path, the numbers were reused:
`ADR-0002 D9` named a newtype rule when it was written and names the widening
accumulator now, so a reader who followed it reached a real record with the
wrong content.

**Follows.** This is the record that no longer describes the code, in its
cheapest form. A number that is reused is more dangerous than a path that is
gone, because the reader gets an answer. When a record is split or renumbered,
the sweep is part of the same change, and the search command belongs in the
commit body. A check that reads citations in source files, not only in
records, would have caught this. It does not exist yet.

### FND-041 — The orientation said the project had no code

**Believed.** The project is in design and no code exists. The orientation,
the testing rule, the recurring-defect rule and the budgets register all said
so.

**True.** The foundation crate, the Python bindings, the gates and the two
determinism tests exist and pass.

**Evidence.** Four documents carried the claim while `cargo test --workspace`
was green.

**Follows.** The redundant declaration shape again. Four documents declared
one fact about the project state, and no check compared any of them against
the tree. A status sentence that a reader can check against the tree in one
command is worth writing; one that repeats a fact four times is not.

### FND-042 — The tile index was designed against an unanswered question

**Believed.** Tiles are indexed by odd-r offset, not by raw axial. Registry
row 0017 stated that claim, and row 0016 was written above it.

**True.** The world is a rhombus, so a tile address is a raw axial pair and a
tile access converts no coordinate. Row 0017 now states the opposite of what
it stated before.

**Evidence.** BLK-014 held the world shape and named the offset conversion as
the cost that the shape decides. The row was written anyway, under the
rectangular assumption, while the blocker that governed it stayed open.

**Follows.** The scope rule already forbids a record from holding a value
that an unanswered question governs, and requires the value to be expressed
parametrically with the blocker cited.[^7] A registry row is not a record,
and the rule did not reach it. It should: a row states a claim, and a reader
takes a claim from the registry as the project's position. When a blocker
governs a claim, the row says so, or the row waits. No record was written
against the old row, so this cost one table edit rather than a supersession.

**A second instance, found by audit.** ADR-0056 D4 said the capacity values
depend on the tile scale, "which is an open blocker". BLK-001 and BLK-009
were both resolved before the audit ran, so the record stated a live question
that the project had already settled. The correction points at the scale
constants table, which is where the values now live.

The shape is not a stale value. It is a stale *reference to a blocker*, which
survives longer because it looks like the careful thing to do. Correct
parametric writing creates a claim that must be revisited when the blocker
closes, and nothing revisits it. **When a blocker closes, search the records
for its number.** The command is in the commit that closed it.

### FND-043 — A value type that cannot hold zero can lose a real value

**Believed.** The entity identity packs a generation above a slot index into
a value that is never zero, so an absent identity costs no extra space. The
niche is free.

**True.** The niche is not free. It removes one bit pattern from the value
space, and that pattern names a real entity: slot zero at generation zero.
The first entity the engine ever allocates takes slot zero, and a generation
that starts at zero leaves that entity without a representable identity.

**Evidence.** `Entity::new(0, 0)` returns nothing, because the packed value is
zero and the type cannot hold zero. A record review found it while reading
the identity record against the value type. The project had held the type for
some time and had a test for the refusal, which asserted the refusal was
correct rather than asking which value it refused.

**Follows.** A niche optimisation removes a value from the space. Say which
value, and prove that no real thing needs it. Here the identity record now
starts every generation at one, which vacates the pattern honestly.

The test shape matters more than the fix. Every test that allocates a second
entity before checking anything would pass. **When a type refuses a value,
write the test that asks whether the refused value was needed**, not only the
test that confirms the refusal happens.

### FND-044 — The saved property seeds were never read or written

**Believed.** A property test that fails saves the seed of the failing case
to a regression file. The file is checked in, so the case that caught a
defect runs first on every later run. A commit message asserted this.

**True.** Nothing was saved and nothing was replayed. The default persistence
finds the source root by walking up from the test file looking for a library
or binary root. An integration test has neither above it, so persistence
silently disabled itself and the files were inert. They had been committed,
read as evidence of a working practice, and cited in a commit message.

**Evidence.** Every failing run printed a line saying persistence was set but
found no root, and the message was lost in the test output. A code review
counted four of them in one run. Deleting a regression file and re-running a
deliberately broken build wrote nothing back until the persistence path was
named explicitly.

**Follows.** The file existed, so the practice looked healthy. This is the
inert capability shape in a form that is worse than usual, because the
artefact is present and has content: the seeds were real seeds, written by
hand or by an earlier run under different conditions, and they simply never
ran.

**When a tool reports that it disabled itself, that report is a failure.** It
appeared on standard output among passing tests, where nothing reads it. The
lesson is not to read more output. It is that a claim about the test suite
needs the same proof as a claim about the engine: delete the artefact, cause
the failure, and check that the artefact came back.

### FND-045 — A record was accepted while its review was still running

**Believed.** The review ceremony ran, its findings were in hand, and the
records could be accepted.

**True.** Four records were accepted on a first review while a second review
was still reading them. The second review found a claim in one record that
another record falsifies, and named it as blocking acceptance. By then the
record was accepted.

**Evidence.** ADR-0018 rejected an offset array over every tile on the
grounds that a structure growing with the tile count is unaffordable.
ADR-0056 D4 assumes exactly such an array exists and derives a capacity bound
from it. Accepting ADR-0018 would have bound the project to a reason that
another record falsifies. Three further records had outstanding amendments of
their own, including one record that called the same leak bounded in a
decision and unbounded in a consequence.

Nothing was pushed, so the acceptances were reverted, the amendments applied,
and the records accepted once with both reviews read.

**Follows.** The rule that was missing is simple: **prefer not to accept
while a review is in flight.** A verdict that has not arrived is not a
verdict, and the absence of an objection is not the absence of a defect.

The remedy chosen at the time was wrong, and the correction matters more than
the original finding. The records were reverted to `Draft`, amended, and
accepted a second time. That was unnecessary. The freeze on an accepted
record protects the things built on it, and nothing had been built on these:
no other record cited the amended claims, no code implemented them, and the
acceptance was an hour old. The registry now states a retcon window for
exactly this case, and states that a draft may simply be edited.[^8]

A process that punishes a cheap correction produces expensive corrections, or
none.

The second review also disagreed with the first about ADR-0018, which is the
argument for having had two. A single reviewer that agrees with the author is
indistinguishable from no reviewer.

### FND-046 — A sweep verified by the local gate misses what the local gate never runs

**Believed.** A rename is complete when the whole tree is searched and the
check command runs green. The commit that changed the world constructor from
a tile count to a width and a height searched the source tree and passed
`just check`.

**True.** The change was incomplete. Continuous integration failed on a smoke
test written inline in the workflow, which still called the old argument. The
local gate never ran that line, so it could not have caught it. The value
type's own text output also still named the removed argument.

**Evidence.** The workflow failed with a type error naming the removed
keyword, on a pipeline that a green local run had preceded.

**Follows.** Two things, and the second is the useful one.

A whole-tree search must cover the whole tree. A search over the source
directories is not a search over the tree, and a workflow, a build manifest
and a justfile are call sites.

**The deeper cause is that the smoke test lived inline in the workflow.** It
was a second declaration site for the public interface, in a file nobody
greps when renaming an argument, and no local command ran it. It is now a
script that both the workflow and the local gate run, so the interface has
one usage site and the local gate covers it. **When a check exists only in
continuous integration, the local gate is not the gate.**

### FND-047 — A record was written for a subsystem nobody had built

**Believed.** The research phase produced records for the subsystems it
studied. A record that states a design well is worth keeping until the
subsystem arrives.

**True.** `ADR-0057` specified a portal graph, a flow tile cache keyed on a
chunk and an exit, and a coarse biasing field, in five numbered decisions,
before any path-finding existed and before any product record asked for a
long path. It was retired rather than accepted.

**Evidence.** The record check reported it as cited by nothing for the whole
life of the project. That note was the only standing one, and it stopped the
moment the record went. No record, no source file and no registry row named
it. The three citations that existed were in the backlog item that scheduled
its own audit.

**Follows.** The scope rule's first condition asks whether a contributor could
reasonably choose otherwise, and it assumes a choice was made. Here nothing
had chosen anything: there was no path-finding to constrain, so the record
preserved a design rather than a decision. **A record written before its
subsystem exists cannot pass the first condition, because the decision it
claims to hold has not been taken.**

The research is not lost. Report 10 holds the reasoning, and a future record
starts from the report with a fresh number.

Retiring cost one row and one backlog edit. Accepting it would have bound the
first person to write a path-finder to a design nobody had tested against a
need nobody had stated.

### FND-048 — A determinism test cannot see a broken invariant

**Believed.** The two determinism tests are the project's strongest guard.
A run that repeats byte for byte at every thread count, and matches a stored
hash, is a run the project can trust.

**True.** They guard one property and only one. A defect that is itself
deterministic passes both, because the wrong answer is the same wrong answer
on every thread and every run.

**Evidence.** An audit of the movement record found that its admission rule
does not hold the capacity invariant the rule exists to hold. Departures were
counted from intents rather than from admitted moves, so a unit rejected at
its own target still released room behind it, and a tile could end a tick
above its capacity. The failure is a pure function of the intent set, so the
thread-count test and the golden state test would both have passed.

The defect was in a draft record and not yet in code, which is the only
reason this cost a paragraph rather than a supersession.

**Follows.** An invariant needs a test that asserts the invariant. This
sounds obvious and was not done: the movement work was planned with
thread-count equivalence and a golden hash as its determinism coverage, and
both would have shipped the defect.

**A property that must hold after every tick belongs in a test that checks it
after every tick.** For admission that is: no tile holds more units than its
capacity allows. The testing rule already says a determinism test cannot tell
correct from consistently wrong; this is the same lesson reaching an
invariant rather than a keyed draw.

### FND-049 — The cost of a step is not where the project assumed

**Believed.** The tile system is the large cost of a step. It touches every
tile of a 16.7 million tile world, and the units are far fewer, so the tiles
dominate.

**True.** The tiles are cheap and the units are expensive. On the measured
world a tile costs about 17 nanoseconds a tick and a soldier costs about
0.93 microseconds. A soldier costs about fifty times a tile. More than half
of what a soldier costs is not the movement at all: it is the rebuild of the
derived unit-to-tile bridge.

**Evidence.** An example runs worlds that differ in one thing at a time. On a
development machine, release profile, 12 threads, 40 ticks each, a world of
281600 tiles with no soldiers cost 6904 microseconds a step, and a world of
2816 tiles with no soldiers cost 2083. The same 281600 tile world cost 31954
with 22000 soldiers and 13593 with 2200. The rebuild is public and was timed
alone: 14497 microseconds at 22000 soldiers and 3830 at 2200, against zero
with no soldiers.

Two further facts came from the same run. A fixed cost near 2 milliseconds a
step does not depend on the world size. The dense world cost more than the
sparse world at the same soldier count, 38168 against 31954, so density makes
movement worse.

**Follows.** Three things.

**A derived cost figure can be wrong about which term dominates, not only
about its size.** Every cost figure in this project is derived, and the
project treated that as a question of accuracy. It is also a question of
shape. A derivation that names the wrong dominant term sends the optimisation
work to the wrong subsystem.

**A cost that grows with the units belongs to the derived structures, not
only to the systems.** The bridge is rebuilt at the barrier and nothing in the
design made its cost visible, because it is not a system and has no place in
the frame schedule that a reader would look at.

**The first measurement changed the plan the moment it was taken.** BLK-007
asks for measurement on the target platform and stays open. This is a
development machine, one run, with a cache line the target does not have. It
is enough to say which term dominates. It is not enough to say what any term
costs.

### FND-050 — An allocator that reads the tree cannot serve parallel work

**Believed.** The backlog needs no registry. The three directories are the
index, and a number is allocated by reading the highest one and adding one.
The decision records need a registry because their numbers carry status; the
backlog numbers carry nothing.

**True.** The rule is not about status. It is about who reads and who writes,
and when. A rule that derives the next value from the tree is safe only while
one worker acts at a time. Two workers that read the same tree derive the same
value, and each writes a file that looks correct.

**Evidence.** Two agents worked at the same time in separate worktrees. Both
read the highest backlog number, 0031. Both added one. Both delivered a pull
request holding items 0032, 0033 and 0034, for six different pieces of work.
Neither agent did anything wrong: each followed the written rule exactly.

The decision record numbers and the product record numbers did not collide in
the same run, because they were allocated centrally before the work was
dispatched. The backlog was not, and the reason it was not is that its guide
says an index is unnecessary.

**Follows.** Three things.

**An allocator that reads the tree is a redundant declaration site with the
timing hidden.** The recurring defect list already holds the shape: one value
declared in more than one place, with nothing that fails when the copies
disagree. Here the second declaration is the second reader, and the window is
the time between the read and the write.

**A registry was not the fix.** Adding one would put the numbers in two places
and create the shape this project keeps finding. The fix is a check that fails
when one number names more than one item, and a rule that the person who
dispatches parallel work allocates the numbers first.

**The rule that fails is the rule nobody suspected.** The project guarded the
record numbers because a collision there had happened three times. It left the
backlog unguarded because a collision there had never happened. Neither fact
was about the mechanism. **Guard the derivation, not the place that has been
burned.**

### FND-051 — A fixture chosen for realism can hide the defect it should show

**Believed.** A test that uses the same world the demonstration binary uses
is a realistic test. Realism makes a test better, because the test then
exercises what a person will actually run.

**True.** Realism and coverage are different properties, and they can oppose
each other. A uniform fixture produces a uniform result, and a defect that
only appears at an extreme of the distribution never gets its input.

**Evidence.** The head-up display clipped a row of the panel to eighteen
characters, so a value of nineteen characters printed as a word fragment.
The first test written for it used the demonstration world's placement
stride, 9973, which puts a unit in nearly every block. The count of skipped
blocks therefore stayed at zero, the value fit in exactly eighteen
characters, and **the test passed against the restored defect**. A stride of
37 clusters the units, drives both counts to two digits, and the test then
fails as it should.

The same session produced a second instance in a different subsystem. Every
existing test of the viewer placed its camera at the origin, so no test could
observe a defect in the block range that begins a row.

**Follows.** Three things.

**Ask what distribution the fixture produces, not whether it looks like the
real thing.** A defect lives at an extreme. A fixture that models the typical
case supplies no extreme, and the test then measures the fixture rather than
the code.

**A test that passes against the restored defect is the only proof that
matters.** Both instances here were found by putting the defect back and
watching the test stay green. A test nobody has seen fail is a test nobody
has checked.

**This is not the weak-test shape.** The testing rule already says a
determinism test cannot tell correct from consistently wrong, and that a
determinism test must be able to fail. Those are properties of the assertion.
This is a property of the input. An assertion strong enough to catch the
defect still catches nothing when the data never produces it.

### FND-052 — A register was restored from a copy, and an entry left silently

**Believed.** A register is safe under parallel work as long as two writers do
not edit the same rows. The merge is clean, so the content is correct.

**True.** A clean merge says the two sides did not conflict. It says nothing
about whether one side carried the whole file. A register restored from a copy
of an older base loses every entry added between that base and now, and the
loss looks exactly like a file that was never changed.

**Evidence.** A reviewing agent ran a checkout in the shared working tree and
moved a session off its branch mid-edit. Recovering meant copying
`docs/FINDINGS.md` back from a saved copy, and that copy came from the other
branch, whose base predated FND-049. The restored file therefore held a
correct new entry and no FND-049. It merged cleanly, because the two sides
never touched the same lines, and the entry was gone from the register for
three merges.

The register states the next free number in its own text, and that pointer
still read FND-049 while three entries above it existed. Neither the loss nor
the stale pointer failed anything.

**Follows.** Three things.

**A register carries its own allocator, and nothing checked it.** The findings,
blockers and decisions registers each state a next number. That is a second
declaration site for a value the rows already carry, which is the shape this
project keeps finding. A check now fails when a number names two entries, and
when the stated next number is not one above the highest.

**A whole-file restore is a delete of everything it does not contain.** Copying
a file back is not the same as restoring a change. Prefer the version control
history to a saved copy, and name the commit the content came from.

**A shared working tree is not safe for a reviewer.** The agent that caused
this was reviewing, not writing, and it changed no tracked file. Running a
checkout was enough. A reviewing agent gets its own worktree.

## References

[^1]: Findings register, FND-038, in this document.
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: Movement timing note. `docs/research/movement-timing.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Findings register, FND-004, in this document.
[^6]: Record scope research. `docs/research/adr-scope-findings.md`
[^7]: Decision Record Scope, section 4.5. `.claude/rules/adr-scope.md`
[^8]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`
