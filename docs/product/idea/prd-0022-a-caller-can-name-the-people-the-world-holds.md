---
id: 0022
title: A caller can name the people the world holds
status: Idea
created: 2026-09-03
---

# PRD-0022 — A caller can name the people the world holds

## Who this is for

A researcher who runs language agents inside the world, and who must attach an
agent to a person who already lives there.

## What the person cannot do today

**A caller cannot address anybody it did not create itself.**

Founding a run seats a population. That population is the world: it holds the
factions, it works the ground, and it is what a run is about. A caller receives
a report of what the founding chose, and the report holds no identity for any
of the people it seated.

One verb hands back identities, and it is the verb that adds a person. So the
only people a caller can name are the ones it added, and those people are
outside the population the founding built. A researcher who wants an agent to
be one of the villagers must instead stand a stranger next to the village.

This has two costs.

**A study cannot sample its subjects.** An experiment picks its subjects from
the population under study. Here the population under study is unreachable, so
every subject is an addition to it, and the act of observing changes the thing
observed.

**An agent cannot be given a life it did not start.** A person seated by the
founding has a faction, a place and a job. A person the caller adds has none of
that history. An agent attached to the second is a tourist.

## What good looks like

- A caller reads the identities of the people the world holds, and the read
  does not change the world.
- A caller narrows that read to one faction, or to one window of the world.
- An identity read this way works with every verb that already accepts one.
- The read costs no more than the number of people it returns, and a caller
  that wants a count does not pay for the identities.
- The same seed returns the same people in the same order.

## What this does not do

It does not add a way to change a person. It is a read.

It does not promise that an identity outlives the person. A person who dies
leaves an identity that no longer names anybody, and that is correct.

## What is not worked out

The population is one million at the target scale. A verb that returns every
identity returns one million of them, which is a data-plane answer to a
control-plane question. The bound belongs in this record before it is shaped,
and this record does not yet hold it.
