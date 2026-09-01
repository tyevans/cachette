---
id: 0015
title: A unit has parents and children
status: Accepted
created: 2026-08-31
---

# PRD-0015 — A unit has parents and children

## Who this is for

A developer who builds a strategy game on this engine, and who needs a
population to hold a line that outlives any one member of it.

## What the person cannot do today

A developer cannot say where a unit came from.

A unit that is born stands in no relation to any other unit. It arrives, it
holds a job, and it ends. The population is therefore a set of strangers, and
the set is the same set after every member of it has been replaced.

This has three costs.

The developer cannot make a death reach past the unit that died. Nobody is
left behind, because nobody was attached. A loss that nothing survives is a
change to a count.

The developer cannot let a watcher follow anything across a generation. A
watcher can follow one unit for as long as it lives. When it ends, the thread
ends with it, so a run of many lifetimes holds no continuity at all.

The developer cannot make anything pass from one unit to another by right. A
claim, a position or a holding needs somebody with a reason to receive it, and
no unit has a reason to receive anything.

## What good looks like

Each statement below can be checked.

- A unit that is born has a recorded parent, and a watcher can ask who it is.
- A watcher can walk from a unit to its ancestors and from a unit to its
  descendants.
- The relation between two units is a value the world computes, and the value
  is exact.
- A unit can found a line with no ancestors. Its relation to every existing
  unit is zero, and the world says so rather than inventing a parent.
- Units who live together form a household, and a watcher can ask who is in
  one. A household follows from where people live. It is not a second fact
  that somebody declares.
- A line can end. When no descendant remains, the world reports that the line
  ended.
- The record of descent survives the death of the unit it names. A watcher can
  ask who the parent of a living unit was after that parent has died.
- The same seed and the same world give the same parentage and the same
  households, at every thread count, on every run.
- A unit is never recorded as its own ancestor.

## What this does not do

- It does not decide who has children, or when. That belongs with birth.
- It does not model marriage as a negotiated event. Whether two units pair by
  a rule the world applies or by a choice one of them makes is a separate
  need.
- It does not model a heritable trait, a statistic that passes down, or
  resemblance between a parent and a child.
- It does not model succession, a title or an inheritance. Who takes a
  position when its holder dies is a separate need. This record supplies the
  descent that such a rule may read.
- It does not keep descent for every unit that ever lived. How far back a line
  is kept is a design question, and the cost governs it.
- It does not name an extended relation. Whether the world has a word for a
  cousin is a viewer question, not a simulation question.
- It does not model adoption, a false claim of descent, or an unknown parent
  as a category with its own rules.
- It does not decide where a child is born or who raises it. Where a unit
  lives is a separate need.

## What it costs at the target scale

The cost driver is the number of lines the world keeps, not the number of
units alive.

Two shapes are rejected. A full ancestor tree for every unit in the target
population stores a history for a unit that nothing ever asks about. A
relation measured by walking two lines up to a common ancestor pays the depth
of the lines for each pair, and a pair is asked for often.

These properties follow.

- Descent is kept for a bounded set of units, not for every unit in the world.
  The size of that set is answered, so this record states no number.[^3]
- A parent is read at a bounded cost. The cost does not grow with the
  population and does not grow with the depth of the line.
- A relation between two units is an exact integer or a fixed-point value, so
  a comparison between two relations gives one answer whatever order the work
  ran in.
- A line that ends releases what it held. Storage grows with the lines that
  live, not with every unit that has ever been born.
- A recorded parent names an identity that can never be reissued. A reference
  to a dead parent resolves to that parent or to nothing, never to a different
  unit.
- Births are recorded in an order fixed by a stable key, never by thread
  completion order.

No cost figure appears here, because the one measurement this project has was
taken on a development machine and not on the target.[^1] That measurement
established the shape that matters: the term that grows with the number of
things dominates the term that grows with the number of tiles, and a line is
one of those things.[^2]

## Which blockers govern this

- **No measurement exists on the target platform.**[^1] Every cost statement
  above states a shape, not a number.

The size of the population that carries a line is answered, so this record
states no number.[^3] The project already holds that a unit raised from the
ranks receives no invented ancestry, so this record requires a line that
starts at zero to be expressible.[^4]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Findings register, FND-049. `docs/FINDINGS.md`
[^3]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^4]: Blockers register, BLK-011. `docs/BLOCKERS.md`
