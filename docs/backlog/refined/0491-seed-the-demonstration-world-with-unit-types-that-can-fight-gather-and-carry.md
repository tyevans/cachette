---
id: 0491
title: Seed the demonstration world with unit types that can fight, gather and carry
status: refined
created: 2026-09-05
implements: [ADR-0145 D1, ADR-0145 D2, ADR-0145 D4, ADR-0122 D1]
changes: []
creates: []
serves: [PRD-0048, PRD-0011]
blocked-by: [BLK-050]
---

## Why

Every unit that the seeding founds carries the worker row, whose attack is
zero. An attacker whose attack does not exceed the armour of the defender
contributes zero, so a meeting between two founded groups inflicts no
casualties.[^1] The default table holds four filled rows, but seeding reaches
only one of them.

Three problems follow. The unit clause of the domination reader cannot fire
through combat, because a contest never eliminates a faction's units. The
fallen log remains permanently empty in a seeded run, and test fixtures derived
from the demonstration world never execute the combat casualty paths.[^2]
A developer watching a world to an end needs factions that can fight, win, and
lose units through battle.[^3] [^4]

## Impact review

**Governed by.** ADR-0145 D1 and D2 define unit types as rows of capability
columns where zero indicates inability. ADR-0145 D4 defines the default unit
type table containing worker, soldier, merchant, and leader rows. ADR-0122 D1
requires attack to exceed armor for combat resolution. ADR-0144 D2 requires the
controller to act through caller verbs.[^6]

**Changes.** None to decision records. The seeding logic in
`crates/cachette-core/src/world/seeding.rs` and `founding.rs` is updated to
assign diverse types to founding groups.

**Creates.** None.

**Blockers.** BLK-050 governs downstream game rules and type ratios.[^5] The ratio
of workers, soldiers, and merchants is stored as a named configuration in the
balance register.

**Precedent.** FND-486 recorded that all seeded runs had zero fallen units and
untested combat paths. FND-483 established that census reporting must explicitly
show zero vs missing rows,[^7] and subsystem reporting aligns with item 0278.[^8]

**Conflict surface.** `crates/cachette-core/src/world/seeding.rs`,
`crates/cachette-core/src/world/founding.rs`, and
`crates/cachette-core/src/census.rs`.

## Done when

- Seeding founds initial groups with a mix of unit types (workers and soldiers,
  plus merchants when group size permits) according to balance parameters.
- Initial type distribution reflects drawn faction weights (warrior factions
  receive more soldiers; builder factions receive more workers).
- Contests between hostile factions resolve with positive combat damage and
  generate casualty events when attack exceeds armor.
- The fallen log records unit deaths and the census reports unit counts per
  type along with total fallen casualties.
- Domination victory by unit annihilation is proven reachable in test fixtures.
- Determinism checks and state hashes remain reproducible across 1, 2, and 12
  threads.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0122, an attacker whose attack does not exceed the defender's armour contributes exactly zero, decision D1. `docs/adrs/draft/adr-0122-an-attacker-below-the-armour-contributes-exactly-zero.md`
[^2]: Findings register, FND-486. `docs/FINDINGS.md`
[^3]: PRD-0048, a developer watches factions play a game to an end. `docs/product/accepted/prd-0048-a-developer-watches-factions-play-a-game-to-an-end.md`
[^4]: PRD-0011, a unit is born, holds a job, and dies. `docs/product/accepted/prd-0011-a-unit-is-born-holds-a-job-and-dies.md`
[^5]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^6]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^7]: Findings register, FND-483. `docs/FINDINGS.md`
[^8]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/complete/0278-say-what-the-demonstration-world-never-produced.md`
