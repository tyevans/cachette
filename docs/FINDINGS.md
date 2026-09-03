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

**Next number: FND-338**

**This line answers from merged history, so it cannot see a number that a
branch has taken and not merged.** A dispatcher issues ranges above it for that
reason, and those ranges live in prompts that no register can read. Four
collisions in one session came from the gap between the two, and a finding holds
the case.[^ALLOC2]

## A. Corrections to stated rules

### FND-327 — The demonstration drew and stepped at one rate, and a record said so as if it were a property of the viewer

**Believed.** The viewer runs after the step, on the stepping thread, so the
drawing rate and the tick rate are one number.[^F218B] The Python
demonstration says the same in its own docstring.

**True.** The first half is a property of the viewer and it still holds. Every
draw follows the steps of that frame, on one thread. The second half described
the loop that existed when the record was written. It is not a property of the
viewer, because the caller owns the loop and decides how many steps a frame
runs.[^F327B]

**Evidence.** The demonstration now steps the engine as many times each frame
as a clock says, and it draws once. A paused world runs no step and still
draws, so the camera moves and the panel reads while the tick stands still.

**Follows.** A watcher can stop the world, run it at four speeds, and ask for
exactly one tick. That last one matters because the engine keeps its logs for
one tick, so a promotion or a rationing was readable for one thirtieth of a
second and no longer.

**The record is not edited here.** ADR-0067 is accepted, and the sentence sits
in prose that describes the viewer correctly. A worker who supersedes it should
say that the caller owns the number of steps in a frame.

### FND-306 — A check that prints its failure while the commit lands looks exactly like a check that never ran

**The symptom is the finding.** A commit landed on the trunk holding conflict
markers. The marker check had run in the same command, and its failure text had
scrolled past on the way. That is what a working check looks like when
something discards its answer, and it is also what a check looks like when it
does not run. Nothing in the output distinguishes the two.

**Believed, twice, and wrong both times.** The first explanation was that the
check reports a failure and exits zero. The second was that the check skips a
worktree's own files when it runs from inside that worktree, because a worktree
path is in its skip list. Two readers reached for "the check is broken" before
"my shell discarded its answer", because the symptom points that way.

**Both are false, and the check is sound.** A probe put a real marker in a file
inside a worktree and ran the check from inside that worktree. It named the
file, named all three lines, and exited 1. The skip list holds the path of a
worktree nested inside the tree being scanned, not the tree itself, because the
root is derived from the location of the script. The skip is also gated on the
scan being the root.

**The cause is the shell that called it.** The check ran on the left of a pipe,
inside a chain that continues on success. A pipeline exits with the status of
its last command, so the status belonged to the command reading the output and
not to the check. The chain continued and the commit landed.

The failure text is written to the error stream, so it printed past the reader
and was visible the whole time. It was read as noise from the merge that ran in
the same chain.

**How to take the measurement again.** An example prices every stage on a
world of a given extent, group and thread count, and prints the state hash
beside the split so a sweep is also a determinism check.[^F300B]
**Evidence.** The same check, on the same file, two ways:

```
python3 scripts/check_conflict_markers.py | tail -2 && echo REACHED
  ... 3 failures
  REACHED
python3 scripts/check_conflict_markers.py > /dev/null && echo REACHED
  ... 3 failures
  (nothing)
```

**The defect appeared inside the attempt to verify it.** The second reader ran
the probe piped through a reader of its output, saw exit zero, and concluded
the check was blind. Unpiped, the same tree gives exit 1. So a status was
masked by a pipeline during the checking of a claim about a status masked by a
pipeline.

**Follows.** Four things.

**Read the symptom as ambiguous, not as evidence.** Failure text with a
successful outcome has three possible causes: the check did not run, the check
ran and cannot see, or the check ran and something threw away its verdict. The
output is identical in all three. Anyone who names one of them without a probe
is guessing, and two readers guessed wrong in one night.

**Never put a gate on the left of a pipe.** Redirect its output if the output
is long. A gate exists to stop the next command, and a pipe takes that away
without saying so.

**A failure that prints and does not stop looks exactly like a check that does
not run.** That is why the first two explanations both blamed the check. The
symptom carries no information about which of the three causes produced it, so
the next person to see it will guess as well unless they probe.

**Probe the check before you blame it.** Both wrong explanations were reasoned
from reading the source. The right one took one command and a temporary file.
Putting the defect back is the rule for a test, and it is the rule for a check
that is under suspicion.[^13]

### FND-305 — The stage apparatus flags a stage that gains from threads it does not take, and not one that loses by taking them

**Believed.** The stage table finds a wrong `takes_threads` declaration. The
benchmark states the rule in its own documentation: a stage declared `false`
that improves with the thread count means the declaration is wrong. The table
prints the declaration beside the measurement in every row so that a reader can
compare them.

**True in one direction only.** The rule names one case and there are two. A
stage declared `false` that improves is caught. A stage declared `true` that
gets worse is the same class of error, it costs more, and nothing looks for it.
The declaration and the measurement disagree in both cases.

**Evidence.** The influence solve is declared as taking a thread count. On a
256 by 256 world it is 5.9 times slower at twelve threads than at one, and it
is still 7.6 times slower at twelve threads on a world of 1,048,576 tiles. The
register holds the sweeps.[^F305A]

**The instrument was built the same night, to find this class of error, and it
stayed silent.** The table printed the declaration and the cost of the stage in
adjacent columns, at one thread and at twelve, and reported no note. A reader
who ran it saw the rows and read nothing wrong in them, because the
documentation told them which direction to look.

**Follows.** Three things.

**A rule that names one direction of a symmetric test teaches the reader to
look one way.** The cost of the omission is not that the check missed
something. It is that a person holding the output missed it too.

**The declaration is a value with two declaration sites and no check.** The
source states `takes_threads` and the measurement states what threading did.
Nothing fails when the two disagree, which is the first recurring defect
shape.[^13]

**The repair is a rule, not a constant.** A speedup far from one in either
direction is the signal. A stage that declares it threads and measures slower
threaded fails the same comparison as one that declares it does not and
measures faster.

### FND-300 — The influence solve gets slower as the thread count rises, on a small world

**Believed.** A stage that takes a thread count runs faster with more threads.
The stage register declares `influence_solve` as taking one, and the target
scale measurement agrees: it is 12.68 milliseconds and 5.25 percent of a 241
millisecond frame at 16.7 million tiles and twelve threads.

**True at the target extent, and the reverse on a small world.** On the
demonstration world the solve costs more with every thread added. Measured on
an x86-64 development machine, 256 by 256 tiles, 256 units, four factions, the
mean of 120 frames after 30 warm-up frames:

| Threads | The frame | `influence_solve` | Its share |
|---|---|---|---|
| 1 | 36.314 ms | 9.343 ms | 25.7 percent |
| 2 | 30.229 ms | 11.465 ms | 37.9 percent |
| 4 | 35.786 ms | 19.749 ms | 55.2 percent |
| 8 | 55.716 ms | 37.313 ms | 67.0 percent |
| 12 | 81.471 ms | 55.482 ms | 68.1 percent |

The solve is 5.9 times slower at twelve threads than at one, and the frame is
2.7 times slower at twelve than at two. **The solve also costs more in absolute
terms on the small world than on the target one**, 55.5 milliseconds against
12.7, on a world with 256 times fewer tiles.

**How to take the measurement again.** An example prices every stage on a
world of a given extent, group and thread count, and prints the state hash
beside the split so a sweep is also a determinism check.[^F300B]

**The cause is the shape of the parallel section, not the arithmetic in it.**
One relaxation pass opens a thread scope for each faction, and a solve runs a
fixed count of passes, so a frame opens the pass count multiplied by the
faction count of them. At the target extent each spawned thread relaxes
thousands of cells and the spawn is paid back. On the demonstration world each
one gets a handful of cells and the spawn is the whole cost.

**The guard that exists cannot catch it, by design.** The code holds a thread
back only when the cell count is at or below the thread count, and it says why:
the rule reads the two numbers the caller already supplied and holds no
constant of its own. A world of this extent has far more cells than twelve, so
the guard never fires and every scope spawns.

**The solve does not cross over at any extent measured.** A second sweep took
three extents at one thread and at twelve. The frame crosses over between
65,536 and 262,144 tiles, where other stages start to carry the win. The solve
itself is still 7.6 times slower with twelve threads than with one at 1,048,576
tiles, so the extent at which threads pay this stage is above a million cells
of world.

**The determinism holds.** The state hash is identical at one thread and at
twelve, at every extent measured, so this is a cost defect and not a
correctness one.

**The absolute figures are noisy and the direction is not.** The machine ran
other builds throughout, and one point measured 81 ms in one sweep and 192 ms
in another. Every point in both sweeps has the same sign.


**The scope count is most of the cost and it is not the cause of the
inversion.** A change that opens one thread scope for each pass instead of one
for each faction in each pass was built and measured, and it was not taken. It
removes twenty-four of the thirty-two scopes a frame opens. Measured on an
x86-64 development machine, 256 by 256 tiles, four factions, the mean of 120
frames after 30 warm-up frames:

| Threads | Solve, one scope for each faction | Solve, one scope for each pass |
|---|---|---|
| 1 | 8.761 ms | 2.732 ms |
| 4 | 47.167 ms | 6.055 ms |
| 12 | 179.561 ms | 15.447 ms |

**The spawn is about three quarters of the solve at every thread count, and
the solve is still 5.7 times slower at twelve threads than at one after it is
gone.** Removing the scopes does not move the sign. It follows that the spawn
is a large fixed cost and that threading this stage loses on this world for a
second reason as well.

**The change was not taken, and the reason is a record and not a measurement.**
The version measured above gives each faction a scratch plane of its own, so
the write half of the solve grows with the faction count. An accepted record
decides against exactly that and names its own reopening condition.[^F300C]
The figures stay here as evidence about the cause. They are not a proposal.

**The determinism holds across the change.** The state hash reads
`9d81e94936b9f445` at one, four and twelve threads, before and after, on the
same world and the same commit.

**Follows.** Three things.

**A stage declared as taking a thread count can still lose by taking one.** The
stage apparatus states that a measured speedup far from one on a stage declared
`false` means the declaration is wrong. It checks one direction. A negative
speedup on a stage declared `true` is the same class of error and nothing looks
for it.

**A measurement taken only at the target extent cannot see this shape.** Every
frame figure in this project was taken at the target extent, where the defect
is invisible. The small end is a different regime and it had never been
measured.

**The demonstration is slow for this reason and for no other.** Both front ends
ask for the smaller of the machine's parallelism and twelve. Asking for fewer
would make the demonstration between two and three times faster today with no
engine change, and that is the reason not to change the requested frame rate.
The repair belongs in the solver, so that no caller has to know. An item holds
it.[^F300A]

### FND-293 — No golden scenario reached a promotion, and the files still moved

**Believed.** A golden file that moves when a subsystem lands is evidence that
the golden set covers the subsystem. The state hash is exact and
order-sensitive, so a file that moved saw the change.

**True that it moved, and false that it covered anything.** All eight golden
files moved when the promotion work landed. **None of the eight promoted
anybody.** The files moved because three new unit columns and two new schedules
enter the state hash, which happens whatever the pass does. A change to the
promotion rule itself would have moved nothing.

**The evidence.** The gathering scenario is the only one that gathers, and
gathering is what the promotion reads. A temporary assertion that it produced a
character failed. A second probe printed the reason: the best unit in that
scenario gathers 5 over the frames it runs, against a default threshold of 24.
Every other scenario gathers nothing at all, so no unit in the golden set has
ever had a deed to its name.

**The repair was in the scenario, not in the engine.** The gathering scenario
already states its own recovery periods, on the stated ground that a period is
a parameter of the kind and the engine holds no test value. The threshold is
the same kind of parameter, so the scenario now states it too, at a value this
world reaches. Two mutations confirm the file now guards the behaviour: ranking
the candidates by identity instead of by deeds moves the file, and promoting
nobody trips the assertion.

**Follows.** Three things.

**A golden file that moved is not evidence that a golden file covers.** A
change that widens the hashed state moves every file, and it looks exactly like
a change that altered behaviour. The question to ask is not whether the file
moved but whether the scenario reaches the pass, and the answer to that is an
assertion in the fixture rather than a diff.

**Assert the case in the scenario, in the same place the parameters are set.**
The gathering scenario already carried two such assertions, for a stored take
and for a recovery, each written after a fixture covered half a rule while
looking complete. This is the third of the same kind, and the pattern is worth
following rather than rediscovering.

**The same gap is open for the positions and nothing has closed it.** Item 0279
records that no golden scenario reaches the position pass. The promotion case
was found and repaired in the same round; the position case still stands, and
the two together say that a new pass should be assumed uncovered until a
scenario asserts otherwise.


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

### FND-159 — A one-byte influence cell loses the field to truncation when the solve iterates

**Believed.** An influence cell is one byte against a fixed reference. The
research measured the usable gradient of that cell at about 4.4 octaves, which
it states covers the decision range of every consumer, and it gives a reach of
about 35 cells.[^F159A]

**True.** The measurement holds for a plane that is computed once. It does not
hold for a plane that an iterated relaxation writes back into itself. Each
pass narrows the accumulator to the cell, and the narrowing truncates, so a
small value loses a whole step on every pass. The tail does not decay at the
rate the stencil sets; it decays to zero.

**Evidence.** One fixture, one stencil, one source at the ceiling, run to rest
at one thread on the development machine. With a one-byte cell the field
reached four cells and every cell beyond that held zero. With a two-byte cell,
and nothing else changed, the field reached thirteen cells and fell smoothly
over all of them. The reach is a property of the arithmetic, so it does not
depend on the machine.

**Follows.** The cell is two bytes wide, and the record states the reason
beside the decision.[^F159B] The research finding is right about the gradient a
consumer reads and wrong about the gradient the solve can carry, and the two
are different quantities.

**The general shape.** A width argument made for a value that is written once
does not carry to a value that is written repeatedly into itself. Ask how many
times the narrowing happens before anybody reads it.

### FND-160 — The thread-count test does not guard a fixed iteration count

**Believed.** The two determinism tests catch a solver that stops on a
convergence test, because a convergence test is the defect the record names
and the thread-count test is the highest-value test in the project.[^F160A]

**True.** They do not. A convergence test that reads an exact integer
comparison gives the same answer at every thread count, because the comparison
does not depend on how the work was split. Both runs stop after the same pass,
both produce the same field, and the thread-count test compares them and finds
them equal.

**Evidence.** The perturbed build stops the influence solve when a pass changed
nothing. Under it the influence thread-count assertion passes, at one thread
against twelve, over a field solved to rest. The assertion that fails is the
one that reads the pass count. A second perturbation was needed to give the
thread-count assertion a failure mode, and it is a different defect: a pass
that reads a neighbour outside the run it is filling as nothing.[^F160B]

**Follows.** **A fixed iteration count needs its own test, and that test reads
the count.** This is the case the testing rule states in the general: a defect
that repeats gives one answer on every thread and on every run, and a test
which compares two runs cannot see it.[^F174A] The rule named a draw keyed on
the wrong field. A solver that stops early is the same shape.


### FND-319 — The package says its verb interface is not written, and a verb ran

**Believed.** The Python package states in its own top-level docstring that it
is a stub, that it re-exports the compiled module, and that the selector
interface, the verb interface and the view scope are not written yet.

**True.** The selector interface is not written. The other two are. The
compiled module carries set-valued verbs and columnar reads, and the worked
example in the project orientation calls both.[^F319A]

**Evidence.** The example in the orientation document was run unchanged
against the installed package. It built a world, spawned a set of units, gave
one gather order, stepped the world at four threads, checked the invariants,
and read a state hash, two column dictionaries and a gather count. It printed
a tick and a gather count and raised nothing.

```
grep -n "are not written yet" python/cachette/__init__.py
grep -n "fn spawn_soldiers\|fn order_gather\|fn event_log_columns" crates/cachette-py/src/lib.rs
```

**Follows.** The one sentence the package writes about itself is wrong about
two of the three things it names, and nothing fails, because a docstring is
prose. This is the shape the recurring defect rule names last: a document that
no longer describes the code.[^F319B] It is also the need that product record
0021 states, and that record cites the same shape as the cost it must
carry.[^F319C]

**The repair is not made here.** The sentence and the documentation of this
package are one statement, and correcting the sentence alone would create a
second place that states what the package holds. The worker who chooses how
this package is documented owns both.


## C. Defects found in specified rules

### FND-326 — The panel cut one line kind of eight, and the other seven stayed inside it by luck

**Believed.** The panel cuts a value that does not fit, so that text can never
be written over the panel edge.[^F326A]

**True.** One line kind cut its value. The title, the note, the heading, the
legend row, the ground row and the founding row all wrote from the left margin
with no right bound. The length of the text was the only thing holding them
inside the rectangle.

**Evidence.** One note of 32 characters had 30 glyphs of room, and the stored
layout picture shows its ink two glyphs into the padding. The longest note that
fits is 30 characters, and every note in the panel was under that bound by
accident. No author had anything to tell them what the bound was, because the
bound follows from the panel width and the glyph size and neither was written
down where a writer would look.

**Follows.** The cut now lives in one writer that takes a right edge, and every
line kind of both the head-up display and the deck writes through it. An author
cannot reach the map, whatever the text says. The bound is derived from the
width and the glyph table, so nobody counts characters and nobody counts them
wrongly.

**A cut is still a defect.** A cut line states something other than what it was
given, in silence. The check that reports one now covers every line kind, and
one note was rewritten rather than left to be cut.

### FND-328 — Nothing in the engine could say how many units a faction had left

**Believed.** The panel reports the units of each faction, so a watcher sees
how the factions stand.

**True.** Every unit count the panel stated was a count of the window. The
drawing pass counts what it painted, by colour, over the tiles the camera
reached. No value anywhere in the engine held the population of a faction in
the world, and nothing could compute one without reading every live unit.

**Evidence.** A search of the world interface found `holding_of`, which gives
the tiles a faction holds, and no counterpart for its people. The soldier arena
held one total live count and no split by faction.

**Follows.** A faction whose last unit starves vanishes from the picture with no
number falling to zero anywhere. That is the one thing a watcher most wants to
see, and the demonstration could not show it.

**A count is not the fix; a maintained count is.** Counting the units of a
faction reads every live unit, and at the target scale that is one million reads
for one row of one panel, every frame, which the panel record forbids.[^F328A]
The arena now maintains the count at the two sites that change a slot's live
byte. That is one fact in two places, so the arena check recounts and compares,
and it fails when the copies disagree.[^13]

### FND-237 — The record check reads no source file when it runs in a worktree

**Believed.** The record check reports a record that no other record and no
source file cites. Running it from a worktree gives the same answer as running
it from the main checkout.

**True.** It reads no source file at all from a worktree. The check walks the
tree for Rust and Python files and skips any path that holds a part named
`.git`, `target` or `worktrees`. A worktree of this project lives under
`.claude/worktrees/`, so every path below it holds that part and every source
file is skipped.

The effect is confined to the uncited note. The section check, the volatile
figure check, the citation check and the registry check all read documents,
and those are unaffected.

**Evidence.** The file walk of the check was run against a worktree root with
the same skip set. It returned zero Rust files. The same run reports the
uncited note for twelve records, including one whose claim is cited from two
source modules.

**Follows.** **A filter that names a directory by its base name reaches every
path that passes through it.** The skip set means "do not descend into a
worktree from the main checkout". Inside a worktree it means "skip
everything", because the root itself is inside one.

**A check that reads nothing reports the same shape as a check that found
nothing.** Zero files scanned and zero citations found produce the same note.
Nothing distinguishes them, and the note reads as a finding about the record.
A check that walks a tree should state how many files it read.[^37]

The uncited note is a note and never a failure, so no gate went green that
should have been red.[^237B] The cost is a false signal to whoever reads it.

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
derivation names every multiplier on its path.**

**Outcome, 2 September 2026. The terrain table now states the multiplier.**
Every kind of ground answers a step multiplier beside its capacity, because
the capacity and the multiplier describe the same tile.[^37HOME] The mountain
kind carries two and every other kind carries one. The scale constants table
holds the row and says the value is derived from the ratio of the two accepted
crossing times.[^99] The value is derived, not measured, and nobody decided
it directly.[^28] The forest kind and the hill kind carry the baseline,
because no accepted crossing time separates them from level ground, and a
register row holds that open choice.[^37NEXT]

**The 4-tick difference has a candidate explanation, and it is unverified.**
The closed-form law gives 125 ticks and the timing check measured 129. The law
counts the steady state only. A formation pays two stages outside the steady
state: the leading rank spends one dwell to enter the chokepoint before any
unit leaves it, and the last rank spends one dwell to clear the exit tile
after the steady state ends. Two dwells at the baseline dwell of 2 ticks is
4 ticks, and 125 plus 4 is 129. The arithmetic matches exactly.

**Mark this unverified.** The match is an arithmetic identity, not a
measurement. The movement kernel does not exist, so nothing re-ran the check
and nothing instrumented the entry tick or the clearing tick. **What would
verify it:** run the same formation through the movement kernel when it
exists, count the tick that the leading rank enters the chokepoint and the
tick that the last rank leaves the exit tile, and confirm that the two stages
cost one dwell each. A record must not state a crossing time as an exact
figure until that check has run.

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

**The reads held more of the cost than the writes, and nothing predicted
that.** A measurement on the target platform after the change found the frame
1.92 times cheaper, and the merge that justified the work is a little over half
of what the change returned. The larger single surprise is the level 1 rebuild,
which costs 11.6 times less with nothing in it changed: it sums the value of
every tile, and reading one tile used to be a binary search into a list holding
an entry for almost every tile. The table holds the rows.[^F292C]

**Follows.** Four things.

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

### FND-186 — A panel test could not tell the stock left from the stock the ground gave

**Believed.** A test that read the panel's two stock numbers back against the
engine proved that the panel states what a tile still holds. The panel prints
"what is left, of what the ground gave", and the test compared both numbers
against the engine's own readers.

**True.** The test could not tell the two apart. Its fixture was a world in
which nobody had gathered, and in such a world the stock left equals the stock
the ground gave, for every tile. A panel that read the generated stock into
both rows passed it.

**Evidence.** The defect was put back: the panel's reader for the stock left
was changed to call the reader for the generated stock. Every test in the file
stayed green.[^F186A] A second test was written that gathers first and then
asserts that the two numbers differ. With that test present, the same
substitution fails.

**Follows.** This is the shape the testing rule already names: a fixture that
models the typical case supplies no extreme, so the assertion never receives
the input that would fail it.[^23] The rule needed no change. What it
needed was the falsification, and the falsification found the hole in one run.

The general form is worth stating. **When a panel prints two numbers that are
usually equal, a fixture in which they are equal tests neither.** The engine
holds several such pairs: the stock and the generated stock, the demand and
the grant of a ration, the production and the upkeep of a site. A test of any
of them needs a world in which the pair has come apart.

### FND-187 — The food of a tile shades it more than the height does, within one kind of ground

**Believed.** The viewer shades a tile by its height, and a second, smaller
term rides on top. The second term was the tile stub value, and it was chosen
to be small against the height range so that the ripple never hid the ground.

**True.** The height range of one kind of ground in one world is far narrower
than the full height range. In a 128 by 128 world at one seed, the tallest
plain tile and the shortest plain tile were 26249 and 40476 in the fixed-point
range, which is about twelve of the fifty-six brightness steps the height is
given. Any second term wider than twelve steps therefore decides the order of
two tiles of one kind.

**Evidence.** The colour now reads the food stock, over thirty-four steps. A
test that compared the tallest and the shortest plain tile then failed: the
shorter tile carried food and drew brighter.[^F187A] The test was repaired by
holding the food at zero, which is the isolation it always needed and did not
have.

**Follows.** A watcher reads the food of a tile more strongly than its relief,
within one kind of ground. That is the trade this project chose, because the
food is what the product record asks a watcher to see and the relief is
readable across kinds by hue.[^F187B] Any later shading term must state which
of the two it is willing to dominate.

A test that compares two tiles on one property must hold the others fixed. The
height test did not, and it passed only because the term it ignored was small.

## D. Cost estimates that were wrong

### FND-222 — A frame at the target scale costs eleven times its budget

**Believed:** nothing. No measurement existed on the target platform, so the
project had no figure for the cost of a frame at the target scale. The frame
budget of 100 milliseconds is derived from the tick rate and the tile edge,
and nobody had checked the engine against it.

**True:** one frame at 16,777,216 tiles and 1,000,000 units costs a median of
1,135 milliseconds on two Graviton3 hardware threads. That is 11.4 times the
budget. One thread costs 1,861 milliseconds, so a frame holds 1.86
core-seconds of work.

**The cost is two straight lines.** A tile costs 78 ns for each frame on one
thread and a unit costs 557 ns. The two constants predict the target scale row
to one part in two hundred. The unit count is therefore the larger half of the
frame at the target scale, and it holds 1,000,000 units against 16,777,216
tiles.

**Evidence:** the target platform register holds the run, the machine, the
commit and every row.[^F222]

**Follows:** the tile pass ran 1.81 times faster on two threads and the unit
passes ran 1.35 times faster, so the half of the frame that is larger is also
the half that scales worse.

**The two constants in this entry are weaker than it says.** A pre-registered
prediction later missed by 6.2 percent, and FND-240 holds it. Read them as an
approximation good to about ten percent. The measured rows in this entry and
in FND-224 do not move, because neither is computed from a constant. The measured world held no settlement, so every
figure above is a lower bound.

**This entry first said the budget needs at least 19 cores at perfect
efficiency. A later run on 16 cores refuted the reasoning behind that
sentence, and FND-224 holds the correction.** Perfect efficiency is not
available: the unit passes reach a floor and stop.

### FND-224 — The frame budget is out of reach on any core count

**Read FND-330 before quoting a figure from this entry.** The 500 millisecond
frame and the 300 millisecond floor below are superseded. Later work made the
frame 177.9 milliseconds at the target scale on 12 threads. The title of this
entry still holds, because the budget is 100 milliseconds and the frame is
above it.

**Believed:** FND-222 divided 1.86 core-seconds by a budget of 100
milliseconds and concluded that 19 cores would reach it. That reasoning
assumed the work divides.

**True:** it does not all divide. A run on 16 Graviton3 hardware threads gives
a frame of 500 milliseconds at the target scale, which is 5.0 times the
budget, at a speedup of 3.69 on 16 cores. **The unit passes reach a floor near
300 milliseconds and gain nothing between 12 threads and 16.** The tile pass
has no floor in the range measured and reached 6.13 on 16 threads.

**A frame at the target scale therefore cannot fall below about 300
milliseconds on this engine, whatever the core count.** The budget is 100
milliseconds. No machine reaches it.

**That floor is about twice as high as this entry says.** Every figure here
was taken with the units packed into a band, and FND-245 shows a unit costs
about twice as much at the density the project states. The conclusion holds
under both patterns and the level does not.

**Evidence:** the target platform register holds both runs, the machines, the
commits and every row.[^F222] The unit half is the difference between a world
of 1,000,000 units and a world of none, at 4,194,304 tiles, taken at five
thread counts.

**Follows:** buying a larger machine does not deliver the tick rate. Either a
unit must cost less in a frame, or fewer units must do work in a frame. The
project already holds the second shape as a design principle, because a
set-valued command permits a cheaper algorithm than a per-entity loop. This is
the first measurement that says the principle is load-bearing rather than
tidy.

**Two cautions.** The measured world held no settlement and no character, so
the unit half is a lower bound and the floor may be higher. The step starts
threads for each parallel stage rather than using a pool, so part of the floor
may be that cost rather than the work.

### FND-225 — The tiles hold the memory at the target scale, and the units do not

**Read FND-246 before quoting a figure from this entry.** The 545 MB below is
the figure at one thread. A machine needs about 960 MB.

**Read FND-246 before quoting a figure from this entry.** The 545 MB below is
the figure at one thread. A machine needs about 960 MB.

**Believed:** memory at the target scale would be dominated by the entity
tiers rather than by the tiles, on the reasoning that a tile is generated and
only its change is stored. A derivation put the total at one to three
gigabytes.

**True:** a world of 16,777,216 tiles and 1,000,000 units holds **545 MB
resident at one thread**, and 876 MB at 12. FND-246 holds the thread rows. The same world with no unit holds 456 MB, so the whole population
of one million adds 89 MB. **A tile costs 27 bytes and a unit costs 89 bytes.**
The tiles are five sixths of the total.

**The generated-tile records are not contradicted.** Nothing stores a tile
value or a tile stock. The 27 bytes are the columns the world does allocate
for each tile, and one proposed item already names the holder column as one of
them.[^F162D]

**Building the world needs more than holding it.** The high water mark at the
target scale is 872 MB against 545 MB resident, and the gap holds at every
large extent. A machine sized to hold the world will fail to build it.

**Evidence:** the target platform register.[^F222] Each point was measured by
a process that built one world and exited, because a process that has already
built a large world does not return the memory and would report the high mark
of the run.

**Follows:** the derived figure was high by a factor of two to six, and it was
wrong about which half dominates. The measured world holds no settlement and
no character, and the scale constants table names 5,000 settlements and 50,000
living characters, with the character layer derived at about 85 MB. Adding
those to 545 MB does not reach one gigabyte, so the shape of the answer is
unlikely to change.

### FND-240 — A pre-registered prediction refuted the two-constant model

**Believed:** the cost of a frame is the tile count times a constant plus the
unit count times a second constant. The register said the two constants
predict the target scale row to one part in two hundred, and called that the
strongest evidence in the run.

**True:** the agreement was one point. A prediction written into the register
and committed **before** the run that tested it named 396.6 milliseconds for
16,777,216 tiles and 500,000 units at 12 threads, with a hit defined as five
percent. The measurement is 371.9 milliseconds, which is 6.2 percent low and
outside the band.

**The cost of a unit is not one constant.** At 16,777,216 tiles it is 248 ns
at 500,000 units and 270 ns at 1,000,000, so it rises with the population. The
constant the prediction used came from a world of 4,194,304 tiles, where it is
298 ns, so it depends on the extent as well.

**Evidence:** the register holds the prediction, the band, the result and both
commits.[^F222] The commit that states the prediction holds no result, and the
commit after it holds the result, so the order of the two is the evidence.

**Follows:** the two constants are an approximation good to about ten percent,
and the register now says so. **The headline result does not move**, because
it never rested on the model: the frame at the target scale, the tile pass and
the unit passes are each measured rows, and the floor in the unit passes is a
difference between measured rows.

**The method is the finding as much as the result is.** A consistency check
computed after the data is in hand cannot fail, and this one would have stood
in the register as strong evidence. It cost ten cents and one commit ordering
to learn that it was worth about a tenth of what it claimed.

### FND-241 — The unit cost of a frame is not one stage, and most of it cannot be measured from outside

**Believed:** the unit passes are 298 milliseconds at 12 threads, and finding
which stage holds them would give the project one optimisation target.

**True:** no single stage holds them. At the target scale, of 274 milliseconds
of unit cost in a 521 millisecond frame, the choice scoring is 71
milliseconds, one bridge rebuild is 26, the economy is 6, and **170
milliseconds is a residual that the public interface cannot divide.** The
largest part is the part nobody can name.

**The residual holds** the movement intents, admission, the holder spread, the
death scan, the part of the level 1 rebuild that reads the units, and the walk
over every live unit inside the choice pass that the interval does not remove.

**Evidence:** the register holds the rows.[^F222] Each part was priced by
running a whole frame with a switch off and taking the difference, because the
engine holds no instrumentation and the benchmark added none. Three switches
exist on the public interface: the economy schedule, the choice interval, and
the bridge rebuild, which is public and was priced directly.

**A correction inside this finding.** The step calls the bridge refresh three
times in a frame, and a first reading of that put the bridge at three rebuilds
and 79 milliseconds. The refresh compares a revision counter and returns when
the bridge is still accurate, so the frame pays one rebuild and two constant
checks. The figure is 26 milliseconds, not 79.

**Follows:** a stage that cannot be measured from the public interface is a
stage nobody can prove they improved, and that may be why none of this has
been optimised. Something must make the stages separable before the largest
part of the unit cost can be worked on. Backlog item 0229 already asks for a
stage measurement and names the constraint that a clock must not enter the
engine.

### FND-245 — The placement of the units doubles the cost of a frame

**Believed:** the benchmark measured the engine at the target scale. The
fixture placed the units by walking the world from the first tile and filling
each tile that admits one.

**True:** that pattern packs 1,000,000 units into a band across the top of the
map at one unit for each tile, and leaves the rest of the world empty. It is
about seventeen times denser than the target scale describes, because one
million units over 16,777,216 tiles is one unit for each seventeen tiles.

**A unit costs about twice as much at the density the project states.** The
same frame, in one process on one machine from one build, under two placement
patterns:

| Threads | Unit cost packed, ms | Unit cost scattered, ms | Ratio |
|---|---|---|---|
| 1 | 578.5 | 1,363.4 | 2.36 |
| 2 | 422.5 | 962.2 | 2.28 |
| 4 | 329.5 | 746.5 | 2.27 |
| 12 | 278.5 | 587.5 | 2.11 |

**Evidence:** the register holds both patterns at every thread count.[^F222]

**Follows:** every packed unit figure in the register is a lower bound. A frame
at the target scale costs 835 milliseconds at 12 threads at the stated
density, which is 8.4 times the budget, against 526 and 5.3 times packed.

**The shape survives and the level does not.** The unit passes scale to 2.08
on 12 threads packed and 2.32 scattered, so the conclusion in FND-224 that the
budget is out of reach holds under both. The floor is about twice as high as
that entry says.

**The memory does not move**, at one part in four thousand between the two
patterns.

**This is the shape the testing rule names, found in a benchmark rather than
in a test.** The rule says to ask what distribution the work needs rather than
to copy a convenient world, and to put the defect back and watch the test stay
green. Nobody asked it of this benchmark, and the fixture was chosen because
it was easy to write.

### FND-246 — A memory figure without a thread count is not usable

**Believed:** a world at the target scale holds 545 MB. FND-225 states it and
the register stated it without naming a thread count.

**True:** 545 MB is the figure at one thread. The same world holds 572 MB at
two threads and 876 MB at 12. The step gives each thread its own output slot,
so the resident size grows with the thread count by about 30 MB for each
thread.

**The peak moves much less**, from 872 MB at one thread to 957 MB at 12,
because the peak is set by the build and the build runs at one thread whatever
the caller asks for.

**Evidence:** the register holds the three rows.[^F222] Each was measured by a
process that built one world and exited.

**Follows:** a machine needs about 960 MB free to build and step a world at
the target scale, not 545 MB. The conclusion of FND-225 does not change: the
tiles still hold the memory and the units still do not, and both figures move
together with the thread count.

### FND-242 — A list that could not go stale had gone stale

**Believed:** the budgets register carried a table of the records that still
hold a derived cost figure in their bodies, naming ADR-0003, ADR-0005 and
ADR-0006. The register said the record check carries the list, and that the
check fails when an entry matches nothing, so the list cannot go stale.

**True:** all three records hold no figure of any kind. The work that cleared
them did not clear the table, so the table named three records as carrying
figures they do not carry.

**The safeguard was not what the register said it was.** The check carries a
baseline of tolerated figures, and it is the baseline that fails when an entry
matches nothing. The baseline is empty. Nothing reads the table in the
register, so the table is prose like any other and always was.

**Evidence:**

```
grep -oE "[0-9]+(\.[0-9]+)? ?(percent|ms|ns|byte)" docs/adrs/*/adr-000[356]*.md
grep -cv "^#|^$" scripts/adr-volatile-baseline.txt
```

The first command finds nothing in any of the three records. The second
reports that the baseline holds no entry.

**Follows:** the table now says that no record holds a figure, and it names
this finding. **A claim that a list is checked is worth nothing unless the
reader can see what checks it.** This one named the check in a footnote, and
the footnote pointed at a file that holds a different list for a different
purpose. That is close enough to be convincing and far enough to be false.

**This is defect shape 1 with the roles reversed.** The usual shape is one
value in two places with nothing failing when they disagree. Here the second
place was a safeguard that did not cover the first, and the register asserted
that it did.

### FND-255 — The collapse measurement measured the cell count, because every unit holds the same need

**Believed:** counting the distinct pairs of level 1 cell and need against the
live unit count would say how far a choice pass collapses if it decided for
each cell rather than for each unit.

**True:** the count returned a pair count exactly equal to the cell count, at
every bucket width including the exact need, under both placement patterns.
**The need column holds one value for all 1,000,000 units.** It is 65536,
which is one in the fixed point scale, and it is the value a unit spawns with.

The measured world holds no settlement, so no unit has a home to draw from and
consumption never moves a need. The measurement therefore reports the units
for each cell and nothing about the need.

**What it does establish.** The geometry gives 14,970 occupied cells of 16,384
for 1,000,000 units at the density the project states, so **66.8 is a ceiling
on the collapse factor** and no need distribution can beat it. The packed
figure of 740.2 is a property of a fixture that puts the whole population into
8 percent of the cells.

**What decides the real answer.** A cell holds 64 units at the median under
the scattered pattern. The distinct pairs in a cell are the smaller of the
units in it and the need values they take, so the collapse in the median cell
is about 64 divided by the number of need buckets. At 4 buckets it is 16, at
16 buckets it is 4, and at 64 buckets it is 1. **The need is a Q16.16 quantity
and takes about four thousand million values, so unbucketed the collapse is 1
and the rule buys nothing.** The bucket width is the mechanism, not a detail.

**Evidence:** the register holds every row, both placements and all six bucket
widths.[^F222] The counts come through the public crate interface, and the
engine gained no instrumentation: the live units, the tile of a unit, the need
of a unit and the block layout are all public. The pairs pack into one word
and the count is a sort and a scan, so no hash iteration order reaches the
result.

**Follows:** the record must not carry a collapse figure from this run. The
number it needs is how many need values coexist in one cell in a world that
consumes, and no fixture in this project produces one, because that needs
settlements, home sites and a running economy. **A ceiling of 66.8 and an
unmeasured floor of 1 is what this run supports.**

**This is the third fixture defect this benchmark has produced in one
session.** The first packed the population into a band, the second placed 76
percent of the units it was asked for, and this one measured a column that
never changes. Each was found by looking at a number that was too clean.

### FND-247 — A comparison that isolated one variable had two, and it was discarded

**Believed:** the first placement comparison answered whether the fixture
biased every unit figure. It ran the same frame under a packed and a scattered
population and reported that the scattered one cost 86 percent more.

**True:** it placed 762,599 units under the scattered pattern against
1,000,000 under the packed one. The two rows differed in their population as
well as in their placement, so a comparison built to isolate one variable had
two, and neither row explained the other. **The result was discarded rather
than reported with a caveat.**

The cause was in the fixture and not in the engine. The scattered pattern
walks the world at a stride and searched one stride for a tile that admits a
unit. A stride at the target scale is sixteen tiles, and a run of sixteen
tiles of water ends that unit's search while every later unit keeps its own
target, so the shortfall accumulates. The cursor now never goes backwards and
never stops early, so water costs a longer walk rather than a lost unit.

**Evidence:** the discarded run reported 762,599 in its unit column, which is
where the defect showed. The repaired run reports 1,000,000 under both
patterns, and the corrected result is a 60 percent difference rather than 86.

**Follows:** a caveat is not a substitute for a second run when the second run
costs three cents and four minutes. The discarded numbers were in the right
direction and would have supported the same conclusion, which is exactly why
reporting them would have been wrong: **a result that happens to point the
right way is not evidence, and publishing it teaches the reader that the
apparatus was sound when it was not.**

**The unit column is what caught it**, and it was in the output only because
the benchmark reports what it placed rather than what it was asked for. A
fixture that reports its request rather than its result cannot show this
class of defect at all.

### FND-248 — A guard that fires correctly is not evidence that the thing it guarded against was acceptable

**Believed:** the teardown trap makes a launch safe to attempt, because a run
that goes wrong terminates itself.

**True:** the trap works, and it is not a licence. One command chained a source
edit and an instance launch. The edit failed its assertion and wrote nothing.
The launch went ahead against a benchmark mode that did not exist, so the
instance built the tree, ran a sweep nobody asked for, and billed for it.

The trap did its job. An interrupt terminated the instance, deleted the key
pair and the security group, and the region was verified empty afterwards.
**That is the good news and it is not the finding.**

**Evidence:** the launch and its teardown are in the run log, and the commit
that repaired the sequencing describes it.

**Follows:** **an edit and a launch do not belong in one command.** The launch
must depend on the edit having succeeded, and a shell that runs the next
command after a failed one gives no such dependency. Separate them, or make
the launch conditional on the edit's exit status.

**The general shape.** A guard exists because the guarded action can go wrong.
Once the guard is trusted, the action gets attempted more casually, and the
guard is then load-bearing for cases it was never designed to cover. The trap
was written for a build that fails and a connection that drops. It was not
written for a launch that should never have happened, and it happened to
cover that too. **Next time it may not.**

### FND-249 — Three kinds of work, three kinds of scaling, measured on one machine

**Believed:** the exit field derivation would either scale like the tile pass
or floor like the unit passes, and which one it did would say whether the
lattice claim survives its first instance.

**True:** it does neither, and the reason was structural and readable before
the run. The derivation takes no thread count, and its own documentation says
the pass runs on the calling thread. A prediction saying so was written into
the register and committed before the run, with a hit defined as a speedup
inside 0.9 to 1.1 and a cost under 10 milliseconds.

**All three predictions hold.** The derivation costs 2.15 milliseconds at 1,
2 and 12 threads, flat to three figures, which is 132 ns for each of 16,384
cells. The level 1 rebuild beside it reaches 11.59 times on 12 threads, which
is 0.97 of the machine and the best scaling measured anywhere in this project.

**The result that matters is the comparison, not any one row.** Three kinds of
work, on one machine at one extent:

| Work follows | Speedup on 12 threads |
|---|---|
| The cells | 1.00, and it does not need threads |
| The tiles | 11.59 |
| The population | 2.08 packed, 2.32 scattered |

**Evidence:** the register holds the prediction, the rows and both
commits.[^F222] The derivation was measured directly rather than by a
difference, because the field, its constructor and the level it reads are all
public. The engine gained no instrumentation. The units were scattered,
because a packed population would flatter a per-cell claim in the direction
the record wants to hear.

**Follows:** work that follows the lattice is small enough not to need
threads, and work that follows the tiles takes them almost perfectly. **Work
that follows the population barely takes them at all**, and that is the half
of the frame the project cannot currently reduce. The lattice claim is
supported by its first instance.

**One caution against reading this as a general law.** The exit field is flat
because it is small, not because per-cell work cannot be large. A per-cell
pass over 16,384 cells that did much more for each cell would want threads and
would not have them, because the derivation takes no thread count and nothing
would notice.

### FND-261 — A count of what one layer asked for cannot see a generation below it

**Believed.** The drawing generated the ground of every visible tile twice,
and one of the two answers was a value it already held.[^F209C] A count of the
grounds the drawing asks for would say whether the repair worked.

**True.** The count says half of it. The counter stands in the drawing, so it
counts the calls that the drawing makes into the core. It cannot see a
generation that a reader below the drawing runs, and the second generation was
exactly that: a stock reader that started from the address and generated the
ground again to answer.

A contributor can therefore put the defect back and no test fails. The drawing
would call the reader that starts from the address alone, the picture would
not change, and the count of grounds the drawing asked for would still equal
the count of painted tiles.

**Evidence.** Two defects were put back, one at a time, against the tests that
claim the rule. A second generation written into the drawing was caught,
because both calls increment the counter. A second generation written into the
reader below was caught only by a test in the core crate, which gives the
reader a ground the address does not carry and asserts that the answer follows
the argument. No test in the drawing saw it.

A third defect was put back and caught nothing. A test named "the count
follows the window and not the world" was written first as a sweep over the
zoom steps. The drawing was then made to walk every column of every visible
row. The count still equalled the count of painted tiles, and the count still
fell at each step in, because the rows were still held to the window. **The
test measured less than its name claimed.** The test that replaced it draws one
window at one tile size over two worlds of different sizes and requires the
same count, which is the shape an existing test of the holder count already
uses.[^F261B]

**Follows.** **Put the defect back at each layer, not at the layer you were
thinking about.** A count is evidence about the layer that holds the counter,
and about no layer below it.

**Name a test for what it proves, not for the property you want.** The test
above was correct in every assertion it made and wrong in its name, and a
reader would have taken the name as the guarantee.

**A count of the generations that one frame runs does not exist, and an item
holds it.**[^F261C] Until it does, the claim that the ground is generated once
rests on two tests in two crates and on reading the one call site.

### FND-257 — A check that searches for a moved path must first ask whether the path is a name

**Believed:** the moved-path rule of the merge-defect check was ready for a
merge. It reads the paths the change moved away from, searches the tree for
each one, and reports every place that still names an old path. The rule is
sound and the check passed every fixture and every real change until now.

**True:** the rule searched for a path called `0` and reported 14571 failures,
all of them false. The search is a fixed-string search. A path of one
character is a substring of ordinary text, so the search matched every version
field of a lock file. The check blocked a merge that held no moved-path defect
at all.

**The evidence is the run.** The merge of the benchmark branch deleted three
files whose names are `0`, `40` and `600`. They are image dumps that a render
script wrote to a numeric file name and that a commit then captured. The gate
reported 14571 failures and named the lock file on every line.

**What follows.** A search for a name needs two conditions that a fixed-string
search does not carry. The name must be distinctive enough that a match means
a reference, and the match must stand on its own rather than sit inside a
longer name. The check now asks both.

**The two conditions do different work, and the measurement says which does
which.** The first repair asked both at once, and the tidier reading was that
the second condition carried it. That reading is wrong. Over the tree at the
merge, a fixed-string search for `0` matches 14588 lines. The
stands-on-its-own condition alone leaves 2066 of them, because prose holds
sentences such as "level 0 holds individual tiles". It annihilates the lock
file only because a lock file holds no prose. **The distinctive-name condition
is the one that carries this defect, and anyone who simplifies it away
reopens it.**

**The first form of the distinctive-name condition was too wide.** It searched
for a path only when the path held a directory separator or a file extension.
That rejects the bare number and it also rejects `justfile`, which 16 lines of
this project name and which no prose holds by accident. A move of it reported
nothing and the gate passed. The condition now asks the narrower question:
can ordinary prose hold this token? A bare number can, a word cannot, so the
check skips a path whose last segment is all digits and searches every other
path. A bare number one directory down is the same defect and is skipped for
the same reason.

**A skipped path is reported, not dropped.** A moved file the check does not
search for is still a moved file, and the check prints a note for it. The note
says why the search did not run, so a reader can search by hand. A residual
gap remains for a name that is neither a number nor distinctive, such as a
one-letter file at the root. It is stated rather than closed, because no such
file exists and an escape written before a real instance is a capability
nobody invokes.[^37]

**A match now survives a full stop that ends a sentence.** A full stop
continues a name in `docs/a.md.bak` and closes one in "the detail is in
docs/a.md." The two are told apart by the character after the stop. Prose that
names a path outside a code span breaks the documentation rule, and it is
therefore the prose most likely to be stale.

**The rule now has a probe, and it did not before.** Each condition above was
put back as a defect, one at a time, and the probe was run against it. The
condition that shipped first fails exactly the case built for `justfile`. A
probe is the thing that would have found this hole without anyone reasoning
about it, and its absence is why two repairs were needed rather than one.

**The shape.** This is the inert-capability shape inverted. The rule was not
inert; it ran, and it ran on the first input that was outside the distribution
its fixtures held. A fixture built from citation-shaped paths supplies no
one-character name, so the assertion never received the input that would fail
it.[^84] The defect lived at an extreme of the distribution, and the
fixture modelled the typical case.

**The cost of finding it was one gate, and then one review.** The check is
nine hours old, this was its first live merge, and its first repair carried a
second instance of the same shape.[^F257B] Both instances are one shape: a
rule was narrowed against the input in front of it, and nothing measured what
the narrowing cost on the inputs that were not.


### FND-256 — The fixture of a benchmark is its least reviewed code and its most load-bearing

**Believed:** the benchmark measured the engine. Its figures were reviewed, its
method was stated, and its rows carried the machine, the commit and the date.

**True:** three of its figures described the fixture rather than the engine,
and all three were found in one session. None was found by reading the
benchmark. Each was found by distrusting a number that looked too clean.

| The defect | What it made false | How it showed |
|---|---|---|
| The population filled the first open tiles, packing 1,000,000 units into a band at one unit for each tile | Every unit figure, by about a factor of two | A reviewer asked what density the pattern gave |
| The scattered pattern gave up at water and placed 762,599 units of 1,000,000 | A comparison built to isolate the placement | The unit column in the output |
| The need column held one value for every unit, because no unit had a home to draw from | The whole collapse measurement | A pair count exactly equal to a cell count |

**The transferable claim.** A benchmark's fixture is the least reviewed code
in a project and the most load-bearing for every figure the benchmark
produces. The measurement code gets read, because a figure is what people
argue about. The world the measurement runs on gets written once, early, by
whoever wanted a number, and then every later figure inherits it.

**Evidence:** the register holds all three, with the corrected figures beside
the ones they replaced.[^F222] FND-245 holds the placement, FND-247 holds the
discarded comparison, and FND-255 holds the constant need column.

**Follows:** the register now names the fixture beside every table, and its
format rule demands the fixture as well as the machine. **A figure whose
fixture is not named is not reproducible**, and that is a stronger statement
than the one this project already makes about the machine.

**Three tests, in the order they cost.** Ask what distribution the measurement
needs, before writing the world. Make the fixture report what it produced
rather than what it was asked for, because that is what caught the second
defect. Distrust a number that is too round, too flat or exactly equal to
another number, because that is what caught the first and the third.

**The testing rule already said the first of those**, and it says it about
tests rather than about benchmarks.[^23] Nobody applied it here, because a
benchmark is not a test and the rule did not say the word. The rule is right
and its scope was too narrow.

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

### FND-309 — Four instruments in one night reported a figure about something other than what they measured, and not one of them said so

**One of the four instances below is not recorded anywhere yet.** The first is
FND-297, which the trunk holds and which this entry now cites.[^F314X] The
fourth was reported and is being written by the worker who found it, so it is
cited by description and owes a number. The citation check refuses a number the
register does not hold, which is how the debt was noticed rather than shipped.

**Believed.** An instrument that finishes and prints a number has measured the
thing it was pointed at. A run that fails is visible, so a run that reports is
a run that worked.

**True.** Four instruments broke that in one night. Each returned a figure a
reader could not tell from a valid one. In every case the invalid run was
cheaper, cleaner or steadier than the valid one, so the wrong answer was the
one a tired reader wants.

| The instrument | What the figure described | What caught it |
|---|---|---|
| The local benchmark | A binary built two hours earlier | A probe printed nothing when it should have printed a line for each frame |
| The gate timing harness | A gate that stopped early, priced as a gate that ran | Nothing. It was read out of the code before it cost a run |
| The gate timing harness | A tree that changed under the run, because three branches merged into it | A person read the log |
| The stage cost measurement | The code layout, not the change | Four stages the change never touched moved, and moved reproducibly |

**Each was caught by a control outside the instrument, and three of those
controls existed by accident.** The probe of the first was written for another
purpose.[^309A] The moved stages of the fourth were nobody's control. The third
was caught by a person reading a log at the right moment. Only the second was
found on purpose, and it was found by reading rather than by running.

**The second and third are the same instrument, and the interval between them
is the point.** The stopped-gate hazard was read out of the code and repaired
before it cost anything. The repaired instrument then met the tree-move hazard
on its first live run. An apparatus meets its own failure modes sooner than
anyone expects, and the repair paid for itself the same night.

**The fourth instance carries the sharpest form of the rule.** This project
builds with link-time optimisation and one code generation unit, so a change to
any file relays the whole binary. One fixed tree measured twice gave 36.46 and
29.07 milliseconds for one stage, a span of 25 percent, and one of those runs
sat below both runs of the tree it was compared against. The same stage on the
base tree repeats to 0.7 percent. Four stages the change never touched moved,
and the compute-bound stages least sensitive to layout barely moved. So a
single binary against a single binary cannot separate the change from the code
layout, and every performance figure taken that night carries an unquantified
layout term of up to about 8 percent.[^309B]

**The discriminator is worth more than the figure.** A pair of runs is evidence
only when the things that should not have moved did not. That test is what
turned a plausible story about a slow memory host into a refuted one, and the
worker refuted its own explanation rather than letting it stand.

**Follows.** Three things.

**An instrument must carry its own control, and the control must fail loudly.**
Two were added to the gate timing harness. It now names every command line that
failed and says in those words that the run is not a measurement, so a gate
that stopped cannot read as a gate that is cheap. It also reads the commit
before and after the run and reports when the two differ, so a table cannot
describe a tree that never existed.[^309C]

**State the discriminator beside the figure.** A figure with no stated way of
being wrong is not a measurement, and the reader cannot supply one later. Every
row above was saved by a discriminator, and three of those were luck.

**A discipline is not a control.** The answer to a merge landing mid-run is not
a rule that nobody merges during a run. That rule lives in one person's head at
the end of a long night with four things moving, and this register exists
because facts kept in heads do not survive. The answer is that the instrument
reads the commit and says when it moved.

**This is the same shape the testing rule states for a fixture, one level
up.**[^309D] A fixture that supplies no extreme measures itself. An instrument
with no control measures itself. In both the test passes, the number arrives,
and nothing says which question was answered.

### FND-330 — A stale frame figure sat in a blocker row and in a register, and nothing failed

**Believed.** A frame at the target scale costs 500 milliseconds on the target
platform. The blocker row about derived cost figures said so in its own words,
and two sections of the target platform register concluded from it that a frame
cannot fall below about 300 milliseconds on any core count.[^F222] [^28]

**True.** The frame is 177.9 milliseconds at the target scale, on 12 threads,
on a world whose units are scattered. Two later runs of a changed tree gave 167
to 169 milliseconds. The register holds every row with its machine, its commit
and its fixture.[^F222]

**The two figures do not measure one thing, and that is part of the finding.**
The 500 millisecond figure came from the first run, on 16 threads, on a world
whose units were packed. A packed unit costs about half a scattered one, so the
later figure is better than the difference between the two numbers shows. **A
figure without its fixture is not comparable to another figure**, and this
register already states that rule for its own rows.[^F222]

**Evidence.** The stage tables in the target platform register, taken on 3
September 2026 on `c7g.4xlarge` instances. The 500 millisecond figure entered
the blocker row on the same date, from the first two runs of that day.

**What follows. The register moved eleven times in one day and the prose did
not move with it.** Every stage table carries a line that supersedes the tables
above it. No such line reaches a document outside the register. The blocker row
held the figure as its own sentence rather than as a citation, so no check could
see it go stale, and a priority row carried an 836 millisecond frame as the
denominator of a share for the same reason.

**This strengthens the case for two open items and does not change either.**
One item adds a check that fails when a document states a register in its own
words.[^F258C] This is a fourth instance of that shape, and the first where the
stale copy sits in a register row rather than in a report. The other item makes
a check compare the two project orientation files.[^F259CHECK] It is the same shape
at a smaller radius, and this instance says the family of phrases the first
check reads must include a frame figure, not only the state of measurement.

**A blocker row is not a place for a figure.** The scope rule states this for a
decision record, and the reason is the same here: a row that quotes a number
must be edited when the number moves, and nothing fails when nobody edits
it.[^12] The row now cites the register and states no frame figure.

## E. Layout and platform corrections

### FND-277 — The residual was one pass nobody had named, and it was 61 percent of the frame

**Believed.** The 170 milliseconds the stage split could not divide held
several passes in roughly comparable parts. The register named six of them:
the movement intents, admission, the holder spread, the death scan, the part
of the level 1 rebuild that reads the units, and the walk over live units
inside the choice pass.[^FND277A]

**True.** It is one pass, and inside that pass it is one function. With every
stage of a frame named and timed, the holding spread is 514.3 milliseconds of
a 836 millisecond frame, which is 61.5 percent of the whole frame. The three
largest stages together are 85.3 percent. The death scan is 0.04
milliseconds, admission is 29.3, and the movement intents are 4.7.

**Inside the spread, the candidate list is 49.1 percent of the whole frame**,
and it runs on the calling thread. It walks every held tile and every live
unit, pushes an index for each, then sorts several million indices and removes
the duplicates. The half of the pass that decides takes a thread count and
costs 71.1 milliseconds. The half that chooses what to decide about takes none
and costs five and a half times as much.

**Evidence.** Two runs on a Graviton3 instance at 16,777,216 tiles, 1,000,000
units scattered, 12 threads. The first named every stage, and the second
divided the largest one. Every stage is a row and the register holds
them.[^FND277A] The sum of the stages is 835,957,085 nanoseconds against
835,978,143 for the same frames timed from outside, so the part that is still
unattributed is 0.0025 percent.

The two runs measured the frame at 836.0 and 816.1 milliseconds under the same
setting, so they differ by 2.4 percent. Read a share here as a proportion and
not as an amount.

**Follows.** Four things.

**A split by subtraction finds only what has a switch.** The old method priced
a pass by running a frame without it, and the holding spread has no switch, so
the largest thing in the frame was invisible to the method that existed to
find it. The residual was not a mixture. It was one pass that the instrument
could not point at.

**The next optimisation is not the one the backlog names.** Four items propose
a change to the layout or the allocation, and the priority index put them
above the item that made the frame measurable. The measurement moved the
order: one pass is worth more than the four items together.

**Prices go stale, and one in the register was stale by a factor of 125.** The
same split measured the choice at 71.4 milliseconds and called it 26 percent
of the cost of a unit. The choice now costs 0.571 milliseconds, because item
0238 made the pass decide once for each pair of cell and need. Nothing failed
when that figure went stale, and nothing would have.

**One measurement is enough to name a pass and not enough to name the part of
it that costs.** The first run said the spread was 61 percent. It took a
second run, with three spans inside the spread, to say that one serial
function is 49 percent of the frame. A stage is the unit that a reader can
act on, and this one needed two.

**This row is history, and the code no longer matches its present tense.**
Backlog item 0291 replaced the list and the sort with a bit plane over the
tiles, and gave the pass a thread count. The pass costs 16.7 milliseconds
instead of 400.9, and a frame costs 463.4 milliseconds instead of
825.4.[^FND277A] Two findings hold what that work corrected, and one of them
shows that the figure this row rests on was read from the wrong
world.[^F277B] [^F277C]

### FND-278 — Huge pages are worth 3.9 percent, and they cost five times the memory the estimate gave

**Believed.** Huge pages might explain part of the unattributed cost, and the
memory they waste at the end of an allocation would be under one part in two
hundred.[^FND278A]

**True.** Both halves are answered, and the second was wrong. A frame at the
target scale costs 3.9 percent less with the kernel giving 2 MB pages, and the
resident set grows by 2.65 percent rather than by 0.5.

**Evidence.** One run on a Graviton3 instance, one commit, one binary, three
processes, one for each kernel setting. The register holds every row.[^FND277A]
The frame fell from 835,978,143 nanoseconds under the default setting to
803,042,781 under `always`. Of the resident set, 719,323,136 bytes sat on huge
pages under `always` and none did under either other setting, so the setting
demonstrably reached the process. The resident set grew by 27,889,664 bytes.

**Follows.** Three things.

**The prediction named the shape and the shape held.** The item said a
translation cost would appear spread across every pass that touches a large
array, and invisible to a split that measures stages. Three stages account for
the whole 32.9 milliseconds and every other stage moved by less than half a
millisecond. The three are the passes that write scattered over a large array.

**A time row without an occupancy row is a claim rather than a
measurement.** Two of the three settings gave this engine identical pages,
because the engine calls no advice. Only the huge page column separates "the
setting did nothing" from "huge pages do nothing", and they are different
answers.

**Two conditions that should be identical differed by 0.64 percent**, so that
is the noise floor of this apparatus at one run for each condition. A result
this size is worth quoting to one figure and not to two.


### FND-235 — A stored watermark read nothing, because a sentinel already said it

**Believed.** The record of descent needed a stored count of how many rows the
Euler labels covered. A query about a row above that count would answer
nothing rather than answering from a stale label.

**True.** The count read nothing. The two label columns already mark an
unlabelled row with an absent-value sentinel, which a birth writes and a
relabel clears. The sentinel alone decides every answer. The stored count was
a second declaration of the same fact, and the length of the Euler order was a
third.

**Evidence.** The watermark check was removed from the label reader and the
whole suite stayed green: twelve tests, none of which could see the
difference. The behaviour was covered. The declaration was redundant, and only
removing it showed that.

**Follows.** Two things.

**A green suite after a removal is evidence about the removal, not about the
test.** The suite proved the watermark unnecessary. It would equally have
proved a necessary check unnecessary if no test reached the case, which is why
the removal was paired with a test that does reach it: a character born after
the relabel must answer nothing, and that test fails when the sentinel is
wrong.

**Count the declaration sites of a derived quantity before storing it.** The
number of labelled rows is derivable from the Euler order length, and the
label columns already answer per row. Neither needed a third site. The record
now derives the count and stores nothing.[^235A]

### FND-236 — The descent columns went to the record, not to the arena

**Believed.** The backlog item said to add the descent columns to the
character arena: the two parent edges, the house, and the two Euler
labels.[^236A]

**True.** Three of the five belong to the record of descent, and two were
already there. Item 0067 had put the parent edges in a separate append-only
record keyed on a descent identity rather than on a slot, and it gave the
reason: the arena reuses a slot after a death, so a slot names a different
character later.[^236B]

The same reason governs the other three. A house on a slot column would be
released when its holder died, so a dynasty would lose its founder and every
dead member of its line. A Euler label on a slot column would be worse, since
the father forest holds every character the world ever created and the labels
must cover all of them. The character arena holds a reader for each, and the
reader resolves the identity to a descent row.

**Evidence.** A test removes a character and asserts that the record of
descent still answers its house, and that a living descendant keeps the house
of the dead ancestor.

**Follows.** **A backlog item names the structure it expects, and the item may
be wrong about it.** This one was written before the work that built the
record, and it named the arena because that is where the columns sat when it
was written. Read an item's structure name as its author's expectation and not
as a constraint. A constraint lives in a record.[^16]

This is the same shape as the misattribution that ADR-0021 now forbids: a
sentence that names one home for a set of fields whose passes disagree about
where they belong.[^236D]

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

### FND-273 — The packed against scattered ratio measures the density, not the arena order

**Believed.** The unit arena holds units in spawn order, nothing reorders it,
and that is why a unit costs 2.11 times as much scattered as packed at 12
threads. The backlog item took the ratio as the size of the prize for
reordering the arena by cell.[^F273A] [^F273B]

**True.** The ratio measures the placement of the units over the world. Both
fixtures spawn in ascending tile order, so the arena is in cell order in both
rows. One puts the units on consecutive tiles and the other puts them one in
every seventeen. **A reorder of the arena cannot recover any part of that
ratio, because the row that pays it already has the order the reorder would
produce.**

**Evidence.** A third fixture puts the units on the same tiles as the
scattered row and spawns them in a permuted order, so the population is fixed
and the arena order is the only thing that moves. The mean slot distance
between two units next to each other in cell order is 108 in the ascending row
and 83,269 in the permuted one, which is the decorrelation the item
describes.[^F273C] The unit cost rises by 1.45 at one thread and 1.34 at four,
not by 2.11. Both rows come from one process on one machine. The machine is a
development x86-64 machine that other work shared, so the figures bound the
shape of the effect and are not evidence about the target platform.[^28]

**Follows.** Two things.

**A benchmark that moves two variables together cannot price either.** The
placement rows change the density and the arena order in one step, and the
arena order happens not to move at all. The row that separates them is the one
the decision needed.

**The prize is real and it is smaller than the item claimed.** A drifted arena
costs 1.24 to 1.45 on the unit half rather than 2.11, and only the part of
that which survives the walk order is what a physical reorder would
buy.[^F274A]

### FND-274 — The unit pass is bound by the tile side, so the walk order costs more than the arena order

**Believed.** A pass over the units of one cell is slow because the unit
columns of those units sit far apart, and the fix is to move the units in the
arena.[^F273A]

**True.** The movement pass walks the live units and reads five things for
each one. Four of them are on the tile side of the world: the exit of its
cell, the address of its tile, the ground of its target, and the address of
that target. One is on the unit side. **The tile side is the larger footprint
by an order of magnitude, and the order that makes it ascending is the order
of the walk, not the order of the arena.**

Walking the arena in slot order reads the tile side at random once the arena
has drifted. Walking in cell order reads it in ascending tile order whatever
the arena holds. The bridge already sorts every live unit on the tile key once
for each frame, at the barrier, so that order costs the frame nothing
more.[^F274B]

**Evidence, and what it does not cover.** The movement pass now walks the
bridge order. The golden state hash did not move, at any scenario, and the
whole suite stayed green.[^F274C]

**The claim that no result reads the walk order was checked by perturbation
and not by assertion.** The walk was reversed, and the golden hash still
matched the stored file. Admission sorts what it receives on a total key of
the target tile and the whole identity, so the same set in another order gives
the same answer.[^F274D]

**The size of the gain was not measured.** Four runs of the before and the
after build, at four threads, gave ratios of the drifted unit cost to the
packed one between 1.06 and 2.26, in both directions, with the two builds
swapping places between runs. The development machine ran between eleven and
fifty-seven runnable threads on sixteen cores while the runs took their
samples, and the load moved by a factor of four inside one run. **The spread
of the apparatus is larger than the effect it was measuring, so no figure here
states what the change bought.** The measurement that would state it belongs
on the target platform.[^28]

**What the change rests on instead is structural and checked.** The pass reads
four tile-side values for each unit. Walking in cell order makes those reads
ascending in the tile index, and walking a drifted arena in slot order makes
them random. That the penalty for a drifted arena exists at all is measured,
in two independent runs of the before build: 1.24 and 1.34 at four threads,
and 1.45 at one.

**Follows.** Two things.

**Ask what a pass reads before deciding where to move what it reads.** The
item proposed moving the units. The larger of the two footprints was on the
other side, and the order that fixes it was already built and thrown away
every frame.

**A change can be right and still unmeasured, and the report must say which.**
The residual that only a physical reorder removes is now separable from the
part the walk order removes, and both belong on a target-platform run before
anyone spends a refactor on them.[^F274A]
### FND-285 — The held ground of the demonstration world is 140 times smaller than the held ground of the benchmark world

**Believed:** the holding covers a few tens of thousands of tiles. The register
holds the figure. The demonstration world, founded for four factions with a
group each, holds 7,866 tiles at tick 50 and 46,992 at tick 200.[^F285A] That
figure was read as the size of a holding, and work on the candidate pass was
planned from it: at 46,992 held tiles the pass would sort about 1.3 million
indices, of which one million come from the units.

**True:** the benchmark world holds 6,615,358 tiles after nine frames. That is
39 percent of the world and 141 times the demonstration figure. The raw
candidate list reaches 14,884,176 entries, of which 13.9 million come from the
held tiles and one million from the units. **The units are 6.7 percent of the
list, not 77 percent.**

**The measurement.** A probe inside the candidate pass, printing the held
count and the list length for each frame. Machine: the development machine,
not the target platform, because the two counts are properties of the
simulation and not of the processor. 16,777,216 tiles, 1,000,000 units
scattered, 12 threads, ten frames.

| Frame | Tiles held | Raw candidate entries | Distinct candidates |
|---|---|---|---|
| 1 | 0 | 1,000,000 | 998,551 |
| 2 | 998,551 | 7,984,285 | 2,616,812 |
| 5 | 3,218,758 | 12,724,736 | 4,355,868 |
| 10 | 6,615,358 | 14,884,176 | 4,821,144 |

**The cause is the population, and it is a fixture difference and not a
defect.** The demonstration places 192 units on 16,777,216 tiles. The
benchmark places 1,000,000 scattered. A unit takes the ground it stands on
when nobody holds it, so the held ground grows with the units and then spreads
from every seed at once. The two worlds are the same rule at two densities.

**Follows.** Three things.

**Do not carry a count from the demonstration world into a statement about the
target scale.** The demonstration is built to look right, and the testing rule
already says that a fixture chosen to look right supplies no extreme.[^84]
The two worlds differ by four orders of magnitude in the population, and every
derived quantity differs with it.

**A figure in the register names the world it was measured in.** The row above
does. The reading of it did not, and the reading is what reached a plan.

**The unit walk was the wrong thing to remove.** The plan that came from the
small figure proposed dropping the walk over the live units, because it looked
like three quarters of the work. It is 6.7 percent. The sort was 68 percent
and the walk over the held tiles was 31 percent, and both follow the held
ground rather than the population.

### FND-286 — A pass that allocates on every frame cost twelve milliseconds that no stage measures, and the mapping count is not the cause

**Believed:** the cost of a buffer that a parallel stage allocates for each
thread falls inside the stage that allocates it. The stage table would
therefore see it.

**True:** twelve milliseconds of it fall outside every stage. The candidate
pass began allocating a bit plane of 2,097,152 bytes for each of twelve
threads on every frame. The residual of the frame, which is the part no span
measures, went from 22,202 nanoseconds to 12,196,680. Nothing else changed
between the two runs.

**Then the project believed the mapping count was the cause**, because giving
back a mapping a thread has written reaches every core. **That is refuted.**
Twelve mappings became one array of twelve chunks, which is one allocation
instead of twelve, and the residual measured 11,739,029 nanoseconds. The two
figures are the same to within the spread of this apparatus.

**The measurement.** Machine C, `c7g.4xlarge`, Graviton3, 16,777,216 tiles,
1,000,000 units scattered, 12 threads, nine frames, `stage-cost` feature.
Three runs at one base commit, differing only in the tree.[^F286A]

| The pass allocates | Residual, ns | Share of the frame |
|---|---|---|
| Nothing. It sorts a list | 22,202 | 0.0027 percent |
| One plane for each thread | 12,196,680 | 2.74 percent |
| One array of twelve chunks | 11,739,029 | 2.53 percent |

**The cause is not identified, and this row says so rather than guessing
again.** What is established is that the residual follows the allocation and
not the number of mappings.

**Follows.** Two things.

**The plane should be held across frames rather than allocated in each one.**
That is the change the evidence points at, and it is not made. It asks a
design question the pass does not answer today: the holding is copied by a
derive, and a buffer it holds would be copied with it, so the buffer needs a
statement about what a copy of a holding means.

**A saving of 384 milliseconds bought a cost of 12.** The trade is good and the
cost is recorded so that it is not found again as a mystery.

**This row's conclusion is wrong, and a later row corrects it.** The residual
did not follow the allocation. The pass still allocates and the residual is back
under half a millisecond.[^F277C]


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

### FND-239 — All four merge defects were already checked. What was missing was the timing

**Believed.** A dispatcher produced four defects by hand in one day and none
was caught until the gate ran, so a check for them did not exist.

**True.** Every one of the four was already checked. The register check fails
both on a number that names two entries and on a next-number line that
disagrees with its own entries, which is two of the four.[^32] The footnote
check fails on a label a document defines twice, which is the third.[^239A]
The citation check fails on a footnote naming a path that does not resolve,
which is the fourth.[^148]

The four were not undetected. They were detected minutes later, over the whole
tree, after the commit existed.

**Evidence.** Each rule was found by reading the three scripts before writing
a new one, and each was confirmed by staging the defect and watching the
existing check fail. The four checks cost 1.0, 3.7, 7.7 and 4.0 seconds on a
development machine, which is why they run in the gate and not at the commit.

**Follows.** Three things.

**Before writing a check, ask whether the rule is already checked and only the
timing is wrong.** A check written on the assumption of a gap would have
restated three rules that already had a home. Two copies of a rule drift, and
nothing fails when they disagree.[^22] The check that was written imports the
other two and calls them.

**A slow check and a missing check fail the same way from the outside.** Both
let the defect reach the commit. Whoever watches four defects survive until
the gate reasonably concludes that nothing is watching. The distinction
matters because the remedies differ: one needs a rule, the other needs a place
to run an existing one.

**One rule was genuinely new, and it is the narrow one.** No existing check
ties a move to the citations of the path it moved from. The citation check
overlaps without covering it, because it reads footnote paths under `docs/`,
so a moved crate file named in a comment passes it. Scoping the new rule to
the paths the change moves is what makes it both new and cheap.

**A fifth instance arrived while the work was in progress, and nobody arranged
it.** Merging the trunk into the branch that carries this check left four rows
of the records priority index listed twice: one side had rewritten the table
and the auto-merge kept both. The priority check caught it.[^239B] That is the
same shape as a register naming one number twice, in a file that is not a
register, found by the same mechanism the finding describes. The count of
these defects is not four and it is not five. **Do not write the count down.
Write the shape: a merge that keeps both sides of a table that one side
rewrote.**


### FND-238 — The gate prints ten screens of red when it passes

**Believed.** A reader can tell a passing gate run from a failing one by
reading its output. Red means something is wrong.

**True.** A fully green `just check` prints ten test binaries reporting
`FAILED` and ten `error: test failed` lines. They come from the probe
recipes, whose passing condition is a non-zero exit, so the recipe marks each
one and the gate exits zero with all of them red.[^238A] The probes exist for
a good reason: a determinism test with no proven failure mode is
decoration.[^238B] The cost is not the probes. It is that the gate has spent
its whole vocabulary of alarm on its success path.

**Evidence.** A worker read the block as a determinism defect and
investigated it: decoded the event bytes, found two tile events in reverse
order, traced the emission site, and ran the suite in isolation and under
load. The investigation ended only when the reversed order turned out to be
*exactly* reverse, which is what the probe feature injects and not what a
thread-order defect produces. The dispatcher reports reading past the same
block four times in one day, and recognising it only from having hit it
early.

**Follows.** Three things.

**A check whose success prints as failure has no alarm left for a real
failure.** The reader who has learnt to skim ten `FAILED` lines will skim the
eleventh. This is the same shape as a gate left broken because it cannot
pass: both train everyone to ignore the pipeline.[^238C]

**The probe output is convincing, not merely noisy.** A reader who does not
skim it loses time instead, because the injected defect is a plausible one by
construction. The probe reverses the combine order, and reversed order is
exactly what a result ordered by thread completion would look like.

**What distinguished the probe from a real defect was the exactness.** A
result taken from thread completion order varies between runs. A result that
is exactly reversed, every time, on an idle machine and a loaded one, is a
deliberate switch. Record that as the test to apply next time, because it is
cheaper than decoding the bytes.

This finding proposes no fix. The shape is the finding.
### FND-223 — A sentence about the missing measurement reached ninety documents

**Believed:** no measurement exists on the target platform. About ninety
documents in this tree state that sentence in their own words, in a product
record, in an accepted decision record, in a review, in a completed backlog
item, in two reference registers, in the project orientation and in a doc
comment in the engine.

**True:** the sentence was true when each document was written. It stopped
being true on 3 September 2026, when a benchmark ran on a Graviton instance
and measured four operations. The blocker narrowed on the same day and did not
close, so the sentence is wrong in the general case and right about most
individual figures.

**Evidence:** the search that found the sites, and the register that holds the
figures.[^F222] [^28]

```
grep -rniE "no (measurement|benchmark)|nobody has measured|not been measured" --include="*.md" --include="*.rs" .
```

**Follows:** the sweep was not made, and the reason is not neglect. An
accepted record does not change except in status, so repairing one needs a
record that supersedes it.[^F223C] A review and a completed backlog item are
records of a moment and are correct as written. The three documents that guide
work today were repaired: the project orientation, the target register and the
local register. Everything else cites the blocker register, and that row is
now the current statement.

**This is the shape FND-042 names, at the largest scale the project has seen.**
A blocker that narrows leaves a false sentence in every document that stated
the blocker in its own words. Nothing fails, because a document is prose. The
defence is not a sweep. The defence is that a document states the blocker by
citation and never in its own words.


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

### FND-146 — A check across a boundary can be a tautology that reads as a cross-check

**Believed.** A test that compares two views of one thing is a cross-check. If
the engine hands a caller the same log twice, once as bytes and once as
columns, then comparing the two proves the views agree, and the test fails when
they stop agreeing.

**True.** A comparison proves something only when both sides can be computed
independently. A caller that holds one of the two views and no description of
the other cannot compute the second side, so it compares a value against
something derived from that same value. The comparison then holds for every
input and fails for none.

**Evidence.** A test of the new event columns compared the number of rows in a
column against the length of the raw byte buffer, to prove the columns describe
the same log the bytes do. Python holds no layout of an event, deliberately,
because holding one is the defect the work removes. It therefore cannot know
the width of an event. The assertion reduced to whether the byte length divides
by the event count, which is whether a length divides itself. **No change to
the layout, to the field order, or to the columns could have made it fail.**

The shape is worse than an ordinary weak test, because the tautology was
invisible. The test named two sources, read from both, and looked like the one
check that guards a second declaration site.

The same session produced a second instance in another subsystem: a terrain
assertion that read two constants and never exercised the rule between them.

**Follows.** Three things.

**A test that cannot fail is worse than no test.** It occupies the place a real
test would take and reports success forever. A missing test is visible to
anyone who looks for it.

**Before writing a comparison, name what computes each side.** If one side is
derived from the other, the test measures nothing. If the caller cannot compute
the second side at all, then the check belongs on the side of the boundary that
can, or it does not belong.

**A boundary drawn to remove a second declaration site removes the ability to
check that site from outside.** That is the cost of the boundary and it is
correct. Do not buy the check back by smuggling the layout into the test.

### FND-147 — An API that cannot say where to act invites the caller to sweep

**Believed.** The control plane rule is a matter of discipline. Python must not
loop over entities, so a caller that sweeps the world has ignored the rule, and
the repair is to write the caller properly.[^ORIENT2]

**True.** A caller sweeps when it has no other way to find the thing it must
act on. A rule that forbids a shape and offers no alternative is a wish. The
rule loses to the absence of a read every time, because the read is what the
caller needed and the rule is only prose.

**Evidence.** A test had to produce one gather event. The gather resolve grants
from the tile a unit stands on, and only when that tile holds the resource. The
control plane has no read that answers which tile holds a resource. The test
therefore put a unit on every open tile of a sixteen by sixteen world, ordered
each one to gather, and let the engine find the ground.

**The person who wrote that sweep had recorded the rule against sweeping in the
same change.** Discipline was not the missing part. The read was.

**The obvious repair is the same defect one layer out.** A per-tile call that
answers whether a tile holds a resource moves the sweep from units to tiles,
and the tile population is larger than the unit population. A caller that walks
the world asking a question one tile at a time is the data plane, whatever the
question is.

**A second instance, through the type system rather than through a missing
read.** The spawn verb returns the identities of the units it made, as one
column. The type stub for the verbs that take units declared a sequence of
integers, and a column is not a sequence of integers. A caller passing the
column straight to the next verb would have been told to convert it, and every
conversion of a mass-tier column is a loop.

**Make the boundary accept what the boundary produces.** If a verb returns a
column and the next verb refuses one, the API has instructed the caller to
sweep. The instruction arrives as a type error rather than as missing
functionality, which is why it is easy to satisfy in the wrong way.

**A red gate does not point at the right repair.** The type check would have
failed on the signature. A contributor could have satisfied it by writing a
list around the column at the call site, which passes the gate and puts the
loop back. The gate sees the narrow fault. It cannot see the consequence.

**Follows.** Four things.

**When a rule forbids a shape, check that the API offers the shape it wants
instead.** The design says the caller builds a selector and the engine resolves
it. A selector that names a place by a property is the missing piece here.
Until something like it exists, every caller that needs a place will sweep, and
each one will look like a discipline failure.

**When you find yourself sweeping, do not add discipline. Ask which read is
missing.** The sweep is a symptom and it names its own cause.

**Check that a verb accepts what the verb before it returns.** Two instances
here came from different mechanisms, one a missing read and one a type. Both
made the honest call site impossible, and a caller that cannot write the honest
call writes the sweep.

**A rule with no mechanism is worth recording as unenforced.** A reserved
registry row holds the claim that the API refuses the loop for a declared
tier.[^F147A] Nothing implements it. A reader who meets the rule and not the
gap concludes the project enforces something it does not.

### FND-148 — A second check downstream can make an upstream check untestable

**Believed.** A test that drives the public interface, watches a stale identity
refuse, and sees a typed error covers the resolution that refused it. If
resolution broke, the test would go red.

**True.** It goes red only when nothing else refuses first. Two independent
checks on one condition each stop the same defect, so removing either one
leaves the other to answer, and every test above them stays green. The tests
then measure the pair and cannot name which member did the work.

**Evidence.** The engine resolves an identity a caller hands back by comparing
the generation in the identity against the generation the arena holds. The
comparison was deleted and the suites were run.

Two Rust tests went red, and they are the two that assert on the refusal
itself. Four stayed green, correctly: two exercise conditions the deletion did
not touch, one resolves a live identity, and one reads the gather log.

**On the Python side the read stayed green and only the write verbs went red.**
Reading the tile of a dead unit still refused with the same typed error and the
same message, because the arena compares the generation a second time when it
reads a tile. The read is protected twice, so it cannot fail when one of the
two is removed.

The protocol client test stayed green for the same reason, and it asserts only
that the tool reported an error. It would have stayed green even without the
second check, because a refusal by any cause is still an error.

**Follows.** Three things.

**Defence in depth costs the ability to test each layer through the front
door.** The second check is right and stays. What must change is the claim: a
test above both checks demonstrates the behaviour and does not cover either
check on its own.

**Say which assertion caught the defect, in the test.** Both tests above now
record what the experiment measured, so a later reader does not take the read
as the coverage. A comment that names the measured result is worth more than
one that restates the intent.

**A test that asserts only that an error happened cannot tell one cause from
another.** That is enough for an integration test whose claim is that the
refusal reaches the caller. It is not enough for a test whose claim is that a
particular check refused, and the two read alike.

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

### FND-156 — The live filter of the household read cannot fail through the engine

**Believed.** The household read must skip a dead unit, and a test can prove
it by putting the defect back. A reader that walked every slot instead of
every live unit would put a dead unit into a household.

**True.** It would not. The unit arena clears the home of a slot when it frees
it, and the arena invariant states that a slot which is not live holds no
home. So a dead slot never names a dwelling, and the walk over every slot
gives the same answer as the walk over the live units. The filter is a second
guard against a case that a checked invariant already excludes.

**Evidence.** The filter was replaced by a walk over the whole home column,
rebuilding each identity from the slot and its generation. The whole household
suite stayed green. Three other perturbations of the same function each failed
at least two tests: a comparison that let a moved unit stay in the dwelling it
left, a reversed read order, and a removed guard against the value that means
no home.

**Follows.** Two things.

**The filter stays, and this row is why it has no test.** A reader that walked
the whole column would be correct only for as long as the free path keeps
clearing the home. That is a coupling to a distant mechanism, and nothing in
the reader would say so. The live walk is the same walk the unit-to-tile
bridge makes, and it is correct on its own terms.

**A perturbation that changes no answer is a result, not a failed
experiment.** It says the guard is redundant with something else. Record which
mechanism already excludes the case, so that a later reader does not delete
the guard and the mechanism in two separate changes.

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

**Repaired on 1 September 2026.** The world now reserves the unit columns from
a number its settings name, and a spawn past the reservation gets a typed
refusal.[^139] The two sentences above describe the engine before that change.
The third consequence stays open: a cost statement of a product record is still
a claim about the engine that no gate checks.

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

### FND-143 — A derived copy of an arena drops its reservation, and nothing reports it

**Believed.** A reservation is a property of the arena, so a copy of a world
holds the reservation that the world holds. The arena derived its copy, and a
derived copy copies every field.

**True.** A derived copy of a growable column allocates for what the column
holds, not for what it reserved. A copy of a world part way through a run
therefore holds a column sized to the live population, and it grows on the
next spawn where the original does not.

**Evidence.** The method found it, not a failure. The question asked of every
new guarantee was: **which code paths could drop this without failing?** A
copy is one, because the copy was derived rather than written, and a derived
copy carries each field by copying it. The standard library states the rest: a
copy of a growable array allocates for its length. A test copies a world
holding one unit, fills the copy to the reservation, and asserts that the
address of the first entry of every column did not move. That test fails
against a derived copy and passes against a written one.

The method transfers. When a change adds a guarantee that lives in a capacity,
an ordering, or a reservation rather than in a value, list the operations that
rebuild the structure — copy, serialise, merge, resize — and ask of each one
whether it carries the guarantee or only the value. A derived implementation
carries the value.

**Follows.** Three things.

**A reservation is a fact that lives in one field and in every column that
serves it.** The field says how much, and each column either holds that much
or does not. A derived copy carries the field and drops what it means, so the
capacity of the copy reads back correctly and states something false. This is
the shape the recurring defect rule names first.[^22]

**A capacity assertion cannot see this.** The copy reports the same capacity
number, because the number lives in a field the copy carried. Only the address
of the first entry moves, so the address is what a test must read.

**A property that a derived implementation can drop needs the implementation
written.** Nothing in the type system says that a copy must reserve, so
nothing fails when it does not.

**The general form: a test can pass for a reason unrelated to the thing it is
named for, and only a reinserted defect reveals which reason.** The test that
asserts a copy keeps its reservation stayed green when the reservation was
removed from the constructor, because the copy called a reserving constructor
of its own. It was named for the copy and it covered the copy, and it covered
nothing else, but nobody could have known that from reading it.

Three instances of this shape were found on one day, in three pieces of work.
A test stayed green because a second check downstream protected the value the
deleted check guarded, so the test covered neither check. A test stayed green
because it asserted only that an error occurred, and a refusal by any cause is
an error. This entry is the third.

**Reinsert each defect separately.** A test that covers two guarantees at once
cannot be told apart from a test that covers one, until the two are removed
one at a time. The cost is one build for each guarantee, and the alternative
is a suite whose coverage nobody can state.

### FND-144 — A founding that a spawn refused left a settlement standing and a part of its group alive

**Believed.** A founding leaves nothing half-founded. The engine says so in
its own comment, and one path undoes the settlement and the people when the
placement cannot seat the whole group.

**True.** Only that one path undid anything. A refusal from the spawn itself
returned immediately, and it left the settlement standing and the people
already seated alive. The store arrives after the group is seated, so a
founding stopped this way left a settlement with no production and no store.

**Evidence.** The founding seats the settlement first and then seats the group
one member at a time. The spawn returned its error through the question mark
operator, which returns before the undo. Nothing found this, because the
refusals a spawn could give were unreachable from a founding: the ground was
filtered before the spawn, the faction came from the run, and the storage
refusal needed a full arena, which no run could reach while the arena grew on
demand.

**Follows.** Two things.

**An unreachable error path is untested code that looks tested.** The path
compiled, it was covered by the type system, and a reader saw a refusal that
was handled. Making the storage refusal reachable made the defect reachable in
the same change.

**Undoing belongs in one place, not at each return.** The founding now undoes
through one function, and every refusal after the settlement stands goes
through it. A second undo written at the second return would have been a
second copy of the same fact.

**A refused founding is not hash-neutral, and a reader will assume it is.**
The undo owes that nothing lives and nothing stands. It does not owe, and
cannot give, the state hash the world held before. The arena never compacts
the slot index space and a generation never rewinds, so the slots the founding
opened stay open and their generations stay advanced.[^144] A founding that
failed therefore moves the state hash, deterministically, and that is correct
behaviour rather than a defect.

A test was written asserting the opposite and was removed before it ran. A
passing test that asks a failed operation to leave no trace would have
recorded a rule nobody chose, and the next reader of the golden state test
would have inherited it. The engine states the true outcome in a test of its
own instead, so that a later reader does not read the open slots as
wreckage.

### FND-145 — The world settings priced one new field at 82 struct literals, not 25

**Believed.** The settings struct that builds a world prices a new parameter
at about twenty-five files. The backlog item that exists to fix this states
that figure, and it comes from the site rate work, which met the cost, counted
the files and moved its parameter elsewhere rather than pay it.[^141] [^142]

**True.** The price is 82 struct literals. The reservation work added one
field, `unit_capacity`, and the compiler refused every exhaustive literal in
the workspace. The figure is three times the one the item states, and it has
grown because the tree has grown: every test written since the site rate work
added literals of its own.

**Evidence.** The whole-tree search that found them:

```
grep -rn "WorldConfig {" crates
```

It reports 94 occurrences. Three are not literals: the struct definition, a
return type, and one in the Python binding. Nine literals already used struct
update syntax or a base value and needed nothing. The remaining 82 took the
field. The sweep was mechanical and was scripted rather than done by hand,
because a sweep done by hand is done when the files look right rather than
when a search comes back clean.[^46]

**Follows.** Three things.

**The cost is not fixed. It grows with the tree.** An item that states a cost
as a number states it as of the day it was written. The right shape for this
one is that the cost grows with the number of places that build a world, and
that nothing bounds that number, so the cost only ever rises. Whoever refines
the item should say it that way and cite this row for the two measurements.

**The second measurement is what makes it a trend.** One figure is an
anecdote. Two figures, taken by two pieces of work that each paid the price,
say that the item is getting more expensive to defer, not less.

**A mechanical price is not a small price.** Every one of the 82 edits was
correct, none needed judgement, and the whole sweep was one script. It still
put a one-field change into 41 files, across three crates, two of which
belonged to other workers at the time. The cost that matters is the merge
surface, not the typing.
### FND-149 — A register can hold the comparison its own text forbids

**Believed.** A register that states the rule for reading its rows protects a
reader from misreading them. The development budget register says a row is a
snapshot and does not support a comparison against a row taken hours earlier,
and it cites the finding that gives the reason.[^101] [^97]

**True.** The register states that rule and then holds two rows that break it.
The warm rows are 153 s at `opt-level 1` and 435 s with no optimisation. The
commit that recorded them says plainly that the two runs are not next to each
other in time: the fair pair is "the earlier warm baseline of 435 s". FND-142
lists 429 s, 432 s, 435 s and 435 s as third-hour figures for the unchanged
suite, and 263 s, 283 s and 296 s as first-hour figures for the same suite.
The 435 s row is therefore a sample of a quantity that measured 263 s earlier
in the same session, and the ratio a reader takes from the table is inflated.

The conclusion the rows support is still sound, and it is sound for another
reason. The commit alternated the two profiles back to back for the test
execution and measured 429 s and 430 s against 84 s and 79 s. That pair is the
evidence, and it is in a commit body where the register cannot reach it.

**Evidence.** The register, the commit that added the rows, and FND-142 in
this document. The review of ADR-0083 found it.[^102]

**Follows.** **A rule in a register does not check the register.** A reader
takes a table as evidence and a paragraph as advice. Put the paired figures in
the table, or mark the row that no comparison may use.

**Carry the paired figure into the register when the pair is what matters.** A
commit body never decays and a register is what a reader consults. A
measurement that supports a decision belongs in both.

### FND-150 — The sweep that corrected the accumulator example reached one document and left the tree

**Believed.** FND-141 corrected a false example, and the correction reached
the document that held it. The project owner's document now states the
arithmetic correctly, so the example is repaired.

**True.** The false example is still alive across the tree. Two accepted
records hold it: ADR-0002 D3 says a byte-wide field summed over millions of
tiles overflows a 32-bit accumulator, and ADR-0053 says the same of the target
tile count. Three source comments hold it, in the value types module and twice
in the pyramid module. One research report holds it in its strongest form,
where it says the sum reaches 2^32 exactly. One complete backlog item holds it.
FND-141 named the owner's document alone, so the sweep stopped where the
finding pointed.

The rule that an accumulator widens is right in every one of those places.
What is wrong is the example, in every one of them.

**One nearby claim is not an instance, and a sweep must not take it.** A
research report says a two-byte field summed over one million individuals
reaches 6.5 times 10^10 and overflows a `u32` by a factor of fifteen. That is
true, and it is a different claim. A sweep that matches on the shape of the
sentence rather than on the arithmetic would repair a correct statement.

**Evidence.** A whole-tree search for the phrasings of the example. The
review of ADR-0083 ran it and reported a list.[^102] The list was short by
one site, and a second search by a second reader found it. The commit that
carries this entry holds both commands and the enumeration.

**A count of the sites does not belong here, and the first version of this
entry carried one.** It said six, and the true figure was seven. The entry
that records this shape reproduced it within the hour.

**Follows.** **A finding names the instance it found, and a sweep searches the
tree.** The commit rule already says a sweep is done when a whole-tree search
comes back clean, and a finding is where that rule is easiest to skip, because
a finding reads as a note rather than as a change.

**One search by one reader is not a clean search.** The review ran the search
and believed it complete. A second reader ran it again and found a site the
first pattern missed. Report the command, so that the second reader can widen
it.

**Correcting an example inside an accepted record is a decision, not a
repair.** Two of the six sites are accepted. The retcon window does not cover
them, so the repair needs an owner.



### FND-152 — Two footnote rules are stated, and only one of them can be a gate

**Believed.** The two footnote rules of the documentation rule are mechanical,
so one check enforces both.[^76] A previous finding says the check is cheap and
opens an item for it.[^145]

**True.** One of the two rules cannot be a gate today. A check written against
both rules and run over every Markdown document in the tree found the ordering
rule broken in a large minority of the documents, in every directory. Three of
those documents are the project orientation and two rules under `.claude/`,
which a worker may not edit. The repair for one document is a renumbering
sweep across the whole file, which is the operation this project gets wrong
most often.[^17]

The other rule holds. A marker with no definition, a label defined twice, one
source under two labels, and a definition nothing cites are all checkable, and
each names a document a reader can repair one line at a time.

**Evidence.** The commit that added the check holds the counts by test and by
directory, and the survey command that produced them. The check runs the
ordering test and reports it without failing. The four other tests fail the
gate, and the documents that already break them are in a falsifiable
baseline.[^146]

**Follows.** **A rule is not one gate because it is one sentence.** Split a
written rule by what a check can drive to zero. The part that can be driven to
zero fails the gate. The part that cannot is reported, and the reason it is
reported is written down where the check is read.

A gate nobody can turn green is a gate everybody learns to skip, which costs
more than the defect it catches.[^147]


### FND-153 — A footnote names its source in a code span, so its prose is not its identity

**Believed.** Two footnotes hold the same source when their definition lines
read the same after the code is removed. The citation check removes a code span
before it reads a line, because the documentation rule exempts an identifier in
code, and a new check copied that step.[^148]

**True.** A footnote is the one place where the documentation rule puts the
path inside the code span. Removing the span removes the only part that
distinguishes two footnotes whose prose lead is the same. The decision record
priority index holds two footnotes that both read "Backlog item 0123" and name
two different paths, and the check called them one source.

**Evidence.** The check reported the pair as a repeat before it read the
definition from the raw line, and reported nothing after. No other document in
the tree held the shape, so a survey of the tree found one instance and the
check would have produced a false failure on it.

**Follows.** **Read a footnote definition from the raw line.** The exemption
that a rule gives to code in prose does not extend to the reference section,
because that is where the rule puts the reference. A check that copies a step
from another check must ask what the step was for.

A false failure trains everybody to ignore a red gate. This one was found by
running the new check against the real tree before wiring it into the gate,
which is why the item asked for that run.[^146]


### FND-155 — Two recipes checked the target and only the narrower one could pass

**Believed.** The gate suite verifies that the engine compiles for the target
platform. Two recipes do it, so the property is covered twice.

**True.** The two recipes checked different things and only one of them could
pass. `target-check` checks the core and the bindings, and says in its own
comment why it stops there: the viewer links a window library that needs a
cross-compiler, and a window on a headless server means nothing. The slow gate
checked the whole workspace for the same target, so it pulled the viewer in and
demanded the cross-compiler the comment says the project does not need.

**Evidence.** The slow gate had never run on the trunk for the profile change,
so nobody had met it. On the first run the release tests passed, the licence
audit passed, and the target check failed to build the window library because
the cross-compiler is absent. The narrow recipe passes on the same machine in
about fourteen seconds. The engine and the bindings do compile for the target;
only the recipe was wrong.

**Follows.** Three things.

**A recipe nobody runs is not a gate.** This one was in the justfile, named in
the definition of done, and had never been run to completion on the trunk. It
failed the first time anybody tried it, on a machine the project expects a
contributor to use.

**Two recipes for one property is the same defect as two declaration sites for
one value.** The narrow one carried the reasoning in a comment and the broad one
carried none, so the broad one silently contradicted a decision the project had
already taken and written down. The slow gate now calls the narrow recipe rather
than restating its command.

**A gate that cannot go green teaches a reader to skip it.** That is the reason
the footnote check reports its ordering rule instead of failing on it, decided
independently on the same day.[^151]

### FND-162 — The build made three passes, and the repair removed one of them

**Believed.** Building a world cost one pass over every tile, and the tile
stub value column made it. The product record for the ground states that the
build must not make such a pass, and an earlier finding recorded that the
engine made one.[^F162A] The item that repaired it named the same one
column.[^F162B]

**True.** The build made three passes over every tile, and the column made one
of them. The other two belong to the first level of the pyramid. Building the
level reads the ground of every tile of every block. The build then closes by
rebuilding the moving part of every cell, and that sums the tile value of
every tile. A fourth cost is a proportional allocation rather than a pass: the
holder column holds one entry for each tile, and that column is a dense column
by decision rather than a defect.[^40]

**The premise of the item was wrong, not merely incomplete.** Its title says
that a world is built without a pass over every tile. **A world is still not
built without a pass over every tile.** The repair removed the column and the
pass that filled it, and two passes remain. A reader who takes the title for
the state of the engine will be wrong.

**Evidence.** A visit counter behind a test-only switch. The tile value field
counts the tiles it generates, and the build test reads the count. After the
column became generated, the build still visited each tile once, and the
remaining visit was the pyramid rebuild. The counter reported it on its first
run. Nobody had found it by reading the build, over two items and one finding.

**Follows.** Four things.

**A repair scoped to the thing that was named repairs what was named.** The
earlier finding described the defect through the column that made it, and
every reader after it took the column for the defect. The statement in the
product record is about the build, and nobody counted what the build did.

**A cost claim needs an instrument before it needs a repair.** The counter was
written to prove the repair. It found the larger defect in its first run, and
it would have found it before the repair as well.

**A test asserts what the work removed, not what the item hoped for.** The
build test asserts that the build visits each tile once and that the value
field adds none of those visits. It does not assert that the build visits no
tile, because that is not true.

**The product record stays `Accepted` and it stays false of the engine.**
Accepting a need the engine does not meet is correct. Nothing about this work
makes the need met, and an item holds what remains.[^F162D]

### FND-164 — The branch tip did not compile, and every gate would have said so

**Believed.** A merged change compiles. The gate command builds every target
of every crate, so a change that broke a test fixture could not reach the
trunk.

**True.** The tip did not compile. One test fixture built a world from a
settings literal that no longer had every field, and `cargo clippy --workspace
--all-targets` failed on it alone. The field it lacked arrived with the
reserved unit columns, and the sweep that added the field reached the source
and not that fixture.

**Evidence.** A worker started an item, ran the lint over the whole workspace,
and got the failure before it had changed anything. It then stashed its own
work and got the same failure, which is what proves the failure was already
there.

**Follows.** Two things.

**Test code is a call site, and the rule already says so.**[^57] The sweep
that added the field searched the source and stopped. A whole-tree search for
the settings literal finds the fixture, and the commit body must hold that
search.

**A gate that nobody runs is not a gate.** Nothing about this defect is
subtle. The gate command catches it in one run. It reached the trunk because
the run did not happen, and the next worker paid for it by having to decide
whether the failure was its own.

### FND-154 — Completing an item is not idempotent, and it leaves two of everything

**Believed.** An item is completed once. The guide gives four steps: fill in
the outcome, update the registers, move the file, and set the status.[^149]
Nothing says what to do when the step runs twice, because nothing expects it
to.

**True.** It runs twice. Four completed items each hold two `## Outcome`
sections and two `## References` sections, one appended after the other. The
second completion did not read the first. In two of the four the two reference
sections are identical, in one the later section is a superset, and in one the
same label names two different sources, so a marker in the body resolves to
whichever definition a reader reaches first.

**Evidence.** The footnote check found all four, because a second reference
section defines a label the first already defines. Nothing else in the tree
saw them. The commit that added the check names the four items. A parallel run
produces this directly: two workers complete one item, or one worker completes
it twice across a rebase.

**Follows.** **The completion step must read the item before it writes.** An
item that already holds an outcome is an item somebody already completed, and
the second writer is either repeating work or contradicting it.

This is adjacent to the item that fails when a merged item still reads as
open.[^150] That item checks the status against what merged. The shape here is
the other direction: the status is right and the document holds the work
twice. A check that reads one item and finds one outcome section would catch
it, and the footnote check catches it today only as a side effect.

### FND-168 — The phrase that FND-144 corrected is still in the comment above the code it corrected

**Believed.** FND-144 corrected the claim that a refused founding leaves no
trace, and the correction reached the tests. The tests state that the arena
never compacts the slot index space and that a generation never rewinds, so the
phrasing is repaired.

**True.** The claim is still alive in two places. The doc comment above the
function that undoes a founding says that a refused founding changes nothing
that a caller can observe. A caller can observe the state hash, and the hash
function writes the slot count, every generation and the free queue. The draft
record that governs the reservation says, in bold, that a founding a refusal
stops leaves nothing behind.[^F168A]

**Evidence.** A whole-tree read of the founding path, the hash function and the
draft record, made during the review of that record.[^F168B] The test that
exercises the refusal already states the true outcome in its own comment, two
lines below a test name that repeats the false phrasing. The correct sentence
and the incorrect one therefore sit in the same file.

**Follows.** Two things.

**A finding repairs the place it was written from, not the shape.** FND-144
was written from the test, and the test is right. The comment above the
function and the record above the comment were left, because nobody searched
for the phrasing.

**A test name is a claim too.** The name of the test says the founding leaves
nothing, and the comment inside it says what the founding actually leaves. A
reader who lists the test names reads the claim the finding corrected.


### FND-171 — A pedigree fixture that kills nobody never reaches a reused slot

**Believed.** The property tests over a random pedigree covered the case where
a character dies and the arena reuses its slot. Each of them calls the whole
world invariant check, and that check compares the descent row of every live
slot against the identity the arena minted for it. A defect that let a new
character keep the descent row of the character before it would therefore fail
the properties.

**True.** The fixture built the pedigree from births alone. It never removed a
character, so the arena never freed a slot, so no slot was ever reused, so the
comparison the invariant makes was never asked a question it could answer
wrongly. The properties measured the fixture.

**Evidence.** The defect was put back and the tests were run twice. The
injected defect kept the descent row of the previous character when the arena
reused a slot. With the original fixture, one test failed: the one that kills a
character on purpose and asserts the reuse before it asserts anything else.
Fourteen tests passed, including both property tests and the whole-world
invariant check inside them. A removal was then added to the fixture, and a
test was added that asserts the fixture reuses a slot. The same defect then
failed three tests. The commit body holds the two runs.

**Follows.** Three things.

**The thread-count property stayed green under the defect, and that is
correct.** A defect that is itself deterministic gives the same wrong answer at
every thread count. The determinism tests cannot tell correct from
consistently wrong, and this is a local instance of that.[^F171A]

**An invariant check inside a property proves nothing about a state the
property never builds.** The check was right and it was never reached. Counting
the tests that call it says nothing about the cases it saw.

**A fixture needs a test of its own when the case it must reach is a state
rather than a value.** The added test asserts that the plan reuses a slot. It
fails if a later change to the fixture stops reaching the case, and nothing
else would report that.
### FND-174 — Neither determinism test defends work that takes several ticks

**Believed.** The two determinism tests are the project's safety net. A change
that broke the simulation would move the golden state hash, or would make two
thread counts disagree.

**True.** They defend neither the number of ticks a build takes nor the
storage that carries it between them. The build was made to finish in the tick
it started, and both determinism tests stayed green. So did the thread-count
equivalence assertion inside the upgrade test file.

**Evidence.** The work each upgrade kind asks for was set to one, which is
what one builder adds in one tick. Seven tests of the upgrade file then
failed, and among them were the two that carry the claim: the build finishes
in one tick, and the work does not persist across an interruption. The two
determinism tests passed unchanged, because no golden scenario builds
anything, so the map is empty in every one of them and its bytes do not move.

**Follows.** **A determinism test cannot tell a correct rule from a
consistently wrong one, and this is the shape at the level of a whole
feature.** The rule is already written for a draw key.[^F174A] The instance
here is larger: a feature that no golden scenario exercises is outside both
determinism tests, whatever the tests cover elsewhere.

The cheap repair is a golden scenario that builds. It was not written here,
because the golden file is shared and several workers were regenerating it in
the same session. The follow-up item holds it.[^F174B]

### FND-175 — A crowd of a hundred and twenty units never fills one tile

**Believed.** A crowded world is enough to test admission. Spawn many units,
let them wander, and some tile will reach its capacity.

**True.** It does not. A test asserted that a tile with a finished road holds
more units than its ground alone allows, over a world of 48 by 48 tiles with
120 units. No tile ever went above the ground capacity, so the assertion could
not distinguish admission that reads the upgrade table from admission that
does not.

**Evidence.** The test failed on the assertion that says the fixture must
reach the case, which was written beside the claim for exactly this reason.
The passing fixture crowds six neighbours of one tile with 24 units each and
compares the paved world against the unimproved one, spawned in the same order
from the same seed.

**Follows.** **A fixture that models the typical case supplies no
extreme.**[^23] Wandering units spread out; they do not crowd. A test about
a full tile must build the crowd on purpose, and must compare against the
world without the change, so that crowding alone cannot pass it.

**Write the assertion that says the fixture reached the case.** It is what
turned a test that would have passed for the wrong reason into a test that
failed loudly.

### FND-176 — An advance that sums a count is order-free, so a probe over its order asserts nothing

**Believed.** Every parallel pass in this project needs a perturbed build that
reverses its order, so that a determinism test over it can be proved able to
fail.[^82]

**True.** The build advance has no order to perturb. Each tile's contribution
is a count of builders multiplied by a whole-number rate, and integer addition
does not depend on order. Reversing the sorted order changes the result only
where a tile carries no site and two builders name different kinds, which is
the tie-break and not the sum.

**Evidence.** The pass sorts by the key vector and then counts. The sound sort
is therefore used in the perturbed build as well, and no probe entry was added
for it. The stored result still reaches the state hash, so both determinism
tests cover what the pass produced.

**Follows.** **A probe is worth adding where the order decides the answer.** A
probe over a pass whose answer is order-free would assert that a perturbation
is visible, and it would fail, which trains a reader to weaken the probe. The
absence is recorded here so that a later reader does not read it as an
oversight.

The order still matters for the tie-break, and it is the same key vector sort
the gather resolve uses. That sort has a probe of its own.

### FND-177 — The position pass reads the capacity table, and no world can tell

**Believed.** A pass that sizes a per-site row from the terrain capacity table
is guarded by the invariant check that compares the two. The check names the
capacity, so a pass that stopped reading the capacity would fail it.

**True.** It would not, and no test that drives a world can. Every ground a
settlement may stand on carries the same capacity, and the row width is folded
from the largest capacity in that table, so the two numbers are equal for every
tile a site can occupy. A pass that used the row width in place of the capacity
gives an identical answer everywhere the engine can reach.

**Evidence.** The pass was changed to use the row width and nothing else was
touched. Every test of the subsystem stayed green, including the invariant that
names the capacity. The mechanism that excludes the case is the founding: it
refuses a tile whose ground admits nobody, and the capacity table gives every
other kind the same number.[^F177A] [^F177B]

**Follows.** Two things.

**The guard and the thing that makes it unnecessary must not be deleted in
separate changes.** The guard is right, and it becomes load-bearing the moment
one terrain kind carries a capacity between zero and the largest. A reader who
finds it unreachable and removes it removes the only thing that would catch the
pass then.

**A test that the engine cannot reach is still worth writing, if it says so.**
The subsystem holds one test that drives the pass directly over ground that
admits nobody. It is the only test that can fail on this, and its comment says
why nothing else can.

### FND-178 — A holder field of zero hides a bare slot index at slot zero

**Believed.** A test that seats a unit in a position, kills it, and lets the
storage reuse the slot proves that the position stores a whole identity rather
than a bare slot index.

**True.** Only when the unit under test does not stand in slot zero. The holder
field uses zero for nobody, so a bare slot index of zero reads back as a
vacancy. The wrong implementation then answers `None`, which is what the test
asserts, and the test passes for the wrong reason.

**Evidence.** The bare index was put back on purpose. Every test stayed green.
The fixture spawned one unit, which took slot zero. A filler unit was added to
take slot zero and hold it, and the same defect then failed four tests,
including the slot reuse test.

**Follows.** **A sentinel value in a field under test is part of the fixture,
not part of the assertion.** When a field carries an in-band value for nothing,
ask whether the case under test can produce that value by accident. Here the
first entity of a fresh arena always does.

This is the fixture shape the testing rule already names, met at a new
place.[^84] The defect was found by putting it back, and reading the test
would not have found it.

### FND-179 — A fixture that runs a pass on every tick cannot tell two cadences apart

**Believed.** A test that kills the holder of a position and steps the world
proves that the engine releases a dead holder on every frame, rather than only
on the interval that resizes the row.

**True.** Not when the fixture set that interval to one tick. Every tick is then
an interval tick, so the two cadences give the same answer and the test measures
neither.

**Evidence.** The release was moved behind the interval test on purpose. Every
test stayed green. A test was added that sets a long interval, kills the holder,
asserts that the next tick is not one the interval names, and then steps. That
test failed under the same defect and passes without it.

**Follows.** **A fixture that makes a schedule trivial removes the schedule from
the test.** When two passes run at different cadences, at least one test must
use a fixture where the cadences differ, and it must assert that the tick it
steps onto is the one it means.

### FND-180 — Movement reads whether a unit chose, not what it chose

**Believed.** The choice pass decides what a unit does, and movement acts on
that decision. The item that built the pass states in its outcome that
movement reads the option column. The record it wrote states that a unit which
holds no intent does not move.[^F180A] Both statements are read as saying that
the option steers the step.

**True.** The movement pass reads the option column, tests that it holds a
value, and discards the value. It then draws a uniform direction from the
counter-based generator, keyed on the system, the frame, the entity and the
draw index. A unit that chooses to forage takes the same distribution of steps
as a unit that chooses to climb. The option column decides one bit: whether the
unit moves at all.

**Evidence.** The intent half of movement discards the option at the first
line of its filter and never reads it again.[^F180B] No other stage of the step
reads the column. No test asserts a consequence of which option a unit chose.
This was found by reading the source. It was not run, so the count of tests
that would survive a pinned column is unverified.

**Follows.** Three things.

**The product record this project points at is unmet at the action.** It asks
that a unit acts on the world it can see, and that a watcher who changes the
world sees the behaviour change.[^F180C] The choice changes. The behaviour
does not.

**Every pass upstream of the column is paid for and unused.** The level 1
rebuild, the cell summary, the need column, the option weights and the stagger
schedule all feed one bit.

**The item was correct about its caller and still shipped this.** It named
movement as the reader, and movement is the reader. Naming the caller is not
enough when the caller discards the payload.[^F180D]

### FND-181 — The rules against inert work look for an absent caller, and this defect has one

**Believed.** Two rules cover work that nothing uses. One says not to declare a
capability before something calls it.[^37] One says that when the engine is
obligated to invoke a thing, the test must start at the engine.[^F181B]
Together they were read as covering the case where the project builds
something and nothing uses it.

**True.** Both rules look for an absent caller. They find inert code. They do
not find inert data: a value that the engine computes, stores, hashes and
tests, where the caller exists and discards the payload. The option column is
that case.[^F181C] The influence field is a second: a solve runs its full pass
count on every tick over a field that no source raises and no consumer
reads.[^F181D] The tile stub value is a third: a full pass over every tile on
every tick writes a random walk that only the viewer and the summary read, and
neither of them decides anything with it.

**Evidence.** Three candidate repairs were tested against the option column,
and all three pass it. A rule that a person must be able to run the feature
passes, because the demonstration runs the choice pass on every tick. A check
that reports a public verb with no caller passes, because both the pass and the
column have callers. A rule that a backlog item names its caller before it is
refined passes, because the item named the right caller.[^F181E]

**Follows.** The test must be about the value, not about the caller. For each
value the work writes into state, name the stage that reads it to decide
something, and write a test that changes the value and asserts that the
decision changes. The falsification is the one the testing rule already
trusts: pin the value to a constant and watch the suite stay green.[^84]

This is the discipline the testing rule already states for a keyed draw, turned
round. That rule says to test what the value depends on.[^43] This says to
test what depends on the value.

### FND-182 — A summary of a generated field splits the way the field splits

**Believed.** The item that put food into the level 1 summary said that the
rebuild takes the resource field as one more argument, and adds the stock of
each tile as it walks the cell.[^F182A]

**True.** That puts a read of a generated field over the whole world into every
frame. A tile stock has two parts: the stock the ground generated, which is a
pure function of the seed and the address, and the stored take, which a frame
can change.[^F182B] [^F182C] The summary splits the same way. The original food
joins the part of a cell that the build computes once, beside the height total.
The rebuild subtracts the stored take alone, and the ledger holds one entry for
each tile that somebody gathered from, so a world that gathered nothing costs
one search for each row of a block and no per-tile read.

**Evidence.** The level already split each cell into a part the ground fixes
and a part a frame changes, and the record that generates the ground calls a
sweep of the whole world every frame a design mistake.[^F182D] The split was
applied to the food total, and the equality against level 0 holds in both the
gathered case and the ungathered case. No figure here is measured, because no
measurement exists on the target platform.[^F182E]

**Follows.** A summary field over a field that is a generated base plus a
stored change splits into a build part and a rebuild part. The build reads the
generator once. The rebuild reads the stored change, whose cost follows the
change and not the size of the world.

### FND-183 — Pinning a reader is not pinning the value

**Believed.** FND-181 says to pin a value to a constant and watch the suite
stay green.[^F183A] That was read as pinning the accessor that reports the
value.

**True.** The accessor is often not the path the consumer takes. The food total
of a cell was pinned to a constant in its accessor, and every choice test
stayed green, because the mean that the option reads divides the private field
and never calls the accessor. The same pin applied to the stored field failed
two choice tests at once, and so did a pin on the mean.

**Evidence.** Three pins were run separately on a development machine, each
with the source restored afterwards. The accessor pin left the twelve choice
tests green and failed six pyramid tests. The stored-field pin and the mean pin
each failed the two choice tests that read the food.

**Follows.** Name the site that is pinned, and pin the site the consumer reads.
A suite that stays green under a pin is evidence only when the pin reaches the
consumer. Where a reader and a stored field are two paths to one value, pin
both, or pin the stored field alone.


### FND-190 — A per-cell field and a per-unit score search do not give the same answer

**Believed.** A field that ranks the neighbours of a cell once gives the same
answer as a search in which each unit scores its six neighbours. The analysis
note states that the search "gives the same answer for every unit of one
cell", and the backlog item repeats it.[^F190A] Both were read as saying that
the two shapes are equivalent, so the field is only the cheaper way to compute
one thing.

**True.** The two agree when the field ranks the neighbours on the cell value
that the option reads. They disagree when the rank reads the score. A score is
the cell value multiplied by what the unit wants, and two properties break the
equivalence. A want of zero makes every score equal, so the tie-break picks the
direction and the ground does not. The multiplication saturates, so two
different cell values can give one score, with the same result.

**Evidence.** The score of an option is one saturating multiplication of the
want by the cell value, and the want is itself the drive scaled by a
content-supplied weight. A weight of zero, or a drive of zero, makes the want
zero.[^F190B] This was found by reading the source. Nothing was run, so no
case of saturation in a live world is verified.

**Follows.** The record that governs the field ranks the neighbours on the
cell value, and says so as a decision rather than as an implementation
note.[^F190C] The equivalence that makes the cheap shape sound is narrower
than the analysis note claimed, and it holds because the ranked quantity
belongs to the cell, not because the score does.

### FND-191 — The number of the food commodity is written wherever the engine needs it

**Believed.** The map from a kind of work to a commodity is the one place the
project holds a placeholder for the commodity set. A backlog item records that
the map carries no information, because the store of a site holds one
commodity.[^F191A]

**True.** The engine does not read that map at all. It writes the commodity
number literally at each site that needs one. The founding sets the production
rate of a new site against commodity zero. The consumption pass draws the
ration against commodity zero. Neither reads a table, and nothing fails when
one is changed and the other is not.[^F191B]

**Evidence.** Two passes of the world module name the commodity by its number
and not through any map. This was found by reading the source while refining
the item that moves a carried load into a store. Nothing was run.

**Follows.** Work that moves a resource into a store must not add a third
literal, because that is one value declared in three places with nothing to
fail when the copies disagree.[^22] The map from a resource kind to a
commodity is declared once, and the item that gives a kind of work its
commodity absorbs it.[^F191A]

### FND-192 — The record check reads a mention of a record number as a citation

**Believed.** A document marks a mention of a record number by writing it in a
code span, and a citation by writing it plain. The registry states that rule
for a retired number, because a retired number holds no claim and a citation
says "follow this for the claim". The registry also states that the citation
check enforces the rule, and that it caught the first attempt to break
it.[^F192A]

**True.** The record check does not make that distinction at all. It runs its
citation pattern over the whole text of a record, so a code span, a fenced
block and plain prose give one result. It then fails when the number names no
record and no registry row, which is exactly what a retired number does. The
two checks therefore hold opposite rules: one outside the records requires the
code span, and the one over the records ignores it.

**Evidence.** The record check applies the citation pattern to the raw text
and not to the text with the exempt material removed, although the same
function strips that material for the check that looks for volatile
figures.[^F192B] A draft record wrote `ADR-0057` in a code span, as the
registry directs, and the check refused it. The record was rewritten to name
the number nowhere.[^F192C]

**Follows.** A record cannot say which retired number held a claim of its
shape, so a reader of that record cannot look the number up. The registry
keeps a retired row precisely so that a reader who meets the number learns
where it went, and the check blocks the one document type that would send them
there. A backlog item holds the repair, and it states the shape rather than a
fix, because a check that skips a code span would also skip a real citation
written inside one.[^F192D]

### FND-193 — The largest capacity the engine states is folded from one of two tables

**Believed.** The terrain module owns the capacity of a tile, and one fold over
that table reports the largest number of units that can stand on a tile. The
fold says so in its own words: a caller that must hold one entry for each unit
that can stand on a tile reads it rather than writing a literal.[^F193A] The
crossing capacity of a made way sits in the same module for the stated reason
that a second declaration of a capacity would be one fact in two places.

**True.** The fold walks the terrain kinds only. It does not see the crossing
capacity, which is larger than every terrain capacity, and which a finished
road gives a tile. So the module holds two capacity constants and folds one of
them. The width of the position table of a site is that fold, and the guard
that clamps a site to it carries a comment saying the clamp takes no effect
today. On a tile with a finished road it does.

**Evidence.** The composition that admission uses reads both the ground and the
finished upgrade and returns the larger.[^F193B] The position table reads the
ground alone, and its own comment says the answer comes from the terrain
capacity table and from nowhere else.[^F193C] The founding seats a group and
estimates the room of a place from the ground alone. This was found by reading
the source while reviewing the upgrade record.[^F193D] Nothing was run, and no
run reaches it today, because no engine rule issues a build order.

**Follows.** Three things.

**One question has two answers.** Admission admits more units onto a roaded
tile than the position table believes the tile holds, and than the founding
believes when it seats a group. Nothing fails when the two disagree, which is
the shape this project meets most often.[^22]

**A record claimed the universal that would break it.** The upgrade record
states that every caller which asks how many units a tile holds calls the
composition. It is fortunate that this is false: a position table sized by the
terrain fold cannot hold the count the composition would give. The review holds
the correction.[^F193D]

**The choice is a judgement, not a defect to sweep.** Whether the positions of
a site should follow the composed capacity, and whether the fold should walk
both tables, is a register row with a backlog item behind it.[^F193F]

**A fourth caller was found later, and it is the one a person sees.** The
drawing pass counted a painted tile as at its capacity against the ground
alone, and painted an over-full marker above that number. A watcher therefore
read a correctly filled made way as over-full. It enforced nothing, because the
viewer never writes to the world.[^F193G] [^F193H]

**That caller was not a judgement, and it is repaired.** The viewer asks the
same question admission asks, so it now calls the same composition. Two tests
hold it: one fills a roaded tile above the ground and below the road and
asserts no mark, and one fills it above the road and asserts the mark. The
first fails when the ground reader is put back. This changes nothing about the
register row, which is about the three callers that ask a different question.

### FND-198 — A second layout of one reading loses the corrections the first one earned

**Believed.** The panel and the cards read one readout, so they cannot
disagree about a number. The readout is the single statement of what the view
knows, and two layouts of it are two arrangements of the same facts.

**True.** They cannot disagree about a number. They can disagree about which
numbers to draw, and that is where a correction lives. The panel sizes its
faction legend by the faction count of the world, because the colour table is
larger than most worlds need and a legend sized by the table names a colour
that no faction uses. The cards were written fresh and sized the legend by the
table. In a world of four factions the cards named six, and two of them stood
for nobody.

**Evidence.** The defect was visible in the first render of the reference layer
and in no test, because no test existed yet. The panel had the same defect once
and its own helper carries the repair. The repair did not travel, because the
new layout called no part of the old one.[^F198A]

**Follows.** **When you write a second view of one model, list what the first
view decided and not only what it read.** The readout carried the faction count
and the new layout did not ask for it. A shared reading is not a shared
judgement.

The general form is the redundant declaration shape, one level up.[^22] Two
layouts are two declaration sites for the question "which rows does a reader
need", and nothing fails when they answer it differently. The repair here was
to give the readout an accessor for the faction count, so both layouts ask the
engine rather than guessing from a table.

### FND-199 — The inspection tool cannot carry what the window stopped showing

**Believed.** The window can drop a section because an agent inspection tool
holds the same numbers. The tool speaks a protocol, holds a world between
calls, and reports what the engine knows, so it is the natural home for detail
that a window cannot afford.

**True.** It reports the tick, the extent, the seed, the faction count, the
tile count, the state hash and the event count. It reads the event log, the
tile changes and the gather events. It spawns, despawns, orders a gather and
names the tile of one unit. It has no reader for the founding survey, the level
1 summary of a region, the store or the rate or the ration of a site, the
choice explanation of a unit, the stock or the holder of a tile, the census of
the ground, or the crowding counts. Those are most of what the panel
holds.[^F199A]

**Evidence.** The tools were counted while designing the overlay. Moving the
panel's detail to that tool would have been about six new readers of work, and
it would have blocked the overlay behind them.

**Follows.** **The detail went to a rendered picture instead, which needed no
new work.** One example already drew the whole panel to an image file at any
height, with no display. A build recipe now names it, so the answer to "where
did the detail go" is a command a person can run.

The order of the work changed because of this. The agent tool still cannot show
a watcher what the panel shows, and that is a separate item rather than a
condition of the overlay.[^F199B]

**A command name was shown to a person before it existed.** The name came from
a mockup in a design conversation and not from the build file. It exists now,
and a test compares the name the window prints against the recipes the build
file defines, so the two cannot drift apart in silence.

**Closed in the same round it was opened.** Another worker took the seven gaps
this entry names and closed every one of them. The server now carries the
founding survey, the level 1 summary of a region, the store and the rate and
the ration of a site, the choice explanation of a unit, the stock and the
holder of a tile, and the ground and crowding counts of a window. The entry
stays because it is why the detail went to a rendered picture rather than to
the tool: the picture needed no new work and the tool needed seven readers, and
that ordering was correct at the time it was chosen.

**What the closure does not change.** The picture is still where a person reads
the record, because a person reads an image and an agent does not. The two
paths answer different readers, and the tool did not replace the picture.

### FND-209 — The camera was not slow at the far zoom. The pan step was the wrong size for the view

**Believed.** A watcher reported that camera navigation is "incredibly sluggish"
at the far zoom, and the reading taken from that was a cost problem. A drawing
at the smallest tile the camera allows costs about a third of a second on a
development machine, which supported the reading.[^F209A]

**True.** **Nothing was slow.** The project owner corrected it from watching the
window move: the step is the same size however far out the view is zoomed. One
press moved a fixed count of tiles, and a tile is a fixed number of pixels only
at one zoom. At the zoom the viewer opens on, a press moved eighteen pixels. At
the smallest tile the camera allows, the same press moved three. The camera was
covering a fortieth of the distance for the same effort, and a watcher reads
that as slowness.

**Evidence.** The step was one and a half tiles, multiplied by the tile width in
pixels. The tile width runs from two pixels to sixty-four, so the pixel distance
of one press varied by a factor of thirty-two across the range the camera
allows. The window polls the keyboard once for each drawn frame, so a held key
crossed the window in about two seconds at the opening zoom and in about
fourteen at the far zoom.

**The cost figure is real and is not the cause.** The drawing cost about a
third of a second at the far zoom on a development machine, and the ground of
every visible tile was generated twice to produce it.[^F209B] That was a
separate defect with its own item, and the item is complete.[^F209C] It made
the frame rate low; it did not make the camera cover three pixels a press.

**Follows.** **A pan covers a share of what the window shows.** The step is now
a share of the window and not a count of tiles, so a press moves the same part
of the picture at every zoom.

**The share preserves rather than improves.** It is the share the old step
covered at the zoom the viewer opens on: eighteen pixels in a window seven
hundred and twenty pixels on its shorter side, which is one in forty. The one
zoom nobody reported is unchanged and every other zoom now matches it. No part
of the value was read off a render, and one test asserts the opening zoom still
moves what it moved before.

**The zoom did not have the matching defect, and now a test says so.** A zoom
press multiplies the tile size by a factor. A press therefore changes the view
by the same proportion wherever it is taken, which is the property the pan
lacked. A zoom that added a count of tiles would produce the same complaint in
the other direction, so a test asserts the ratio is one number across the range.

**A measurement can be right and still answer the wrong question.** The cost of
a drawing was measured carefully, on the correct binary, and it supported a
diagnosis that was wrong. The person watching the window had the answer, and
nobody asked them until after the measurement was taken. **Ask the reporter what
they saw before measuring what you think they meant.**

### FND-207 — The grid a watcher saw is the gap between the tiles, and the gap was neither whole nor bounded

**Believed.** The window draws a black border around each tile, and the lattice
that emerged at some zooms and not others was a defect of that border. The
holding border was the suspect, because it is the only border the drawing
names.

**True.** The drawing draws no black line at all. It fills a square smaller
than the tile and leaves the space around it, and that space shows the colour
of the ground outside the world. What a watcher reads as a grid is the space
the drawing does not fill. The holding border is a separate layer, in the
colour of a faction, and it is not involved.

The gap had two defects, and they answer two different halves of the report.

**The lattice.** The drawing took one integer square width from the tile width
and placed it at the rounded centre of each tile. A tile is a fractional number
of pixels wide at nearly every zoom, because each zoom step multiplies the size
by a fraction. The rounded centres therefore advanced by a whole pixel more
under some tiles than under others, while the square stayed one width, so the
gap was one pixel under some tiles and two under others. The pattern repeats
across the picture at the beat of the fraction, and the eye reads a beat as a
lattice.

**The far zoom.** The square was a fixed share of the tile width, and the share
rounded down to a whole number of pixels. At the smallest tile the camera
allows, the square was one pixel of two and the gap took three quarters of the
cell in area. The picture at that zoom is mostly the colour outside the world,
with the ground showing through as specks. That is the state in which the grid
"emerges", and it emerges because the map disappears.

**Evidence.** The gaps were counted rather than judged, across a hundred
neighbouring tiles in one row, at ten tile widths on a development machine. At
a whole width the gap was one value. At a width of four and a half pixels it
was zero under half the tiles and one under the other half; at six and a half
it was one under half and two under the other half. Rendering the same world at
each width showed the banding at exactly the widths the count named, so the
artefact is in a still picture and is not a motion artefact.

**Follows.** **A separator must be a whole number of pixels, and it must be the
same number under every tile.** The drawing now takes the far edge of a tile
from the near edge of the tile beside it, read the same way, so the two agree by
construction rather than by arithmetic that a reader must check.

**Two snapped values that a reader expects to be equal are one fact in two
places.** The first repair computed the right edge as the rounded centre plus
half a width, and the next tile's left edge as its own rounded centre less half
a width. Those are equal in exact arithmetic and are not always equal in
floating point. A test over twelve widths found the disagreement at a width of
three and three tenths, which no picture would have shown.

**A separator that covers more of the cell than the tile is not a separator.**
The drawing leaves the gap out below the width at which the gap takes half the
cell. The bound comes from that identity and from nothing else. It is not read
off a picture and it does not depend on the world, the seed or the window.[^F207A]

**What the bound does not settle.** At a tile of four pixels the gap passes the
bound and still takes forty-four parts in a hundred of the cell, and the picture
at that width still reads as a grid. The bound is the one value in its family
that a sentence forces. Any other share is a matter of taste, and the register
holds the question rather than a number somebody liked.[^F207B]

### FND-208 — The window stated a drawing cost of zero in every stored picture, and it was a mean over no measurement

**Believed.** The cost card of the window reports what the drawing cost. Every
stored picture of the window said `draw 0.0 ms`, and the drawing was assumed to
be very cheap.

**True.** **A drawing cannot measure itself.** The run records the cost of a
frame after that frame has been drawn. The cost card is drawn inside the frame,
so the mean it states covers the frames before it. A picture written by one
call to the drawing has no frame before it, and the mean is then taken over
nothing. The card printed the result of that division as `0.0 ms`, which reads
as a measurement of a free drawing.

The live window is not affected after its first frame, because by then frames
have been recorded. Every picture anybody looked at was affected, because a
picture is one frame. The one instrument the project had was saying zero in
every image the project examined.

**Evidence.** The drawing was timed outside itself, over five frames at each of
ten tile widths, on a development machine and not on the target platform. At the
smallest tile the camera allows, one drawing of the demonstration world cost
about a third of a second. That is three frames a second, and it is the reason a
watcher reports that the camera is sluggish when zoomed out. The figure the card
would have shown had it been able to measure itself is four orders of magnitude
away from zero.

**Follows.** **A cost the run has not measured is absent, not zero.** The
window now says so in words. The record already required this of a number the
window cannot afford, and a number nobody has taken is the same case.[^F208A]

**A row that reads as a measurement must carry the count it was taken over.**
The count travels with the mean into the readout, so no caller can print a mean
without knowing whether one exists.

**The stored picture did not catch this.** The test that stores a picture of
the panel supplies fixed costs, because a clock gives a new number on every
run. That is correct, and it is why the fixture never reached the case where no
measurement exists. The test that closes this drives the real drawing with a
run that has recorded nothing.[^F200A]

### FND-206 — The holding border already draws only on a boundary, and the busy picture was the fixture

**Believed.** A dense picture of a world was hard to read because the holding
border drew on every held tile. The repair was to draw the boundary of a
holding instead: a tile whose neighbours all share its holder would draw
nothing, and only a tile on the edge of a holding would draw a border. A
register entry and a backlog item both stated it that way.[^F201B]

**True.** The drawing already does that. The border test reads the six
neighbours of a held tile and draws only when one of them has a different
holder. A tile surrounded by its own faction draws no border and never did.
The repair was already in the code, and the entry that proposed it described a
defect that does not exist.

**Evidence.** The world was counted rather than judged. Of the held tiles, 83
in 100 sit on a boundary in the picture that looked bad, 89 in 100 in the
picture that looks best, and 85 in 100 in a third. **The picture that reads
best has the highest share.** That number therefore does not explain the
difference and cannot be the mechanism.

The number that does explain it is the share of the world that is held. It is
74 in 100 in the picture that looked bad and 8 in 100 in the picture that reads
best. A world where three quarters of the ground is claimed draws as a map of
factions, because it is one.

**Follows.** **The fixture made the defect.** The bad picture came from six
hundred units on a world of thirty-six hundred tiles, which is one unit for
every six. Nobody chose that density for a reason; it was carried from one
command line to the next while reproducing a different problem. The
demonstration seats thirty people for each faction in a world of a quarter of a
million tiles, and the default picture command is a fortieth as dense.

**A measurement before a repair would have cost one run.** The first repair for
this was a border weight that fell with the tile size, and it was removed for
keying on the wrong variable.[^F206B] The second repair was the boundary rule,
and it was already implemented. Both were proposed from a rendered picture and
neither was proposed from a count. The count took one throwaway example and it
refuted both.

**What is real.** The drawing borders a holding against unclaimed ground and
against another faction alike, and its own comment says it marks "where one
holding meets another". The code and the comment disagree, and the comment is
the thing that is wrong.[^F206C]

### FND-201 — The holding edge carries no information when holdings interleave at tile scale

**Believed.** A dense picture of a world was hard to read, and the units were
the cause. A unit draws as a disc over its tile, so a crowded world buries the
ground under discs.

**True.** The discs cover about one part in fifty of the painted picture. The
saturation comes from the holding layer: a held tile takes a mix of the
holder's colour, and a held tile whose neighbour has another holder takes a
border in nearly the pure colour. When three factions interleave at tile scale,
almost every held tile borders a differently held one, so almost every tile
draws its border. The border is then not a border. It is a second fill.

**Evidence.** The picture was measured rather than judged. At six hundred units
on three thousand six hundred tiles, pixels of a pure faction colour were two
in a hundred of the painted picture. Drawing the same world with the holding
border suppressed made the ground legible at once, with the holdings still
readable as fields of colour.

**Follows.** **The border is keyed on the wrong variable, and so was the first
repair.** A border of one pixel is a larger part of a small tile than of a
large one, so scaling the border's weight with the tile size looks principled.
It is not the cause. The demonstration window opens near the same tile size and
does not have this problem, because its holdings are small clusters in a large
world and its borders are rare and informative.

**The cause is the density of the holdings, not the zoom.** A repair keyed on
tile size would have improved one picture and left the mechanism untouched,
which is the shape this register already holds twice.[^F201A] [^22] The first
repair was written, rendered, judged better, and then removed for that reason.

**The open question is what a border should mean when everything is a border.**
Drawing the outer boundary of a contiguous holding rather than the boundary of
every tile would carry information at any density. That is real work and it is
a separate item.[^F201B]

**Corrected. The drawing already draws the boundary and not every tile, and
this entry was wrong about the mechanism.** The share of held tiles that sit on
a boundary is highest in the picture that reads best, so it cannot be the
cause. The share of the world that is held is the difference, and the picture
that looked bad came from a fixture nobody chose on purpose. The correction
holds the counts.[^F201C]

### FND-200 — A predicate that accepts a prefix accepts every rename that keeps it

**Believed.** A check that compares a name in one place against a name in
another closes the gap between them. The window prints a command name and the
build file defines the recipe, which is one fact in two places, so a test that
looks for the name in the build file fails when the two drift apart.[^22]

**True.** The test looked for a line that **starts with** the name. A recipe
renamed from `inspect` to `inspect-the-panel` still starts with `inspect`, so
the check passed and the window still told a person to run a command that no
longer existed. The check closed nothing. It only appeared to.

**Evidence.** The recipe was renamed and the suite stayed green. The predicate
now asks for the name followed by an argument or by the colon that ends a
recipe line, and the same rename then fails it. Nothing but the falsification
would have found this: the test read correctly, it named the right two places,
and it passed for the right reason on the unmodified tree.[^F200A]

**Follows.** **A test that compares two names must compare whole names.** A
prefix match, a substring match and a `starts_with` all accept a family of
wrong answers, and the wrong answers are exactly the ones a rename produces. A
rename that extends a name is the most common rename there is.

The general form is wider than a name. **An assertion that accepts a range
where the truth is a point is not an assertion.** It fails only for inputs
nobody was going to write. The project already holds that a fixture must supply
the extreme that would fail an assertion.[^23] This is the same defect on the
other side: the assertion itself was too loose to receive the failing case the
fixture would have supplied.

**Where else this shape sits.** Any check that matches a name against a file:
a citation path, a record number, a registry row, a recipe. A record number
check that accepts a prefix passes `ADR-0007` for `ADR-00071`.

**A second instance was found in the same session, in a filter rather than a
match.** The record check skips a path that holds a directory part naming a
worktree, so that one run does not read a checkout another run is changing.
Every worker here runs inside such a directory, so the filter drops the tree it
was asked to read.[^F194REF] The cause is recorded there; the shape is recorded
here, because the two are the same shape and one row for one cause is
enough.[^22]

"Skip a sibling checkout" and "skip any path that passes through a worktree
directory" are different predicates. The loose one swallowed the strict one, in
the same way `starts_with` swallowed the whole-name match above. **A filter and
a match fail alike when they accept a family where the truth is a member.** The
direction differs and the defect does not: a loose match keeps what it should
drop, and a loose filter drops what it should keep.

### FND-197 — Accepting a record breaks every citation of it

**Believed.** A record is accepted by a reviewer, and the acceptance is an edit
to the registry row. The registry says the file moves between directories as
the status changes, and that reads as a second mechanical step of the same
size.

**True.** A citation of a record names its path, and the path holds the
directory. So a citation written while a record is a draft states the
directory a draft lives in, and the acceptance makes every one of them name a
path that does not resolve. The citation check reads source comments as well
as documents and fails on such a path.[^F197A] The acceptance is therefore a
whole-tree sweep, and it grows with how well the record is cited.

**Evidence.** A review reached the point of accepting two records and could
make neither status change. One of the two carries nineteen citations in
source comments under one crate, and one carries a single citation.[^F193H]
The sweep is invisible until the check runs, and it lands on the tree that a
documents-only worker may not touch. This was found by counting the citations
of both records. Nothing was run beyond the document checks.

**Follows.** Three things.

**A reviewer that may not edit source cannot accept a well-cited record.** The
verdict and the status change have different blast radii, and only the verdict
is the reviewer's work.

**The cost falls hardest on the records that earned it.** A record nothing
cites moves for free. A record that reaches nineteen call sites pays for
each. That is the opposite of the incentive the project wants.

**The choice is a judgement and it is in the register.** Whether a citation
should name the directory at all, and what a stable path would look like, is
an open row.[^F197C]



### FND-194 — The record check reports no source citation inside a worktree

**Believed.** The record check gives one answer for one tree. A worker runs it
in a worktree, reads the note count, and compares it against the count another
worker reported. A note that appears is a note about the records.

**True.** The check reports more notes in a worktree than on the trunk, and
the extra notes are false. The uncited note asks whether any other record or
any source file cites a record. The scan that gathers the source files skips
every path that holds a directory part named `worktrees`, so that a run does
not read a checkout another run is changing. A worktree of this project sits
under that directory, so its own root carries the part, and the filter drops
every file of the tree it was asked to read. The source corpus is then empty
and no record can be cited by a source file.[^F192B]

**Evidence.** The filter was applied to both roots directly. It returns 2162
files for the trunk and none for this worktree. The check reported 11 notes
here against 2 on the trunk. A search of this worktree for each of the 11
numbers found 9 of them in source files, which leaves the 2 the trunk
reports.

**Follows.** Four things.

**The trunk number is the true one, and a worktree number is a floor.** Every
worker on this project runs in a worktree, so every worker reads the inflated
count. The two records this session corrected were both named by a false note,
and both are cited by source files.

**A note that moves on its own is a note that gets ignored.** The check reports
the uncited note rather than failing on it, because the rule treats low
citation as a question and not a verdict. A question that eleven records raise
in one run, nine of them wrongly, teaches a reader to skip the whole block. The
one real note is then invisible.

**The skip set names a directory part and not a boundary.** It was written to
keep one run out of another run's checkout. It cannot tell the tree it is
scanning from a tree it should refuse, because both carry the same part. A
backlog item holds the repair, and no fix belongs in this finding.[^F194C]

**No gate was weakened, and no pass became meaningless.** The source corpus
reaches one assertion, and that assertion is the note. The check joins the
records, the registry and the source files into one string beside it, and it
never reads that string. No failure of this check depends on a source file. A
pass in a worktree therefore means what a pass on the trunk means, and the
whole cost of the defect is the note.

### FND-195 — The record check builds a corpus of every source file and never reads it

**Believed.** The record check reads the source files once, joins them into one
corpus with the records and the registry, and asks that corpus which records a
source file cites. A reader of the script sees the corpus built and infers that
it feeds the check below it.

**True.** Nothing reads the corpus. The script assigns it and never names it
again. The note that asks which records are cited re-reads every source file
instead, once for each record it tests, inside the loop over the records. The
work the corpus did is thrown away, and the work it would have saved is done
again.

**The corpus was dead in the commit that created the script.** It is not a
reader that a later change removed. It was never read.[^F192B]

**Evidence.** A search of the history for the name returns one commit, the one
that introduced the whole script. The collection was lifted out of the script
and run against the trunk on a development machine. It joins 34 megabytes of
source into one string, and the loop beside it makes 4644 file reads over 2162
files. The reads stop early for a record that a source file cites, so the count
follows how many records no source file cites.

**Follows.** Three things.

**This is a second defect in one function, and it is not the first one.** A
separate finding records that the same collection drops every file when the
check runs inside a worktree.[^F194C] The two are independent. Repairing the
skip makes the corpus full and still unread.

**A dead assignment reads as an assertion.** The corpus names the records, the
registry and the source files together, which is the shape of a check. Nothing
tells a reader that it holds no claim. This project already records that a
value which is read back correctly and changes nothing is the failure that
hides best.[^22]

**Cost is not the reason to repair it.** The check runs in continuous
integration and a reader waits for it. The reason is that the script states one
thing and does another, in the place a contributor looks to learn what the
check asserts.

### FND-202 — A growth policy lived in a completed backlog item, where nothing read it

**Believed.** The rule that governs how the agent protocol server grows was
recorded, and a worker who needed it would find it.

**True.** The rule existed in one sentence, in the closing section of the
backlog item that built the first slice of the server: add a tool when an agent
needs it, and not before.[^F202A] That file sits in the completed directory. No
product record and no decision record held the rule. The three product records
about watching a world all serve a person at a window, and none of them
mentions an agent at a protocol.[^F202B] [^F202C] [^F202D]

The project owner considers the rule binding and stated it as the original
intent. Nothing a worker reads before starting work carried it.

**Evidence.** A search of the tree for the rule returns the one sentence, in
`complete/`. In the same round, one worker designed a change to the viewer,
went looking for the detail the change would need, and reported that the server
could not carry it, without any sign that a rule already governed what to do
next. A second worker was then dispatched to close the gap and had to be told
the rule in the dispatch message, because it could not be read.

**Follows.** Three things.

**This is the project's own recurring shape, applied to a rule.** One fact
stored where nothing reads it, with nothing that fails when a reader misses
it.[^F202E] The registers exist because a fact in prose decays silently, and a
rule in a completed work item is a fact with no reader at all.

**A completed backlog item is not a place to record a rule.** It is the correct
place to record what a change cost and what it left open, which is what the
rest of that section does. A rule that governs later work outlives the item,
and it belongs in a record.

**The repair is a record, and it names the audience.** The growth policy is a
constraint on a surface, so it is a decision record.[^F202F] The need behind it
is a question an agent could not answer, so it is also a product record, and
that record names an audience this directory had not served before.[^F202G]

### FND-203 — The control plane could not read the ground, so a fixture recorded it in a comment

**Believed.** The seed of the gather fixture in the agent tests was chosen for
a reason a reader could check.

**True.** It was chosen against the engine's own read, from Rust, and the
result was written into a comment beside the fixture. The comment says so
plainly, and it says why: no read told Python where a resource sat, so nothing
in the test could ask. The comment names three addresses that hold food under
one seed, and two that hold stone under another. Nothing checked any of it
again.

**Evidence.** The comment is in the agent test file and it states the gap as a
finding rather than an accident. The first version of that test used a seed
with no food at the addresses it gathered from, and failed for that reason.

**Follows.** Two things.

**A measured fact in a comment is a fact with no defence.** The generator is
derived from the seed, so the claim is true until any change to terrain
generation, and nothing fails on the day it stops being true. This is the same
shape as a count in a decision record.[^F202E]

**A reader that closes the gap turns the comment into a check.** The tile read
now reports what the generator gave, what units took, and the difference, at
one address. The claim about the seed is now an assertion in the same test that
depends on it, so the day it stops being true is the day the suite goes red.

### FND-204 — A world where every tile admits a unit hid a swapped read

**Believed.** A test that compares a level 1 cell against every tile of the
world, one at a time, checks that the cell reports what it summarises. The
comparison covers each field it names.

**True.** It covered one of the two fields it named. The fixture built an 8 by
8 world of hill and mountain, in which every tile admits a unit. The count of
tiles and the count of open tiles were therefore the same number. A read that
returned the open tiles where the tiles belong satisfied the assertion, because
both sides moved together.

**Evidence.** The defect was put back in the binding and the whole agent suite
stayed green, at exit code zero, with no test failing. The same run caught six
other defects, each by the test written for it. A search over seeds with the
engine's own census found that seed 51 gives three tiles of water in the same
extent. The fixture moved to that seed, the defect was put back again, and the
test then failed.

**Follows.** Three things.

**The rule already existed and the test still shipped with the hole.** The
testing rule says that a uniform input hides a defect, and that putting the
defect back is the only proof a fixture reaches the case.[^23] The register
holds two earlier instances, in two subsystems.[^11] [^F204C] This is a
third, written after the rule, by someone who had read it. **The rule is not
enough on its own.** Only the probe found this.

**A fixture chosen for convenience is chosen for the wrong reason.** The seed
came from a neighbouring test, where it was correct. Nothing asked what
distribution this assertion needed. The question to ask of a fixture is what
value would fail, not what world is nearby.

**A test can now say when its fixture stops reaching the case.** The repaired
test asserts that the open tiles are fewer than the tiles, and that the window
holds more than one kind of ground. Those assertions fail if a change to the
generator makes the world uniform again, so the hole cannot come back quietly.

### FND-214 — A reserved row is a guess that a constraint exists, and half of them were wrong

**Believed.** Eighteen registry rows reserved for the log and for the Python
boundary described eighteen constraints waiting to be written down. Two refined
backlog items said so: each named its range of rows and listed the whole range
under what it creates. The rows were derived from two long drafts by giving each
decision of a draft its own number, so a reader could reasonably read the count
of rows as a count of claims.

**True.** Nine of the eighteen stated no constraint that this project can hold.
Seven were written, and two already had files. The nine that went failed in four
distinct ways, and the pattern is worth more than the count.

- **One stated the alternative that another row refuses.** The rejection of
  classic event sourcing is what the arena record rejects, and the scope rule
  puts a rejected alternative inside the record that rejects it.[^F214A]
- **Two restated a constraint an accepted record already holds.** The barrier
  concatenation is two decisions of the record on parallel stages, and the
  region aggregate is the same record seen through a vocabulary this project
  does not use.[^F214B]
- **Four described a mechanism for which nothing had chosen anything.** No
  command queue, no rejection summary, no snapshot and no save format exists,
  and no product record asks for one. This is the failure that retired the
  first number the project ever retired, and the row for it already names the
  shape.[^F192A]
- **One guarded a hazard the project does not have.** Three layers of view
  safety defend a borrow of engine memory. Every read across the boundary
  copies, so no borrow is handed out and nothing was being defended.[^F214D]

**Evidence.** Each row was read against the three-condition test of the record
scope rule, with the removed draft that the row came from open beside it, and
with the code the row would govern. The tree was searched for every citation of
each of the eighteen numbers: six numbers are cited by a source file, a build
manifest, a gate script or a Python test, and every one of those six was
written. Of the nine retired, one was cited, by an accepted record that cited it
for its absence and said so in its own text.

**Follows.** **A reserved row reserves a number and asserts nothing.** The
registry already says this and the record priority index already says it twice.
This entry is the measurement that says how often it matters: for one block of
rows derived from drafts rather than from code, the yield was half.

**Derive a row from a claim and it can still be a topic.** The registry was
re-derived from claims specifically to avoid topic titles, and it worked: none
of the eighteen was a topic. Seven of them were nonetheless not constraints. A
claim title bounds a record. It does not prove that the record should exist.

**A citation from a source file was the strongest single signal.** Every row a
source file, a manifest, a gate or a test cited passed the test, and every row
nothing cited outside the registry failed it or was one of two written for a
reason the code cannot yet show. That is one block of rows and not a law, and it
is worth testing against the next block.

### FND-215 — The control plane names one entity of a mass shape in four places

**Believed.** The control plane rule was unenforced but not yet broken. The
records say nothing at the Python boundary reads the tier, so the rule is prose,
and the repair is the enforcement that ADR-0043 describes.[^F215A]

**True.** The rule is already broken, by the interface itself, in four places.
Enforcement would not have prevented three of them, because the engine offers
them deliberately and a caller reaches them by calling a documented method once
for each member of a set.

**Evidence.** Both the soldier shape and the settlement shape declare the mass
tier in the core crate, and the soldier's declaration carries its reason in the
same comment: a soldier is one of a million, so no caller walks the
population.[^F215B]

The bindings answer the tile of one soldier, the positions of one site and the
preference of one site. The agent server wraps the first as a tool that reads one
unit.[^F215C] [^F215D] A caller that wants any of those answers for a set calls
once for each member, so the crossing count is a function of the population,
which is the case the control plane record refuses in its own checkable
form.[^F215E]

**The type stub directs a reader into the pattern.** The documentation of the
gather event columns tells a reader to take a value from the unit column and hand
it back to the per-unit read.

**A test pays it four times for each site.** The thread count test for the site
positions sets a preference one site at a time, then reads the positions of each
site twice. Eight sites cost twenty-four crossings. It does that because the
set-valued command takes one target for the whole set and the test wants a
different target for each site, and because no read answers for a set.

**Follows.** **A set-valued verb is not a set-valued interface.** The project
took the owner's rule that a command takes a set and applied it to the commands.
The reads kept the singular form, and a read is a crossing like any other. Half a
rule applied is not a weaker version of the rule; it is a boundary that still
grows with the population, through the half nobody looked at.

**A command that takes a set and one value still forces a loop.** The preference
command is set-valued and cannot say a different thing about each member, so a
caller that needs per-member values sends one command for each member. The set
form removed the loop from the signature and left it in the caller. This is the
same shape the register already holds: a principle satisfied in form and not in
substance.[^F215F]

**This is FND-147 again, at the read rather than at the write.** That entry
records that a rule which forbids a shape and offers no alternative loses to the
absence of the read. The alternative is still absent, and the sweep it produces
is still written by people who know the rule.[^F215G]

**The enforcement record would not have caught it.** ADR-0043 describes a
refusal raised at a caller's loop over a set. Three of these four are one call
that the engine documents and answers. Nothing would refuse them, because the
engine offers them.

### FND-216 — The declared tier of a mass shape reaches no code in either crate

**Believed.** The tier is declared once and the core crate checks it when it
builds the storage of a shape. The gap is that the tier reaches no code outside
the core crate, so nothing at the Python boundary reads it.[^F216A]

**True.** The tier of a mass shape reaches no code inside the core crate either.
One shape out of three reads a declared tier: the character arena reads its own
ceiling to size itself at construction.[^F216B] The mass tier states no ceiling,
by design, because the mass tier is the one no ceiling bounds. So the check that
consumes a tier has nothing to consume for a mass shape.

**Evidence.** The soldier module and the settlement module contain no mention of
the tier, the trait or the constant. Their declarations live in the tier module,
and a search of both modules returns nothing. The only reader of a declared tier
is the character arena's ceiling.

**Follows.** **A declaration is not a mechanism, and this one reads as both.**
The tier is an associated constant on a sealed trait, which is the shape of an
enforced thing. It is enforced for one shape and inert for two, and the two it is
inert for are the two the whole rule is about.

**A record said so, and the sentence was believable because it was nearly
true.** The tier record's context claims the core crate checks the tier when it
builds a shape's storage. That is a correct description of the character arena
and a false description of the soldier and settlement arenas. A reader concludes
there is a mechanism to extend to the boundary, when there is a mechanism to
build.

**This is the inert-capability shape, in a place the project already trusted.**
The rule says to ask who is obligated to invoke a declared capability.[^65]
Nobody invokes the tier of a mass shape. It passes no test of its own because
there is nothing to test, and the compile-time assertion that exists asserts the
character tier.

**The repair is not to delete the declaration.** The declaration is the single
site the boundary rule needs, and a second site at the boundary is the failure
the tier record's D1 refuses. The repair is to make something read it, and the
review that found this says so.[^F216D]

### FND-217 — The tiles are stored row by row, and one record reads the reserved row as settled

**Believed.** The tiles are stored in blocks at the aggregation block size, and
the summary pyramid divides the world into the same blocks, so a structure that
walks blocks walks the storage in memory order.

**True.** The pyramid divides the world into blocks. The tile columns are stored
row by row. The grid derives a tile index as the row times the width plus the
column, and that is the only storage order the engine has.[^F217A]

A block-major space does exist, and it is a different space. The block layout of
the unit-to-tile bridge builds a key by interleaving the block of an address with
the position inside that block, and the engine converts between that key and the
tile index.[^F217B]

**Evidence.** The pyramid's own fold says it, in its own comments. Summarising
one block reads one contiguous run for each row of the block, not one run for the
block, because a block is a rectangle over a row-major column.[^F217C] A block of
edge `n` costs `n` runs.

**Follows.** **The reserved row for a block-tiled layout has no record and no
implementation, and the corpus is otherwise careful about it.** Three accepted
records cite that row and every one of them speaks conditionally: the storage
record says the derivation is arithmetic whichever order the tiles sit in, the
tile index record says the record holding the memory order may choose a block
order rather than a row order, and the bridge record cites it without asserting
it.[^F217D] [^F217E] [^F217F]

**One draft asserted it as present fact, and built a cost case on it.** The
selector range record says tiles are stored in blocks at the aggregation block
size, and its D3 concludes from that that a verb reads a run rather than
gathering scattered values, and that the engine maps no space onto another. Both
conclusions fail under the storage that exists.[^F217G]

**A reserved row accumulates belief.** Nothing about the row changed. It was
cited carefully three times, and the fourth citation read the careful ones as
evidence that the thing was true. This is the same shape as a summary going
stale, arriving from the other direction: the register did not decay, the reading
of it did.

**A row that governs a layout should say what the layout is today.** The registry
row states the claim the record would make and says nothing about what the code
does instead. A reader who wants to know how the tiles are stored has to read the
grid.

### FND-218 — The retcon window governs what a record claims, not where its pointers go

**Believed.** An accepted record does not change except in status, and the
retcon window is the only door. Any edit to an accepted record therefore has to
pass all three of its conditions, and the first is that nothing depends on the
record yet.[^8]

**True.** The window governs an amendment, which is a change to what the record
claims. A citation is not a claim. Repairing a pointer that became false through
no fault of the record is not an amendment, and the three conditions do not
apply to it.

**Evidence.** A review retired nine reserved registry rows. One of them,
`ADR-0036`, was cited once by ADR-0067, which is accepted, in a footnote, and
cited for its absence: the record says an alternative design needs a snapshot
mechanism that no record holds.[^F218B]

The number is written in a code span above, because a retired number is
mentioned and never cited, and the citation check enforces it. **This entry
broke that rule on its first draft and the gate caught it.** That is a second
instance of the cost the register already carries: a document that must explain
why a number went cannot name the number the ordinary way.[^F218F]

Retiring the row made that footnote name nothing. The record check fails on a
citation of a number that no record file and no registry row has, so the
citation was also a red gate.

**The record's first condition could not be met and the other two could.**
ADR-0067 has a dependent, so the window as written was shut. The edit was made
anyway: the footnote now names the registry's retired numbers, and the body
sentence it supports is unchanged and still true.

**Three roads were open and two were worse.** Leaving the citation gives an
accepted record that names a number holding nothing, which is a record that lies
with the authority of an accepted one. Writing a superseding record whose entire
content is a corrected file path is a record for a topic and not for a
constraint, which is the failure the scope rule measures against.[^16] The
third was to repair the pointer, which changes nothing a reader decides.

**Follows.** **The registry now states the rule, and this entry holds the
case.**[^F218D] The test is whether the edit changes what a reader would decide.
Repointing a footnote at the row that now holds the material does not.
Rewording the sentence that carries the marker does, and the window governs
that.

**The rule cannot be used to smuggle a change through.** Adding, removing or
reversing a claim, a force, a rejected alternative or a consequence is an
amendment however small the diff and whatever else the commit touches. A commit
that repairs a citation and edits a sentence in the same breath is an amendment,
and it is reviewed as one.

**A retirement is a sweep, and the sweep reaches accepted records.** The rule
that a closed blocker must be searched for across the tree already exists, and
this is the same operation for a retired number.[^F218E] The difference is that
a retired number fails a gate and a closed blocker does not, so this one cannot
be forgotten. That is luck rather than design, and the register records which of
the two it is.

### FND-233 — Four cell fields with incompatible ranges go into one argmax, so a weight is a preference times an unwritten unit conversion

**Believed.** A weight expresses how much a unit prefers an option. The default
profile gives every option a weight of one, which was read as no preference.

**True.** The score of an option is the drive times the weight times the field
the option reads. With every weight at one, the score is the drive times the
field. **The four fields do not share a range**, so the comparison is decided
partly by which field carries the larger units.

- The open share is a count over a count, bounded at one by construction.
- The mean food is a stock over a tile count, and its ceiling is whatever a
  tile can hold.
- The units for each open tile is a count over a count, and its ceiling is the
  capacity of a tile.
- The mean height is a mean over the height range.

**Evidence.** A short unit of the demonstration was seen to choose roam while
standing on ground that carried food. Its need was 0.48, so roam took the met
drive at 0.48 and forage took the unmet drive at 0.52. The open share of its
cell was near one, giving roam a score near 0.48. The mean food of its cell was
0.669, giving forage 0.348. **Nothing is broken. The unit preferred open ground
to food, because a share of one outscores a mean stock of two thirds.**

**The comparison works in this world by coincidence.** The food generator puts
the mean stock near one, so a share and a stock happen to be comparable. Raise
the ceiling of a tile and forage wins everywhere, whatever a unit needs,
because the mean food climbs past one and a share cannot follow.

**Follows.** **A weight is not a preference. It is a preference multiplied by a
unit conversion, and nobody has written the conversion down.** A content author
who sets a weight of two for forage cannot tell whether they doubled a
preference or corrected half a scale error, and the two are not distinguishable
from the value.

**No fix is proposed.** Whether the fields are normalised, or a weight carries
its scale explicitly, is a design decision and it needs a record. This entry
records the fact.

### FND-234 — The exit field does steer the crowd, and a prediction from one frame said it did not

**Believed.** A pre-registered prediction, written before the run: hungry units
would not stand on better ground than fed ones, most would choose roam over
forage, and only a minority in food-rich cells would forage. The prediction
came from one rendered frame in which one short unit chose roam, and from the
scale defect that explains why it did.[^F234A]

**True. The prediction is refuted on both counts.** Over eight hundred ticks of
the demonstration, at three sampled ticks:

- Hungry units stand on cells whose mean food is 1.041, 0.781 and 0.876.
- Fed units stand on cells whose mean food is 0.243, 0.224 and 0.257.
- The world mean over a spread sample of the same cells is near 0.61.

**A hungry unit stands on three to four times the food that a fed unit stands
on**, and well above the world mean, while a fed unit stands well below it. The
share of hungry units choosing forage is 28 in 48, then 34 in 48, then 36 in
48. **It rises with time rather than holding.**

**Follows.** The field steers. A unit that needs food moves onto ground that
carries it, and the population separates into two by where it stands.

**What the measurement does not settle.** The hungry units belong to the two
poorest sites and the fed units to the two richest, so a raw comparison between
the two groups confounds steering with where their sites sit. The world mean is
the baseline that survives that, and both groups sit far from it in opposite
directions. The rising share of foragers is the second independent signal,
because geography does not change with time and that share does.

**The mean food under the hungry crowd falls and then recovers**, from 1.041 to
0.781 and back to 0.876. That is consistent with a crowd that depletes where it
stands and moves on while the ground behind it recovers, which is the negative
feedback the work was built for. It is consistent, not proven.

**The lesson is about the prediction, not the engine.** The frame that produced
it was real and the unit in it was in the quarter that chose roam. **One frame
generalised to a population is the same error as one render generalised to a
world**, which this register already holds twice.[^F201C] The pre-registration
is what makes this entry cheap: the prediction was written down before the run,
so its refutation is a fact rather than a memory.

### FND-231 — A unit draws from its home site wherever it stands, so nothing it does can make it hungry

**Believed.** A unit that walks away from the site it belongs to gets hungry,
so a demonstration in which units migrate would show hunger somewhere. The
brief for the work that found this asked for a population fed near its site and
hungry away from it, which assumes distance costs a unit something.

**True.** **Feeding has no distance term anywhere.** The draw is keyed on the
home site of a unit and on its faction, and on nothing else. A founding sets
the home site of every person it seats, once. A unit seventy-three tiles from
that site draws from its store exactly as one standing on the site does, and it
keeps drawing for as long as the store pays.

**Evidence.** The demonstration was run for twelve hundred ticks with four
groups of thirty. Every unit was fed at every sampled tick, no unit was ever
short or starved, the ration never failed, and not one tile of the world was
ever gathered from. Over the same run the mean distance from a founding rose
and the furthest unit reached seventy-three tiles. **The units migrated and
stayed fed.**

**Follows.** **A unit cannot be made hungry by going anywhere.** Hunger is a
property of the site a unit belongs to and of the group that site carries, and
a unit changes neither by moving.

**The consequence reaches past the demonstration.** The forage option scores
against the deficit of a unit. If no movement can produce a deficit, then no
movement can drive the option, and the option cannot close a loop with
movement. Two open items exist to make a step read the option a unit chose and
to let the engine order a gather.[^F226A] [^F226B] Neither can produce a unit
that grew hungry by travelling, because nothing does.

**It also bears on a strategy that a unit would follow.** A record under
consideration holds that a strategy is a field a unit follows. **"Return to my
site" is the first strategy anybody will want, and today a unit has no reason
to return**, because being away costs it nothing.

**This is why the engine tests are built the opposite way from the
demonstration.** The starvation fixture gives every second site a store that
empties and no production, and says in its own comment that the demonstration
world is chosen to look right and that every unit in it eats. That fixture
reaches the hungry case by choosing the site, which is the only handle there
is.[^F231C]

**No fix is proposed here.** Whether feeding gains a distance term is a design
decision and it needs a record, not a repair. This entry records the fact and
what it forbids.

### FND-232 — The demonstration fed every unit forever, so the food layer decided nothing a watcher could see

**Believed.** The demonstration exercises the food loop. A watcher of the
running window sees units that gather, ground that empties, and a choice that
varies with what the ground carries.

**True.** No unit of the demonstration was ever hungry, so the forage option
scored zero on every unit on every frame whatever the ground held, and **not
one tile of the world was ever gathered from.** The window showed a food layer
that decided nothing. The panel stated it plainly and it was read as a healthy
world: the nearest unit reported the food under it and a score of zero for the
option that reads that food, because it was fed.

**Evidence.** Twelve hundred ticks, four groups of thirty. Fed at every sampled
tick, none short, none starved, no ration ever failed, and zero tiles of two
hundred and eighty-one thousand six hundred showed any depletion. The site
stores grew without interruption for the whole run.

**Two candidate causes were eliminated by measurement before anything
changed.** It is not the starting store: **at the first tick every store is
zero and every unit is still fed**, so an empty store does not make a unit
hungry, and the repair anyone would reach for first would have changed nothing.
It is not the declared upkeep rate either, which is zero at every group size,
because the founding never sets one. **A cohort does not draw the declared
upkeep. It draws its own ration from the store.**

**The relation that governs it is exact.** The founding sets the production
rate of a site to a sixteenth of the food its survey reached, and the default
need rule gives a person a ration of a sixteenth of a full need for each
application. The two cancel. **A site feeds exactly as many people as the food
its survey measured**, and that number is one the founding already prints.

**Follows.** **A fixture that produces one condition everywhere measures
itself.** The group size now falls inside the spread of what the founded sites
reach, so some ground cannot carry its group and other ground can, and a
watcher sees both conditions at once. The split follows the ground rather than
the number.

**A run that loses the split now says so.** The founding report states for each
site whether its ground carries its group, and the run prints a note when every
seated group came out the same way. The defect that this entry records is that
nobody noticed for as long as the demonstration ran, and a silent fixture is
what allowed that.

### FND-219 — The next-number line reflects merged state, so it cannot allocate across branches

**Believed.** Each register carries an explicit next number, and a writer claims
it before writing the row. That remedy was installed after two writers collided
on FND-035 and DEC-013, and it was believed to close the class.[^1]

**True.** The remedy holds for writers who commit between claims. It does not
hold for writers on separate branches, and the project now works that way by
default.

**The line reflects merged state and nothing else.** A writer on a branch reads
it, takes the number, and writes the row. The line does not change for anybody
else until the branch merges. Six workers on six branches read the same number
on the same day, and every one of them followed the documented procedure
correctly.

**A second authority exists and no register can see it.** The dispatcher holds
ranges and issues them in prompts. A prompt is not a file. No writer can consult
it, no check can read it, and it disagrees with the line by construction,
because the line answers from merged history and the range answers from work in
flight.

So one question has two authorities, neither is observable to the other, and
both give a correct answer to the question they were asked.

**Evidence.** Four collisions in one session. Two workers filed FND-198 and
FND-199 for different subjects. One worker took `FND-209` from the line while
the dispatcher had reserved it. The same happened to FND-218. One worker
renumbered its FND-198 and FND-199 to `FND-202` and `FND-203` to get out of the
way.

**Three of those numbers are written in code spans, and the reason is this
entry's own subject.** The citation check resolves a register number against
the rows this branch holds, and those rows live on branches this one cannot see.
Citing them the ordinary way turned the gate red. So a finding about numbers
being invisible across branches cannot cite its own evidence, for the same
mechanical reason that a document explaining a retired number cannot name
it.[^F218F] That is a third arrival of one cost, and it is the cheapest possible
demonstration of the claim above.

**This branch can verify one of the four.** FND-198 and FND-199 stand here under
one worker's subjects, and this entry's own number was reported as reserved by
somebody the register cannot name. The rest is the dispatcher's account, because
no branch can see another branch. **That is not a weakness of the evidence. It is
the finding.** A writer who wanted to check whether a number was taken has
nowhere to look.

**Follows.** Four things.

**A remedy carries a scope that nobody states.** FND-038 fixed the allocator for
serial writers, because serial writers were what the project had. The fix was
correct and it is still correct. What went unrecorded is the condition it rests
on: that a claim becomes visible to the next writer before the next writer
claims. Parallel branches removed that condition without anybody deciding to.

**The failure is invisible at the moment it happens.** A collision costs nothing
when it occurs. Both writers are correct, both rows are good, and both branches
are green. It surfaces at the merge, and by then two documents cite two
different rows under one number.

**The cost is the renumber, not the collision.** Moving a row means finding every
citation of the old number across records, registers, reviews, backlog items and
source comments. That is the sweep this project gets wrong most often, and one
of the four collisions has already required it.

**This is the third face of one shape.** A record's status is carried by the
directory its file sits in, so accepting it breaks every citation of the
path.[^F197C] A retired number is a row that the citation check cannot resolve,
so a document explaining why a number went cannot name it.[^F218F] A number is
allocated by a line that cannot see uncommitted work. In each case the registry
stores a piece of state somewhere that cannot be read atomically by everyone who
needs it, and in each case nothing fails until two readers disagree.

### FND-226 — The demonstration feeds every unit, so it never forages

**Believed.** The engine now steers a step by the option a unit chose, and it
writes a gather order for the unit that chooses to forage.[^F226A] [^F226B]
The demonstration runs the same engine, so a watcher was expected to see units
walk to food and take it.

**True.** No unit in the demonstration ever forages. The `forage` row is driven
by what a unit lacks, and the demonstration founds each group on a site whose
store feeds it.[^F226C] Every unit therefore holds a whole need on every frame,
the drive of the row is zero, and the score is zero whatever the ground
carries. Every unit chooses `roam`, which reads the share of a cell that admits
a unit.

**Evidence.** A run of the demonstration world was measured on a development
machine, at one, sixty, two hundred, six hundred and one thousand two hundred
ticks. At every reading the mean need was the full need, every live unit held
the `roam` option, no unit held a gather order, and the depletion ledger held
no entry. The panel of a founded run states the same thing in one card: the
nearest unit reads a food value of 0.86 from its cell and scores 0.00 for the
option that reads it, because it is fed.

**Follows.** Two things.

**The chain of items 0185 and 0186 is real in the engine and inert in the
demonstration.** The engine tests drive a hungry unit and it forages, gathers,
and works a deposit down. The demonstration supplies no hungry unit, so the
negative feedback never engages and a watcher sees one behaviour and not the
loop.

**The migration a watcher does see is toward open ground.** The `roam` row
reads a property of the ground, which no system writes, so the field it steers
by never changes. The units walk to a local maximum of open ground and stop
there. A directed walk is visible: over three hundred ticks the mean distance
from the starting tile rose from 13 tiles under the uniform draw to 36, and the
furthest unit from 40 tiles to 74. Both figures are one run each, on a
development machine, at the same seed and settings.[^28]

The repair is a demonstration parameter and not an engine change. An item holds
it.[^F226E]

### FND-227 — A fixture in which two options point one way cannot see a pinned option column

**Believed.** A test that drives the step, reads the exit direction of a cell,
and names the tile a unit must reach proves that the step read the option of
that unit. The register recommends exactly that falsification: pin the value to
a constant and watch the suite stay green.[^F183A]

**True.** The first fixture failed the falsification. The option column was
pinned to option zero at the site where movement reads it, and every test
stayed green. The cell under test pointed the same way for the pinned option as
for the option the test set, so the assertion could not tell the two apart. The
test measured the fixture.

**Evidence.** The pin was applied on a development machine and the whole suite
of both new test files was run. Ten exit field tests and six gather tests
passed. The fixture was then rebuilt to choose a direction that no other option
points at, and the same pin failed the movement test. Three further pins were
run separately, each with the source restored afterwards: the uniform draw put
back in place of the field, the engine write of the gather order removed, and
the map from the option to the resource kind set to nothing. Each failed the
tests that name it.

**Follows.** This is the shape the register already holds, on a new value. One
finding says to pin the site the consumer reads.[^F227B] This adds the other
half: **the fixture must separate the value under test from every other value
the consumer could have read.** A pin reaches the consumer and still proves
nothing when two inputs give one answer.

The rule generalises to any lookup keyed on a small set. Build the fixture so
that each key gives a different answer, and assert that the other keys do not
give the answer under test. The exit field test asserts that now.

### FND-228 — An engine writer of a control-plane column replaces what a caller wrote

**Believed.** Adding an engine writer to the gather order column leaves a
caller that sets the order from the control plane working as before. The verb
stays, and the item that added the writer states the precedence in prose.

**True.** The engine write replaces the caller write on every frame that the
level 1 cell of the unit chooses. A probe fixture set the choice interval to
every tick and then ordered a gather from outside, so the choice replaced that
order before the resolve read it. The contest the fixture existed to build
disappeared, and the test that proves the gather sort can fail stopped failing.

**Evidence.** The determinism probe run under its feature reported one failure
after the engine write was added, in the test that asserts the gather sort has
a proven failure mode. The fixture now puts the choice far enough apart that no
cell of a gatherer chooses on the frame under test, and it asserts that rather
than assuming it. Nineteen probe tests then pass.

**Follows.** **When a pass gains a second writer of a column, every fixture that
wrote that column from outside is now a fixture about precedence.** Search for
the callers of the setter, not only for the readers of the column. A fixture
that sets a value and then runs a frame is the shape that breaks, and it breaks
quietly: the value is read back correctly and it is the wrong value.

### FND-250 — A behavioural strategy is one constraint, not three

**Believed.** A behavioural strategy decomposes into three parts that each need
a record: a trigger, which is a condition on unit state; a field to follow,
never a per-unit search; and an action on arrival, which is what happens when
the field runs out.

**True.** Only the middle part states a constraint this project does not
already hold. The other two fail the scope test for different reasons, and one
of them also misdescribes the engine.

**Evidence, part one: the trigger is already recorded, and it is not a condition
on unit state.** A unit takes an option by scoring a fixed set and taking the
highest, and an accepted record governs that.[^F250A] The score is the unit's
drive multiplied by a weight and then by the value the option reads from the
level 1 cell.[^F250B] It reads unit state **and** world state, so a trigger
described as a condition on unit state describes something the engine does not
do. A record stating it would be a second declaration of the choice pass and a
wrong one.

**Evidence, part two: arrival is partly answered and partly unbuilt.** A unit
holds its intent until it chooses again, and a cell that no neighbour beats
strictly holds no exit direction, so a unit there keeps the behaviour it already
had.[^F190C] That is what happens when a field runs out, and a draft record
already says it. What is left is what a unit does at a destination, and the
engine has no destination, no field and no concept of arrival, so nothing has
chosen anything. That is the shape that retired the first number this project
ever retired.[^F192A]

**Follows.** **A three-part framing of a mechanism is not three claims.** The
parts of a mechanism are how a builder thinks about it. A record binds a choice
somebody could get wrong, and the three parts here have three different
statuses: one settled, one open, one not yet a question.

**The test that separated them is the ordinary one.** Ask of each part whether
a contributor could reasonably choose otherwise, whether choosing otherwise
costs more than changing it later, and whether the reasoning is invisible in the
artefact.[^16] The trigger fails the first, because the choice is made and
recorded. The arrival fails the first in the other direction, because nothing
has been chosen at all.

**A plausible decomposition is the most expensive kind of wrong framing**,
because each part sounds like it needs a record and the reviewer's instinct is
to write three. The evidence that separates them is in the source and in the
registry, not in the framing.

### FND-270 — Slot order and identity order agree in an ordinary fixture, so a test cannot tell them apart

**Believed.** A test that asserts which unit took the first position proves
that the assignment follows the identity of the unit rather than the order the
units were read in. The two are different rules, so a test that names one of
them distinguishes them.

**True only when the fixture separates the two orders, and an ordinary fixture
does not.** A unit spawned earlier holds both a lower slot and a lower
identity, because an identity is a slot index with a generation above it and a
fresh arena hands out slots in ascending order. In such a fixture the two rules
give exactly the same answer.

**The evidence.** The assignment pass sorts its applicants by a key vector
whose last field is the whole identity. The first version of the test spawned
four units in order and asserted that the positions went to them in identity
order. **The sort was then replaced with the identity permutation, so the pass
seated in the order it read the units, and all eight tests passed.** The test
that existed to name the order could not see the difference.

**The repair is in the fixture and not in the assertion.** A slot that is freed
and filled again carries a higher generation, so the unit in the lowest slot
holds the highest identity. The fixture now spawns two units, despawns the
first, and spawns a third into the freed slot. Read order then names the third
unit first and identity order names the second, and the same mutation fails the
test.

**Follows.** Three things.

**This is the fixture shape the register already holds, in a new place.**
FND-051 and FND-048 in this register record a fixture that modelled the typical
case and so never supplied the extreme. This one is narrower and worth naming on its own: **any test
about entity order needs a fixture in which a slot has been reused**, because
generation is the only thing that separates slot order from identity order.

**A test about an order must be mutated, not read.** Nothing in the test looked
wrong. It named the right rule, asserted the right sequence, and was green for
the right-looking reason. Only putting the defect back showed that it was
measuring the fixture.[^23]

**The other three mutations were caught.** Removing the pass from the step
failed seven of eight tests, storing a bare slot index instead of an identity
failed the reuse test, and seating an already seated unit failed two. The suite
was sound apart from this one case, which is why the case is worth recording
rather than the suite.


### FND-269 — Three completed subsystems produce nothing at all in the demonstration world

**Believed.** The demonstration shows less than the engine holds because the
drawing has not caught up. The repair for a feature a watcher cannot see is
therefore drawing work, in the way that items 0216 and 0240 were fixture
work.[^F269A]

**True for some subsystems and false for three.** Positions, characters and
improvements do not merely go undrawn in the demonstration. **They never
happen.** A front end that drew them today would draw an empty set, and the
test that proved the drawing correct would be measuring the fixture.[^23]

**The measurement.** The world the demonstration builds, founded for four
factions with a group each, stepped on four threads. The counts are read from
the public readers of the engine at the tick named.

| Quantity | Tick 50 | Tick 200 |
|---|---|---|
| Units alive | 192 | 192 |
| Settlements | 4 | 4 |
| Position seats that exist | 16 | 16 |
| Position seats a unit holds | 0 | 0 |
| Characters | 0 | 0 |
| Upgrade sites | 0 | 0 |
| Units the shortage ended | 0 | 0 |
| Units carrying a load | 0 | 45 |
| Units whose chosen option is forage | 0 | 44 |
| Tiles held, summed over the factions | 7,866 | 46,992 |

**Each zero has a cause, and every cause is an open item.** Nothing seats a
unit in a position, which is what item 0063 does. Nothing promotes a unit into
the character tier, which is what item 0088 does. No option in the option set
orders a build, which is what item 0180 does. The subsystems are complete and
correct; what is absent is the caller that makes one instance of each occur.

**Every one of those three items sits in `Later`.** They sit there for reasons
that are each defensible on their own, and the priority index states them. The
consequence that no row states is the one the project owner names most often:
the demonstration cannot show a feature that never occurs, so the work that
makes the demonstration show its features is ordered below the work that makes
it faster.

**Follows.** Three things.

**Ask whether a feature occurs before asking whether it is drawn.** An audit
that reads the drawing finds the features the drawing omits. It cannot find a
feature the world never produces, because both look identical from the front
end: an empty set.

**A zero is the cheapest evidence this project has, and nothing prints one.**
Every figure above came from a throwaway probe that no test holds and no
command runs. The demonstration prints whether a founded ground carries its
group, and that line exists because a silent fixture cost the project a
round.[^F269A] It prints nothing about a subsystem that produced no instance.
Item 0278 asks for that.

**The three items are not equivalent in cost.** Item 0063 writes into a
structure that exists and holds sixteen empty seats already. Item 0088 has no
stated blocker. Item 0180 adds an option to the pass that item 0238 is
rewriting, and the index says it should read that pass afterwards. Moving the
first two is cheap; moving the third against a stated sequencing reason is not.

[^F269A]: Findings register, FND-232, in this document.


### FND-263 — The needs of one cell spread widest while a store empties, and they polarise afterwards

**Believed.** Two things, and both were reasonable. The first is that a need is
a fixed-point quantity, so two units in one cell almost never share one and a
key on the exact need reduces nothing.[^F263A] The second is that no fixture in
this project produces the distribution, because it needs settlements, home sites
and a running economy and the benchmark world holds none of the three.[^F263B]

**True, with a correction to the first and a replacement for the second.** A
fixture now exists and the distribution is measured.[^F263C]

**The measurement.** A world of 65,536 tiles, 64 level 1 cells, 64 settlements,
about 4,000 units with a home each, and the economy running on every tick. The
median cell holds about 75 units, which is the density the project states for
the target scale. Two placements bound the answer: **mixed** gives neighbouring
units different homes, **clustered** gives a run of neighbouring units one home.
The figures are the median cell. The collapse is the units of that cell divided
by the distinct keys in it.

| Frame | Distinct exact needs | Buckets at the matched width | Buckets four times finer |
|---|---|---|---|
| 4 | 14 | 5 | 14 |
| 8 | 26 | 9 | 26 |
| 16 | 41 | 17 | 41 |
| 32 | 24 | 11 | 22 |
| 64 | 12 | 4 | 9 |

The clustered placement holds 7 to 9 distinct exact needs at every frame.

**The first belief is false as stated, and its conclusion holds for another
reason.** An exact key does not give one distinct value for each unit. It gives
14 to 41 out of about 75, because a cohort is one site and one faction, and every
unit of one cohort draws one ration and holds one need exactly. **The distinct
keys of a cell are bounded by the cohorts standing in it**, and that is a
structural fact rather than a figure of this fixture.

**The bound belongs to the content, and that is what decides the question.** A
world that gave every unit a site of its own would put one key on every unit, and
nothing in the engine refuses such a world. **A bound a content author can remove
cannot carry a claim that the population must not raise the work.** The bucket
gives a bound the engine holds, so the conclusion that the key must be a bucket
survives. What does not survive is the reason given for it.

**The measured collapse says the same thing in the other direction.** At the
moment the needs are most spread, the exact key collapses the median cell 1.8
times. A pass that does half the per-unit work is not a pass whose cost follows
the lattice. So the exact key fails the claim on the measurement as well as on
the argument, and it fails it in a world whose cohorts are already sharing.

**The spread is transient and the steady state is polarised.** The decay takes a
fixed amount and the gain is a share of a store. A cohort whose share is below
the decay falls to the floor and stays. One whose share is above it rises to the
ceiling and stays. The values between the two are what a store produces while it
empties, so the spread rises to a peak and then falls as each site settles on one
side. **The peak is the case that decides the width**, and it is the case a run
of a few frames misses.

**Follows.** **The width is matched to the rate at which a need moves, and the
measurement is why.** The default need rule takes a fixed amount off a unit on
every tick. A bucket of that amount means a unit crosses one bucket in one tick.
A finer bucket separates two needs that the rule cannot separate inside a tick,
and the table above shows what that costs: four times finer than the matched
width gives 41 distinct keys against 17 at the peak, and the collapse falls from
4.4 to 1.8. **The finer bucket buys nothing and it is not free.**

**The decay is a parameter of the need rule, so the two are coupled.** A caller
who changes the decay and leaves the width alone has unmatched them. That
coupling is why the width is a parameter of the world and not a constant of a
module. An open decision closed against this measurement.[^F263D]

**The width changes where the population ends up, and the fixture showed it by
accident.** The measurement was taken twice, once at each of two default widths,
because the fixture reads the width the world holds. The median cell held 71
units at the finer width and 67 at the matched one, at the same frame of the same
seed. A unit takes its option from its bucket and its step from its option, so a
different width moves a different population. **No golden file sees any of
this**, and a separate finding records why.[^F263F]

**This is not a cost figure and no blocker governs it.** The simulation is
deterministic integer arithmetic, so every number above is the same on every
machine. The register that holds measured cost figures is for the target
platform, and this belongs in neither that register nor the derived one.[^F263E]

**What this does not measure.** Whether the matched width dithers a unit's
behaviour, and what the real placement of homes in a played world looks like.
The two placements bound the answer and neither predicts it.

### FND-262 — The choice quantised the need, and no golden scenario reaches the case it changes

**Believed.** Deciding the choice for each cell and each bucket of need changes
behaviour, so the golden state hash moves and the move has to be stated rather
than discovered. The item that asked for the work said exactly that, and the
record that decides it says so in its own text.[^F262A] [^F262B]

**True.** The change moves no golden hash. All eight scenarios match their
stored files, at the recording thread count, with the quantisation in place.

**Evidence, and it is direct.** The bucketing was removed from the pass, so that
the pass scored the exact need again, and the golden test was run against the
unchanged files. It passed. **No golden scenario reaches a need where the bucket
changes the answer.** The removal is the check, because a scenario that reached
the case would fail with the quantisation and pass without it.

The case exists and it is easy to reach. A purpose-built fixture searches the
need range against the summary its own cell holds, finds a need whose bucket
answers differently, and drives the engine to that need by naming the decay. The
unit then takes the answer of the bucket. The search asserts that it found a
divergent need before it asserts anything else, so a fixture that stopped
reaching the case would fail rather than pass quietly.[^23]

**Part of the reason is visible and part is not.** The default need rule takes a
sixteenth of the need range off a unit on every tick, and the bucket is a
sixty-fourth of that range, so a decay lands on a bucket bound exactly. A unit
that is never partly fed therefore holds a need that the quantisation does not
move. **Whether every unit of every golden scenario is in that condition is not
verified**, because the store divides among a cohort and a share need not be a
multiple of the bucket width. The removal experiment above proves the outcome
without proving the mechanism.

**Follows.** **A behavioural change can be real, checked and invisible to the
whole gate at once.** The golden files are the project's answer to "did the
simulation change", and here they answer no to a change that a record was
written for. A reviewer reading a green gate would conclude the change is inert.
It is not inert. It is unreached.

**A green golden file is evidence about the scenarios, not about the engine.**
This is the same shape the register already holds twice, where a fixture modelled
the typical case and supplied no extreme.[^F262D] The new part is that the
fixture here is the golden corpus itself, which the project treats as the
authority on whether behaviour moved.

**The width of the bucket can be varied over most of its range and no golden file
moves.** The width was set to four different values spanning a factor of eight,
and every scenario matched its stored file each time. So the gate is blind not
only to the quantisation but to the parameter that decides how much the
quantisation does. **That parameter is the mechanism of the decision**, and a
review of the governing record said so before any of this was measured. A
register holds the value and the measurement it was chosen against.[^F263D]

**The remedy is not a new golden scenario.** A scenario built to sit on a bucket
boundary would pin the boundary rather than the behaviour, and the record says
that a test which wants the option set states a need in the middle of a
bucket.[^F262E] The test that reaches the case belongs beside the decision, and
it is written there.

### FND-251 — There are no unit types, and one weight profile serves every unit alive

**Believed.** A unit type is an index into a shared table, and types
parameterise the verbs rather than multiplying them. This is one of the four
design principles the project orientation states.

**True.** No unit type exists. The soldier arena carries no type index and no
weight column, and the engine holds exactly one weight profile, as a single
field on the world, built with every option weighted equally.[^F251A]

So every unit alive scores the options identically. **Two units standing in one
cell with the same need always make the same choice**, and no mechanism can
make them differ, because there is nothing to differ by.

**Evidence.** The world holds one `WeightProfile` field, initialised to the even
profile, and the explanation path passes that one profile for every unit it
explains. A search of the soldier arena for a weight or a profile returns
nothing.

The option set compounds it. There are four options and every one of them reads
a level 1 summary field, so a unit's behaviour is a function of its need, its
cell, and nothing else. One of the five summary fields the option set can read
is read by no option, and the field that the `roam` option reads is derived from
terrain and never changes.[^F251B]

**Follows.** **A principle with no instance is a principle nobody has tested.**
The orientation states that types parameterise the verbs and that a type is an
index into a shared table. Nothing in the engine is a type in that sense, so the
principle has never had to hold, and the first thing that needs it will discover
what it costs.

**This is a different gap from the one about destinations, and it must not be
folded into it.** A field over cells lets a unit go somewhere. It does not let
two units want different things, and a record about fields does not touch this.
The product record that states the destination need says so in its own scope.

**Nothing is wrong today.** One profile is the correct amount of machinery for a
world with one kind of unit. The finding records what is true so that the next
person who reads the principle does not assume an implementation behind it,
which is the mistake this register keeps recording.[^65]

### FND-252 — The unit passes obey the rule about disjoint writes and still do not scale

**Believed.** A parallel pass that does not scale is contending. The accepted
record on parallel stages forbids two threads writing to one place and requires
the partition to come from the data, so a pass that scales badly must be
breaking it.[^F252A] [^F252B]

**True.** The unit passes keep that record completely, and they scale badly for
reasons the record does not govern. **Obeying it is necessary and it is not
sufficient.**

**Evidence, in the source rather than in a profile.** The choice pass gives each
thread a contiguous chunk of the live units and one output slot of its own, and
it joins the slots in index order.[^F252C] That is the required shape, and
nothing in it contends.

Three things cost it anyway.

It collects every live unit into one list before any thread starts. That is
serial and it grows with the population.

It applies the results afterwards by walking the collected list and writing an
intent for each entry. That is serial too, and it also grows with the
population.

Inside the parallel part, each unit reads its tile, converts the tile to a cell,
reads that cell's summary and reads its own need from a column. The arena is in
spawn order and never compacts, so consecutive units in a chunk touch scattered
tiles, scattered cells and scattered needs. The choice record already states
that condition and declines to claim the locality as a property of the
engine.[^F252D]

**Follows.** **The axis is the thing, and the record about writes does not
choose the axis.** Partitioning by unit index satisfies every requirement of the
parallel-stage record and destroys locality in the same move. Partitioning by
cell satisfies the same requirements and keeps the reads contiguous, and it
satisfies them as a side effect rather than as a constraint to be met.

**A rule that is necessary invites the reading that it is sufficient.** The
parallel-stage record is correct, it is well written, and it is about
correctness under a weak memory model. Nothing in it claims to be about scaling,
and nothing in it warned that a compliant pass could scale badly. A reviewer
holding it would have passed the choice pass, because the choice pass passes.

**Two serial phases were hiding in a pass everyone called parallel.** The
collect and the apply are both O(population) and neither is threaded. A
description of the pass as parallel is true of its middle and false of its ends,
and the ends are what a thread count cannot help.

**The general shape.** When a pass does not scale, read what it is indexed by
before reading what it contends on. Contention is the failure that a rule
already prevents, so it is the least likely one to be present in a project that
has the rule.

### FND-258 — Every document check passes with a false sentence in a record, and that was tested

**Believed.** A sweep repairs the documents, and the checks hold the line
afterwards. This project runs eight checks over its prose. One of them reads a
decision record for a version pin and for a figure a measurement can change, so
a reader can take the set as a guard against a record that states something
false.

**True.** No check reads meaning. A sentence that states the state of a register
in its own words is ordinary prose to all eight, and every one of them stays
green when that sentence becomes false.

**Evidence.** After the sweep that repaired the sentence about the missing
measurement, one repaired sentence was put back into a product record in its
stale form. All eight checks ran and all eight passed. The record then went back
to its repaired form. The commit body holds the command and the file.

**Follows.** Three things.

**The defence the earlier finding names has no enforcement.** That finding says
the defence is not a sweep, and that a document must state a blocker by citation
and never in its own words.[^F258A] Nothing fails when a document does the
opposite, so the defence is a habit and not a rule.

**The check that would work is a search, not a parser.** The stale sentence is a
small family of phrases about the state of measurement. A check can forbid that
family everywhere except the registers that own the statement, and this
repository already has the pattern for the sites that may not be repaired: a
baseline file that lists the known failures and may only shrink.[^F258B] A
backlog item holds the design.[^F258C]

**A sweep proved by reading the files is not proved.** The testing rule already
says that the only proof a test reaches a case is to put the defect back and
watch the test stay green.[^84] A sweep over prose earns the same treatment,
and the result here was the one the rule predicts.

### FND-259 — The project orientation has two copies, and the sweep repaired one

**Believed.** The project orientation was repaired when the blocker about the
missing measurement narrowed. The finding that recorded the spread says so, and
it names the orientation as one document.[^F262A]

**True.** The orientation is two tracked files and a symlink. One carries the
repaired sentence and the other still says that no measurement exists on the
target platform, so an agent that reads the second reads something false. The
two files differ in nothing else except the numbering of their footnotes and the
paths inside them.

**Evidence.** A search for the phrase family across the whole tree, after the
sweep, returned the second file. A difference between the two files shows that
every other line agrees. The commit body holds both commands.

**Follows.** Three things.

**This is the redundant declaration site, with both copies in the tree and
nothing that compares them.** The rule already states the defence: when a second
site must exist, add a check that fails when the copies disagree.[^22] A
backlog item holds the check.[^F259CHECK]

**A mirror that differs only by a rewrite is derivable, so the check is cheap.**
The five rule files beside these two differ from their mirrors only in the
directory named inside a footnote. A check can apply that rewrite and compare,
and it needs no judgement to do it.

**A count of repaired sites is a count of the sites the writer knew about.** The
earlier finding named three documents and repaired three. The tree held four,
and nothing in the writing of that finding could have shown the fourth, because
the count came from a reader rather than from a search.

### FND-260 — A search for a stale sentence must normalise the wrap, and the first sweep did not

**Believed.** A whole-tree search finds every site of a sentence. The rule that
governs a sweep says the sweep is done when the search comes back clean, and it
treats the search as the reliable half of the work.[^F218E]

**True.** A line-based search finds the sites where the sentence fits on one
line. Prose in this repository wraps at about 78 columns, so a sentence of five
words or more is usually split, and a pattern that spans the break matches
nothing. The search comes back clean and the sites are still there.

**Evidence.** The sweep for the blocker about the missing measurement ran a
line-based search, reported clean, and left sites in four decision records. A
second search that joined each paragraph into one line before matching found
them, and it found more sites again in the accepted records. Both commands are
in the commit body.

The same second search found the phrase with no blocker number anywhere near it,
in the rule files, in the project orientation and in the header of the script
that took the measurement.

**Follows.** Three things.

**Normalise the text before matching, not the pattern.** Joining a paragraph
into one line and collapsing the runs of space is one line of code, and it makes
an ordinary pattern work. Writing a pattern that tolerates a newline at every
word boundary does not scale past three words.

**Search for the claim, and never only for the number.** A document that names a
register is easy to find and is not the hard case. The hard case is a document
that states the register's content and cites nothing, because the search that
starts from the register's number cannot reach it.

**A sweep is not evidence until it has found something the previous pass
missed.** The first pass here reported clean against its own search. That report
was true about the search and false about the tree, and nothing in the result
distinguished the two.

### FND-281 — The tile-indexed exit direction costs an order of magnitude more than it saves, and the smaller item beats it on the read as well

**Believed.** A unit reads its exit direction through a chain of scattered
lookups, so writing the direction once for each tile removes the chain and the
movement pass gets faster. The item that proposes it prices the memory at about
67 MB against 31 GB the engine does not use, and it names the added pass over
the tile count as a risk to measure.[^F281A]

**True.** The added pass costs between five and twenty-five times what the
movement pass saves, and the proposed read is itself slower than the read it
replaces. The item is refused on the measurement, and it is refused twice over.

**Evidence.** A benchmark builds the tile-indexed array outside the crate, from
the public exit field, and prices both halves in one process on one build.[^F281B]
The world holds 16777216 tiles and one million units. The lookup rows read
every live unit rather than only the units that hold an intent, which is the
most the change can ever reach and is therefore the reading most favourable to
the item. The commit body holds the table and the command.

The two halves do not meet. The cheapest shape of the added write pass costs a
figure in the low hundreds of milliseconds. The difference between the two
lookup rows is a figure in the tens. The frame budget is one hundred
milliseconds, so the added pass alone is larger than the whole budget.[^99]

**The second refusal was not predicted.** The read the item proposes is slower
than the read it replaces, before any write cost is counted. The
cell-indexed array covers 16384 cells and four options, which is 64 kibibytes
and stays in cache for a whole pass. The tile-indexed array covers 16777216
tiles and four options, which is 64 mebibytes and cannot. Removing the
arithmetic in front of the read does not pay for missing cache on every unit.

**Follows.** Three things.

**An indirection into a small table is not a cost, and the word scattered hid
that.** The item described four scattered lookups. In the movement pass the
middle two are arithmetic that touches no memory, and the last is a read of a
table that fits in cache. Only the first reads a scattered address. A
description that counts steps says nothing about what each step costs.

**Price the destination of a move, not only the source.** The item measured
what the chain costs and assumed the replacement was free. Both sides needed a
row, and the replacement lost on its own row.

**A trade that spends memory must name the level of the hierarchy it spends
it at.** The register that opened the memory question states that the engine
uses about three percent of the machine.[^F281D] That is true of capacity and
says nothing about cache. Every candidate trade under that register has to
answer the cache question separately, and this one is the first to be asked it.

### FND-282 — The chain the two items name is mostly one integer division, and the four megabyte item takes the larger share of it

**Believed.** The cost of turning a tile into a cell is the scattered memory it
touches. Two items describe the conversion as scattered reads, and the finding
they both cite reads the conversion as part of a locality problem.[^F281A]
[^F282A] [^F282B]

**True.** The conversion touches no memory at all. It is one remainder and one
quotient by the world width, and the width is a runtime value, so both are a
hardware division.[^F282C] The division is the largest single part of the
chain, and a stored cell index removes it.

**Evidence.** The same benchmark holds the downstream read fixed and changes
only how the cell is reached.[^F281B] Reading a stored cell index costs a small
fraction of deriving it. A row that isolates the address conversion alone
accounts for most of the difference. The machine was loaded while the rows were
taken, so the figures spread; the ordering between the rows did not.

**Follows.** Two things.

**The two items were not two halves of one win, and the priority index said
they were.** The index placed the tile-indexed direction above the stored cell
index, because it removes the whole chain rather than two steps of it. The
measurement reverses that. The two steps the smaller item removes are the
expensive ones. It costs 4 MB rather than 67 MB, and it adds no pass over the
tile count. The index now carries the measured order.[^F282D]

**Counting steps ranked these two items, and it ranked them backwards.** Both
items came from one reading of one pass, and the reading counted lookups. The
item that removes four steps loses. The item that removes two steps wins,
because those two are a division and the other two are a cache hit.
### FND-283 — The orientation claimed Miri checks the unsafe code, and no toolchain the project pinned could run Miri

**Believed.** The project orientation states that keeping the core crate free
of the interpreter binding "allows Miri to check the unsafe code".[^F283A] The
crate split record calls Miri running over the storage code the benefit that no
test gives.[^F283B] The testing guide said there was no Miri job because there
was no unsafe code yet.[^F283C]

**True.** Three things, and each is separately enough.

Miri ships on the nightly channel and on no other. The project pinned a stable
release, so no contributor and no continuous integration run could have run
Miri at any moment since the claim was written. The claim was not merely
unperformed. It was unperformable.

Nothing in the tree ran it. There was no recipe, no workflow step and no
script. A search for the name found six documents that discuss Miri and no
invocation of it.

The core crate does hold unsafe code. Two items assert that the settlement
store is plain data.[^F283D] So the testing guide's stated reason for having no
Miri job was false as well, and it was false for a different reason than the
channel.

**Evidence.** A whole-tree search for the name, excluding the build directory,
returned prose only. The toolchain file named a stable release. A search for
the `unsafe` keyword across the crates returned two items and one doc comment.
The commit body holds both commands.

**Follows.** Three things.

**This is the inert capability, and the rule already names the shape.** A
project declares a capability, documents it, and nothing calls it.[^F283E] The
defence the rule gives is to ask who is obligated to invoke the thing. Here the
answer was nobody, and the document that claimed the benefit was the same
document that would have said so.

**A benefit stated in a record must name what invokes it.** The crate split
record gives up something real for this benefit, and the reader of that trade
had no way to learn that the benefit was never collected.

**The benefit was worth collecting, which is why this cost something.** The
state hash reads whole structures and whole columns as raw bytes, and an
undeclared padding byte would put an uninitialised byte into that read. Such a
hash differs between two runs of one binary. No other gate in this project sees
it, because a padding byte on a development machine is reliably zero. A gate
now exists.[^F283F]

### FND-284 — The reassociating float methods were blocked by the compiler, not by the lint, and clippy says nothing about a lint entry that reaches nothing

**Believed.** The project orientation says the script catches "the
reassociating methods, which do not resolve on the pinned toolchain and so
cannot be named in a lint".[^F262A] The float ban script says the same in its
own header.[^F284A]

**True.** The obstacle on the pinned stable release was the compiler, not name
resolution. A call to the reassociating add on that release is a hard error,
because the library feature is gated and the gate cannot be opened on a stable
channel. So on that channel the method could not be written at all, and the
script's coverage of it guarded a door that the compiler had already locked.

On the dated nightly the project now pins, the same call compiles with no
feature attribute. The method is writable for the first time. In the same step,
clippy resolves it and rejects it through its disallowed-method list.

A third thing came out of the same probe, and it is the one that matters most.
**Clippy silently ignores a disallowed-method path it cannot resolve.** It
emits no warning and no note on either channel. A lint entry that names nothing
is inert, it reads as a live rule, and nothing announces the difference.

**Evidence.** A scratch crate outside the tree, built on both toolchains. On
the stable release the call failed with the unstable-feature error and named
the tracking issue. On the nightly the same file compiled with no attribute,
and a feature gate was proved still to be enforced there by a second probe that
the compiler rejected. With the method named in the disallowed list, clippy on
the nightly rejected the call and quoted the reason. With a path that names no
method at all, clippy on both toolchains finished clean. The commit body holds
the probe.

**Follows.** Three things, and none of them is that a mechanism should be
removed.

**The lint gains real work, and it did not have any before.** The entries
should be added, because the method is now writable. Until they are, the
script's name check is the only thing standing where the compiler used to
stand. A backlog item holds the addition.[^F284B]

**The script's second job became load-bearing at the moment it stopped being
redundant.** Before the move it duplicated a compiler error. After the move it
is the sole guard until the lint entries land. Reading the move as a reason to
retire it would have been exactly backwards.

**A lint entry can never be assumed live, and that is now measured rather than
argued.** The rule that two mechanisms exist because one is not enough rested
on judgement. It now rests on an observation: the first mechanism can be
switched off by a typing mistake in a configuration file, and the tool that
reads that file will not say so.

### FND-288 — Miri cannot drive the engine at the fixture sizes the suite uses, because every world reserves the unit columns at the target population

**Believed.** That a Miri gate would run over the tests the project already
has, or over some subset chosen for relevance.

**True.** Almost every engine test sets the unit capacity of its world to the
target unit population, even where the world is sixteen tiles across. The world
reserves those columns when it is built.[^139] Miri interprets each of those
writes rather than executing it, so such a test does not finish.

The relevance of a test is therefore not what decides whether Miri can run it.
The reservation is.

**Evidence.** Two runs on the development machine, on the dated nightly. The
value-type test, which builds no world, finished under Miri in under two
seconds of interpretation. A rate test, whose world is sixteen tiles by sixteen
and which reserves the target population, had produced no result after nine
minutes and was stopped. The two differ in the reservation and in little else.

**Follows.** Two things.

**A Miri gate needs a fixture of its own, and the fixture is the whole design
of the gate.** The gate that now exists builds a world that reserves a few
thousand slots, spawns soldiers into it, steps it, and hashes it. It reaches
the byte-level read through the engine rather than around it.[^F283F]

**This is the fixture rule again, from the other direction.** The rule says
that a fixture modelled on the demonstration world supplies what looks right
rather than what the test needs.[^23] Here the copied value made the test
impossible rather than merely weak, which is the friendlier of the two
failures, because it announced itself.

### FND-289 — The overflow record's reason for preferring a test to a lint rests on a channel property, and the pin changed it

**Believed.** The overflow record says that the switch naming the gate build
"is not stable on the pinned toolchain, so a lint cannot see this and a test
must".[^100]

**True.** The switch is still unstable on the dated nightly. It is no longer
unreachable: the compiler now names the feature attribute that would enable it,
and a crate that declares that attribute can read the switch. So the sentence
has gone from stating an impossibility to stating a cost.

**Evidence.** The same scratch crate. On both toolchains the switch is
rejected as an unstable feature and both errors name the tracking issue. Only
the nightly error names the attribute that enables it, which is the compiler
saying that the gate exists and can be opened there.

**Follows.** Two things.

**The test the record requires stays, and this finding does not amend the
record.** The record gives a second and better reason for the test: it reads
the outcome by catching a real panic rather than by reading a compiler switch,
and a switch that is set says nothing about whether the panic happens. That
reason is untouched.

**A record should not rest a decision on a property of the channel, because a
pin can change the channel.** The reasoning that survives is the reasoning
about what the test proves. The reasoning about what the toolchain permits is
the part that decayed, and it decayed within one commit of the pin moving.

**Neither item was built.** This finding recommended the smaller one, and a
later measurement removed the division outright, so the smaller item was
refused as well.[^F282A] A division is not a thing that needs a stored copy to
avoid. The grid now holds a reciprocal of its width, one value derived from one
value at one site, and every caller of the conversion is cheaper rather than
only the unit passes.

### FND-290 — Three items competed to optimise under one percent of a frame, and none of them had a denominator

**Believed.** The cost of turning a tile into a cell is worth an item. Three
shapes were proposed for it: a tile-indexed exit direction, a stored cell index
on the unit, and an arithmetic replacement for the division.[^F281A] [^F282A]
Two findings priced the shapes against one another, and both compared a loop to
a loop.[^F287A] [^F287B]

**True.** None of the three was ever measured against a frame. A frame at the
target extent costs seconds on the machine that took these figures, and the
whole conversion is a fraction of one percent of it. **The three shapes were
ranked correctly against each other and none of them matters.**

**Evidence.** A row that times a whole step was added to the same
benchmark.[^F287C] At the target extent, with one million units at twelve
threads, a frame costs a figure in the seconds. The conversion loop that all
three items address costs a figure in the tens of milliseconds under the same
load. The share is below one percent whichever end of either spread is taken.

Two further rows say what the frame is made of. **Dropping the population by a
factor of ten barely moves the frame cost**, so the frame follows the tile count
and not the population, and every one of these three items is a unit-side
change. **A frame at one thread costs about twice a frame at twelve**, on a
machine with sixteen cores, which is the speedup a half-serial pass allows and
matches the serial stage the stage split names.

The machine carried a load average above thirty on sixteen cores throughout, so
every absolute figure here is inflated. The share is a ratio of two figures taken
under that same load, and the ratio is what the finding rests on.

**Follows.** Three things.

**A saving needs a denominator, and a loop is not one.** Both earlier findings
compared one loop against another loop. That is the right way to rank two
shapes and it says nothing about whether either is worth building. The ranking
was sound and the question was not asked.

**Measure the frame before ranking the parts of it.** The benchmark had rows for
a derivation, a lookup and a write before it had a row for a step. The row that
would have stopped all three items was the cheapest one to write.

**An item that names a cost should name its share.** All three items opened by
naming a mechanism and pricing it in bytes or in steps. None of them stated what
fraction of a frame it stood to win, and none of the reviews of them asked.

### FND-291 — A property test over the whole legal range passed against a broken reciprocal, and an exhaustive test over a narrow one caught it

**Believed.** A property test that samples the whole legal range of a parameter
covers that range better than a test that walks a small part of it
exhaustively. The wider strategy was written first for that reason.

**True.** The wide property passed against a defect that the narrow exhaustive
test failed on immediately. **A uniform sample over a large range is a weak
fixture when the defect lives at a sparse set of inputs.**

**Evidence.** The grid now converts a tile index to an address by multiplying by
a reciprocal of the width, rather than dividing.[^F282C] The classic error in
that construction is to omit the increment, which makes the reciprocal one too
small. The defect was put back, and five tests were watched.

The wrong reciprocal gives a wrong quotient at an exact multiple of the width,
and at no other index. A uniform index below the tile count is a multiple of the
width with probability one over the width. The strategy drew widths across the
whole legal range, so a typical width was near two thousand million, and the
property never drew a failing index in any run.

The exhaustive test over widths one to sixty-four failed at once, because at
those widths almost every draw is near a multiple.

**The fix was to the fixture, not to the assertion.** The strategy now draws a
row and an offset of minus one, zero or one, so every case sits on or beside a
row boundary. The defect was put back a second time and the property then
failed. Both the assertion and the range were correct throughout; only the
distribution was wrong.

**Follows.** Three things.

**A wide range and a good fixture are different things, and the first looks like
the second.** The strategy that sampled every legal width read as the more
thorough test. It was the weaker one, and nothing in reading it would have said
so.

**Ask where the defect lives before choosing the distribution.** The failing set
here is describable in one sentence, and the sentence names the distribution the
test needed. That question is cheaper than the test.

**The rule that caught this is the rule that says to put the defect back.** The
project already holds two instances of a fixture that hid a defect.[^F262D] This
is the third, and it is the first where the fixture covered a wider range than
the test that worked.

### FND-292 — The sparse tile change list reaches every tile in ten frames, so the sparse form costs four times what a dense one would

**Believed.** The tile value field is a generated base and a stored list of
changes, and the list holds what the frames have changed rather than the size
of the world. The reader that returns the count says so in its own words, and a
product record rests on it.[^F292A]

**True.** The list reaches 99.8 percent of the tiles within fourteen frames at
the target extent, and it saturates in about ten. **The sparse form is sparse
for one second of simulated time.** After that it is a dense array with an
index column attached, and it costs four times what a dense array would.

**Evidence.** A row counts the entries after each frame at 16777216 tiles and
one million units.[^F292B] The count is entries and not nanoseconds, so a
loaded machine does not disturb it. Frame 0 stores 6291370 entries. Frame 5
stores 15777624. Frame 13 stores 16753789, which is 998 parts in a thousand of
the tiles, and it added 14016 that frame.

**The merge rewrites the whole list on every frame.** It walks the stored list
and the new run together and writes both into a second buffer, so its cost
follows the length of the list and not the length of the run. The stage table
measured that pass at 120,529,050 nanoseconds, which is 14.4 percent of a
frame, and it takes no thread count.[^F292C]

**The run itself is not small, and an earlier draft of this finding said it
was.** The added column above counts the tiles that became changed for the
first time, and that column falls to a few thousand. The run does not fall with
it. The tile scan draws for every tile and keeps three cases in eight, so it
offers about three eighths of the world on every frame, which is 6291456 tiles
at this extent and matches the first frame of the table exactly. **Almost every
tile in the run is a tile the list already holds.**

So the merge walks about 23 million entries to apply about 6.3 million, and the
dense form applies the same 6.3 million and walks nothing else. The saving is
the walk and the second buffer, and it is a factor of about four in the work
rather than the factor of a thousand that reading the added column alone would
suggest.

**The memory runs the same way.** An entry holds a tile index and a value, so
it is eight bytes, and the second buffer holds as many. At saturation the two
together are about 268 megabytes. A dense value for every tile is four bytes
and needs no second buffer, so it is about 67 megabytes.

**Follows.** Three things.

**This is not a trade.** A dense field is smaller and faster at once. The usual
question of what to spend memory on does not arise, because the sparse form is
the expensive one on both axes as soon as it saturates. A decision that
compares them must be read against the saturated state and not the empty one.

**A structure named for what it stores was measured for what it stores, and
never for how long that stayed true.** The count is honest on frame one and
false by frame ten. Nothing in the reader or in the record that governs it
states the horizon over which the claim holds.

**A claim about growth needs the frame at which it stops being true.** The
useful figure is not that the list grows with what changed. It is that what
changed reaches everything in ten frames.

**A structure is not priced by the pass that carries its name.** This work
started from a stage called the change merge, so it measured merging. The
sparse form cost more in the passes that read the field than in the one that
wrote it, and no stage is named after reading a tile value. The stage table is
a map of where time goes and not a map of what causes it.

### FND-298 — A benchmark that measures its own process reports the history of the run, and this project has read it twice

**Believed.** A process that has built a world and stepped it can report what
that world costs, by reading its own resident size.

**True.** It reports the high mark of everything the process has done. An
allocator does not return memory to the kernel when a vector is dropped, so a
transient buffer that existed for one frame is still in the figure. **The
number is the history of the run and not the cost of the world.**

**Evidence, and this is the second instance.** A recent change to the tile
value field was reported as saving 225,906,688 bytes, from the `resident_bytes`
line of a stage-cost run. The dedicated instrument, which starts one process
for each point, gave a different answer for the same pair of trees: the
resident size fell by 34,074,624 bytes and the peak by 196,821,088. The
direction was right and the size was not. The benchmark already carried the
right instrument, and its own comment says why it exists.[^F298A]

The first instance is the resident memory entry, where a figure taken at one
thread was quoted for a machine that needs a larger one.[^F298B]

**Follows.** Three things.

**Prefer the instrument that was built for the question.** The stage-cost run
reports a resident size because it is cheap to print, not because it answers a
memory question. A line that a run happens to emit is not a measurement of
anything in particular.

**A memory figure needs one process for each point.** That is the rule the
benchmark already follows where it means to measure memory, and the rule was
available to be read.

**This is now a shape rather than an incident.** Two figures in this project
have been wrong because a process measured its own past. Treat any resident
size taken after other work in the same process as an upper bound and nothing
more.
### FND-299 — The spread read the ground of every candidate tile, and the ground can only refuse

**Believed.** The ground governs a claim, so the rule that decides a tile reads
the ground first. The ground says how much support the tile asks for, and open
water asks for more than any claim can raise, so a pass that reads it first
refuses the impossible cases before it does any work.

**True.** The ground can only refuse. It never turns a losing challenger into a
winning one, so a tile whose best challenger does not beat the holder keeps its
holder whatever the ground says. Reading it first therefore paid for the answer
on every candidate and used it on few. **The ground of a tile is generated from
the seed rather than stored, so the read is a draw and not a load.**[^F299A]

**The measurement.** An ablation that replaced the ground read with a constant.
Development machine, 4,194,304 tiles, 250,000 units scattered, 12 threads, nine
frames, the minimum of three runs because the machine was loaded.

| The pass reads | ns for each frame |
|---|---|
| Everything | 311,285,062 |
| No ground | 87,708,130 |
| No units | 54,872,312 |
| No supporter sort | 294,343,620 |

**The ground read was 72 percent of the pass and the unit read was 82 percent,
and the two sum to more than the whole.** They overlap: removing one lets the
other's memory latency overlap with the arithmetic. Read each figure as an
upper bound on what removing that one thing buys, not as a share of a partition.

**The supporter sort is 5 percent**, and a plan that started from the source
would have gone there first, because a sort inside a per-tile loop looks worse
than a field read.

**Follows.** Three things.

**Order the tests in a decision by what they can do, not by what they are
about.** A test that can only refuse belongs after the tests that can decide.
The ground is the subject of the rule, and that is why it was first.

**A generated field is a computation wearing the clothes of a read.** The
project chose to generate the ground rather than store it, and the call site
looks like an array index.[^F299A] Nothing at the call site says that it costs a
draw, and the pass that read it 4.8 million times each frame did not know.

**Ablate before you plan.** The pass had four parts and the plan named the wrong
one. Removing one part and measuring took three builds and found a 72 percent
term that no reading of the source had suggested.

### FND-294 — Both repairs after a holding change cost the holding, not the change

**Believed.** The pass that writes a decided change is a scattered write for
each change, so its cost follows the number of tiles that changed hands.

**True.** The write is 7 percent of it. The two repairs after the write are 92
percent, and neither follows the change count.

**The held list is rebuilt by a merge that reads all of it.** The list holds
every tile somebody holds, in ascending tile order, and the merge walks the old
list and the changed tiles together into a new buffer. At the target scale it
rewrites 6.9 million entries to apply 359 thousand.

**A block mask is rebuilt by reading the whole block.** A block loses a bit only
when the last tile of a faction leaves it, and no running count can see that
without reading the block. At 13 thousand dirty blocks of 1,024 tiles each, that
is 13.3 million holder reads for 359 thousand changes.

**The measurement.** A probe inside the pass. Development machine, 16,777,216
tiles, 1,000,000 units scattered, 12 threads. The last frame of ten:

| Part | ns | Share |
|---|---|---|
| Join the buffers of the deciding threads | 4,045,295 | 0.5 percent |
| Write the holders and the census | 50,583,368 | 6.6 percent |
| Rebuild the held list | 326,193,156 | 42.7 percent |
| Rebuild the mask of every dirty block | 383,562,012 | 50.2 percent |

**Follows.** Two things.

**A stage named for a write can be dominated by its repairs.** The name said
what the pass was for. The cost was in keeping three derived declarations of one
fact in agreement, which is the shape this project records most often.[^22]

**Both repairs take a thread count now, and neither has stopped following the
holding.** Threading them divided the cost. It did not change what the cost
follows, and a holding that reaches the whole world will bring it back.[^F277B]

### FND-295 — The derived unit structure was searched once for each tile, and the tile order makes it a walk

**Believed.** Asking which units stand on a tile is a bounded search: the
structure holds one contiguous run for each block, and the search is bounded by
the block size rather than by the unit count.[^F295A] A pass that asks for many
tiles therefore pays a small bounded cost for each.

**True.** The bound is small and the cost is not, because the search is two
binary searches into an eight megabyte key array and every probe is a cache
miss. The spread asked 4.8 million times in each frame. The reading that made it
look cheap counted probes, and a probe is not the unit of cost.

**The order the caller already had makes the search unnecessary.** The low part
of a bridge key is the row-major offset of a tile inside its block, so a caller
that visits tiles in ascending tile order asks for ascending keys inside every
block. One cursor for each block therefore only ever moves forward, and the
whole walk costs the candidate list plus the units rather than a search for each
candidate.

**This is not a property of the structure alone.** It holds only while the
caller visits in ascending tile order, and nothing in the type can enforce that.
A test drives both the walk and the search over every tile of a small world and
compares them, and the walk was made to fail once on purpose to prove the test
reaches it.[^84]

**Follows.** Two things.

**A bounded cost is not a small cost.** The bound here is the block size and it
is correct. What the bound omits is that each of its steps is a miss, and that
the caller runs it millions of times.

**When a search is bounded, ask what order the caller already has.** The caller
had ascending tile order, which the candidate pass produces by construction, and
that was enough to remove the search entirely.

### FND-296 — A stage was 16.9 percent of a frame and was 2.2 percent four hours later, and the plan that named it was written in between

**Believed.** The level 1 rebuild is 16.9 percent of a frame and is the third
largest stage. That figure came from a measurement on the target platform, and a
plan was written from it.[^F286A]

**True.** It is 6,600,185 nanoseconds of a 300,016,572 nanosecond frame, which
is 2.20 percent. Nothing changed in the rebuild. Another change made the frame
smaller and made the rebuild itself cheaper, and the figure the plan rested on
expired before the plan was read.

**The measurement.** Machine C, `c7g.4xlarge`, Graviton3, 16,777,216 tiles,
1,000,000 units scattered, 12 threads, nine frames, `stage-cost` feature.[^F296B]

**Follows.** Two things.

**Take the denominator again when you start, not when the plan was written.**
The project already holds a finding about three items that competed over a
region under one percent of a frame.[^F296C] This is the same failure at a
different point in the pipeline: the figure was right when it was measured and
wrong when it was used, and nothing between the two said so.

**A share moves when anything in the frame moves.** A stage that costs the same
number of nanoseconds can double its share while getting no worse. Quote the
nanoseconds beside the share, and name the run both came from.

### FND-297 — A local benchmark reported a two-hour-old build, and nothing said so

**Believed.** `cargo build --release --bench target_cost --features stage-cost`
rebuilds the benchmark binary, so a run of that binary measures the tree.

**True.** It stopped doing so. The command reported success, compiled the
library, and left the executable it had built two hours earlier untouched. Three
successive measurements of three different trees returned the same figures, and
the only reason it was caught is that a probe printed nothing when it should
have printed a line for every frame.

**The build that works is `cargo bench -p cachette-core --bench target_cost
--features stage-cost --no-run`, and it puts the executable somewhere else.**
The stale binary sat under `target/release/deps/` and the fresh one under
`target/release/build/`, so a path that had worked all evening kept resolving to
the old file.

**The measurements on the target platform were never affected.** The harness
copies the tracked files to a clean instance and builds there, so it cannot
carry a stale artefact.[^F297A]

**Follows.** Two things.

**A measurement apparatus needs a liveness signal that fails loudly.** A stale
binary and a change that did nothing look identical, and the second is the
answer a tired reader wants. The probe that caught this was there for another
reason.

**Check the time on the artefact you are about to measure.** One `ls` would have
caught it two hours earlier.

### FND-304 — Admission searched two count tables once for each segment, and it visits them in the order they are stored

**Believed.** The passes that grant an intent read two small tables of counts,
one for arrivals and one for departures. A lookup in a small table is cheap, so
reading them once for each target tile costs little.

**True.** The tables are not small. At the target scale they reach one entry for
almost every target tile, and a lookup is a binary search over them. The grant
passes were 60 percent of the whole admission stage, and they run on the calling
thread.

**The order the pass already has removes the search.** The segments are in
ascending tile order because the admission sort put them there, both tables are
in ascending tile order because the merge that fills them requires it, and
neither table changes while the pass walks the segments. One forward reader for
each table therefore replaces every search, and the walk costs the segments plus
the entries.

**The measurement.** A probe inside the stage. Development machine, 16,777,216
tiles, 1,000,000 units scattered, 12 threads. The last frame of ten, with
314,158 intents and 291,047 segments:

| Part | ns | Share | Runs on |
|---|---|---|---|
| Sort the intents by target tile | 208,785,397 | 20.1 percent | one thread |
| Follow the permutation | 13,198,816 | 1.3 percent | one thread |
| Build the segment table | 59,892,786 | 5.8 percent | one thread |
| Read the capacity and the occupancy | 132,558,620 | 12.8 percent | every thread |
| Grant the intents | 623,729,193 | 60.1 percent | one thread |

**Eighty-seven percent of the stage runs on one thread**, and the stage declares
that it takes a thread count. The declaration is not wrong, because the one
parallel part is real, but the column cannot say that seven eighths of a stage
ignores the number.

**Follows.** Two things.

**A table that grows with the tiles is not a small table, whatever it is named
for.** These hold arrivals and departures, which sound like a handful of events,
and they reach almost every target tile at the target density.

**This is the third place tonight where a search was removed by the order the
caller already had.** The other two are the spread deciding a tile and the same
stage reading how many units stand on a target.[^F304A] The shape is worth
looking for: a bounded search whose caller walks in the order the searched
structure is stored in.

### FND-301 — The bridge rebuild is 16.4 percent of a frame, and the record that gives it one thread says it has not earned one

**Believed.** The bridge rebuild does not earn a thread. The record says so in
those words, and it argues that splitting a radix histogram across threads is
more machinery than the whole rebuild costs.[^F301A]

**True as a design argument and false as a cost claim.** The rebuild is
30,754,053 nanoseconds of a 187,862,216 nanosecond frame, which is 16.37
percent. It is the third largest stage in the engine. The record was written
before any measurement existed on the target platform, and the phrase "does not
earn one" is a claim about cost.

**Nothing here changes the record.** The reasoning it gives about the histogram,
the fixed combine order and the placement offsets is unaffected by how large the
stage is, and every one of those is a place where a result could take its order
from a thread. What the measurement changes is the premise, not the argument.

**Where the time goes.** A probe inside the rebuild. Development machine,
16,777,216 tiles, 1,000,000 units, 12 threads, the last frame of ten:

| Part | ns | Share |
|---|---|---|
| Walk the arena and build the keys | 109,373,561 | 16.7 percent |
| Order the keys | 412,957,538 | 63.0 percent |
| Follow the permutation | 117,637,598 | 17.9 percent |
| Rebuild the block ranges | 16,026,693 | 2.4 percent |

**The stage declared that it takes a thread count, and the record says it
accepts none.** Three stages wrap this one call and all three declared `true`.
Measured at one thread and at twelve on the development machine, at 4,194,304
tiles and 250,000 units, the stage costs 43,040,085 and 57,165,452 nanoseconds:
it does not improve, and it may get worse. The declarations are now `false`.

**Follows.** Two things.

**A record that argues from cost needs its cost re-read when a measurement
arrives.** This one is still right, and nobody could have known that without
taking the figure.

**A declaration in the source is a claim, and this project already said so.**
The cost register states that the thread-count column is a declaration rather
than a measurement, and that the table is where the two can be compared. Three
rows disagreed with an accepted record for as long as the table has existed, and
the comparison the register invites is what found them.

### FND-302 — Every ordering pass sorts the whole key set a second time, to check that no identifier repeats

**Believed.** The key vector sort costs its radix passes. The guard that refuses
a repeated identifier is a check, and a check is cheap beside the work it
guards.

**True.** The guard allocates one 64-bit value for each key and comparison-sorts
them, before the radix runs. It is a second full sort of the same set, and both
passes that this project measures tonight pay it: the admission sort over about
314 thousand intents and the bridge rebuild over one million units, on every
frame.

**The measurement is not clean and it is reported that way.** The development
machine was loaded, and the figures for one configuration varied by a factor of
five between runs. What is not in doubt is the shape: the guard is
`sort_unstable` over a vector it allocates, and the radix that follows is three
counting passes. Take the shares below as an order of magnitude and not as a
figure.

| Set | Guard, ns | Radix, ns |
|---|---|---|
| 1,000,000 keys | 11,781,361 to 60,213,363 | 131,517,658 to 337,357,895 |
| about 300,000 keys | 86,882,731 to 111,072,468 | 71,317,708 to 71,643,957 |

**The guard protects determinism, and that is why it is not simply removed.**
The identifier is what makes the order total. Two keys that share both the
ordering field and the identifier tie, and a tie is a place where the result can
depend on something the caller did not state.

**The guard is stronger than the property it protects.** It refuses two keys
that share an identifier even when their ordering fields differ, and such a pair
ties nothing. The narrower property, that no two keys share the pair, is
adjacent in the sorted result and therefore free to check after the radix rather
than before it.

**Nothing is changed here.** Narrowing the contract of a shared sort is a
determinism decision, and the scope rule says a determinism decision always gets
a record even when it looks obvious. The decisions register holds the
options.[^F302A]

### FND-303 — The twelve millisecond residual was not the allocation, and it has gone

**Believed.** The unmeasured part of a frame grew from 22,202 nanoseconds to
about 12 milliseconds when the candidate pass began allocating a bit plane on
every frame, and it stayed there when twelve allocations became one. The
register concluded that the residual follows the allocation.[^F277C]

**True.** The residual is 310,942 nanoseconds, which is 0.17 percent of a
187,862,216 nanosecond frame. **The candidate pass still allocates a bit plane
on every frame, and nothing in it changed.** What changed is that the tile value
field became a dense delta and the change merge stage went away.

**So the earlier conclusion was wrong.** The residual did not follow the
allocation. It followed something in the pass that has since been removed, and
the coincidence in timing was strong enough to survive one attempt to refute it.

**Follows.** Two things.

**One refutation is not enough when the alternative was never tested.** The
earlier row ruled out the number of mappings and then concluded that the
allocation was the cause, without ever testing a build that did not allocate.
The remaining hypothesis inherited the confidence of the one that was tested.

**A residual is a difference of two large numbers, and it moves when either
does.** Twelve milliseconds against a 463 millisecond frame and 0.31
milliseconds against a 188 millisecond frame are not the same measurement twice.
### FND-307 — The holding apply spent more on rereading blocks than on the list everyone was looking at

**Believed.** The holding apply is expensive because it rebuilds the held tile
list on every frame, in the same shape as the change merge that this project
removed the same night. The stage table names the whole apply and nothing
divides it.

**True.** The apply has three parts, and the list rebuild is the smaller of the
two large ones. **The larger is the repair of the block masks, which reread
every tile of every block a change touched.**

**Evidence.** A counting switch on the apply, at 16,777,216 tiles with one
million units. At frame 21 the apply moved 327,757 tiles, dirtied 11,362 blocks
of 16,384, rebuilt 8,565,250 held entries, and read 11,634,688 holders to
repair the masks. **About 20 million memory operations to apply 328 thousand
changes**, and the list is under half of it.

The repair is now gone rather than smaller. The holding keeps a count of the
tiles each faction holds in each block, so a mask gains a bit when a count
leaves zero and loses one when a count reaches zero. A moved tile touches two
counters. No block is read again.

**The figure this finding first carried was taken against a baseline that no
longer existed.** It said the apply was 2.18 times cheaper and the frame 17.7
milliseconds cheaper, and both were measured against the serial apply. Another
branch had already made both repairs after a write take a thread count, so the
reread was being divided among twelve threads rather than done once. A pair
taken after the two branches merged, on one instance type and one extent, gives
the apply 1.99 times cheaper and the frame 9.6 milliseconds cheaper.[^F307A]

**The work removed did not change and the value of removing it halved.** The
count says 11,634,688 holder reads either way. What changed is what those reads
were costing when they were deleted.

**The test that should have covered this could not.** The existing test
compares every mask against a full pass after a run, so it proves the masks
agree at one moment. A bit that is set and never cleared agrees at every moment
in which nothing was vacated. The defect was put in — never clear the bit — and
the whole suite was run. Sixteen tests in the holding file passed, and one test
in the consumption file failed by accident.

**Follows.** Three things.

**The stage table names a pass, and a pass is not a cause.** This is the second
time in one night that the cost of a named stage sat somewhere the name did not
point.[^F307B] Divide a stage before optimising it, and divide it by counting
rather than by reading.

**A test that compares two states cannot see a transition.** The mask agreed
with the tiles at every moment the fixture reached, and the defect lives in the
moment the fixture never reached. The replacement asserts that at least one
block lost a faction during the run, so it cannot pass by never reaching the
case it exists for.[^F307C]

**Ask what a running count cannot see, then check whether it still cannot.**
The comment beside the reread said that no running count can see a block lose
its last tile of a faction. That is true of a count of tiles in a block, and
false of a count of tiles for each faction in a block. The sentence was correct
about the count it described and it stopped anyone reaching for a different one.

### FND-308 — The held tile list does not saturate, so the move that fixed the tile value field does not transfer to it

**Believed.** The held tile list is rebuilt whole on every frame, which is the
shape that made the tile value field expensive, so the same repair applies. The
tile value field saturated and its sparse form became a dense array with an
index column attached.[^F307B]

**True.** The held list converges to about 10.0 million entries, which is 59.7
percent of the tiles, and it is flat by frame 60. It has a ceiling below the
world and the tile value list did not.

**Evidence.** A counting run at 16,777,216 tiles with one million units. The
list holds 998,551 entries after the first frame, 8,346,340 at frame 19,
9,928,297 at frame 60, and 10,019,403 at frame 89, where it is still adding
about two thousand a frame and decaying.

**The cause of the difference is in the rule and not in the numbers.** An entry
left the tile value list only if it was never removed, and it never was: a tile
that changed once held a stored change for ever. A tile leaves the held list
when its last holder goes. **One list could only grow and the other can shrink**,
so the first was certain to saturate and the second was not.

**Follows.** Two things.

**A repair is not a pattern until its premise is checked.** The two lists have
the same shape, the same rebuild and the same cost curve at first sight. They
differ in whether an entry can leave, and that decides whether the repair
applies. The record for the dense field says in its own text that it governs one
field and is not a licence to convert others, and this is the first case that
tested the refusal.[^F313B]

**Sixty percent is not sparse, and it is not saturated either.** The list is
still 40 megabytes and it still drives the largest stage in the engine. The
answer here is neither the dense array nor leaving it alone, and naming the
ceiling is what makes that visible.
### FND-310 — The sort's guard was tested only on pairs that tie nothing, so the property it protects had no test

**Believed.** The key vector sort refuses a repeated identifier, and two tests
prove it. The guard therefore protects the order from a tie, and the tests hold
it to that.

**True.** Both tests used keys that share an identifier and differ in an
ordering field. Such a pair is separated by the field it differs in, so it ties
nothing and the order is total without the identifier deciding anything. **A
repeated key, meaning two keys that agree in every field including the
identifier, appeared in no test in the repository.**

The property that determinism actually needs was therefore untested, and the
tests that looked like they tested it exercised only the case that does not
matter.

**The evidence is an experiment, not a reading.** The guard was narrowed to
refuse a repeated key and nothing else, and the whole suite was run. Exactly
two tests failed, and they were the two that assert the refusal. Every other
test in the crate passed, so nothing else in the project depended on the wider
refusal.

Then the narrow guard was removed entirely and the suite was run again. Three
tests failed, and all three were written in the same change. **Before that
change, removing the guard altogether would have broken nothing.**

**Follows.** Three things.

**A test that asserts an error can still test the wrong case.** Both tests named
the right thing, cited the right record, and constructed an input that the
property does not care about. Reading them would not have found it. Running the
suite against a narrowed guard did, in one command.

**Narrow a check before you trust it.** The wide guard passing tells you nothing
about the narrow property inside it. The way to find out which part is load
bearing is to remove the rest and see what fails.

**Put the defect back, and put it back in the new place too.** The check moved
once more after this, into the pass that orders ties, and it was broken again
there to prove that the three tests still reach it.

### FND-311 — A sequential sort was replaced by a walk over a permutation, and the frame got nine percent worse

**Believed.** Replacing a comparison sort of a whole key set with one pass over
the finished order, and no allocation, is cheaper. The pass is linear and the
sort is not.

**True.** It was measurably worse. The first form of the narrowed guard walked
the sorted order and read two keys through it for each neighbouring pair, which
is two random gathers into a sixteen megabyte array, one million times. The
comparison sort it replaced read one million identifiers sequentially into a
fresh vector and sorted that vector, which is contiguous and vectorises.

**The measurement.** Machine C, `c7g.4xlarge`, Graviton3, 16,777,216 tiles,
1,000,000 units scattered, 12 threads, nine frames, `stage-cost` feature.

| Stage | Before, ns | After the first form, ns |
|---|---|---|
| `bridge_refresh_barrier` | 31,394,191 | 42,867,673 |
| `admit` | 21,274,145 | 19,895,342 |
| **The frame, timed from outside** | **177,862,658** | **193,848,102** |

The bridge rebuild rose 36.5 percent and the frame rose 9.0 percent. The stage
that sorts the smaller set fell, which is what the change was supposed to do
everywhere.

**The repair puts the check where the ties already are.** The pass that orders
tied entries already visits every run of one ordering field, and only a run
holding more than one entry can hold a repeated key. Nearly every run holds one.
The check therefore reads entries the tie sort has just touched, and a run of one
entry costs nothing at all.

**Follows.** Two things.

**A pass over a permutation is not a linear pass.** The comment written for the
first form said it cost one pass and no allocation. Both halves were true and
the conclusion was wrong, because the unit of cost is the cache miss and not the
element.

**This project already held the same lesson in the other direction.** A bounded
search was found not to be a small search, because each of its steps was a
miss.[^F311A] The same arithmetic run backwards says a sequential sort can beat
a random-access scan, and it was not applied.

**This row rests on a single pair of runs, and a later finding puts a floor
under what such a pair can claim.**[^F306B] The figure survives that floor
rather than comfortably: 42,867,673 nanoseconds is above every later reading of
the same stage, and the base it is compared against reproduces to 0.7 percent.
But the repaired form of the same change measured 36,459,500 and 29,068,978 on
two runs of one binary, so **the 36.5 percent in this row should be read as a
regression that happened, not as its size.** Nobody re-ran the first form,
because it was replaced rather than kept.

### FND-312 — Two checks were blind in two different ways, and a pipeline hid the second

**Believed, first.** The record check reports which records no source file cites,
and a worker running it from a worktree gets that report about their own tree.

**True.** It reported every record as cited by nothing. The check skipped any
path whose parts include `worktrees`, so that one run does not read files another
run owns. A worktree of this project lives under `.claude/worktrees`, so when the
check ran from inside one, every file it would scan had `worktrees` among its
parts and the skip removed all of them.

**The measurement.** One commit, checked twice. From the repository root: two
notes. From inside a worktree: fifteen. Thirteen records were reported as cited
by no source file while their citations sat in files the run never opened. Three
sibling checks name paths rather than components and none has the defect. The fix
makes this one name paths too.

**Believed, second.** The conflict marker check has the same defect, so running
it from inside a worktree reports a clean sweep of files it never looked at.

**False, and it was disproved by a probe rather than by reading.** That check
skips a path built from the script's own location, so inside a worktree the skip
names that worktree's own empty nested directory and removes nothing. A real
marker planted in a worktree and the check run from that worktree, unpiped:
three named failures and exit 1.

**The blindness is real but it points the other way.** The same probe, with the
root copy of the script run against the same planted marker: 864 files, no
failure, exit 0. **A worker whose changes are in a worktree must run the check
from the worktree.** Running it from the repository root is the blind
configuration for exactly the files that changed.

**Neither of those is how markers reached the trunk.** A pipeline exits with the
status of its last command, and the failures go to standard error:

```text
python3 scripts/check_conflict_markers.py | tail -2 && git add -A
```

The `git add` runs whatever the check found, because the status belongs to
`tail`. Verified in this worktree on a tree holding three markers: unpiped the
script exits 1, and through `| tail -1` the shell reports 0.

**Follows.** Three things.

**A skip that names a component is a skip that matches its own root.** Name the
path.

**Read the count a check prints, not the status of the pipeline you put it in.**
A check that scans nothing and a check whose failures were swallowed both look
like success. The file count is the part a reader can compare against what they
expected to change.

**Two wrong diagnoses of the same script were corrected by the same method.** The
first was corrected by planting a marker, the second by running the command
unpiped. Neither was found by reading the script, and one of them was written
into a probe that itself used the pipeline it was testing for.

### FND-313 — Every figure from a single pair of runs carries an unmeasured layout term, and the discriminator is which stages moved

**Believed.** A stage cost measured before a change and after it, on the target
platform, at the same extent, unit count and thread count, gives what the change
was worth. The apparatus is tight, so the difference is the change.

**True for the first half and not for the second.** The apparatus is tight: two
runs of one tree agree to 0.7 percent on the stage in question and to 0.8
percent on the frame. What the difference holds is the change **plus the cost of
relaying the binary**, and the second term is not small.

**The project builds with full link-time optimisation and one code generation
unit.**[^F313A] Editing one module therefore moves code everywhere, and the
instruction cache and the branch predictor answer differently for loops that
were not edited at all.

**The measurement.** Machine C, `c7g.4xlarge`, Graviton3, `us-west-2`,
16,777,216 tiles, 1,000,000 units scattered, 12 threads, nine frames after two
warm-up frames, `stage-cost` feature. Two runs of the base tree at commit
`79d851d`, two runs of a tree that differs from it only inside the key vector
sort. Each run is its own instance.

| Stage | Base A | Base B | Changed A | Changed B |
|---|---|---|---|---|
| `bridge_refresh_barrier` | 31,394,191 | 31,181,809 | 36,459,500 | 29,068,978 |
| `holding_apply` | 19,048,855 | 18,762,498 | 20,686,347 | 20,353,567 |
| `holding_spread` | 71,234,385 | 70,731,392 | 74,083,110 | 73,897,963 |
| `holding_candidates` | 16,991,822 | 17,041,445 | 17,505,343 | 17,523,387 |
| `tile_scan` | 14,224,080 | 14,213,068 | 14,379,662 | 14,390,942 |
| `influence_solve` | 12,614,492 | 12,625,658 | 12,653,479 | 12,600,545 |
| `admit` | 21,274,145 | 20,923,668 | 18,226,851 | 17,929,533 |
| `frame_wall` | 177,862,658 | 176,501,059 | 184,219,536 | 175,551,738 |

**Nothing in the changed tree touches the holding, and three of its stages moved
together.** The apply is 8.5 percent higher in both runs, the spread 4.2 percent
and the candidate pass 2.9 percent. Those three share no code with the sort. The
two stages that are compute-bound and least sensitive to placement, the
influence solve and the tile scan, moved by 0.1 and 1.2 percent.

**One stage produced opposite answers from one binary.** The bridge rebuild
measured 36,459,500 and 29,068,978 nanoseconds on two runs of the identical
tree, a spread of 25 percent, and the lower of the two is below both base runs.
A single pair would have supported "a 16.9 percent regression" or "a 7 percent
improvement" with equal authority.

**Follows.** Four things.

**A single pair of runs bounds a claim at about eight percent, and no better.**
That is the size of the layout term seen here on stages that were not touched. A
claim below that floor from one pair is not evidence. Claims far above it are
untouched: a pass that fell twenty-four times, a stage that no longer exists and
a frame that fell more than four times are not layout.

**The discriminator is not the size of the change. It is whether the stages that
should not have moved did not.** A change is separable from a relayout when the
passes sharing no code with it are flat. This is the same instrument that showed
one repair to be the larger half of a pair, and another worker used it
independently to defend a 5.4 percent fall by showing every untouched stage
moving by less than half a millisecond. **A single pair is evidence only when
the stages that should not have moved did not.**

**Take a second run of the changed tree, not only of the base.** The base pair
establishes the apparatus. It says nothing about whether this particular binary
is stable, and the binary that changed is the one whose placement changed.

**Every figure names its instance, its commit and its thread count, or it cannot
be checked later.** A ratio that travels without its conditions cannot be
distinguished from a relayout by anyone who reads it afterwards.
### FND-314 — The decide pass sorted five million lists on each frame, three quarters of them held one entry or none, and removing the sort bought nothing

**Believed.** The decide pass is the largest stage in the engine and it already
takes a thread count, so it may simply be that expensive. Nothing had divided
it.

**True.** It reads about five million candidate tiles on each frame, and a
quarter of them raise more than one supporter. **It sorted every one of them.**
Three quarters of the sorts ordered a list of one entry or of none.

**Evidence.** A counting switch on the pass, at 16,777,216 tiles with one
million units. At frame 15 it read 4,965,973 candidates, raised 6,609,326
supporters, and 1,245,193 candidates raised more than one. About 1.33
supporters for each candidate, and 1,281,555 candidates had a challenger able
to beat the holder, which is 26 percent. A candidate with no challenger does
not read the ground, so that count is also the count of ground reads.

**The sort was not needed at all.** It existed to put the supporters in
ascending identifier order, so that a strict comparison would give the stable
key of support descending and identifier ascending. **Stating the tie in the
comparison gives the same key from any order**, so the sort is gone and the
result no longer depends on one.

The two are equivalent by construction rather than by measurement. The old form
takes the first faction that reaches the highest support in ascending
identifier order. The new form takes the highest support and, on a tie, the
lower identifier. Both name the same faction for every input.

**Follows.** Three things.

**A sort that exists to produce an order for one comparison can usually move
into the comparison.** The ordering was never wanted for itself. It was a way
of expressing a tie-break, and a tie-break is two lines where a sort is a pass.

**Removing it made the pass order-independent, which is worth more than the
time.** The result used to depend on the sort putting the supporters in one
order. It now depends on nothing but the set of supporters, so there is no
order left for a defect to disturb.

**The counting says what the pass does and not where its time goes, and the
measurement then said the sort was not the cost.** Four runs on the target
platform give the stage at 34,174,920, 34,134,468, 34,128,759 and 34,054,721
nanoseconds. The first two are the tree with the sort and the last two are the
tree without it. **The whole spread is 120 microseconds on 34 milliseconds, and
the change sits inside it.** Removing five million sorts on each frame bought
no time that this apparatus can see.

So the stage costs what it costs for the other reason: every candidate pays for
six neighbour reads and a walk of the units on its tile, and only a quarter of
them produce anything. A quarter of five million is still more than a million
tiles that change hands, so the work is not obviously wasted; it is the reading
of the other three quarters that is.

**The sort removal is kept anyway, and not for the time.** It removes work that
provably does nothing, and it makes the result depend on the set of supporters
rather than on an ordering of them. A change that buys no time and removes an
ordering is still worth making, and saying that it bought no time is the
honest way to keep it.

**Two runs of one tree separate the two kinds of noise, and this run pair did
it by accident.** Another finding records that this project links with whole
program optimisation and one code generation unit, so editing any file relays
the whole binary and every figure carries a layout term.[^F306B] **A layout is a
property of a binary.** Two runs of one tree run one binary, so their layout is
identical and any difference between them is not layout.

Runs 3 and 4 above are one tree. They gave the bridge refresh 39,742,813 and
30,695,188 nanoseconds, which is 29 percent apart on a stage that neither run
changed and whose layout was the same in both. **So that spread is the machine
and not the code.** The decide stage meanwhile held to 120 microseconds across
all four runs and two trees, so it carries little of either kind of noise.

That gives a protocol sharper than reading which stages moved. **Run each tree
twice.** The spread inside one tree bounds the machine. A difference between
trees that exceeds it is the layout and the change together, and a difference
that does not exceed it is nothing at all.
### FND-315 — A unit against a shoreline was told to walk into the water on every frame, and it stood there for ever

**Believed.** Movement takes its direction from the per-cell exit field, and a
cell that no neighbour beats holds no direction, so the field leaves no unit
without a rule.[^F190C] A unit that the ground refuses stays put for that frame
and moves on a later one.

**True.** It stays put for ever. The exit of a cell, the option of a unit and
the direction that pairs them are all inputs that hold from one frame to the
next, so a refusal repeats exactly. The fall-back to a draw fired only when the
cell held no direction at all. It did not fire when the cell held a direction
that the ground refused, which is the case that repeats.

**A cell covers a block of 32 tiles on a side, and the ground of one tile of
that block is not a fact the block carries.** So the field can hold a direction
that most of a block can take and that the tiles along one edge cannot. That
edge is a shoreline.

**Evidence.** The demonstration world, 256 by 256 at four factions. A unit was
placed on the first tile whose cell exit the ground under it refuses, and the
step was driven for 32 frames. The unit held its starting tile for all 32. With
the fall-back in place it left on an early frame. The test is the record of it,
and the commit body holds the command.[^F315B]

**A second cause sat above it: the rank could name ground that admits nobody.**
No summary field says whether a unit may stand in a cell, so a cell of open
water competes on the same terms as dry ground. The mean height row is where it
happens: the water of one cell may be shallower on average than the water of
the cell beside it, so the closed cell reads higher and takes the direction.

**That case is a tail of the generator and not the typical world.** A sweep of
forty seeds at 256 by 256 found a cell ranking closed ground above itself in
eleven of them, between one and three cells each. The seed this project uses
for its other fixtures is one of the twenty-nine that miss it.

**Follows.** Three things.

**A fall-back that fires on one refusal and not on another is not a fall-back.**
The record said the field leaves no unit without a rule, and it was true of the
case the author had in mind. The rule it left the other unit was a direction
that nothing could take, which is worse than no rule, because no rule reaches a
draw and this reached a wall.

**The fixture was the finding, and the first version of the test proved
nothing.** Written against the ordinary seed, the closed-ground assertion passed
with the defect put back, because no cell of that world ever ranked closed
ground first. The rule that a fixture must be built for the distribution the
test needs caught it, and only because the defect was put back and the test was
watched.[^23] A green test on the ordinary world would have shipped an
assertion that could not fail.

**A field at block pitch cannot answer a tile.** D5 removes the cells that admit
nobody and D6 stops the freeze, and neither makes a unit walk around a bay. The
field holds one direction for a block, and routing around an obstacle needs a
field that reaches further than one neighbour. That is a different claim and it
has no record.

### FND-316 — The delivery had no golden scenario and two of its own tests were empty

**Believed.** A pass that writes into state the golden hash covers is guarded by
the golden file, and a test that asserts an equality across that pass is a test
of the pass.

**True.** Neither held for the delivery of a carried load.

**The golden scenarios did not reach the pass.** The gathering scenario spawns
its units on deposit tiles and gives none of them a home site, so no unit ever
stood on the tile of its own site holding a load. Adding the whole delivery
pass moved no golden file at all. A golden file that cannot move is a guard
that has already stopped working, and this is the third pass to meet that gap.
The item that holds the class names the other two.[^F316A]

**Two of the pass's own tests could not fail.** The first asserted that the
store gained exactly what the carry lost. With no delivery both sides read
zero, and zero equals zero. The second compared three worlds at one, two and
twelve threads and found them identical, which they are when the pass never
runs in any of them.

**Evidence.** The delivery was removed and the suite was watched. Seven tests,
three failed. The two above passed, and so did the two negative tests, which
are correct to pass. After both were given a check that the fixture reached the
case, five of the seven fail with the pass removed.

**Follows.** Three things.

**An equality across a pass needs a witness that the pass ran.** An equality
between two quantities that a missing pass leaves at zero is not an assertion
about the pass. It is an assertion about zero.

**A thread-equivalence test is the easiest test in this project to write
empty.** It compares runs against each other and never against a stated value,
so it holds whenever the runs agree, including when they agree on having done
nothing. Every equivalence test wants a second assertion that the work happened
in each run.

**Removing the pass is the cheap check that finds all of this at once.** It
took one line and it separated the three real tests from the four that were
either empty or negative. The rule already says to put the defect back, and
this is the case where the defect is the whole pass.[^23]

### FND-317 — The delivery works and never runs: nothing gives a unit a reason to go home

**Believed.** Moving a carried load into a store closes the chain from the
ground to the store to the ration, so what a settlement holds now depends on
what its people fetched.[^F317A]

**True.** The chain is closed and nothing walks along it. The delivery fires
only for a unit that stands on the tile of its own site, and no rule in the
engine ever puts a unit there on purpose. A unit gathers where it stands and
then steps wherever the exit field of its cell points, which is chosen by the
food, the height, the open share or the crowd of a neighbouring cell. **None of
the four options is "go home".**

**Evidence.** The demonstration world, 256 by 256 at four factions and 64
people each, driven 4000 ticks. **The delivered total is zero for every kind on
every tick.** At tick 3200 the first faction had all 64 of its units carrying
food and its store read zero. The store of every short faction read zero on
every tick of the run, exactly as it did before the delivery existed, and both
factions that died before died at the same ticks: 820 and 3450.

**The measured zero belongs to the tree of that day.** A later change to
movement gave a refused unit a keyed draw, and the same run then delivered a
small amount by accident. A later finding holds that number and what it
cost.[^F317E]

**The pass is not wrong. It is unreachable from the behaviour.** Its own tests
pass because they construct the case, seating a site and a unit on one tile.
The engine does not construct it. This is the shape the rules call a capability
nobody invokes, met from the other side: the caller exists, and no behaviour
ever satisfies its condition.[^37]

**Follows.** Three things.

**A test that drives the engine can still miss an unreachable pass.** The rule
says to ask whether the engine or the user must invoke a mechanism, and to
start the test at the engine when it is the engine.[^F317C] These tests did
start at the step. They set up a world the engine never produces, so they
proved the pass works without proving anything reaches it. **Driving the real
caller and reaching the real case are two requirements, not one.**

**The economy is still a constant.** Every conclusion drawn from the demand
side of it still holds: a site's food rate is set once at founding from the
survey, and it never moves. Closing the sink did not change that, because the
sink takes nothing.

**A laden unit needs a reason to go home, and that is a behaviour claim with no
record.** The exit field ranks a neighbouring cell on a summary field, and a
site is not a summary field. Whatever answers this either adds a field that
says where a unit belongs, or admits that a unit's own site is a fact no cell
carries. The backlog holds the question.[^F317D]

### FND-334 — A delivered total above zero does not prove that anything steers a unit home

**Believed.** The delivery of a carried load never runs, and the total the
demonstration world delivers is zero for every kind on every tick.[^F334A] A
test that drives the demonstration world and asserts a delivered total above
zero therefore proves that a unit now goes home.

**True.** Neither half holds.

**The total is no longer zero, and it was not zero before this work started.**
The same run of the demonstration world, 256 by 256 at four factions and 64
people each, driven 4000 ticks at four threads, delivers 38 food. The
difference from the earlier measurement is the movement fall-back that landed
between the two: a unit whose ground refuses the exit of its cell now takes a
keyed draw, so a random walk puts a unit on the tile of its own site once in a
long while.[^F334B] **The delivery was reachable by accident and by nothing
else**, which is the same defect with a smaller number on it.

**A total above zero passes on an engine that steers nothing.** After the
option and the field were built, the return field was disconnected from the
movement pass, so a laden unit stopped gathering and then took a uniform draw.
The test that drives the demonstration world for 300 ticks and asserts a
delivered total above zero **passed**. A unit that only stops gathering still
walks, and it still reaches its own site once in a while.

**Evidence.** Five defects were put back one at a time and the suite was
watched. The disconnected field is the one that separated the two tests: the
test that asserts a total passed, and the test that names the tile a laden
unit must step onto failed. The commit body holds the list.

**Follows.** Three things.

**A test that asserts a total is a test about the total.** The behaviour under
it moves the total, and so does luck. An assertion that separates the two has
to name what the behaviour did, and the smallest such thing here is the tile
that the field sent a unit to.

**A finding that states a measured zero states it about a tree.** The zero of
the earlier finding was correct when it was taken. The next change to movement
made it wrong, and nothing failed, because a finding is prose. A reader who
takes a measured number from a register must take the date with it.

**Put the defect back for every claim, not for the work as a whole.** Four of
the five defects were caught by the test written for them, and the fifth was
caught by no test until one was written. Restoring the whole feature would
have shown four red tests and hidden the gap.

### FND-335 — Three properties of one golden fixture each blocked the same pass, one after another

**Believed.** The gathering golden scenario seats a site and gives its
gatherers a home, so a golden file moves when the delivery of a carried load
changes.[^F335A]

**True.** It did not reach the new option at all, and closing one gap only
uncovered the next.

**The interval.** A unit of that scenario chooses about once over the frames
the scenario runs, so it forages, gathers, and never chooses again. The option
that carries a load home is taken at the second choice, which never came.

**The load.** The deposits of that world hold between one and ten units each,
and the largest load any unit reaches over the frames is four, against a
default carry mark far above that. No unit was ever laden.

**The need.** The site held no store, so every unit of the scenario starved
inside the frames it runs. The option is driven by the need a unit still
holds, so a starving unit forages whatever it carries.

**Evidence.** The defect was put back after each repair and the golden test was
watched. It passed after the first, passed after the second, and failed only
after the third. The measurement of the loads and of the deposits came from a
harness that rebuilt the scenario outside the test, and the commit body holds
the command.

**Follows.** Two things.

**A fixture reaches a case or it does not, and repairing one property proves
nothing about the next.** The scenario looked closer after each change, and a
reader who stopped at the first would have recorded a guard that cannot fire.
Only the defect being put back after each step separated them.

**A parameter of the world is a parameter of the scenario.** Three values that
the engine holds a default for had to be stated by the scenario, for the same
reason the promotion threshold and the recovery periods already are: a default
chosen for a world that runs for a long time reaches nothing in a scenario
that runs for a few frames.

### FND-318 — The per-unit accumulator does not remove the cliff, and the draw that replaced it created food

**Believed.** A whole-group model has a cliff: a place is fine a little above
its demand and starves entirely a little below it. The per-unit deficit
accumulator removes the cliff, because a shortage then degrades before it
kills. An accepted record says this three times.[^F318A]

**True.** The accumulator delays the cliff and does not spread it. Every unit
of a cohort gains the same share, decays at the same rate and founds with the
same need, so every unit holds the same accumulator value for ever and crosses
the death bound on one tick.

**Evidence.** The demonstration world, 256 by 256 at four factions and 64
people each, driven 4000 ticks with the deficit of every unit read on every
tick. **One distinct deficit value for each faction, on every tick of the
run.** Two of the four factions lost all 64 units on a single tick each, at
tick 820 and tick 3450. The other two never lost one.

**The equilibrium is what makes it certain.** The decay saturates at zero, so a
unit's need settles at exactly the ration it is granted, and nothing moves that
number afterwards. Measured per application: the four sites granted 32,000,
29,440, 37,760 and a full ration against a threshold of 32,768. The two below
the threshold accrued deficit at a constant rate and died. The two above it
never accrued any.

**The population is the variable that would fix it and it never changed.**
Supply is fixed and demand scales with the headcount, so the second faction's
supply covered 46 of its 64 units. Losing 18 would have left 46 fed to full,
for ever. The engine removed 64.

**The first replacement created food.** A keyed draw for each unit, on each
application, gives each unit an independent chance of eating. The number that
eats is then binomial and not the count the store paid for. Measured on a
fixture whose share covered exactly one ration: **two units ate on one
application and none on another.** The conservation check did not fail, because
a need is not a conserved quantity and the commodity had correctly left the
store. The units simply received more ration than the store handed over.

**What works is a rotation.** Each unit holds its place inside its own cohort,
which the rebuild already walks and counts. A cohort draws one offset for the
frame, and a unit eats when its ordinal advanced by that offset falls below the
served count. A rotation is a bijection, so exactly as many units eat as the
share covered.[^F318B]

**After the change**, the same 4000-tick run loses units a few at a time and
settles. The four factions end at 53, 46, 62 and 64 of 64. **The second lands
on exactly the 46 its supply carries**, with a deficit of zero and a store that
now grows.

**Follows.** Three things.

**A record can state a property that its own decisions do not deliver.** ADR-0063
D1 is correct about what a per-unit need buys. The claim that it removes the
cliff is a claim about a distribution, and nothing in the record produces one.
Every input to a need was shared, so the outputs were shared too. **Look for the
thing that differs between two units before believing that two units differ.**

**A draw is not a shuffle, and the difference is conservation.** An independent
draw for each member gives the right answer on average and the wrong answer on
any one application. Where a count has been paid for, the selection has to be a
bijection. This was caught by a test that asserted the served count against the
count the share covered, and by nothing else: the conservation check passed
throughout, because the quantity it conserves is not the one that was wrong.

**An invariant that passes is not a proof that the right invariant exists.** The
store conservation check and the carry conservation check both held while a
cohort was eating rations the store never gave. The account they balance is the
commodity, and the ration a unit receives is not that account.

## References

[^F326A]: The head-up display, the row drawing. `crates/cachette-view/src/hud.rs`
[^F218B]: ADR-0067, the viewer reads the world and never writes to it, decision D4. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^F327B]: ADR-0094, the caller owns the camera and the pixels, decision D1. `docs/adrs/draft/adr-0094-the-caller-owns-the-camera-and-the-pixels.md`
[^F328A]: ADR-0070, the head-up display reports what the drawing pass read, decision D1. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`
[^F315B]: The refused step test. `crates/cachette-core/tests/a_refused_step_does_not_freeze.rs`
[^F318A]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D1 and the alternatives rejected. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^F318B]: ADR-0106, a cohort serves whole rations to a keyed subset, never an equal share to everybody, decision D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^F317A]: Backlog item 0187, give a carried load somewhere to go. `docs/backlog/complete/0187-give-a-carried-load-somewhere-to-go.md`
[^F317C]: Testing rules, section 5. `.claude/rules/testing.md`
[^F317D]: Backlog item 0305, give a laden unit a reason to go home. `docs/backlog/complete/0305-give-a-laden-unit-a-reason-to-go-home.md`
[^F316A]: Backlog item 0279, let a golden scenario reach the position pass. `docs/backlog/proposed/0279-let-a-golden-scenario-reach-the-position-pass.md`

### FND-320 — The type stub claims a check that regenerates it, and no generator and no check exist

**Believed.** The type stub for the compiled extension module is a generated
artefact. Its own docstring says that the contributing guide requires the
continuous integration system to check the stubs, that the build regenerates
them, and that the job fails when the result differs from the file.[^F320A]

**True.** No stub generator exists anywhere in the tree. No workflow job
regenerates the stub, and no job compares it against anything. The stub is
hand-written. The contributing guide never states the requirement that the
docstring attributes to it. Its only use of the word "stub" describes the Rust
crates as unimplemented.[^F320B]

**The claim is wrong twice.** It names a guide that says something else, and it
names a mechanism that does not exist. A contributor who changes a binding and
reads that docstring concludes that a job will catch a stale stub. Nothing will.

**A second half makes the first half expensive.** The stub carries a docstring
for each typed dictionary and for each exception class. It carries none for any
method of `World` and none for any method of `Camera`. The Rust bindings crate
carries that prose, and PyO3 puts it on the compiled objects. So the stub and
the compiled module are two declaration sites for the public interface, they
already disagree about the prose, and nothing fails.[^F320C]

**Evidence.**

```
grep -rniE "stub|pyi|pyo3-stub-gen" justfile scripts/ .github/ crates/cachette-py/Cargo.toml
grep -niE "stub|\.pyi" CONTRIBUTING.md
```

The first command finds two matches, and neither generates or compares a stub.
The second finds one match, and it describes the Rust crates.

A documentation build measured the second half. A site built from the compiled
module produced a page of 105,348 bytes that held the method prose. The same
site built with module inspection turned off fell back to the stub, produced
29,038 bytes, and held no method prose at all.[^F320D]

**Follows.** Repair the stub docstring, or make the claim true with a
generator. The research report on the documentation toolchain treats the
Rust doc comment as the single source of the prose for exactly this
reason.[^F320D] A generator would also remove the second declaration site for
every signature, which is the shape the recurring defect rule names.[^F320E]

### FND-321 — The stub already carries prose the Rust source owns, and the copy that drifted dropped the paragraph that warns against a copy

**Believed.** The register records that the type stub and the compiled module
are two declaration sites for the public interface, and that they already
disagree about the prose.[^F321A] The disagreement it names is an absence: the
stub carries no docstring for any method of `World` and none for any method of
`Camera`.

**True, and the sharper half is what the stub does carry.** Nine exception
classes carry a docstring in the stub. All nine are the same words as the string
that the Rust macro gives the same exception, character for character.[^F320C]
The `World` class docstring is a one line copy of the Rust doc comment, and it
agrees. The `Camera` class docstring is an abridged copy of the Rust doc comment,
and it does not agree: it keeps the first and the last paragraph and drops two.

**One of the dropped paragraphs is the one that names this defect.** The Rust
doc comment says that a pan share and a zoom step written on both sides of the
boundary would be one value in two places, with nothing failing when the copies
disagreed. The stub is that second place, and the sentence did not survive the
copy into it.

**Evidence.**

```
grep -n "create_exception" crates/cachette-py/src/lib.rs
sed -n '1303,1320p' crates/cachette-py/src/lib.rs
sed -n '280,336p' python/cachette/_core.pyi
```

**Follows.** The count of copies is eleven, not zero, and nine of them agree
today. **An agreeing copy is the worse case**, because it reads as a maintained
file and gives a contributor no reason to look for the other site. The record on
the provenance of the documentation prose forbids a docstring on a stub member
that the compiled module provides, and it names this as a defect it does not
fix.[^F321C] A backlog item holds the removal and the check.[^F321D]
### FND-329 — A backlog item decays fastest at the sentence that tells a reader not to take it

**Believed.** A backlog item in `proposed/` is an idea and costs nothing while
it waits. The guide says an item there may be one sentence, so a stale item is
a small loss and the priority index carries the judgement that matters.[^F329A]

**True.** An item decays like any other document, and the sentence that decays
first is the one a reader acts on. An audit of the 89 items in `proposed/` on
3 September 2026 found the same shape in nine of them, and in three cases the
stale sentence was the reason nobody had taken the item.

**The worst case is a deferral whose reason is gone.** Item 0039 carried a
section headed "Do not build this yet". Its reason was that a unit has no plan
and draws a fresh direction on each frame, so a refused unit does not repeat the
choice that failed. A unit now takes its direction from the exit of its cell for
the option it chose, and every input to that holds from one frame to the
next.[^F329B] The demonstration world was driven for 400 ticks: a unit held its
tile against a target the ground admits 61 times, and one unit was refused on
five consecutive frames. **The item said the condition could not arise, and the
engine produced it 61 times in one run.**

**Two other deferrals had the same shape.** Item 0272 says no measurement exists
on the target platform. One does, and it names the pass this item is about: the
choice costs 0.571 milliseconds of a frame of about 836.[^F329C] Item 0270 asks
for a vector rewrite of that pass and quotes 71.4 milliseconds for it from the
same register, which marks the figure stale in its own words on another
page.[^F329C] **One register said both things at once, and two items read the
half that suited them.**

**A closed premise also makes an item invisible.** Item 0206 states in its own
first section that another worker closed every gap it names, and asks whoever
merges the two to close it. It stayed open. The priority index carried a row
saying "Do not take it", so the index was paying to point at an item that should
not have existed.

**Four items carried a count that the tree had moved past.** Item 0278 said the
demonstration world holds no character and that no unit holds a ranked position.
Driven again, the same world seats 16 of 32 ranked positions and holds 26
characters at tick 200. Item 0145 said the faction coercion is written at six
places; there are five in the module and more outside it. Item 0222 said the
error hierarchy holds seven leaves; it holds eight. Item 0072 said one test
calls the panel fit check; two do.

**Evidence.** The audit read every item in `proposed/`, then measured the three
claims above by driving the engine rather than by reading it. The probe built
the demonstration world, founded a run for every faction, stepped 400 frames and
printed the counts. The commit body holds the command and the output.

**Follows.** Four things.

**A count in a backlog item rots exactly as a count in a decision record
does.** The scope rule bans a count from a record and sends it to the commit
message.[^53] The backlog has no such rule, and it does not need the same
one, because a count is often the whole argument for an item. It needs the
weaker rule: **a count in an item is dated evidence, not a current fact, and a
reader who plans against it derives it again first.**

**A "do not take this yet" section is the highest-value line in the backlog and
nothing rechecks it.** It stops work. Its reason is a claim about the code, and
the code moves. **State the reason as a condition that can be tested, not as a
description of how things are**, so a later reader can run something and find
out.

**An item that says it is superseded is not closed by saying so.** Item 0206
said it for a day and the index repeated it. The close is the work.

**The priority index and the item can hold opposite claims and no check sees
it.** The index said the project pins a stable toolchain, so the portable vector
library is out of reach. Item 0270 says the project moved to a dated nightly and
that the move was made for that item, and the toolchain file agrees with the
item.[^F329E] Both documents were right when written. The checks compare an
index against the set of open items and never against what an item says.


## References

[^F320A]: The type stub for the compiled module. `python/cachette/_core.pyi`
[^F320B]: Contributing guide, the opening section. `CONTRIBUTING.md`
[^F320C]: The Python bindings crate. `crates/cachette-py/src/lib.rs`
[^F320D]: Research report 19, the documentation toolchain, sections 4.2 and 7. `docs/research/reports/19-documentation-toolchain.md`
[^F320E]: Recurring Defect Shapes, shape 1, redundant declaration sites. `.claude/rules/recurring-defects.md`
[^F321A]: Findings register, FND-320, in this document.
[^F321C]: ADR-0107, the Python reference is generated from the compiled module, decision D3. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^F321D]: Backlog item 0307, generate the type stub from the compiled module. `docs/backlog/proposed/0307-generate-the-type-stub-from-the-compiled-module.md`

[^F329A]: Backlog guide, the line between proposed and refined. `docs/backlog/README.md`
[^F329B]: Findings register, FND-315, in this document.
[^F329C]: Target platform costs, the stage split and where the unit cost goes. `docs/reference/graviton-costs.md`
[^F329E]: The pinned toolchain. `rust-toolchain.toml`

[^F261B]: The holder count test of the viewer. `crates/cachette-view/tests/shows_who_holds_the_ground.rs`
[^F261C]: Backlog item 0271, count the ground generations that one frame runs. `docs/backlog/proposed/0271-count-the-ground-generations-that-one-frame-runs.md`

[^F262A]: Backlog item 0238, decide per cell and need rather than per unit. `docs/backlog/complete/0238-decide-per-cell-and-need-rather-than-per-unit.md`
[^F262B]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^F262D]: Findings register, FND-051 and FND-048, in this document.
[^F262E]: ADR-0098, the choice is decided for each cell and each bucket of need, the consequences. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^F263A]: Review of ADR-0096, correction 1. The review artefact sits on the branch that holds it, so this branch cannot resolve its path and the citation names it instead.
[^F263B]: Target platform costs, would the choice pass collapse if it decided for each cell. `docs/reference/graviton-costs.md`
[^F263C]: The need spread measurement. `crates/cachette-core/tests/need_spread.rs`
[^F263D]: Decisions register, DEC-106. `docs/DECISIONS.md`
[^F263E]: Budgets and costs, what belongs here. `docs/reference/budgets.md`
[^F263F]: Findings register, FND-262, in this document.
[^F226A]: Backlog item 0185, steer a step by the option the unit chose. `docs/backlog/complete/0185-steer-a-step-by-the-option-the-unit-chose.md`
[^F226B]: Backlog item 0186, let the engine order a gather. `docs/backlog/complete/0186-let-the-engine-order-a-gather.md`
[^F226C]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^F226E]: Backlog item 0240, let the demonstration make a unit hungry. `docs/backlog/complete/0240-let-the-demonstration-make-a-unit-hungry.md`
[^F227B]: Findings register, FND-183, in this document.
[^F258A]: Findings register, FND-223, in this document.
[^F258B]: The footnote baseline. `scripts/footnote-baseline.txt`
[^F258C]: Backlog item 0242. `docs/backlog/refined/0242-fail-a-check-when-a-document-states-a-register-in-its-own-words.md`
[^F259CHECK]: Backlog item 0244. `docs/backlog/refined/0244-fail-a-check-when-the-two-project-orientations-disagree.md`
[^F222]: Target platform costs. `docs/reference/graviton-costs.md`
[^F223C]: ADR Registry, how a record changes. `docs/adrs/REGISTRY.md`

[^F177A]: The founding refuses ground that admits nobody. `crates/cachette-core/src/world.rs`
[^F177B]: The terrain capacity table. `crates/cachette-core/src/terrain.rs`
[^F180A]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^F180B]: What a unit does in a tick, section 3.1. `docs/research/what-a-unit-does-in-a-tick.md`
[^F180C]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^F180D]: Backlog item 0064, choose an action by scoring a fixed option set. `docs/backlog/complete/0064-choose-an-action-by-scoring-a-fixed-option-set.md`
[^309A]: The local benchmark finding of 2 September 2026: a release build served an executable two hours old, and three trees measured the same. It is recorded on the trunk and this branch predates it, so its number is owed here.
[^309B]: The stage cost measurement, reported on 3 September 2026 and not yet recorded. It gains a number when it lands.
[^309C]: The per-recipe timing harness. `scripts/gate-times.sh`
[^309D]: Testing rules, section 2a, a uniform input hides a defect. `.claude/rules/testing.md`
[^F181B]: Testing Rules, section 5. `.claude/rules/testing.md`
[^F181C]: Findings register, FND-180, in this document.
[^F181D]: What a unit does in a tick, section 3.4. `docs/research/what-a-unit-does-in-a-tick.md`
[^F181E]: What a unit does in a tick, section 6.1. `docs/research/what-a-unit-does-in-a-tick.md`
[^F182A]: Backlog item 0183, carry the food of a cell into the level 1 summary. `docs/backlog/complete/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
[^F182B]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D1. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^F182C]: ADR-0072, a tile stock is generated, and only what was taken is stored, decision D4. `docs/adrs/accepted/adr-0072-a-tile-stock-is-generated-and-only-what-was-taken-is-stored.md`
[^F182D]: ADR-0068, terrain is generated from the seed and is never stored as a map, the consequences. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^F182E]: Blockers register, BLK-007, in the blockers register. `docs/BLOCKERS.md`
[^F183A]: Findings register, FND-181, in this document.

[^F168A]: ADR-0084, the world reserves the unit columns at construction, decision D3. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
[^F168B]: Review 0175, the unit reservation record. `docs/reviews/0175-the-unit-reservation-record.md`
[^F171A]: Testing rules, sections 2 and 2a. `.claude/rules/testing.md`
[^F174B]: Backlog item 0179. `docs/backlog/proposed/0179-give-a-golden-scenario-a-build.md`

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
[^F137D]: Backlog item 0153. `docs/backlog/refined/0153-let-python-read-an-event-without-repeating-its-layout.md`
[^97]: Development budgets, the gate suite budget. `docs/reference/development-budgets.md`
[^98]: Testing rules, section 2a. `.claude/rules/testing.md`
[^99]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^100]: ADR-0083, the gate build checks every integer overflow, decision D2. `docs/adrs/draft/adr-0083-the-gate-build-checks-every-integer-overflow.md`
[^F147A]: ADR Registry, row 0043. `docs/adrs/REGISTRY.md`
[^ORIENT2]: Project orientation, the design principles. `CLAUDE.md`
[^139]: ADR-0084, the world reserves the unit columns at construction. `docs/adrs/draft/adr-0084-the-world-reserves-the-unit-columns-at-construction.md`
[^141]: Backlog item 0080. `docs/backlog/proposed/0080-give-the-world-settings-a-constructor.md`
[^142]: Findings register, FND-064, in this document.
[^144]: ADR-0014, entity identity is an index plus a generation, decisions D1 and D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^101]: Findings register, FND-142, in this document.
[^102]: Review 0164, the gate build profile record. `docs/reviews/0164-the-gate-build-profile-record.md`
[^145]: Findings register, FND-130, in this document.
[^146]: Backlog item 0144. `docs/backlog/complete/0144-check-the-footnotes-of-a-record.md`
[^147]: Definition of Done, pass the gates. `.claude/rules/definition-of-done.md`
[^148]: The citation check. `scripts/check_citations.py`
[^149]: Backlog guide, completing an item. `docs/backlog/README.md`
[^150]: Backlog item 0163. `docs/backlog/proposed/0163-fail-when-a-merged-item-still-reads-as-open.md`
[^151]: Findings register, FND-152, in this document.
[^F162A]: Findings register, FND-086, in this document.
[^F162B]: Backlog item 0112. `docs/backlog/complete/0112-build-a-world-without-a-pass-over-every-tile.md`
[^F162D]: Backlog item 0171. `docs/backlog/proposed/0171-build-the-first-level-without-a-pass-over-every-tile.md`
[^F159A]: Influence maps, section 5.1. `docs/research/reports/09-influence-maps.md`
[^F159B]: ADR-0060, an influence map is stored as a shared basis, decision D2. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^F160A]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^F160B]: ADR-0009, parallel stages write disjoint outputs, because the memory model is weak. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^F174A]: Testing rules, section 2. `.claude/rules/testing.md`
[^F190A]: What a unit does in a tick, section 5. `docs/research/what-a-unit-does-in-a-tick.md`
[^F190B]: The option score. `crates/cachette-core/src/choose.rs`
[^F190C]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^F191A]: Backlog item 0181, give a kind of work the commodity it fills. `docs/backlog/proposed/0181-give-a-kind-of-work-the-commodity-it-fills.md`
[^F191B]: The founding provisions a site, and the consumption pass draws a ration. `crates/cachette-core/src/world.rs`
[^F192A]: ADR Registry, the retired numbers. `docs/adrs/REGISTRY.md`
[^F192B]: The record check script. `scripts/check_adrs.py`
[^F192C]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^F192D]: Backlog item 0198, tell a mention of a record number from a citation of it. `docs/backlog/proposed/0198-tell-a-mention-of-a-record-number-from-a-citation.md`
[^F193A]: The terrain capacity table and its fold. `crates/cachette-core/src/terrain.rs`
[^F193B]: The composition of the ground and a finished upgrade. `crates/cachette-core/src/upgrade.rs`
[^F193C]: The capacity that bounds the positions of a site. `crates/cachette-core/src/position.rs`
[^F193D]: Review 0199, the influence, tile field, upgrade and housing records. `docs/reviews/0199-the-influence-tile-field-upgrade-and-housing-records.md`
[^F193F]: Decisions register, DEC-081. `docs/DECISIONS.md`
[^F194C]: Backlog item 0201, scan the tree the check was asked to read. `docs/backlog/proposed/0201-scan-the-tree-the-check-was-asked-to-read.md`
[^F193G]: The drawing pass counts a tile at its capacity. `crates/cachette-view/src/paint.rs`
[^F193H]: Review 0204, the two corrected records. `docs/reviews/0204-the-two-corrected-records.md`
[^F197A]: The citation check script. `scripts/check_citations.py`
[^F197C]: Decisions register, DEC-083. `docs/DECISIONS.md`
[^F186A]: The viewer suite for the food and the reason. `crates/cachette-view/tests/shows_the_food_and_the_reason.rs`
[^F187A]: The viewer suite for the ground. `crates/cachette-view/tests/draws_the_ground.rs`
[^F187B]: PRD-0003, a developer sees a world worth looking at. `docs/product/accepted/prd-0003-a-developer-sees-a-world-worth-looking-at.md`
[^F198A]: The panel and the cards. `crates/cachette-view/src/`
[^F199A]: The agent protocol server. `python/cachette/agent/server.py`
[^F199B]: Backlog item 0206. `docs/backlog/complete/0206-let-the-agent-tool-read-what-the-panel-reads.md`
[^F200A]: The viewer suite for the glass. `crates/cachette-view/tests/shows_the_moment_on_the_glass.rs`
[^F201A]: Findings register, FND-193, in this document.
[^F201B]: Backlog item 0208. `docs/backlog/complete/0208-draw-the-boundary-of-a-holding-and-not-of-every-tile.md`
[^F194REF]: Findings register, FND-194, in this document.
[^F202A]: Backlog item 0152, what is still open. `docs/backlog/complete/0152-let-an-agent-drive-the-engine-through-a-protocol-server.md`
[^F202B]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
[^F202C]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^F202D]: PRD-0005, a watcher can tell what is happening and why. `docs/product/shipped/prd-0005-a-watcher-can-tell-what-is-happening-and-why.md`
[^F202E]: Recurring Defect Shapes, shapes 1 and 2. `.claude/rules/recurring-defects.md`
[^F202F]: ADR-0092, the agent tool surface grows one tool at a time, against a stated need. `docs/adrs/draft/adr-0092-the-agent-tool-surface-grows-against-a-stated-need.md`
[^F202G]: PRD-0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`
[^F204C]: Findings register, FND-048, in this document.
[^F206B]: Findings register, FND-201, in this document.
[^F206C]: The holder layer of the drawing pass. `crates/cachette-view/src/paint.rs`
[^F207A]: The tile rectangle of the drawing pass. `crates/cachette-view/src/paint.rs`
[^F209A]: Findings register, FND-208, in this document.
[^F209B]: The drawing pass of the viewer. `crates/cachette-view/src/paint.rs`
[^F209C]: Backlog item 0210, generate the ground of a drawn tile once. `docs/backlog/complete/0210-generate-the-ground-of-a-drawn-tile-once.md`
[^F207B]: Decisions register, DEC-088. `docs/DECISIONS.md`
[^F208A]: ADR-0070, the head-up display reports what the drawing pass read, decision D2. `docs/adrs/accepted/adr-0070-the-head-up-display-reports-what-the-drawing-pass-read.md`

[^F214A]: Decision Record Scope, section 5. `.claude/rules/adr-scope.md`
[^F214B]: ADR-0009, parallel stages write disjoint outputs, decisions D1 and D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^F214D]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/draft/adr-0044-what-copies-and-what-does-not-is-declared-at-the-call-site.md`
[^F215A]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop, decision D5. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^F215B]: The shape tier declarations. `crates/cachette-core/src/tier.rs`
[^F215C]: The bindings. `crates/cachette-py/src/lib.rs`
[^F215D]: The agent protocol server, the one-unit tool. `python/cachette/agent/server.py`
[^F215E]: ADR-0040, Python is a control plane, not a data plane, decision D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^F215F]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^F215G]: Findings register, FND-147, in this document.
[^F216A]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^F216B]: The character arena ceiling, and the shape tier declarations. `crates/cachette-core/src/tier.rs`
[^F216D]: Review 0223, the tier record. `docs/reviews/0223-the-tier-record.md`
[^F217A]: The grid index function. `crates/cachette-core/src/hex.rs`
[^F217B]: The block layout key. `crates/cachette-core/src/bridge.rs`
[^F217C]: The block fold of the pyramid. `crates/cachette-core/src/pyramid.rs`
[^F217D]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^F217E]: ADR-0017, the world is a rhombus, so a tile index is raw axial, decision D1. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^F217F]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^F217G]: Review 0223, the selector range record. `docs/reviews/0223-the-selector-range-record.md`
[^F218D]: ADR Registry, repairing a citation is not an amendment. `docs/adrs/REGISTRY.md`
[^F218E]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
[^F218F]: Findings register, FND-192, in this document.
[^ALLOC2]: Findings register, FND-219, in this document.

[^237B]: Decision Record Scope, section 6. `.claude/rules/adr-scope.md`
[^238A]: The gate recipes. `justfile`
[^238B]: Testing Rules, section 1. `.claude/rules/testing.md`
[^238C]: Definition of Done, section 5. `.claude/rules/definition-of-done.md`
[^FND277A]: Target platform costs, every stage of a frame by name, and huge pages. `docs/reference/graviton-costs.md`
[^FND278A]: Backlog item 0269, map the large arrays with huge pages. `docs/backlog/complete/0269-map-the-large-arrays-with-huge-pages.md`
[^235A]: The record of descent, the labelled row count. `crates/cachette-core/src/descent.rs`
[^236A]: Backlog item 0097. `docs/backlog/complete/0097-write-the-layout-record-with-the-descent-columns.md`
[^236B]: Backlog item 0067, record a parent and walk a line. `docs/backlog/complete/0067-record-a-parent-and-walk-a-line.md`
[^236D]: ADR-0021, a layout claim names one structure and one pass, and never a tier, decision D1. `docs/adrs/draft/adr-0021-layout-follows-the-access-pattern.md`
[^37HOME]: Decisions register, DEC-017. `docs/DECISIONS.md`
[^37NEXT]: Decisions register, DEC-093. `docs/DECISIONS.md`

[^F250A]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^F250B]: The choice pass. `crates/cachette-core/src/choose.rs`
[^F251A]: The world, the weight profile field. `crates/cachette-core/src/world.rs`
[^F251B]: The choice pass, the cell fields and the option set. `crates/cachette-core/src/choose.rs`
[^F252A]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^F252B]: ADR-0009, parallel stages write disjoint outputs, decision D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^F252C]: The choice pass of the world. `crates/cachette-core/src/world.rs`
[^F252D]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D4. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^239A]: The footnote check. `scripts/check_footnotes.py`
[^239B]: The priority check. `scripts/check_priority.py`
[^F257B]: The merge-defect check. `scripts/check_merge_defects.py`
[^F231C]: The starvation suite of the core. `crates/cachette-core/tests/starvation.rs`
[^F234A]: Findings register, FND-233, in this document.
[^F201C]: Findings register, FND-206, in this document.

[^F281A]: Backlog item 0267, hold the exit direction on the tile. `docs/backlog/complete/0267-hold-the-exit-direction-on-the-tile.md`
[^F281B]: The exit locality benchmark. `crates/cachette-core/benches/exit_locality.rs`
[^F281D]: Decisions register, DEC-105. `docs/DECISIONS.md`
[^F282A]: Backlog item 0268, hold the cell index on the unit. `docs/backlog/complete/0268-hold-the-cell-index-on-the-unit.md`
[^F282B]: Findings register, FND-252, in this document.
[^F282C]: The grid address conversion. `crates/cachette-core/src/hex.rs`
[^F282D]: Backlog priority index. `docs/backlog/PRIORITY.md`
[^F283A]: Project orientation, hard invariants 2 and 4. `CLAUDE.md`
[^F283B]: ADR-0041, a crate split enforces the boundary at compile time. `docs/adrs/draft/adr-0041-a-crate-split-enforces-the-boundary-at-compile-time.md`
[^F283C]: The testing guide, section 3.5. `docs/TESTING.md`
[^F283D]: The settlement store asserts that it is plain data. `crates/cachette-core/src/site.rs`
[^F283E]: Recurring defect shapes, inert code that nothing invokes. `.claude/rules/recurring-defects.md`
[^F283F]: The state-byte gate. `crates/cachette-core/tests/state_bytes_are_initialised.rs`
[^F284A]: The float ban script. `scripts/check-float-ban.sh`
[^F284B]: Backlog item 0272, name the reassociating methods in the lint. `docs/backlog/proposed/0293-name-the-reassociating-methods-in-the-lint.md`
[^F273A]: Backlog item 0266, order the unit arena by cell. `docs/backlog/refined/0266-order-the-unit-arena-by-cell.md`
[^F273B]: Target platform costs, the packed and scattered rows. `docs/reference/graviton-costs.md`
[^F273C]: The cost benchmark, the arena order mode. `crates/cachette-core/benches/target_cost.rs`
[^F274A]: Decisions register, DEC-110. `docs/DECISIONS.md`
[^F274B]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D1. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^F274C]: The drifted arena suite of the core. `crates/cachette-core/tests/drifted_arena.rs`
[^F274D]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^F287A]: Findings register, FND-281, in this document.
[^F287B]: Findings register, FND-282, in this document.
[^F287C]: The exit locality benchmark, the frame row. `crates/cachette-core/benches/exit_locality.rs`
[^F285A]: Findings register, FND-269, in this document.
[^F286A]: Target platform costs, every stage of a frame after the candidate pass became a bit plane. `docs/reference/graviton-costs.md`
[^F277B]: Findings register, FND-285, in this document.
[^F277C]: Findings register, FND-286, in this document.
[^F292A]: The world, the stored tile change count. `crates/cachette-core/src/world.rs`
[^F292B]: The exit locality benchmark, the growth row. `crates/cachette-core/benches/exit_locality.rs`
[^F292C]: Target platform costs, the stage table. `docs/reference/graviton-costs.md`
[^F298A]: The cost benchmark, the memory point mode. `crates/cachette-core/benches/target_cost.rs`
[^F298B]: Findings register, FND-246, in this document.
[^F299A]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D1. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^F295A]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D4. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^F296B]: Target platform costs, every stage of a frame after the ground read moved last. `docs/reference/graviton-costs.md`
[^F296C]: Findings register, FND-290, in this document.
[^F297A]: The target platform benchmark script. `scripts/graviton-benchmark.sh`
[^F300A]: Backlog item 0277, hold a thread back when the work will not pay for it. `docs/backlog/proposed/0277-hold-a-thread-back-when-the-work-will-not-pay-for-it.md`
[^F300B]: The demonstration stage split. `crates/cachette-core/examples/demo_stage_split.rs`
[^F300C]: ADR-0060, an influence map is stored as a shared basis, not one plane per faction, decision D4. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^F305A]: Findings register, FND-300, in this document.
[^F304A]: Findings register, FND-299 and FND-295, in this document.
[^F301A]: ADR-0071, the bridge rebuild orders on one thread, decision D2. `docs/adrs/accepted/adr-0071-the-bridge-rebuild-orders-on-one-thread.md`
[^F302A]: Decisions register, DEC-111. `docs/DECISIONS.md`
[^F307A]: Target platform costs, every stage of a frame after the block masks became counts. `docs/reference/graviton-costs.md`
[^F307B]: Findings register, FND-292, in this document.
[^F307C]: The holding suite of the core. `crates/cachette-core/tests/holding.rs`
[^F313B]: ADR-0103, the tile value field stores a dense delta, never a sparse change list, decision D4. `docs/adrs/draft/adr-0103-the-tile-value-field-stores-a-dense-delta.md`
[^F314X]: Findings register, FND-297, in this document.
[^F313A]: The workspace manifest, the release profile. `Cargo.toml`
[^F306B]: Findings register, FND-313, in this document.
[^F311A]: Findings register, FND-304, in this document.
[^F319A]: Project orientation, the Python example. `README.md`
[^F319B]: Recurring defect shapes, shape 5. `.claude/rules/recurring-defects.md`
[^F319C]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`

### FND-322 — The document a newcomer reads first sells a pyramid level that nothing writes

**Believed.** The orientation document of this repository states that the engine
organises the world into three levels of detail, and it describes the third:
region-scale summaries that combine blocks of level 1 cells. It lists the
spatial pyramid among the key capabilities of the engine, and it says that three
levels of detail provide instant answers for continental aggregate
queries.[^F322A]

**True.** Level 2 does not exist. The pyramid holds one derived level. The
project instruction file says so in bold, and it warns that a reader who takes
the three-level paragraph for the code plans against a level that nothing
writes.[^F322B] The Rust source agrees in its own words: the pyramid names the
whole-world summary as what level 2 **would** hold, in the conditional.[^F322C]

**Evidence.**

```
grep -n "Level 2" README.md
grep -rn "level 2" crates/ --include="*.rs"
```

**Follows.** The two orientation documents disagree, and one of them is the only
document a person outside this project reads. This is the shape a backlog item
already holds for a different sentence in the same pair of files, and the finding
behind it records the same cost.[^F322D] The documentation plan inherits the
claim, because the reader the plan serves arrives through this document, and the
worked example the plan turns into a tutorial sits on the same page as the false
paragraph.[^F322E] Repair the paragraph, or delete it and let the generated
reference report what the pyramid holds.

### FND-323 — The orientation document sells the agent server as a product capability, and the package does not install it

**Believed.** The orientation document lists native support for artificial
intelligence agents among the key capabilities of the engine. It says an
integrated Model Context Protocol server connects external agents directly to
the engine, it names the capability again under a third audience, and it gives
the command that starts the server.[^F322A]

**True, in the sense that the server exists, and false in the sense a reader
takes.** The server is a tool for a contributor to this repository. Its own
docstring says so, and it says that the reference implementation of the protocol
is a development dependency and not a runtime dependency of the
package.[^F323A] The project manifest agrees: the protocol library sits in the
development dependency group, with a comment that states the reason. The runtime
dependency list holds one entry, and it is not that library.[^F323B]

**A reader who follows the document gets an import error.** Install the package
from an index, run the command the document gives, and nothing starts, because
the package never installed the library the server imports.

**The product records draw the line the other way round.** The record that holds
the game developer states three times that the tool server is not part of that
need: it serves an agent that works on this repository, and another record holds
that audience.[^F323C] [^F323D]

**Evidence.**

```
grep -n "cachette.agent\|Model Context Protocol" README.md
grep -n "mcp>=" pyproject.toml
grep -n "dependencies" pyproject.toml
```

**Follows.** The product record requires the documentation to separate the
surface a program may depend on from the surface that exists for this
repository, and to state what the package cannot do yet, so that a reader who
wants a missing thing does not conclude that they failed to find it.[^F323C]
This finding is one instance of that gap and the explanation quadrant of the
documentation plan holds the page that states the line.[^F322E]

[^F322A]: Project orientation, the key capabilities and the architectural foundation. `README.md`
[^F322B]: Project instructions, the levels of detail. `CLAUDE.md`
[^F322C]: The pyramid of the core crate. `crates/cachette-core/src/pyramid.rs`
[^F322D]: Findings register, FND-259, in this document.
[^F322E]: Backlog item 0308, the documentation plan. `docs/backlog/refined/0308-the-documentation-plan.md`
[^F323A]: The agent-facing protocol server of the control plane. `python/cachette/agent/server.py`
[^F323B]: The project manifest. `pyproject.toml`
[^F323C]: Product requirement record 0021, what this does not do. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^F323D]: Product requirement record 0019, an agent can ask the running engine what it holds. `docs/product/shaped/prd-0019-an-agent-can-ask-the-running-engine-what-it-holds.md`

### FND-324 — The strict mode of the site builder does not see the fallback to the type stub

**Believed.** A research report measured that a documentation build with module
inspection turned off falls back to the type stub, loses the method prose, and
reports no error.[^F324A] A reader of that report can conclude that the strict
mode of the builder closes the hole, because a strict build ends on a warning.

**True.** It does not. The documentation job now runs the builder in strict
mode. With module inspection off, that build reported no issue, exited zero, and
46 of the 57 member summaries that the import finds were not on any page. The
site looked complete. Every signature was there and the prose was gone.

**Evidence.** The probe recipe breaks the job in the two ways the record names
and requires it to fail each time.[^F324B] The first case takes the compiled
module out of the environment. The second case builds the broken configuration
that turns inspection off.[^F324C]

```
just docs-probe
```

**Follows.** No setting of the builder is the guard. The guard must compare the
built site against the module that the import creates, and the documentation job
runs that comparison after the site build.[^F324D] The expected text is derived
from the module on every run, so the check holds no copy of any docstring.

### FND-325 — Every public member of the compiled module already carries prose

**Believed.** The record on the provenance of the documentation prose says that
a member with no Rust doc comment publishes with no prose and that nothing
fails.[^F325A] The priority index says the reference publishes empty prose until
the doc comments exist.[^F325B] Both read as though members with no prose exist
in numbers.

**True.** None does. The import of the compiled module finds 57 public members
and every one of them carries a docstring. The count covers the module level
names and the members that each class declares itself.

**One thing the count does not cover is the constructor, and it has no prose.**
The binding library does not copy the doc comment of a constructor onto the
Python object, so the module carries the standard interpreter sentence there and
nothing else. The reference therefore says what a class is and never says how to
build one. The prose for that belongs in the doc comment of the class.

**Evidence.**

```
uv run python scripts/check_reference.py --import-only
```

**Follows.** The item that repairs the doc comments keeps its second half and
loses its first.[^F325C] The gap is the audience of the prose and not the
absence of it: a doc comment written for a contributor to the core under-serves
the Python developer that the product record names, and no check can see
that.[^F325A] [^F319C] The check that the documentation job runs reports every
member with no prose, without failing, so the first half stays covered if a new
member arrives with none.

### FND-331 — Three exception classes the package exports and documents, and nothing raises

**Believed.** The compiled module declares nine exception classes and exports
every one of them.[^F320C] The package re-exports all nine, and a test asserts
that each one is a subclass of the root class.[^F331B] Each carried a docstring
that reads as a statement of when the engine raises it. A reader takes the list
for the set of failures the engine reports.

**True: six of the nine are raised, and three are not.** No call site in this
repository raises `SelectorError`, `DeterminismError` or `EnginePanic`. Each of
the three appears twice in the tree: once where the macro declares it, and once
where the module registers it. Nothing else names any of them, in the core
crate, in the bindings crate, in the package or in the tests.

**The panic class is the sharper case, because a panic does happen and does not
produce it.** The bindings crate installs no panic hook. The binding library
catches a panic at the boundary and raises its own `pyo3_runtime.PanicException`,
which is not a subclass of the root class. A caller that catches the root class
in order to survive a panic does not survive one.

**Evidence.**

```
grep -rn "SelectorError\|DeterminismError\|EnginePanic" crates python tests scripts
```

The search returns the declaration, the registration, the package re-export and
the membership test. It returns no raise site.

**Follows.** This is the shape the project already names: a capability that
nothing invokes, declared and documented as if it were fact.[^F331C] The product
record requires a reader to learn which error a
call raises, and to learn what the package cannot do yet rather than conclude
that they failed to find it.[^F331E] The three docstrings now say plainly that
nothing raises them. Either something raises each class, or the class goes. Do
not delete one while its docstring is the only place that says it is inert.

### FND-332 — A refused faction names the project ceiling, and the engine applied the faction count of the world

**Believed.** The refusal that a verb returns for a faction it will not accept
reads `the faction 3 is at or above the ceiling 63`. A caller reads the number
in the message as the bound that was applied.

**True: the world applies its own faction count, and the message names a
different number.** The world checks the faction against the faction count that
its constructor took, and it raises the arena's own ceiling error to report
that.[^F332A] The arena formats that error with the project-wide ceiling, which
is a constant of the storage.[^F332B] The two bounds are unrelated, and the
message names the one that was not applied.

**Evidence.** A world built with a faction count of one refuses the faction
three, and says that three is at or above 63.

```
uv run python -c "import cachette; cachette.World(8, 8, faction_count=1).spawn_soldiers([(2, 2)], 3)"
```

The same shape reaches the settlement verbs, which raise the same error from the
settlement arena.

**Follows.** The message misdirects the reader that the product record serves,
because that reader learns which error a call raises from the message and from
the reference alone.[^F331E] A caller who reads it raises the faction count of
the world and meets the same refusal. This work documents the behaviour and
does not repair the message, because the repair changes the text a verb returns
and that is engine work rather than documentation work.

[^F324A]: Research report 19, the documentation toolchain, section 4.2. `docs/research/reports/19-documentation-toolchain.md`
[^F324B]: The documentation probe. `scripts/docs-probe.sh`
[^F324C]: The broken site configuration. `tests/fixtures/docs-inspection-off/mkdocs.yml`
[^F324D]: The reference check. `scripts/check_reference.py`
[^F325A]: ADR-0107, the Python reference is generated from the compiled module, the consequences. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^F325B]: Backlog priority index, the row for item 0310. `docs/backlog/PRIORITY.md`
[^F325C]: Backlog item 0310, write the Rust doc comments for the Python reader. `docs/backlog/complete/0310-write-the-rust-doc-comments-for-the-python-reader.md`
[^F331B]: The public interface test of the package. `tests/test_public_api.py`
[^F331C]: Recurring Defect Shapes, shape 3, inert code that nothing invokes. `.claude/rules/recurring-defects.md`
[^F331E]: Product requirement record 0021, what the person cannot do today. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^F332A]: The world of the core crate, the soldier spawn. `crates/cachette-core/src/world.rs`
[^F332B]: The soldier arena of the core crate. `crates/cachette-core/src/soldier.rs`

### FND-333 — Nine error classes named a module that was not the module they belong to, and the reference dropped every one

**Believed.** The reference publishes each public member that the import of the
compiled module finds. The nine error classes are public, they are named in the
export list of the module, and each one carries prose, so the reference carries
all nine.

**True.** It carried none of them. Every error class reported the bare name
`_core` as its module, and every other member reported the dotted path
`cachette._core`. The documentation builder reads the module of a member and
skips one that names a different module, because such a member is an import
from somewhere else rather than a member of the module it documents. The nine
classes left the page. The page rendered fifteen members and looked complete.

**The macro writes the module name, and the binding library writes the dotted
path.** The declaration of an error takes a module name as its first argument,
and it puts that argument into the class. A class that the binding library
builds takes the dotted path instead. So one module described itself in two
ways, and the two disagreed.

**Evidence.** The import reports the module of each member.

```
uv run python -c "import cachette._core as m; print(m.World.__module__, m.CachetteError.__module__)"
```

The built site names each member it rendered.

```
grep -o 'id="cachette._core\.[A-Za-z_]*"' target/site/reference/index.html | sort -u | wc -l
```

That count was fifteen before the repair and twenty-four after it. The
documentation job now reports that every one of the fifty-nine summaries
reached the site.

**Follows.** This is one fact held in two places, with nothing that fails when
the copies disagree.[^F320E] The module registration now sets the dotted path on
each error class before it adds it, so one statement of the path reaches every
member.

**A check that derived its expectation is what found this.** The check asks the
imported module which members carry prose, and it compares that set against the
built site.[^F324D] A written list of the members to document would have omitted
the nine classes as well, and it would have agreed with the site for ever.


[^F334A]: Findings register, FND-317, in this document.
[^F334B]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^F335A]: Findings register, FND-316, in this document.
[^F317E]: Findings register, FND-334, in this document.

### FND-336 — The site address drives the page for an unknown address, and without it that page pointed at the root of the domain

**Believed.** The site address of the configuration drives the canonical link
of each page and the sitemap. A build that states no address loses those two
things and nothing else. The item that built the site left the address unset
under a blocker, and it stated that only the publishing step depends on
it.[^F336A]

**True.** The builder also writes the address into the page it generates for an
address the host does not hold. That page cannot use a relative link, because
the host serves it for any address. The builder therefore writes an absolute
path into every link and every asset of it, and it derives that path from the
site address. With no address, it wrote the root of the domain.

**This site does not answer at the root.** The repository does not carry the
name of its owner, so the site is a project site and it answers on a path below
the host name of the owner. A page that points at the root points outside the
site, and the host answers with its own page rather than with the one this
build made.

**Evidence.** Two builds of one tree, with the address set and with it removed.
The two pages for an unknown address hold different links.

```
grep -o '\(href\|src\)="/[^"]*"' target/site/404.html | sort -u
```

With the address set, each link starts with the repository name. With the
address removed, each link starts at the root. Every other page of both builds
holds relative links only, and the two builds are identical in every other
respect. The site build reported no issue in either case, and the reference
check passed in both.

**Follows.** A site that states no address is not a site that loses two
features. It is a site with one broken page, and no check sees it. State the
address in the configuration as soon as the host is known.

### FND-337 — A blocker row asked for a fact and for an action, and only the fact was information

**Believed.** The blocker on the documentation site closed when the project
owner named the address of the site and confirmed that the hosting was turned
on. The row said so in its own closing condition.[^F337A]

**True.** The two halves are not the same kind of thing. The address is
information the project did not have, and the register holds exactly that. The
hosting source setting is an action that one person takes in the settings of
the host. No contributor can take it, and no contributor can read it from the
tree either, but a register of blockers is not where an action waits.

**The register states the distinction itself.** It says that a blocker needs
information the project does not have, and it compares the decisions register,
which holds choices that need judgement.[^F337A] An action belongs in the
backlog, where the priority index states when it is taken.[^F282D]

**Evidence.** The owner answered the first half on 3 September 2026 and the
second half stayed open, because he takes it in a browser rather than in this
repository. The row would have stayed open with every fact in the tree.

**Follows.** Write a closing condition that names a fact. When a row needs an
action as well, open the backlog item for the action in the same change, and
let the row close on the fact. A row that waits for an action reads as missing
information, and it stops work that has everything it needs.

[^F336A]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/complete/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
[^F337A]: Blockers register, BLK-035 and the opening statement. `docs/BLOCKERS.md`
