# ADR-0165: A build order holds a unit on its tile, and the hold is derived and never stored

## Context

A unit of this engine builds by standing still. The build pass reads where each
unit stands, and it adds the work of that unit to the site on that tile.[^1] One
level of one upgrade asks for more work than one unit adds in one tick, so a
site finishes only when the same unit stands on the same tile for several ticks
in a row.

The movement pass read no build order. A unit that carried one moved in the same
way as a unit that carried none, so it left the tile on the next tick and its
labour landed on another site. A sweep of many seeds of the demonstration world
measured what that cost. The labour of one faction was enough for eleven roads,
it reached thirty-one tiles, and it finished none of them.[^2]

**No balance value repairs this.** The same sweep ran again with a smaller plan
bound, and more seeds then finished nothing, because a smaller plan gives a
wandering unit fewer tiles to stand on. A larger bound spreads the same labour
over more tiles. The register holds both readings.[^2]

The rule that makes the repair hard is a deliberate one. **Movement takes its
direction from a field over cells, and never from a search that starts at a
unit.**[^3] A set-valued command buys a cheaper algorithm, and this project
chose one flow field over many path searches.[^4]

A build order is not a field. It names one unit and one tile. A direction that
said "stay" could be written into the cell field, but a cell holds a block of
tiles and every unit of that block reads one entry, so the whole block would
stop. The engine therefore needs a per-unit answer inside a pass that was built
to avoid per-unit answers.

The obvious per-unit answer is a stored flag. A stored flag needs a rule for
every way it can end: the work finishes, the plan drops the project, the ground
changes hands, the order is revoked, and the unit dies. A flag that outlives one
of those freezes a unit for the rest of the run, which is worse than the defect
it repairs. The register already holds an open item about a unit that cannot
move.[^5]

## Decision

### D1. A unit that stands on the work its order names does not move

The movement pass asks, for each unit, whether the tile the unit stands on is
work that the unit's own order names and that the unit can advance. When the
answer is yes, the unit produces no movement intent for that tick.

A reviewer finds a violation when a unit under a build order that the build pass
would accept leaves its tile, or when a unit that the build pass would refuse
stays on one.

**This binds the engine and not the controller.** A learner that drives the same
verbs gets the same rule, because the rule lives in the movement pass and not in
the built-in controller.[^6] A controller that re-issued the order on every tick
would reach the same behaviour by accident, and it would spend one command slot
for each builder on each tick.

### D2. The hold is derived on every tick, and nothing stores it

The pass computes the answer from the build order of the unit, the ground under
it, the upgrade that stands there, the holder of the tile, the plan of the
unit's faction, and the row of the unit's type. It writes nothing.

**A derived hold has no release rule, because there is nothing to release.** The
answer stops being true on the tick the work finishes, on the tick the plan
drops the project, on the tick the ground changes hands, and on the tick a
caller revokes the order. A unit that dies leaves no state behind, because no
state was ever written.

The engine already resolves the same question in the build pass and in the verb
that gives the order. **One function states the rule, and three paths call it.**
Two statements of one rule would let a unit be held for work that the build pass
refuses.[^7]

A reviewer finds a violation when any column, map or side table records that a
unit is held.

### D3. A unit that adds no work is never held

The hold requires that the unit's own type contributes work above zero. A type
whose build rate is zero, and a type whose rate scales the base work below one,
contribute nothing.[^8]

Without this clause the hold has no bound. A unit that adds no work never
finishes the site it stands on, so the condition that holds it never stops being
true, and the unit stands there for the rest of the run.

**The clause is not defensive.** The engine ships types that build at zero, and
the built-in controller orders every unit of its faction that stands on a zoned
tile, whatever its type.[^6]

With the clause, the hold is bounded. Work never runs backwards, so a unit that
adds work above zero on every tick reaches the work the row asks for in a number
of ticks that the row and the rate decide.[^1]

## The alternatives this rejects

**A stored hold flag on the unit, written by the build pass.** Rejected because
the build pass runs after the movement pass in the step, so the flag it writes
is one tick old when movement reads it. A unit that arrives on a project and
takes its order at the end of one tick would walk away on the next one, which is
the defect this record repairs. It is also rejected because a stored flag needs
five release rules and a leak in any one of them freezes a unit.

**A direction of "stay" in the cell field.** Rejected because a cell holds a
block of tiles and one entry serves every unit of the block. One builder would
stop every unit of its cell, including the units that carry no order.[^3]

**A per-unit destination that names the unit's own tile.** Rejected because the
destination field steers by the reach of a cell, so a unit already inside the
seeded cell reads no direction and falls back to a keyed draw.[^9] The draw is
what makes the unit wander, so the repair would restate the defect.

**A controller that re-issues the order on every tick.** Rejected because it
binds no other caller, and because it spends one command slot for each builder
on each tick. The controller acts through the caller's verbs and its command
budget is the thing that bounds it.[^6]

**A bound of ticks on the hold.** Rejected because D3 already bounds it. A tick
bound would be a balance value that governs a case that cannot arise, and a
value that governs nothing decays without anybody noticing.[^10]

## Consequences

**A builder no longer feeds itself.** A held unit does not walk to food, so its
need falls while it builds. Whether a builder starves before it finishes is a
reading that a run settles, and the decisions register holds the question.[^11]

**The movement pass reads one more column for every live unit.** The read is
gated on the build order byte. A unit that carries no order costs that byte and
no lookup, so a population that does not build pays one column.[^12]

**The engine resolves the build rule twice in one tick.** The movement pass asks
it before the barrier and the build pass asks it after. The two answers can
differ, because the barrier moves other units and the holding pass rewrites the
holder column between them. That is correct: each pass asks about the world it
acts on.

**A unit under an order it can satisfy stays until it finishes.** A caller that
wants the unit back revokes the order through the verb that gave it.

## References

[^1]: ADR-0151, an upgrade is a category with a ground fit and a level, decisions D1 and D2. `docs/adrs/accepted/adr-0151-an-upgrade-is-a-category-with-a-ground-fit-and-a-level.md`
[^2]: Findings register, FND-545. `docs/FINDINGS.md`
[^3]: ADR-0091, movement takes its direction from a per-cell field, never from a per-unit search, decisions D1 and D4. `docs/adrs/draft/adr-0091-movement-takes-its-direction-from-a-per-cell-field.md`
[^4]: ADR-0095, a behavioural strategy arrives as a field over cells, never as a search from a unit, decisions D1 and D2. `docs/adrs/draft/adr-0095-a-behavioural-strategy-arrives-as-a-field-over-cells.md`
[^5]: Backlog item 0039, a rejected unit is not stuck. `docs/backlog/proposed/0039-a-rejected-unit-is-not-stuck.md`
[^6]: ADR-0152, a faction plans its roads and zones with one solver, decision D5. `docs/adrs/accepted/adr-0152-a-faction-plans-its-roads-and-zones-with-one-solver.md`
[^7]: Recurring defect shapes, shape 1. `.agents/rules/recurring-defects.md`
[^8]: ADR-0145, a unit type is a row of capability columns, and zero means cannot, decisions D1 and D2. `docs/adrs/accepted/adr-0145-a-unit-type-is-a-row-of-capability-columns-and-zero-means-cannot.md`
[^9]: ADR-0125, the control plane names the seed set of a destination field, decision D4. `docs/adrs/draft/adr-0125-the-control-plane-names-the-seed-set-of-a-destination-field.md`
[^10]: Decision Record Scope, section 4.1. `.agents/rules/adr-scope.md`
[^11]: Decisions register, DEC-274. `docs/DECISIONS.md`
[^12]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader, decision D1. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
