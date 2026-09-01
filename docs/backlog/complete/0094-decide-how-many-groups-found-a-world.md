---
id: 0094
title: Found one group for each faction, at a fixed minimum distance
status: complete
created: 2026-08-31
implements: [ADR-0075 D1, ADR-0075 D2, ADR-0075 D4, ADR-0075 D5, ADR-0053 D1]
changes: []
creates: [ADR-0076]
serves: [PRD-0012]
blocked-by: []
---

## Why

A run begins with one group, in one place, of one faction. The world holds a
faction count, and every faction except the founding one begins with nobody
and nothing.

The product record does not decide how many groups found a world. It names one
group and one group for each faction as the two candidates, and it says
plainly that this is not its question.[^1]

The choice changes the early run more than any rule that acts on it. One
founding gives a run with one society and an empty map around it. One founding
for each faction gives a run in which the factions meet, and the tick on which
they meet follows from how far apart the engine put them.

## What the answers are

**Every faction founds one group.** The blocker is resolved, and the owner
chose the second candidate the product record named.[^2] A run begins with one
founding for each faction the world holds.

**Two foundings keep a fixed minimum distance, and the sample stays
fixed.**[^3] A founding that finds no admissible place in its sample fails.
A failed founding is a correct outcome, and the product record already allows
it.[^1] The distance does not scale with the faction count, and the world is
not partitioned into a region for each faction. Both of those were considered
and both were refused.

The engine already takes the group size and the faction at the founding call,
so a caller founds one group or several without an engine change.[^2] The work
is the founding loop, the separation rule, and the record that holds it.

## What the work does

1. The run founds one group for each faction the world holds, in one fixed
   order.
2. A founding refuses a place closer than the minimum distance to any place a
   founding before it took.
3. A founding that finds no admissible place in its sample fails, and it
   reports the failure. It does not draw again and it does not widen.
4. A watcher can see every founded place and compare them.
5. The demonstration founds one group for each faction.

## Impact review

**Governed by.**

- ADR-0075 D1. The number of tiles a founding reads does not depend on the
  tile count.[^4] The separation adds a comparison against the places already
  taken. That comparison grows with the faction count and not with the world
  extent, so the record still holds. A test asserts the tile count over two
  worlds of different extent, as the record requires.
- ADR-0075 D2. Each candidate address is a draw keyed on the system, the
  frame, the entity and the draw index, and the candidate ordinal fills the
  entity slot.[^4] **With one founding for each faction, the candidate ordinal
  alone repeats the same sample for every faction.** The faction must enter
  the key, or every founding reads one sample and only the first one succeeds.
  This is the point where the work meets the record, and the record does not
  say which slot the faction takes.
- ADR-0075 D4. Two candidates that score the same resolve by the tile
  index.[^4] The founding order across factions must be as fixed as the order
  inside one sample. The run founds in ascending faction index, never in the
  order a thread finished.
- ADR-0075 D5. The chooser reports what made the place the choice.[^4] The
  report now covers several foundings, and a watcher compares them.
- ADR-0053 D1. A faction is a bit in a mask.[^5] The founding loop reads the
  faction set the world holds. It holds no second count of its own.

**Changes.** No accepted record changes yet. **ADR-0075 D2 may need one.** The
key that D2 states is written for one founding, and several foundings need a
faction in the key. Whether that is an amendment to ADR-0075 or a decision in
the record this item creates is a judgement the author of the record makes.

**Creates.** ADR-0076: a founding keeps a fixed minimum distance from every
founding before it, and a founding that finds no admissible place fails.

The decision needs a record under all three conditions of the scope rule.[^6]
A contributor could reasonably choose otherwise, because a separation that
falls as the faction count rises seats a crowded world that this rule refuses
to seat. Choosing otherwise costs more than changing it later, because a
separation derived from the faction count is a second declaration site, and a
partitioned map is a claim about map structure. The reasoning is not visible
in the artefact, because a reader of the comparison sees a constant and not
the two options it refuses.

**The record states the ordering claim as well as the distance.** Founding N
reads the places that foundings 1 to N-1 took, so the foundings are a sequence
and not a set. ADR-0075 assumes one founding standing alone, so no record
states what fixes the order of a sequence of them. An unstated order is a
determinism defect, and the scope rule says a decision that governs
determinism earns a record even when the answer looks obvious.[^6]

The record states the constraint and not the distance. The distance is a
tuning knob, in the way that the sample size is a tuning knob.[^4]

The registry holds row 0076, and the record is not written yet. Writing it is
the first part of this work.[^7]

**Blockers.** BLK-018 is resolved, and its answer is above.[^2] BLK-007
governs every cost figure, so this item states none.[^8] No blocker governs
the separation rule.

**Precedent.**

- Shape 1 of the recurring defect rule governs the faction count. The founding
  loop reads the faction set the world holds. It does not take a count from a
  caller, and it does not hold one of its own.[^9]
- FND-054 records that a small test world holds one kind of ground.[^10] A
  world that seats several separated foundings must be wide enough to hold
  them, so the fixture extent follows from the separation and from the faction
  count. State that in the fixture rather than choosing an extent that happens
  to work.
- FND-070 records that a restored defect must be the smallest change that
  violates the claim.[^11] For the separation, that is one step off the
  boundary.

**Serves.** PRD-0012. The record asks that a run begin with a small group,
that the engine choose the place by reading the world, and that a watcher
compare the founded place against the places that were not chosen.[^1] Several
foundings answer the same statements, and the record already states that a
founding may fail.

**Conflict surface.** The founding path in the world, where a run is founded.
The founding tests. **It cannot run beside item 0092 or item 0095**, which
change the same founding path.

## The open question the record must answer

**Which slot of the draw key holds the faction?** ADR-0075 D2 keys each
candidate address on the system, the frame, the entity and the draw index, and
it puts the candidate ordinal in the entity slot.[^4] That key is written for
one founding. Every faction founds from the same key, so every faction draws
the same sample. Only the first founding can then satisfy the separation rule,
and the rest fail for a reason that has nothing to do with the world.

The faction must enter the key. The record does not say where, and this item
does not choose. **The work that writes ADR-0076 answers it, by one of two
routes.**

- An amendment to ADR-0075 D2, which states the key for several foundings
  rather than for one.
- A decision inside ADR-0076, which states the key that the separation rule
  needs and leaves ADR-0075 as the rule for one founding.

**It cannot be left to the implementer.** An unkeyed field is the defect shape
the testing rule names: a draw keyed on the wrong field draws the same wrong
value on every thread, on every run and on every machine.[^14] Both determinism
tests pass, because both compare a run against another run and the two runs
agree. The engine has carried this defect once already, on the same shape, and
only the golden hash noticed, by accident.

**The test that catches it changes the faction and asserts that the sample
changes.** A test that only repeats a run cannot see it.

## What fixes the order of the foundings

**The faction index fixes it, and nothing fixes it today.** The engine has no
call that founds more than one group. The call that founds a run takes one
faction from the caller, so the caller chooses which faction founds first. With
one founding the choice does not matter, because no founding reads another. With
the separation rule it decides which faction gets the better place.

This is a defect to close in this item and not a detail for the implementer.
The new run-level call reads the faction set the world holds and founds in
ascending faction index. It does not take an order from the caller, and it does
not found in the order a thread finished.[^12]

## What the caller sees when a founding fails

The engine already has the refusal. A founding that finds no place in its
sample returns the error variant that carries the number of candidates it
drew.[^13]

**One result for the whole run is not enough.** A run of several foundings can
have some succeed and one fail. A single result either hides the foundings that
stood or hides the one that did not, and both are wrong. The run-level call
reports one outcome for each faction, so a caller reads which factions were
seated, which were refused, and how many candidates the refused founding drew.

A failed founding leaves the foundings before it standing. The run does not
undo them, because a failed founding is a correct outcome and not an error in
the run.[^1]

## Done when

- The run founds one group for each faction the world holds, in ascending
  faction index.
- ADR-0076 is written, and it states the distance claim and the ordering claim.
- A test asserts the separation at its boundary. Two foundings at exactly the
  minimum distance are admitted, and one step closer is refused.
- A test asserts that a founding which finds no admissible place fails, and
  that it reports the failure rather than drawing again.
- The faction is in the draw key, and the record says which slot holds it.
- A test changes the faction and asserts that the sample changes, so two
  factions read different candidates and a second founding is not refused for
  reading the first one's.[^14]
- A test asserts that the tiles a founding reads do not depend on the world
  extent, over two worlds of different extent.[^4]
- A test asserts the founding order. The faction that founds first is the
  lowest faction index, whatever order the caller listed.
- A test lets one founding fail in a run of several, and asserts that the
  caller reads the outcome of every faction and that the earlier foundings
  still stand.
- The same seed gives the same set of founding places at 1, 2 and 12 threads.
- The separation rule is put back, and the tests are watched failing, before
  the item is claimed done. The restored defect is the smallest change that
  violates the claim.[^11]
- The golden state hash files are regenerated where a founding moved, and the
  commit body says which files moved and why.
- `just check` exits 0.

## Outcome

**A run founds one group for each faction, in ascending faction index.** The
run-level call reads the faction set the world holds. It reports one outcome
for each faction, so a caller reads which factions were seated and which were
refused. A refused faction leaves the foundings before it standing.

**ADR-0076 holds the three claims.** A founding refuses a place inside the
minimum distance of a place taken. A run founds in ascending faction index and
reports one outcome for each faction. The faction fills the frame slot of the
draw key. The record is a draft and a reviewer holds the acceptance.[^15]

**The minimum distance is a constant of the founding rule, and the record
states no value.** A compile-time assertion fails when the distance falls to
twice the survey radius, so the floor and the radius cannot disagree in
silence.

**The demonstration founds one group for each faction.** A watcher sees four
settled places in the window rather than one.

**Two findings came out of the work.** A shared sample does not starve the
foundings after the first, which the item predicted that it would.[^16] A
fixture of four factions cannot see the separation rule at all, and the test
that measured it was a guard until the fixture crowded.[^17]

**The golden state hash file for the founding scenario moved**, because the
scenario now founds one group for each faction. The commit body names the file
and the command that recorded it.

**One decision closed.** The frame slot holds the faction.[^18]

## References

[^1]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^2]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^3]: Decisions register, DEC-037. `docs/DECISIONS.md`
[^4]: ADR-0075, the founding choice reads a bounded sample of the world. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^5]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^6]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^7]: Backlog guide, the line between proposed and refined. `docs/backlog/README.md`
[^8]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^9]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^10]: Findings register, FND-054. `docs/FINDINGS.md`
[^11]: Findings register, FND-070. `docs/FINDINGS.md`
[^12]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^13]: The founding error type. `crates/cachette-core/src/founding.rs`
[^14]: Testing rules, section 2. `.claude/rules/testing.md`
[^15]: ADR-0076, a founding keeps a fixed distance from the foundings before it. `docs/adrs/draft/adr-0076-a-founding-keeps-a-fixed-distance-from-the-foundings-before-it.md`
[^16]: Findings register, FND-106. `docs/FINDINGS.md`
[^17]: Findings register, FND-107. `docs/FINDINGS.md`
[^18]: Decisions register, DEC-051. `docs/DECISIONS.md`
