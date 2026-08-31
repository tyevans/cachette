---
id: 0053
title: Hold a resource stock on a tile and gather from it
status: proposed
created: 2026-08-31
implements: [ADR-0002 D1, ADR-0004 D4, ADR-0007 D1]
changes: []
creates: []
serves: [PRD-0007]
blocked-by: []
---

## Why

The ground offers nothing. Every tile is worth the same to a unit, so
movement carries no meaning and no later need can start. A wage needs
something to pay, a meal needs something to eat, and a trade needs something
to trade.

This item gives a tile a stock and lets a unit take from it. It is the first
quantity the world produces, and the conservation test that covers it is the
test every later quantity reuses.

## What the work does

1. A tile carries an integer stock of one commodity, placed from the seed and
   correlated with the terrain.
2. A unit standing on a stocked tile takes from it. The stock falls by exactly
   what the unit took, and the unit carries exactly that amount.
3. Two units that want more than the tile holds resolve as one set-valued
   operation over the whole set of takers, in the same shape that movement
   admission already uses: sort by a stable key, then admit in that order.
4. A conservation test sums the tile stocks and every unit's carried amount
   and asserts that the total is unchanged by a gathering pass.

## Impact review

**Governed by.** ADR-0002 D1 forbids a floating point quantity, so a stock is
an integer. ADR-0004 D4 requires a stable sort key. ADR-0007 D1 requires the
sort to take a key vector rather than a comparison function. ADR-0056 D2 and
D3 hold the intent-then-admission shape for movement, and this work uses the
same shape for a different verb.[^1] [^2] [^3] [^4]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
The commodity count follows the recommendation in DEC-001 and is not invented
here.[^5]

**Precedent.** FND-043 records that a value type which cannot hold zero can
lose a real value. An empty deposit is a real state.[^6]

**Serves.** PRD-0007.

**Conflict surface.** `crates/cachette-core/src/resource.rs` is new.
`crates/cachette-core/src/world.rs` at the constructor, the step, the state
hash and the invariant check. `crates/cachette-core/src/soldier.rs` gains a
carried-amount column. It touches the same three functions of `world.rs` as
item 0052, so it rebases on that item and does not merge beside it.

## What is missing before this is refined

**The registry row.** This work states a constraint that no reserved row
holds: **a contested take resolves by one set-valued admission over the whole
set of takers, never by an atomic subtract, a lock or a retry.** The three
conditions of the scope rule all hold.[^7] A contributor could reasonably
choose an atomic subtract, because it is shorter and it conserves the total. It
does not fix *which* taker got the last unit, so it is the defect that both
determinism tests pass over. Changing it later means rewriting every gathering
call site. The reasoning is not visible in the code.

Whoever picks this item up **allocates the row in the registry before writing
the record**, and does not choose the number themselves.[^8] Do not reuse row
0058, which states the flux-pair claim for a field and is a different claim.

**The placement rule.** Whether the stock is a generated field read from the
seed, as terrain is, or a stored sparse column, is a second decision. ADR-0068
holds the claim for terrain and the same reasoning may or may not carry.[^9]
Answer it in the impact review, not during the work.

## Done when

- A tile answers what it holds, and different tiles hold different amounts.
- A unit takes from the tile it stands on, and both quantities change by the
  same integer.
- A tile that holds nothing gives nothing, and the refusal is a typed error or
  a zero, stated once.
- A property test asserts that the conservation sum is unchanged over a run of
  many gathering passes.
- A property test asserts that the result is identical at 1, 2 and 12 threads,
  including the case where more units want the stock than the tile holds.
- A test asserts the contested case at the boundary: exactly one unit more
  than the stock can serve.
- The fixture is built to produce the contested case rather than copied from
  the demonstration world, and the commit body says how that was checked.[^10]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^4]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^5]: Open decisions register, DEC-001. `docs/DECISIONS.md`
[^6]: Findings register, FND-043. `docs/FINDINGS.md`
[^7]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^8]: ADR Registry. `docs/adrs/REGISTRY.md`
[^9]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/REGISTRY.md`
[^10]: Findings register, FND-051. `docs/FINDINGS.md`
