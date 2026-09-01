# ADR-0054: An entity belongs to one of three tiers, declared at creation

Status: Accepted

## Context

This project simulates a world of tiles and units. The target scale is large
enough that no script can walk the population of a unit. The control plane is
Python. Python builds a selector, and the Rust core resolves it.

The entity storage holds four fixed shapes, and each shape gets its own set of
columns. The shapes are the soldier, the settlement, the living character and
the tile upgrade. The shapes do not vary while the engine runs.[^1] [^2]

The four shapes do not all have the same size of population. A soldier
population is larger than any script can walk. A living character population is
bounded, and a watcher must be able to follow one character. The control plane
therefore needs two different rules for two different shapes.

The project must decide what selects the rule. Two answers are available. The
engine can read the current population and decide from the count. Or a shape can
declare the rule once, at the type.

The research examined this question against the cost of the control plane
decision pass. It reached a rule, and it also derived the ceiling of the bounded
population. The scale constants table holds that ceiling.[^3] [^4]

## Decision

### D1. Every entity shape declares one of three tiers

The tiers are the mass tier, the character tier and the singleton tier.

- A mass tier holds more entities than a caller can walk. A caller sends one
  selector, and the engine resolves it.
- A character tier holds a bounded population. A caller may walk it.
- A singleton tier holds one thing.

A shape declares its tier at the type. The declaration is a constant, so it
resolves without a world and without a population.

A shape declares the stricter tier that its population admits. Widening a tier
later keeps every script that already works. Narrowing one does not.

### D2. The tier is a property of the shape, never a check on the count

The engine must not refuse a call because the population is large.

A check on the count makes one script work against a small world and fail
against a large one. An author develops against a few hundred entities and ships
against many thousand. The failure then appears far from its cause, and it
appears only at scale. That is the worst failure mode available here, and the
research names it as such.[^4]

A declared tier has no such state. A call that the tier forbids fails on the
first attempt, in development, and no world size makes it succeed.

### D3. The tier states the population ceiling, and the engine checks it once

The tier of a shape states the largest population that the shape admits. The
engine checks that ceiling when it builds the storage of the shape. It never
checks a population on a later call.

The ceiling of the character tier is a derived figure. The scale constants table
holds it with its derivation, and this record states no value.[^3] Every cost
figure in this project is derived and not measured.[^5]

The ceiling of the mass tier is the range of the slot index. That is a property
of the identity layout and it is not a budget.[^6]

### D4. An entity never changes tier while it lives

An entity belongs to the tier of its shape, and it holds that tier from the
moment the engine creates it until the moment the engine frees it.

There is no demotion. An entity that leaves the character tier would have to be
deleted, and a character can hold assets and relations that a deletion would
have to resolve. Identity also does not un-happen, and a history that deletes
its own subjects is not a history.[^4]

A promotion from the mass tier is therefore a creation in the character tier. It
is not a change of tier on one entity. The promoted entity gets a new identity,
and the identity it held in the mass tier ends under the ordinary rule: the
generation advances when the arena frees the slot, so the entity that comes next
in that slot never answers to the old identity.[^7]

### The alternative this rejects

A hard check on the count, at every call, is simpler to write. It needs no
declaration on the shape, and it adapts to a world of any size.

The project rejects it for the reason D2 gives. The check is correct on every
call it makes and wrong about when it makes them. It also cannot be read: no
reader of a script can tell whether that script will run, because the answer
depends on a world the script does not name.

A second alternative gives each shape a configured tier, read from content. The
project rejects that too. A configured tier is a run-time value, and D1 requires
a constant. A configured tier also puts a determinism-relevant choice in the
hands of content, and this project keeps such choices in the code.

## Consequences

**A fifth tier is a decision, not a configuration.** Adding one means a new
constant and a new rule at the control plane boundary. That cost is deliberate.

**A shape cannot be promoted into a looser tier for one world.** A world that
holds few soldiers still cannot walk them. This is the price of D2, and the
project accepts it, because the alternative is a script whose correctness
depends on the size of the world it meets.

**The ceiling refuses a large world at load, and not during a run.** An
operator who asks for a bounded population above the ceiling gets a refusal when
the world is built. The refusal names the tier and the limit. Nothing surprises
a running script.

**A promotion path must create and not mutate.** D4 forecloses the cheaper
implementation, in which one entity gains columns and keeps its identity. The
promoted entity gets a new identity in the new shape. Any link between the two
is an ordinary column, and it is a later decision.

**The tier is not a permission model.** It says how a caller reaches a
population. It says nothing about who may act on an entity.

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D1. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR-0066, entity storage holds four fixed shapes, decision D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^4]: The character graph and inheritance, sections 8.4 and 9.8. `docs/research/reports/14-character-graph-and-inheritance.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: ADR-0014, entity identity is an index plus a generation, decision D1. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^7]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
