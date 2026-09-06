# ADR-0156: A faction's option weights are policy, set through one verb

## Context

A unit in this engine chooses one option from a small fixed set. It scores each
option and takes the highest. Each score is one multiplication of how much the
unit wants a thing by how much of that thing is near.[^1] How much is near
comes from the level 1 cell the unit stands in. A level 1 cell summarises one
block of tiles. The pass never searches the world.

"How much the unit wants a thing" is one weight for each option. **The engine
holds one weight set for every unit alive.** The weight set is an input to the
world rather than a fact the world holds, so it reaches no state hash, and no
faction can differ from another in what its units want.

A tile carries one holder, and the holder names one faction or nobody.[^2] No
option reads the holder. No summary field of a cell says whose ground it is.

**The project owner stated the need on 5 September 2026.** Workers wander
outside their own faction's ground, and the game then makes no sense. A unit
should care about improving its own faction's land. When road planning lands,
some units should take road work as it is needed. The owner also said that the
built-in controllers should work this way as far as they can, and that a
learner will reach its own conclusions.

Four forces fix the shape of the answer.

**A preference baked into the pass becomes physics.** A learner plays one
faction against the built-in controllers, and it acts through a bounded action
table the engine declares.[^3] [^4] A term the pass always applies is a term
the learner cannot choose otherwise. The engine would then have decided the
strategy that the learner exists to find.

**The pass must stay a read of one cell.** The cost of the pass follows the
lattice and not the population.[^5] A term that made a unit look at more ground
would move the cost back onto the population.

**The engine shares one answer between many units.** The pass scores the option
set once for each cell, each bucket of need and each class of the unit's own
state.[^6] [^7] A term keyed on something unbounded would give one answer to
each unit.

**A faction must still reach ground it does not hold.** A road project may lie
off held ground, and the build rule exempts a road from the own-ground
refusal.[^8] [^9] A rule that kept every unit inside its own ground would take
that reach away.

## Decision

**The weight a faction gives each option is that faction's policy. One verb
sets it, and a caller, the built-in controller and a learner all use that one
verb.**

### D1. The engine holds one weight set for each faction, and a unit reads the set of its own faction

The engine holds one weight set for each faction. Each weight is an integer or
a fixed-point value, and the engine bounds it.[^10] The choice pass reads the
weight set of the faction of the unit that asks.

**A weight set is simulated state, and it enters the state hash.** A caller
writes it during a run, so two worlds that differ in it diverge on the next
tick. A weight set that stayed outside the hash would be a value the step reads
on every tick and that nothing hashes.

**The mechanism carries no preference of its own.** This decision says where a
weight lives and who reads it. It says nothing about which option a faction
should favour.

A reviewer finds a violation when the pass reads a weight that is not the
weight of the unit's own faction, when a weight is a floating point number,
when a weight has no bound, or when a weight set reaches no state hash.

### D2. One weight is the own-ground term, and the engine states no value for it

The weight set gains one term for whose ground the option reads. The term holds
three parts: how much a unit favours ground its own faction holds, how much it
favours ground another faction holds, and how much it favours ground nobody
holds.

**This record states no value for any of the three.** They are game values of
the downstream game. The balance register holds the rows and the defaults, and
one blocker governs them.[^11] [^12]

The pass picks the part by comparing the holder of the cell with the faction of
the unit. The value it reads is a property of the cell, it is an integer, and
the pass reads it through the path that carries every other option value, so no
second declaration of the holder rule exists.[^13]

The cell value names one faction, or nobody, or more than one faction. The
combine of two such values is associative and commutative, so a cell equals the
exact combination of its tiles in any order.[^14] The tile holder column stays
the one place that says who holds what.[^2] **A summary field indexed by the
faction is refused**, because a field with one entry for each faction would
multiply the tile side of the world by the faction count.[^15]

A reviewer finds a violation when a weight value appears as a literal in the
code or in this record, when a second rule computes a holder for a cell, or
when the combine depends on the order of the tiles.

### D3. One verb sets a faction's weights, and every actor uses that verb

One verb writes the weight set of one faction. A Python caller calls it. The
built-in controller calls it. No path exists for the controller alone.[^16] The
verb refuses a weight outside the bound and counts the refusal.[^17]

**The action table reaches this verb.** A learner acts through one integer that
indexes a bounded table the engine declares, and a learner that could not set
its own weights would play a strategy the engine chose for it.[^4]

A reviewer finds a violation when the controller writes a weight set by a path
a caller cannot call, when the verb reads who called it, or when the action
table names no entry that reaches this verb.

### D4. The built-in controller favours the ground its own faction holds

The built-in controller writes a weight set that favours ground its own faction
holds, and that gives road work the faction planned a weight above work nobody
asked for. It draws the strength of the preference from the weight vector the
seeding already draws for it, by one keyed draw.[^18] [^19]

**This states what the controller does. It states nothing the engine forces.**
A learner that writes the opposite weights is playing the game correctly. A
watcher who reads the demonstration sees the behaviour the owner asked for,
because the controller chose it and not because the pass imposes it.

A reviewer finds a violation when the controller's preference is written into
the choice pass rather than into a weight set, or when a weight the controller
writes cannot be written through the verb of D3.

### D5. A weight changes a score, and never the option set, the cell or the order

These stay engine physics, and no weight touches them.

- The option set is fixed, and the engine owns it.
- The score is one multiplication of the want by what is near.
- A unit reads the level 1 cell it stands in, and the pass searches nothing.
- The scan order is ascending option index, the comparison is strict, and the
  lowest option index wins a tie.[^20]

**The own-ground term is therefore a weight and never a fence.** A unit may
still leave the ground its own faction holds, and it needs a stronger reason to
do so. A fence would strand a unit that already stands outside its own ground,
because the engine founds a unit at a city and a unit walks away from it. A
fence would stop a faction ever taking new ground, because a faction takes new
ground by founding a city on it or by using it.[^21] [^22] A fence would also
refuse the walk that the road exemption of the build rule permits, so two rules
would state opposite things about one road.[^8]

A reviewer finds a violation when a weight adds an option, removes one, changes
the cell a unit reads, opens a search, or changes the tie rule.

### D6. The own-ground term never refuses work the faction's plan zones

A unit takes work its own faction's plan zones ahead of work nobody asked for.
A faction writes a bounded plan of zoned projects, and a unit builds only what
the plan zones.[^8] An idle unit takes the nearest project through the build
verb.[^9] A road project may lie off the ground the faction holds, because the
plan is what bounds a road and the ground rule is what gives the reach.[^8]

**The two rules must agree.** The own-ground term must not lower a zoned
project below what a unit takes. A project the plan names is work the faction
asked for, whoever holds the ground under it.

**No pass reads a plan today.** The choice pass scores a fixed set of cell
fields and one class of the unit's own state. This decision states the
constraint that binds the option which reads a plan, when someone writes that
option. It does not describe an option that exists.

A reviewer finds a violation when an option that reads a plan carries the same
holder term as an option that reads the ground, or when a unit refuses a zoned
road because another faction holds the tile.

### D7. The choice key gains the faction, and the own-ground term needs no term of its own

The key of a shared answer is the cell, the bucket of the need, and the class
of the unit.[^6] [^7] The key gains the faction of the unit.

The key must gain it, because D1 gives each faction its own weight set. Two
units of one cell and one need, in two factions, no longer share an answer.

**The own-ground term adds nothing further to the key.** The cell is already in
the key, and the faction is now in it. Whose ground the cell is, is a function
of those two, so the term changes a score without changing the key again.

The faction is a bounded class of the unit's own state, and the engine states
the faction ceiling.[^15] The table fills as a unit asks for an entry, so a
cell pays for the factions whose units stand in it and for no other.[^23]

A reviewer finds a violation when two units of one cell, one need, one carry
class and one faction receive two answers, when the key holds a second term for
whose ground it is, or when a cell scores an entry for a faction that no unit
of it stands in.

## The alternatives this rejects

**Apply the own-ground term in the pass, for every unit.** It is the shortest
answer, and one reviewer can check it in one read. It is rejected under D1 and
D4. A term the pass always applies is physics, and a learner that plays one
faction could never choose otherwise, which is the whole point of the learner
seat.[^3]

**Fence a unit inside the ground its own faction holds.** It states the owner's
words directly. It is rejected under D5. A fence strands a unit that is already
outside, it stops a faction ever taking new ground, and it contradicts the road
exemption of the build rule.[^8]

**Let a controller order name the tiles a unit may stand on.** A controller
could compute the ground of its faction and send a set of units to it. It is
rejected because the control plane would then loop over entities and name
places one at a time. The control plane builds a selector and sends one
command, and the engine resolves it.[^24]

**Give the controller a weight set the caller cannot write.** It would give the
built-in controller the behaviour the owner asked for in one change. It is
rejected because the controller emits commands only through verbs a caller can
also call, and a private path would put the controller outside the rule that
makes a learner able to replace it.[^16]

**Leave it to the lease.** Repeated use of a tile moves its lease, and a lease
at the claim threshold holds the tile.[^22] A faction whose units wander would
come to hold the ground they wander over. It is rejected for two reasons. The
lease changes who holds the ground, and it does not change what a unit wants.
The lease also takes many ticks, and the owner's complaint is about what a
watcher sees on one tick.

## Consequences

**A faction that weights its own ground highly wins no new ground by use, and
the faction's own policy is what pulls against the lease rule.** Repeated use
of a tile raises a lease on it, and a lease at the claim threshold holds the
tile against the reach of a city.[^22] That rule rewards a faction whose units
go out. A weight set that keeps every unit at home puts the lease out of reach.
**The lease rule is engine physics, and the weight set is the faction's
policy.** Neither refuses the other, and the weights are what balance them. A
faction that never wins ground by use has chosen that, and a learner may choose
the other way.

**The weight set enters the state hash, so every golden file moves.** The set
was an input and is now simulated state.

**The choice answer table gains a factor of the faction count in entries.** The
lazy fill means a cell pays for the factions whose units stand in it, and a cell
usually holds units of one faction or two.

**A test that asserts a choice must now state the faction of the unit and the
weight set of that faction.** A fixture that leaves every faction on the same
weights measures the mechanism and never the policy.

**A watcher reads the term.** The engine answers a question about a choice by
computing the scores again, and the explanation reports the weights and the
class it scored.[^25]

**Nothing enforces D6 yet.** No pass reads a plan, so no gate can fail. A
reviewer checks the constraint when the option that reads a plan arrives.

**The engine still cannot tell two units of one class and one faction apart.**
This record widens the key. It does not lift the rule that a cost follows the
lattice and not the population.[^5]

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D2. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^3]: PRD-0056, a learner plays one faction against the controllers. `docs/product/accepted/prd-0056-a-learner-plays-one-faction-against-the-controllers.md`
[^4]: ADR-0154, the observation and the action of a faction are schema-declared bounded tables, decision D4. `docs/adrs/accepted/adr-0154-the-observation-and-the-action-of-a-faction-are-schema-declared-bounded-tables.md`
[^5]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^6]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^7]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D1. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
[^8]: ADR-0152, a faction plans its roads and zones with one solver, decision D4. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^9]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^10]: ADR-0002, simulated and aggregated state holds no floating point number, decision D1. `docs/adrs/accepted/adr-0002-state-holds-no-floating-point-number.md`
[^11]: Balance register, the choice. `docs/reference/balance.md`
[^12]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^13]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^14]: ADR-0023, an aggregate combines exactly, in any order, decision D1. `docs/adrs/accepted/adr-0023-an-aggregate-combines-exactly-in-any-order.md`
[^15]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^16]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D2. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^17]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D3. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^18]: ADR-0144, a faction controller runs inside the step and acts only through the caller's verbs, decision D4. `docs/adrs/accepted/adr-0144-a-faction-controller-runs-inside-the-step-and-acts-only-through-the-callers-verbs.md`
[^19]: ADR-0003, every random draw is keyed, never stateful, decision D1. `docs/adrs/accepted/adr-0003-every-random-draw-is-keyed-never-stateful.md`
[^20]: ADR-0004, iteration order is explicit, and unordered reductions need slots, decisions D1 and D3. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^21]: ADR-0150, held ground is the ground within reach of a city its faction owns, decisions D4 and D5. `docs/adrs/draft/adr-0150-held-ground-is-the-ground-within-reach-of-a-city-its-faction-owns.md`
[^22]: ADR-0153, a tile's lease follows the units that stand on it, decisions D2 and D5. `docs/adrs/accepted/adr-0153-a-tiles-lease-follows-the-units-that-stand-on-it.md`
[^23]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D3. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^24]: ADR-0040, Python is a control plane, not a data plane, decision D1. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^25]: ADR-0109, the choice key holds a bounded class of the unit's own state, decision D4. `docs/adrs/draft/adr-0109-the-choice-key-holds-a-bounded-class-of-the-unit-state.md`
