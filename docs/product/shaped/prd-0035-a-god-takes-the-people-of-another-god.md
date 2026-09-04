---
id: 0035
title: A god takes the people of another god
status: Shaped
created: 2026-09-03
---

# PRD-0035 — A god takes the people of another god

## Who this is for

A developer who builds a strategy game on this engine, and whose game is won by
taking people rather than by taking ground.

One real project needs it. In that game a god leads a congregation, and the
game is about belief spreading from one congregation into another. A person who
comes to believe stops belonging to one god and starts belonging to another.
The need is stated here in general terms, because any game in which a side can
gain a unit without producing it asks the same question.

The engine already gives each unit a side, and it already gives each side a
reach across the world that falls with distance.[^1] [^2] This record is about
the moment a unit stops belonging to one side and starts belonging to another.

## What the person cannot do today

**A developer cannot move a unit from one side to another at all.**

A unit takes its side when it is created and keeps it until it dies. There is
no call that changes it, and there is no rule inside the engine that changes
it.

A developer who wants the effect today has one route, and it is wrong in three
ways. The developer ends the unit and creates a new one for the other side.

**The identity breaks.** The engine hands out an identity that a caller keeps.
Ending a unit invalidates it, so every list, every order and every record the
game held about that person now names nobody. The person the game was telling a
story about is gone, and a stranger stands where they stood.

**What the unit carried is lost or duplicated.** A unit carries a load, a home,
a job and sometimes a named person. A developer who rebuilds all of that by
hand rebuilds it from what the control plane happened to remember, and anything
it forgot is silently dropped.

**The counts move for the wrong reason.** The engine reports a death and a
birth. A game that watches those numbers sees a war where there was a
conversion, and a watcher cannot tell the two apart.

There is a second gap beside the verb. **A developer cannot make the change
happen by itself.** A god that wants belief to spread has to decide, one unit
at a time, who believes. That is the loop over the population that the control
plane rule forbids.[^3]

## What good looks like

Each statement below can be checked.

- A unit changes side and keeps its identity, so every handle the caller holds
  still names the same person.
- A developer names a set of units and a side in one call, and every unit of the
  set changes side. The call visits no unit from Python.
- The call is all or nothing. A set holding a dead unit changes nobody, and the
  error names what was wrong.
- A god can also make the change happen without naming anybody, by acting on
  the world, so that belief spreads on its own.
- What the engine reports about how many units each side holds agrees with the
  change, immediately and without the caller asking for a recount.
- A watcher reads which units changed side, on the step it happened, and each
  entry names the place, the side that lost the unit and the side that gained
  it.
- A unit that changed side does not change back on the next step while the
  world stands still.
- The same world gives the same result at every thread count and on every run.
- A developer who names a side that the world does not hold meets an error that
  names the side.

## What this does not do

**It does not give a unit two sides.** A unit belongs to exactly one side. A
game that wants a unit to fight for one side and believe in another models that
above the engine.

**It does not charge a price.** The engine states no cost, no cooldown and no
eligibility rule for taking a person. A game that wants belief to be expensive
charges for it itself.

**It does not decide who deserves to be taken.** The engine holds no notion of
loyalty, of faith or of resistance on a unit. What makes one person easier to
take than another is the game's business.

**It does not move a place.** Taking the people who stand somewhere is not
taking the ground they stand on. The engine already decides who holds a tile,
and this record changes none of that.

**It does not remember where a unit came from.** The unit carries no history.
The record of the change is the report of the step it happened on, and a game
that wants a longer memory keeps one.

**It does not name the people it took.** A god learns which units it gained by
reading the report of the step. It does not receive a list built for it.

## What it costs at the target scale

The engine holds far more tiles and units than a script can visit, and the
scale constants table holds the figures.[^4]

**The deliberate call costs the set the caller named, and nothing else.** It
resolves each identity once and writes each unit once. It does not visit the
population, and it does not visit the world.

**The rule that makes the change happen by itself must not cost the
population multiplied by anything.** The engine already visits every unit once
for each step, to keep the derived position of the population. A rule that
rides on that visit and reads a value from the place the unit stands costs one
read for each unit. A rule that asked each unit about the units near it would
cost the population multiplied by a neighbourhood, and that is the shape the
project rejects.[^5]

**The size of what a watcher reads follows what changed, not what exists.** A
step in which nobody changed side reports nothing.

**No figure is stated here.** One blocker governs every cost figure in this
project, and it says which figures are measured and which are derived.[^6] The
statements above are arguments about which term the cost follows, not results.

## Which blockers govern this

**One blocker governs every cost claim above.**[^6] Nothing here was measured.

**One blocker governs the game this need came from.** The rules of that game
are one paragraph.[^7] The need above is stated in general terms so that it
does not wait on them. What the game charges for belief, and what makes one
person easier to take than another, are game rules and this record deliberately
leaves them out.

**Nothing here waits on a question the project owner holds about the engine.**
The project owner decided the shape of the change directly: taking a person
changes the side of that person outright, and the engine holds no second value
beside the side.

**One thing is open and it does not block this.** Whether a built thing changes
hands when the ground under it does is unanswered.[^8] This need moves people
and never moves ground, so the answer does not change either way.

## References

[^1]: PRD-0011, a unit is born, holds a job, and dies. `docs/product/accepted/prd-0011-a-unit-is-born-holds-a-job-and-dies.md`
[^2]: PRD-0001, a faction sees only what its own units observe. `docs/product/accepted/prd-0001-a-faction-sees-only-what-it-observes.md`
[^3]: Project orientation, the design principles. `CLAUDE.md`
[^4]: Budgets and costs, the scale constants. `docs/reference/budgets.md`
[^5]: ADR Registry, row 0096. `docs/adrs/REGISTRY.md`
[^6]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^7]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^8]: Blockers register, BLK-036. `docs/BLOCKERS.md`
