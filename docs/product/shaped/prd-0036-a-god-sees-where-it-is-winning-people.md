---
id: 0036
title: A god sees where it is winning people
status: Shaped
created: 2026-09-03
---

# PRD-0036 — A god sees where it is winning people

## Who this is for

A developer who builds a strategy game on this engine, in which a side can gain
a unit that another side used to hold.[^1]

The same real project needs it. In that game a god acts on the world, and the
people of another god come over on their own, a few at a time. The god that
acted does not choose who comes over and cannot know in advance who will.

## What the person cannot do today

**A developer cannot tell that anything happened.**

A change that a side did not ask for, and that no report describes, is
invisible until a headcount happens to move. A developer who watches only the
headcount learns three things too late.

**Where.** A headcount is one number for the whole world. It says a side gained
people and never says which part of the map they came from. A game whose whole
subject is belief spreading across a map cannot draw that map.

**From whom.** A headcount that rose says nothing about which side fell. In a
world with several sides, a developer cannot tell whether the people came from
a rival or from a neutral.

**Which people.** A developer who was following one person cannot tell that
they changed side, because nothing named them.

There is a further cost. A developer who has only a headcount cannot debug the
rule. An act on the world that produced nothing looks exactly like an act that
the developer has not waited long enough for.

## What good looks like

Each statement below can be checked.

- A developer reads which units changed side on the last step, in one call, and
  the call visits no unit from Python.
- Each entry names the step, the person, the place, the side that lost them and
  the side that gained them.
- The report covers changes the world produced and changes a developer asked
  for, in one place, so a developer never merges two reports to get one answer.
- A step in which nobody changed side reports nothing, and reporting nothing is
  distinguishable from not having asked.
- The report describes the step that just ended, and it crosses the boundary
  once for that step rather than once for each unit.
- A developer reads how far each side reaches at a place, so it can see where
  the next change is likely, before it happens.
- The counts of how many units each side holds agree with the report, with no
  recount and no second call.
- The same world gives the same report at every thread count and on every run.

## What this does not do

**It does not keep a history.** The report describes the last step. A developer
who wants a season of them keeps them.

**It does not explain a decision.** The report says what happened, not why the
engine chose one person over another. A developer who wants to reason about
likelihood reads how far each side reaches at the place.

**It does not predict.** Nothing here says who will change side next.

**It does not add a new total.** How many units each side holds is already
reported, and this need is answered without a second number that could disagree
with the first.

**It does not draw anything.** A picture of where belief is moving is the
game's business. The engine gives the places and leaves the drawing alone.

**It does not notify.** A developer reads the report after a step. Nothing
calls back into the game, because no game code runs while a step runs.[^2]

## What it costs at the target scale

**The report follows what changed and never what exists.** A world in which
nobody changed side produces an empty report, whatever the population.

**One crossing for each step.** The whole report crosses the boundary once, in
the batch that already crosses at the end of a step. A report delivered one
entry at a time would cost one crossing for each person, which is the cost the
project's control plane rule exists to prevent.[^3]

**The read of how far a side reaches at a place is one lookup.** It reads the
summarised level the caller already reads, and it walks nothing.

**No figure is stated here.** One blocker governs every cost figure in this
project.[^4] The statements above are arguments about which term the cost
follows, not results.

## Which blockers govern this

**One blocker governs every cost claim above.**[^4] Nothing here was measured.

**One blocker governs the game this need came from.**[^5] What that game shows
a player, and how often, is a game rule. The need above states what the engine
must make readable, and it leaves the presentation out.

**Nothing here waits on a question the project owner holds about the engine.**

## References

[^1]: PRD-0035, a god takes the people of another god. `docs/product/shaped/prd-0035-a-god-takes-the-people-of-another-god.md`
[^2]: Project orientation, the hard invariants. `CLAUDE.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-050. `docs/BLOCKERS.md`
