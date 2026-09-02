---
id: 0185
title: Steer a step by the option the unit chose
status: refined
created: 2026-09-02
implements: [ADR-0002 D1, ADR-0002 D2, ADR-0004 D1, ADR-0004 D3, ADR-0018 D2, ADR-0022 D1, ADR-0022 D2, ADR-0022 D3, ADR-0024 D2, ADR-0056 D2, ADR-0056 D3, ADR-0064 D1, ADR-0064 D5, ADR-0064 D6, ADR-0091 D1, ADR-0091 D2, ADR-0091 D3, ADR-0091 D4]
changes: []
creates: [ADR-0091]
serves: [PRD-0009]
blocked-by: []
---

## Why

**The engine computes a decision for every unit on every tick, and then
throws the decision away.** The movement pass reads the option column, tests
that it holds a value, discards the value, and draws a uniform direction. A
unit that chooses to forage takes the same distribution of steps as a unit
that chooses to climb.[^1]

Everything upstream of that column is paid for and unused. The level 1
rebuild, the cell summary, the need column, the option weights and the stagger
schedule all feed one bit: whether the unit moves at all.

This is the break that makes every other coupling invisible. The product
record this project points at is unmet at the action, although each of its
statements passes at the choice.[^2]

**Do not schedule this item alone.** The exit field turns a random walk into a
migration in one direction. Nothing turns the crowd back until a unit takes
what it walked to and the ground under the crowd falls. Item 0186 is that
negative feedback, and it is not decoration.[^3] Item 0187 closes the chain
from the ground to the store.[^4]

**Take items 0183 and 0184 first.** The `forage` option scores the mean stub
value of a cell, which is a random walk that no other system reads or
writes.[^5] [^6] A field derived from that quantity steers a crowd toward
noise. After 0183 and 0184 the option scores food, which the gather of item
0186 then removes.

## What the work does

1. The engine derives one exit direction for each level 1 cell and each
   option. The direction names the neighbouring cell that holds the highest
   value of the field that the option reads.
2. A cell that no neighbour beats strictly holds no exit direction.
3. The movement pass reads the option of a unit and the exit direction of the
   cell the unit stands in. It steps the unit to the neighbouring tile in that
   direction. A unit in a cell with no exit direction keeps the uniform draw
   that it takes today.

## The answers this item takes, stated plainly

**The exit field is a projection, not carried state.** The engine derives it
again at every rebuild of level 1, from the summaries that the rebuild
produced. It writes every entry, and it accumulates nothing. The field is
therefore a pure function of level 0, and the open question about a plane that
carries the state of a solver does not reach it.[^7]

**The derivation runs with the rebuild of level 1, and in one place.** The
choice pass and the movement pass both run before the rebuild of their own
frame, so both read the level 1 that the last barrier left. A field derived at
that same barrier is the field that matches the summary the choice read. The
public rebuild that a caller runs outside a frame derives the field as well,
because a field left behind by one of the two paths is a stale value that
nothing fails on.[^8]

**The field sits beside the summary and never inside it.** A summary field is
extensive, and two summaries combine by adding their fields. Two directions do
not add.[^9]

**A cell ranks its neighbours on the cell value, not on the score.** The score
multiplies the value by what one unit wants, and the multiplication saturates.
A want of zero and a saturated product both hand the answer to the tie-break.
The finding holds the evidence.[^10]

**The lowest direction index wins a tie.** The scan reads the six directions
in ascending direction index and compares strictly. That is the order every
other walk over the neighbours of a hex uses, and it is the same rule the
choice pass already uses to break a tie between two options.[^11] [^12]

**The lattice of cells is a hex grid at the pitch of one block.** The influence
field already treats it as one, so this item declares no second geometry.[^13]

**A cell moves as a block, and this item does not soften that.** Every unit of
one cell that holds one option takes one direction. Whether a watcher reads
that as a migration is a question that only a run settles, and the register
holds it open.[^14] This item adds no share of units that deviate, because
that share is a value that nothing supports.[^15]

**Admission does not change.** A cell that streams into one face can exhaust
the capacity of the tiles there. Admission refuses the surplus in the order it
already fixes, and a refused unit stays where it is. That behaviour is
existing, and a separate item holds what a refused unit should do
instead.[^16] [^17]

## Impact review

**Governed by.**

- **ADR-0091 D1 to D4.** This item creates that record and implements every
  decision in it.[^18]
- **ADR-0022 D1, D2 and D3.** Level 0 is the only truth, every level above it
  is a pure function of level 0, and no system writes above level 0. The exit
  field is derived from level 1, which is derived from level 0, and no pass
  writes a fact into it.
- **ADR-0064 D1, D5 and D6.** A unit scores a fixed option set, a tie breaks
  by the lowest index and never by a draw, and the choice reads level 1 and
  writes nothing above it. The exit field is what the choice steers, and the
  tie rule of D5 is the rule this item reuses for a tie between two cells.
- **ADR-0056 D2 and D3.** A move is an intent, and a separate admission grants
  it. This item changes what the intent names. It does not change the
  admission, and it does not let a unit skip it.
- **ADR-0004 D1 and D3.** Iteration order is explicit. The scan over the six
  directions is ascending, and the derivation over the cells names every
  result by its cell rather than by the thread that wrote it.
- **ADR-0024 D2.** An extensive field combines by its own operation. A
  direction is not extensive, which is why it is not a summary field.
- **ADR-0002 D1 and D2.** No floating point, and simulation arithmetic goes
  through the arithmetic module. A comparison between two cell values is a
  comparison of two fixed-point values.
- **ADR-0018 D2.** The derived unit structure partitions the world into
  blocks, and level 1 aggregates over the same block. The lattice of cells
  comes from there and this item declares none of its own.

**Changes.** No record changes. This item contradicts no accepted record.

**Creates.** ADR-0091, movement takes its direction from a per-cell field,
never from a per-unit search. The registry row was added before the record was
written, and the record is a draft. The author of the record is not its
reviewer.[^19]

**Blockers.** BLK-007 governs every cost figure, because no measurement exists
on the target platform. This item therefore states no budget for the field,
puts no cost figure in the code, and chooses no deviation share against a
figure.[^15]

**Open choices this item carries.** DEC-079 asks whether a cell that moves as
a block reads as a crowd, and only a run answers it. Whoever runs the
demonstration after this item reports what they saw into that row.[^14]
DEC-074 asks how the project finds a value that nothing reads, and this item
is the first work that applies its recommendation.[^20]

**Precedent.** FND-180 records that movement reads whether a unit chose and
not what it chose, and that naming the caller was not enough when the caller
discarded the payload.[^1] FND-190 records that a per-cell field and a
per-unit score search are equivalent only when the rank reads the cell
value.[^10] FND-029 records that a stale read gives a confident wrong answer,
which is why both rebuild paths derive the field.[^8]

## Done when

- The engine holds one exit direction for each cell and each option, and a
  watcher reads one entry through the public interface.
- A unit whose cell holds an exit direction for its option steps in that
  direction. A test drives the step and asserts the tile the unit reaches.
- **A test changes the cell value of one neighbour and asserts that the
  direction changes.** This is the test that the option must have, and the
  register recommends it for every value that the work writes into
  state.[^20]
- **A test pins the option column to one value and fails.** A suite that stays
  green proves that nothing reads the column, which is the defect this item
  repairs.[^1] [^21]
- A test builds two neighbours of equal value and asserts that the unit takes
  the lower direction index.
- A test builds a cell that no neighbour beats, and asserts that a unit there
  still moves, by the uniform draw.
- A test asserts that the field is identical, entry for entry, at 1, 2 and 12
  threads.
- A test derives the field twice from one level 1 and asserts that the two are
  identical, so that nothing carries between frames.
- A test rebuilds level 1 through the public path and asserts that the field
  matches the one the step derives. A field derived by one path and not the
  other fails it.
- The fixture is built for this test and is not copied from the demonstration
  world. It holds a cell with one strictly best neighbour, a cell with two
  equal best neighbours, and a cell that is its own best. The commit body says
  how that was checked: the uniform draw was put back, and each direction test
  was watched to fail.[^21]
- No floating point value appears, and every comparison goes through the
  arithmetic module.
- ADR-0091 is written, its registry row reads `Draft`, and it holds no count,
  no file table and no cost figure.
- The two determinism tests pass, and `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-180. `docs/FINDINGS.md`
[^2]: PRD-0009, a unit acts on the world it can see. `docs/product/accepted/prd-0009-a-unit-acts-on-the-world-it-can-see.md`
[^3]: Backlog item 0186, let the engine order a gather. `docs/backlog/refined/0186-let-the-engine-order-a-gather.md`
[^4]: Backlog item 0187, give a carried load somewhere to go. `docs/backlog/refined/0187-give-a-carried-load-somewhere-to-go.md`
[^5]: Backlog item 0183, carry the food of a cell into the level 1 summary. `docs/backlog/complete/0183-carry-the-food-of-a-cell-into-the-level-1-summary.md`
[^6]: Backlog item 0184, score the forage option against food. `docs/backlog/complete/0184-score-the-forage-option-against-food.md`
[^7]: Decisions register, DEC-067. `docs/DECISIONS.md`
[^8]: Findings register, FND-029. `docs/FINDINGS.md`
[^9]: ADR-0024, every summary field is declared extensive or intensive, decision D2. `docs/adrs/accepted/adr-0024-every-summary-field-is-declared-extensive-or-intensive.md`
[^10]: Findings register, FND-190. `docs/FINDINGS.md`
[^11]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^12]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^13]: ADR-0060, an influence map is stored as a shared basis, decision D1. `docs/adrs/draft/adr-0060-an-influence-map-is-stored-as-a-shared-basis.md`
[^14]: Decisions register, DEC-079. `docs/DECISIONS.md`
[^15]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^16]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^17]: Backlog item 0039, a rejected unit is not stuck. `docs/backlog/proposed/0039-a-rejected-unit-is-not-stuck.md`
[^18]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^19]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^20]: Decisions register, DEC-074. `docs/DECISIONS.md`
[^21]: Testing Rules, sections 2 and 2a. `.claude/rules/testing.md`
