---
id: 0066
title: Provide the character column set and the tier declaration
status: complete
created: 2026-08-31
implements: [ADR-0066 D1, ADR-0066 D3, ADR-0012 D3, ADR-0014 D1, ADR-0014 D3, ADR-0004 D4, ADR-0007 D1]
changes: []
creates: [ADR-0054]
serves: [PRD-0011, PRD-0015, PRD-0016]
blocked-by: []
---

## Why

Every unit is interchangeable, so a watcher cannot follow one. A story needs
somebody it is about, and an interchangeable unit cannot be that.

The accepted storage record names the living character as one of the four
shapes, and nothing implements it.[^1] Descent and a ruler both need it, and
neither can start without it. This item builds the shape and the rule that
decides who is in it. It builds no relation and no succession.

## What the work does

1. A character arena exists, with the same identity rule as the soldier arena
   and none of the soldier's columns.
2. An entity declares its tier when it is created, and the tier is a static
   property of the shape rather than a count checked at call time.
3. A soldier who crosses an achievement bound is promoted into the character
   tier by one rule: filter, sort, then allocate against a budget.
4. A promoted soldier gets no invented ancestry.
5. There is no demotion. A character whose unit ends stops being embodied and
   keeps its identity.

## Impact review

**Governed by.**

- ADR-0066 D1. A living character carries no tile position and none of the
  soldier's columns. That is the reason for a separate shape, and this item
  must not give it one.
- ADR-0066 D3. The shape does not vary at run time.
- ADR-0012 D3. The character lives in the generational arena.
- ADR-0014 D1 and D3. A character identity is a slot and a generation, and a
  dead character never hands its identity to the next one in the slot.
  **PRD-0015 states this as a requirement of the descent record**, so it is
  settled here rather than there.
- ADR-0004 D4 and ADR-0007 D1. The promotion sort takes a key vector and never
  a comparison function. The research reaches the same rule independently: a
  content-supplied comparator can be intransitive, and an intransitive
  comparator makes the output depend on the sort algorithm, which no tie-break
  on the identifier repairs.[^2] [^3] [^4] [^5] [^6]

**Changes.** No record changes.

**Creates.** ADR-0054. The registry reserves the row and states the claim: an
entity belongs to one of three tiers, declared at creation.[^7] This item
writes it. The claim passes the three-condition test, and the reasoning is the
part that must be written down: **enforcing the tier with a count at call time
makes the same script work on a small world and fail on a large one**, which
is the worst available failure mode and is invisible in the code.[^2]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-004 is resolved: the target is 50,000 living characters and the hard
ceiling is 262,144, both in the scale constants table and read rather than
invented.[^8] [^9] **BLK-011 is resolved and it is binding here**: a promoted
soldier gets no invented ancestry, founds a new line, has a relation of zero to
everybody, and cannot inherit by blood, though he may be appointed.[^10] This
item implements that resolution and item 0068 reads it.

**Precedent.** FND-007 records that the promotion and demotion problem does not
exist, and FND-031 records that opinion state belongs to the character tier and
not to every unit.[^11] [^12] FND-022 records that characters are stored as an
array of structures and cohorts as a structure of arrays, which bears directly
on the layout of this arena.[^13]

**A decision this item must take and not assume.** The promotion scan is
correct only if the achievement value never falls, because the scan reads a
level and not an edge.[^2] **That is a constraint on the content, not an
implementation detail.** State it, and check it in a debug build, or the scan
misses a promotion silently.

**Serves.** PRD-0011 for a unit a watcher can follow, and PRD-0015 and
PRD-0016 as the tier they both need.

**Conflict surface.** `crates/cachette-core/src/character.rs` is new.
`crates/cachette-core/src/lib.rs`, `crates/cachette-core/src/soldier.rs` at an
achievement column and a character back-reference, and
`crates/cachette-core/src/world.rs` at the state hash and the invariant check.
**It is independent of every item from 0053 to 0065** except for the shared
edit to `world.rs` and `soldier.rs`, so **it is the one item in this plan that
a second worker can start on day one, beside item 0052.**

## Done when

- A character arena exists, and a character carries no tile position.
- An entity declares its tier at creation, and a shape that is not one of the
  four is a compile-time error.
- The declared ceiling is checked when the world is built, not at each call,
  and a test asserts the refusal above it.
- A soldier who crosses the bound is promoted by filter, sort and a budget,
  and the budget never lets the population exceed the ceiling.
- The sort goes through the key vector interface, never through a comparison
  function.
- A promoted soldier has no parents, has a relation of zero to every existing
  character, and founds a new line. A test asserts all three.
- A character whose unit ends keeps its identity, and there is no demotion. A
  test asserts that the character survives the unit.
- A debug build fails when the achievement value falls, and a test proves that
  it fails.
- A property test asserts that the promotions are identical, and in the same
  order, at 1, 2 and 12 threads.
- ADR-0054 is written, the registry row moves to `Draft`, and the record holds
  no character count and no cost figure. Those are in the reference
  tables.[^9]
- `just check` runs green.

## Outcome

The living character column set exists, and the tier declaration exists. A
character holds a generational identity, a faction, the tick they were created
on, and a renown. The world builds the arena, hashes it, and checks it.

**A character carries no tile position.** The arena holds no grid and no tile
column. It builds without a world shape, which is the test that states the
absence, because a test cannot assert that a method is missing.

**The identity rule holds.** The generation advances when the arena frees the
slot, so a character loses their identity at the moment they are lost. A test
creates a character, removes them, creates another into the same slot, and
asserts that the old identity resolves to nothing and reads nothing of the new
one. The test asserts first that the fixture reused the slot, because a fixture
that opened a second slot would prove nothing.

**The tier is a property of the shape.** Each shape declares its tier as an
associated constant on a sealed trait. A test reads the tier in a constant
context, so the item fails to compile if the tier ever becomes a run-time
value. A caller outside the crate cannot declare a shape or a tier, so an
assembled component set is a compile-time error.

**The tier states the ceiling, and the check runs once.** The arena refuses a
capacity above the ceiling of its declared tier when a caller builds it. The
world builds its arena at that ceiling. No call reads a population and refuses
on the count.

**The state hash covers the arena.** Every golden file moved, because an empty
character arena writes its three counters into the hash. A test proves that the
faction column, the birth column, the renown column and the generation column
each reach the hash, and it asserts that the fixture leaves the renown it writes
at zero, so the write is a real change.

**Zero is a real state.** A new character holds a renown of zero, and the type
represents it. A write of zero is a write.

**There is no demotion here, because there is nothing to demote to.** A test
removes every soldier in the world and asserts that the character survives with
their identity.

### What changed from the plan

**Promotion from the mass tier is not built.** The plan asked for the promotion
scan, the achievement column, the budget, and the debug check that the
achievement value never falls. The work was narrowed to the shape and the tier,
so that nothing which uses the shape is built before the shape is reviewed.
None of the deferred work is foreclosed. A new item holds it.

**A promoted soldier gets no invented ancestry, and the record states why.**
The decision record forecloses a promotion that mutates one entity into another
tier, so a promotion must create a character with a new identity. That is the
rule the resolved blocker asks for, and it holds without a lineage column.[^10]

**The arena holds no relation edge, no house, no parent, no office and no
biography log.** The research names the biography log as the one character
structure that grows without bound, and it states the rule that must govern it
before anything writes one.[^2] Nothing here writes one.

**One finding was opened.** The world's call to an arena invariant check breaks
no test when it is removed, because no public path can put an arena into a
state its own check rejects. The finding records what that does and does not
prove, and it applies to the soldier arena and the settlement arena too.[^14]

### Deliberately out of scope

Promotion, descent, succession, offices, houses, opinion, appointment, and the
link from a character to the unit that embodies them. Each is a separate item
with its own record.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: The character graph and inheritance. `docs/research/reports/14-character-graph-and-inheritance.md`
[^3]: ADR-0012, tiles are dense columns and units are a generational arena. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^4]: ADR-0014, entity identity is an index plus a generation. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^5]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^6]: ADR-0007, content supplies a key vector, never a comparator. `docs/adrs/accepted/adr-0007-content-supplies-a-key-vector-never-a-comparator.md`
[^7]: ADR Registry, row 0054. `docs/adrs/REGISTRY.md`
[^8]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^9]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^10]: Blockers register, BLK-011. `docs/BLOCKERS.md`
[^11]: Findings register, FND-007. `docs/FINDINGS.md`
[^12]: Findings register, FND-031. `docs/FINDINGS.md`
[^13]: Findings register, FND-022. `docs/FINDINGS.md`
[^14]: Findings register, FND-068. `docs/FINDINGS.md`
