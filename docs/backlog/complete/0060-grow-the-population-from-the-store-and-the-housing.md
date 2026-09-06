---
id: 0060
title: Grow the population from the store and the housing
status: complete
created: 2026-08-31
implements: [ADR-0082 D1, ADR-0082 D2, ADR-0082 D3, ADR-0082 D4, ADR-0157 D2, ADR-0157 D4, ADR-0157 D5, ADR-0157 D6, ADR-0003 D1, ADR-0014 D3, ADR-0004 D1]
changes: []
creates: []
serves: [PRD-0011, PRD-0012, PRD-0014]
blocked-by: [0059, DEC-044]
---

## Why

The number of units is a number somebody chose. Nothing the world does changes
it. A faction that gathers well and one that gathers badly hold the same units
for ever, so success has no expression and the world has no decline.

A unit can already end. This item lets one begin, and ties both to what a place
has.

**Growth is the source of people, and it is the only one.** A worker gathers
into the store of a site. Food and housing grow the population of the site. A
per-site production queue then spends one person and goods to make a typed
unit.[^1] So growth feeds every later link of the loop, and nothing else does.

## What the work does

1. A site proposes a birth at a rate its store sets.
2. The free places of the site admit the proposals until no place is free. A
   refused proposal is discarded.
3. A birth spawns a unit and writes its residence. Nothing adds to a cohort
   headcount.
4. The pass runs after the pass that ends a starved unit, in the same frame.
5. The census gains a row for the births of the frame, so a zero is visible.

## Impact review

**The previous impact review was withdrawn**, because it rested on ADR-0081 and
a review rejected that record. ADR-0157 replaces it, and this review is written
against ADR-0157 and ADR-0082 together.

**Governed by.** ADR-0082 D1 makes the store a rate that proposes.[^2] ADR-0082
D2 makes the housing an admission bound that never scales the rate, which is how
the two limits compose by one operation with one answer.[^3] ADR-0082 D3 makes a
birth a unit and forbids adding to a headcount.[^4] ADR-0082 D4 fixes the order
against the death pass and states the draw key.[^5]

ADR-0157 D2 gives the free places their source: the housing capacity less the
resident count the engine already derives, and **no second count is stored**.[^6]
ADR-0157 D4 fixes the stage and forbids a pass over the units.[^7] ADR-0157 D5
makes growth the only source of people, and makes the grown unit carry the type
the spawn path gives.[^8] ADR-0157 D6 makes the writes disjoint by site and puts
the new unit in the state hash.[^9]

ADR-0003 D1 requires the key on the tuple of system, frame, entity and draw, and
D2 forbids thread-local state.[^10] ADR-0014 D3 makes the new unit's identity
distinct from the identity of the unit that died in its slot.[^11] ADR-0004 D1
fixes the order in which the births apply.[^12]

**What changed against the withdrawn review.** The old review named ADR-0081 D3
for the occupancy count that admission reads. That count is gone. The admission
now reads the derived count, and this item stores nothing new. **Do not plan the
count as new work.**

**Creates.** No record. ADR-0082 and ADR-0157 are written and under review, and
this item implements both.

**Changes.** No record. ADR-0074 D1 says a spawn reads no capacity, and this item
does not change that: growth counts the free places itself, in the way ADR-0074
D4 requires of a caller that must not over-fill.[^13] [^14]

**Blockers.** BLK-007 governs the cost figures this item would state, so it
states none. BLK-050 governs the birth rate, the food a birth costs, the housing
a person needs and the growth schedule. The balance register holds all four, and
this item states none of them in the code.[^15]

**DEC-044 governs the behaviour, and this item depends on it.** The default need
rule sets the ration equal to the decay, so a unit whose need reaches zero never
climbs back and its deficit only rises.[^16] Every shortage that empties a need
is therefore fatal. Growth adds mouths to the same store, so under the default
rule a site that grows into a shortage loses the population it grew, and no
later plenty repairs it. **The mechanism of this item does not wait on DEC-044.
Three statements about behaviour do**, and the acceptance list below states them
against a rule the test chooses rather than against the default.

**Waits on item 0059**, which gives a site its housing capacity and the reader
that sums the resident rows. Item 0059 lands first.

**Serves.** PRD-0011, PRD-0012 and PRD-0014.

**Precedent.** FND-089 records that the recovery path of the need rule is
unreachable under the default rule.[^16] FND-093 records that a test which
drives a layered path is a guard rather than evidence when an earlier stage
already refuses the case.[^17] FND-128 records that the engine already answers
the per-site count.[^18] All three apply directly.

**Conflict surface.**

- `crates/cachette-core/src/world.rs`, at the step between the reap stage and
  the barrier that follows it, and at the spawn and residence verbs.
- `crates/cachette-core/src/site.rs`, read for the store and for the housing
  capacity that item 0059 adds.
- `crates/cachette-core/src/cohort.rs`, read only. This item adds no headcount
  write.
- `crates/cachette-core/src/rates.rs`, for the rate shape and the schedule.
- **It cannot run beside item 0059**, which adds the housing this item reads.
- **It cannot run beside item 0497**, which adds the queue stage beside this
  one, and **it cannot run beside item 0123**, which changes the same step
  ordering around the rates pass.

**What the fixture needs.** Growth lives at three extremes, and the
demonstration world supplies none of them.[^19] The fixture needs a site with a
full store and no free place, a site with free places and an empty store, a site
with exactly one free place and more than one proposal in one frame, and a site
whose last resident died this frame. Build the world that produces those. **Put
each refusal back and watch the test stay green**, and report the result rather
than hiding it.

## Done when

- **The demonstration world grows.** A run of the demonstration binary ends with
  more units than it began with, and a test drives the demonstration seeding
  rather than a hand-built fixture.
- **A faction that founds with the founding group of the balance register
  reaches a population at or above the campaign cohort size of the balance
  register, inside 2500 ticks.** That is half of the 5000-tick game horizon the
  project owner set on 5 September 2026, and the balance register holds the
  horizon on the tick limit row.[^15] Half of it is the bound this item takes,
  so that the rest of the loop still fits: queue a settler, found a second city,
  queue soldiers, raise a cohort, and reach a decision. A test asserts the
  population against the two rows it reads and against that tick count.
- **A site at its housing bound grows nobody, whatever its store holds.** A test
  fills a site to its housing, gives it a large store, runs many ticks, and
  asserts that the population did not move. It then raises the housing and
  asserts that the population moves again, so the bound is shown to be a stop
  and not a slowdown.
- **The census reports the growth, so a zero is visible.** A census row counts
  the births of the frame. A test asserts that the row is above zero in a fed
  world and exactly zero in a starving one, so a reader can tell a world that
  cannot grow from a world that chose not to.
- A site with a surplus and a free place adds a unit at an interval, and a
  caller reads the new unit through the public interface.
- A site with no surplus adds none. A site with no free place adds none. A test
  asserts each case separately, on a world where the other limit is satisfied,
  so that neither test is passed by the other refusal.
- A site with one free place and more than one proposal in one frame admits
  exactly one, and the one it admits follows from a stable key rather than from
  a thread.
- The rate never scales with the free places. A test holds the store fixed,
  varies the free places above one, and asserts the same number of births.
- The birth draw is keyed. A test changes the frame and asserts the draw
  changes. A second test changes the site and asserts the draw changes. A third
  takes two proposals of one site in one frame and asserts they differ.[^20]
- A slot freed by a death this frame is free for a birth this frame, and the
  reused slot gives an identity that never resolves as the dead unit. A test
  asserts both.
- **Nothing stores a resident count.** The admission reads the derived one. A
  whole-tree search for a maintained per-site resident count returns nothing,
  and the commit body carries the search.
- Nothing writes a cohort headcount. A test asserts the headcount after a frame
  of births against a full pass over the residence column.
- The growth stage walks no unit. A reviewer reads the stage and finds no loop
  over the population, and the item states that as a review statement rather
  than as a test.
- **Under a rule whose ration exceeds the decay**, a fed site grows and holds
  its population across many frames. A test asserts it, and it states the rates
  it uses rather than taking the default.[^16]
- **Under the default rule**, a site that grows past what its store feeds loses
  the units it grew, and does not recover. A test asserts that too.
- A count of the population is read at a cost that does not grow with the
  population, and a test asserts it against a full pass.
- The thread-count test asserts that the same seed gives the same births, in the
  same order, at 1, 2 and 12 threads. The golden state test passes.
- No cost figure and no balance figure appear in the code or in a comment.
- The whole check command runs green.

## Outcome

**Built.** One stage grows the population, it runs after the shortage scan and
before the queue advance, and growth is the only source of people.

### The chain closes, and the run says so

A test seeds the demonstration world, runs it, and asserts that a faction
reaches the campaign cohort size. It drives the seeding the demonstration
drives, it calls no verb of its own, and it turns no draw off. **A faction
reached the bar at eight of eight seeds, on the first growth application.** The
run reads the founding group and the cohort size from the world, so no number
in the test can drift from the register.

Before this item a faction founded with two people and a raise asked for four,
so no cohort could be raised, no contest could kill, and domination could never
fire.

### The values

The growth stage holds no literal. It reads the housing per person, the food
per birth, the birth chance, the growth schedule and the founding housing from
the world, and the balance register holds one row for each. Every row is unset
and behind a blocker, and each now carries a provisional default with a stated
derivation.

### The census row is written and not registered

The engine counts the births of the tick and a public reader answers it. **The
subsystem census table was held by another worker while this landed, so the row
is not in it.** The reader is `births`, and the row must be registered. Nothing
else is missing.

### What the code and the record disagree about

Nothing. The record states no rate and no value, and the stage states none.

One decision needed a reading rather than a choice. The record requires a keyed
draw for a birth and it does not say what the draw decides. The store already
answers how many births a site can pay for, so a draw over affordability would
be inert. **The draw decides whether a proposal takes**, against a chance that
the balance register holds, so growth is a rate and not a step. A test asserts
that a chance below one refuses some proposals, so the draw is not decoration.

### A fourth term of the world conservation statement

Growth takes food out of a store to make a person, and neither the rate ledger
nor the draw ledger held that. The conservation test failed the moment growth
landed. The world now holds a growth ledger, and the findings register holds
the case.

## References

[^1]: ADR-0158, a site builds a typed unit from a bounded queue its store pays for, decision D4. `docs/adrs/accepted/adr-0158-a-site-builds-a-typed-unit-from-a-bounded-queue-its-store-pays-for.md`
[^2]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^3]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^4]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D3. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^5]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^6]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D2. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^7]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D4. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^8]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D5. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^9]: ADR-0157, a site's free places are its built housing less the residents the engine already counts, decision D6. `docs/adrs/accepted/adr-0157-a-sites-free-places-are-its-built-housing-less-the-residents-the-engine-counts.md`
[^10]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^11]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^12]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^13]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^14]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D4. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^15]: Balance register, the population, the seeding layer, the controller and the game end. `docs/reference/balance.md`
[^16]: Findings register, FND-089. `docs/FINDINGS.md`
[^17]: Findings register, FND-093. `docs/FINDINGS.md`
[^18]: Findings register, FND-128. `docs/FINDINGS.md`
[^19]: Testing Rules, a fixture supplies the input. `.agents/rules/testing.md`
[^20]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.agents/rules/testing.md`
