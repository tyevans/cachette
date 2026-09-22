---
id: 0058
title: Two factions must meet before they can interact
status: Accepted
created: 2026-09-21
---

# PRD-0058 — Two factions must meet before they can interact

## Who this is for

A game developer who builds a game with fog of war and exploration mechanics.

An AI researcher studying diplomatic interactions and multi-agent coordination
under incomplete information.

## What the person cannot do today

A player cannot explore before interacting.

Previously, a player or an autonomous controller could declare war, change
diplomatic relations, or negotiate trade with any other faction from tick zero,
even if neither faction had ever observed a unit or settlement of the other.
Diplomacy operated with global omniscience, ignoring the fog of war.

This had three costs:

1. A faction could declare war on an unknown adversary across the world without
   any reconnaissance.
2. A controller would select distant, unseen factions as rivals or prey,
   distorting territorial expansion and strategic focus.
3. Fog of war was broken at the diplomatic level: exploration gave no tactical
   information advantage for opening diplomatic channels.

## What good looks like

Each statement below can be checked.

- Two factions start with no contact with each other.
- The engine refuses an attempt to move relations or declare war between two
  factions that have never met.
- Two factions meet symmetrically when a live unit of one faction has line of
  sight to a live unit of the other faction.
- Two factions meet symmetrically when a live unit of one faction has line of
  sight to a settlement of the other faction.
- An autonomous faction controller does not pick an unmet faction as a rival
  or prey, and does not declare war on an unmet faction.
- Meeting records the contact permanently for the life of the simulation.
- Once met, factions can interact, negotiate trade, and move relations.
- Every run with identical seeds produces identical meeting events and
  diplomatic states across all thread counts.

## What this does not do

- It does not lift the fog of war across the map upon meeting. Factions still
  only observe what their live units see each tick.
- It does not change the geometric calculation of line of sight or shadowcasting.
- It does not mandate hostile or friendly standing upon first contact; initial
  standing remains neutral.
- It does not decide the internal memory representation; that belongs in a
  decision record.

## What it costs at the target scale

The contact state depends on the number of factions, not the number of units
or tiles.

With a ceiling of 16 factions, contact between all pairs fits in a compact
symmetric matrix. The contact evaluation reuses the unit visibility masks and
settlement positions already computed during the observation rebuild pass.

Reading whether two factions have met costs a constant-time indexed lookup.

## Which blockers govern this

- Target platform costs. One blocker governs every cost figure, and states
  which are measured and which are derived.[^1]

## References

[^1]: Target platform costs. `docs/BLOCKERS.md`
