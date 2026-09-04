# ADR-0142: A god inflicts weather only on ground its own faction holds

## Context

A game built on this engine gives a god a congregation to direct. The god is a
person or a language model, and it acts through the control plane. Inflicting
weather is a divine power, so it is a verb a god calls and not only something
that happens.

**A god that can freely make any weather anywhere is not a game.** The power
needs a bound, and the bound is the decision. Nothing in the engine bounded it
before this record.

**The project already has one central mechanic for what a faction may do at a
distance.** A faction may speak to another only when its people stand on the
ground of that other faction, and the presence relation answers that
question.[^1] [^2] A contract moves a quantity only when a unit carries it onto
the ground of the other party.[^3] Both rules say the same thing: a faction acts
where its people are, and not where they are not.

A divine power that ignored that rule would be the one action in the engine
that escapes the central mechanic. A god could then reach across the map with
no footing at all, and every other verb would look arbitrary beside it.

**A verb of the control plane is set-valued and all or nothing.** Python builds
a selector and sends one command, and Rust resolves it.[^4] Every existing
set-valued verb resolves the whole set before it writes anything, so one
refusal leaves the world unchanged.[^5]

The engine holds one derived answer that fits the gate exactly. The holding
records a mask of the factions that hold ground in each block, and a block is a
level 1 cell. Reading it is one lookup, and weather already lives on that
lattice.[^6] [^7]

## Decision

### D1. The cell of every place must hold ground of the god's own faction

**A god may put weather on a level 1 cell only when its faction holds at least
one tile inside that cell.** The engine reads the block mask of the holding and
refuses when the faction is not in it.

The gate is the ground and not the presence relation. The presence relation
answers whether one faction stands on the ground of another, and it is empty on
its own diagonal by design.[^8] The question here is whether the god's own
people have a footing in the place, and the holder column answers that
directly.

The alternatives are three. **No gate at all** makes the power unbounded and
makes every other verb look arbitrary. **A gate on standing rather than
holding** would let a god act on a place its people had walked through and
never held, which is weaker than the mechanic the project uses everywhere else.
**A gate on the influence field** would work, and it is rejected because the
control plane arms the influence sources itself, so a world that armed none
would silently give a god no power at all.

### D2. The god names a strength, never a quantity

**The verb takes a small whole number and the engine turns it into a quantity.**
A caller that named the quantity directly could put any amount of water into the
world in one call, and the gate above would not bound it.

The strength has a ceiling. The value of the ceiling and the quantity that one
point of strength carries are content constants that no measurement chose, and
a blocker holds the question of what the power should be worth.[^9]

### D3. The verb takes a set of places, answers once, and is all or nothing

**One call names a whole set of places, and the engine resolves the whole set
before it writes anything.** Every place is resolved, every gate is checked, and
the cooldown is checked first. One refusal leaves the world exactly as it was.

The set has a ceiling, and that ceiling is what stops one call from covering the
world. The cells are sorted and repeated cells are removed before the write, so
the result does not depend on the order the caller named the places in.[^10]

A cheaper whole-set algorithm follows from the set: the call writes each cell
once, and the weather that follows costs the lattice rather than the number of
places.[^11]

### D4. A faction waits between one storm and the next

**A successful call sets the first tick at which that faction may inflict
weather again.** A refused call does not, because a refusal costs the world
nothing and a caller that mistyped an address should not lose the power.

The wait is per faction and not per god, because the engine holds factions and
not gods. The number of ticks is a content constant that no measurement chose,
and the same blocker holds it.[^9]

## Consequences

A god with no ground has no power at all. A faction that has lost every tile
cannot act on the world through this verb, and that is deliberate: the power
follows the congregation.

The engine cannot express a divine power that reaches beyond the faction. A
game that wants one adds a second verb, and that verb needs a record of its own.

The engine cannot express two gods over one faction, and it cannot give one god
two congregations. The cooldown is stored against the faction, so both cases
would share one wait.

A god cannot choose where inside a cell the weather lands, because weather lives
on the cell.[^7]

A god cannot take weather away. The verb only raises water, and the field only
loses water by the rule it applies each tick. A game that wants a god to end a
storm needs a second verb, and a decision register row holds the question.[^12]

## References

[^1]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D1. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^2]: PRD-0031, a god knows whose ground its people stand on. `docs/product/shaped/prd-0031-a-god-knows-whose-ground-its-people-stand-on.md`
[^3]: ADR-0128, a contract moves a quantity only when a unit carries it onto the ground of the other party, decision D1. `docs/adrs/draft/adr-0128-a-contract-moves-a-quantity-only-when-a-unit-carries-it.md`
[^4]: ADR-0040, Python is a control plane, not a data plane, decisions D1 and D2. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^5]: ADR-0125, the control plane names the seed set of a destination field, decision D1. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^6]: ADR-0053, a faction is a bit in a mask, and a relation is a plane, decision D3. `docs/adrs/accepted/adr-0053-a-faction-is-a-bit-in-a-mask-and-a-relation-is-a-plane.md`
[^7]: ADR-0140, weather is a field over the level 1 cell lattice, decision D1. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^8]: ADR-0111, the presence relation is derived at the end of the step and never stored as a fact, decision D3. `docs/adrs/draft/adr-0111-the-presence-relation-is-derived-at-the-end-of-the-step.md`
[^9]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^10]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^11]: ADR-0140, weather is a field over the level 1 cell lattice, decision D2. `docs/adrs/draft/adr-0140-weather-is-a-field-over-the-level-1-cell-lattice.md`
[^12]: Decisions register, DEC-238. `docs/DECISIONS.md`
