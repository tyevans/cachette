---
id: 0024
title: A run stays eventful for as long as it is watched
status: Idea
created: 2026-09-03
---

# PRD-0024 — A run stays eventful for as long as it is watched

## Who this is for

A researcher who studies what agents inside the world come to believe about it,
and who therefore needs the world to keep giving them something to believe
about.

## What the person cannot do today

**A run goes quiet, and then stays quiet.**

The world is loud when it is founded and near silent afterward. In a run of
sixty ticks on a world of forty-eight by forty-eight tiles with three factions
and two hundred and forty three added workers, the engine recorded events on
six ticks. The founding accounted for the largest of them. Gathering happened
on two ticks near the start and never again. The rest of the run recorded
occasional changes to the ground and nothing else.

Two behaviours produce this together. A worked tile is spent and does not
recover, so the ground near a settlement is bare within a few ticks. A person
stays where they were placed, so nobody reaches ground that is not yet bare.
A population that cannot move, standing on ground that cannot recover, has
nothing left to do.

This has two costs for this audience.

**A study of belief has nothing to study.** An agent that perceives, remembers,
forgets and tells needs a supply of things to perceive. Where the world falls
silent, every agent's account converges on the founding, and the differences
between accounts are differences in noise rather than in what anybody saw.

**The length of a run stops meaning anything.** A run of a thousand ticks holds
the same history as a run of ten. Time is the axis every question about memory
is asked along, so a run that does not fill it cannot answer one.

## What good looks like

- A run of a thousand ticks records events on most of its ticks, and not only
  near the start.
- The events are spread across the world rather than gathered at the founding
  sites.
- A person standing anywhere in a settled region has something happen within
  sight of them, often enough to remember.
- Event volume settles to a rate rather than falling to zero. A world may be
  quieter late than early, and it may not be silent.
- The same seed produces the same history.

## What this does not do

It does not ask for more kinds of event. The kinds the engine already records
are enough, if they keep happening.

It does not ask for a busier world. A rate that is steady and low is a pass.

## Which needs it sits on

This record states an outcome. Three records already state behaviours that
would produce it, and this one exists because none of them names the outcome
and because meeting one alone does not reach it.

- A depleted deposit that comes back gives the ground a reason to be worked
  twice.[^1]
- A unit that goes somewhere it cannot see gives a person a reason to leave
  bare ground.[^2]
- A unit that consumes to continue gives both of them a reason to happen at
  all.[^3]

If those three land and a run still goes quiet, this record is the one that
says so. If they land and it does not, this record ships with them.

## References

[^1]: PRD-0018, a depleted deposit comes back.
`shaped/prd-0018-a-depleted-deposit-comes-back.md`

[^2]: PRD-0020, a unit goes somewhere it cannot see.
`shaped/prd-0020-a-unit-goes-somewhere-it-cannot-see.md`

[^3]: PRD-0013, a unit consumes to continue.
`accepted/prd-0013-a-unit-consumes-to-continue.md`
