---
id: 0410
title: Seed luxury resources and score the variety over them
status: complete
created: 2026-09-03
implements: [ADR-0001 D4, ADR-0002 D1, ADR-0004 D1, ADR-0006 D1, ADR-0022 D1, ADR-0023 D1, ADR-0023 D2, ADR-0024 D2, ADR-0040 D1, ADR-0053 D3]
changes: []
creates: []
serves: [PRD-0032]
blocked-by: [BLK-110, BLK-111]
---

## Why

The engine holds three gatherable resource kinds, and a unit takes an amount
of each one into a store. The number of kinds is a constant that the compiler
fixes, and the code uses it as an array length, so no caller can add a
kind.[^1] The project owner asked for a resource concept that a caller seeds
into a world, with any number of resources, so that resource variety becomes a
score.

A downstream game is the consumer. A god directs a congregation, and the god
needs to know what its ground is rich in.[^2]

## What the work does

1. A luxury is a presence and not a quantity. A tile carries a luxury or it
   does not, and no unit gathers one.
2. A set of luxuries is one 64-bit word, so the catalogue addresses 64
   luxuries. A caller that names a higher number gets a typed refusal.
3. The control plane names every placement in one call, in any order, and the
   engine sorts them. Python never loops over tiles.
4. The engine stores one entry for each tile that carries a luxury, and it
   stores nothing else.
5. The variety of a set is its population count. It is an exact whole number,
   so it aggregates with no drift and it needs no floating-point arithmetic.
6. Level 1 holds the union of the luxuries of each block, beside the cell
   summaries rather than inside them.
7. The control plane reads the variety of a tile, of a level 1 cell, of the
   ground one faction holds, and of the whole world.

## The answers this item takes, stated plainly

**A luxury lives on a tile.** The register holds the decision and the two
options it rejected.[^3]

**Nothing in the engine consumes the score.** The register holds that
decision, and a blocker holds what should consume it.[^4] [^5]

**The engine refuses a luxury above the ceiling.** It never folds two
luxuries onto one bit, because that reports the variety as one less than it
is.[^6]

## Impact review

**Governed by.**

- **ADR-0001 D4.** The engine hashes the whole state each frame. A luxury is
  authored rather than generated, so no other input produces it, and the
  field enters the hash. The golden files therefore move.
- **ADR-0002 D1.** No floating point in simulated or aggregated state. Every
  value here is a bit set, a population count or a 64-bit accumulator.
- **ADR-0004 D1.** Iteration order is explicit. The seed sorts by tile, the
  hash walks the entries in tile order, and the derivation of level 1 runs on
  one thread and writes each cell once.
- **ADR-0006 D1.** A stored entry is plain data with declared padding. The
  entry holds four padding bytes, and a test asserts that every one is zero.
- **ADR-0022 D1.** Level 0 is the only truth. The luxuries of a cell are
  derived from the tiles, and the whole level can be thrown away and rebuilt.
- **ADR-0023 D1 and D2.** An aggregate combines exactly, in any order. The
  union of two sets is associative, commutative and idempotent.
- **ADR-0024 D2.** Every summary field is extensive. A union is idempotent
  and it has no inverse, so it does not join the cell summaries. It sits
  beside them, in the way a direction does.
- **ADR-0040 D1.** Python is a control plane. The seed is one set-valued
  call, and every read answers with one fixed-width number.
- **ADR-0053 D3.** A set is one word. The luxury set follows the shape of the
  faction mask, and it departs from it in one place: it refuses an
  unaddressable member rather than folding it onto an overflow bit.

**Changes.** No record changes.

**Creates.** No record. The decision on where a luxury lives deserves one, and
a separate item holds it.[^7] The number was not taken here, because four
other workers were writing at the same time and the registry allocates the
number.

**Blockers.** BLK-110 holds what the score should change, so the engine
changes nothing because of it.[^5] BLK-111 holds whether 64 luxuries is
enough.[^8] BLK-007 governs every cost figure, so this item states none.[^9]

**Precedent.** FND-420 records that the gatherable catalogue cannot grow at
run time, which is why a luxury is a second tier.[^1] FND-421 records that the
field invariant check is unfalsifiable, because the constructor makes an
invalid field unrepresentable.[^10]

**Storage.** The field allocates nothing for a tile that carries no luxury. A
world in which nobody seeded a luxury holds no entry at all. Level 1 holds one
word and one accumulator for each cell, and the level is derived once, when
the seed lands.

## Done when

- A caller names any number of luxuries and any number of placements in one
  call, and the engine sorts them.
- Two worlds that carry different luxuries report different varieties, and two
  that carry the same report the same.
- A level 1 cell reports exactly the number of different luxuries on the tiles
  it covers. A test walks every tile and derives the answer a second way.
- The whole state hash reports a luxury, and the golden files are regenerated.
- The determinism tests pass at 1, 2 and 12 threads.
- A fixture supplies a world with no luxury, a world that carries the whole
  catalogue, and a tile that carries several.
- Each rule is broken on purpose, and the report says which tests caught it.

## Outcome

**Done.** The review holds the evidence, the gates and what was left
undone.[^11]

The golden state hash files moved, because the luxury field entered the hash.
The review says why, and it records the regeneration.

Two guards are not covered by a test. The field invariant check cannot fail,
and the finding records why.[^10] The 64-bit width of the deposit accumulator
cannot be tripped at a scale a test reaches, because the world would need
about a billion deposits.

## References

[^1]: Findings register, FND-420. `docs/FINDINGS.md`
[^2]: PRD-0032, a god knows what its ground is rich in. `docs/product/shaped/prd-0032-a-god-knows-what-its-ground-is-rich-in.md`
[^3]: Decisions register, DEC-201. `docs/DECISIONS.md`
[^4]: Decisions register, DEC-200. `docs/DECISIONS.md`
[^5]: Blockers register, BLK-110. `docs/BLOCKERS.md`
[^6]: Decisions register, DEC-202. `docs/DECISIONS.md`
[^7]: Backlog item 0411, record where a luxury lives. `docs/backlog/proposed/0411-record-where-a-luxury-lives.md`
[^8]: Blockers register, BLK-111. `docs/BLOCKERS.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: Findings register, FND-421. `docs/FINDINGS.md`
[^11]: The review of this item. `docs/reviews/0410-luxuries-and-variety.md`
