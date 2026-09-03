---
id: 0206
title: Let the agent tool read what the panel reads
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## This item is closed against the work that superseded it

**Another worker took this work in the same round this item was written, and
that work is on the trunk.** The server carries a tool for every reader this
item names. The outcome below states what closed it and how that was checked.

Two items that name one job are two declaration sites, and nothing fails when
they disagree about whether the job is done.[^1]

## Why

An agent that works on this repository drives the engine through a protocol
server. The server holds a world between calls and reports what the engine
knows. It reports the tick, the extent, the seed, the faction count, the tile
count, the state hash and the event count. It reads the event log, the tile
changes and the gather events. It spawns, despawns, orders a gather and names
the tile of one unit.[^2]

**It has no reader for most of what the viewer shows.** The founding survey,
the level 1 summary of a region, the store and the rate and the ration of a
site, the choice explanation of a unit, the stock and the holder of a tile, the
census of the ground, and the crowding counts all reach the panel and none of
them reaches the tool. The finding holds the count.[^3]

The window now draws cards and the panel moved to a rendered picture.[^4] The
picture carries the detail, so nothing is lost, and a person reads it. An agent
cannot read an image. An agent that wants to know why a unit chose what it
chose must still write a throwaway test.

## What the work might do

**Read the section at the top first.** The rest of this item describes work
that another worker has done, and it is kept as the record of what was missing
and why it mattered.

The engine already exposes every one of these through the public crate
interface. The work is a reader in the binding and a tool in the server, and no
change to the engine.

The questions this item must answer before it is refined:

- Which readers earn a tool. A tool for each is the obvious answer and it is
  probably wrong, because the control plane is not a data plane and a tool that
  returns one row for each tile is a loop.[^5]
- Whether a selector answers this instead. The register names the selector as
  the destination for saying where to act, and a reader that takes a selector
  would replace several tools with one.[^6]
- What the layout cost is. The event log already hands Python bytes with no
  description of the layout, and a decoder in Python would repeat what the Rust
  source declares.[^7]

## Done when

- The agent server reports every reader this item names.
- A test drives a live server over the protocol and lists its tools.

## Outcome

**Closed as done, against work that carried no item number.** The commit that
expanded the agent server added seven tools in one change, and it touched no
backlog file, so this item is the only record of the need it answered.[^8]

**What the server holds now.** Eighteen tools. Every reader this item names has
one: the founding survey, the level 1 summary of a region, the store and the
rate and the ration of a site, the choice explanation of a unit, the stock and
the holder of a tile, and the ground counts and crowding counts of a window.
The ground counts and the crowding counts arrive as one tool and not as two,
because one pass over one window answers both.

**How that was checked.** By running, not by reading. One test lists the tools
of a live server over the protocol, in a subprocess, and compares the sorted
names against a stated list of eighteen. It passes.[^9] The audit that closed
this item ran that test rather than reading the source.

**What did not change.** This item asked for readers in the binding and tools
in the server, and no change to the engine. That held.

**What stays open.** The gap this item was written from can open again, because
nothing compares the tool surface against the engine. Item 0211 holds that, and
it is the reason two items named one job here.


## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: The agent protocol server. `python/cachette/agent/server.py`
[^3]: Findings register, FND-199. `docs/FINDINGS.md`
[^4]: Backlog item 0133. `docs/backlog/complete/0133-let-a-watcher-reach-a-panel-longer-than-the-window.md`
[^5]: Project orientation, the design principles. `CLAUDE.md`
[^6]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^7]: Backlog item 0153. `docs/backlog/refined/0153-let-python-read-an-event-without-repeating-its-layout.md`
[^8]: The commit that expanded the agent server, `8d94c89`. `python/cachette/agent/server.py`
[^9]: The agent protocol test, `test_a_client_reaches_every_tool`. `tests/test_agent_mcp.py`
