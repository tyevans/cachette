---
id: 0390
title: Expose the fallen log to the control plane
status: complete
created: 2026-09-03
implements: [ADR-0040 D1, ADR-0044 D1, ADR-0085 D1, ADR-0085 D3, ADR-0121 D4]
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

The engine resolves a meeting between two factions and writes one event for
each unit that fell. The event names the tick, the unit, the tile, the faction
and the type. **No binding read it**, so a caller in the control plane watched
its faction population fall and could not see where or to what.

A fight that nobody can read is a fight that nobody can repair. The research
report names the event as one of the five things a contest needs, and it names
reading it as the reason.[^1]

## Impact review

**Governed by.** Each record and decision below binds this work.

- ADR-0040 D1. Python is a control plane. The read returns columns for the
  whole log in one crossing, so no caller loops over the dead.
- ADR-0044 D1. The read copies each column, and the doc comment says so at the
  call site.
- ADR-0085 D1. The unit column carries the whole identity, never a slot index.
- ADR-0085 D3. Every identity in the column is dead, and the engine refuses a
  dead identity rather than answer for the next occupant of the slot.
- ADR-0121 D4. The log holds the units of one frame, in ascending slot order,
  and the pass empties it at its start. The doc comment states both.
- ADR-0004 D1. The order of the entries is the ascending slot order the step
  ended the units in, so it does not follow a thread.
- ADR-0001 D4. A read moves no state, so no golden state hash moves.
- ADR-0002 D1. Every column is an integer array. No floating point value
  crosses the boundary.
- ADR-0006 D1. The event is plain data with declared padding. The padding is
  not a field, so no column carries it.
- ADR-0046 D1. The read raises a typed error.
- ADR-0107 D1. The doc comment is the published reference, so it states every
  column, its element type, its order and its lifetime.
- ADR-0041 D1. The core crate keeps no binding dependency. This work adds no
  line to the core crate.
- DEC-060. The read returns one column for each field, keyed by the field
  name, so no caller holds a byte offset or a field order.

**Changes.** None. No record is contradicted and none is superseded.

**Creates.** None. The four logs the bindings expose already set the shape, so
a fifth of the same shape states no new constraint and needs no record.[^3]

**Blockers.** None. No value here waits on an unanswered question.

**Precedent.** FND-148 records that a test above two checks demonstrates a
behaviour and covers neither. This work met the same shape one layer out, and
FND-443 records it.

**Whether this is a row of item 0319.** It is not. Item 0319 holds three logs
that the engine writes and no binding reads, and each of the three needs its
own columns and its own fixture. This item is a fourth log with the same shape,
and it is finished on its own. Item 0319 stays open and unchanged.

## Done when

- A Python caller reads the fallen log as columns, in one crossing.
- The columns match the shape and the naming of the logs already exposed.
- The doc comment states every column, its element type, its unit, the order of
  the entries, the lifetime of the log and the error it raises.
- A test starts at the Python boundary: it builds a world, puts two factions
  beside each other, steps, and reads the log.
- A step with no fight gives an empty log rather than a stale one, and a test
  holds it.
- The two determinism tests pass at 1, 2 and 12 threads, and no golden state
  hash moves.
- Every gate runs green.

## Outcome

**Done as planned.** The bindings gained `fell_log_columns` and `fell_count`,
which follow `gather_log_columns` and `gather_count` field for field. The
columns are `tick`, `unit`, `tile`, `faction` and `unit_type`. Six tests at the
Python boundary hold the behaviour, and four defects were put back to measure
them.

**The last-step lifetime stayed as it is, and the doc comment states it.** A
queue inside the engine is a decision with a bound, a drop rule and a record,
and it buys a caller nothing that a recorder in the control plane cannot buy
outside it. DEC-224 holds the options with a recommendation, and item 0432
holds the work.

**No golden state hash moved**, because a read moves no state.

**Registers.** FND-443 opened. DEC-224 opened. No blocker opened or closed. No
record was written, and the registry is unchanged. The review holds the
detail.[^2]

## References

[^1]: Research report 21, what a god needs from this engine, section 4.5. `docs/research/reports/21-what-a-god-needs.md`
[^2]: Review of backlog item 0390. `docs/reviews/0390-the-fallen-log.md`
[^3]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
