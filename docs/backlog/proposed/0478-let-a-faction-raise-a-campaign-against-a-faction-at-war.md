---
id: 0478
title: Let a faction raise a campaign against a faction at war
status: proposed
created: 2026-09-05
implements: [ADR-0144, ADR-0146, ADR-0003 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0049]
blocked-by: [BLK-007]
---

## Why

**A faction at war does nothing about it.** The relation says war and no
cohort moves. This item is pass 7 of the living world game layer.[^1]

A campaign is (faction, objective, cohort). Each faction holds a small bounded
register of campaigns, and the register size is a row in the balance
register.[^2] The register is simulated state. Three objective kinds exist:
take a site, wear an upgrade, and relieve an own site.

Raising a campaign is two verbs that exist: `set_unit_type` to the soldier row
and `send_units_to` the objective tile. The controller chooses an objective
from the relation band and the war weight. It raises no campaign against a
faction in the alliance or peace band. A campaign closes when its objective
holds or when its cohort is empty. No new movement machinery exists.

**This pass does not touch `fn step` in `world.rs`.** It may run beside passes
6 and 8 once pass 5 merges.

## What is missing before this is refined

- The impact review, decision by decision. The controller record ADR-0144 and
  the relation record ADR-0146 are being written beside this item.[^3] The
  review must say whether a campaign register is a controller reading or a
  world fact, because that decides whether it enters the hash.
- Whether a campaign needs a record at all. The scope rule gives a
  three-condition test, and the review must run it.[^4]
- How the controller chooses among the three objective kinds, and which key
  field the choice draws on. Each field needs a per-field test.
- The extreme that the fixture reaches: a register that is full, a cohort of
  one unit that falls on its first step, and an objective the faction already
  holds.
- The "Done when" statements, in the shape of item 0472: the two determinism
  tests at 1, 2 and 12 threads, each keyed draw with a per-field test, the
  defect put back and the test red, and the type stub edited by hand in the
  same commit as any new reader.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 8 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Decision Record Scope, section 1. `.agents/rules/adr-scope.md`
[^5]: Findings register, FND-320. `docs/FINDINGS.md`
