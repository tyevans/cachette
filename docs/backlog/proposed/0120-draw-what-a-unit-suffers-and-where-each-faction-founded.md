---
id: 0120
title: Draw what a unit suffers and where each faction founded
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0003]
blocked-by: [0057, 0094]
---

## Why

The engine gains state that nobody can see. A watcher opens the window and
reads terrain, soldiers, tile ownership and a panel of counts. Every rule
added since then is invisible.

Two items now in flight add state of exactly this kind. One gives a unit a
condition that gets worse under a shortage and ends the unit when it lasts too
long. The other founds one group for each faction at a minimum distance. Both
state their result as a value a watcher reads through the public interface.
Neither draws anything.

The product record asks for a world worth looking at, and its index note
already says the viewer trails the engine.[^1] No open item answered that
note. Two items name the record as what they serve, and neither draws: one is
a world-build cost item and the other is a terrain regression test.[^2] [^3]

A finding records the shape.[^4] The word "watcher" covers two interfaces, the
library and the window, and an item can satisfy its whole acceptance list
against the first while adding nothing to the second.

## What the work does

1. The viewer draws the condition of a unit, so a watcher sees a shortage
   spread through a group and sees which units it takes.
2. The panel says how many units the shortage holds and how many it has ended.
3. The viewer marks each founded place, so a watcher sees the factions apart
   from each other and can judge the distance between them.
4. The panel names each faction that founded and each faction that failed to
   found.

## The questions this item must answer before it is refined

**Which interface supplies each value.** The engine holds no value that exists
because something draws it. Every value here must already be readable, or the
item must say why the engine gains a reader rather than the viewer gaining a
pass.

**Whether the condition is a colour, a mark or a panel row.** A condition on
every unit competes with the faction colour a unit already carries. The
viewer holds one faction colour table and must not gain a second.

**What the founding marks cost after the founding frame.** A founded place is
history, not state that changes. The panel reads counts of the pass that just
ran, and a mark that persists is a different thing from a count.

**Whether a picture test can hold either of these.** A stored picture of the
panel cannot read a clock, and the condition changes every tick.

## What this item does not do

It adds no rule to the engine, and it changes no simulated value. It does not
widen into the resource display, which a later item owns.[^5] It writes no
decision record; the boundary between the engine and the viewer is already
recorded, and this item obeys that record rather than restating it.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Product priority index. `docs/product/PRIORITY.md`
[^2]: Backlog item 0112. `docs/backlog/proposed/0112-build-a-world-without-a-pass-over-every-tile.md`
[^3]: Backlog item 0034. `docs/backlog/proposed/0034-measure-the-generated-terrain-against-a-stored-one.md`
[^4]: Findings register, FND-100. `docs/FINDINGS.md`
[^5]: Backlog item 0106. `docs/backlog/proposed/0106-show-a-watcher-what-is-moving-and-where-it-goes.md`
