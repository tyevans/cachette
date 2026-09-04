---
id: 0341
title: Bind the build order and the upgrade removal to the control plane
status: complete
created: 2026-09-03
implements: [ADR-0040 D1, ADR-0043 D1, ADR-0046 D1, ADR-0085 D3, ADR-0090 D1, ADR-0090 D2, ADR-0090 D4, ADR-0107 D2, ADR-0110 D1]
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

The core orders a build for one unit and one kind, stops that order, reports it,
and removes a finished upgrade at an address. No line of the bindings crate
names any of the four, and no Python file names them either. The findings
register holds the search that measured it.[^1]

So a developer who wants to build anything can only found a settlement, which
creates an entity rather than marking the ground. A downstream game names
building as one of the six things its players must do.[^9]

This is a capability that nothing invokes, which is a shape this project already
lists.[^2] The mechanism is built and its own tests pass. Nothing reaches it.

## Impact review

**Governed by.**

- ADR-0040 D1. Python is a control plane. A write verb takes a set and answers
  once, and Python never loops over units.[^4]
- ADR-0043 D1. A soldier is the mass tier, so no interface may ask a caller to
  hold one identity at a time for a write.[^5]
- ADR-0046 D1. Every refusal crosses as a typed error under one root class.[^6]
- ADR-0085 D3. An identity crosses as one opaque number, and the engine
  resolves it against the generation. A dead identity is refused rather than
  answered for the next occupant of the slot.[^7]
- ADR-0090 D1, D2 and D4. The upgrade store is sparse, a build holds progress
  between ticks, and a removal returns the tile to the generated world.[^3]
- ADR-0107 D2. The prose of each binding lives in the Rust doc comment, and the
  published reference is generated from an import of the compiled module.[^8]
- ADR-0110 D1. The return field holds one direction for each level 1 cell and
  each faction.[^10]

**Changes.** None. No record changes, and the core keeps its behaviour.

**Creates.** No decision record. The work states no new constraint: every claim
it needs is already in the records above, and a record that only described the
binding would be a description rather than a constraint.[^11] The shape choices
go to the decisions register instead.

**Blockers.** BLK-036 governs whether an upgrade changes hands when the ground
does.[^12] The work states no answer to it. BLK-007 governs every cost figure,
so the work states none.[^13]

**Precedent.** FND-352 records that three integer scales share the name `kind`,
and that a range check cannot separate two numberings that overlap.[^14] The
upgrade kind is a fourth such scale, and it overlaps the other three. The doc
comment must name the hazard, and item 0331 owns the repair.

FND-360 holds the search that found the gap.[^1]

## Done when

- The control plane orders a build for a set of units in one call, and stops the
  order for a set of units in one call.
- The control plane removes the upgrade at a set of addresses in one call.
- The control plane reads back the build order of one unit, and reads back the
  upgrade on one tile with its progress.
- The control plane reads the return direction of one faction at one address.
- Each verb resolves every identity and checks every argument before it writes,
  and one refusal leaves the world unchanged and raises a typed error.
- A test drives each path from the Python boundary, and a defect put back into
  each path makes that test fail.
- The doc comment of each new member states every argument, its type, its unit,
  its default and the error class the call raises.
- A test pins every number that a new doc comment states.
- The whole check command runs green.

## Outcome

**Six members are new on the `World` class, and one report gained three keys.**
The set-valued write verbs are the build order and the stop order, which take a
set of identities, and the removal, which takes a set of addresses and answers
with the number of upgrades it removed. The reads are the build order of one
unit, the direction home for one faction at one address, and the offset of each
direction. The report of one tile now carries the upgrade, the work done and
whether the work is finished.

**The core is unchanged.** Every method the bindings call was already public,
so no core change was needed to make the capability reachable.

**Two findings and two decisions went to the registers.** FND-380 records that a
unit builds on ground of any faction, and that the rule which says otherwise
reaches no code. FND-381 records that a finished terrace changed nothing the
control plane could read. DEC-160 chose the answer of the removal. DEC-161 put
the holding rule in the core rather than in a binding.

**One item came out of it.** Item 0370 holds the core change that enforces the
holding rule.

**What changed from the plan.** The plan named three core methods. The work
bound five, because the stop order and the read of the build order are the rest
of the same capability, and because the return direction is unusable without the
table of direction offsets. The plan did not foresee that a finished terrace was
invisible from Python, and the report of one tile gained the three upgrade keys
for that reason.

**The work opened no blocker.** A blocker number was allocated to it and stays
unused, so the register holds no new row. The one open question that touches
building is BLK-036, and it was open before this work started.

## References

[^1]: Findings register, FND-360. `docs/FINDINGS.md`
[^2]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^3]: ADR-0090, a tile upgrade is stored sparsely, as the difference from the generated world. `docs/adrs/draft/adr-0090-a-tile-upgrade-is-stored-sparsely.md`
[^4]: ADR-0040, Python is a control plane, not a data plane. `docs/adrs/draft/adr-0040-python-is-a-control-plane-not-a-data-plane.md`
[^5]: ADR-0043, a declared tier enforces the no-loop rule, and the API refuses the loop. `docs/adrs/draft/adr-0043-a-declared-tier-enforces-the-no-loop-rule.md`
[^6]: ADR-0046, every error is typed. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
[^7]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^8]: ADR-0107, the Python reference is generated from the compiled module. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^9]: PRD-0030, a developer builds a game the engine did not anticipate. `docs/product/shaped/prd-0030-a-developer-builds-a-game-the-engine-did-not-anticipate.md`
[^10]: ADR-0110, a unit returns by climbing a reach field seeded at every site of its faction. `docs/adrs/draft/adr-0110-a-unit-returns-by-climbing-a-reach-field.md`
[^11]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^12]: Blockers register, BLK-036. `docs/BLOCKERS.md`
[^13]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^14]: Findings register, FND-352. `docs/FINDINGS.md`
