---
id: 0490
title: Let the control plane read the store of a seeded settlement
status: proposed
created: 2026-09-05
implements: []
changes: []
creates: []
serves: [0047]
blocked-by: []
---

## Why

The site economy reader takes the whole identity of a settlement. It refuses a
slot index, and it is right to refuse one.

The seeding verb founds every faction in one call and gives back one report for
each faction. That report holds the place, the people and the refusal. It holds
no settlement identity. The demonstration builds its world with that verb and
with no other, so nothing in the control plane can name a settlement of that
world.

A watcher outside the window therefore cannot read a store, a production rate
or an upkeep rate of any settlement the seeding made. FND-485 holds the
reading.

PRD-0047 asks that a developer set what a settlement holds and then read the
value back. A developer cannot do either one for a seeded world today.

The likely repair is one key. The founding report of the seeding verb carries
the identity, in the way the founding group report already does. Refining this
item should say whether the two reports become one shape.
