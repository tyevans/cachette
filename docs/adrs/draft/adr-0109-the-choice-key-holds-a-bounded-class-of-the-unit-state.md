# ADR-0109: The choice key holds a bounded class of the unit's own state

## Context

A unit in this engine chooses one option from a small fixed set. It scores
each option against the level 1 cell it stands in and takes the highest. A
level 1 cell summarises one block of tiles.[^1]

The engine does not score the set once for each unit. It scores it once for
each cell and each bucket of need, and a unit reads the answer. The key of an
answer is therefore the cell and the bucket, and nothing else.[^2] That key
works because the engine holds one weight profile for every unit alive, so two
units of one cell with one need always receive one answer.[^3]

**Every option reads the ground. None reads the unit.** Each option names one
value that the cell already summarises, and a unit prefers the neighbour that
holds more of it. The only thing a unit brings to a score is its need.

That has a measured consequence. A unit gathers a load into a carry column. A
pass moves that load into the store of the unit's home site, and it fires only
while the unit stands on the tile of that site. Nothing puts it there on
purpose, so a run of the demonstration world delivered almost nothing. The
findings register holds the measurement, and the register also holds what the
number became when a later change let a refused unit take a draw.[^4] [^5]

**The behaviour the engine is missing is not a preference for a kind of
ground. It is a consequence of what the unit itself holds.** A unit that
carries a full load has a reason to go home that a unit standing beside it
with an empty load does not have. No cell field can express that difference,
because both units read one cell.

### Why the obvious answers do not work

**Score each unit at its exact load.** A load is a whole number for each
resource kind. Two units of one cell almost never hold one load, so the key
becomes one key for each unit and the pass computes one answer for each
reader. That is the shape the cost record refuses.[^6]

**Leave the option set alone and give the unit a rule outside it.** A rule that
overrode the choice for a laden unit would be a second place where behaviour
is declared. The option set is the one declaration site today, and one fact in
two places with nothing to fail when the copies disagree is the defect shape
this project records most often.[^7]

**Let the content author add a class.** A class that content could add would
put the bound on the key in the content rather than in the engine. A world
that declared many classes would raise the work back toward one answer for
each unit, and nothing would refuse it. The bucket of a need is a bound the
engine holds for exactly this reason, and a class must be one too.[^2]

## Decision

### D1. An option may rank a class of the unit's own state, and the choice key holds that class

The key of an answer is the cell, the bucket of the need, and the class. Two
units that share all three share one answer, and the engine computes it once.

An option ranks either one summary field of the cell or one class of the unit.
It never ranks both, and it never ranks a quantity the unit carries.

**A class is not a measurement.** The value an option reads from a class is
the class itself, so an option that ranks a class is worth one whole unit of
value to a unit in the class it names and nothing to any other. A unit outside
that class scores zero, which is below the floor, so it can never take the
option.[^8]

### D2. A class holds a fixed number of values, declared by the engine

The engine states the class count at compile time. The answer table is the
bucket count multiplied by the class count, so the table has a ceiling that
neither the population nor the content can raise.

The table still fills as a unit asks for an entry, so a cell scores the
smaller of the units it holds and the entry count.[^9] A class therefore
costs a cell nothing until a unit of that class stands in it.

### D3. The first class is the carry, and a unit with no home is never laden

A unit is laden when it holds a home site and its load reaches the carry mark.
Every other unit is free.

**The mark is a parameter of the world, and this record does not set it.** A
low mark sends a unit home for almost nothing and spends its life walking. A
high mark keeps it in the field until the deposit beside it runs dry. The
reference table holds the value and the derivation of it.[^10]

**A unit with no home is never laden, whatever it carries.** The delivery
moves a load into the store of a home site. A unit that holds none can deliver
to nothing, so an option that sent it home would be a capability that no
behaviour can act on, which is the shape the rules name.[^11]

### D4. An explanation reports the class that the pass scored

The engine answers a question about a choice by computing the scores again,
because it stores none.[^12] That recomputation takes the class of the unit,
and the explanation reports it.

An explanation that scored one class and reported another would name a winner
that the unit did not take. A watcher would then read a correct engine as a
broken one.

### D5. An option that ranks a class wins a tie against an option that ranks the ground

The option that ranks a class takes the lowest option index, and the lowest
index already wins a tie.[^13]

**The tie is the ordinary case and not an edge.** The open share of a cell
reaches one whole unit wherever a whole block admits a unit, and the value of
a class is one whole unit. The two scores are then equal, so without this rule
a laden unit keeps walking in every world that has no water in it. That was
measured rather than argued.

The rows that rank the ground keep the order they had between themselves, so
every tie that they already decide keeps the winner it has.

## The alternatives this rejects

**Rank the load rather than the class.** It is exact, and it would let a unit
that carries more go home sooner than one that carries less. It is rejected
under D1: the load is nearly continuous, so it gives one key to each unit and
the pass stops sharing anything.

**Quantise the load, in the way the need is quantised.** It would keep the key
bounded and keep some of the resolution. It is rejected because nothing asks
for the resolution. The behaviour is a threshold: a unit goes home or it does
not. A bucket of a load would be a mechanism with no reader, and this project
records what an unused capability costs.[^11]

**Put the class in the column, so that the stored state is the class.** It
would remove the rule that derives one from the other. It is rejected because
the load is a quantity that the gather resolve writes and the delivery reads,
and a stored class would be that quantity summarised in a second place.[^7]

**Give a laden unit an option outside the set.** A rule above the choice would
send it home without touching the option set. It is rejected because the
option set is where behaviour is declared, and a second declaration site has
nothing that fails when the two disagree.[^7]

**Let the class hold more than the carry today.** A unit whose need is
critical, and a unit that holds no home, are both classes a reader thinks of
at once. They are rejected for now because the engine has no verb that either
of them could reach. A starving unit already forages, and no verb gives a unit
a home. A class with nothing to act on is a capability nobody invokes.[^11]

## Consequences

**The choice table doubles in entries when a class of two values is added.**
The lazy fill of the answer table means a cell pays only for the entries its
units ask for, so the cost of a class a cell never holds is the storage and
nothing else.

**A later class multiplies the table again.** The table is one array on the
stack, so its length is a property of the layout. A project that wanted many
classes would have to decide whether the table stays an array.

**A test that asserts a unit's choice must now state the class it depends
on.** A fixture that gives its units no home, or no load, never reaches an
option that ranks a class. Three properties of one golden fixture each blocked
it independently, and the findings register holds them.[^14]

**The engine still cannot tell two units of one class apart.** The cost record
already made that true for two units with one cell and one need. This record
adds a term to the key rather than removing the constraint.

**Nothing enforces this record against a new option.** A later option that
read a quantity from the unit would compile, and no gate would fail. A
reviewer reads what the answer depends on and compares it with the key.

## References

[^1]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D1. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^2]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D1. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^3]: Findings register, FND-251. `docs/FINDINGS.md`
[^4]: Findings register, FND-317. `docs/FINDINGS.md`
[^5]: Findings register, FND-319. `docs/FINDINGS.md`
[^6]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decisions D1 and D4. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^7]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^8]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D3. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^9]: ADR-0098, the choice is decided for each cell and each bucket of need, decision D3. `docs/adrs/draft/adr-0098-the-choice-is-decided-for-each-cell-and-each-bucket-of-need.md`
[^10]: Budgets and costs, the choice pass. `docs/reference/budgets.md`
[^11]: Recurring defect shapes, shape 3. `.claude/rules/recurring-defects.md`
[^12]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D2. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^13]: ADR-0064, a unit chooses by scoring a small fixed option set, decision D5. `docs/adrs/accepted/adr-0064-a-unit-chooses-by-scoring-a-small-fixed-option-set.md`
[^14]: Findings register, FND-320. `docs/FINDINGS.md`
