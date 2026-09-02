---
id: 0059
title: Give a site a capacity and a resident roll
status: proposed
created: 2026-08-31
implements: [ADR-0081 D1, ADR-0081 D2, ADR-0081 D3, ADR-0081 D4, ADR-0066 D1, ADR-0014 D1, ADR-0004 D4]
changes: []
creates: [ADR-0081]
serves: [PRD-0014]
blocked-by: []
---

## The impact review is withdrawn

This item was refined against ADR-0081. A review rejected that record, so the
impact review no longer names a governing decision that binds.[^R1]

The rejection is not a detail of wording. ADR-0081 states that nothing in the
engine answers how many units a site holds. The engine answers it today. A
per-site headcount exists, it is derived from the unit home column, and the
public check that compares the two already exists. So the decision this item
implements either forbids code that ships, or it buys nothing.

**Do not take this item until a record replaces ADR-0081.** Refining it again
starts by reading what the engine already does, not by planning the count as new
work.

## Why

A unit stands on a tile and belongs to nothing that limits it. Any number of
units can fill a region and no place holds anybody. Growth costs nothing, no
place is worth defending, and crowding cannot happen.

This item gives a site a capacity and makes the site a unit belongs to the
place it lives. Where a unit stands and where it lives stay two different
facts.

## What the work does

1. A site holds a housing capacity, and the capacity follows from what has
   been built there rather than from the ground.
2. A site holds an occupancy count, kept by the change and never recomputed
   by a sweep over the units.
3. The invariant check compares the occupancy count against the residence
   column, and fails when they disagree.
4. A watcher reads the capacity and the occupancy of a site through the
   public interface.

**The residence column is not new work.** The soldier arena already holds the
site a unit belongs to, and the founding already writes it. The destroy path
already clears it for every resident of a lost site. This item extends that
column and adds no second one.[^1]

## Impact review

**Governed by.** ADR-0081 D1 puts the capacity on the settlement shape and
refuses to derive it from the ground.[^2] ADR-0081 D2 makes the existing site
column the residence and forbids a second column.[^3] ADR-0081 D3 makes
occupancy a maintained count and requires a check that can fail.[^4]
ADR-0081 D4 refuses a reverse index from a site to its residents, so the
eviction stays a pass over the units.[^5] ADR-0066 D1 fixes the settlement as
the shape that holds the capacity.[^6] ADR-0014 D1 makes a site identity a
slot and a generation, so a lost site never hands its identity to the site
founded next in that slot.[^7] ADR-0004 D4 requires a stable key for the
assignment and for the eviction.[^8]

**Creates.** ADR-0081. The registry row exists and the draft is written. This
item implements it. A reviewer who rejects a decision of the draft changes
this item.

**Changes.** No record. ADR-0074 D3 rejects a dense per-tile occupancy count,
and ADR-0081 D3 keeps a per-site one for reasons the tile case does not have.
The two stand together and neither supersedes the other.[^9]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-005 is resolved and gives the settlement count.[^10] BLK-003 is resolved:
the population counts everybody, so everybody needs somewhere to live.[^11]

**Serves.** PRD-0014.

**Precedent.** FND-116 records that the residence column and the eviction path
already exist, and that item 0059 planned both as new work.[^1] FND-093
records that a test which drives a layered path is a guard rather than
evidence when an earlier stage already refuses the case.[^12]

**The household is not in this item.** DEC-039 decides that a dwelling is
stored and a household is derived, and item 0103 holds the derived read.[^13]
Do not build a household structure here.

**Conflict surface.**

- `crates/cachette-core/src/site.rs`, at the settlement column set for the
  capacity and the occupancy. **The arena already has a method named
  `capacity`, and it means the ceiling on the number of slots the arena
  opens.** A housing capacity that takes that name gives one word two
  meanings. Name the new one for housing.
- `crates/cachette-core/src/world.rs`, at the invariant check, at the state
  hash, at `set_home_site` and at `destroy_settlement`.
- `crates/cachette-core/src/soldier.rs`, read only. The column exists. Adding
  one there is the defect this item's precedent names.
- **It cannot run beside item 0060**, which reads the occupancy to admit a
  birth, and **it cannot run beside item 0103**, which reads the residence to
  derive a household. Both wait on this item.
- Item 0113 touches the admission capacity path for tiles. That is a different
  capacity and a different file, and the two do not collide.

**What the fixture needs.** A world built from the demonstration binary
supplies no extreme, and both facts of this item live at one.[^14] The fixture
needs a site at its capacity, a site above its capacity, a site with capacity
and no residents, and a unit that lives nowhere. Build the world that produces
those. Do not copy the demonstration world.

## Done when

- A caller reads the housing capacity of a site through the public interface,
  and the value came from what was built rather than from the terrain. A test
  builds two sites on the same terrain with different capacities.
- A caller reads how many units live in a site, through the public interface,
  without walking the units.
- A test asserts the occupancy count against a full pass over the residence
  column, after a run that assigned, evicted and killed units.
- The invariant check fails when the occupancy count and the residence column
  disagree. **A test proves it fails**: it makes the two disagree and asserts
  the refusal. A test that only reads both in a healthy world is a guard, not
  evidence.[^12]
- Losing a site clears the residence of every unit that named it, and a test
  asserts that no unit still names the lost slot. The test runs on a world
  where a second site holds residents, so a clear-everything defect fails
  it.
- A unit that lives nowhere reads back as living nowhere, and it is still a
  unit that the world steps.
- A population above the capacity of a site is representable, and a caller
  reads both numbers and names the difference.
- The state hash covers the capacity and the occupancy. A test changes each
  one alone and asserts the hash changes.
- The thread-count test and the golden state test pass with the new columns,
  at 1, 2 and 12 threads.
- No cost figure appears in the code or in a comment.
- No second residence column exists. `grep -rn "resid" crates/` returns no
  new column, and the commit body carries the search.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-116. `docs/FINDINGS.md`
[^2]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D1. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^3]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D2. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^4]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D3. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^5]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D4. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^6]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^7]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^8]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^9]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D3. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^10]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^11]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^12]: Findings register, FND-093. `docs/FINDINGS.md`
[^13]: Decisions register, DEC-039. `docs/DECISIONS.md`
[^14]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
[^R1]: Findings register, FND-128. `docs/FINDINGS.md`
