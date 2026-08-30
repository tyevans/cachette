---
id: 0023
title: Implement sort-then-admit movement
status: proposed
created: 2026-08-30
---

A move is an intent. A separate admission step grants it, sorts by a stable
key, admits in that order, and respects the tile capacity that BLK-009 fixed
at eight.

This is where determinism is easiest to lose and hardest to see, so the
thread-count test must cover a tile that is oversubscribed.

Refine this at sprint 3 planning, after item 0021 settles ADR-0056.
