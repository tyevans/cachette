---
id: 0461
title: Tell a caller which arena an identity belongs to
status: proposed
created: 2026-09-03
---

## Why

Each arena numbers its own slots. The first unit of a world, the first
settlement of a world and the first character of a world therefore carry the
identical number. A call that takes a unit reads the soldier arena, a call that
takes a character reads the character arena, and neither refuses the number of
the other. A caller that mixes two kinds of identity gets a legal, wrong
answer, and no check reports it. A finding holds the measurement.[^1]

The mistake repeats exactly, because the engine is deterministic. Both
determinism tests compare a run against a run, so neither can see it.[^2]

The character work made this reachable in a new way: a caller now holds two
identities for one person, the unit and the character it was raised into.
Passing the wrong one of the two is a natural mistake and it raises nothing.

A decision row holds the options and the recommendation.[^3] The recommendation
is to put the arena into the identity, because it is the only option that makes
the mistake impossible rather than documented, and because the cost falls now
rather than after a game has stored identities.

This item also holds a second read that the same finding work turned up. A
caller cannot ask whether a line has ended, because the engine answers that
for a character who is gone and every boundary read takes a living
identity.[^4] A second decision row holds the options, and the recommendation
is a read that takes a house rather than an identity.[^5]

Refining this item should decide whether the two are one piece of work or two.
They share the question of what a caller may name that is not a living
entity.

## References

[^1]: Findings register, FND-472. `docs/FINDINGS.md`
[^2]: Testing rules, section 2. `.claude/rules/testing.md`
[^3]: Decisions register, DEC-266. `docs/DECISIONS.md`
[^4]: Findings register, FND-471. `docs/FINDINGS.md`
[^5]: Decisions register, DEC-265. `docs/DECISIONS.md`
