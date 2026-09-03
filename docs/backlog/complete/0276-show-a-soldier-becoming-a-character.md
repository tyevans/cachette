---
id: 0276
title: Show a soldier becoming a character
status: complete
created: 2026-09-03
implements: [ADR-0070]
changes: []
creates: []
serves: [PRD-0005]
blocked-by: []
---

## Why

**The promotion pass produces a moment, and a count throws the moment away.**
A unit is promoted for what it did. A watcher who sees only a running total
learns that the number went up, not that a soldier they were watching earned
it. The engine writes a log of the promotions of each frame, so the moment is
available and nothing was showing it.

**A count alone would also be nearly invisible.** In the demonstration world
only a small share of frames promote anybody, so a surface that reported the
total would sit unchanged almost all of the time and a watcher would stop
reading it.

## Done when

- The window and the panel say that somebody became a character, which faction
  they were, and the deeds that earned it.
- The moment stays readable for longer than the frame it happened in.
- The surface appears only once somebody has been promoted.
- The viewer keeps no memory of its own to do it.

## How it came out

**The moment is held without the viewer remembering anything.** The log holds
one frame, so a card driven by the log alone would show a promotion for a
fraction of a second. A character stores its birth tick, so the viewer finds
the newest character and says how long ago it was. The engine already holds
the fact; the viewer does not keep a second copy of it, which is what the
boundary record requires.[^1]

**The walk is the one the records permit.** Counting the characters and
finding the newest is a walk of the character tier. That tier holds a bounded
population and a caller may walk it; the mass tier does not and a caller may
not.[^2] Both answers come from one pass, because reading them separately
would walk the tier twice for one picture.

**The control plane reacts rather than polls.** The Python demonstration reads
the count the frame gave it and prints a line when it is not zero. It walks no
entity and asks the engine nothing further.

**Measured before any of it was drawn.** The demonstration world holds 34
characters at tick 200 and 50 at tick 400, the first promotion lands at tick
50, and 22 frames of 400 promote anybody. That last figure is why the moment
is held rather than shown for one frame: a card driven by the log alone would
have been blank on 94 percent of frames.

## References

[^1]: ADR-0067, the viewer reads the world and never writes to it, decision D2. `docs/adrs/accepted/adr-0067-the-viewer-reads-the-world-and-never-writes-to-it.md`
[^2]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
