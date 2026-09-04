---
id: 0047
title: A game states its own economy
status: Shaped
created: 2026-09-03
---

# PRD-0047 — A game states its own economy

## Who this is for

A developer who builds a game on this engine from Python, and who wants the
numbers that govern the economy of that game to be the numbers of the game.

A named downstream game reads this first. It is a game of gods and
congregations, and its designer wants to change what a settlement earns, what
it owes, how often it earns, how fast a worked deposit returns, and how far a
faction reaches, without touching the engine.

## What the person cannot do today

**The developer cannot state a single economic number from Python.**

The engine holds every one of these knobs. It exposes none of them. A
developer who wants a settlement to earn twice as much has three choices. Fork
the engine and change a default. Write a Rust binding. Or accept the number
the engine happens to hold.

This has four costs.

The developer cannot tune a game. Balance is the largest part of the design
work of a strategy game, and every turn of that loop costs a compile of a Rust
crate.

The developer cannot ship two games from one engine. Two games differ first in
their numbers. A number that lives in the engine belongs to the engine, so a
second game is a second engine.

The developer cannot state what a faction reaches. What ground a faction
influences is driven by a value that no rule inside the engine writes. The
engine leaves that value for a caller, and no caller can reach it.

The developer cannot repair a world by hand. A test, a scenario, or a demo
that wants a settlement to start with a full store must gather for it.

## What good looks like

Each statement below can be checked.

- A developer sets what a settlement earns of a commodity, from Python, and a
  later step puts that amount into the store of that settlement.
- A developer sets what a settlement owes of a commodity, from Python, and a
  later step takes that amount out of the store.
- A developer sets how often the economy applies, from Python, and the store
  moves on the ticks the developer named and on no other tick.
- A developer sets what a settlement holds now, from Python, and a read of
  that settlement answers with the value written.
- A developer sets how fast a worked deposit returns, for each kind of
  resource, from Python, and a deposit that units drew from returns at that
  rate.
- A developer sets how much a unit must do before it becomes eligible for
  promotion, from Python.
- A developer names the settlement that a unit draws from, from Python.
- A developer sets what a faction injects at a place, from Python, and a later
  step spreads it.
- A developer reads back every value above, and the value read equals the
  value written.
- Every value the developer passes states its unit and its scale in the
  published reference. A developer never has to read the engine source to know
  whether one means one or one part in 65536.
- A value the engine refuses raises an error that names the value, and the
  world is unchanged. Nothing is half written.
- A write that names many places takes them all in one call and answers once.
  A developer never loops over places.
- Two runs that set the same values in the same order give the same world, at
  every thread count.

## What this does not do

- It does not add a knob the engine does not hold. Every value here already
  governs the engine. This record is about reach, not about mechanism.
- It does not add a second commodity. The world holds one, and the store
  therefore answers how much arrived and never what arrived.
- It does not let a game define a new kind of resource, a new ground, or a new
  improvement. Those are catalogues, and a catalogue is a separate need.
- It does not add a rule that decides a value. The engine holds no rule that
  writes what a faction injects at a place, and this record adds none. It gives
  a caller the write.
- It does not state a default. What the engine starts with stays what the
  engine starts with.
- It does not give a developer a way to describe a whole economy in one file.
  A configuration format is a separate need.
- It does not change what the engine does with any of these values.

## What it costs at the target scale

**A write must not follow the population.** The world holds one million units
and a settlement for a group of them. A developer who sets a rate for every
settlement must pay for the settlements and never for the units that live in
them, and must cross the language boundary once and not once for each place.

These properties follow, and this record requires them.

- A write over a set of places costs one crossing, whatever the size of the
  set. The caller passes the places as one column.
- A write over a set of places costs the size of the set and nothing more. It
  reads no unit and it visits no tile.
- A read of one place costs one crossing and answers about that place alone.
- A world-wide value, such as how often the economy applies, is one write for
  the world. It is never a write for each place.
- Setting a value adds no storage. Every value here already has a home in the
  engine.
- A value that a later step reads is part of what makes two runs the same run.
  Setting it must therefore reach the check that compares two runs.

No cost figure appears here. One blocker holds every cost figure this record
would state, and it says which figures a run has measured.[^1]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape and not a number.
- **One blocker governs what a downstream game needs.**[^2] It holds the list
  of things the named game must do that the control plane refuses today. This
  record answers the economic part of that list and not the whole of it.

No other blocker governs this record. Every value it names already exists in
the engine with a unit and a scale, so nothing here waits on a number the
project has not chosen.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
