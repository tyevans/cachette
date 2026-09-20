---
id: 0490
title: Let the control plane read the store of a seeded settlement
status: refined
created: 2026-09-05
implements: [ADR-0014 D1, ADR-0014 D7, ADR-0062 D1, ADR-0062 D5, ADR-0063 D1]
changes: []
creates: []
serves: [PRD-0047]
blocked-by: []
---

## Why

The site economy reader takes the whole identity of a settlement. It refuses a
slot index, and it is right to refuse one.[^1]

The seeding verb founds every faction in one call and gives back one report for
each faction. That report holds the place, the people and the refusal. It holds
no settlement identity. The demonstration builds its world with that verb and
with no other, so nothing in the control plane can name a settlement of that
world.[^2]

A watcher outside the window therefore cannot read a store, a production rate
or an upkeep rate of any settlement the seeding made. A finding records the
measurement.[^3]

A product requirement asks that a developer set what a settlement holds from
Python and read the value back.[^4] A developer cannot do either action for a
seeded world today, because no call gives the required identity.

The repair is one key. The founding report of the seeding verb must include the
settlement identity under key `"site"`, matching the shape that the single-group
founding call already produces.[^5]

## Impact review

**Governed by.** ADR-0014 D1 holds that an entity identity is a slot index and
a generation, so a bare slot index is not an identity.[^6] ADR-0014 D7 holds that
the location table is a dense array indexed by slot, and that the Python binding
exposes an opaque 64-bit integer rather than an internal pointer.[^7] ADR-0062
D1 and D5 hold that production and upkeep are rates attached to a site, and that
the site economy reader reads those rates by site identity.[^8] ADR-0063 D1 holds
that a need is a rate with a threshold attached to a site.[^9]

**Changes.** None to decision records. The Python founding report is an external
representation of the internal `FoundingOutcome` structure.

**Creates.** None. No new decision record is needed.

**Serves.** PRD-0047 (A game states its own economy: a developer sets what a
settlement holds from Python and reads it back).[^4]

**Blockers.** None (`blocked-by: []`).

## Done when

- `crates/cachette-py/src/world/founding.rs` populates `"site"` in the dictionary
  for every seated founding in `founding_reports`.
- The docstring for `found_run_for_every_faction` and `seed_world` documents the
  `"site"` key.
- `python/cachette/_core.pyi` declares `site: int` on `FoundingReport`.
- A Python test asserts that `seed_world()` reports the site identity for every
  seated faction, and that `site_economy(site)` and
  `set_settlement_store([site], ...)` accept that identity and read back the
  written store.
- A test proves able to fail when the site key is missing or an invalid identity
  is provided.
- All repository citation, priority index, and record checks pass clean.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The site economy reader. `crates/cachette-py/src/world/settlement.rs`
[^2]: The seeding verb. `crates/cachette-py/src/world/founding.rs`
[^3]: Findings register, FND-485. `docs/FINDINGS.md`
[^4]: PRD-0047, a game states its own economy. `docs/product/shaped/prd-0047-a-game-states-its-own-economy.md`
[^5]: Group founding report. `crates/cachette-py/src/world/founding.rs`
[^6]: ADR-0014, an identity is a slot index and a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^7]: ADR-0014, an identity is a slot index and a generation, decision D7. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: ADR-0062, production and upkeep are rates attached to a site, decisions D1 and D5. `docs/adrs/accepted/adr-0062-production-and-upkeep-are-rates-attached-to-a-site.md`
[^9]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D1. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
