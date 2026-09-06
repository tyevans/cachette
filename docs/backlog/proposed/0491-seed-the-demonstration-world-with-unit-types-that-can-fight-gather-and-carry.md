---
id: 0491
title: Seed the demonstration world with unit types that can fight, gather and carry
status: proposed
created: 2026-09-05
implements: []
changes: []
creates: []
serves: [PRD-0048, PRD-0011]
blocked-by: [BLK-050]
---

## Why

Every unit that the seeding founds carries the worker row, whose attack is
zero. An attacker whose attack does not exceed the armour of the defender
contributes exactly zero, so a meeting between two founded groups kills
nobody.[^1] The default table holds four filled rows, and the seeding reaches
one of them.

Three costs follow. The unit clause of the domination reader cannot fire
through combat, because a contest never empties a faction. The fallen log
cannot hold an entry in a seeded run, so a reader who opens it sees the seeding
and reads it as the contest. A test fixture built from the demonstration world
supplies no fight, so the assertion never receives the input that would fail
it. The finding holds the reading and the evidence.[^2]

A developer who watches a game to an end needs a faction that can lose
units.[^3] A unit that holds a job needs a job the seeding gives it.[^4]

## What is missing before this is refined

Four questions. This item stays in `proposed/` until each has an answer.

**Which types a founded group gets, and in what proportion.** The proportion is
a balance value, and the blocker that holds the rules of the downstream game
governs it.[^5] Express the proportion as a named value in the balance
register, not as a number in a record.

**Whether the controller sets the types, or the seeding does.** The controller
acts only through verbs that a caller can also call, and the type verb is one
of them, so both paths work.[^6] The refined item must say which one serves the
need. A seeding that types the group makes the world true at tick zero. A
controller that types the group makes the mix answer to what the faction
lacks.

**Whether the first units of a faction differ by its seeded weights.** A
faction that is seeded to fight and a faction that is seeded to trade could
start from different mixes, or from one mix. This is a game rule, and the same
blocker governs it.[^5]

**What the census should report, so that a zero is visible.** The subsystem
census holds no row for the fallen, so nothing states that no unit died. This
is the shape of the seat census, where a zero meant that no position existed
rather than that no position stood empty.[^7] Item 0278 asks for the same thing
across every subsystem, and the refined item must say whether it adds a row
here or answers through that item.[^8]

## Done when

Filled in when the item is refined.

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
[^8]: Backlog item 0278, say what the demonstration world never produced. `docs/backlog/proposed/0278-say-what-the-demonstration-world-never-produced.md`
