---
id: 0305
title: Give a laden unit a reason to go home
status: complete
created: 2026-09-03
implements: [ADR-0064 D1, ADR-0091 D6, ADR-0095 D1, ADR-0095 D3, ADR-0096 D4, ADR-0098 D3]
changes: [ADR-0098 D1]
creates: [ADR-0109, ADR-0110]
serves: [PRD-0007]
blocked-by: []
---

## Why

**The delivery of a carried load works and almost never runs.** A unit gives
its load to the store of its site only while it stands on the tile of that
site, and no rule in the engine ever puts it there on purpose. A unit gathers
where it stands and then steps wherever the exit field of its cell points.

The four options are `roam`, `forage`, `climb` and `join`. Each ranks a
neighbouring cell on a summary field. **None of them is "go home".**

The findings register holds the measurement that opened this item: a run of the
demonstration world for 4000 ticks delivered nothing of any kind.[^1]

Until this exists, what a settlement holds is the rate the founding set from
the survey and nothing else, and the whole economy is decided before the first
frame.

## The question this item has to answer

**A site is not a summary field.** The exit field ranks a neighbour on a value
that a cell carries, and one unit's own site is not a fact that a cell
carries.[^2] So a fifth option that reads a cell field cannot express "my
home", and the shape that would express it is a per-unit search, which two
records refuse.[^3] [^4]

At least three shapes could answer it, and choosing between them is an
architectural decision that needs a record.

1. **A cell field that says how much of the ground here belongs to the unit's
   faction.** The influence field already solves over the same lattice, so a
   laden unit could climb it. It steers a unit toward its faction rather than
   toward its own site, which may be near enough.
2. **A carry threshold that changes the option.** A unit whose load is full
   switches to an option that ranks on that field. This keeps the option set as
   the one place behaviour is declared.
3. **A per-unit direction toward a stored home address.** It answers exactly
   and it is the shape ADR-0091 D1 refuses on cost.

## Impact review

### The measurement this item starts from

The demonstration world, 256 by 256 at four factions and 64 people each,
4000 ticks at four threads. **The run delivers 38 food, no wood and no stone,
while 145 live units hold 13341 units of resource between them.** The finding
that opened this item measured zero. The difference is the movement fall-back
that landed after it: a unit the ground refuses now takes a keyed draw, so a
random walk puts a unit on its own site tile once in a while.[^5] **The
delivery is therefore reachable by accident and by nothing else**, which is the
same defect with a smaller number on it.

### Which records govern this work, decision by decision

**ADR-0064, a unit chooses by scoring a small fixed option set.** Accepted.

- D1, a unit scores a fixed set and takes the highest. Honoured. The set grows
  from four rows to five and stays fixed at compile time.
- D2, the engine stores no score and recomputes an explanation. Honoured.
- D3, the choice writes an intent and movement reads it. Honoured.
- D4, the interval. Untouched.
- D5, a tie goes to the lowest option index and never to a draw. Honoured. The
  new row takes the highest index, so every tie that exists today keeps the
  winner it has.

**ADR-0095, a behavioural strategy arrives as a field over cells.** Draft.

- D1, a strategy takes its direction from a field over cells and never from a
  search that starts at a unit. **This decision names shape 3 above and refuses
  it in the words the item uses.** The work follows D1 and drops shape 3.
- D2, the cost follows the cell count and the strategy count. Honoured. The new
  field is derived once for each cell and each faction.
- D3, a strategy that names a place is seeded at that place, and several
  destinations are one field. Honoured, and it is the mechanism this work
  builds: every site of a faction seeds the plane of that faction at once.
- D4, whether a strategy field carries between frames is not settled. The work
  derives from nothing at each rebuild, which is the option the register
  recommends, so no answer is needed.[^6]

**ADR-0091, movement takes its direction from a per-cell field.** Draft.

- D1, the direction comes from a per-cell field. Honoured. The return direction
  is a per-cell field with one plane for each faction.
- D2, the field is a projection of level 0 and carries nothing between frames.
  Honoured. The return field is derived again at every rebuild of level 1.
- D3, the exit field is not a summary field. Honoured, and the return field is
  not one either.
- D4, a cell ranks its neighbours and the lowest direction index wins a tie.
  Followed exactly, against the reach rather than against a summary field.
- D5, a cell that admits no unit is not a candidate. Followed exactly.
- D6, a refused direction falls back to a keyed draw. Reused unchanged, and it
  is what carries a unit the last block to the tile of its site.

**ADR-0098, the choice is decided for each cell and each bucket of need.**
Draft.

- D1, one answer serves a cell and a bucket. **Contradicted as written.** The
  new option is worth something to a unit that carries a full load and nothing
  to a unit beside it that carries none, so two units of one cell and one
  bucket no longer share an answer. ADR-0109 states the widened key, and D1 of
  this record gains a sentence that cites it.
- D2, a bucket is scored at its lower bound. Untouched.
- D3, the table fills as a unit asks for a bucket. Honoured. The table now
  fills over a bucket and a class, and the fill still changes no answer.
- D4, an explanation scores the need that the pass scored. Honoured, and the
  explanation reports the class as well.

**ADR-0096, cost follows the lattice, not the population.** Draft.

- D1 and D4, the engine computes one answer once for every reader that would
  compute the same answer. Honoured. Two units of one cell, one bucket and one
  class share one answer, and the widened key holds a bounded number of
  classes, so the table has a ceiling the population cannot raise.

**ADR-0053, a faction is a bit in a mask, and a relation is a plane.** Accepted.

- D3, a field indexed by the faction multiplies the world by the faction count,
  and a summary field must not be one. Honoured. The return field is indexed by
  the faction at the pitch of a level 1 cell and not at the pitch of a tile,
  which is the pitch at which the influence field is already one plane for each
  faction.[^7] ADR-0110 states that distinction, because a reader who knows
  only D3 reads the new field as a violation of it.

**ADR-0002, state holds no floating point number.** Accepted. D1 and D2
honoured: the reach of a cell is a small whole number, the score of the new
option is a Q16.16 value, and every operation goes through the arithmetic
module.

**ADR-0003, every random draw is keyed.** Accepted. D1 honoured, and the work
adds no draw. The two draw indices the movement pass holds are enough, because
the return step falls back to exactly the draw that ADR-0091 D6 already gives a
refused unit.

**ADR-0004, iteration order is explicit.** Accepted. D1 honoured. The
derivation walks the factions in ascending identifier, the cells in ascending
index, and the six neighbours in ascending direction index.

**ADR-0005, a solver runs a fixed iteration count.** Accepted. D1 honoured. The
reach relaxation runs a stated number of passes, reads no residual and tests no
convergence.

**ADR-0009, parallel stages write disjoint outputs.** Accepted. The derivation
runs on the calling thread, beside the exit field derivation it sits next to.

**ADR-0062, production and upkeep are rates attached to a site.** Accepted. D2
and D3 untouched. The delivery pass is not changed by this work.

### Which records this work creates

- **ADR-0109**, the choice key holds a bounded class of the unit's own state.
- **ADR-0110**, a unit returns by climbing a reach field seeded at every site of
  its faction.

Both rows are in the registry before either file exists.

### Which registers open or close

The findings register gains the measurement of what the change did and what it
did not do. No blocker closes. The decisions register gains the row for what a
unit does when the nearest site of its faction is not its own home.

## What this item does

1. **A class of the unit's own state enters the choice key.** The class is
   `Free` or `Laden`. A unit is laden when it holds a home site and its carry
   reaches the carry mark, which is a parameter of the world.
2. **A fifth option, `deliver`, ranks that class rather than a cell field.** It
   is worth one unit of value to a laden unit and nothing to any other, so a
   unit that carries nothing never takes it.
3. **A return field steers it.** One plane for each faction over the level 1
   cell lattice, seeded at every site of that faction, holding the direction of
   the neighbouring cell that is nearer to a seed.
4. **A laden unit stops gathering**, because the `deliver` row names no
   resource kind and the choice pass already writes the gather order from that
   row.
5. **A unit with no home is never laden**, so it never takes the option. The
   engine has no verb that gives a unit a home, so an option for a homeless
   unit would be a capability nothing could act on.[^8]

## What it costs at the target scale

The choice table doubles in entries, and it fills lazily, so a cell scores the
smaller of the units it holds and the entry count. The return field costs one
byte for each cell and each faction, plus a fixed number of relaxation passes
over the same. Both follow the cell count and the faction count, and neither
follows the population. **No figure appears here, because one blocker governs
every cost figure this project holds.**[^9]

## Outcome

The work is complete. What follows is what was built and what was measured.

**The engine holds five options, and the new one ranks the state of the unit.**
A unit that holds a home site and carries at least the carry mark is laden. A
laden unit takes the option that carries its load home, it stops gathering
while it holds that option, and a field over the level 1 cells steers each
step.

**The field is one plane for each faction, seeded at every live site of that
faction.** It holds the number of cells to the nearest seed, and the direction
of the neighbour that is nearer. It is derived again at every rebuild of level
1 and carries nothing between frames.

**The measurement. The demonstration world, 256 by 256 at four factions and 64
people each, driven 4000 ticks at four threads.**

| Run | Food | Wood | Stone | Held in the carries at the end | Live units |
|---|---|---|---|---|---|
| Before | 38 | 0 | 0 | 13341 | 225 |
| After | 3125 | 0 | 0 | 3274 | 228 |

The delivered food rises by a factor of 82, and what the live units hold at the
end falls to about one quarter of what it was. The two move together, which is
what a working sink looks like. The first delivery of the run lands at tick
131. About one unit in four holds the option at any moment.

**Wood and stone stay at zero, and this work did not change that.** One option
gathers, and the resource it names is food. No unit ever gathers wood or stone
without an order from the control plane.

**The measurement before the work is not the one the finding holds.** The
finding measured zero, and the tree delivers 38 by accident, because a movement
change landed between the two.[^5] A later finding holds the correction and
what it cost a test.[^10]

### What it did not do

**The field steers a unit to the nearest site of its faction, and the delivery
needs its own home site.** Each faction of the demonstration holds one site, so
the two are the same place. A world with two sites of one faction can steer a
unit to the wrong one, and the decisions register holds that row with a
recommendation.[^12]

**The last block is a random walk.** The field ends at the cell that holds the
site, and the tile of the site is one tile of that block. The keyed draw
carries a unit the rest of the way. A field at block pitch cannot answer a
tile, and the findings register already holds that.[^13]

**No class beyond the carry was added.** A unit whose need is critical already
forages, and no verb gives a unit a home, so both would have been options that
nothing could act on.

### The gates

Every gate is green. The golden gathering scenario moved, and three of its
parameters had to be stated before it could reach the new option at all.[^14]
The panel picture moved, because the viewer lists the option set and the set
gained a row.

## References

[^1]: Findings register, FND-317. `docs/FINDINGS.md`
[^2]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D3. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D1. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decision D1. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^5]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decision D6. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^6]: Decisions register, DEC-095. `docs/DECISIONS.md`
[^7]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^8]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^9]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^10]: Findings register, FND-319. `docs/FINDINGS.md`
[^12]: Decisions register, DEC-117. `docs/DECISIONS.md`
[^13]: Findings register, FND-315. `docs/FINDINGS.md`
[^14]: Findings register, FND-320. `docs/FINDINGS.md`
