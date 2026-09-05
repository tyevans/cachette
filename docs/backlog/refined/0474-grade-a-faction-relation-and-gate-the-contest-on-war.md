---
id: 0474
title: Grade a faction relation and gate the contest on war
status: refined
created: 2026-09-05
implements: [ADR-0146, ADR-0121 D1, ADR-0144 D2, ADR-0144 D3, ADR-0144 D4, ADR-0144 D5, ADR-0145 D3, ADR-0133 D1, ADR-0001 D4, ADR-0002 D1, ADR-0003 D1, ADR-0004 D1, ADR-0004 D4, ADR-0006 D1]
changes: [ADR-0121 D1]
creates: []
serves: [PRD-0049]
blocked-by: [BLK-007, BLK-080, BLK-130]
---

## Why

**Two adjacent factions always fight.** The contest pass fires wherever two
factions meet, so no faction can be at peace with a neighbour. This item is
pass 3 of the living world game layer.[^1]

A dense matrix of signed integers covers the ordered faction pairs. The entry
for (A, B) is what A feels toward B. The matrix is simulated state and enters
the state hash. Four bands cover the integer range: alliance, peace, tension
and war. The band edges are rows in the balance register, and no band name
appears in code.[^2]

Four passes read the relation. The contest fires only when at least one side
is in the war band toward the other. A unit converts only when the leading
faction is in a permitted band toward the faction of the unit. The engine
refuses an offer when either side is in the war band. A unit may not enter
ground that another faction holds when the holder is below a stated band.

Five causes move the relation by an integer step, and a drift moves it one
step toward peace on a period and a phase. The verb `move_relation(speaker,
other, step)` moves one entry, and it refuses when the speaker holds no unit
with command reach. A crossing of the war edge writes one plain-data event.

**This pass touches `fn step` in `world.rs`. Only one worker may hold it at a
time.** It waits for pass 1 to merge.

## Impact review

**Governed by.** ADR-0146 holds the whole shape: one signed integer for each
ordered pair in a dense matrix that enters the hash (D1), a band as a threshold
that a pass reads from the balance register (D2), every cause as an integer
step from the register (D3), the contest gated on the war band and three other
readers of the integer (D4), one gated verb (D5), and one plain-data event on a
crossing of the war edge (D6). The record is a draft, and the registry holds
its status.[^3] ADR-0144 D2 to D5 hold that the controller acts only through a
verb a caller can call, that a refusal is counted, that it draws from the keyed
generator a fixed number of times, and that its commands apply in an order the
data fixes. The relation move of the controller goes through `move_relation`
and no other path. ADR-0145 D3 holds that a gate on a faction power reads the
type column of the units the faction holds, so the verb reads the command reach
of the speaker unit and no per-faction flag. ADR-0133 D1 holds when a unit
converts; this pass adds one condition and changes nothing about the field.
ADR-0001 D4, ADR-0002 D1, ADR-0003 D1, ADR-0004 D1 and D4, and ADR-0006 D1 hold
the determinism rules the new state, the new draw and the new event obey.

**Changes.** ADR-0121 D1 states that contact is adjacency and that a tile is
contested when some unit within reach belongs to a faction that some unit on
the tile does not. ADR-0146 D4 adds one condition to the resolution and none to
the contact: the pass resolves the meeting only when at least one of the two
factions is in the war band toward the other. The contest passes from always-on
to gated. ADR-0146 D4 already states the change, so this item edits ADR-0121
nowhere. The reviewer of ADR-0146 decides whether the registry row of ADR-0121
records it.

ADR-0053 D7 states that a relation between two factions is one mask row for
each faction and never a field of the world. The graded matrix keeps the
constraint that matters, a table whose size follows the faction ceiling and
never the tiles, and widens the entry from one bit to one signed integer. The
two records answer different questions: a mask row says whether, and the
integer says how much. This item treats the matrix as sitting beside D7 and
lists the widening as a disagreement for the reviewer of ADR-0146.

**Creates.** None. ADR-0146 is allocated and written.

**Blockers.** BLK-007 governs the cost of every pass this item touches.[^5]
BLK-080 asks whether the engine's own movement ever brings two factions into
contact; the gate makes the question sharper, because after this pass a
contact fires only across a war pair, and the register row must say so.[^5]
BLK-130 governs the storm step, so the storm cause stays unset and unwired: a
god inflicts weather only on ground its own faction holds today, so no source
for that cause exists before pass 5.[^5] Every edge, step, period and bound is
a row in the balance register, and each row this item fills is provisional
with its derivation written.[^2]

**Which causes this pass wires.** The contract causes are wired through one
function of the relation module, called from the existing settle path with a
one-line edit at the delivery and at the deadline. The fallen cause and the
conversion cause are wired from the contest apply and the conversion apply. The
storm cause is stored as a row and not wired. The drift is wired as its own
stage.

**Precedent.** FND-048 records that a determinism test cannot see a broken
key, so the controller relation draw has a test for each field of its key.
FND-051 records that a fixture chosen for realism hides the defect, so the
contest fixture puts maximum attack against zero armour on adjacent tiles.
FND-320 records that nothing regenerates the type stub, so each new verb and
reader edits the stub by hand in the same commit.[^4]

**Serves.** PRD-0049.

## Done when

- The world holds one signed integer for each ordered faction pair, in a dense
  matrix that enters the state hash. The golden files move and the commit says
  so.
- The band edges, the six steps, the drift schedule and the verb bound are
  named constants that cite a balance register row, and every row they fill is
  marked provisional with a derivation.
- The contest resolves a meeting only when at least one of the pair is in the
  war band toward the other. A property test puts two factions on adjacent
  tiles with maximum attack against zero armour, and it kills nobody at peace
  and somebody at war, at 1, 2 and 12 threads. Every existing contest fixture
  sets its pair to war.
- A unit converts only when the leading faction is below the peace edge toward
  the faction of the unit. A test proves a pair at peace converts nobody.
- Admission refuses a step onto ground a faction holds when the holder is below
  the refusal edge toward the guest. A test proves the refusal and the grant.
- An offer is refused when either side is in the war band toward the other.
- A delivered contract raises both directions, a failed contract lowers the
  party that was owed toward the defaulter, a fallen unit lowers the victim
  toward the killer, and a conversion lowers the faction of the unit toward the
  leader. Each is one step from the register.
- The drift moves one step toward the peace band on its schedule, and a test
  proves a war pair returns to peace inside a bounded tick count.
- `move_relation(speaker, other, step)` refuses a speaker whose type has a
  command reach of zero, refuses a step above the bound, and otherwise applies
  through the one path that logs a crossing.
- A crossing of the war edge writes one `RelationCrossed` event: plain data,
  `repr(C)`, declared padding, no `bool`. A test proves exactly one event for
  each crossing and none for a move inside a band.
- The controller draws once for each faction with a leader, keyed on the
  controller system, the tick, the faction and a draw index past the
  evaluation indexes, and moves the relation toward the largest other faction
  through the verb. One test for each key field proves the field reaches the
  draw.
- The census holds `relation_moves` and `wars_declared`.
- The Python binding holds `move_relation`, `relation`, `relation_band`,
  `set_relation` and `relation_log_columns`, and the stub is edited by hand in
  the same commit.
- The demonstration prints a declaration and a peace from the relation log
  each frame, and walks no entity.
- The thread-count test and the golden state hash test pass at 1, 2 and 12
  threads with a war in the scenario.
- Each new test is put back to red once, and the commit body names the defect
  and the test.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Design: the living world game layer, sections 3 and 13. `docs/superpowers/specs/2026-09-05-living-world-game-layer-design.md`
[^2]: Balance register. `docs/reference/balance.md`
[^3]: ADR Registry. `docs/adrs/REGISTRY.md`
[^4]: Findings register, FND-320. `docs/FINDINGS.md`
