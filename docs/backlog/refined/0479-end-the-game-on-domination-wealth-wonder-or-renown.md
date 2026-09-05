---
id: 0479
title: End the game on domination, wealth, wonder or renown
status: refined
created: 2026-09-05
implements: [ADR-0148 D1, ADR-0148 D2, ADR-0148 D3, ADR-0148 D4, ADR-0090 D1, ADR-0090 D2, ADR-0002 D1, ADR-0002 D3, ADR-0004 D1, ADR-0001 D4]
changes: []
creates: []
serves: [PRD-0048]
blocked-by: [BLK-050, BLK-150, BLK-007]
---

## Why

**Only one win path exists after pass 1, and it fires only at the tick
limit.** A faction that removes every rival keeps playing to the limit. This
item is pass 8 of the living world game layer.[^1]

Three readers join the territory reader. Domination fires when one faction
holds every seat, or when every other faction has no units. Wealth or wonder
fires when a faction stock total reaches a target, or when a wonder upgrade
completes on ground the faction holds. Renown fires when a character of the
faction reaches a renown target. The controller stage checks the readers in
the fixed order of the record before it evaluates. The stock total sums the
stores of the own sites in a 64-bit accumulator.

Two upgrade kinds join the catalogue. The wonder has large work, and its
completion fires the wealth-or-wonder path. The store raises the store
capacity of the site on or beside its tile. The work of each kind is a row in
the balance register.[^2]

**This pass does not touch `fn step` in `world.rs`.** It runs beside passes 6
and 7. The wall kind waits for pass 4, because a wall needs the condition that
pass 4 adds.

## Impact review

**ADR-0148 D1.** The record keeps its shape: winner, path, tick. The path
numbers already name four paths. The boundary gains `standing(faction)`, one
running value for each path, which D1 asks for.[^3]

**ADR-0148 D2.** `check_game_end` returns while the record is set. It calls
`record_end` at most once per tick, and `record_end` refuses a second write.

**ADR-0148 D3.** Each reader is a pure function of the world. It runs inside
`run_controller`, before the plan. No reader walks the units or the tiles.
The domination reader reads the live count of each faction, which the soldier
arena keeps as a running total, and reads the holder of each seat tile, which
is one lookup for each faction. The wealth reader walks the settlement arena,
which is not the population. The wonder reader walks the sparse upgrade map,
which holds one entry for each improved tile and nothing else.[^6] The renown
reader walks the character arena. Every total is an `i64`. A tie resolves by
the lowest faction identifier, because each reader visits the factions in
ascending order and stops at the first that fires. The order is domination,
territory, wealth or wonder, renown, as the record states it. The dispatch
text for this pass said that territory stays last; the record says otherwise,
and the record wins.

**ADR-0148 D4.** Unchanged. The controller stage reads the record at its start.

**ADR-0090 D1 and D2.** The two new kinds are rows in the same catalogue. The
store holds no new column. A wonder under construction is a clamped whole
number, as every other kind.

**ADR-0002 D1 and D3.** Every threshold is an `i64` or a raw Q16.16 integer.
The stock total and the renown compare against a raw Q16.16 integer.

**ADR-0004 D1.** Each reader visits the factions in ascending identifier
order, and the settlement, upgrade and character walks are in slot or tile
order.

**ADR-0001 D4.** No new state. The readers write only the record that already
enters the hash.

**No record on site store capacity exists.** The engine holds no store
capacity. The store kind therefore states its raise as a catalogue row, and
the world exposes the raise of a site as a reader. No pass reads it yet, and
the doc comment of the reader says so. Item 0348 on the upgrade catalogue has
not landed, and this item does not wait for it.

**BLK-150.** No pass writes renown. The reader ships behind the blocker: it
reads a column that only the control plane writes, so a game built outside
the engine can end on renown and the engine alone cannot. The balance harness
records that the path did not fire, and the register row stays under the
blocker.[^4]

**BLK-050 and BLK-007.** Every target and every work value is provisional.
The register row holds the derivation and the commit.

## Done when

- `UpgradeKind` holds `Wonder` and `Store`. The kind count, `ALL`, `from_u8`,
  the work table, the capacity table and the gather bonus table each hold a
  row for both, and the whole-tree search for exhaustive matches is in the
  commit body.
- Each reader fires on a fixture at its extreme: a faction that holds every
  seat while a rival still lives; a world where every other faction has zero
  units; a stock total at the target and one raw unit below it; a stock total
  that overflows a 32-bit accumulator; a wonder one unit of work short and
  then complete; a character at the renown target and one below.
- Two paths true on one tick record the earlier path of the fixed order.
- The record is written once. A second path that becomes true later changes
  nothing.
- Each defect was put back and the test went red: a reader dropped, the order
  swapped, a second write let through.
- The thread-count test and the golden hash test pass at 1, 2 and 12 threads
  with a wonder completing in a scenario.
- `score(faction)` keeps its meaning. `standing(faction)` returns one running
  value for each path. `game_end()` may return each of the four path names.
  The type stub is edited by hand in the same commit.[^5]
- The census holds `wonders_complete` and `stores_built`. The demonstration
  prints each path name as words.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 5, 6.4 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR-0148, a game end is recorded once and stops the controllers. `docs/adrs/accepted/adr-0148-a-game-end-is-recorded-once-and-stops-the-controllers.md`
[^4]: Blockers register, BLK-150, BLK-050 and BLK-007. `docs/BLOCKERS.md`
[^5]: Findings register, FND-320. `docs/FINDINGS.md`
[^6]: ADR-0090, a tile upgrade is stored sparsely, decision D1. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
