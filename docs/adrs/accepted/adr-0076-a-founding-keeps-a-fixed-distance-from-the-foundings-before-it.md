# ADR-0076: A founding keeps a fixed distance from the foundings before it

## Context

A run of this engine begins with a small group of people in one place. The
engine chooses the place. It draws a fixed number of candidate places from the
keyed generator, reads a fixed number of tiles around each one, and takes the
best candidate it read.[^1] The number of tiles it reads is a property of the
choosing rule and not a function of the world extent.

A run now founds one group for each faction the world holds. The owner chose
that shape over a run with one founding, and the two produce different
games.[^2] One founding gives a run with one society on an empty map. One
founding for each faction gives a run in which the factions meet, and the tick
on which they meet follows from how far apart the engine put them.

Several foundings raise three questions that one founding does not raise.

**How close may two foundings stand?** A group settles over a disc of tiles
around its place.[^1] Two places closer than twice that radius seat their
groups over one piece of ground. The founding does not read the occupancy of a
tile, so the engine accepts the overlap and the two groups start inside each
other.[^3]

**What fixes the order?** Founding N reads the places that the foundings
before it took, so the foundings are a sequence and not a set. An unstated
order is a determinism defect, and the scope rule asks for a record even when
the answer looks obvious.[^4]

**What key does each founding draw on?** The record for one founding keys each
candidate address on the system, the frame, the entity and the draw index, and
it puts the candidate ordinal in the entity slot.[^1] That key holds nothing
that separates one founding from another.

## Decision

### D1. A founding refuses a place inside the minimum distance of a place taken

A founding of a run compares each candidate place against every place a
founding before it took. A candidate closer than the minimum distance is not
eligible, whatever its ground holds.

The minimum distance is a parameter of the founding rule, in the way the
sample size is a parameter of it.[^1] This record states the constraint and not
the value. The floor is not a parameter: the distance is greater than twice
the radius that a group settles over, so two groups never start over one piece
of ground.

**A founding that finds no eligible place in its sample fails.** It does not
draw again and it does not widen the sample, because a sample that widens
until it succeeds is a pass over every tile with extra steps.[^1] A failed
founding is a correct outcome. The product record already allows a run in
which a group finds no place.[^5]

Two alternatives were considered and both are refused.

- **A distance that falls as the faction count rises.** This seats a crowded
  world that this rule refuses to seat. It also derives the distance from the
  faction count, which makes the faction count a second declaration site for
  a property of the founding rule.[^6]
- **A partition of the world into a region for each faction.** This is a claim
  about the structure of the map, and it removes the case in which the engine
  refuses to seat a faction. The refusal is the honest outcome, and this
  project keeps it.

### D2. A run founds in ascending faction index, and reports one outcome for each faction

The run founds one group for each faction the world holds, in ascending
faction index, and for at least one faction. The order is a property of the
run. A caller gives no order, so no caller can win the better place for one
faction by naming it first. The order never comes from the order a thread
finished.[^7]

The faction count comes from the world settings. The founding loop holds no
count of its own, and it derives the faction identifiers from that one count.
A second count in the loop would be a second declaration of how many factions
the world holds, and nothing would fail when the two disagreed.

**The run reports one outcome for each faction.** A run of several foundings
can seat some factions and refuse another. One result for the whole run hides
either the foundings that stood or the one that did not, and both are wrong. A
caller reads which factions were seated, which were refused, and how many
candidates a refused founding drew.

A failed founding leaves the foundings before it standing. The run does not
undo them.

### D3. The faction fills the frame slot of the draw key

Each candidate address stays a draw from the keyed generator, on the tuple of
the system, the frame, the entity and the draw index.[^8] The candidate
ordinal keeps the entity slot and the axis keeps the draw slot.[^1] The
faction fills the frame slot.

A founding happens before the first frame, so the frame slot carries no frame.
It carried a constant. It now carries the faction, and two factions therefore
read two samples.

This is the alternative to an amendment of the record for one founding. That
record states the key for a founding that stands alone, and it stays true: a
run of one founding draws on the frame slot of one faction. This record states
what a run of several needs.

**Without the faction in the key every founding reads one sample.** The defect
is invisible to both determinism tests, because the sample repeats on every
run, at every thread count and on every machine.[^9] A test that changes the
faction and asserts that the sample changes is the only test that sees it.

## Consequences

**A world can be too small for its faction count.** A run seats what the
separation admits and refuses the rest. The engine does not shrink the
distance to fit, and it does not enlarge the world.

**A caller reads a list, not a result.** Every caller of a run handles a
mixture of seated factions and refused ones, including the demonstration
binary and every test.

**The comparison grows with the faction count.** It does not grow with the
world extent, so the bounded cost of the founding still holds.[^1] The faction
count is bounded by the width of the faction mask.[^10]

**The first faction gets the best place in its sample.** The rule is
positional, and the lowest faction index reads no taken place. This is a
consequence of the fixed order, and the fixed order is what the project needs
more than it needs fairness between factions. A later record may make the
foundings equal; it will supersede this one.

**A shared sample no longer starves a founding, but it still narrows one.**
The engine seats every faction even when every faction draws one sample,
because a sample holds many places that stand far apart. The finding holds the
evidence.[^11] The key test is therefore the only guard on D3.

## References

[^1]: ADR-0075, the founding choice reads a bounded sample of the world, decisions D1, D2 and D4. `docs/adrs/accepted/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^2]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^3]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decision D1. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^4]: Decision Record Scope, the counter-test in section 1. `.claude/rules/adr-scope.md`
[^5]: PRD-0012, a world starts small and grows. `docs/product/accepted/prd-0012-a-world-starts-small-and-grows.md`
[^6]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^7]: ADR-0004, iteration order is explicit, decision D4. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^8]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^9]: Testing rules, section 2. `.claude/rules/testing.md`
[^10]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D1. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^11]: Findings register, FND-106. `docs/FINDINGS.md`
