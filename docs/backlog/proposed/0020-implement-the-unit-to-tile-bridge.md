---
id: 0020
title: Implement the unit-to-tile bridge
status: proposed
created: 2026-08-30
---

A tile must answer which units stand on it, and a unit must answer which tile
it stands on. Registry row 0018 holds the claim: the bridge is three
structures, and units stay sorted by tile.

The key vector sort of item 0014 is what keeps them sorted. Refine this at
sprint 2 planning, after item 0007 writes row 0018.
