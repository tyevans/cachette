# Blockers (Register)

This document is a **register**. It lists work that is stopped and names what
must happen to start it again.

A blocker needs **information** the project does not have. Compare
`DECISIONS.md`, which lists choices that need judgement. If work can continue
under a stated assumption, it is a decision, not a blocker.

Numbers are permanent. Never reuse one. A resolved blocker keeps its row.

| Field | Meaning |
|---|---|
| Blocks | What cannot proceed |
| Owner | Who can resolve it |
| Status | `Open`, `Resolved`, or `Dropped` |

## Open

### BLK-001 — Tile scale, and therefore world extent

**Owner:** project owner. **Blocks:** every movement constant, the byte
budget, and the character of the world.

Report 17 found a hard incompatibility. An 80 metre tile at a historical march
rate of 24 km per day is consistent: dwell 2, crossing capacity 16, a world
about 330 km across. A 1 km tile at the same march rate needs dwell 25, and
holding the approved 12.5-second crossing then needs capacity 200 on a bridge
tile. That exceeds `u8` headroom and looks absurd.

**A continental tile scale and a 12.5-second crossing cannot both hold.**

Three ways out: accept a regional world of about 330 km; accept a slower
crossing; or shorten the game day, which forces a re-bake of every per-tick
rate including needs decay.

The movement constants are written parametrically, so they resolve when this
is answered.

### BLK-002 — Name three archetypes you expect to exist

**Owner:** project owner. **Blocks:** about 2,000 lines of storage code, and
the zero-copy story for Python.

An archetype is the exact set of component types an entity carries, not a
category. If all units share one shape, the storage is a generational
struct-of-arrays arena and the archetype machinery is dead weight. If several
shapes exist, the design differs.

Open since the first research round.

### BLK-003 — Is one million the whole population, or one million soldiers?

**Owner:** project owner. **Blocks:** storage and cost figures in the needs
report and the agency report.

If civilians are separate, the entity count rises and several budgets move.

### BLK-004 — Target living character population

**Owner:** project owner. **Blocks:** the character tier budget and the
opinion edge storage.

The character report recommends 20,000 to 50,000 and keeps 262,144 as a hard
ceiling. A living character costs 840 bytes with opinion. The whole layer is
34 MB at 20,000, 173 MB at 100,000, and 453 MB at the ceiling.

### BLK-005 — Settlement count

**Owner:** project owner. **Blocks:** every storage figure in the entity
economy report.

The reports assume 5,000 settlements. That number is an assumption, not a
decision.

### BLK-006 — Tile upgrade fraction

**Owner:** project owner. **Blocks:** the choice between dense and sparse
storage for tile upgrades.

The entity economy report estimated 2.7 percent of tiles carry an upgrade.

### BLK-007 — No measurement exists on the target platform

**Owner:** engineering. **Blocks:** confidence in every cost figure in every
report.

Every number in the research is derived, not measured. The research agenda
names benchmarking on Graviton as blocking most of its own conclusions.

This blocker cannot be resolved by a decision. It needs a benchmark harness
and a machine.

## Resolved

### BLK-008 — Upkeep per unit or per formation

**Resolved.** A unit is an individual soldier. The three-tier split makes it
affordable: individual decay, pooled consumption, aggregate decisions.

### BLK-009 — Tile capacity

**Resolved.** Eight units, stored as `u8`, with capacity as a data-driven
parameter. Crossing terrain raises it to 16.

### BLK-010 — Do formations exist as entities

**Resolved.** Formation membership is an ownership column plus a reverse
index. A formation is not a spatial region: a region is not stable under
movement, so a move order would change its own recipient set across frames.

### BLK-011 — Promoted soldier lineage

**Resolved.** A promoted soldier gets no invented ancestry. He founds a new
house, his kinship to everyone is zero, and he cannot inherit a title by
blood. A title holder may **appoint** him. His children inherit from him
normally.
