---
id: 0451
title: Let Python read the four unread step logs
status: complete
created: 2026-09-03
implements: [ADR-0006 D1, ADR-0006 D2, ADR-0044 D1, ADR-0046 D1, ADR-0085 D1, ADR-0085 D3, ADR-0107 D2, ADR-0002 D1]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The engine writes five logs at each step. It records that a unit starved, that a
site could not pay its upkeep, that a site could not serve its cohorts, that a
soldier became a character, and that two factions spoke about a trade.

The trade log reached Python. **The other four reached nothing.** A god in the
downstream game cannot see its people starve, go short, be rationed or rise in
rank. A mechanic that a player cannot observe is one they cannot play.

This item is the four crossings. It changes no engine behaviour.

## Impact review

**Governed by.** ADR-0006 D1 makes each event plain data with declared padding.
D2 delivers events to Python at the frame barrier and never inside a step.
ADR-0044 D1 makes each call site declare what copies. ADR-0046 D1 puts every
refusal under one root class. ADR-0085 D1 and D3 make an identity opaque and
resolved by the engine, so each log crossing carries whole identities and never
a slot index. ADR-0107 D2 puts the prose in the Rust doc comment. ADR-0002 D1
keeps every value an integer or a fixed-point value, so each doc comment states
which columns carry the Q16.16 scale.

**Changes.** None. No record is superseded.

**Creates.** No record. Each crossing follows the convention that the gather log
and the tile change log already set, so it states no new constraint.

**Blockers.** None. This work opened none.

**Register.** FND-460 records that no caller could set an upkeep rate, so the
shortfall log could never hold an entry. FND-461 records that a small world
feeds itself, so a fixture built from one measures the fixture.

**Precedent.** Recurring defect shape 3 governs the whole item. A reader whose
writer no caller can reach is inert with an extra step in it.

## Done when

- Each of the four logs crosses as columns, in the shape the gather log uses.
- Each doc comment says what the log records, when the engine writes it, and
  that the log holds the last step alone.
- Each doc comment states the unit of every number, and names the columns that
  carry the Q16.16 fixed-point scale.
- A caller can reach the writer of each log.
- A test at the Python boundary reads each log with an entry in it, and a test
  reads an empty log on a step that recorded nothing.
- The whole check command runs green.

## Outcome

Done. The boundary gained four log reads and one verb.

**The verb is the upkeep rate, and it is what makes a shortfall possible.** No
pass and no binding ever wrote one, so the shortfall log could hold no entry at
all. The verb takes a set of sites and one rate, it resolves every identity and
checks the rate and the commodity before it writes anything, and one refusal
leaves the world unchanged.

**The last-step lifetime is stated in every doc comment, and no queue was
invented.** Each pass clears its own log before it does anything, so the next
step destroys what the last one recorded.

## References

None.
