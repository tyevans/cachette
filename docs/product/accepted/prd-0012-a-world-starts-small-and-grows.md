---
id: 0012
title: A world starts small and grows
status: Accepted
created: 2026-08-31
---

# PRD-0012 — A world starts small and grows

## Who this is for

A developer who builds a strategy game on this engine, and who needs a run to
have a beginning.

A modeller needs this too. A modeller studies how a quantity grows, and a
growth curve needs a start that the model did not choose for its answer.

## What the person cannot do today

A developer cannot begin a run from a founding condition.

A run starts with a population that somebody chose, spread across the world by
a rule with no reason behind it. The world at tick zero is therefore the same
size as the world at tick one million. Nothing began, so nothing can grow.

This has three costs.

The developer cannot show growth. A population that starts at its target has
nowhere to go. Every rule that makes a population rise or fall acts on a
number that is already at the ceiling, so the rule cannot be seen to work.

The developer cannot judge a place. A group placed everywhere is placed
nowhere in particular. Nothing in the world says that one spot was a good
choice and another was a poor one, so the map has no good places.

The developer cannot make the early run interesting. The first hundred ticks
show a full world doing what a full world does. A beginning is the part of a
run where a small change has a large effect, and this world has no beginning.

## What good looks like

Each statement below can be checked.

- A run begins with a small group. The size of the group is an input to the
  run, and it is not the target population.
- The engine chooses where the group starts by reading the world. A watcher
  can ask which properties of the place made it the choice.
- A different seed gives a different founding place, and the new place answers
  the same test as the old one.
- A watcher can see the founding place and can compare it against the places
  that were not chosen.
- The group changes something a watcher can see inside the first hundred
  ticks.
- A group founded in a poor place does worse than a group founded in a good
  place, and a watcher can see the difference.
- The population at the end of a run follows from the run. It does not follow
  from the number the run started with.
- The same seed gives the same founding place and the same founding group, at
  every thread count, on every run.
- A run that starts with a small group costs the small group, not the target
  population.

## What this does not do

- It does not decide how large the founding group is. That is an input to the
  run.
- It does not decide which properties make a place good. Water, food, high
  ground and reachable ground are candidates. This record needs a rule that
  chooses for a reason, not a list of reasons.
- It does not decide how the group grows. Growth follows from birth, from
  consumption and from housing, and each of those is a separate need.
- It does not let a person edit the starting condition, save one, or load one
  from a file. The seed and the group size are the whole input.
- It does not model a second founding. Whether a group can split off and found
  another place is a separate need.
- It does not promise that a founding succeeds. A group may fail, and a failed
  founding is a correct outcome.
- It does not decide how many groups found a world. The engine is told the
  number, and this record states none.
- It does not change the ground. The world the generator made is the world the
  group arrives in.

## What it costs at the target scale

Two cost drivers matter, and they act at different times.

The first acts once, before the first frame. A founding that scores every tile
in the world to choose a place pays the whole world before anything runs. The
project already refuses that shape: a developer who changes a seed must see
the new world at once. This record refuses it again.

The second acts for the whole run. A world sized for its founding group cannot
reach its target population without growing its storage under a running
simulation. A world sized for its target pays the target from tick zero, while
a hundred units live in it.

These properties follow.

- Choosing the founding place costs a bounded sample of the world. It does not
  cost a pass over every tile.
- The cost of a tick grows with the number of units that live, not with the
  number of units the world is sized for.
- The storage the world reserves is sized for the target population and does
  not change during a run. A run does not stop to grow.
- The founding choice is ordered by a stable key. Two candidate places that
  score the same resolve by that key, never by thread completion order.
- Every score in the choice is an exact integer or a fixed-point value, so the
  comparison gives one answer whatever order the work ran in.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and this record
makes that number start low.[^2]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] It says which figures
  are measured and which are derived. Every cost statement above states a
  shape, not a number.

The target population is answered, and it counts everybody rather than the
soldiers alone, so this record states no ceiling of its own.[^3] The settlement
count is answered, so this record states no number of founding places.[^4] The
number of founding groups is answered, so this record states no number of its
own.[^5]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-003. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-005. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-018. `docs/BLOCKERS.md`
