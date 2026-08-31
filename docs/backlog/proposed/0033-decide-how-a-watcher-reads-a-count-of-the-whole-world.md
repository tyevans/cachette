---
id: 0033
title: Decide how a watcher reads a count of the whole world
status: proposed
created: 2026-08-30
implements: []
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

The head-up display states how many soldiers of each faction stand in the
window. It does not state how many each faction has in the whole world.

The omission is deliberate and it is recorded. Nothing knows the world total
without reading every soldier, the world holds one million soldiers at the
target scale, and a per-frame pass over them would cost more than the picture
it labels. ADR-0070 D2 forbids showing an estimate instead, so the panel shows
nothing.

A developer will want the number. Deciding how they get it is real work, and
it is not the panel's work.

## What is missing before this is refined

This item names a question, not an answer. Refining it needs the impact
review, and the review needs one of these to be chosen first.

- **The engine already derives it.** The unit-to-tile bridge rebuilds at the
  barrier and already reads every soldier while it does. A census taken during
  that pass would cost almost nothing. It would also be a value the engine
  holds, which ADR-0067 D2 permits only if the engine wants it for its own
  reasons. Deciding that is a decision record, and the number is reserved
  before the item is refined.
- **The control plane answers it.** Python is the place a person asks a
  question about numbers. A count of every faction is a question about
  numbers. This costs the watcher a second window and answers PRD-0005 badly,
  because a person watching a picture should not have to leave it.
- **The panel asks on demand.** A count taken when a person presses a key
  costs one pass, once, rather than one pass each frame. This needs no engine
  field and no record beyond ADR-0070, which bounds the panel's per-frame
  cost and says nothing about a key press.

The third option looks cheapest and smallest. It is not chosen here, because
choosing it is the impact review this item has not had.

## Done when

Not yet stated. The item is not refined.

## Outcome

Filled in when the item moves to `complete/`.
