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

## The sequence

The sprint number is a position in this list. It is not a date.

| Sprint | Goal | Items |
|---|---|---|
| 1 | The world exists, is indexed, and the citations cannot rot | 0012, 0015, 0016, 0017, 0018 |
| 2 | Entities exist in storage and reach the ordering primitives | 0007, 0013, 0014, 0019, 0020 |
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

## References

[^1]: Backlog guide, and the definition of done. `docs/backlog/README.md`
[^2]: PRD-0002, a developer watches the world run. `docs/product/REGISTRY.md`
[^3]: Blockers register. `docs/BLOCKERS.md`
[^4]: Findings register. `docs/FINDINGS.md`
