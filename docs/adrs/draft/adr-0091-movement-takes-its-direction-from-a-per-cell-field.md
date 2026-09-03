# ADR-0091: Movement takes its direction from a per-cell field, never from a per-unit search

## Context

A unit chooses one option from a small fixed set. It scores each option
against the level 1 cell it stands in, and the engine writes the winner into a
column.[^1] A level 1 cell summarises one block of tiles.

The movement pass reads that column, tests that it holds a value, and discards
the value. It then draws a uniform direction. A unit that chose to forage
takes the same distribution of steps as a unit that chose to climb. The
findings register holds this.[^2]

The product record that this project points at asks that a unit acts on the
world it can see. It asks that a watcher changes the world and sees the
behaviour change.[^3] The choice changes today. The step does not.

Something must turn an option into a direction. Two shapes do it.

The first shape gives each unit a search. The unit reads the six neighbouring
cells, scores each one on the field its option reads, and steps toward the
best. The cost follows the population.

The second shape gives each cell a direction. The engine ranks the neighbours
of every cell once, for each option, and a unit reads one entry. The cost
follows the cell count. This record calls that array the exit field.

The orientation states the preference in general terms. A set-valued command
permits a cheaper algorithm, and it names a field over a set against many
searches.[^4] A general preference is not a decision. It does not say what the
field holds, what fixes a tie, or which invariant governs the field.

The lattice of cells is already a grid. The derived unit structure partitions
the world into blocks, and level 1 holds one cell for each block.[^5] The
influence field solves over that same lattice, at the pitch of one block.[^6]
The six neighbour offsets of a tile therefore name the six neighbours of a
cell as well.

The registry retired one number that held a claim of this shape.[^7] It
described a portal graph and a flow tile cache for a long path, before any path
search existed and before any record asked for a long path. This record is
narrow against that precedent. It governs one step to one
neighbouring tile, for an option that the engine already computes.

## Decision

### D1. Movement takes its direction from a per-cell field, and never from a per-unit search

The engine derives one exit direction for each cell and each option. A unit
that holds an option reads the entry of its cell and its option. It steps to
the neighbouring tile in that direction.

**No unit reads a neighbouring cell. No unit scores a neighbour.**

**An option that ranks no cell field takes its direction from another
per-cell field.** This decision binds where a direction comes from, and it
does not bind every option to this one array. An option that ranks the state
of the unit itself has no summary field to rank a neighbour on, so the exit
field holds no entry for it and a separate per-cell field steers it. That
field is one plane for each faction over the same lattice, and a later record
states it.[^26] The rule that a unit reads one entry and searches nothing is
unchanged.

The ranked quantity belongs to the cell and not to the unit. Every unit of one
cell reads one summary, so every unit of one cell ranks the neighbours in one
order. A per-unit search computes that one order again for each unit.

### D2. The exit field is a projection of level 0, and it carries nothing between frames

The engine derives the field again at every rebuild of level 1, from the
summaries that the rebuild produced. It writes every entry. Nothing
accumulates. A field that is thrown away and derived again holds the same
value.

Level 0 stays the only source of truth, and the field states no fact of its
own.[^8] It is a pure function of level 0, in the way that level 1 is.[^9]

A unit reads the option, the summary and the field in one frame, and all three
come from one barrier. A field derived at one barrier and read against another
lets a unit act on ground that it never saw.

### D3. The exit field is not a summary field

A summary field is extensive. Two summaries combine by adding their fields,
and an intensive reading is a division of two stored fields at read time.[^10]
[^11]

A direction is neither extensive nor intensive. Two directions do not add. The
field therefore sits beside the summary and never inside it, and no reader
combines two of its entries.

### D4. A cell ranks its neighbours on the field value, and the lowest direction index wins a tie

The rank reads the value that the option reads from a cell. It does not read
the score of the option.

A score is that value multiplied by what the unit wants.[^1] Two properties
make the score the wrong key. A want of zero makes every score equal. The
multiplication saturates, so two different values can give one score. Under
either property the tie-break decides the direction, and the ground does not.

The scan reads the six directions in ascending direction index and compares
strictly. The lowest direction index therefore wins a tie. That direction
order is fixed, and every other walk over the neighbours of a hex uses it.[^12]

A neighbour outside the lattice is not a candidate.

**The scan starts at the value of the cell itself.** A neighbour must beat the
ground the unit already stands on, so a cell that no neighbour beats holds no
exit direction. A unit in such a cell keeps the uniform draw that it takes
today, so the field leaves no unit without a rule.

A scan that started below every value would give a direction to a cell that is
already the best of its neighbourhood, and every unit there would step onto
worse ground for ever.

### D5. A cell that admits no unit is not a candidate

No summary field says whether a unit may stand in a cell. A cell of open water
therefore competes on the same terms as dry ground, and it wins wherever the
ranked field happens to favour it. The mean height of a cell is such a field:
the water of one cell may be shallower on average than the water of the cell
beside it, so the closed cell reads higher and takes the direction.

A whole block is then sent at ground that admits nobody, and every unit of it
is refused at the tile in front of it. The rank therefore drops a neighbour
whose open tile count is zero, before it compares any value.

**The rule reads the open tile count, and it states no second rule of its
own.** That count is the same one the open share reads, and it is derived from
the capacity table.[^20] [^21] A passability test written here would be that
fact in a second place.[^22]

**The rule refuses a cell that admits nobody at all, and it refuses no other.**
A cell that admits somebody stays a candidate, whatever the shape of the ground
inside it. The field holds one direction for a whole block, and which tile a
unit stands against is not a fact that a block carries. D6 is what answers that
case.

### D6. A direction the ground refuses falls back to a keyed draw, and it never freezes a unit

A cell holds one direction for a block of tiles. The ground under one unit of
that block may refuse it, and D5 does not remove that case: a cell that admits
somebody may still hold water at the tile one unit stands against.

**The refusal repeats.** The cell, the option and the direction all hold from
one frame to the next, so a unit that only stayed put would stay put for ever.
A unit against a shoreline is the case that showed it, and the findings
register holds the measurement.[^23]

A unit whose direction the ground refuses therefore takes a draw from the
counter-based generator, at the next draw index of the same system and frame.
The draw is keyed on the frame, so a unit the draw refuses again takes a
different direction on the next frame.[^24] A unit that both directions refuse
stays put for that frame alone.

This does not give a unit a search. The unit reads one tile, which is the tile
it would step onto, and the engine already read that tile to answer the ground.
No unit reads a neighbouring cell and no unit scores one, so D1 holds.

**The fall-back takes its own draw index.** A fall-back that reused the index
of the first draw would hand the refused unit the direction that was just
refused, and the unit would freeze exactly as before.

## Consequences

**A cell moves as a block.** Every unit of one cell that holds one option
takes one direction. Whether a watcher reads that as a migration is a question
that only a run settles, and the decisions register holds it open.[^13]

**The engine gains one array, indexed by the cell and by the option.** It
costs the cell count times the option count, and it does not cost the
population. No figure appears here, because one blocker holds the
cost figures this record would state.[^14]

**A unit no longer steps in a direction that its own reading chose.** The
engine cannot give one unit of a cell a different direction from another,
because the mechanism that would do it is the search that D1 forbids.

**Admission does not change.** A cell that streams into one face can exhaust
the capacity of the tiles there. Admission refuses the surplus, in the order
that it already fixes, and a refused unit stays where it is.[^15] [^16]

**An option added later must state what a cell ranks on.** An option that
names no cell field cannot steer a step.

**A unit near ground that admits nobody moves less predictably than one inland.**
D6 hands it a uniform draw whenever the field points at ground that refuses it,
so the option it chose stops steering it there. The field says which way the
block should go, and it cannot say how one unit gets around the tile in front
of it. A rule that routes a unit around an obstacle is a different claim, and
it needs a field that reaches further than one neighbour.

**A caller that changes level 0 outside a frame derives the field again with
level 1.** A stale derived value is a confident wrong answer, and this project
has recorded that cost already.[^17]

## Alternatives rejected

**Give each unit a search over the six neighbouring cells.** This is the shape
a contributor reaches for first, because the movement pass already reads one
unit at a time. It is rejected on cost, and because the equivalence in D1 is
invisible inside such a loop. A reader sees six reads for each unit. A reader
cannot see that the same six reads repeat for the unit standing beside it.

**Store the direction inside the cell summary.** It puts the value where every
reader already looks. It is rejected because the summary combines by addition,
and a fold of two summaries would then produce a direction that means
nothing.[^10]

**Rank the neighbours by the score of the option.** It matches the choice pass
exactly, so the two would agree by construction. It is rejected for the two
properties in D4: a want of zero and a saturating multiplication both hand the
answer to the tie-break.

**Carry the field between frames and relax it.** A carried field reaches
further than one derivation. That is what the project chose for the reach of a
faction. It is rejected here because an exit direction needs no reach. It
answers one step to one neighbour. A carried value would also state a fact
that appears nowhere at level 0, and the register holds that question open
against the influence field alone.[^18]

**Let a refused unit stay put.** This is what the engine did, and it costs no
draw. It is rejected because the refusal repeats: every input to the direction
holds from one frame to the next, so the unit is not delayed by one frame, it
is stopped for ever.[^23]

**Let the rank drop a neighbour by the share of it that admits a unit, rather
than by whether any of it does.** It would steer a block away from a coast
rather than only away from open water. It is rejected because the share is a
value that an option already ranks on, and a rule that mixed a threshold on one
field into the rank of every field would decide by a constant that no record
holds.[^25]

**Scan the six neighbouring tiles for one the ground admits.** It would move a
refused unit every frame rather than most frames. It is rejected on cost: it is
a per-unit search over a neighbourhood, which is the shape D1 exists to refuse,
and the movement pass runs for every unit of the population.

**Break a tie with a keyed draw.** A draw would spread a crowd across two
equal neighbours. It is rejected because the project already refuses a draw
for the tie between two options, for the same reason: a draw makes the answer
depend on a key that carries no meaning.[^19]

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: Findings register, FND-180. `docs/FINDINGS.md`
[^3]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^4]: Project orientation, the design principles. `CLAUDE.md`
[^5]: ADR-0018, the unit-to-tile bridge is derived, and it rebuilds at the barrier, decision D2. `docs/adrs/accepted/adr-0018-the-unit-to-tile-bridge-is-derived-and-rebuilds-at-the-barrier.md`
[^6]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^7]: ADR Registry, the retired numbers. `docs/adrs/REGISTRY.md`
[^8]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D1. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^9]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^10]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^11]: ADR-0024, every summary field is declared extensive or intensive, decision D3. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^12]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^13]: Decisions register, DEC-079. `docs/DECISIONS.md`
[^14]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^15]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^16]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D5. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^17]: Findings register, FND-029. `docs/FINDINGS.md`
[^18]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^19]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^20]: ADR-0024, every summary field is declared extensive or intensive, decision D4. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^21]: ADR-0068, terrain is generated from the seed and is never stored as a map, decision D4. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^22]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^23]: Findings register, FND-315. `docs/FINDINGS.md`
[^24]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^25]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^26]: ADR-0108, a unit returns by climbing a reach field seeded at every site of its faction, decision D1. `docs/adrs/draft/adr-0108-a-unit-returns-by-climbing-a-reach-field.md`
