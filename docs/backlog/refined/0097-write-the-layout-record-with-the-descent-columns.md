---
id: 0097
title: Write the layout record with the descent columns
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: [ADR-0021]
serves: []
blocked-by: []
---

## Why

The character arena holds five columns and no parent edge. The descent and
succession pass is not built. A register row says the character tier wants
array-of-structs, and a reader takes that to govern the arena.[^1] The
correction says otherwise, and the decision row states the recommendation and
the reasoning.[^2] [^3]

Neither a register row nor a decision row binds anybody. The registry reserves
a number for the claim that layout follows the access pattern, and no file
holds it.[^4] The work that adds the descent columns is the work that can write
that claim honestly, because that work is the first to have a pass to point at.

## What the work does

1. Add the descent columns to the character arena: the two parent edges, the
   house, and the two Euler labels.
2. Write the record on the reserved row. State the constraint as the rule that
   a layout claim names one structure and one pass, and never a tier.
3. Give the alternatives that the decision row rejected, and say why the
   character row keeps its columns.
4. Cite the two research reports and the finding. Put no figure in the record.
5. Set the registry status to `Draft`, and close the decision row.

## What holds this back

**Nothing writes the trait record.** The array-of-structs recommendation covers
a separate 64-byte trait record for the personality influence pass.[^5] That
structure does not exist. The record must state the constraint that decides a
layout, not name a structure that nobody has written.[^6]

**The record must hold no measured figure.** The crossover between the two
layouts is a measured figure, and every figure in this project is derived or is
taken on a machine that is not the target.[^7] Put a figure in the reference
tables and cite it.

**Do not write the record before the columns land.** A record for a subsystem
that does not exist states an intent as a fact.[^8]

## Impact review

Not done. This item is `proposed` and needs one before anyone takes it.

## References

[^1]: Findings register, FND-022. `docs/FINDINGS.md`
[^2]: Findings register, FND-072. `docs/FINDINGS.md`
[^3]: Open decisions register, DEC-032. `docs/DECISIONS.md`
[^4]: ADR Registry, reserved row 0021. `docs/adrs/REGISTRY.md`
[^5]: Vector entity representation, section 9 and decision D155. `docs/research/reports/18-vector-entity-representation.md`
[^6]: Decision Record Scope, section 4.4. `.claude/rules/adr-scope.md`
[^7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^8]: Decision Record Scope, section 4.6. `.claude/rules/adr-scope.md`
