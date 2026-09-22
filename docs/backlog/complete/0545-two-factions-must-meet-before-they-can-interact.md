---
id: 0545
title: Two factions must meet before they can interact
status: complete
created: 2026-09-21
implements: [ADR-0205 D1, ADR-0205 D2, ADR-0205 D3, ADR-0205 D4]
changes: []
creates: [ADR-0205]
serves: [PRD-0058]
blocked-by: []
---

## Why

**Factions could interact and declare war across the world without ever meeting.**
Previously, diplomatic relations had global scope from frame zero. Any faction
could move relations, offer treaties, or declare war on an unseen faction. The
built-in controller would also select distant, unobserved factions as rivals,
prey, or victory threats.

This violated fog of war at the diplomatic level. Reconnaissance and scouting
yielded no diplomatic advantage. Factions must pass within line of sight of each
other or of a settlement before diplomacy opens between them.

## Impact review

**Governed by.** ADR-0205 D1 to D4 state that factions start unmet, meet
symmetrically when units pass within line of sight of each other or a settlement,
and that controllers ignore unmet factions.[^1] ADR-0146 D1 and D3 state the
relation matrix representation and drift schedule.[^2] ADR-0144 D1 governs the
controller and its verbs.[^3] ADR-0059 D1 to D3 govern fog of war and observation
passes.[^4]

**Changes.** No existing record is changed.

**Creates.** ADR-0205 states the meeting rules and contact tracking.[^1]

**Blockers.** BLK-007 governs Graviton costs, and this item adds a constant-time
matrix lookup.[^5]

## Done when

- Two factions start with no contact with each other.
- The engine refuses relation moves between unmet factions.
- Units passing within line of sight of each other establish symmetric contact.
- A unit passing within line of sight of a settlement establishes symmetric contact.
- The autonomous controller considers only met factions for rivalries, prey,
  and victory threats.
- Contact tracking enters the world state hash.
- Unit and integration tests verify meeting rules and controller behavior.

## Outcome

**What was done.**
- Added `contacts: Vec<u8>` to `RelationMatrix` to track symmetric meeting
  state for each ordered pair.
- Implemented `has_met`, `meet`, and `set_met` on `RelationMatrix` and `World`.
- Added `RelationError::UnmetFaction` returned when moving relations with an
  unmet faction.
- Implemented `World::update_contacts` during the observation step to detect
  when units observe other units or settlements.
- Filtered rivals, prey, and victory threats in the faction controller to only
  consider met factions.
- Contact matrix entries are included in the state hash.
- Added tests in `crates/cachette-core/tests/factions_meet_within_line_of_sight.rs`
  and updated existing relation tests in Rust and Python.

**Registers.** DEC-285 is recorded and closed.[^6] PRD-0058 is accepted.[^7]
ADR-0205 is drafted.[^1]

## References

[^1]: ADR-0205, two factions meet when a unit has line of sight to another unit or a city. `docs/adrs/draft/adr-0205-two-factions-meet-when-a-unit-has-line-of-sight-to-another-unit-or-a-city.md`
[^2]: ADR-0146, a faction relation is one signed integer per ordered pair, and a pass reads a threshold. `docs/adrs/accepted/adr-0146-a-faction-relation-is-one-signed-integer-per-ordered-pair-and-a-pass-reads-a-threshold.md`
[^3]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^4]: ADR-0059, fog storage grows with observed area, not with world area. `docs/adrs/accepted/adr-0059-fog-storage-grows-with-observed-area.md`
[^5]: Target platform costs. `docs/BLOCKERS.md`
[^6]: Decisions register, DEC-285. `docs/DECISIONS.md`
[^7]: PRD-0058, two factions must meet before they can interact. `docs/product/accepted/prd-0058-two-factions-must-meet-before-they-can-interact.md`
