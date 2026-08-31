# Sprints (Register)

This document is a **register**. It holds the sequence of sprints toward the
current goal, and the outcome of each ceremony.

The backlog holds the work. This document holds the order and the outcome.
When the two disagree, the backlog is right, because an item carries its own
status.[^1]

## The goal

**A developer watches the world run.** A window shows the hex world. Entities
move on it under random behaviour. The engine that drives the window is the
real engine, at the real thread count, under every accepted record.

The product record states the need and the bound.[^2]

## The ceremonies

Four ceremonies run for each sprint. Each one produces a written outcome in
this document. A ceremony with no written outcome did not happen.

**Planning.** Take the next sprint from the sequence below. Refine every item
in it, which means completing the architectural impact review.[^1] An item
that cannot answer the review stays proposed, and the sprint takes the next
one. State the sprint goal as one checkable sentence.

**Standup.** Run at the start of each working session inside a sprint. Name
what is done, what is next, and what is blocked. A blocked item opens a
blocker row rather than waiting.[^3]

**Review.** Run when the sprint goal is met or abandoned. Demonstrate the
goal against its checkable sentence. State what shipped and what did not.

**Record review.** Run before a record moves from `Draft` to `Accepted`. An
agent that did not write the record reads it against the scope rule and
against the code that implements it. The review states what it tried to
reject. A review that lists no attempted objection did not happen, and the
record stays a draft. The registry says who holds review rights and why the
delegated form needs this.[^5]

**Code review.** Run before work merges. An agent that did not write the code
reads the diff against every governing record, decision by decision, and
against the recurring-defect shapes. It reports findings; it does not fix
them. The author decides what to do with each one and says so.

**A review writes a file.** Both reviews deliver a file under the review
directory, and the file is the deliverable.[^6] A reviewer that reports only
by message delivers nothing when the message does not arrive, and a missing
message looks exactly like a message that says nothing is wrong. This
happened on the first attempt: four reviewers ran, none delivered, and the
work went forward on one reading while the log said it had two.

**Retrospective.** Run after the review. Record what to change in the next
sprint. A correction that the project believed goes to the findings
register, not here.[^4]

## The balance rule

**Every sprint carries record work and code work.** A sprint of only records
builds nothing. A sprint of only code accumulates undocumented decisions, and
an undocumented decision becomes an assumption.[^1]

**A record is written when the work needs it, not before.** The registry
reserves a number. A row without a file is not a debt. Write the file when an
item is about to implement the claim, so the record states what the code does
and the two cannot drift.

**A record is audited before the work that depends on it.** A draft record
that governs a sprint is either accepted in that sprint or its claim is
treated as provisional and cited as a draft.

**Correcting a record is cheap until something depends on it.** A draft is
edited. A record accepted in the same sprint, with nothing built on it, is
repaired in place and the commit says so.[^7] Only a record with dependents
needs a supersession. A process that makes a small correction expensive gets
expensive corrections, or none.

**Work runs in parallel where the files do not overlap.** Two agents that
write to one file produce a conflict, and resolving it costs more than the
parallelism saved. Split by directory: the records, the core, and the
bindings are three surfaces. When two items must touch one file, sequence
them and say so in the plan.

**Never stage by wildcard while parallel work runs.** A command that adds
everything captures whatever another agent has half-written, and the commit
message then describes work the commit does not contain. Stage the paths the
commit is about, by name. This has happened once: a record that an agent was
still writing entered a commit about an unrelated repair, and the commit had
to be rebuilt.

The parallel surfaces are also the review surfaces. An agent that writes to
one surface reviews nothing on it.

## The sequence

The sprint number is a position in this list. It is not a date.

| Sprint | Goal | Items |
|---|---|---|
| 1 | The world exists, is indexed, and the citations cannot rot | 0012, 0015, 0016, 0017, 0018 |
| 2 | Entities exist in storage and reach the ordering primitives | 0029, 0013, 0014, 0019, 0020 |
| 3 | Entities choose and move, deterministically | 0021, 0022, 0023 |
| 4 | The world is visible | 0024, 0025, 0026 |
| 5 | The demonstration is honest | 0027, 0028 |

## Sprint log

### Sprint 1 — planning

**Goal.** The core builds a rhombus hex world, indexes a tile by raw axial
coordinates with no conversion, and a gate fails when a citation names a
record decision that does not exist.

**Committed.** Items 0012, 0015, 0016, 0017 and 0018.

**Why these.** BLK-013 and BLK-014 are answered, so the values they governed
can be recorded and the rows they blocked can be written. Nothing in the
renderer can be built before a tile has an index. Item 0012 comes first
because every later sprint adds citations, and the gate that protects them
costs less than the sweep that repairs them.[^4]

**Balance.** Record work is 0015, 0016 and 0017. Code work is 0012 and 0018.

### Sprint 1 — review

**Goal met.** The core builds a rhombus world, indexes a tile by a raw axial
pair with no conversion, and the citation gate fails when a citation names a
decision that does not exist.

All five committed items shipped. Nothing was abandoned and nothing carried
over.

| Item | Outcome |
|---|---|
| 0012 | The citation check runs in the gates and has a proven failure mode |
| 0015 | PRD-0002 states the need in nine checkable statements |
| 0016 | BLK-013 and BLK-014 closed; the scale constants hold both values |
| 0017 | ADR-0017 states four decisions and the row is `Draft` |
| 0018 | The world has a grid, and the geometry has a real caller |

**Demonstrated.** `just check` and `just test-slow` both exit 0. The
geometry tests were checked against three mutations, each of which failed
four tests.

**Not shipped.** ADR-0017 is a draft. An author may not accept their own
record, so it stays a draft until a reviewer reads it. The code that
implements it cites it as a draft, which is correct.

### Sprint 1 — retrospective

**What worked.** Writing the gate first paid inside the same sprint. Item
0018 added 23 citations, and the check read every one of them without anyone
remembering to look.

**What to change.** Two of the five items found work the plan did not name.
Item 0017 found a fourth decision, and item 0018 found that the world
constructor needed to return a result. Both were small. Neither was a
surprise in hindsight, and both would have been found by asking one question
at refinement: what can the caller get wrong? Sprint 2 planning asks it.

**What not to change.** The mutation check on the new tests cost little and
found that the tests were real. Sprint 2 repeats it. It is not a ceremony; it
is the difference between a test and a decoration.

**A correction went to the register, not here.** FND-042 records that a
registry row stated a claim while the blocker that governed it was open. That
is precedent, and precedent does not live in a sprint log.

## References

[^1]: Backlog guide, and the definition of done. `docs/backlog/README.md`
[^2]: PRD-0002, a developer watches the world run. `docs/product/REGISTRY.md`
[^3]: Blockers register. `docs/BLOCKERS.md`
[^4]: Findings register. `docs/FINDINGS.md`
[^5]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^6]: Reviews index. `docs/reviews/README.md`
[^7]: ADR Registry, the retcon window. `docs/adrs/REGISTRY.md`

### Sprint 2 — planning

**Goal.** A soldier exists in storage, a tile answers which soldiers stand on
it, and both the slot reduction and the key vector sort have a real caller.

**Committed.** Items 0029, 0013, 0014, 0019 and 0020.

**Why these.** Nothing can move until something holds the movers, and nothing
can be admitted to a tile until the tile can count what is already there. The
two ordering mechanisms come first inside the sprint, because item 0020 sorts
through the interface that item 0014 provides, and a mechanism written to fit
one caller is written twice.

**Balance.** Record work is 0029. Code work is 0013, 0014, 0019 and 0020.

**One item was returned to `proposed`.** Item 0007 was refined and would have
written ten storage records in one pass. It is not badly refined; it is too
large, and the size is the defect. Nine of those ten records have no code
that needs them, and a record written ahead of its work states what the
author expects rather than what the code does. Item 0029 takes the three rows
this sprint reaches. The rest arrive the same way.

**The retrospective's question is in each item.** Every refined item now
answers what the caller can get wrong, in its own section. Sprint 1 found
that answer late twice.

**Order inside the sprint.** 0029 first, because it writes the records that
0019 and 0020 implement. Then 0013 and 0014, which are independent of each
other. Then 0019. Then 0020, which needs 0014 and 0019 both.

### Sprint 2 — review

**Goal met.** A soldier does not yet exist in storage, so the sprint goal was
met in part. The records that govern the soldier are accepted, and both
ordering mechanisms have real callers. Items 0019 and 0020 carry over to
sprint 3.

| Item | Outcome |
|---|---|
| 0029 | ADR-0012, ADR-0014 and ADR-0018 accepted after two reviews |
| 0013 | The slot reduction, with the world step as its caller |
| 0014 | The key vector sort, which accepts no function of any kind |
| 0019 | Carried to sprint 3 |
| 0020 | Carried to sprint 3 |

**Demonstrated.** Every gate green, locally and in continuous integration.
Three defects were found that no gate would have caught: an identity that
could not represent the first entity the engine allocates, a set of property
seeds that were never read, and two records making incompatible claims about
one array.

**Not shipped.** The two code items. The sprint committed five items and
delivered three, which is the honest count.

### Sprint 2 — retrospective

**What worked, and it is the only thing that found the worst defect.** Two
reviews read the same four records and disagreed with each other. The second
found the unrepresentable identity, which the approver and the first reviewer
both missed. A single reviewer who agrees with the author is
indistinguishable from no reviewer.

**What went wrong, three times, all the same shape.** Work was judged against
a tree that something else was still writing. A staging wildcard captured a
half-written record. A gate was run while an agent was mid-edit and its
failure was nearly reported as real. Records were accepted while a second
review was still reading them.

The rule that follows is not about care. It is about the read: **establish
that the tree is settled before judging it.** Sprint 3 dispatches agents to
non-overlapping surfaces, as sprint 2 did, and waits for each to report
before running any gate over its work.

**What the process got wrong about itself.** The remedy for the premature
acceptance was heavier than the defect: the records were reverted to `Draft`
and accepted again, recording a history that did not happen. The registry now
holds a retcon window, and a draft is simply edited. A process that punishes a
cheap correction produces expensive corrections, or none.

**Estimation.** Five items committed, three delivered. The two that slipped
were the two that depended on everything else in the sprint. Sprint 3 commits
to those two and nothing else.

### Sprint 3 — planning

**Goal.** A soldier exists in the arena, a tile answers which soldiers stand
on it, and neither answer depends on the thread count.

**Committed.** Items 0019 and 0020. Nothing else.

**Why only two.** Sprint 2 committed five and delivered three. These two are
the two that slipped, they are the largest remaining pieces of the goal, and
0020 depends on 0019. Adding a third item would add a third surface to an
already serial pair.

**Balance.** Both items are code. The record work for them is done and
accepted, which is the point of having done it first. A decision that the
work finds is a deliverable of the item, and both items say so.

**ADR-0014's retcon window closes here.** Item 0019 implements the identity,
so the record acquires its first dependent. After this sprint an amendment to
it needs a supersession.

**Order.** 0019 first, alone. Then 0020, which needs the arena and the sort.
The two do not run in parallel, because both write to the entity storage.

### Sprint 3 — review

**Goal met.** A soldier exists in the arena, a tile answers which soldiers
stand on it, and neither answer depends on the thread count.

| Item | Outcome |
|---|---|
| 0019 | The soldier arena, plus the two repairs the reviews demanded |
| 0020 | The bridge, rebuilt at the barrier from the shared sort |

**Demonstrated.** Every gate green. The review of item 0019 found a defect
that the test suite was demonstrating while passing: three faction ceilings
existed and one was enforced, and the thread-count test spawned soldiers of
factions its world did not hold.

**Not shipped.** Nothing. The sprint committed two items and delivered two,
which is what committing to two rather than five buys.

### Sprint 3 — retrospective

**What worked.** Committing to two items. Sprint 2 committed five, delivered
three, and the two that slipped were the two that mattered. Sprint 3 took
those two alone and finished both.

**What worked twice.** The mutation check. It found that a repair had no
test, on the day the repair was written, by reverting it and watching nothing
fail. A fix with no test is indistinguishable from no fix once the author
moves on.

**What went wrong, and it is a measurement problem.** Two attempts to count
mutation kills were both unsound. The first ran one test target and missed
the unit tests inside the module, which nearly produced a report that a rule
was untested. The second ran every target but let the runner stop at the
first failing binary, which undercounted by a factor of five. Both numbers
were published before they were checked.

**A count is a claim, and a claim needs the same proof as any other.** The
command is now written down. `cargo test -p cachette-core --no-fail-fast`.

**What to keep.** Serial items when they share a surface. Items 0019 and 0020
both write to the entity storage, so they ran one after the other, and no
conflict cost anything. The one cost of that discipline was waiting, and the
waiting was cheaper than a merge.

### Sprint 4 — planning

**Goal.** One command opens a window, the hex world appears in it, and
entities move on it under random behaviour while a developer watches.

**Committed.** Items 0024, 0022, 0025 and 0026.

**Why four, when sprint 3 committed two.** Three of them are small and one is
the goal. Item 0024 is a record. Item 0022 is a behaviour that the arena and
the keyed generator already support. Item 0025 is the paint routine. Item
0026 is the loop that joins them. They are serial, they share the same new
crate, and none of them is the size of the arena.

**Two decisions the project owner made at planning.**

The viewer opens a window through a software framebuffer, with one small
dependency. The alternative was a graphics-backed surface, which brings a
large dependency tree, an asynchronous event loop that would shape the
demonstration binary, and more continuous integration surface. The
demonstration draws a world small enough to watch, which the product record
already bounds, so the scale the alternative buys is not needed yet.

The viewer reads the world on the stepping thread: step, then draw. The
alternative was a published frame with the engine on its own thread, which
honours the product record's "never slows the engine" literally and would
exercise the snapshot row. That row has no record yet, and writing one to
serve a demonstration is the wrong order. The consequence is stated in the
viewer record rather than left implicit.

**Balance.** Record work is 0024. Code work is 0022, 0025 and 0026.

**Order.** 0024 first, because it states what the viewer may not do. Then
0022, which is engine work and touches no viewer code. Then 0025 and 0026,
which share the new crate and run serially.
