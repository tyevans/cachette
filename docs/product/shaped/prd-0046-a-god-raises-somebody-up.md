---
id: 0046
title: A god raises somebody up
status: Shaped
created: 2026-09-03
---

# PRD-0046 — A god raises somebody up

## Who this is for

A developer who builds a game in which a player directs a group of simulated
people, and who needs the player to change who matters in that group. The
player may be a language model.

## What the person cannot do today

A developer cannot make anybody notable, and cannot start a line.

The engine raises a person from the ranks when the deeds of a unit reach a
level. It holds the level, it holds how often it looks, and it holds a value
for how much each person is thought of. It also makes a person outright, bears
a child of two people, and ends a person. The control plane reaches none of
those six things.

This has three costs.

The developer cannot seed a world with anybody. A run starts with nobody
named, and it stays that way until the engine raises somebody by its own rule.
A game that wants a founder, a first family or a rival cannot state one.

The developer cannot tune how rare a person is. The level that a unit must
reach is fixed for the whole run, and how often the engine looks is fixed too.
A game that wants many small figures and a game that wants one hero in a
generation get the same world.

The developer cannot say that somebody matters. The engine holds a value for
how much a person is thought of, no pass writes it, and no caller can write it
either. It is therefore always zero, and a game cannot rank two people by it.

## What good looks like

Each statement below can be checked.

- A caller makes a set of people for one faction in one call, and receives an
  identity for each.
- A caller bears a set of children, each from two named parents, in one call,
  and receives an identity for each child.
- A caller ends a set of people in one call.
- A caller writes how much a set of people is thought of, in one call.
- A caller sets the level of deeds at which the engine raises somebody, and
  reads the level back.
- A caller sets how often the engine looks for somebody to raise.
- Every write is all or nothing. One refusal leaves the living population
  unchanged and raises a typed error.
- A caller cannot make an identity. Every identity a write takes is one the
  engine gave, and the engine refuses one it no longer holds.
- What a caller may not decide is stated. The engine chooses who it raises,
  and a caller only changes the level and the schedule.
- The same seed, the same world and the same calls give the same people, at
  every thread count, on every run.

## What this does not do

- It does not let a caller choose who the engine raises. The engine ranks the
  eligible by a rule of its own, and this need supplies the level and the
  schedule, not the choice.
- It does not decide when two people have a child on their own. A caller
  states a birth. Whether the world states one by itself is a separate need.
- It does not model courtship, a household or a marriage.
- It does not make how much a person is thought of change by itself. This need
  makes the value writable. What raises it and what lowers it is a separate
  need.
- It does not give a person a job, a title or a claim.
- It does not model an heir, a will or an inheritance.
- It does not model a name, a face or a description. Those are content.
- It does not let a caller give somebody an ancestor they were not born with.

## What it costs at the target scale

The cost driver is the number of people a caller writes in one crossing, not
the number of units in the world.

Two shapes are rejected. A write that takes one person, called once for each
person, pays a crossing for each member of the set. A write that half succeeds
makes the caller work out which half, and the caller must then read the whole
population back to find out.

These properties follow.

- Every write takes a set and crosses once.
- Every write resolves every identity and checks every argument before it
  changes anything.
- The cost of a write grows with the size of the set and not with the size of
  the population.
- How much a person is thought of is a fixed-point value, so two people
  compare the same way whatever order the work ran in.
- The level of deeds is a whole count and never a fraction.
- The engine raises people on a schedule, so the cost of looking is paid on the
  frames the schedule names and not on every frame.

No cost figure appears here, because the one measurement this project has that
was taken on the target platform does not cover the boundary.[^1] Every cost
statement above states a shape, not a number.

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape, not a number.

The ceiling on the number of people alive at once is answered, so this record
states no number.[^2] The project already holds that a person raised from the
ranks founds a line of their own and inherits nothing by blood, so a caller
that makes a person outright makes a founder.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-011. `docs/BLOCKERS.md`
