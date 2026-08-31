---
id: 0061
title: Found a run with a small group in a chosen place
status: proposed
created: 2026-08-31
implements: [ADR-0003 D1, ADR-0004 D4, ADR-0002 D1, ADR-0068]
changes: []
creates: []
serves: [PRD-0012]
blocked-by: [0052]
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

**Governed by.** ADR-0003 D1 keys the sample draw.[^1] ADR-0004 D4 requires
that two candidate places which score the same resolve by a stable key.[^2]
ADR-0002 D1 makes every score an integer or a Q16.16 value, so the comparison
gives one answer whatever order the work ran in.[^3] ADR-0068 states that
terrain is generated from the seed and is never stored as a map, and the
founding choice reads it under that claim; the level 1 rebuild already learned
that a whole-world sweep of the ground is the design mistake that record
names.[^4] [^5]

**Blockers.** BLK-007 governs every cost figure, so this item states none. The
target population and the settlement count come from the register, and the
founding group size is an input to the run rather than a value this item
invents.[^6] [^7]

**Serves.** PRD-0012.

**Conflict surface.** `crates/cachette-core/src/founding.rs` is new. It reads
`crates/cachette-core/src/terrain.rs` and writes through the settlement and
soldier arenas. `crates/cachette-core/src/world.rs` at the constructor.
**It is the only item in this plan that changes the constructor**, so it
merges alone with respect to that function, and every scenario fixture in the
tree changes with it.

## What is missing before this is refined

**The registry row.** This work states a constraint that no reserved row
holds: **the founding choice reads a bounded sample of the world, never a pass
over every tile.** All three conditions of the scope rule hold.[^8] Scoring
every tile is the obvious implementation and it is what a contributor would
write; PRD-0012 rejects it, and the project has already paid for the same
mistake once in the level 1 rebuild.[^5] The cost of changing it later is the
time to the first frame, which is the thing a developer notices. **Allocate
the row in the registry before writing the record.**[^9]

**What every existing fixture does.** Each golden scenario and each property
fixture spawns its units directly. If a founding replaces that, every fixture
in the tree changes and every golden file is re-recorded. Whether the founding
is the only way to people a world, or one of two, is a decision this item must
take before it starts, not during. **FND-054 records that a fixture smaller
than the terrain lattice spacing holds one terrain**, so a founding fixture
that is too small will find no good place and the test will measure the
fixture.[^10]

**Point six of item 0050.** Several shaped records reason about a world that
already holds its target population, and this item makes the early ticks a
different shape.[^11] Every cost statement in those records states a shape and
not a number, so none of them is wrong. **Confirm that by reading them, rather
than by repeating this sentence.** That confirmation is part of this item's
impact review and it is one of the reasons the item is not refined.

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
  and a test asserts the difference.
- The world reserves its storage once and does not grow it during a run.
- A property test asserts that the same seed gives the same place and the same
  group at 1, 2 and 12 threads.
- The fixture is large enough to hold more than one lattice cell, and the
  commit body names the extent and why.[^10]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^2]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^3]: ADR-0002, simulated and aggregated state holds no floating point number. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^4]: ADR-0068, terrain is generated from the seed and is never stored as a map. `docs/adrs/REGISTRY.md`
[^5]: Backlog item 0042. `docs/backlog/complete/0042-build-level-1-of-the-pyramid.md`
[^6]: Blockers register, BLK-003 and BLK-005. `docs/BLOCKERS.md`
[^7]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^9]: ADR Registry. `docs/adrs/REGISTRY.md`
[^10]: Findings register, FND-054. `docs/FINDINGS.md`
[^11]: Backlog item 0050. `docs/backlog/proposed/0050-close-the-gaps-the-product-shaping-opened.md`
