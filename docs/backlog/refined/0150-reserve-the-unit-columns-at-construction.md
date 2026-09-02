---
id: 0150
title: Reserve the unit columns at construction
status: refined
created: 2026-09-01
implements: [ADR-0084 D1, ADR-0084 D2, ADR-0084 D3]
changes: []
creates: [ADR-0084]
serves: [PRD-0012]
blocked-by: []
---

## Why

The accepted product record for a founding states that the storage the world
reserves is sized for the target population, that it does not change during a
run, and that a run does not stop to grow.[^1]

The engine does the opposite. The unit arena opens as many slots as the slot
index holds, its own comment says that the limit is the range of the index and
not a budget, and it reserves no memory. Each spawn appends one entry to each
of its ten columns, so the storage grows with the population under a running
simulation. A driver founded 120 people through the public interface and read
the capacity back as the range of the index.[^2]

A record the code contradicts is worse than no record, because it lies.[^3]
The project owner closed the choice and the code changes to match the
record.[^4]

## Impact review

**Governed by.** Five records govern this work.

ADR-0012 D3 places every unit in a generational arena. The arena is the
storage this item reserves, so the reservation belongs to it and to nothing
else.

ADR-0014 D1 states that an identity is a slot index and a generation, and that
the arena never compacts the slot index space. The slot count is therefore a
high water mark that never falls. A reservation is an upper bound on that high
water mark, so it bounds a quantity the arena already refuses to reduce.

ADR-0014 D6 reserves the generation zero for a slot that carries no identity.
A reserved slot is not an opened slot, so the reservation writes no generation
and mints no identity. The reservation changes the capacity of a column. It
never changes its length.

ADR-0066 D1 gives each of the four fixed shapes its own column set, and D2
states that a pass over a column never changes which units exist. The
reservation is a structural property of the column set and it belongs to the
arena that owns the columns.

ADR-0001 D4 requires that one binary gives one answer at any thread count. The
state hash of the arena covers the length of each column, the free queue and
the counts. It covers no capacity. A reservation therefore changes no hash,
and the golden file does not move.

**Changes.** No record changes. PRD-0012 stands as written, because the
decision makes the code match the record rather than the record match the
code.[^4]

**Creates.** ADR-0084. The row is allocated and the record is a deliverable of
this work, not a byproduct of it.

The three-condition test passes.[^5] A future contributor could reasonably
choose to grow the arena on demand, because that is what the code does today
and it is the default shape of a growable column. Choosing otherwise costs
more than changing it later, because a caller that relies on a spawn which
never refuses spreads through the tree, and the reallocation it permits
arrives inside a step at a moment nobody chose. The reasoning is not visible
in the artefact: a call that reserves a column says how much it reserves and
never says why the column must not grow past it.

**Blockers.** BLK-007 stays open. No measurement exists on the target
platform, so the record and the code state the shape of the cost and no
figure. The shape is that the reservation is paid once, at construction, and
that no later spawn pays it.

BLK-003 is resolved and gives the reservation its value. One million is the
whole population, and soldiers are a fraction of it rather than a million on
top of the civilians. The scale constants table holds the row.[^6] The work
cites that answer and invents no number of its own.

**Precedent.** FND-135 records this disagreement and states what follows from
it. Two of the three consequences are the work of this item. The third is
that a cost statement of a product record is a claim about the engine that
nobody checks, and this item does not close that; it answers one instance of
it.

**Product record.** PRD-0012.

### What refining had to answer

The proposed item named four questions. Each has an answer.

**Where the reservation lives.** The world settings name it. DEC-059 says so,
and one declaration site is what shape 1 of the recurring defect rule
asks for.[^7] The arena takes the value it is given and states no default of
its own, so no second site can disagree with the settings.

**Where the refusal surfaces.** It already exists. The arena returns a typed
refusal when it can open no further slot, and the founding wraps that refusal
in a variant of its own error. Nothing new is needed on the refusal path. What
was missing is a capacity that a run can reach, because a capacity of the
whole index range makes the refusal unreachable.

**What the state hash does.** Nothing. The hash covers the length of each
column and not its capacity, so the golden file does not move.

**Whether the other three arenas hold the same question.** The settlement
arena and the character arena hold it. Both carry a capacity that refuses and
neither reserves. The character arena also carries a tier ceiling that is
larger than its target, so it has a second question the unit arena does not:
whether the reservation is the target or the ceiling. DEC-059 asks about the
unit arena alone, so this item does that and opens a row for the other
two.[^8] The tile upgrade shape is sparse by decision and holds no arena.

## What fails if somebody changes it back

Three defects can be put back, and each has a test that must fail when it is.

- **Remove the reservation.** A test fills a world to its named capacity
  through the public interface and asserts that the address of every unit
  column is the address it held before the first spawn. A column that
  reallocates moves, so the test fails. A test that asserted a capacity
  instead of an address would stay green, because a capacity that a growing
  column reports is the capacity it happens to have reached.
- **Remove the refusal.** A test spawns one unit past the named capacity and
  asserts the typed refusal. Without the check the spawn succeeds and the
  test fails.
- **Let a copy of a world lose the reservation.** A derived clone of a column
  allocates for the length and not for the capacity, so a copied world grows
  where the original does not. A test copies a filled-to-nothing world and
  runs the address assertion against the copy.

Put each defect back and watch the test before claiming it covers the
case.[^9] The fixture must fill the world to its named capacity, because a
world that never approaches its reservation supplies no input that could fail
either assertion.[^10]

## Done when

- The world settings name the unit reservation, and no second site states a
  default for it.
- The reservation takes the target population from the answered blocker, and
  the code cites that answer rather than a number of its own.
- The unit columns and the free queue reserve that many entries when the
  world is built.
- A spawn past the reservation returns the typed refusal, through the world
  and through the founding.
- A founding that a refusal stops leaves nothing behind. No settlement stands
  and no person lives.
- Each of the three tests above has been proven able to fail, and the report
  names which test caught which defect.
- ADR-0084 is written, holds no figure and no file table, and sits at `Draft`
  in the registry.
- DEC-062 states the question that the settlement arena and the character
  arena hold.
- The two determinism tests pass at 1, 2 and 12 threads, and the golden file
  does not move.
- The whole gate command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^2]: Findings register, FND-135. `docs/FINDINGS.md`
[^3]: Definition of Done, section 3. `.claude/rules/definition-of-done.md`
[^4]: Decisions register, DEC-059. `docs/DECISIONS.md`
[^5]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^6]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^7]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: Decisions register, DEC-062. `docs/DECISIONS.md`
[^9]: Findings register, FND-051. `docs/FINDINGS.md`
[^10]: Testing Rules, section 2a. `.claude/rules/testing.md`
