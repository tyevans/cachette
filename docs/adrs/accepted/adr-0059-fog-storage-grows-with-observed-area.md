# ADR-0059: Fog storage grows with observed area, not with world area

## Context

A faction must see only what it observes. An accepted product record states
that need. A faction sees a tile when one of its own units observes it. A
faction that never observed a tile reads nothing about it. A faction that
observed a tile and moved away reads what it last saw, and the engine marks
that reading as remembered.[^1]

**Nothing in the engine implements this today.** No column, no plane and no
table holds what a faction has observed. Every reader answers the truth of the
world. The reader that counts each subsystem counts the whole world and carries
no faction column at all.[^2]

An accepted record already binds the reader. A caller passes a faction and
receives what that faction sees. No argument asks for the truth, and the
engine applies the rule inside the reader. **That record decides the reader and
states plainly that it does not decide the storage.**[^3] This record decides
the storage.

Five forces fix the shape.

**No field of the world is indexed by the faction.** A record forbids a plane
whose extent is the world multiplied by the faction ceiling.[^4] A bit for each
tile for each faction is exactly that plane. The product record refuses it for
the same reason, from the need rather than from the storage: a design that pays
for unseen tiles fails the need.[^1]

**A summary must not leak what the tiles beneath it hide.** The product record
asks for the rule at every level, not at the tile alone.[^1] The pyramid is a
pure function of level 0, so a faction view of a cell must derive from a
faction view of its tiles.[^5]

**Some of the answer is in the present frame and some is not.** The engine
derives the presence relation at the end of every step and stores no presence
fact, because every part of that answer is already in the world.[^6] What a
faction sees now has the same shape. What a faction saw once does not. A memory
of ground that no unit of the faction now watches is in no frame.

**A learner reads this before it reads anything else.** An accepted product
record takes the sight rule whole and makes it the first checkable statement of
the training harness.[^7]

**A research report measured three designs and recommends one.** The report
compares a sparse tile set for each faction, a tree of shared masks, and one
transposed grid that gives each tile a faction mask. It rejects the second and
the third, and it gives the arithmetic for each.[^8] This record takes that
recommendation and does not repeat the evidence.

## Decision

**A faction observes from its own live units. What it sees now is derived each
step. What it saw once is remembered in a container whose size follows the
tiles observed. Every reader answers for one faction.**

### D1. A faction observes from its own live units, and from nothing else

A live unit of a faction observes the tiles inside its sight, and it is the
only thing that gives that faction sight.[^1] A settlement gives no sight.
Ground a faction holds gives no sight. An upgrade gives no sight. A faction
with no live unit observes nothing, whatever it holds.

**Reject a settlement, held ground or an upgrade as a source of sight.** A game
may reasonably want a city to watch its own walls, and a future contributor
will ask for it. The accepted product record states the checkable rule as the
units of a faction, and the research this record rests on argues no other
source.[^1] [^8] A record that widens the rule needs a product record that asks
for the widening.

The sight of a unit is a whole number of hex steps.[^9] The engine rounds that
number to one of a fixed set of values before it computes anything, so units
that stand on one tile at one rounded radius share one answer. This record states no
radius. The sight of each unit type, the rounded set, and the largest sight the
engine admits are balance values, and the balance register is where a balance
value belongs.[^10] No record may state one.

Ground blocks sight. The engine computes the tiles a unit observes by a
shadowcast over the six sextants of the hex grid, so ground that blocks sight
hides everything behind it.[^8]

A reviewer finds a violation when anything other than a live unit of the
faction adds a tile to what that faction sees, when a sight radius is not a
whole number, or when a unit sees through ground that blocks sight.

### D2. The record of observation is a block-adaptive tile set, and no layer is a dense bitmap over the world

Each faction that holds a bit in the faction mask carries two layers.[^4] One
layer names the tiles the faction sees now. One layer names the tiles the
faction has ever seen.

Each layer is an array of blocks. The world divides into blocks of a fixed
size, and a layer holds one entry for each block. The block is the block the
pyramid aggregates over, so a fog layer and a summary share one lattice.[^5]
Each block holds one of four forms. A block holds no payload when the faction
sees no tile in it. A block holds a sorted array of offsets when the faction
sees few. A block holds a bitmap when the faction sees many. A block holds no
payload again when the faction sees every tile in it. The threshold is the
population at which the array and the bitmap cost the same bytes. It is a
derived cost figure, so the cost register is where it belongs, and this record
states no value.[^11] The block order is the block number, and the offsets inside
a block ascend.

**Reject a dense bitmap over every tile for each faction.** This is the obvious
choice, and it is the choice this record exists to refuse. A dense bitmap costs
the world tile count for every faction that exists, whether that faction
observes the whole world or nothing at all.[^11] It is a field of the world
indexed by the faction, which a record forbids.[^4] It fails the product
record, which asks that adding a faction that observes little costs
little.[^1] The block form costs the tiles a faction observes, plus one header
for each block, so a faction that observes one valley pays for one valley. At
the extreme, a faction that observes the whole world costs what the dense
bitmap costs and no more, and its remembered layer collapses to the
payload-free form.[^8]

**Reject a transposed grid that gives each tile a faction mask.** That grid
holds the same bits in a different order, so it saves nothing, and it costs the
same fixed extent in a game of two factions as in a game at the faction
ceiling.[^8]

A reviewer finds a violation when a fog layer allocates for a tile that no
faction observes, when a layer allocates on the creation of a faction rather
than on its first observation, or when the offsets inside a block are unsorted.

### D3. What a faction sees now is derived. What a faction saw once is remembered

**The layer of what a faction sees now is derived, and the step rebuilds it.**
The units, their tiles and their sight are in the frame, so the layer is a pure
function of the frame. The rebuild runs at the end of the step, after the last
change to a unit position, as the presence relation does.[^6] The rebuild
visits only the blocks that a unit entered or left, and it rebuilds each of
those blocks from the units that observe it now. Nothing counts how many units
cover a tile.[^8]

**The layer of what a faction saw once is remembered, and the step carries it
forward.** This layer is not a function of the frame. A faction that saw a
valley and marched away holds a fact that no present unit produces, and no
derivation returns it. The step adds the tiles seen now into the remembered
layer and never removes one. The layer therefore only grows, and it is the one
part of fog that is state.

**The remembered layer does not contradict the record that level 0 is the only
truth.** That record forbids a fact that lives above level 0, and it asks every
level above level 0 to be an exact function of the level below.[^5] The
remembered layer lives at level 0, so the second rule never reaches it. The
first rule asks that a fact be stored once, and the remembered layer is stored
once. Nothing in that record asks a level 0 fact to be a function of the
present frame. A stockpile is not one either.

**A memory is not a derived level, because no level beneath it holds one.** A
derived level answers from the level below it. No level below the remembered
layer holds what a faction saw. A rebuild from the ground of level 0 therefore
returns the ground as it stands now, and never the ground as the faction last
saw it. That is the whole of the split this decision makes. The fog rebuild is
the one mechanism that writes the projections below, and no simulation system
writes one.[^12]

Three projections follow from the two layers, and nothing else holds them. A
faction mask for each level 1 cell says which factions see that cell. A second
mask for each level 1 cell says which factions have ever seen it. A mask for
each unit says which factions see that unit. Each is a union of sets, and a
union is exactly associative and commutative, so a fold gives one answer
whatever the grouping.[^13] Shared sight is derived from a relation row and
stored nowhere, in the shape a record already fixes for a relation between
factions.[^14]

A reviewer finds a violation when the step removes a tile from the remembered
layer, when a pass reads the remembered layer to answer what a faction sees
now, or when anything but the rebuild writes a derived mask.

### D4. A remembered place answers with its ground and with no unit

A faction reads a place it has seen and does not see now. The reader answers
with the ground of that place as the faction last saw it. The reader answers
nothing about the units that stand there now, nothing about the holder as it
now stands, and nothing about what the place now holds. The reader marks the
answer as remembered, so a caller tells a memory from a sighting.

A faction reads a place it has never seen. The reader answers nothing at all.

**This binds every reader, and not the fog module alone.** A reader that names
a faction resolves each place against the two layers of that faction before it
answers. It answers the present value for a place the faction sees now, the
remembered ground for a place the faction saw once, and nothing for a place the
faction has never seen. A summary reader combines the values that the same rule
admits for the tiles beneath it, so a cell cannot state what its tiles
hide.[^1]

A reviewer finds a violation when a reader that names a faction reports a unit
on a tile that faction does not see now, when such a reader reports a present
value for a remembered place, or when a summary for a faction counts a tile
that faction has never seen.

### D5. The remembered layer enters the state hash, holds no float, and every parallel write is disjoint

The remembered layer is state, so the hash of the world covers it. The hash
walks the factions in faction order and the blocks in block number order, and
it hashes the payload of each block. A layer outside the hash lets two worlds
that differ hash the same, and the golden state test then passes on a world
that is not the golden one.[^15]

No value in either layer is a floating point number. A layer holds tile
offsets, bit words, populations and masks, and every one of these is an
integer.[^16] Every count over a layer widens, so no count depends on the
margin of a narrow type.[^17]

The rebuild gives each worker a disjoint set of blocks. Two workers therefore
never write one block, and the update needs no atomic operation.[^18] The
partition comes from the block count and the thread count, and never from the
schedule.[^18] Nothing reads which worker finished first. Fog draws no random
number anywhere.

A reviewer finds a violation when a fog value is a float, when a fog value the
step carries forward sits outside the hash, when two workers may write one
block, or when a fog result takes its order from a thread.

### D6. Every reader at the boundary answers for one faction, and a world-wide reader is not a fog reader

A reader that a faction may act on takes a faction and answers for that
faction. It offers no argument that asks for the truth, as an accepted record
already requires.[^3]

**A reader that counts the whole world is not a per-faction reader, and nobody
may read it as one.** The subsystem census counts each subsystem over the whole
world and carries no faction column today.[^2] It serves a developer who
watches the engine, and it stays that reader.

A per-faction form of a world-wide count is a separate reader. This record
states what such a reader must satisfy, and it does not describe one as if it
existed. Such a reader takes a faction. It counts only what that faction sees
now, or only what that faction has ever seen, and it names which of the two it
counted. It never counts a subject the faction has not observed. It reaches the
control plane as an array, and never as a call for each subject.[^19]

A reviewer finds a violation when a reader that names a faction has an argument
that widens its answer, when a world-wide count reaches an observation a
faction reads, or when a per-faction count includes a subject the faction has
not observed.

## Consequences

**The project can no longer give every faction a fixed fog cost.** A faction
that observes widely costs more than a faction that observes little, so the
memory of a world now depends on how the factions play it. A plan that wants
one figure for every faction cannot have one.

**The project can no longer answer "which factions see this tile" with one
load.** That question now costs one lookup for each faction at level 0. The
derived level 1 mask answers the cell form of the question, and the per-unit
mask answers the form the engine actually asks.[^8] A contributor who wants the
level 0 form must reopen this record with a measurement, and the report states
the trigger that admits it.[^8]

**The remembered layer is state, so it enters every golden file.** A change to
the layer form, to the block order, or to the faction mask width invalidates
every stored hash and every recorded replay.[^15] An accepted record fixes the
mask width, and this record does not move it.[^20]

**A faction with no live unit goes blind and keeps its memory.** D1 gives sight
to units alone, so a faction that loses its last unit sees nothing, while every
tile it ever saw stays in its remembered layer. A game that wants a city to
watch its own walls must put a unit there.

**Every reader that names a faction gains work.** Such a reader now resolves
each place against two layers before it answers. A reader that answers the
world does not change.

**A caller must choose which world it reads.** The demonstration and the
balance harness read the truth of the world today. Each must now state whether
it keeps reading the truth or reads the view of one faction. A watcher who
debugs the engine wants the truth. A harness that measures whether a game is
fair between factions may want either, and reading the truth while the
controllers play under fog measures a game that nobody plays. This record
decides neither, and it permits neither to stay unstated.

**No figure behind this record is measured.** The shape of the growth is the
requirement, and every cost figure for it is derived on the target platform. A
blocker holds that gap.[^21]

## References

[^1]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^2]: The world module of the core crate, the subsystem census. `crates/cachette-core/src/world.rs`
[^3]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables the engine owns, decision D3. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^4]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^5]: ADR-0022, level 0 is the only truth, and every level above it is derived, decisions D1 and D2. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^6]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decisions D1 and D2. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^7]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^8]: Research report 08, fog of war representation. `docs/research/reports/08-fog-of-war-representation.md`
[^9]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^10]: Balance register. `docs/reference/balance.md`
[^11]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^12]: ADR-0022, level 0 is the only truth, and every level above it is derived, decision D3. `docs/adrs/accepted/adr-0022-level-0-is-the-only-truth-and-every-level-above-it-is-derived.md`
[^13]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^14]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D7. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^15]: ADR-0001, one binary gives one answer at any thread count, decisions D1 and D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^16]: ADR-0002, simulated and aggregated state holds no floating point number, decisions D1 and D2. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^17]: ADR-0002, simulated and aggregated state holds no floating point number, decision D3. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^18]: ADR-0009, parallel stages write disjoint outputs, decisions D1, D2 and D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^19]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^20]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^21]: Blockers register, BLK-007. `docs/BLOCKERS.md`
