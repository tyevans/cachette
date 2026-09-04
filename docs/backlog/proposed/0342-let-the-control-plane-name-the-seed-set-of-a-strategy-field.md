---
id: 0342
title: Let the control plane name the seed set of a strategy field
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: [PRD-0020]
blocked-by: []
---

## Why

The engine derives a reach for each faction and each level 1 cell, seeded at
every live site of that faction, and a laden unit climbs it home. The core
answers that direction today, so the mechanism is built rather than
proposed.[^1]

**The seed set is fixed by the record.** So a unit goes to a site of its faction
and nowhere else. A developer cannot send a set of units to a mountain, to a
frontier, or to a place a player chose, because none of those is a site.

Naming a set of addresses that seeds a plane gives two of the six things a
downstream game asks for: move units somewhere, and gather units in a place. No
unit gains a search, and the derivation, the relaxation and the tie-break all
stay as they are. The decisions register holds the options.[^2]

## What is missing before this can be refined

- The choice in the decisions register must close.[^2]
- How the record that fixes the seeds is amended. It is a draft, so it is edited
  rather than superseded, and the item must say which decision changes.
- How many planes a world may hold at once. The derivation is indexed by the
  cell and by the strategy, so a project that wants many strategies pays for
  each of them whether any unit holds it or not.

## References

[^1]: Findings register, FND-363. `docs/FINDINGS.md`
[^2]: Decisions register, DEC-142. `docs/DECISIONS.md`
