---
id: 0045
title: A god knows its congregation by name
status: Shaped
created: 2026-09-03
---

# PRD-0045 — A god knows its congregation by name

## Who this is for

A developer who builds a game in which a player directs a group of simulated
people, and who drives the engine from the control plane. The player may be a
language model. A language model reasons about a named person with a history.
It cannot reason about a count.

## What the person cannot do today

A developer who drives the engine from the control plane cannot name one
person in the world.

The engine holds a character tier. A character carries a faction, a birth
tick, a renown, a sex, a house and two parents. The engine records descent for
every character it has ever made, and it computes an exact relation between
two of them. The control plane reaches none of it. A search of the control
plane package finds no member that answers any of those facts.

This has four costs.

The developer cannot tell a player who its people are. The player sees a unit
count for each faction and nothing else. Every population of the same size
looks the same.

The developer cannot make a death land. A unit that ends is one less in a
count. Nobody is named, so nobody is missed, and a player cannot say who died.

The developer cannot follow anything across a generation. The engine keeps the
parent of a dead character for exactly this reason, and no caller can read it.

The developer cannot answer "who is this person to me". The engine computes
the relation between two characters exactly, and the value never leaves the
engine.

## What good looks like

Each statement below can be checked.

- A caller asks once and receives every living character of the world, or
  every living character of one faction, as parallel columns.
- The answer carries, for each character, an identity the engine minted, the
  faction, the birth tick, the renown and the sex.
- A caller asks once for the lineage of one character and receives the
  parents, every ancestor and every descendant in the same answer. The caller
  never walks the record one step at a time.
- Each person named in a lineage answer carries the identity the engine minted
  at their birth, and a flag that says whether they are alive now.
- A caller asks once for the relation between one character and a set of other
  characters, and receives one exact value for each member of the set.
- A caller reads what a unit has done, for a set of units, in one call.
- A caller reads which character a unit was raised into, for a set of units, in
  one call.
- An identity the world does not hold is refused with a typed error, and the
  refusal changes nothing.
- Every number the answer carries states its unit, and a fixed-point value
  says so.
- The same seed and the same world give the same answers, at every thread
  count, on every run.

## What this does not do

- It does not change what the engine simulates. It states what a caller can
  read, and nothing more.
- It does not decide who has children, or when. That belongs with birth.
- It does not name an extended relation. The engine answers with a number, and
  whether a game has a word for a cousin is a question for the game.
- It does not give the relation between two people when one of them is dead.
  What the engine can compute is a separate question from what a caller wants.
- It does not model a biography, a memory or a spoken word.
- It does not decide how a player chooses what to read. A selector is a
  separate need.
- It does not publish a name as a string. The engine holds an identity and a
  set of facts. A name is content, and content belongs to the game.
- It does not give a house a title, a seat or a claim.

## What it costs at the target scale

The cost driver is the number of people a caller asks about in one crossing,
not the number of units in the world.

Two shapes are rejected. A read that answers about one person, called once for
each person, pays a crossing for each member of the population, and the number
of members is the thing that grows. A lineage handle that a caller follows one
edge at a time pays a crossing for each edge, and a line is deep.

These properties follow.

- Every read answers about a set and crosses once. The cost of a read grows
  with the size of the answer and not with the size of the population.
- The answer is a block of numbers with one entry for each row. It is not a
  list of objects that the caller unpacks.
- Every value in an answer is an exact integer or a fixed-point value, so two
  answers compare the same way whatever order the work ran in.
- The order of every answer is fixed by a stable key and never by thread
  completion order.
- A read takes a copy of what it answers, and the copy is stated at the call.

No cost figure appears here, because the one measurement this project has that
was taken on the target platform does not cover the boundary.[^1] Every cost
statement above states a shape, not a number.

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape, not a number.

The size of the population that carries a line is answered, so this record
states no number.[^2] The project already holds that a person raised from the
ranks receives no invented ancestry, so a lineage answer with no ancestor at
all is a real answer and not a missing one.[^3]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-004. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-011. `docs/BLOCKERS.md`
