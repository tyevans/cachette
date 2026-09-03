---
id: 0016
title: Somebody is in charge
status: Accepted
created: 2026-08-31
---

# PRD-0016 — Somebody is in charge

## Who this is for

A developer who builds a strategy game on this engine, and who needs a faction
to be somebody rather than a colour.

## What the person cannot do today

A developer cannot point at anybody who decides for a faction.

A faction is a label. It marks a unit and it marks ground. Nothing in the
world holds the position of deciding, so a faction has no character, no
intent, and nothing that can change about it.

This has three costs.

The developer cannot make two factions differ except by position. Every
faction applies the same rules in the same way. A faction on good ground does
well and a faction on poor ground does badly, and that is the whole of the
difference between them.

The developer cannot make one death matter more than another. The unit that
runs a faction is worth exactly what any other unit is worth, because there is
no such unit. Nothing in the world is worth striking at.

The developer cannot produce a crisis. A crisis is a period when who decides
is in doubt. A faction whose decisions come from a fixed rule can never be in
doubt, so its history has no turning points.

## What good looks like

Each statement below can be checked.

- A faction has at most one ruler at a time, and a watcher can ask who it is.
- The ruler is a unit in the world. It stands somewhere, a watcher can find
  it, and it can end.
- What the ruler is changes what the faction does. A watcher can replace the
  ruler in an otherwise identical world and see the faction behave
  differently.
- When a ruler ends, another unit takes the position by a rule the world
  applies. Nothing outside the simulation chooses.
- The rule reads the world. Who succeeds depends on which units exist and on
  what they are, not on a list fixed before the run.
- A faction can hold the position vacant for a time, and the world states what
  a faction without a ruler does.
- Two units can claim the position at once, and the world resolves the claim
  by a stated rule. A watcher can see the contest and its outcome.
- A watcher can read who has ruled a faction, in order, back to the founding.
- The same seed and the same world give the same rulers and the same
  successions, at every thread count, on every run.

## What this does not do

- It does not model government. A council, an assembly, a law and an office
  below the ruler are not this need.
- It does not decide the succession rule. Descent, appointment and strength
  are candidates. This record needs one rule that works and that reads the
  world.
- It does not decide which properties of a ruler change what a faction does. A
  small fixed set is the need. A catalogue of traits is not.
- It does not give a person control of a ruler. This is a simulated ruler in a
  simulated faction.
- It does not model an order travelling. Whether a decision takes time to
  reach a distant unit is a separate need.
- It does not model a title held apart from the position. One faction has one
  position, and holding it is the whole of being the ruler.
- It does not model deposing, exile and abdication as three systems. It needs
  one rule by which the position falls vacant.
- It does not model diplomacy between rulers. Whether two factions agree on
  anything arrives with trading.

## What it costs at the target scale

The cost driver is the number of factions. That number is bounded and small,
and it is answered, so the rule that runs the rulers is not where the time
goes.[^3]

The cost that matters is the one this need can push onto the units. A unit
that follows a link to its faction, and then to that faction's ruler, each
time it chooses puts two indirections on the hot path for the whole
population. That is the wrong shape, and this record rejects it. A succession
that searches the population for a candidate is the second wrong shape, and
this record rejects that too.

These properties follow.

- What a ruler changes reaches a unit as a value the unit already reads. A
  unit's choice does not walk to the ruler.
- The cost of running the rulers grows with the number of factions. It does
  not grow with the population and it does not grow with the size of the
  world.
- A succession considers a bounded set of candidates. The set is derived from
  what the world already indexes, not by a search over every unit.
- A ruler changes at a frame barrier. No unit reads a half-changed faction
  inside one tick, and no unit in a tick sees two different rulers.
- Every value that decides a succession is an exact integer or a fixed-point
  value, so two candidates compare the same way whatever order the work ran
  in.
- Ties between claimants break by a stable key, never by thread completion
  order.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and this record
must not add to that term.[^2]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] It says which figures
  are measured and which are derived. Every cost statement above states a
  shape, not a number.

The faction ceiling is answered, so this record states no faction count.[^3]
The size of the population that can hold a position is answered, so this
record states no candidate count.[^4] The project already holds that a unit
raised from the ranks cannot inherit a position by blood but may be appointed
to one, so this record requires a succession rule that admits appointment.[^5]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-013. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-011. `docs/BLOCKERS.md`
