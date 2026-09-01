---
id: 0095
title: Record that a spawn may over-fill a tile
status: refined
created: 2026-08-31
implements: [ADR-0074 D1, ADR-0074 D2, ADR-0074 D3, ADR-0074 D4]
changes: []
creates: []
serves: [PRD-0002]
blocked-by: []
---

## Why

A spawn reads no occupancy of the tile it places into. Two foundings whose
discs overlap therefore put a tile above the capacity of its ground, and the
engine accepts it. Admission then only ever takes units off that tile, because
admission grants no intent that raises a tile above its capacity.[^1]

This item once proposed to stop that. The project owner has since ruled the
other way. A spawn may over-fill a tile, and an over-full tile is a state of
the world rather than a fault. The engine already behaves that way, so no
engine change follows from the ruling.

What follows is repair. Three documents state the reversed answer or rest on
it, and one product statement is false against the ruling. A document that
states something false with the authority of a record costs every decision
made from it.

## Impact review

**Governed by.** The movement record binds the behaviour. D3 states that
admission reads the occupancy of a target tile from the derived unit-to-tile
structure and carries no per-tile array of its own, and that only an admitted
departure releases room.[^1] D4 states that capacity is a property of the
terrain.[^2] The engine honours both today, and this item changes no code that
touches them.

The record under this item states the ruling.[^3] It holds four decisions: a
spawn places without reading the capacity, admission is the only enforcer and
its guarantee is monotone, the engine holds no dense per-tile count, and a
caller that wants a tile filled to capacity counts its own placements.

**Changes.** No record. The record this work needed is a draft, and it takes
the number that the reversed record held. The movement record gives up
nothing, and the deferral it carried about occupancy storage is answered by
the ruling that no new storage exists.[^1]

**Creates.** No record.

**Blockers.** None.

**Precedent.** The findings register records that two accepted records
disagree about whether a per-tile array of counts exists, and it says the
reversed record closed that disagreement.[^4] It did not, because the reversed
record is gone. The replacement record closes it the other way: the movement
record holds the true claim, and the bridge record named a mechanism that
never existed.[^5]

## What the work does

1. A second reviewer, who did not write the replacement record and did not
   write the record it replaced, reads it and sets its status.[^6]
2. The decisions register states the owner's ruling in place of the reversed
   outcome it holds for the spawn question.
3. The findings register states how the record pair was settled, in place of
   the claim that the reversed record settled it.
4. The product record that states that no tile holds more units than its
   capacity allows is repaired, because the engine does not meet that
   statement and will not.[^7]

## Done when

- A test asserts the monotone invariant: no tile rises above the capacity of
  its ground, and a tile that a caller over-filled does not rise. A test that
  asserts that no tile is ever above its capacity fails on a legitimate world
  and must not exist.
- A test over-fills a tile through the public interface and asserts that the
  spawn is granted.
- A test asserts that an over-full tile admits nobody while its units may
  depart.
- No document says that a spawn refuses a tile at capacity. A whole-tree
  search for the claim comes back clean, and the search command is in the
  commit body.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^2]: ADR-0056, decision D4. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^3]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity. `docs/adrs/draft/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^4]: Findings register, FND-081. `docs/FINDINGS.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^7]: PRD-0002, a developer watches the world run. `docs/product/shipped/prd-0002-a-developer-watches-the-world-run.md`
