---
id: 0145
title: Give the faction count one rule for zero
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The world settings hold a faction count. A count of zero is accepted, and the
engine then treats the world as holding one faction.

That rule is written at several places in the world module, each as
`faction_count.max(1)`. The settlement founding uses it to refuse a faction the
world does not hold. The run founding uses it to bound its loop. Two more
refusals use it, and the faction ceiling of the holding pass uses it. Test code
and the benchmarks repeat the same coercion again.

**Do not trust a count of the sites, including one this item states.** This
item first said six and an audit on 3 September 2026 found five in the module,
with more outside it. Nobody restated it and nothing failed, which is the same
defect the item is about: the number of declaration sites is itself a fact this
document declares a second time. **Derive the count from the tree when you
refine this**, and put the search command in the commit body.

Nothing fails when the copies disagree. A contributor who changes one of them,
or who adds a seventh site and forgets the coercion, gets a world whose parts
disagree about how many factions exist. This is the first shape of the recurring
defect rule, and the project has recorded it before.[^1]

A review of the founding record raised this against the record and then against
the code. The record is right to state the rule once. The code states it six
times.[^2]

## What the work does

Make the settings answer the question once. The candidate shape is a reader on
the settings that returns the effective faction count, so every caller reads one
value and no caller repeats the coercion.

A second candidate is to refuse a faction count of zero at construction, so no
coercion is needed anywhere. That changes the public interface, and it may break
a caller that builds a world with no factions on purpose.

## The questions this item must answer before it is refined

**Whether a world with no factions is a world the project wants.** If it is not,
the refusal is the smaller change and it removes the rule rather than centring
it.

**Whether the settings constructor item covers this.** Item 0080 gives the world
settings a constructor. If that item lands first, the refusal belongs in it.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Review 0143, the housing, growth, founding and recovery records. `docs/reviews/0143-the-housing-growth-founding-and-recovery-records.md`
