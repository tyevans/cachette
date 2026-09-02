---
id: 0060
title: Grow the population from the store and the housing
status: proposed
created: 2026-08-31
implements: [ADR-0082 D1, ADR-0082 D2, ADR-0082 D3, ADR-0082 D4, ADR-0003 D1, ADR-0014 D3, ADR-0004 D1]
changes: []
creates: [ADR-0082]
serves: [PRD-0011, PRD-0014, PRD-0012]
blocked-by: [0059, DEC-044]
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

The number of units is a number somebody chose. Nothing the world does changes
it. A faction that gathers well and one that gathers badly hold the same units
for ever, so success has no expression and the world has no decline.

A unit can already end. This item lets one begin, and ties both to what a
place has.

## What the work does

1. A site proposes a birth at a rate its store sets.
2. The free places of the site admit the proposals until no place is free. A
   refused proposal is discarded.
3. A birth spawns a unit and writes its residence. Nothing adds to a cohort
   headcount.
4. The pass runs after the pass that ends a starved unit, in the same frame.

## Impact review

**Governed by.** ADR-0082 D1 makes the store a rate that proposes.[^1]
ADR-0082 D2 makes the housing an admission bound that never scales the rate,
which is how the two limits compose by one operation with one answer.[^2]
ADR-0082 D3 makes a birth a unit and forbids adding to a headcount.[^3]
ADR-0082 D4 fixes the order against the death pass and states the draw
key.[^4] ADR-0003 D1 requires the key on the tuple of system, frame, entity
and draw, and D2 forbids thread-local state.[^5] ADR-0014 D3 makes the new
unit's identity distinct from the identity of the unit that died in its
slot.[^6] ADR-0004 D1 fixes the order in which the births apply.[^7]
ADR-0081 D3 gives the occupancy count that the admission reads.[^8]

**Creates.** ADR-0082. The registry row exists and the draft is written. This
item implements it.

**Changes.** No record. ADR-0074 D1 says a spawn reads no capacity, and this
item does not change that: growth counts the free places itself, in the way
ADR-0074 D4 requires of a caller that must not over-fill.[^9] [^10]

**Blockers.** BLK-007 governs every cost figure, so this item states none.
BLK-003 gives the population target and BLK-005 gives the settlement
count.[^11] [^12]

**DEC-044 governs the behaviour, and this item depends on it.** The default
need rule sets the ration equal to the decay, so a unit whose need reaches
zero never climbs back and its deficit only rises.[^13] Every shortage that
empties a need is therefore fatal. Growth adds mouths to the same store, so
under the default rule a site that grows into a shortage loses the population
it grew, and no later plenty repairs it. **The mechanism of this item does not
wait on DEC-044. Three statements about behaviour do**, and the acceptance
list below states them against a rule the test chooses rather than against the
default. The row is named in `blocked-by` because the default the engine ships
with decides whether a grown population can survive at all, and this item is
the work that makes that consequence visible.

**The two open questions this item carried are answered.**

**A birth is a unit, not a headcount.** A cohort headcount is not stored
independently. The frame rebuilds the cohort table from the columns of the
units after a structural change, so a birth that added to a headcount would
declare the population twice and nothing would fail when the copies
disagreed.[^14] This item therefore touches the spawn path. ADR-0082 D3 holds
the answer.

**The composition of the two limits is decided.** The store sets a rate and
the housing admits, and neither limit is applied twice. ADR-0082 D2 holds it,
and it is the same propose-then-admit shape the project uses for movement and
for gathering.[^15]

**Serves.** PRD-0011 and PRD-0014.

**Precedent.** FND-089 records that the recovery path of the need rule is
unreachable under the default rule, and that a test of a rise alone passes
while the fall is unreachable.[^13] FND-093 records that a test which drives a
layered path is a guard rather than evidence when an earlier stage already
refuses the case.[^16] Both apply directly: a growth test that runs a fed world
never reaches the refusals, and a growth test that takes the default rule
measures the default rule.

**Conflict surface.**

- `crates/cachette-core/src/world.rs`, at the step between `reap` and
  `refresh_bridge`, at `spawn_soldier` and at `set_home_site`.
- `crates/cachette-core/src/site.rs`, read for the store and for the occupancy
  and the capacity that item 0059 adds.
- `crates/cachette-core/src/cohort.rs`, read only. The rebuild already follows
  a structural change, and this item adds no headcount write.
- `crates/cachette-core/src/rates.rs`, for the rate shape and the schedule.
- **It cannot run beside item 0059**, which adds the occupancy this item
  reads. Item 0059 lands first.
- **It cannot run beside item 0123**, which changes the same step ordering
  around the rates pass.

**What the fixture needs.** Growth lives at three extremes, and the
demonstration world supplies none of them.[^17] The fixture needs a site with
a full store and no free place, a site with free places and an empty store, a
site with exactly one free place and more than one proposal in one frame, and
a site whose last resident died this frame. Build the world that produces
those. **Put each refusal back and watch the test stay green**, and report the
result rather than hiding it. A green test says the case does not reach the
assertion.[^16]

## Done when

- A site with a surplus and a free place adds a unit at an interval, and a
  caller reads the new unit through the public interface.
- A site with no surplus adds none. A site with no free place adds none. A
  test asserts each case separately, on a world where the other limit is
  satisfied, so that neither test is passed by the other refusal.
- A site with one free place and more than one proposal in one frame admits
  exactly one, and the one it admits follows from a stable key rather than
  from a thread.
- The rate never scales with the free places. A test holds the store fixed,
  varies the free places above one, and asserts the same number of births.
- The birth draw is keyed. A test changes the frame and asserts the draw
  changes. A second test changes the site and asserts the draw changes.[^18]
- A slot freed by a death this frame is free for a birth this frame, and the
  reused slot gives an identity that never resolves as the dead unit. A test
  asserts both.
- A birth writes the residence of the new unit, and the occupancy count of
  the site rises by one. A test asserts the count against a full pass.
- Nothing writes a cohort headcount. A test asserts the headcount after a
  frame of births against a full pass over the residence column.
- **Under a rule whose ration exceeds the decay**, a fed site grows and holds
  its population across many frames. A test asserts it, and it states the
  rates it uses rather than taking the default.[^13]
- **Under the default rule**, a site that grows past what its store feeds
  loses the units it grew, and does not recover. A test asserts that too, so
  that the consequence DEC-044 governs is recorded in the suite rather than
  discovered later.
- A count of the population is read at a cost that does not grow with the
  population, and a test asserts it against a full pass.
- The thread-count test asserts that the same seed gives the same births, in
  the same order, at 1, 2 and 12 threads. The golden state test passes.
- No cost figure appears in the code or in a comment.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D1. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^2]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D2. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^3]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D3. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^4]: ADR-0082, the store sets the rate of a birth and the housing admits it, decision D4. `docs/adrs/draft/adr-0082-the-store-sets-the-rate-of-a-birth-and-the-housing-admits-it.md`
[^5]: ADR-0003, every random draw is keyed, never stateful. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^6]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^7]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: ADR-0081, a residence is a stored column and occupancy is a maintained count, decision D3. `docs/adrs/draft/adr-0081-a-residence-is-a-stored-column-and-occupancy-is-a-maintained-count.md`
[^9]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^10]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D4. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^11]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^12]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^13]: Findings register, FND-089. `docs/FINDINGS.md`
[^14]: ADR-0063, a need is a rate with a threshold, and crossing it is a fact, decision D2. `docs/adrs/accepted/adr-0063-a-need-is-a-rate-with-a-threshold-and-crossing-it-is-a-fact.md`
[^15]: ADR-0056, movement is tile-discrete and admitted by sort-then-admit, decision D3. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^16]: Findings register, FND-093. `docs/FINDINGS.md`
[^17]: Testing Rules, a fixture supplies the input. `.claude/rules/testing.md`
[^18]: Testing Rules, a determinism test cannot tell correct from consistently wrong. `.claude/rules/testing.md`
[^R1]: Findings register, FND-128. `docs/FINDINGS.md`
