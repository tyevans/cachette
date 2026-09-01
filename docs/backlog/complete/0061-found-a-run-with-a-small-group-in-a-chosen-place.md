---
id: 0061
title: Found a run with a small group in a chosen place
status: complete
created: 2026-08-31
implements: [ADR-0003 D1, ADR-0004 D4, ADR-0002 D1, ADR-0068 D1]
changes: []
creates: [ADR-0075]
serves: [PRD-0012]
blocked-by: []
---

## Why

A run starts with a population somebody chose, spread by a rule with no reason
behind it. The world at tick zero is the same size as the world at tick one
million. Nothing began, so nothing can grow, no place was a good choice, and
the first hundred ticks show a full world doing what a full world does.

This item gives a run a beginning. It is the item that makes every growth rule
in this plan visible, because a rule acting on a number already at its ceiling
cannot be seen to work.

## What the work does

1. A run begins with a group whose size is an input, and the size is not the
   target population.
2. The engine chooses where the group starts by reading the world, from a
   bounded sample and never from a pass over every tile.
3. A watcher can ask which properties of the place made it the choice.
4. The storage is sized for the target and does not change during a run. The
   cost of a tick grows with the units that live.

## Impact review

**Governed by.**

- ADR-0003 D1 keys the sample draw.[^1] The candidate ordinal fills the entity
  slot, and the column and the row take different draw indices, so the two
  coordinates of one candidate do not correlate.
- ADR-0004 D4 requires that two candidate places which score the same resolve
  by a stable key.[^2] The key vector carries the score first and the tile
  index last, and the tile index is unique inside the sample.
- ADR-0002 D1 makes every score an integer or a Q16.16 value, so the
  comparison gives one answer whatever order the work ran in.[^3]
- ADR-0068 D1 states that terrain is generated from the seed and is never
  stored as a map, and the founding choice reads it under that claim.[^4] The
  level 1 rebuild already learned that a whole-world sweep of the ground is the
  design mistake that record names.[^5]

**Changes.** No record changes.

**Creates.** ADR-0075. The registry holds the row and the claim: the founding
choice reads a bounded sample of the world, never a pass over every tile.[^9]
The claim passes the three-condition test of the scope rule.[^8] A contributor
would score every tile, because that is the simple way and it gives the better
answer. The product record refuses it, and the cost of changing it later is the
time between a changed seed and the first frame, which is what a developer
notices. The reasoning is not in the code: a bounded loop looks like a choice
about a constant, and nothing in it says the constant may never become a
function of the extent.

**Blockers.** BLK-007 governs every cost figure, so this item states none.[^6]
The target population and the settlement count come from the register, and the
founding group size is an input to the run rather than a value this item
invents.[^6] [^7]

**Serves.** PRD-0012.

**Conflict surface.** `crates/cachette-core/src/founding.rs` is new. It reads
`crates/cachette-core/src/terrain.rs` and
`crates/cachette-core/src/resource.rs`, and it writes through the settlement
and the soldier arenas. `crates/cachette-core/src/world.rs` gains the founding
calls. `crates/cachette-view/src/main.rs` founds a run instead of spreading
soldiers over the world. **It is the only item in this plan that changes the
world constructor**, so it merges alone with respect to that function.

## The three questions this item had to settle

**The registry row.** Settled. The argument above holds and the record is
written. The registry row is allocated and the record is a draft.[^9]

**What every existing fixture does.** Settled in the decisions register as
DEC-030.[^10] The founding is one of two ways to people a world, not the only
one. The direct spawn stays, because the founding is built on it, because a
test must be able to place a unit where the test chooses, and because a
re-recorded golden file proves nothing about the change that caused it. No
existing fixture changes and no existing golden file moves. The founding adds
one scenario and one golden file, and the old files stay as the control.

The extent of the new fixture is wider than the coarsest lattice spacing of the
generator, because a world narrower than that spacing holds one kind of
ground.[^11] A founding in such a world finds no good place, and the test then
measures the fixture.

**What a founding score reads.** Settled in the decisions register as
DEC-031.[^10] The score reads the ground and the stock the ground carries,
because those are the only properties a world has before it steps.

**Point six of item 0050.** Confirmed by reading every shaped record, not by
repeating the sentence.[^12] No shaped record requires a run to begin at its
target population. Two records describe a full starting population, PRD-0011
and PRD-0012, and both describe it as the present defect rather than as the
requirement. Every cost statement in the shaped records is a growth shape. The
one literal population count is in PRD-0005, which says that the world holds
one million units at the target scale; that statement is scoped to the target,
so a run of thirty people leaves it describing a case not yet reached. No
amendment is needed and no record is wrong.

## Done when

- A run begins with a group of a size given as an input.
- The engine chooses the place by reading the world, and the choice costs a
  bounded sample. A test asserts that the cost does not grow with the world.
- A watcher asks why the place was chosen and gets an answer from the engine.
- A different seed gives a different place, and the new place answers the same
  test. A property test asserts it over many seeds.
- Two places that score the same resolve by the stable key, and a test
  constructs the tie rather than hoping for it.
- A group founded in a poor place does worse than one founded in a good place,
  and a test asserts the difference on a quantity the test computes itself.
- The world reserves its storage once and does not grow it during a run.
- A property test asserts that the same seed gives the same place and the same
  group at 1, 2 and 12 threads.
- The fixture is large enough to hold more than one lattice cell, and the
  commit body names the extent and why.[^11]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

**Done.** A run begins with a group of a size the caller gives, in a place the
engine chose from a bounded sample. The demonstration binary founds a run of
thirty people instead of spreading soldiers over the world.

**What the founding does not do.** It gives the group nothing to do. The
founded people walk at random, exactly as the spawned soldiers did, because
behaviour is a separate item.[^13] That is the correct outcome for this item.

**The three decisions.** ADR-0075 holds the bounded sample claim, as a draft.
DEC-030 keeps the direct spawn beside the founding, so no existing fixture and
no existing golden file changed. DEC-031 records the properties the score
reads. A fourth decision was found during the work and opened as BLK-018: how
many groups found a world, and whether every faction founds one. The engine is
written parametrically under it.

**What the tests cover.** Six defects were restored in the source and the
suite was run against each. Five were caught at once. The sixth, a score that
reads nothing the ground holds, was **not** caught, because the poor-place
test compared one pair and the pair still ordered correctly by luck of the
sample. That is the uniform-input shape.[^14] The test now ranks every
eligible place by a stock the test computes for itself and requires the chosen
place to sit in the best quarter, and it then catches the defect.

**Two findings.** FND-070 records that a restored defect must be the smallest
change that violates the claim, because the first attempt made the suite
unrunnable. FND-071 records that the whole-world pass the pyramid gave up was
still live in the demonstration binary.

**Three items raised.** 0093 asks the panel to say what the founding chose.
0094 asks how many groups found a world. 0095 asks the founding for a count of
who already stands on a tile.

## References

[^1]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/accepted/adr-0068-terrain-is-generated-from-the-seed-and-is-never-stored-as-a-map.md`
[^5]: Backlog item 0042. `docs/backlog/complete/0042-build-level-1-of-the-pyramid.md`
[^6]: Blockers register, BLK-003, BLK-005 and BLK-007. `docs/BLOCKERS.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: ADR Registry, row 0075. `docs/adrs/REGISTRY.md`
[^10]: Open decisions register, DEC-030 and DEC-031. `docs/DECISIONS.md`
[^11]: Findings register, FND-054. `docs/FINDINGS.md`
[^12]: Product requirement records. `docs/product/shaped/`
[^13]: Backlog item 0064. `docs/backlog/complete/0064-choose-an-action-by-scoring-a-fixed-option-set.md`
[^14]: Testing rules, section 2a. `.claude/rules/testing.md`
