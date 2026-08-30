---
id: 0001
title: A faction sees only what its own units observe
status: Shaped
created: 2026-08-30
---

# PRD-0001 — A faction sees only what its own units observe

## Who this is for

A developer who builds a strategy game on this engine.

The other two audiences do not need this. A modeller who studies an economy
usually wants full information. A researcher who reproduces a run wants the
world state, not one side's view of it. This record serves one audience on
purpose.

## What the person cannot do today

A game developer cannot hide part of the world from a player.

Every faction reads the same world. A player therefore knows where every
enemy stands, what every province holds, and what happened in a place no
unit of theirs has ever visited. The developer has no way to express that a
faction has not learned something yet.

Hidden information is what makes a strategic choice a choice. Without it,
scouting has no purpose, an ambush cannot exist, and a player optimises
against complete knowledge rather than deciding under doubt.

## What good looks like

Each statement below can be checked.

- A faction sees a tile when one of its own units observes that tile.
- A faction that has never observed a tile reads nothing about it.
- A faction that observed a tile and moved away reads what it last saw, and
  the engine marks that reading as remembered rather than current.
- Two factions that share vision see the union of what they each observe.
- A query answered for a faction never reveals a value that faction has not
  observed. This holds for a level 1 and a level 2 answer as well, so a
  summary must not leak what the tiles beneath it would hide.
- The result is identical at any thread count, like every other part of the
  simulation.

## What this does not do

- It does not model partial or uncertain sight. A tile is observed or it is
  not. There is no probability of detection.
- It does not hide a faction's own units from itself.
- It does not model deception, false reports, or planted information.
- It does not decide what a unit can observe. The shape of a sight radius
  is a separate need.
- It does not apply to the event log. The log records the truth. What a
  faction may read is a separate question from what happened.

## What it costs at the target scale

The cost driver is the faction count, not the tile count.

A naive design stores one bit for each tile for each faction. At the target
tile count that is a fixed cost for every faction that exists, paid whether
or not that faction can see anything, and it grows without limit as
factions are added.

That is the wrong shape. This need is only affordable if the storage grows
with **what is actually observed**, and not with the product of the faction
count and the tile count. A design that pays for unseen tiles fails this
record.

Two properties follow, and a solution must have both:

- Storage grows with observed area, not with the world area.
- Adding a faction that observes little costs little.

No cost figure appears here, because none has been measured on the target
platform.[^1] The shape of the growth is the requirement. The figure is not.

## Which blockers govern this

- **The tile scale is unresolved.**[^2] The area a single unit observes
  depends on what one tile represents. This record therefore states the
  sight rule in tiles and does not state a radius.
- **The maximum faction count is unresolved.**[^3] The cost argument above
  is written in terms of the faction count for that reason. A design that
  is correct only below a specific faction count must say so.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: Blockers register, BLK-001. `docs/BLOCKERS.md`
[^3]: Blockers register, BLK-005 and the open faction question. `docs/BLOCKERS.md`
