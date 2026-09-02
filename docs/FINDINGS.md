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

**Next number: FND-143**

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

### FND-085 — A rule that bans a measurement can leave a real cost unowned

**Believed.** That every cost figure in this project belongs to the target
platform, and that no measurement exists there, was a complete rule about
cost. It reads as complete. It is stated as a general fact about figures, and
two rules enforce it.

**True.** The rule governs one quantity: how the engine performs at the target
scale. A second quantity exists, and the rule is silent about it. How long a
contributor waits for the gate suite is a property of the development loop,
and the machine that runs it owns that. The silence read as a prohibition, so
nothing owned the cost.

**Evidence.** The golden state hash test grew as each subsystem entered the
state hash, four in one session, until it dominated a debug run of the suite.
No gate, no register and no budget noticed. The rule that would have caught an
equivalent regression at the target does not apply to a development machine,
and no other rule did.

**Follows.** Two things.

The project now keeps two registers with different standing. A figure from one
is never evidence about the other. The separation is a file boundary rather
than a note in a shared file, so a row cannot drift across it. The blocker
about the target stays open, because it is about the target.

**The general shape is the part worth keeping.** A rule that correctly refuses
a class of figure can leave a legitimate quantity with no owner. When the
project bans a measurement, ask what the ban also silences.


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

### FND-081 — Two accepted records disagreed on whether a per-tile count exists

**Believed.** The accepted records agreed about how admission reads the
occupancy of a tile.

**True.** They did not. ADR-0018 rejects an offset array, and it rests part of
that rejection on the claim that a per-tile array of counts already exists,
because admission needs the occupancy of a target and its departure count in
one tick. ADR-0056 D3 states that admission reads occupancy from the derived
structure and carries no per-tile array of its own. Both records are accepted.
Both are cited by code.

**Evidence.** An agent found the pair while it read ADR-0056 to write a
superseding record. Neither record cites the other on this point, so nothing
brought the two claims together. The check suite passed throughout, because a
record is prose.

**The code settles which claim is true, and it is not the one the first
attempt assumed.** Admission builds a working set of segments over the tiles
that some intent named, fills each from the derived structure, and discards
the set at the end of the tick. That is what ADR-0056 D3 describes. No array
over every tile exists, and none ever did.

**Follows.** Three things.

**ADR-0018's stated reason is false, and it was never load-bearing.** The
first answer to DEC-020 would have made it true by adding the array. The owner
reversed that answer, so nothing adds one. ADR-0074 now records that no dense
count exists, and it is the document a reader lands on when they meet the two
claims. ADR-0018's rejection of the offset array stands on its other argument,
that such an array must be exact everywhere before any query is correct, so
its rebuild follows the tile count rather than the work.

ADR-0018 is not superseded for this. It has four dependent records and eight
citing source files, and the false sentence changes no decision it makes.
Superseding a record over an aside is disproportionate, and a reader who meets
the disagreement is served by the record that resolves it.

A record that rejects an option on the strength of another record's mechanism
must cite that record. ADR-0018 named a mechanism it did not own, and the
mechanism did not exist.

This is the same pair of records as FND-045, which found them disagreeing on
whether a per-tile structure is affordable. One pair of records has now
produced two findings. Read both before either record is amended again.


### FND-086 — A product record stated a cost the engine never met

**Believed.** The product record for a world worth looking at states that
building a world must not cost a pass over every tile before the first frame.
The record reached `Shaped`, and nobody had checked the statement against the
code.

**True.** The engine makes that pass. Building a world loops once for each
tile, draws a value, and fills two vectors sized to the tile count. At the
target scale that is a whole-world pass and two proportional allocations
before anything is drawn.

**Evidence.** An agent checked each statement of the record against the crate
while it carried out the owner's ruling that every product record moves past
`Shaped`. The columns that make the pass belong to the tile stub, not to the
terrain the record introduced, so the record is false of the code by
inheritance rather than by its own subject.

**Follows.** Two things.

**A checkable statement is not checked until somebody runs it.** The shaping
gate asks for statements a reader can check. It does not ask anyone to check
them, and the record passed every gate the project has while stating something
untrue. The promotion review is where that gets caught, so a record must not
reach `Shipped` on the strength of its citing items having closed.

The record stays `Accepted` and an item tracks the repair. Accepting a need
the engine does not meet is correct. Claiming the need is met would be the
defect.

### FND-087 — A product record can carry a structure without naming one

**Believed.** The rule that a product record states a need and never states a
structure is enforced by a check, and the check looks for a citation to a
decision record.

**True.** A record can fix a structure without citing anything. The record for
parents and children said that units who are related and who live together
form a household. That sentence decides that a household is a kinship
grouping. The project has since decided the opposite: a household is every
unit that carries one dwelling slot, and two strangers under one roof are one
household.

**Evidence.** An agent found the sentence while it checked the record against
the newly closed decision. Every check passed before and after, because the
record cites no decision record and names no data structure. The structural
claim sat in ordinary prose about people.

**Follows.** Two things.

**Read a product record for the choices its wording forecloses, not only for
the structures it names.** A need stated in terms of who and what is still
capable of deciding how, and the check cannot see it.

A structural decision that contradicts a product record must send someone back
to the record. This one did, and the record was corrected in the same pass.
Had the decision landed without that reading, the two documents would have
disagreed quietly, which is the shape that costs every future decision made
from either.

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

**Later:** the row names the tier and means one structure. The pass is the
personality influence pass and the structure is a separate 64-byte trait
record. The descent and succession pass reads two or three columns, so it wants
struct-of-arrays, and the character arena is correct as it stands. Read this row
as scoped to the trait record.[^27]

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

### FND-137 — The event log crosses to Python as bytes with no layout

**Believed:** the bindings publish the event log to Python, so a control plane
reader can see what a step changed. The method returns the whole log as bytes,
and the crate documents it as the log the determinism test compares.[^F137A]

**True:** it publishes a buffer, not events. The field order, the field widths
and the padding of an event live in the Rust source and nowhere else.[^F137B] A
reader in Python must repeat that layout to get a field out of the buffer. The
copy and the original then hold one fact in two places, and nothing fails when
they disagree. The recurring defect rule names this pair as a place the shape
will recur in this project.[^F137C]

**Evidence:** the agent-facing protocol server was built over the existing
bindings. Every other tool calls one method and returns what the engine said.
The event log tool cannot, so it returns the bytes and a digest of them. An
agent can prove that two runs emitted the same log and cannot see which tile
changed.

**Follows:** the bytes are enough to compare two runs and not enough to read
one. Give Python the events from the place that declares them, by a column for
each field, by a derived description of the layout, or by a query the engine
answers. Do not add a format string to Python. A backlog item holds the
choice.[^F137D]

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

### FND-082 — A register cited a record that did not support its claim

**Believed.** The decisions register stated that the engine clears a lost
site's units by scanning every unit, and it cited a record for that claim.

**True.** The cited decision says that the location table is a dense array
indexed by the slot, and that it is never a hash map. It says nothing about a
scan from a site to its units. The claim is right and the citation does not
reach it. The register also wrote the clear in the present tense. No
settlement destruction path exists in the engine, so the clear it describes is
not code. The only site-to-unit traversal in the tree is an invariant check.

**Evidence.** An agent read the record while it confirmed that DEC-036 changed
nothing. A second instance sits nearby: ADR-0067 cites a product requirement
record in a footnote, and DEC-012 decided that a decision record cites no
product record. That citation predates the decision.

**Follows.** Two things.

A footnote is a claim about a source. A reader who follows it and finds
nothing loses more than the citation, because the register spent its
authority. Check that the cited decision states the thing, not that the cited
record covers the topic.

**Write the tense the tree supports.** A register that describes intended code
in the present tense reads as a description of the engine. That is the shape
the scope rule names as recording an intent as a fact.


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

**Second instance, in the same file.** The status section of the same
orientation file said "Eight records are binding" and named them. The registry
held thirteen accepted records at the time it was noticed, and fifteen by the
time it was repaired. The sentence sat two sentences before the file's own
statement that the registry holds the status of every record and is the only
place that does, so the file contradicted itself.

Nothing failed. The count decays every time a record is accepted, which the
project does on purpose and often, and no check reads prose for a number. The
status section now points at the registry and says plainly that a count there
is not the list.

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
after every tick.** The testing rule already says a determinism test cannot tell
correct from consistently wrong; this is the same lesson reaching an
invariant rather than a keyed draw.

**Corrected on 1 September 2026.** This entry named the property for admission
as "no tile holds more units than its capacity allows". That is false of the
engine, and a contributor who wrote a test in that form would watch it fail on a
legitimate world. A spawn places a unit without reading the capacity, admission
is the only reader of it, and nothing establishes the strong form at rest. The
property for admission is that **no tile gains a unit beyond its capacity**. A
tile that a caller over-filled may stay above its capacity, and it may never
rise. The record that decides this states the reasoning.[^56]

The lesson of this entry is unchanged. Only the sentence naming the invariant
was wrong, and it was not visible as wrong when it was written, because the
question of whether a spawn refuses was an open decision at the time.

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

### FND-053 — A record stated an algorithm that the code never had

**Believed.** The unit-to-tile bridge sorts the occupying units with a radix
sort on the integer key. The record for the bridge says so in the text of one
of its decisions, and that record is accepted.[^9]

**True.** The code ran a parallel comparison sort. It divided the keys into
chunks, ordered each chunk with a comparison sort, and merged the runs. No
radix pass existed anywhere in the crate. The record had said radix since the
day it was accepted, and nothing had ever implemented it.

**Evidence.** A measurement of the rebuild found a cost for each unit that
matched a comparison sort, and a fixed cost that matched a thread spawn for
each chunk. A reading of the sort module then found the comparison sort.

The record was not wrong about the constraint. It was wrong about the
artefact. A reader who took the sentence as a description of the code would
have concluded that the rebuild was already as cheap as the design allowed,
and would not have measured. That is what the sentence cost: it did not cause
a defect, it prevented a measurement.

**Follows.** Three things.

**This is the first local instance of the record-without-code shape, and it
runs in the direction nobody guarded.** The recurring defect list already
holds two neighbours, and both are imported priors with no local instance.
One is inert code that nothing invokes. The other is a record that claimed a
list of telemetry keys described real behaviour, when several of those keys
had no write site. Both describe code and a record disagreeing. **This one is
the inverse: not code without a record, but a record without the code.** A
reviewer who checks that every capability has a record will not find it,
because the record is the part that exists.

**A record's statement of an algorithm is not evidence that the algorithm
exists.** The corpus holds other records that name an algorithm. Nothing in
this project has ever checked one of those sentences against a source file,
and the check script cannot: it reads prose. Treat every such sentence as a
claim to verify, not as a fact to build on. When a record names an algorithm
and you are about to act on the consequence, open the code.

**Write the constraint the code must satisfy, or write what the code does.**
The scope rule already forbids a record that states an intent as a fact.[^10]
The failure is cheap to make, because one sentence can be true of the design
and false of the code at the same time, and the author sees only the design.
A record that names an algorithm the project has not written must say plainly
that nothing implements it yet.

### FND-054 — A test world smaller than the lattice spacing holds one terrain

**Believed.** A small world is a scaled-down large world. A fixture of twelve
tiles by twelve carries the same mix of ground as a fixture of ninety-six by
ninety-six, so a small fixture is a cheap way to test what a large one tests.

**True.** The ground comes from a lattice, and the coarsest octave of that
lattice spans sixty-four tiles. A world narrower than that spacing sits inside
one lattice cell. Every tile of it then falls on the same side of the water
threshold, so the world is all water or all dry, and which one is a property
of the seed.

**Evidence.** Three instances, all in one change, the change that made the
ground refuse a unit.

A property test over arbitrary seeds peopled a world of twelve by twelve. One
seed put water on every one of its tiles, so the fixture spawned nobody and
two empty runs compared equal.

The golden state hash suite held one populated scenario, of twenty-four tiles
by twenty-four. It held no water, so no soldier in it ever met water. The
suite passed unchanged against the new rule and against the rule removed
again. A scenario of ninety-six by ninety-six fails in the second case and
passes in the first.

The generator makes no one-tile island, because the field is coherent. A test
written for a soldier whose every neighbour is water found no such tile in a
world of nine thousand.

**Follows.** Three things.

**State the extent a fixture needs, against the lattice and not against
taste.** A fixture that must hold two kinds of ground is wider than the
coarsest lattice spacing. A fixture narrower than that tests one kind of
ground and must say which.

**A property test over seeds must reject a world it cannot use.** A run with
no soldier in it compares two empty results and passes. That is the
uniform-input shape, and it is already recorded.[^11]

**The extent belongs in the fixture, not in the record.** The spacing is a
constant of the generator. A record that quoted it would hold a value that a
change to the generator moves.[^12]

### FND-055 — A citation carried a status, and the status went stale

**Believed.** The registry is the only place that holds the status of a
record, so a citation elsewhere in the tree cannot disagree with it.

**True.** A citation carries the status whenever its prose says so. Five
citations described ADR-0056 as a draft record. The registry had said
`Accepted` since the review that accepted it. The claim was false and nothing
failed, because a citation in a comment is not compiled and no check read what
a citation says about a record.

The failure inverts the record. A reader who follows the citation reaches an
accepted record, and the prose beside the citation tells them that nothing may
cite it as binding.

**Evidence.** The review that accepted the record is in the tree, so the sweep
that should have followed it never ran. The citations sat in the engine, in
three test files, and were copied into new work twice more before a check
found them.

**Follows.** Two things.

**A check reads the claim, and it derives the truth from the tree.** The
citation check now fails when a footnote definition names a path under the
accepted directory and calls that record a draft. The path is the derivation,
so no second listing exists to drift. The opposite drift already fails: a
record that moves out of the draft directory breaks every citation of its old
path.

**The check reads a footnote definition and nothing else.** A review states
what a record's status was on the day it was reviewed, and that statement is
true and must stay. The line that carries it is a table row, not a footnote,
so the shape of the line is what separates a live claim from a historical
one.

### FND-056 — A constraint can be stated over a subject wider than itself

**Believed.** A decision record states a constraint, and a reviewer checks the
code against it. If the constraint is right, the record is right.

**True.** A constraint has a subject as well as a claim, and the subject can be
wrong while the claim is right. A subject wider than the constraint forbids
things the constraint was never about, and the record then contradicts itself
if another of its decisions requires one of them.

**Evidence.** ADR-0067 D1 said the viewer holds a shared reference to the
world, calls no method that takes a mutable reference, never spawns or moves
an entity, and never advances a tick. D5 said the viewer is a crate. D4 said
one loop steps the engine and then draws, and that loop is in that crate. The
demonstration binary therefore did all three things D1 forbids, and the
record's own consequences named that binary and described it stepping.

The claim was never in doubt: D1's rationale is that a drawing must not put a
person's choice of what to look at into simulated state. Stepping the engine
is not that. The subject was the whole crate when it should have been the path
from the world to the picture.

Nothing caught it for the life of the draft. The code was right, every test
passed, and a reader who knew the intent read past the words.

**Follows.** Two things.

**Read a constraint's subject as carefully as its claim.** Ask what the words
forbid, not what the author meant to forbid, and then ask whether the same
record requires any of it. Two decisions in one record that contradict each
other are cheaper to find than two records that do, and nobody looks.

**A constraint a reviewer must read past is not a constraint.** The test is
whether a reviewer asked to enforce the words alone would reach the right
answer. If enforcing the record as written would refuse something the project
requires, the record needs an amendment even though the code needs none.

### FND-057 — A deferral can name a record that refuses the work

**Believed.** A product record bounds itself by deferring work to another
record. The deferral is safe, because the other record is named and the reader
can follow it.

**True.** A deferral names a destination. It does not check that the
destination accepts the delivery. Two records can each push one need onto the
other, and the need then belongs to nobody. Nothing fails, because both
records read as complete and each one points somewhere real.

**Evidence.** Two instances, found while shaping six further product records
against the eleven that existed.

PRD-0011 states that a unit is assigned a job and that choosing belongs with
unit behaviour. PRD-0009 is unit behaviour, and it excludes a group decision,
excludes any goal that outlives a tick, and states that a unit chooses for
itself. Assigning a job is a decision made for a place and it persists for
many ticks, so PRD-0009 refuses exactly what PRD-0011 sends it. Neither record
owned the assignment.

PRD-0007 states that it consumes nothing and that consumption arrives with
unit lives. PRD-0011 is unit lives, and it carries consumption as one line of
its checklist and no section. The destination accepted the need in name and
shaped none of it.

**Follows.** Three things.

**Read the destination before you write a deferral.** A bound is only a bound
when somebody else holds what it excluded. Open the named record and find the
statement that accepts the work.

**A one-line mention is not ownership.** A need that appears in another
record's checklist and in none of its six gate answers has been acknowledged,
not shaped.

**This is the redundant declaration shape with no declaration at all.** The
recurring defect rule warns about one fact stored in two places with nothing
that fails when they disagree.[^13] A deferral pair is the inverse: one fact
stored in no place, with two records that each say it lives in the other.
Neither check can see it, because each record is well formed on its own.

### FND-058 — A registry dependency is not a build order

**Believed.** The `Depends on` column of the record registry gives the order in
which the records should be written. A row that depends on another row waits
for it. The registry's own writing-order section reads that way: write the
core, then the cross-cutting models, then the subsystems.[^14]

**True.** The column states which record a record may cite. It says nothing
about whether the depended-on record needs to exist yet, or whether its claim
needs to exist at all. Reading it as a build order produces the failure the
project has already recorded once: a record written for a subsystem nobody had
built.[^15]

**Evidence.** Found while sequencing the seventeen product records into a build
plan.

Row 0062 states that production and upkeep are rates attached to a site, and it
depends on row 0055, an ordered modifier pipeline for an effective stat.
Production is the next thing the project needs. A modifier pipeline is not: one
source modifies a rate today, so the pipeline fails the first condition of the
scope test, because with one source there is no decision to preserve. Writing
0055 first would produce a record binding a mechanism nothing invokes.[^16]

The same shape appears at row 0058, which states that a field update is a flux
pair on an edge, and at row 0061, which states that trade solves a flow. Both
are prerequisites of nothing that exists, because no place in the world holds a
surplus until production and consumption run.

**Follows.** Three things.

**Read the column as a citation constraint, not as a schedule.** A record may
cite the record it depends on. It is not obliged to wait for it, and the
depended-on record may never be written.

**Apply the scope test to the depended-on row before writing it.** A reserved
row reserves a number and does not promise a record. The registry says so, and
the dependency column is where that sentence is easiest to forget.[^14]

**A record may state a dependency on a row that stays reserved.** Say in the
record that the depended-on claim does not exist yet and why. That is a truthful
statement about the project, and it is cheaper than either writing the record
early or removing the dependency.
### FND-059 — A completed item's outcome decays like any other document

**Believed.** An outcome section is history. It says what one item did at one
moment, so it stays true in the way a commit message stays true.

**True.** An outcome sits in the tree and reads in the present tense. It is a
document, and it decays. The commit message rule puts a count in the commit
because a commit is fixed to one change. An outcome is not fixed to one
change, because later work changes the thing it describes.

**Evidence.** The outcome of the demonstration binary item said the binary
builds a world of a stated extent with a stated soldier count. Later work
raised both by more than an order of magnitude. Nothing failed. The audit of
the product record found it by reading the binary and then the record.

A second instance sits in the review of the viewer boundary record, which
states the soldier count the binary spawned when the review ran.

**Follows.** Three things.

**Do not put a count in an outcome section.** Put it in the commit message,
which the item's history already reaches.

**Say what the work achieved, not what the code now holds.** An outcome that
names a behaviour survives a change to a constant. An outcome that names the
constant does not.

**This is the document rot shape, in a place the rule did not name.** The
recurring defect rule warns that a decision record holds no count.[^17] The
warning applies to every document that a later change can falsify, and an
outcome section is one.

### FND-060 — A comment can claim that one fact has one site while a second site exists

**Believed.** Tile passability is the tile capacity being zero, and nothing
else states it. The terrain module says so in its own words, and it says so
because two rules that can disagree would be one fact in two places.

**True.** A second site exists, and it is the site every caller uses. The
kind's passability test matches on the water kind by name. It does not read
the capacity. Four call sites in the engine read the passability test. None
reads the capacity to decide whether a unit may stand.

The two agree today, because water is the one kind with a capacity of zero. A
kind added with a capacity of zero and no water in its name would be passable
and would admit nobody. Nothing fails, and no test compares the two.

**Evidence.** Found while auditing the product record for the viewer. The
comment that denies the second site sits directly above the capacity table,
and the passability test sits directly above the comment.

**Follows.** Three things.

**A comment is not a check.** The recurring defect rule already says this: do
not add a comment that names the winner.[^13] This instance is stronger,
because the comment does not name a winner. It denies that a second site
exists at all, which reads as a check and is not one.

**Derive the second site from the first, or add a check that compares them.**
The passability test can return the capacity being greater than zero. That
removes the site rather than reconciling it.

**Local evidence now exists for the redundant declaration shape in code.** The
shape's only local instance was a numbering collision in a register. This one
is in the engine.

**Closed.** The second site is gone. The passability reader now returns the
capacity being greater than zero, and it matches no kind by name. The capacity
table is the one declaration, and its match is exhaustive, so the compiler
refuses a kind that states no capacity. A test asserts the two answers agree
over every kind. The proof that the test can fail restored the name match and
set the water capacity to the ordinary value; the test then reported that
water answered the two questions differently.[^54]

### FND-061 — A fixture assertion stated over the inputs cannot see the case

**Believed.** A fixture that must produce a contested case proves it by
asserting over its own inputs. Count the demand, count the supply, and assert
that the demand is larger.

**True.** The assertion has to be over the outcome. An input assertion needs a
model of the rule, and the test does not hold that model. When the model is
wrong the assertion is wrong, and it fails on a fixture that does produce the
case, or passes on one that does not.

**Evidence.** The gather fixture counted one unit of demand for each gatherer
and compared it against the stock of the deposits. The engine grants a whole
rate to each unit until the deposit is empty, and the rate is not visible to
the test. Eight gatherers on a deposit of nine therefore contended, because
the first two took the whole rate and the third took what was left. The
assertion said they did not, and it stopped a run of a fixture that was
correct.

Restating it over the outcome fixed it: after the first frame, the number of
grants must be below the number of gatherers. That statement needs no model of
the rate and it is exactly the case the resolve exists for.

**Follows.** Assert that the fixture produced the case, not that the inputs
should produce it. This extends the rule that a fixture must be checked rather
than assumed.[^11] The check is what the engine did, and the rule stays true
when the engine's constants change.

### FND-062 — A probe build perturbs every subsystem at once

**Believed.** Each probe test has a companion that holds everything else
fixed. The pair says that the perturbation changed the order and changed
nothing else, which is what makes the failure evidence about the order.

**True.** The perturbed build turns on every perturbation together. A
subsystem downstream of another perturbed subsystem therefore sees changed
inputs, so nothing about it is held fixed and no such companion exists.

**Evidence.** The gather resolve runs after movement, and movement admission
is perturbed in the same build. A companion test asserted that the total taken
was the same at one thread and at twelve. It failed, and correctly: the units
stood on different tiles, so they gathered from different deposits.

The movement companion holds, because nothing upstream of movement is
perturbed in a way that changes the population.

**Follows.** Write the companion only where nothing upstream of the subject is
perturbed. Where it cannot hold, say so in the file rather than leaving a
reader to wonder why one probe has a companion and the next does not.

### FND-068 — The world's call to an arena invariant check cannot fail today

**Believed.** The world's invariant check covers each entity arena, because it
calls the check of every arena and refuses the world when one of them fails.
Removing one of those calls would therefore break a test.

**True.** It breaks no test. Every failure state that an arena check detects is
unreachable from the public interface. The arena marks a slot live before it
mints the identity, it advances the generation before it queues the slot, and
it writes every column in one call. So a caller who drives the world can never
put an arena into a state its own check rejects.

**Evidence.** Found while building the character column set. The call to the
character arena check was removed from the world, and the whole suite stayed
green: the arena tests, the thread-count suite and the golden state hash all
passed. The arena's own failure states are reached only by the unit tests
inside the module, which write the columns directly.

The soldier arena and the settlement arena have the same property. Nothing in
the tree distinguishes the three.

**Follows.** Three things.

**Keep the call.** It is a guard against a future write path that edits the
columns outside the arena, and the batched structural path will be exactly
that. A guard for a path that does not exist yet is not the same as a
capability nobody invokes.

**Do not read a green suite as proof that the world-level call works.** The
unit tests prove the check works. Nothing proves the world reaches it. Say
which of the two a test proves.

**The test that would close this needs a write path that can corrupt an
arena.** Write it when that path exists, and not before, because a test that
reaches through a private field pins the implementation rather than the
behaviour.[^21]

### FND-063 — A refined item asked for a path that no record and no code held

**Believed.** A backlog item that adds an entity shape can require the founding
and the loss of that shape to go through the batched structural path. The
storage record states that a structural change is a move between column sets,
and that the batched tombstone and compact path applies to it.[^2] [^18]

**True.** No batched structural path exists. The record that would define it
holds a reserved registry row and no file, so the claim the item cites is a
number and not a decision.[^14] The one arena that exists edits its columns at
once, inside the call. An item cannot honour a path that nothing has written.

**Evidence.** The settlement arena was written against the item, and the item
asked for the batched path in its list of checkable statements. The soldier
arena, which the same storage record governs and which the project accepted,
spawns and despawns inside the call. Building the settlement arena on a
different path would have made the two shapes disagree, with no record that
says which is right.

**Follows.** Three things.

**A cited decision that has no file cannot gate an item.** Read the registry
status of every record an item names before the item is refined. A reserved row
reserves a number and does not promise a record.[^14] This is the same shape as
the registry finding, seen from the backlog rather than from the registry.[^19]

**The identity rule carries the weight the batched path was asked to carry.**
The generation advances when the arena frees the slot, so a destroyed entity
loses its identity at the moment it dies, whatever path the change took.[^20]
That is what the item needed, and it holds today.

**State the gap in the item, and open a row for the work.** The settlement
arena follows the soldier arena, and the backlog holds the item that moves both
to the batched path when the record exists.

### FND-064 — A settings struct with public fields prices every new parameter

**Believed.** A new parameter of the world belongs in the settings struct that
builds the world. The struct is the one place a caller states what a world is,
so a schedule period belongs there beside the extent and the seed.

**True.** The struct has public fields and no constructor, so every caller
builds it with a struct literal. Adding one field therefore breaks every
literal in the tree at once, in three crates and in the Python type stub. The
work that met this was told to keep its edits inside the core crate and to
leave the viewer alone. The settings struct made that impossible.

**Evidence.** The site rate work added a period and a phase to the settings
struct. The compiler then refused twenty-five files, including the viewer, the
Python binding and the type stub, none of which have anything to say about an
economy. The work moved the schedule to a default on the world and a setter
beside it, and the tree compiled unchanged.

**Follows.** Two things.

**A settings struct with public fields is an interface, and adding to it is a
breaking change.** Treat it as one. A parameter that only one subsystem reads
does not have to sit there.

**Say who must state a value.** The extent and the seed have no default, so a
caller must state them. A cadence has a recommended value, so a caller may
leave it. A value of the second kind belongs on the object with a default and a
setter, not in the constructor argument.

### FND-065 — A conservation check over a column must name the structural moments

**Believed.** A store column conserves when the sum over the column equals what
the rates put in, minus what the rates took out. The rate pass is the only
thing that writes a store, so the sum and the ledger must agree.

**True.** The rate pass is not the only thing that moves the sum. The arena
leaves the store of a destroyed settlement in the dead slot, and it clears the
store when a founding reuses that slot. The sum over the whole column therefore
falls at a founding, and the sum over the live slots falls at a loss. Neither
fall came from a rate, and a check that knew only the ledger reports a leak
that nothing leaked.

**Evidence.** The settlement arena clears a store at the founding and not at
the loss. That is correct: clearing at the founding is what stops a slot from
handing its holding to its successor. It means the store of a dead slot is a
residue and not a holding, and a conservation check has to say which of the two
it is reading.

**Follows.** State the structural moments, and adjust the account at each one.
The account of what the live stores hold moves at four places: a write from the
control plane, the loss of a settlement, the rate pass, and nowhere else. The
check then fails when a fifth place appears, which is what a check is for.

**A conservation check is not a determinism test, and it catches what no
determinism test can.** A rule that leaks the same amount on every run repeats
perfectly at every thread count.

### FND-066 — A constant stated a rule that a comparison already stated

**Believed.** The rule that spreads a holding needs a constant that adds to the
support of the faction which already holds a tile. Without it, the belief ran,
a tile that two factions support equally changes hands on every tick.

**True.** The comparison that admits a challenger already refuses an equal
claim, because it demands support strictly greater than the holder raises. The
constant and the comparison were two statements of one rule. The constant was
the second, and it changed nothing that any test could see.

**Evidence.** The constant was set to zero in the source, and the whole test
suite stayed green. That is the shape of a second declaration site: it reads
back correctly and reaches nothing.[^22] The constant was then removed, and the
comparison was left as the only statement of the rule. A test that gives one
tile to a holder and then puts an equal claim on it fails when the comparison
is loosened, so the rule now has one site and one test.

**Follows.** **Set a constant to a value that must change the answer, and run
the tests.** A constant that no test can see is either dead or duplicated. This
was found by the practice of putting a defect back and watching the tests stay
green, and it would not have been found by reading the code.[^23]

### FND-067 — A spread rule that visits the holding costs the area, not the edge

**Believed.** A rule that spreads a holding must visit every tile that anybody
holds, and the neighbours of each, because any of them might change hands.

**True.** A tile whose six neighbours are all held by its own holder cannot
change hands under the rule. Its holder draws support from all six neighbours
and from holding the tile, and no challenger can raise more than that. The rule
therefore visits the edge of a holding and not its area.

**Evidence.** The candidate list was changed to pass over such a tile, and the
golden state hash files did not move. An optimisation that changes no hash is
the strongest evidence available here that it changed no behaviour. The commit
holds the times measured before and after.

**Follows.** **The cost of the spread grows with the perimeter of a holding and
with the population, and not with the world.** That is the property the product
record asks for, and it is a consequence of the rule rather than of a limit
somebody imposed.[^24] A later rule that lets a claim reach further than one
tile loses this property, and it must state what replaces it.
### FND-074 — A rule that divides a set of one has nothing to divide

**Believed.** A cohort is one row for the units of one kind in one place, and
the kinds of a unit are content that does not exist yet. One kind was therefore
enough to build the draw, and the rule that splits a store between the cohorts
of one place could be written and tested against it.

**True.** With one kind, a place holds one cohort. The split then has one part,
the part is the whole, and no input reaches the remainder rule. The rule would
have shipped inert, and its test would have measured the fixture rather than the
rule.[^37]

**Evidence.** The exactness test was written first and could not be made to
fail. Every store that was short gave everything it held to the one cohort that
asked, whatever the split rule said. The remainder loop was then deleted, and
the whole suite stayed green.

**Follows.** **The cohort is keyed on the faction as well as the place.** That
is not a device for the test. Two factions in one place must not pool a draw
against a store that one of them holds, so the second key was already required
and the inert split is what exposed it. **Before you write a rule over a set,
ask what makes the set hold more than one member. If nothing does, the rule has
no reader.**

### FND-075 — A capped transfer must take what the split handed out

**Believed.** A draw takes the smaller of what a store holds and what the
cohorts asked for. That amount is the cap, so the store falls by the cap and
the split then divides the cap between the cohorts.

**True.** The cap and the sum of the shares are two statements of one quantity.
While the split is exact they agree, and a split that lost a unit would take
that unit out of the store and give it to nobody. The conservation check over
the world compares the store column against the account of it, and both fall by
the cap, so nothing fails.

**Evidence.** The remainder loop was deleted from the split. The store still
fell by the cap, the account still agreed with the column, and the conservation
test stayed green. Only the test that reads the shares saw the loss.

**Follows.** **Take from the source what the sinks received, and never what the
transfer meant to give.** The store now falls by the sum of the shares, which
makes the two copies one copy.[^38] A conservation check that reads only the
source cannot see a quantity that left the source and reached nobody.

### FND-079 — The event log publishes a stub as the owner of a tile

**Believed.** The holder of a tile is the owner of that tile. An accepted
record states it, the holding column carries it, and the viewer draws it.[^40]

**True.** The engine ships two tile-owner facts, and the one it publishes as
the owner is the one no rule ever writes. A second faction column is filled
once at construction from the tile index and the faction count. It never
changes, nothing writes it, and it covers open water as readily as open
ground. It is private and has no public reader, so it looks contained.

It escapes through the event log. The tile-changed event carries a faction
field, and that field is documented as the faction that owns the tile. The
event is public and it reaches the Python control plane, which has no other
view of who holds anything.

**Evidence.** The column is filled as the tile index modulo the faction count.
The event's own documentation calls the field the owner. A developer who
performs the comparison the product record asks for — a unit's faction against
the owner the event reports — gets a confident wrong answer, and nothing
fails. The comparison is type-correct and meaningless.[^41]

**Follows.** **A private value with a public description is not private.** The
doc comment is the interface. This is the second declaration site of shape 1,
made worse: the two sites do not hold the same kind of fact, so they cannot be
reconciled by choosing a winner. The stub goes, and the event carries the
holder with a stated encoding for nobody.[^42]

### FND-080 — A behavioural claim was defended by a test of constants

**Believed.** Terrain influences where a holding spreads, and a test covers
it. The test is named for the claim and it passes.

**True.** The test asserts that water is never held, and that the claim
threshold of plain is below hill and hill below mountain. The second assertion
reads three constants and exercises no behaviour. The test also computes the
held share for each terrain kind and then discards the result without
asserting on it.

The gradient it is named for is not defended.

**This entry first named the wrong experiment, and the correction is a
separate finding.**[^83] It said that flattening the claim threshold to the
same value for every passable kind leaves the test green. It does not. The
test reads the thresholds, so flattening them fails it on a comparison of two
constants, without running the rule at all. The experiment that shows the test
blind is to leave the thresholds ordered and stop the decision function from
reading them. The conclusion above stands; only this line was wrong.

**Evidence.** The behaviour is real and was observed over 40 ticks on a
development machine, by recording the terrain of every unheld tile adjacent to
a holding and whether the next tick took it. Plain converted every tile
offered, hill about a third, mountain a small fraction, water none. The
gradient exists in the system and no assertion holds it there.

**Follows.** **Name a test for the claim and it will be read as covering the
claim.** The testing rule already states that a determinism test cannot tell
correct from consistently wrong; this is the same failure in an ordinary
test.[^43] A test that reads a constant proves the constant was written down.
Only a counted outcome proves the rule acts on it. This is the third instance
in one session of one fact checked in a place that cannot see it go
wrong.[^44] [^45]

### FND-078 — A differential test cannot see a defect that moves both worlds alike

**Believed.** A test that compares a world holding ground against the same
world holding none proves that the holder layer draws the holder. The
difference between the two pictures is the holding, so a wrong holder would
change the difference.

**True.** The difference is blind to any defect that tints both worlds
identically. The holder layer was pointed at the rule of the stub tile faction
column, which derives a faction from the tile index and reads no holder at all.
That rule does not care whether a soldier stands anywhere, so it painted both
worlds the same way and the difference stayed exactly as it was.

**Evidence.** Two tests passed under the wrong column: the one asserting that a
tile nobody holds draws as it did before, and the one asserting that open water
never takes a holder colour. Four other tests failed, and all four read the
holder back rather than comparing two worlds.[^39]

**Follows.** **A differential assertion measures the difference, not the
source.** When the property under test is where a value came from, one side of
the comparison must read that value back. This is the same shape as FND-075,
where a conservation check that read only the source could not see a quantity
that left the source and reached nobody. Both are one fact checked in a place
that cannot see it go wrong.

### FND-072 — A layout finding named the tier when it meant one structure

**Believed.** The character tier wants array-of-structs, and the character
arena therefore has the wrong layout. The register states the correction as
"the character pass is a random graph gather", and readers take the character
pass to mean the descent and succession pass.[^25]

**True.** Three things are wrong with that reading.

The figure belongs to the vector report, not to the character report.[^26]
[^27] The character report recommends struct-of-arrays for the character row
and says in the same paragraph that the row size is an accounting figure and
not a claim about locality.[^27]

The pass is the personality influence pass, and the structure is a separate
64-byte trait record that holds twelve current values and twelve anchor
values. The vector report's own decision keeps the trait record in
array-of-structs and says nothing about the identity columns.[^26] The two
recommendations do not conflict. They cover different structures.

The descent and succession pass is not a twelve-column gather. The character
report lists its kernels: the eligibility filter is a map to a mask and a
compaction scan, the ranking is a map to a key tuple and a sort, the child
list rebuild is a counting sort, and a cadet split is a map over a contiguous
range.[^27] Every one of those is a column pass. The two operations that do
gather at random, the lowest common ancestor walk and the kinship recursion,
read two or three columns for each node, and the report already budgets both
as affordable.[^27]

**Evidence.** A whole-tree search for the phrase found it in the vector report
and in the merge notes, and never in the character report. The character arena
holds five columns today and holds no parent edge, so the pass the finding is
read to govern does not exist.

A gather benchmark on a development machine measured the crossover as a
function of the column count, not of the tier. Struct-of-arrays wins at one
and at two columns, the two layouts meet near three, and array-of-structs wins
above that. The figures are in the commit body. The machine is not the target,
so the measurement fixes the shape of the curve and not the position of the
crossover.[^28]

**Follows.** Three things.

**Scope a layout claim to the structure and to the pass, never to the tier.**
This is the same shape as the finding that a constraint can be stated over a
subject wider than itself.[^29]

**Read FND-022 as scoped to the trait record.** Its closing line is correct
and general: layout follows the access pattern. Its middle line names the
tier, and the tier was never the subject.

**Count the columns before you choose a layout.** The column count of the pass
decides the answer. The name of the tier does not.

### FND-070 — A restored defect must be affordable, or the proof cannot run

**Believed.** Putting a defect back and running the suite is always cheap. The
testing rule asks for it as a routine step, and the cost of the step is the
cost of one test run.[^33]

**True.** A defect in a cost constraint can make the suite unrunnable. The
proof then produces no answer at all, which reads the same as a proof nobody
ran.

**Evidence.** The founding chooses a place from a bounded sample. The first
attempt to restore the defect replaced the sample with a pass over every tile
of the world, which is what the record forbids in its strongest form. The
cost test builds four worlds, the largest of which holds over a million tiles,
and the survey deduplicates its candidates by a linear search. The restored
defect therefore ran a quadratic search over a million entries, in the debug
profile. It was killed after twenty minutes without reaching the first
assertion.

The second attempt made the sample size a multiple of the world width. That
is the same violation of the same claim, in its smallest form, and the test
binary ran in the ordinary time.

**Follows.** Two things.

**Restore the smallest change that violates the claim, not the largest.** The
claim is that the sample size is not a function of the world extent. A
multiplier on the extent violates it exactly. A pass over every tile violates
it and also changes the running time by six orders of magnitude, which the
proof does not need.

**A killed probe is not a green probe.** A restoration that was abandoned
proves nothing, and the report must say so rather than counting the step as
done.

### FND-071 — The whole-world pass the pyramid gave up was still in the viewer

**Believed.** The whole-world sweep before the first frame was a defect of the
level 1 rebuild, and the project corrected it there.[^34]

**True.** A second instance was live, in the demonstration binary, and it ran
on every start.

**Evidence.** The demonstration built a list of every open tile in its world,
by asking the ground about each of its tiles in turn, before it spawned
anything. The world is 640 by 440, so that is over two hundred and eighty
thousand generator calls between the person starting the program and the
first frame. Two test fixtures do the same, and one of them asserts a floor on
the length of the list it built.

The founding replaced the demonstration's list with a bounded sample, and the
whole pass went with it.

**Follows.** Three things.

**A cost claim reaches the examples, not only the engine.** The record binds
the founding choice.[^35] Nothing bound the demonstration, and the
demonstration is what a watcher looks at.

**A fixture may pay a cost that the engine may not.** The two fixtures that
still walk every tile are correct. A test may spend the world to build the
input an assertion needs, and the founding is the thing that may not.[^36]

**Search the tree for a shape, not for a name.** The pass was found by reading
the demonstration for another reason. Nothing failed, because a whole-world
loop is ordinary code and no check can tell a fixture from an engine.
### FND-073 — The remedy for a colliding allocator cannot be written in the register it protects

**Believed.** A number collision between parallel workers is solved by the
rule that the person who dispatches the work allocates every number first and
gives each worker its own.[^31]

**True.** The rule holds for the backlog and for the two registries, because a
number there is a file name or a table row, and a reserved range can be
written down. It does not hold for the findings register, the decisions
register or the blockers register. Each of those states its next free number
in its own text, and the check requires that line to be exactly one past the
highest row the file holds.[^32] A dispatcher therefore cannot reserve a
range inside the register. The reservation lives only in the instruction given
to each worker, and nothing fails when a worker ignores it or never receives
it.

**Evidence.** Five workers ran in separate worktrees in one session. Three
collisions followed, in three registers: two workers took FND-057, a third
took FND-058 that a fourth had already taken, and two took DEC-022. Every
worker had been given its own range. Each collision was found at merge, by
hand, and each cost a renumbering sweep across the tree for the citations that
had moved with it.

One worker was told to set the next-number line past its whole allocated
range, so that the reservation would be visible to the next worker. It could
not: the register check fails on any value other than the maximum plus one,
and that check gates the whole suite. The worker followed the gate and
reported the conflict.

**Follows.** Three things.

**A range reservation must be visible in the file, or it is not a
reservation.** The registers that carry a next-number line cannot hold one
today. Until they can, a dispatcher who runs parallel work must expect to
renumber at merge, and must sweep the tree for citations rather than trusting
the register alone.

**The collision is cheap to find and expensive to repair.** The check catches
a duplicate row immediately. What it cannot catch is a citation elsewhere in
the tree that names the number the merge moved. That sweep is manual, and it
is the part that costs.

**Prefer a register a worker does not have to number.** A register whose next
number is derived at read time, or whose rows are numbered at merge, removes
the window entirely. That is a change to three registers and their check, and
it is not made here.


### FND-083 — A footnote label defined twice sends half its citations elsewhere

**Believed.** A repeated footnote label in one document is a formatting
untidiness.

**True.** It silently retargets citations. The decisions register defined
`[^DEC7]` twice, once as a backlog item and once as an accepted record, and
defined `[^DEC8]` twice in the same way. A Markdown renderer resolves a
duplicate label to one definition. Every citation using the other meaning then
pointed a reader at the wrong document, and the text around it still read
correctly. Two further labels were each defined three and four times.

**Evidence.** An agent found the duplicates while it closed the register's
rows. The citation check passed before and after, because every path in every
definition existed. The check verifies that a path resolves. It did not verify
that a label means one thing.

**Follows.** Two things.

The documentation rule already forbids repeating a footnote. The rule had no
check, so the tree drifted from it. A rule without a check is a preference.

**This is the through-line shape.** One label is one name, and the file gave it
two declaration sites with no precedence and nothing that fails. Prefer a
descriptive label over a serial number, because a serial number invites a
second writer to reach the same value independently.

### FND-084 — A tree-wide repair reached the worktrees the tree ignores

**Believed.** A repair driven by a whole-tree search touches the tree the
project keeps.

**True.** The working copy holds many git worktrees under an ignored
directory. Each is a separate checkout on its own branch. A recursive search
from the repository root walks into all of them. A repair that rewrites what
it finds therefore edits many branches that the change does not concern, and
the ignore rule hides the damage, because the status of the main repository
never mentions it.

**Evidence.** A path repair after several backlog items moved directory
rewrote a line in each of the ignored worktrees as well as in the tree. The
edits were reverted after each worktree was checked to hold no other
uncommitted change. Nothing was lost. The commit rule already asks for a
whole-tree search after a sweep, and the search it asks for is the one that
found the extra files.

**Follows.** Two things.

**Scope a sweep to the tree the change owns, then search wider to verify.** The
search and the repair want different boundaries. A search that is too narrow
misses a call site. A repair that is too wide edits work that is not yours.

Before a repair rewrites a file, confirm the file belongs to the change. An
ignored directory is not empty, and the status command will not say so.

### FND-097 — A wide golden scenario paid its extent on every frame it did not need

**Believed.** The golden state hash test grew because subsystems entered the
state hash, so its cost was the price of the coverage it holds.

**True.** Most of the cost was duration, not coverage. Two scenarios exist for
their extent. The shoreline scenario is wide so that a soldier meets water.
The founding scenario is wide so that the engine has a good place and a poor
one to choose between. Both ran the same 32 frames as the narrow scenarios,
and a wide world costs its tile count on every one of them. The founding
scenario alone held about two thirds of the test.

**Evidence.** The frames of the two wide scenarios went from 32 to 8. The test
went from 143 seconds to 36 seconds on one development machine, and the whole
gate suite fell by about the same amount. The figures and the machine are in
the commit body, and the budget is in the local register.[^47] Not one hash
line changed when the golden files were recorded again. The two files lost
their trailing lines and kept every line they had. The simulation is
untouched, and the coverage the extent buys is untouched.

**Follows.** A scenario states what it is for. When it is for an extent, the
duration is a separate choice and it costs the extent on every frame. Ask what
each dimension of a fixture buys before you copy the dimensions of another
row.

### FND-098 — The perturbation probe covered one of the two determinism tests

**Believed.** The probe recipe proves that the determinism tests can fail.

**True.** It proved that one of them can. The recipe ran the thread-count test
and five key-field tests on the perturbed build. It never ran the golden state
hash test, although the same record asks both determinism tests to be able to
fail.

**Evidence.** The recipe listed the test binaries by name and the golden test
was not among them. The golden test does fail on the perturbed build, and it
fails in under a second, so nothing but the omission kept it out. The recipe
now runs it.

**Follows.** When a rule names two things, check that the check names both. A
list of binaries is a declaration site, and it drifts from the rule it serves
unless something compares the two.

### FND-099 — The cost of the gate suite is not one number

**Believed.** The gate suite has one wall clock cost on a named machine, and
one measurement of it gives the project a budget.

**True.** The cost depends on what else runs on the machine, and the spread is
large enough to change what the figure means. Five workers ran the suite at
once on one machine of sixteen hardware threads. One of those runs was killed
after about forty minutes, at a load average near fifteen, with four other runs
present. A run of the same suite on the same machine, with the load not
controlled but with no other suite seen in the process list, took about nine
minutes.

**Evidence.** The forty-minute run and the load average are reported by the
session that watched them, not measured by the work that wrote this entry. The
nine-minute figure is one run on an Intel Core i7-1260P, x86_64, 16 hardware
threads, debug profile, 31 August 2026. It is a development machine figure. It
is not evidence about the target platform.[^48]

**Follows.** Two things.

**A budget row states the conditions it holds under.** A figure taken under
contention and a figure taken alone are two quantities, and a table that holds
one number for both is the shape this project already knows: one fact, two
meanings, nothing that fails when they disagree. The local register now names
the conditions in the row.[^47]

**Serialise the suite before you compare two runs of it.** Two suites on one
machine do not cost twice one suite, and neither figure is the cost of the
suite.

### FND-089 — A deficit cannot recover under the default need rule

**Believed.** A unit that fails its draw builds a deficit, and the deficit
falls again when the shortage ends. The consumption kernel holds both
directions, and the recovery rate is one of the four values of the rule.

**True.** The recovery is unreachable under the default rule. A deficit falls
only while the need of a unit is at or above the threshold. The need rises by
the ration and falls by the decay, and the default rule sets the two equal, so
a unit that receives its whole ration holds the need it has. A unit whose need
reached zero holds at zero and never climbs back over the threshold. Its
deficit rises at every application until the shortage ends the unit.

**Evidence.** The test that watches a deficit rise and fall was written
against the default rule and failed. It passes with a ration above the decay,
and it then needs many more applications to clear the deficit than it took to
build it. No kernel changed between the two runs.

**Follows.** Three things.

The equality of the ration and the decay is a content choice, and the register
holds it. The choice now decides more than it did: with an end for a starving
unit, the equality makes every shortage that reaches zero need fatal. A new
row asks whether the default ration should exceed the decay.

**A recovery path needs a test that reaches it.** The rise and the fall are
one rule in the kernel, and a test of the rise alone passes while the fall is
unreachable. State the rates the test needs rather than taking the default.

The general shape is the one this register keeps returning to. Two content
values that hold a relation decide a behaviour that neither of them names.

### FND-093 — A rule behind an existing filter cannot be proved by the path that filters

**Believed.** A new refusal is proved by a test that drives the whole engine
path. Remove the refusal, and the test goes red.

**True.** The test stays green when an earlier stage of the same path already
refuses the case. The founding survey drops a candidate whose ground admits no
unit, so the settlement refusal beneath it never receives a refused place. The
run test removed the refusal and stayed green. It also stayed green when the
survey filter itself was removed, because the score prefers good ground and
chose a passable place anyway.

**Evidence.** The settlement refusal on ground that carries no unit. Two tests
that call the founding directly went red when the refusal was removed. The
test that founds a whole run did not.

**Follows.** Two things.

**Name what each test proves.** A test that drives the layered path is a guard
against a later change that removes an upper filter. It is not evidence that
the lower rule works. Prove the lower rule at the call that reaches it.

**A green test after a restored defect is a result, not a failure of the
method.** It says the case does not reach the assertion. Report it rather than
hiding it.[^55]

### FND-100 — "A watcher can read it" was counted as "a watcher can see it"

**Believed.** An item whose acceptance list says a watcher reads a value has
answered the need to show that value. The product record that asks for a world
worth looking at is therefore served by the same work that serves the engine.

**True.** The word names two different interfaces. In the open backlog it means
the public interface of the library in almost every case, and the window in
almost none. An item can satisfy every line of its acceptance list, add state
to the engine, and put nothing on the screen. Nothing fails, because no check
compares what the engine holds against what the viewer draws.

**Evidence.** A scan of the acceptance list of every open item found the word
in ten of them. One of those ten changes what the window shows. Two open items
name the viewer product record as what they serve, and neither of them draws
anything: one is a world-build cost item and the other is a terrain regression
test. The product index already carries the symptom in its own note, and the
note had gone unactioned because no backlog item held it.

**Follows.** Three things.

**An item that adds state a watcher should see states which interface it
reaches.** Write the window, or write the public interface. Do not write the
word that covers both.

**The viewer product record needs an item that draws, or it accrues debt in
silence.** A need with no item against it is not a paused need. It is a need
nobody is measuring.

**Read the display gap as a shape, not as an oversight.** The engine work is
governed by records and the display work is governed by nothing, so the two
drift apart at the speed the engine moves. That is shape 2 with the roles
reversed: the specific thing that rots is the picture, not the prose.[^46]

### FND-104 — Regrowth waited on a cost that the sparse store had already answered

**Believed.** Whether a deposit refills depends on what depletion turns out to
cost. The product record for the resources of the world deferred the question
in those words, and nothing else in the project said anything about regrowth.

**True.** The cost of depletion is not what governs it, and the answer arrived
with the gathering work rather than after it. The engine generates a starting
stock from the seed and the address, and stores only what units took, only for
the tiles they took from.[^49] Recovery is therefore the ageing away of a
stored take, not the growth of a stored amount. Its cost follows the number of
depleted deposits and never the tile count, and a world in which nothing was
gathered has nothing to recover.

**Evidence.** The stored take is a sparse ledger keyed by tile and kind, and a
world with no gathering holds no entry. The generated stock allocates nothing
at any tile count. Both properties are stated in the module and in the record
that governs it.[^49]

**Follows.** Three things.

**A deferral states what would answer it, not only that it is deferred.** The
deferral above named a cost. The thing that answered it was a storage shape,
so nobody reading the deferral would have recognised the answer when it
landed. The product record for a deposit that comes back was written eleven
records later, and only because somebody asked.[^50]

**A cheap shape can arrive before the need that wants it.** The sparse store
was chosen for the memory cost of the resource field. It also removed the
per-tile cost of every later rule that changes a stock. Check the store before
assuming a rule must pass over the world.

**A question deferred to a cost is worth revisiting when the storage changes.**
Search the deferrals when a storage decision lands, and say in the commit which
ones it answered.

### FND-105 — A record justified a decision with a property the engine cannot have

**Believed.** The engine holds the unit array in tile order. A stagger keyed on
the level 1 cell therefore selects a few long contiguous runs, and an identity
key would scatter the same units. ADR-0064 stated this three times, and the
choice module repeated it in a doc comment.

**True.** The unit arena is a slot array in spawn order that reuses a freed
slot, and it never compacts. It is not ordered by tile, and it cannot become so:
an accepted record forbids compaction, because compaction would invalidate every
identity that names a slot.[^51] The array can therefore never hold the property
the justification asserts.

**Evidence.** A review of ADR-0064 against the code found the claim in the fifth
force, in decision D4 and in the rejected alternative for an identity stagger,
and again in the choice module. The arena states its own behaviour in its doc
comment, and the free-list reuse is visible at the spawn path. Nothing failed,
because a justification is prose.

**Follows.** Three things.

**The decision survives; only the reason was wrong.** A cell key gives one frame
to a whole cell whatever order the array holds, and that is the property the
test actually asserts. The register already holds the separate evidence for
staggering by cell rather than by identity.[^52]

**State the condition, not the property.** The record now says that the
contiguity depends on the order of the array, and that the array is not so
ordered today. A conditional claim stays true when the condition changes.

**A justification that names a fact about the engine decays like a count.** The
scope rule keeps a count out of a record because the next change makes it
false.[^53] A load-bearing claim about how storage is arranged carries the same
risk and has no rule against it. Read a "because the engine does X" sentence as
a figure, and check it against the code before you rely on it.

### FND-106 — One shared sample was expected to starve every founding after the first

**Believed.** The candidate ordinal alone gives every faction one sample, so a
run of several foundings would seat one faction and refuse the rest. The
refined item states this, and it treats the refusal as the visible symptom of a
key that holds no faction.[^58]

**True.** A shared sample seats every faction. The sample holds many places,
and the places stand far apart in a world of this extent, so each founding
after the first takes a lower-ranked place that still keeps the minimum
distance. The defect narrows the pool that every founding after the first draws
from. It does not empty it.

**Evidence.** The faction was removed from the frame slot of the draw key, the
smallest change that violates the claim, and the founding tests ran. The test
that changes the faction and compares the samples failed. A test that founded
for four, six, eight and twelve factions and counted the factions seated stayed
green at every count, and so did every other test in the file. The command and
the counts are in the commit body.

**Follows.** Three things.

**A consequence test was written and then deleted.** It asserted that a run
seats every faction, and it named the shared sample as the defect it caught. It
caught nothing. A test that passes because the defect is milder than expected
is a guard, and the register already holds that shape.[^59]

**The key test is the only guard on the faction slot.** It changes the faction
and asserts that the sample changes, which is what the testing rule asks for a
keyed draw.[^43]

**A predicted symptom is not evidence.** The item predicted the symptom from
the key, which is sound reasoning and was still wrong about the size. Put the
defect back and read what fails, rather than writing the test the prediction
implies.

### FND-107 — A four-faction fixture could not see the separation rule

**Believed.** A test that founds a run and asserts the distance between every
pair of places defends the separation rule.

**True.** It defends nothing in a world of four factions. Four samples in a
world of that extent land far apart by chance. The whole separation rule was
removed and the test stayed green, because the places it measured were tens of
tiles apart either way. The same test over eight factions failed at once, on a
pair one tile apart.

**Evidence.** The separation comparison was replaced by a constant, and the
founding tests ran. The pairwise distance test passed over four factions and
failed over eight. The register already holds this shape twice, in two other
subsystems.[^33]

**Follows.** Two things.

**Ask what distribution the assertion needs.** The fixture must crowd the world
enough that two foundings compete for one place. A fixture that models the
ordinary run supplies no such pair.

**The boundary test is the stronger one.** A test that admits a place at the
minimum distance and refuses one step closer caught both the removed rule and a
distance one step off, and it needed no crowd.[^60]

### FND-109 — A whole-tree search cannot find a claim that wraps

**Believed.** A sweep is done when a whole-tree search for the name comes back
clean. The commit rule asks for that search, and asks for the command in the
commit body, so that a reviewer can run it again.

**True.** The search the rule asks for cannot see a claim that spans two lines.
Prose in this repository wraps at eighty columns, so a claim long enough to
matter is split, and a line-based search for the sentence returns nothing. The
control reports clean because it cannot see the site, not because the site is
absent.

**Evidence.** A review of ADR-0074 searched the tree for the old answer and
reported one site. A second search, for the terms separately rather than for the
phrase, found four. The site the first search missed reads "is an open" at the
end of one line and "choice" at the start of the next, and a search for "an open
choice" returns nothing. Two of the four sites were live: a test header that
calls a decided question open, and a register entry whose stated invariant the
decision had made false.[^57]

**Follows.** Three things.

**Search for the terms, not for the sentence.** Pick the two or three words of
the claim least likely to wrap together, search for each, and read the hits. A
single search for a whole sentence is a search that reports clean.

**Join the lines before matching when the claim must be searched as a whole.**
A search that strips the line breaks first sees what a reader sees.

**A control that cannot fail is not a control.** This one returned clean for a
tree that held four sites, and nothing about the result said so. Treat a clean
sweep as evidence only when the search could have found the thing.

The cost is not the missed site. It is that a search command in a commit body
reads as proof, and it is not one. State in the commit body which method the
sweep used, so that a reader can tell a search that could have failed from a
search that could not.


### FND-110 — An over-full tile does admit, once its own units have left

**Believed.** A tile above its capacity offers no room and admits nobody, while
the units standing on it may still depart. The record that permits an over-fill
states it that way.[^56]

**True.** The refusal holds only while the tile stands above its capacity. A
frame runs several admission passes, and a departure releases room at the end of
a pass. A tile that loses enough units inside one frame falls below its
capacity, and a later pass of the same frame admits against the lower count. The
tile then takes units in.

**Evidence.** A test placed three units above the capacity of a tile, ran a
frame, and read the arrivals. The tile held eleven units on ground that admits
eight. Nine units left in the first pass, and two arrived in a later pass. The
tile ended the frame at four.

**The monotone guarantee is untouched.** No tile gains a unit beyond its
capacity, and the over-fill still relaxes toward the capacity and never away
from it. Only the sentence about admitting nobody is wrong, and it is wrong
about the mechanism rather than about the outcome.

**Follows.** Two things.

**State the refusal against the occupancy after the departures.** An over-full
tile offers no room while it stays over its capacity. That is the claim the code
supports.

**A fixture for this rule must hold the tile above its capacity for the whole
frame.** A tile that drains below its capacity inside the frame admits for a
reason the rule does not name, so a test built on a small over-fill measures the
drain rather than the refusal. The suite that found this places enough units
that the tile stays above its capacity through every frame.
### FND-116 — Housing already has a residence column and an eviction path

**Believed.** A unit belongs to nothing. Backlog item 0059 plans a new
residence column on the soldier arena, and plans an eviction that clears the
residents of a lost site. The product record states that a unit lives nowhere
today.[^61]

**True.** Both exist. The consumption work gave the soldier arena a home
column that holds a slot of the settlement arena, with a reader and a writer
on the arena and a wrapper on the world. Under the record that fixes a
settlement to a tile and gives it the pooled store, the site a unit draws from
and the place a unit lives are one fact, so a second column would be a second
declaration of it.[^62] Destroying a settlement already clears the home of
every unit that named the lost slot.

**Evidence.** The soldier arena holds the column, its reader, its writer and
its slice reader. The founding writes the home of every unit it places. The
destroy path clears the home of every resident of the lost site. The
consumption pass and the cohort rebuild both read the column.

**Follows.** Three things.

**Item 0059 extends the home column. It adds none.** A residence column beside
the home column is the defect shape this register meets most often.[^63]

**The eviction exists, and its cost shape is the open part.** The destroy path
reads every unit of the world to find the residents of one site. That is a
pass over the population for a fact that a reverse index would answer
directly. Whether such an index exists is a decision, and the item names it.

**A separation of the two facts is a later decision, not a default.** A unit
that draws from one site and lives in another is a world the project may want.
Nothing needs it today, and the item that needs it owns the record.

### FND-113 — A per-tick ageing pass that resets its clock recovers nothing

**Believed.** A recovery that runs on every tick can hold its progress in the
tick it last ran at. The pass reads the elapsed ticks, divides by the period of
the kind, and stores the tick it ran at as the new anchor.

**True.** That rule recovers nothing at all whenever the period is longer than
one tick. The pass runs every tick, so the elapsed ticks are always one, the
division always gives zero, and the anchor moves forward every time. The
remainder of the division is the whole of the progress, and resetting the anchor
throws it away on every tick.

**Evidence.** The perturbation was put into the ageing function of the resource
module, and six tests of the recovery suite failed. The correct rule advances
the anchor by the whole periods it spent, and never to the tick it ran at.

**Follows.** When a pass reduces a stored value at a rate slower than the pass
runs, the remainder is state. Advance the clock by what was spent, not to the
present. A test that only asserts that the value falls over a long run cannot
tell the two rules apart; assert the exact amount at the exact tick.

### FND-114 — Two defences of the recovery rule are not observable through the engine

**Believed.** Every rule that a function states can be defended by a test that
drives the engine. Putting the defect back and watching a test fail is the
evidence that the test covers the rule.

**True.** Two rules of the ageing function have no observable failure through
the engine as the step runs today, because a second mechanism already excludes
the case.

The first is the clamp that stops an entry recovering more than it owes. The
subtraction already saturates at nothing owed, so removing the clamp changes no
answer.

The second is the ageing of an entry at the moment a new take joins it. The step
ages every entry before it resolves the orders to gather, so no entry is ever
stale when a take arrives.

**Evidence.** Both perturbations were put back, and the whole resource suite
stayed green. Two other perturbations of the same function failed six tests and
one test respectively.

**Follows.** Say plainly which rules a suite defends and which it only states. A
test that passes because an earlier stage already excluded the bad case is a
guard, not evidence.[^64] Both rules stay in the code, because the second
mechanism is an ordering that a later change can move, and neither costs
anything. Neither is counted as covered.

### FND-115 — A fixture that asks for the first kind a tile carries selects one kind

**Believed.** A fixture that walks the open tiles and takes, for each tile, the
first resource kind that the tile carries builds a mixed set of deposits. The
world holds three kinds, so a set of twelve tiles will hold several of them.

**True.** It selected stone on every one of the twelve tiles, and stone is the
one kind that does not recover. The scenario gathered, stored a take for each
tile, and recovered nothing. Every assertion about the ledger passed, because
the ledger held exactly what a correct run would hold.

**Evidence.** The golden scenario for gathering asserts after its run that the
world stored a take and that something recovered. The second assertion failed
on the first run of the scenario. The order the fixture searched in put food
first, so the outcome was not the order but the world: the tiles it reached
carry no food and no wood.

**Follows.** Choose the case, do not search for it. A fixture that asks the
world for "the first thing that fits" takes whatever the world offers first,
and the world is not trying to give the test a hard case. The scenario now
names how many deposits of each kind it wants, and fails when it cannot find
them.

**A fixture needs an assertion about itself.** This one was caught by a check
that the run reached the case, not by reading the code, and the file it would
have recorded would have looked correct for as long as anyone left it.[^55]
### FND-119 — A watcher can never see a unit at the moment a shortage ends it

**Believed.** The engine names three conditions, and a viewer that draws the
condition of a unit draws three cases. Backlog item 0120 planned two marks: one
for a unit a shortage holds, and one for a unit the shortage has taken to the
bound.

**True.** Only two of the three conditions reach a watcher. The engine scans
the death plane inside the step that takes a unit to the bound, so a unit that
a completed step left alive is fed or short. A viewer that reads the condition
after a step never reads the third. A second mark would be a capability that
nothing invokes.[^65] [^66]

**Evidence.** A fixture fed half its sites and starved the other half, and it
read the condition of every unit after each step. Over sixty-four steps the
hungry group went from fed to short at step sixteen and was dead at step
seventeen. No sample held a unit at the bound, at either of two bounds. The
engine's own starvation tests assert only the fed condition and the short one,
and neither names the third.

**Follows.** Three things.

**The viewer draws one mark for the condition, and the panel carries the
death.** The window marks a unit that a shortage holds. The panel states how
many units the last scan ended. That count is the whole record of a death that
a watcher can read.

**The log of the scan falls back to zero one step later.** The engine keeps the
log of one scan, so the row states the deaths of that step and not of the run.
A panel row that summed a run would state a different thing, and its label
would then be false.

**A later change to where the scan runs makes the third mark reachable.** A
contributor who moves the scan out of the step must add the mark, because the
picture would then be silent about a condition a watcher can read.

### FND-120 — A correction reached the record and the tests, and not the comment

**Believed.** The sweep that carried a correction into a record, its tests and
the prose files reached every site of the claim.

**True.** It did not reach the doc comment on the function the record governs.
The comment still stated the claim the register had already recorded as false,
and it stated it on the one function a reader opens first when they want to know
why the key holds what it holds.

**Evidence.** A review of the founding record searched the tree for the terms of
the corrected claim, separately rather than as a sentence.[^57] It found one live
site: the comment on the function that builds the frame slot of the founding draw
key. The record itself carried the correction properly, and so did the test that
guards the key. Two earlier sweeps of the same claim had passed over the
comment.

**Follows.** Two things.

**A correction sweep covers doc comments.** A comment is prose, it decays like
prose, and nothing fails when it goes false. The rule that keeps a count out of a
record exists because the next change makes the count false; a comment that
explains why a decision was taken carries the same risk and has no rule against
it.[^53]

**Search the artefact the record governs first.** The sites that matter most are
the ones a contributor reads before they read the record: the function, its
comment, and the test beside it. A sweep that starts with the documents finds the
documents.

### FND-121 — An index paraphrased a rule, and the paraphrase was quoted as the rule

**Believed.** The scope rule opens by saying that a record for a subsystem
nobody has built is the failure it exists to prevent. A record therefore waits
for its code.

**True.** The scope rule says no such thing. Section 1 opens with "Do not write a
record because a topic exists. Write a record because a constraint exists", and
its three conditions test for a constraint. None of them asks whether the code
exists. The registry states the opposite of the believed rule directly: a record
may be accepted before its implementation, provided the acceptance says plainly
that nothing implements it yet.[^1]

**Evidence.** The sentence lives in the decision record priority index, which
paraphrased the rule in words the rule does not contain and attributed the
paraphrase to it in a footnote. A search of the rule for the terms of the
paraphrase returns nothing. The dispatcher read the index, believed it, and
repeated it in three worker prompts in one night, including in the prompt that
framed a review of two records written ahead of their code. The reviewer checked
the source rather than the framing and reported that the text does not exist.

**Follows.** Three things.

**An index may not paraphrase the rule it points at.** A summary of a rule is a
second declaration of the rule, and nothing fails when the two disagree. State
the rule's own words or state only the pointer. This is shape 1 in prose.

**A footnote makes a paraphrase look sourced.** The citation was correct: it
named the right document. What it could not show was that the sentence above it
was not in that document. A reader who follows a footnote checks that the source
exists, not that it says the thing.

**Check the source before you put a rule in a worker's prompt.** A dispatcher
who quotes a rule to a worker is not passing on a constraint but creating one,
because the worker cannot weigh it against a document it was told not to
question. The reviewer that caught this was told its first duty was to consider
withdrawing two records, on the authority of a rule that does not say that.

### FND-122 — A test for each field of a key cannot find the field the key lacks

**Believed.** The testing rule's remedy for a keyed draw is complete. Write a
test for each field of the key, change that field, and assert that the draw
changes. A draw keyed on the wrong field then cannot survive.

**True.** The remedy inherits whatever the key already gets wrong. It tests the
fields the key names, so it is silent about a field the key should name and does
not. Every such test passes, and it passes for the same reason before and after
the defect is fixed.

**Evidence.** A review of a draft record found a draw key with the site in the
entity slot and nothing in the draw slot. Two proposals from one site in one
frame therefore draw one value. The item that will implement the record requires
a fixture holding exactly that case, and its acceptance list already named two
key tests: change the frame, change the site. Both pass whether or not the draw
slot carries an ordinal. The missing third test is the only one that fails.

The same review found the ordering key had the same shape. Every proposal at one
site carries the same site value, so a sort on the site alone is a grouping and
not an order, and the choice would fall to whatever produced the sequence.

**Follows.** Three things.

**Ask what two things the key must separate, before you ask what fields it
holds.** Name the finest pair the system can produce: two proposals of one site
in one frame, two candidates of one faction in one founding. Then check the key
separates them. A field list read forward cannot show an absence.

**A key constant across the things it orders is a grouping.** Sorting on it
leaves the order to whatever built the sequence, which is the defect the
iteration rule exists to forbid, wearing the clothes of a stable key.

**The project has met this once already and the precedent transfers.** The
founding key put the candidate ordinal in the draw slot for exactly this reason,
because the actor filled the entity slot and a second draw by one actor in one
frame had nothing left to separate it.[^58]

### FND-123 — The determinism tests guard against introducing a divergence, not against shipping one

**Believed.** The two determinism tests cover an unfixed order. If nothing states
what fixes the order of a result, the thread-count test or the golden state hash
will say so.

**True.** They cover two of the three cases and the third is the one that ships.

An order that varies with the thread count fails the thread-count test, and that
test is doing exactly its job. An order that is stable but no longer matches the
recorded hash fails the golden test, though it fails uninformatively, because a
golden file notices that something changed and cannot say which input the output
stopped depending on.[^43]

An order that is stable and matches the recorded hash fails nothing, ever. An
implementer who collects results into a structure and applies them in the order
that structure holds gets a perfectly deterministic order, by accident of the
data structure rather than by decision. Both tests are green and stay green.

**Evidence.** A review of a draft record found that nothing stated the order in
which admitted births are applied to the entity arena. A slot comes from a free
list, the pass that ends a unit runs immediately before growth in the same
frame, and the slot is part of an identity. Which newborn takes which freed slot
therefore follows from the application order, and every identity downstream of a
birth follows from that. The record said only that the new unit takes a slot.

**Follows.** Two things.

**A test cannot distinguish an order fixed by decision from an order fixed by
accident.** Both produce the same bytes. Only a record separates them, which is
what the counter-test in the scope rule means when it says a decision governing
determinism always needs a record even when it looks obvious.[^67]

**Ask the question of the recurring defect rule against the record, not against
the run.** The rule asks what fixes the order of every parallel result, and says
that a change with no answer is a defect.[^68] A green suite is not an answer.
The answer is a sentence in a record, and if no record holds one then the order
is load-bearing and unrecorded, whatever the tests say today.


### FND-124 — A founded group had no food and the whole map died together

**Believed.** The founding seats a group, gives it a home site, and the engine
then runs. The demonstration showed a world that lives.

**True.** The founding left the store empty and set no rate, and nothing else
filled the store. Every unit therefore drew nothing on every draw, and the whole
population ended within one hundred ticks of the start.

Two things can fill a site store. A caller writes a quantity into it, or a
production rate earns one, and a founding did neither. A unit that gathers is
not a third: a gathered amount goes to the carry of the unit, and no pass moves
a carry into a store.[^69]

**Evidence.** A run of the demonstration world, four factions and thirty people
each, printed the store of each site and the condition of each person for two
hundred frames. The store held zero on every frame. Every person was short from
frame nine and gone by frame ninety. The commit body holds the figures and the
command.

**Follows.** Three things.

**The founding now sets the rate.** One unit of food that the place reaches
feeds one person, at the ration the need rule holds. The founding is the one
call that has both the site and the survey that measured the ground, so the rate
is set there and the ration is read rather than repeated.[^70]

**The survey score now decides whether the group lives.** The score weighs the
food a place reaches, and until now that weight changed nothing after the
founding. A place that reaches less food than its group needs now runs that
group short.

**No test asked whether a founded world survives.** Every founding test
asserted on the founding. Every consumption test built its own store. Neither
side drove the founding into the run, so the gap between them was invisible to a
green suite. The test that closes it removes the rate and watches the population
end, which is the only proof that the assertion reaches the case.[^71]

### FND-125 — The case a test was written for was one event in every 158

**Believed.** A fixture that produces all three cases a test needs covers
them. The tile event test needs an unheld tile, a held tile, and a tile that
took a new holder on the tick under test. The fixture produces all three, so
the assertions reach all three.

**True.** The three cases are not the same size, and the one the item exists
for is the smallest by two orders of magnitude. The tick under test reported
3471 events. Of those, 3353 named a tile that nobody held, 118 named a held
tile, and 22 named a tile whose holder changed on that tick. The case that
distinguishes a fresh read from a stale one is 0.6 percent of the log.

An assertion that held over most of the log would therefore say almost
nothing. The stale read reports the correct answer for 3449 of the 3471
events, because a tile whose holder did not change reports the same holder
either way.

**Evidence.** The stamp was put back to a read before the holding spread. The
agreement test failed on tile 105, one of the 22. The test of an unheld tile
stayed green, and so did the fixture test. The counts come from a run on the
development machine, and they are a property of that fixture and that seed,
not a measurement of the engine.

**Follows.** **Count the case, do not only reach it.** A fixture check that
asks whether a case occurred passes on one occurrence, and one occurrence in
3471 is indistinguishable from luck when the seed or the rule moves. State the
size of the case the test depends on, so that a later reader sees when the
fixture has stopped supplying it. This is the fixture shape of the testing
rule, seen from the other side: the rule says a uniform input hides a defect,
and here the input is not uniform but the interesting part of it is
rare.[^84] [^85]


### FND-128 — The engine already counts who lives at a site

**Believed.** Nothing states how the engine answers how many units a site
holds. A record therefore has to decide whether to store an occupancy count
and maintain it by the change. Draft ADR-0081 states the belief in its context
and builds decision D3 on it.

**True.** The engine answers the question today. The cohort table holds one row
for each faction at each site, and each row holds a headcount. The residents of
one site are the sum of its rows. The table is derived from the home column of
the soldier arena, which is the residence.[^72] Every unit that can hold a
residence is a soldier, because the character arena holds no home column, so the
headcount is the whole resident count and not a part of it.

The prohibition of the draft therefore lands wrongly. It forbids a pass over the
units that recomputes the count inside a running frame. The consumption pass
does exactly that, twice in one frame: once before the pooled draw, and once
after the scan that ends a starved unit. Each rebuild walks the whole home
column.

**Evidence.** `CohortTable::rebuild` takes the home column, the faction column,
the live column and the site count, and it increments a headcount for each live
unit that names a site. `World::consume` calls it twice. `World::cohorts` and
`CohortTable::headcount` are public, so a caller reads the row of one faction at
one site through the public interface. The character arena holds no field named
for a home or for a site.

The check the draft asks for also exists. `World::cohorts_describe_the_units`
derives the table again from the home column and compares, and its own
documentation gives the reason the draft gives: a summary that nothing compares
against its source is a second declaration site with nothing that fails on
disagreement.

**Follows.** Three things.

**A stored occupancy count would be the third declaration of one fact**, not the
second. The home column holds it, the cohort table summarises it, and a
maintained count would hold it again. The draft prices the cost as one extra
site, and a check between two copies does not guard three.[^73]

**The caller the draft names is already served.** A birth admission runs after
the consumption pass, so the cohort table is settled where the admission reads
it. What is missing is a reader that sums the rows of one site, because the
table splits the count by faction.

**A record must be read against the code before it is accepted, and the reading
must go past the module the record is about.** The residence work is in the
settlement module and the soldier module. The count that falsifies the premise is
in the consumption module, which the draft never names.

### FND-129 — A stated rule bans a citation that four accepted records make

**Believed.** A decision record cites no product requirement record. The rule is
stated twice, in the project orientation and in the product guide, and it gives
a reason: a product direction changes more often than a constraint does, so a
decision record must not rest on one.[^74]

**True.** No script checks the rule, and the project does not follow it.
ADR-0064, ADR-0067, ADR-0074 and ADR-0075 are accepted and each cites a product
record. Three of the four drafts under review cite one. In draft ADR-0081 the
product record carries the whole justification of decision D1.

**Evidence.** `scripts/check_prds.py` fails a product record that cites a
decision record. Nothing checks the other direction. A search of the accepted
records for the product file prefix returns four files.

**Follows.** Two things.

**A reviewer cannot reject a draft on this ground alone.** The accepted
precedent runs the other way, and a rule applied to new work but not to the work
already accepted is a rule that punishes whoever arrives last.

**The project chose, on 1 September 2026: the rule is dropped.** A decision
record may now cite a product requirement record for the need that made a choice
hard, and it must not take a figure, a budget or a date from one. The two
statements of the ban are removed. No accepted record was repaired, because none
was in breach of a rule that no longer exists. The closed row holds the
reasoning.[^75]

**What this finding cost, and what it bought.** The rule stood for the whole
research phase and bound nobody. It was found by a review that read a record
against the code rather than against the guide.

### FND-130 — Two footnote rules are stated, are broken, and are checked by nothing

**Believed.** The record check catches the mechanical part of the documentation
rule, so a record that passes the gate follows the footnote rules.

**True.** The documentation rule states that footnotes are numbered in the order
they occur in the body, and that a footnote is never repeated.[^76] The record
check tests neither. Three of the four drafts reviewed break one or both, and
the gate passed on all three.

**Evidence.** In draft ADR-0081 footnote 15 names the source that footnote 7
already names, and no footnote 14 exists. In draft ADR-0082 footnote 20 names the
source that footnote 5 already names, and footnotes 19 and 20 occur before
footnotes 12 to 18. In draft ADR-0076 footnote 8 first occurs after footnote 9.
`just records` reported no failure for any of them.

**Follows.** Two things.

**A gate that passes is not evidence that a rule holds.** It is evidence that the
rules the gate encodes hold. The gap between the two is where a written rule goes
quiet.

**The check is cheap and the repair is not.** A duplicate footnote is invisible
to a reader, and it produces two labels that a later edit can move apart. A
backlog item adds the check.


### FND-131 — Item 0059 plans three things the engine has already built

**Believed.** Item 0059 gives a site a capacity and a resident roll. Its
impact review already carries one correction: the residence column and the
eviction path exist, so the item extends the column and adds none.[^77] With
that correction absorbed, the four numbered work steps of the item are new
work.

**True.** Two of the four are built, and a third is half built. Only the
housing capacity is wholly new.

The item asks for an occupancy count. A per-site count exists, split by
faction, and the item's own reader test is already satisfiable: a caller reads
how many units live in a site through the public interface, without walking the
units.[^78] What the item asks for beyond that is one property of the count,
that a change maintains it rather than a sweep. The number is not new. Its
maintenance is.

The item asks for an invariant check that compares the count against the
residence column, and for a test that proves the check can fail. Both exist.
`World::cohorts_describe_the_units` derives the table again from the home
column and compares. A test drives the world until the two disagree, asserts
the refusal, steps until they agree again, and asserts the agreement.

**Evidence.** `World::cohorts` and `CohortTable::headcount` are public.
`World::cohorts_describe_the_units` is public. The test
`every_headcount_sums_to_the_live_population` spawns a unit that no
application has seen, asserts that the check then returns false, steps for one
period, and asserts that it returns true again. That is the proof of failure
the item asks a future author to write.

**Follows.** Three things.

**A withdrawn item must be re-refined against the code, not against the
record.** Items 0059 and 0060 left `refined/` because the record they rest on
was rejected.[^79] Rewriting them against a new record repeats this defect. The
re-refinement starts by asking what the engine already does.

**A correction absorbed once does not inoculate an item.** FND-116 corrected
this same item about this same subsystem, and the item took the correction for
the residence column alone. The correction had a wider reach than the sentence
that carried it. When a finding says a subsystem is further along than the
project believed, read the whole subsystem again rather than the one claim the
finding names.

**The capacity is the work.** An item that says so is smaller, and it is
honest about what remains.


### FND-132 — A commit that cites a register entry on another branch makes the trunk red

**Believed.** A citation of a register entry that another branch holds is a
forward reference. The author knows the entry is coming, says so in the commit
body, and the trunk repairs itself when the other branch lands. Stating the
dependency in the body is enough.

**True.** The citation check derives the truth from the tree it runs on, not
from what an author knew. A commit that cites an entry no branch in the trunk
holds fails the check from the moment it lands until the other branch merges.
Every worker who runs the gate in between reads a red pipeline that their own
work did not cause. A commit body cannot make a check pass.

**Evidence.** A dispatcher commit withdrew two backlog items and cited FND-128
as the reason. FND-128 existed only on the reviewing worker's branch. The
citation check reported two failures on the trunk against items 0059 and 0060,
and it kept reporting them until the review branch merged.

The same commit left four citations of the two paths it had moved. It swept for
citations **to** the rejected record and never searched for citations **of** the
files it moved. Six failures stood on the trunk in total, from one commit.

**Follows.** Three things.

**Order the merge by what the gate needs, not by what the work needs.** A branch
that holds a register entry another branch cites must land first. The dispatcher
reversed the merge order for this reason once the failure was visible, and the
order is derivable before the failure: whichever branch defines the entry goes
first.

**A sweep is done when a whole-tree search comes back clean, and the search must
name the thing that moved.** A search for references to the subject of a change
is not a search for references to the files the change moved. Both are needed,
and only the second one catches a rename.[^80]

**Knowing that a citation is forward does not soften it.** Write the register
entry in the same commit as the work that cites it, or do not cite it yet.
### FND-133 — The experiment that was named to prove a test blind would have failed it

**Believed.** The terrain test of the holding does not defend the gradient it
is named for, and the experiment that proves it is to flatten the claim
threshold to one value for every passable kind. The register states that the
test then stays green.[^81]

**True.** The conclusion holds. The experiment does not. The test read the
thresholds directly and asserted that the threshold of level ground was below
the threshold of a hill. Flattening the thresholds makes that comparison
false, so the test fails, and it fails without running the rule at all.

The experiment that separates the two is to leave the thresholds ordered and
stop the rule from reading them. The constants a reader checks are then
correct, the behaviour they describe is gone, and only a counted outcome can
see the difference.

**Evidence.** Both experiments were run against the test as it stood. With
every passable threshold set to one, the old test failed on the line that
compares two constants. With the thresholds untouched and the decision
function given a fixed threshold of one, the constant comparison passed and
the holding spread over mountain as readily as over level ground. The
replacement test fails in both cases, and it names the counts when it does.

**Follows.** **An experiment that proves a test blind must leave every input
the test reads unchanged, except the behaviour.** A perturbation that also
moves a constant fails the test through the constant and reports nothing about
the behaviour. The green run is the signal, so a red run for the wrong reason
reads as success. This is the shape the testing rule states for a determinism
probe, applied to an ordinary test: a probe must prove that the assertion can
see the defect, and not only that something can go red.[^82]

**Write the failure message so that it names which assertion failed, and on
what numbers.** A red run says only that something went red. It does not say
which rule caught the defect, and under a perturbation that moves two inputs
at once the wrong assertion is the one that fires. This is what made the
correction above findable: the replacement test prints the counts of the two
kinds it compared, so a reader sees that the behaviour was measured. A bare
assertion goes red under the flattened thresholds as well, and the run reads
as a pass of the experiment.

The item that carried this correction listed the flattening as its acceptance
condition, and so did the brief that dispatched it. Neither would have been
wrong about the outcome. Both would have been wrong about what the outcome
proved.

### FND-138 — A gate result read through a pipe reports the exit code of the pipe

**Believed.** A gate run reports whether the gate passed. The command
`just check` exits zero when every gate passes, so a run that exits zero
passed.

**True.** A shell pipeline exits with the status of its last stage. A gate
piped into `tail` therefore exits with the status of `tail`, which is zero
whatever the gate did. The gate can fail, or be killed, and the run still
reads as a pass.

The failure is silent in the direction that matters. A pipeline hides a red
gate and never hides a green one, so the mistake only ever reports success.

**Evidence.** Two instances in one session, made independently by two agents.

The first ran the gate in the background through a pipe into `tail`. The task
reported exit code zero, and the captured output was empty. An empty capture
from a gate that prints hundreds of lines is what said the gate had not
finished. The run had been killed, and `tail` had reported its own success.

The second ran the gate on the trunk through a pipeline and reported a pass.
The gate had been terminated by a signal, which the shell reports as 143, and
the pipeline still reported zero.

Neither instance was found by reading the exit code. The first was found
because the capture was empty, and the second because the termination was
noticed by other means.

**Follows.** **Read the exit status of the command under test, and never of a
pipeline it sits inside.** Run the gate, capture its status directly, and read
the log afterwards.

```
just check > /tmp/gate.log 2>&1; echo "EXIT=$?"
```

Do not pipe a gate into `tail`, `grep` or `head` when the exit code is the
thing being reported. A filter is safe only after the status is captured.

**A capture that is empty or short is evidence, and not a formatting
problem.** A gate prints hundreds of lines. A run that reports a pass and
shows almost nothing did not finish.

**The shape is one outcome read from a place that cannot see it go wrong.**
The register holds that shape twice already, in a conservation check that read
only the source of a transfer, and in a differential test that could not see a
defect which moved both sides alike.[^95] [^96]


### FND-134 — A product record can state a structure, and the check cannot see it

**Believed.** The product record check defends the rule that a product record
states a need and never a structure.[^86] It fails when a record body cites a
decision record, because a citation shows that the record states a structure.

**True.** The check sees the citation and nothing else. It reads each body line
for a decision record number and for the record directory path.[^87] A record
that states the same structure in prose, and names no record, passes.

**Evidence.** The product record for a deposit that comes back states the
storage the engine uses and the algorithm that recovers a deposit. It says that
the world stores only what units took, that recovery removes a stored record,
and that the world answers the amount when a caller asks. A decision record
holds all three claims, and the project has accepted it.[^88] The product record names
no decision record, so the check passes it. A review found it by reading.[^93]

**Follows.** Two things.

**The gate between `shaped/` and `accepted/` is a human review, and the check
does not stand in for it.** The check stops a citation. It cannot stop a
structure. A reviewer reads the cost section against the decision records that
cover the same subject, and treats an overlap as a defect.

**The structure reached the product record through the cost section.** The gate
asks what a need costs at the target scale. A writer that cannot state a cost
without a measurement reaches for the mechanism instead, because the mechanism
is the only concrete thing available.[^90] A cost section states what the cost
must grow with. It does not state what the engine stores.

### FND-135 — The world reserves no storage for the target population

**Believed.** The world reserves storage sized for the target population, and
that storage does not change during a run. A run does not stop to grow. The
accepted product record for a founding states this as a property that follows
from its cost.[^89]

**True.** The world reserves nothing. The unit arena opens as many slots as the
slot index holds, and its own comment says that the limit is the range of the
index and not a cost budget. It reserves no memory for those slots. Each spawn
appends one entry to each of its ten columns, so the storage grows with the
population under a running simulation.

**Evidence.** A driver founded 120 people in a 640 by 440 world through the
public interface, and read the arena capacity back as 4294967295. That is the
range of the slot index. The arena construction takes that limit as its
capacity, and the spawn path appends to each column.

**Follows.** Three things.

**A record the code contradicts is worse than no record.**[^91] The product
record states a property that a reader would design against, and the engine
does not hold it.

**The disagreement is architectural, and a product record cannot settle it.**
Whether the world reserves unit storage at construction, or grows it, is a
constraint. A row in the decisions register holds the question.[^92]

**A cost statement of a product record is a claim about the engine, and nobody
checks it.** The checkable statements of a record get a test. The cost section
gets none. This claim survived acceptance and every gate.

### FND-136 — A merge committed conflict markers into a register, and every gate passed

**Believed.** The gate suite defends the three registers. It checks the
numbering, the priority indexes, the citations and the records.[^94]

**True.** No check reads a register for a conflict marker. A merge left the
three markers of an unresolved conflict in the decisions register, the commit
went to the main branch, and the whole gate suite stayed green.

**Evidence.** The register held the marker lines at its footnote list. The two
sides of the conflict differed by one footnote definition, so a reader skimming
the file would see two nearly identical lines and no error. A product review
found it while reading the footnotes for another purpose, four commits later.

**Follows.** Two things.

**A marker is invisible to a checker that parses structure.** Each check reads
the register for the shape it cares about: a heading, a number, a footnote
label. A marker line is none of those, so every check walks past it. A grep for
the three markers over the whole tree costs nothing and catches this class.

**A register merges more often than any other file, because every worker
updates one.** The registers are where this failure lands, and parallel work
makes it more likely, not less.

### FND-140 — The gate suite is slow because the gate build does not optimise

**Believed.** The gate suite costs what it costs because of how much it tests.
The register holds a wall clock figure for the suite and a budget above it,
and neither says where the time goes.[^97]

**True.** Nearly all of the time is the execution of the Rust tests, and
nearly all of that is the development profile. The workspace manifest declared
a release profile and nothing else, so the tests ran unoptimised. The engine
steps a world of hundreds of thousands of tiles, and unoptimised code on that
work is several times slower than optimised code. Raising the optimisation
level of the development profile cuts the execution of the Rust tests by a
factor near five and leaves everything else alone.

Compilation is not where the time goes. A run of the suite that changes no
source file compiles nothing at all. A run after one edit to a core source
file rebuilds in a few seconds at every optimisation level tested. The
optimisation level costs a full rebuild once, when the profile changes, and
does not touch the loop a contributor runs all day.

**Evidence.** The suite was measured on an Intel Core i7-1260P, x86_64, 16
hardware threads, with no other work on the machine, on 1 September 2026. The
figures are in the commit that carried the change and in the register.[^97]
Each gate of the suite was timed on its own. The Rust tests and the
nondeterminism probe were the whole cost within a few seconds; the formatting
gate, both lint gates, the invariant scripts, the Python tests, the smoke
test, and both record gates cost seconds together.

**Follows.** **A cost figure for this suite means nothing without the build
profile beside it.** The register already asks a row to name the machine, the
architecture and the profile. The profile column was carrying the answer to
the question nobody had asked.

**Look at the profile before you look at the tests.** The five slowest test
binaries were the obvious place to start, and every one of them is slow for a
sound reason: it builds a world large enough to hold more than one kind of
ground, because a smaller world measures its own fixture.[^98] Cutting the
work they do would have bought time by testing less. The profile bought more
time and changed no test.

### FND-141 — A one-byte tile field over the target scale does not overflow a `u32`

**Believed.** A one-byte tile field summed over the whole world overflows a
`u32`, and that is why a level 1 accumulator widens.

**True.** It does not overflow. The largest value a one-byte field holds is
255, the target scale is 16.7 million tiles, and the product is under the
ceiling of a `u32` by less than one part in a hundred. The sum passes the
ceiling above 16,843,009 tiles.

The rule that the accumulator widens is right, and nothing here weakens it.
The margin is under one per cent, so the sum overflows at a world one per cent
larger, at any field wider than one byte, and at any accumulator that also
carries a second field. What is wrong is the example, not the rule.

**Evidence.** Arithmetic. The target scale is the figure the cost register
holds.[^99] The check that this project runs against the claim is now a test:
it sums a two-byte field over the target scale into a `u32` and asserts the
panic, and it sums the same field into the widened accumulator and asserts the
exact total.[^100]

**Follows.** **State the rule, and check the example against it.** A record
that argues from an example is only as good as the example, and this one was
never checked. A reader who checks it and finds it false has no way to tell
whether the rule is false too.

The rule is stated as a hard invariant of the project, and the example sits
beside it. The invariant needs a correction, and the owner of that document
makes it. This entry is the evidence.

### FND-142 — Two runs of this suite hours apart are not comparable

**Believed.** A run of this suite on an idle machine gives the cost of the
suite, so a run before a change and a run after it measure the change. The
register says a row must name the conditions, and it names contention as the
condition that matters.[^97]

**True.** Contention is not the only condition. The same suite, on the same
machine, with no other work on it and no source change between them, took
about four and a half minutes early in a session and about seven minutes after
two hours of continuous running. The later figure repeated three times within
a few seconds of itself, so it is not noise. A laptop under sustained load
settles at a lower sustained clock, and the machine reports a low temperature
while it does, so a reader who checks for heat sees nothing wrong.

The effect is large enough to invert a result. One option was measured against
an early baseline and appeared to make the suite half again as slow. Measured
against a baseline run immediately before it, the same option made no
difference at all, which is what the reasoning had predicted.

**Evidence.** An Intel Core i7-1260P, x86_64, 16 hardware threads, 1
September 2026, one worker with exclusive use of the machine. The baseline
measured 263 s, 283 s and 296 s in the first hour and 429 s, 432 s, 435 s and
435 s in the third. The commit that carried this work holds every figure and
the commands.

**Follows.** **Compare two runs that are next to each other in time, never
two runs from different hours.** Alternate the two configurations and report
the pair. A figure taken an hour before its comparison is a figure about the
machine.

**A single row in the register is a snapshot, not a baseline for a
comparison.** The register already says a row is one run and asks for an
isolated one. Isolation is necessary and it is not sufficient. A row supports
the claim that the suite cost this much at that moment. It does not support
the claim that a change made the suite faster.


## References

[^1]: Findings register, FND-038, in this document.
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: Movement timing note. `docs/research/movement-timing.md`
[^4]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^5]: Findings register, FND-004, in this document.
[^6]: Record scope research. `docs/research/adr-scope-findings.md`
[^7]: Decision Record Scope, section 4.5. `.claude/rules/adr-scope.md`
[^8]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`
[^9]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D3. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^10]: Decision Record Scope, section 4.6. `.claude/rules/adr-scope.md`
[^11]: Findings register, FND-051, in this document.
[^12]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^13]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^14]: ADR Registry. `docs/adrs/REGISTRY.md`
[^15]: Findings register, FND-047, in this document.
[^16]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^17]: Recurring Defect Shapes, shape 2. `.claude/rules/recurring-defects.md`
[^18]: ADR-0066, entity storage holds four fixed shapes, decision D2. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^19]: Findings register, FND-058, in this document.
[^20]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^21]: Testing rules, section 6. `.claude/rules/testing.md`
[^22]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^23]: Testing rules, section 2a. `.claude/rules/testing.md`
[^24]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^25]: Findings register, FND-022, in this document.
[^26]: Vector entity representation, sections 9 and 15, decision D155. `docs/research/reports/18-vector-entity-representation.md`
[^27]: The character graph and inheritance, sections 2.1, 3.3 and 15.3. `docs/research/reports/14-character-graph-and-inheritance.md`
[^28]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^29]: Findings register, FND-056, in this document.
[^30]: Findings register, FND-072, in this document.
[^31]: Findings register, FND-050, in this document.
[^32]: The register check script. `scripts/check_registers.py`
[^33]: Testing rules, section 2a. `.claude/rules/testing.md`
[^34]: Findings register, FND-049, in this document.
[^35]: ADR-0075, the founding choice reads a bounded sample of the world. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^36]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^37]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^38]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^39]: Backlog item 0085. `docs/backlog/complete/0085-show-a-watcher-who-holds-the-ground.md`
[^40]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^41]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
[^42]: Backlog item 0084. `docs/backlog/complete/0084-give-a-tile-one-faction-column.md`
[^43]: Testing Rules, section 2. `.claude/rules/testing.md`
[^44]: Findings register, FND-075. `docs/FINDINGS.md`
[^45]: Findings register, FND-078. `docs/FINDINGS.md`
[^46]: Recurring defect shapes, shape 2. `.claude/rules/recurring-defects.md`
[^47]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^48]: ADR-0008, the primary target is `aarch64-unknown-linux-gnu`, decision D2. `docs/adrs/accepted/adr-0008-the-primary-target-is-aarch64.md`
[^49]: ADR-0072, a tile stock is generated, and only what was taken is stored. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^50]: PRD-0018, a depleted deposit comes back. `docs/product/shaped/prd-0018-a-depleted-deposit-comes-back.md`
[^51]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^52]: Findings register, FND-023, in this document.
[^53]: Decision Record Scope, section 4.3. `.claude/rules/adr-scope.md`
[^54]: Backlog item 0071. `docs/backlog/complete/0071-derive-tile-passability-from-tile-capacity.md`
[^55]: Testing rules, section 2a. `.claude/rules/testing.md`
[^56]: ADR-0074, a spawn may over-fill a tile and only admission enforces the capacity, decision D2. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^57]: Commit Message Rules, after a sweep. `.claude/rules/commits.md`
[^58]: Backlog item 0094. `docs/backlog/complete/0094-decide-how-many-groups-found-a-world.md`
[^59]: Findings register, FND-093, in this document.
[^60]: ADR-0076, a founding keeps a fixed distance from the foundings before it, decision D1. `docs/adrs/accepted/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^61]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^62]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^63]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^64]: Findings register, FND-093, in this document.
[^65]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^66]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D3. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^67]: Decision Record Scope, section 1, the counter-test. `.claude/rules/adr-scope.md`
[^68]: Recurring defect shapes, shape 4. `.claude/rules/recurring-defects.md`
[^69]: ADR-0073, gathering is admitted by sort-then-admit against the tile. `docs/adrs/accepted/adr-0073-gathering-is-admitted-by-sort-then-admit-against-the-tile.md`
[^70]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^71]: Testing rules, section 2a. `.claude/rules/testing.md`
[^72]: Findings register, FND-116, in this document.
[^73]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^74]: Product requirement records, what does not belong here. `docs/product/README.md`
[^75]: Decisions register, DEC-056. `docs/DECISIONS.md`
[^76]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^77]: Findings register, FND-116, in this document.
[^78]: Findings register, FND-128, in this document.
[^79]: Review 0143, the housing, growth, founding and recovery records. `docs/reviews/0143-the-housing-growth-founding-and-recovery-records.md`
[^80]: Commit Message Rules, after a sweep. `.claude/rules/commits.md`
[^81]: Findings register, FND-080, in this document.
[^82]: Testing rules, section 1. `.claude/rules/testing.md`
[^83]: Findings register, FND-133, in this document.
[^84]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^85]: Backlog item 0084, give a tile one faction column. `docs/backlog/complete/0084-give-a-tile-one-faction-column.md`
[^86]: Product requirement records, what does not belong here. `docs/product/README.md`
[^87]: The product record check. `scripts/check_prds.py`
[^88]: ADR-0080, a depleted deposit recovers by ageing the stored take. `docs/adrs/accepted/adr-0080-a-depleted-deposit-recovers-by-ageing-the-stored-take.md`
[^89]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^90]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^91]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^92]: Decisions register, DEC-059. `docs/DECISIONS.md`
[^93]: Reviews, the founding and deposit product records. `docs/reviews/0149-the-founding-and-deposit-product-records.md`
[^94]: The record and register checks. `justfile`
[^95]: Findings register, FND-075, in this document.
[^96]: Findings register, FND-078, in this document.
[^F137A]: The bindings and the event log method. `crates/cachette-py/src/lib.rs`
[^F137B]: The event types. `crates/cachette-core/src/event.rs`
[^F137C]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^F137D]: Backlog item 0153. `docs/backlog/proposed/0153-let-python-read-an-event-without-repeating-its-layout.md`
[^97]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^98]: Testing rules, section 2a. `.claude/rules/testing.md`
[^99]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^100]: ADR-0083, the gate build checks every integer overflow, decision D2. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
