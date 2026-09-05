---
id: 0049
title: A god declares war and makes peace
status: Shaped
created: 2026-09-05
---

# PRD-0049 — A god declares war and makes peace

## Who this is for

A developer who builds a game in which a god directs a congregation. The god
is a person or a language model, and it acts through the control plane.

A modeller needs this second. A modeller wants two populations that can be
at war or at peace. A study can then compare what each state does to them.

## What the person cannot do today

A god cannot choose who its enemy is.

Two factions fight wherever their units meet. Two factions trade whenever
both agree. Neither act asks how the two factions stand toward each other,
because the engine holds no standing between them. Every faction is at war
with every faction. Every faction is also at peace with every faction.

This has three costs.

A god cannot spare anybody. A congregation that meets a friendly congregation
fights it, because the engine knows no friend.

A god cannot make an enemy. A god that wants a war has no act that starts
one. Its units fight only where they happen to stand.

A watcher cannot read the politics. Two factions that fight and two factions
that trade look the same from above. Nothing states which pair is which.

## What good looks like

Each statement below can be checked.

- The engine holds a standing between every ordered pair of factions. How A
  stands toward B and how B stands toward A are two values.
- The standing is graded. It moves by steps, and a step is smaller than the
  gap between peace and war.
- A god moves its own standing toward another god in one call. The engine
  refuses the call when the god has nobody who can speak for it.
- The standing gates contest. Two factions fight only when at least one of the
  pair is at war with the other. Two factions at peace pass each other.
- The standing gates trade. The engine refuses a deal between two factions
  when either is at war with the other.
- The world moves the standing. A deal kept moves it up. A deal failed, a unit
  lost, a unit converted and a storm inflicted each move it down.
- The standing drifts. Two factions that leave each other alone move toward
  peace on their own.
- A crossing into war and a crossing out of it each appear once as an event.
  A watcher reads the event. It names both factions and the direction.
- A treaty is a deal. A god may offer a step in standing as its side of a
  deal. The step applies when the other side delivers.
- The same seed gives the same standings, at every thread count, on every run.

## What this does not do

- It does not give a god a message channel. Speech between gods is a separate
  need.
- It does not name the bands. Where peace ends and war begins is a value of
  the downstream game, and nobody has stated it.
- It does not let a god move the standing of another god toward it. A god
  states only its own feeling.
- It does not decide how the standing is stored. That is an architectural
  question, and it belongs in a decision record.
- It does not make a faction honour a treaty. A faction that makes peace and
  attacks the next tick is permitted. Reputation is a separate need.
- It does not give a god a standing toward itself.

## What it costs at the target scale

The cost driver is the number of factions, not the number of units and not the
number of tiles.

A standing exists for each ordered pair of factions, and the number of
factions has a ceiling. Nothing about the standing grows with the population.

Three properties follow. A solution must have all three.

- The storage grows with the square of the faction ceiling and with nothing
  else.
- A read of one standing is a lookup, not a scan. A pass that gates a meeting
  on a standing pays one lookup for the meeting.
- The drift costs the faction pairs on its schedule and costs nothing between.

No cost figure appears here. One blocker governs every cost figure this
project holds, and it says which figures are measured and which are
derived.[^1]

## Which blockers govern this

- **One blocker governs every cost figure here.**[^1] Every cost statement
  above states a shape and not a number.
- **One blocker holds the rules of the downstream game.**[^2] Where the bands
  lie and how large a step is are rules of that game. So are the drift rate
  and the standing that permits a conversion. The engine holds a value for
  each, and the blocker says that no owner chose them. This record states none
  of them.

This record depends on two factions meeting at a tile, and on two players
dealing with each other. Both exist.[^3] [^4]

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-050. `docs/BLOCKERS.md`
[^3]: PRD-0035, a god takes the people of another god. `docs/product/shaped/prd-0035-a-god-takes-the-people-of-another-god.md`
[^4]: PRD-0034, two players hold each other to a future delivery. `docs/product/shaped/prd-0034-two-players-hold-each-other-to-a-future-delivery.md`
