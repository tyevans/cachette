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
