---
id: 0063
title: Assign a unit to a position
status: proposed
created: 2026-08-31
implements: [ADR-0007 D1, ADR-0007 D2, ADR-0007 D3, ADR-0004 D1, ADR-0004 D4]
changes: []
creates: []
serves: [PRD-0017, PRD-0011]
blocked-by: [0059, 0062]
---

## Why

Item 0062 gives a site positions and nobody to fill them. This item fills
them, and it is the change that lets a place respond to what it lacks.

It is also the item that closes a gap the shaping opened. Point two of item
0050 records that job assignment belonged to nobody: PRD-0011 deferred it to
unit behaviour, and PRD-0009 is unit behaviour and refuses it, because
assigning a job is a decision made for a place and it persists for many
ticks.[^1] **PRD-0017 now owns it, and this item implements PRD-0017.**

## What the work does

1. A site gathers the units that live in it and sorts them by a key vector.
2. It sorts its positions by a key vector.
3. It scans both lists together and pairs them in order.
4. A property of the unit limits which positions it can take, and the limit is
   the first field of the applicant key.
5. The assignment runs at an interval, not on every tick, and a world in which
   nothing changed does almost no work.

## Impact review

**Governed by.** ADR-0007 D1 requires the sort to take a key vector rather
than a comparison function, D2 requires the last key field to be a stable
identifier, and D3 forbids calling content code from inside a sort.[^2] ADR-0004
D1 and D4 fix the order of the scan and require a stable key.[^3] The research
reaches the same conclusion from a different direction: a policy that takes a
comparator can be intransitive, and an intransitive comparator makes the output
depend on the sort algorithm, which no tie-break repairs.[^4]

**Blockers.** BLK-007 governs the cost figures this item would state, so it
states none.
BLK-005 gives the settlement count.[^5]

**Serves.** PRD-0017, and PRD-0011 for the statement that a unit's job changes
during its life.

**Conflict surface.** `crates/cachette-core/src/assign.rs` is new.
`crates/cachette-core/src/site.rs`, `crates/cachette-core/src/soldier.rs` at a
job column and a position back-reference, and
`crates/cachette-core/src/world.rs` at the step. **It cannot run beside item
0065**, which reads the job to drive behaviour, and it shares `soldier.rs` with
items 0056, 0057, 0059 and 0060.

## What is missing before this is refined

**The registry row.** This work states a constraint that no reserved row
holds. Row 0065 holds the membership claim and item 0062 writes it; **the
assignment rule is a second claim and it needs its own record**: an assignment
runs as one set-valued sort and scan for each site, at an interval, and never
as a search over pairs. All three conditions of the scope rule hold.[^6] A
contributor could reasonably score every unit against every position, and
PRD-0017 rejects that by name. Changing it later is a rewrite. The reasoning
is not visible in a scan.

**Allocate the row in the registry before writing the record.**[^7] This item
does not choose a number.

**One thing the record should carry, because it is the part a reader will not
believe.** When the value of putting a unit in a position is the product of one
number about the unit and one number about the position, sorting both sides and
pairing them in order is **exactly optimal and not an approximation**. The
research states the result and names it.[^4] That is the reason the cheap
method is also the right one, and it is reasoning the code cannot hold, so it
belongs in a record. It also states a duty on the content: **design the values
so that they factorise**, or the guarantee does not apply.

**What limits which positions a unit can take.** PRD-0017 requires a property
of the unit to limit it and does not say which. No column exists. Whether that
property is one field or a family index is a decision this item must take, and
it decides the width of the first sort key.

## Done when

- A site pairs its residents with its positions by one sort and one scan, and
  no part of the work scores a pair.
- The sort goes through the key vector interface, never through a comparison
  function, and no content code runs inside it.
- A unit that lacks the required property never takes the position, and a test
  asserts the refusal at the boundary.
- A position can go unfilled, and the site shows a consequence a watcher can
  read.
- The assignment runs at an interval, and a world in which nothing changed
  does almost no work. A test asserts the second half rather than asserting a
  duration.[^8]
- A unit's job changes during its life when what the site needs changes, and a
  test asserts a change.
- A property test asserts that the assignment is identical at 1, 2 and 12
  threads, including when two units tie on every key but the identifier.
- The fixture holds more applicants than positions and more positions than
  applicants, and the commit body says how that was checked.[^9]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
[^2]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: Individual agency and occupations. `docs/research/reports/16-individual-agency-and-occupations.md`
[^5]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^6]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^7]: ADR Registry. `docs/adrs/REGISTRY.md`
[^8]: Testing Rules, section 3. `.claude/rules/testing.md`
[^9]: Findings register, FND-051. `docs/FINDINGS.md`
