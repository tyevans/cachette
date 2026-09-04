---
id: 0345
title: Resolve a meeting between two factions
status: complete
created: 2026-09-03
implements: [ADR-0121 D1, ADR-0121 D2, ADR-0121 D3, ADR-0121 D4, ADR-0122 D1, ADR-0122 D2, ADR-0122 D3, ADR-0123 D1, ADR-0123 D2, ADR-0123 D3, ADR-0123 D4, ADR-0123 D5]
changes: []
creates: [ADR-0121, ADR-0122, ADR-0123]
serves: [PRD-0030]
blocked-by: []
---

## Why

Two factions can stand beside each other and nothing happens. The engine holds
no contest, and a downstream game names attacking as one of the six things its
players must do.

The sketch is a table over unit types rather than a fight for each pair of
units. A type whose effect does not exceed the defender's threshold contributes
exactly zero, so a sum of zeroes stays zero and no number of weak attackers ever
reaches a strong defender. That is the project owner's acceptance test, and the
threshold satisfies it structurally rather than by tuning a constant.[^1]

**Casualties are whole units served to a keyed subset.** The project already
holds the rule: a cohort serves whole rations to a keyed subset, never an equal
share to everybody, and the subset is the ordinals rotated by a keyed
offset.[^2] The arithmetic module already floors a share and leaves the
remainder to the caller. One keyed draw serves a whole group, and a draw for
each unit is what this reuses the rule to avoid.

## Impact review

**Governed by.** Each record is checked decision by decision.

- **ADR-0001 D3.** The pass runs a fixed amount of work for the world it is
  given. It holds no convergence test and no time budget.
- **ADR-0001 D4.** The type table, the type column and the fallen log all reach
  the whole-world state hash, because each decides what a later frame does.
- **ADR-0002 D1 and D3.** Every value is an exact integer or a fixed-point
  value. The harm is a 64-bit accumulator, and the floor and the remainder come
  from one value, so the two cannot disagree.
- **ADR-0002 D2.** Every arithmetic operation on simulated state goes through
  the arithmetic module. The pass calls the combine and the scale of that
  module.
- **ADR-0003 D1.** Every draw is keyed on the system, the frame, the tile and
  the draw index. The contest owns a system identifier of its own and shares it
  with no other pass.
- **ADR-0004 D1.** The marks apply in one ascending scan of the unit slots, and
  never in the order a thread finished.
- **ADR-0009 D1.** The threads do not own disjoint output ranges, because two
  tiles of two threads can hold units whose slots share one word. Each thread
  therefore takes its own plane, and the planes join by a bitwise union.
- **ADR-0023 D1.** The union of the planes and the sum of the harm are both
  commutative and associative, so both give one answer at any thread count.
- **ADR-0006 D1.** The event of a unit that fell is plain data, with an explicit
  layout, declared padding and no boolean field.
- **ADR-0018 D2 and D3.** The pass reads the derived unit structure, which the
  barrier of the frame rebuilt. It removes units, so the step rebuilds the
  structure again before anything else reads it.
- **ADR-0056 D3.** Admission enforces the tile capacity and reads the capacity
  rather than the faction. This is why contact is adjacency: a full tile could
  never be entered, so a rule that fired only on co-occupation would never fire
  against an army that packed itself.
- **ADR-0074.** A spawn may over-fill a tile. The fixtures use that to reach a
  crowd no admission would let walk onto one tile.
- **ADR-0091 D1.** The pass reads the six neighbours of each tile it resolves.
  That is a pass over tile pairs, and it is not the per-unit search this record
  forbids.
- **ADR-0106 D1 and D2.** The casualties reuse the ration rule: a whole count
  served to the ordinals of a group, rotated by a keyed offset.
- **ADR-0120 D1 to D4.** The pass reads the shared unit type table and the type
  column, and it holds no copy of a row.

**Changes.** None. No record is superseded.

**Creates.** ADR-0121, a meeting between two factions resolves at the tile.
ADR-0122, an attacker whose attack does not exceed the defender's armour
contributes exactly zero. ADR-0123, casualties are whole units served to a
keyed subset. The registry rows were allocated before the records were written.

**Blockers.** BLK-052 governed the granularity. The project owner decided it
ahead of the measurement, and the measurement then agreed with him. The
resolution is at the tile.

**Precedent.** FND-318 records that a draw for each unit makes the number that
is served vary around the number the store paid for. Recurring defect shape 4
governs the draw key, because a wrongly keyed draw is invisible to both
determinism tests.

## Done when

- Two factions that stand on one tile, or on neighbouring tiles, lose units.
- One tank ends four bowmen, and ten thousand bowmen end nothing of the tank.
- One keyed draw serves a whole group, and no draw is taken for one unit.
- A test exists for each field of the draw key.
- The two determinism tests pass at one, two and twelve threads.
- The whole check command runs green.

## Outcome

Done. The contest module resolves every meeting in the world and marks the
units that fell. The step runs it after the barrier of the frame and rebuilds
the derived unit structure after it.

**Contact is adjacency, and the brief said co-occupation.** A parallel
measurement found that admission reads the capacity of a tile rather than the
faction of the units on it, so an army at capacity can never be entered. A
resolution that needed two factions on one tile would therefore never fire
against exactly the case a fight is about. The alternative was a rule that lets
an enemy enter a full tile, which supersedes an accepted record and makes the
capacity mean nothing at the moment it matters. Adjacency was chosen, and the
record states it and the reasoning. The register holds the measurement.[^3]

**Every defect was put back and every one was caught.** Aggregating before the
threshold, dropping the frame from the key, dropping the tile from the key,
drawing for each unit instead of rotating, and resolving on co-occupation
alone. The review holds which test caught which.[^4]

Every stored golden hash moved, and a contested scenario joined the golden
suite and the thread-count suite.

**Registers.** DEC-144 closed on the tile. DEC-145 closed on the hard
threshold. DEC-180 opened, on whether the resolution runs on a schedule.
FND-400 and FND-401 opened. BLK-052 stays open and this item no longer waits on
it.

**Left undone.** No binding reads the fallen log, so the control plane sees its
population fall and cannot see where or to what. Item 0390 holds it. No posture
lets a unit refuse a fight; DEC-146 holds that question.

## References

[^1]: Research report 21, what a god needs from this engine, section 4.1. `docs/research/reports/21-what-a-god-needs.md`
[^2]: ADR-0106, a cohort serves whole rations to a keyed subset, decisions D1 and D2. `docs/adrs/draft/adr-0106-a-cohort-serves-whole-rations-to-a-keyed-subset.md`
[^3]: Findings register, FND-402. `docs/FINDINGS.md`
[^4]: Review of backlog item 0345, resolve a meeting. `docs/reviews/0345-resolve-a-meeting.md`
