---
id: 0040
title: A god inflicts weather on a place
created: 2026-09-03
---

# PRD-0040 — A god inflicts weather on a place

## Who this is for

A developer who builds a game in which a god directs a congregation. The god
is a person or a language model, and it acts through the control plane.

A modeller needs this second. A modeller wants to put a condition on a named
region and watch what the population does with it, and a divine power is the
cheapest way to place one.

## What the person cannot do today

A god cannot change the world. It can send its people somewhere, order them to
gather, order them to build, and speak to another faction. Every one of those
acts through a unit. Nothing the god does reaches the ground itself.

This has two costs.

The god has no act that a bystander can see. Every power it holds is a change
to what its own units do next, so a second god watching the map sees only
units moving. A power that changes the place is what makes a god legible.

A modeller cannot place a condition. The world now makes weather on its own,
and it makes it where the sea is. A modeller who wants a drought or a storm at
a named place has to wait for one, so a study of how a population answers a
condition cannot be set up at all.

## What good looks like

Each statement below can be checked.

- A god names a set of places in one call, and the engine answers once.
- The call is refused as a whole, or it is applied as a whole. One refusal
  leaves the world exactly as it was.
- A watcher reads the condition at a named place before the call and after it,
  and the two readings differ.
- Something in the simulation behaves differently because of the call, and a
  developer can point at the difference.
- The power is bounded. A statement says what a god may not do, and the engine
  refuses an attempt to do it.
- The bound is the same bound the rest of the engine uses. A god does not
  escape the rule that a faction acts where its people are.
- Two gods that make the same calls from one seed give the same world, at
  every thread count.
- A refused call costs the god nothing. A caller that mistyped an address does
  not lose the power.

## What this does not do

- It does not give a god a second congregation, or two gods one congregation.
  The engine holds factions, and a god directs one.
- It does not let a god take weather away. This record asks for a power that
  puts a condition on a place. Ending one is a separate need.
- It does not decide what weather changes. Which passes read the condition is
  an architectural question, and it belongs in a decision record.
- It does not price the power. What one storm should cost, and how often a god
  should act, are numbers nobody has stated.
- It does not give the god a forecast. The god sees the world as it is.
- It does not put the power in the simulation. A god calls a verb, and no unit
  and no pass calls it.

## What it costs at the target scale

The cost driver is the number of places the god names, not the size of the
world and not the size of the congregation.

A call resolves each place, checks a gate for each place, and writes one entry
for each place. Nothing in it walks the population, and nothing in it walks
the world.

Three properties follow. A solution must have all three.

- What one call costs grows with the number of places, and the number of
  places has a ceiling.
- The condition the call places costs what the condition already costs. The
  call adds no pass of its own.
- The gate the call checks is a lookup, not a scan. A gate that scanned the
  ground of a place would cost the block, and a block is large.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^1]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape and not a number.
- **One blocker holds what the power should cost.**[^2] How strong a storm may
  be, how many places one call may name, and how long a god waits between
  storms are all values that nobody has stated. The engine holds a value for
  each, and the blocker says that no measurement and no owner chose them.

This record depends on the world holding weather at all, and on a faction
holding ground. Both exist.[^3] [^4]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-130. `docs/BLOCKERS.md`
[^3]: PRD-0004, the world has weather that a watcher can read. `docs/product/accepted/prd-0004-the-world-has-weather-that-a-watcher-can-read.md`
[^4]: PRD-0006, a place belongs to somebody. `docs/product/accepted/prd-0006-a-place-belongs-to-somebody.md`
