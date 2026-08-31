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
