---
id: 0181
title: Give a kind of work the commodity it fills
status: refined
created: 2026-09-02
implements: [ADR-0149 D3, ADR-0147 D1]
changes: []
creates: []
serves: [PRD-0010, PRD-0047, PRD-0050]
blocked-by: [DEC-073]
---

## Why

A site opens positions of each kind of work in proportion to what it lacks of
each. What it lacks is what it wants of a kind, less what it holds of the
commodity that the kind fills.

Today, the store of a site holds only one commodity, so every kind of work
maps onto that single entry. A site holds the identical quantity against every
work kind, and open positions reflect base preference alone rather than
resource scarcity. Crucially, this single-commodity placeholder completely
shuts down the trade subsystem: sweeps of 16 seeds over 20,000 ticks showed zero
offers, zero contracts, and zero carrier assignments.[^2] Because all goods map
to one commodity, every faction offers and desires the identical basket,
preventing any trade match from ever forming.[^3]

## Impact review

**Governed by.** ADR-0149 D3 states that trade boards are simulated state
readable by all factions. ADR-0147 D1 specifies trade considerations and contract
terms. DEC-073 governs the choice of commodity representation.

**Decisions.** DEC-073 is resolved by taking recommended Option 1: expand the
engine commodity table to 3 distinct commodities (e.g. food, raw materials,
and construction goods) within `crates/cachette-core/src/site.rs` and
`position.rs`.[^1]

**Changes.** None to decision records.

**Creates.** None.

**Blockers.** DEC-073 settles the commodity mapping and count.

**Precedent.** FND-549 proved that the entire trade simulation remained dead
across 20,000 ticks because all three trade goods collapsed onto one commodity.

**Conflict surface.** `crates/cachette-core/src/site.rs`,
`crates/cachette-core/src/position.rs`, `crates/cachette-core/src/rates.rs`,
and `crates/cachette-core/src/trade.rs`.

## Done when

- `COMMODITY_COUNT` in `site.rs` is expanded from 1 to 3 distinct commodities.
- `WORK_COMMODITY` in `position.rs` maps each work type to its corresponding
  commodity ID.
- Site stores track inventory and consumption independently for each commodity.
- Work positions open dynamically in response to specific commodity deficits.
- Faction trade boards publish differentiated surplus offers and demand wants
  across different commodities.
- Trade negotiation matches complementary offers and wants between factions,
  yielding active contracts and carrier assignments.
- Tests verify that draining one commodity increases positions only for its
  associated work kind, leaving other position counts intact.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Decisions register, DEC-073. `docs/DECISIONS.md`
[^2]: Findings register, FND-549. `docs/FINDINGS.md`
[^3]: PRD-0010, a good moves to where it is wanted. `docs/product/accepted/prd-0010-a-good-moves-to-where-it-is-wanted.md`
