---
id: 0059
title: Give a site a housing capacity and a resident reader
status: complete
created: 2026-08-31
implements: [ADR-0157 D1, ADR-0157 D2, ADR-0157 D3, ADR-0066 D1, ADR-0014 D1, ADR-0004 D1]
changes: []
creates: []
serves: [PRD-0014]
blocked-by: [BLK-050]
---

## Why

A site holds a store and it holds no people. Any number of units can belong to
one site, no place is worth defending, and crowding cannot happen. Growth has
nothing to stop it, because nothing states how many people a place holds.

This item gives a site a housing capacity, and it gives a caller one reader
that answers how many units live there. It is the last thing the growth item
waits for.

## What half of this item is already done, and how it was found

**The occupancy count is answered, and this item no longer builds one.** The
cohort table holds one row for each faction at each site. Each row holds a
headcount, and the table derives every headcount from the home column of the
unit arena. Both a row reader and a total are public, and a public check
derives the table again and compares it against the column.

The evidence, read on 5 September 2026:

- `crates/cachette-core/src/cohort.rs`, line 366, `pub fn headcount(&self,
  site: u32, faction: FactionId) -> Option<u32>`.
- `crates/cachette-core/src/cohort.rs`, line 381, `pub fn headcount_total(&self)
  -> Accum`.
- `crates/cachette-core/src/cohort.rs`, line 402, `pub fn rebuild(&mut self,
  homes: &[u32], ...)`, which derives every row from the home column.
- `crates/cachette-core/src/cohort.rs`, line 480, `pub fn describes(&self,
  homes: &[u32], ...) -> bool`, the check that derives the table again and
  compares.

**The residence column is answered too.** The unit arena holds the home column,
the founding writes it, and the destroy path clears it for every resident of a
lost site. The household module reads that column backwards, writes nothing,
and holds no array of its own.[^1]

**The housing capacity is not answered.** The settlement arena holds no housing
field. Its member named `capacity`, at `crates/cachette-core/src/site.rs` line
274 and its reader at line 343, is the ceiling on the slots the arena opens. It
is not a number of people.

So one of the four things this item once planned is real work, and one is a
small reader over rows that exist. The other two exist already.

## What the work does

1. A site holds a housing capacity, and the capacity follows from what has been
   built there rather than from the ground. **Give the field a name that is not
   `capacity`**, because that word already means the slot ceiling of the arena.
2. A caller reads how many units live at a site, through one reader that sums
   the cohort rows of that site. Nothing new is stored, and no count is
   maintained by the change.
3. A caller reads the free places of a site: the housing capacity less that
   sum.
4. The state hash covers the housing capacity.

## Impact review

**Governed by.** ADR-0157 D1 puts the housing capacity on the settlement shape
and refuses to derive it from the ground.[^2] ADR-0157 D2 forbids a second
resident count and names the derived one as the source.[^3] ADR-0157 D3 keeps
the home column as the residence, forbids a second column, and refuses a
reverse index from a site to its residents.[^4] ADR-0066 D1 fixes the
settlement as the shape that holds the capacity.[^5] ADR-0014 D1 makes a site
identity a slot and a generation, so a lost site never hands its identity to
the site founded next in that slot.[^6] ADR-0004 D1 fixes the order of the walk
that sums the rows.[^7]

**Creates.** No record. ADR-0157 is written and is under review, and this item
implements it. A reviewer who rejects a decision of that draft changes this
item.

**Changes.** No record. ADR-0074 D3 rejects a dense per-tile occupancy count,
and this item stores no occupancy count at all, so the two do not meet.[^8]

**Blockers.** BLK-050 governs the housing capacity value and the capacity a
founded site starts with. Both are balance register rows, and this item states
neither in the code.[^9] [^10] BLK-007 governs any cost figure, so this item
states none.

**Serves.** PRD-0014. A place holds a stated number of people that follows from
what was built there. A watcher reads how full a place is without walking the
population. A unit lives somewhere and a unit that lives nowhere is still a
unit.[^11]

**Precedent.** FND-116 records that the residence column and the eviction path
already exist, and that this item once planned both as new work.[^1] FND-128
records that the engine already answers the per-site count.[^12] FND-093
records that a test which drives a layered path is a guard rather than evidence
when an earlier stage already refuses the case.[^13]

**The household is not in this item.** A dwelling is stored and a household is
derived, and item 0103 holds the derived read.[^14] Do not build a household
structure here.

**Conflict surface.**

- `crates/cachette-core/src/site.rs`, at the settlement column set, for the
  housing capacity.
- `crates/cachette-core/src/world.rs`, at the state hash and at the public
  readers.
- `crates/cachette-core/src/cohort.rs`, read only. This item adds a reader that
  sums the rows of one site. It changes no write path and adds no store.
- `crates/cachette-core/src/soldier.rs`, read only. The home column exists.
  Adding a second one is the defect this item's precedent names.
- **It cannot run beside item 0060**, which reads the free places to admit a
  birth. Item 0059 lands first.
- Item 0113 touches the admission capacity path for tiles. That is a different
  capacity and a different file, and the two do not collide.

**What the fixture needs.** A world built from the demonstration binary
supplies no extreme.[^15] The fixture needs a site at its housing capacity, a
site above it, a site with housing and no residents, and a unit that lives
nowhere. Build the world that produces those. Do not copy the demonstration
world.

## Done when

- A caller reads the housing capacity of a site through the public interface,
  and the value came from what was built rather than from the terrain. A test
  builds two sites on the same terrain with different housing capacities.
- The housing field is not named `capacity`. A whole-tree search shows one
  meaning for that word inside the settlement arena, and the commit body
  carries the search.
- A caller reads how many units live at a site through the public interface,
  and the answer equals a full pass over the home column. A test asserts it
  after a run that assigned, evicted and killed units.
- **No new count is stored.** A whole-tree search for a maintained resident
  count returns nothing, and the commit body carries the search. A test that
  looked only at the answer would pass whether the count was derived or stored,
  so the search is the evidence.
- A caller reads the free places of a site, and the value equals the housing
  capacity less the resident count. A test asserts it for a site above its
  capacity, where the difference is not positive.
- Losing a site clears the residence of every unit that named it, and a test
  asserts that no unit still names the lost slot. The test runs on a world
  where a second site holds residents, so a clear-everything defect fails it.
- A unit that lives nowhere reads back as living nowhere, and it is still a
  unit that the world steps.
- The state hash covers the housing capacity. A test changes it alone and
  asserts the hash changes.
- The thread-count test and the golden state test pass with the new column, at
  1, 2 and 12 threads.
- No cost figure and no balance figure appear in the code or in a comment.
- The whole check command runs green.

## Outcome

**Built.** A site holds a stored housing column, a caller reads how many units
live there, and a caller reads the free places. Item 0060 landed in the same
change, so the readers have a caller from the first commit.

### The rename, and what it settled

The settlement arena held one member named `capacity`, and it meant the ceiling
on the slot index. The housing field is named `housing`, and the slot ceiling
is named `slot_ceiling`. The two builders that took the old name are now
`with_slot_ceiling` and `slot_ceiling`. The word `capacity` no longer appears
in the settlement arena at all, so the arena carries no word with two meanings.
The commit body carries the whole-tree search.

### What was already built, and what this item added

The item said that half of the work was already done, and that reading was
correct. The cohort table already derived every headcount from the home column,
and the eviction path already cleared the residence of every unit of a lost
site. This item added the housing column, one reader that sums the cohort rows
of one site, one reader for the free places, and the Python readers.

**Nothing new is stored for the resident count.** The reader sums the rows the
cohort table already holds, over the faction ceiling, and it walks no unit.

### The derived count is settled at a barrier, and a caller can read it stale

A caller that spawns a unit and does not step reads a cohort table that does
not yet describe that unit, in the way that the unit-to-tile bridge behaves.
The engine already answers this through `cohorts_describe_the_units`. The
fixtures of the new test file step once before they assert, and the file says
why.

### The done list

Every line of the done list above is met, except that the whole check command
was not run. The commands the dispatcher named were run instead, and the report
of the work states which.

The item asked for a test on two sites of one terrain with different housing.
The stored column answers it by construction: the founding writes one value
from a world parameter, and no pass reads the ground. The test file asserts the
stronger statement, which is that a caller writes a housing that the ground
never sets and that growth then reads it.

## References

[^1]: Findings register, FND-116. `docs/FINDINGS.md`
[^2]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D1. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^3]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^4]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D3. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^5]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^6]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^7]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D3. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^9]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^10]: Balance register, the population. `docs/reference/balance.md`
[^11]: PRD-0014, everyone needs somewhere to live. `docs/product/accepted/prd-0014-everyone-needs-somewhere-to-live.md`
[^12]: Findings register, FND-128. `docs/FINDINGS.md`
[^13]: Findings register, FND-093. `docs/FINDINGS.md`
[^14]: Decisions register, DEC-039. `docs/DECISIONS.md`
[^15]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
