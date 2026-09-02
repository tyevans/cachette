---
id: 0206
title: Let the agent tool read what the panel reads
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

An agent that works on this repository drives the engine through a protocol
server. The server holds a world between calls and reports what the engine
knows. It reports the tick, the extent, the seed, the faction count, the tile
count, the state hash and the event count. It reads the event log, the tile
changes and the gather events. It spawns, despawns, orders a gather and names
the tile of one unit.[^1]

**It has no reader for most of what the viewer shows.** The founding survey,
the level 1 summary of a region, the store and the rate and the ration of a
site, the choice explanation of a unit, the stock and the holder of a tile, the
census of the ground, and the crowding counts all reach the panel and none of
them reaches the tool. The finding holds the count.[^2]

The window now draws cards and the panel moved to a rendered picture.[^3] The
picture carries the detail, so nothing is lost, and a person reads it. An agent
cannot read an image. An agent that wants to know why a unit chose what it
chose must still write a throwaway test.

## What the work might do

The engine already exposes every one of these through the public crate
interface. The work is a reader in the binding and a tool in the server, and no
change to the engine.

The questions this item must answer before it is refined:

- Which readers earn a tool. A tool for each is the obvious answer and it is
  probably wrong, because the control plane is not a data plane and a tool that
  returns one row for each tile is a loop.[^4]
- Whether a selector answers this instead. The register names the selector as
  the destination for saying where to act, and a reader that takes a selector
  would replace several tools with one.[^5]
- What the layout cost is. The event log already hands Python bytes with no
  description of the layout, and a decoder in Python would repeat what the Rust
  source declares.[^6]

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The agent protocol server. `python/cachette/agent/server.py`
[^2]: Findings register, FND-199. `docs/FINDINGS.md`
[^3]: Backlog item 0133. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
[^4]: Project orientation, the design principles. `CLAUDE.md`
[^5]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^6]: Backlog item 0153. `docs/backlog/refined/0153-let-python-read-an-event-without-repeating-its-layout.md`
