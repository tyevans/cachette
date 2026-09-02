---
id: 0152
title: Let an agent drive the engine through a protocol server
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

An agent that works on this repository cannot run the engine. It reads the
Rust source and guesses what a world does, or it writes a throwaway test and
deletes it. Neither gives the agent a state hash, an invariant result, or an
event count from a world that actually ran.

The control plane already exposes what an agent needs. The compiled module
builds a world, steps it at a stated thread count, reports the tick, the tile
count, the event count and the state hash, and runs the invariant check.
Nothing reaches those from outside a Python session.

The gap is the transport. An agent speaks the Model Context Protocol. The
engine speaks Python. One small server closes the gap and needs no change to
the engine.

## What the work does

A Python package holds a server that speaks the protocol over standard input
and output. It holds each world between calls, so an agent builds a world once
and then steps and inspects it.

The server adds no simulation logic. Every tool calls one method of the
compiled module. Python is the control plane and not the data plane, so no
tool loops over entities.[^1]

## What is done

The first slice is on the branch. It holds five tools: build a world, step it,
report it, check its invariants, and read the raw event log. A protocol client
starts the server as a subprocess, lists the tools, and calls each one. The
tests drive that client, not the server object, because a test that builds the
mechanism proves the mechanism and not the reach.[^2]

The determinism the engine promises survives the layer. The same settings and
tick count give the same state hash through the server, and the hash does not
change with the thread count.[^3]

## What is still open

**The event log is opaque.** The engine hands Python a byte buffer and no
description of its layout. The server returns the bytes and a digest of them.
An agent cannot see which tile changed. A decoder in Python would repeat a
layout the Rust source already declares, and nothing would fail when the two
disagreed.[^4] A separate item holds that work.[^5]

**A world lives until the process ends.** No tool frees one. An agent that
builds a large world holds it for the life of the server. Add a tool when an
agent needs it, and not before.

**The protocol library is a development dependency.** An installed wheel
carries the server module and not the library that runs it. Decide whether the
server belongs in the package at all, or in a separate contributor tool.

## Done when

- An agent can build a world, step it, and read its state hash without
  writing code.
- A test starts the server as a subprocess and speaks the protocol to it.
- A test proves the state hash does not change with the thread count, and that
  test has a proven failure mode.
- The Python gates pass.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Project orientation, the design principles. `CLAUDE.md`
[^2]: Testing Rules, section 5. `.claude/rules/testing.md`
[^3]: ADR-0001, one binary gives one answer at any thread count, decision D4. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^5]: Backlog item 0153. `docs/backlog/proposed/0153-let-python-read-an-event-without-repeating-its-layout.md`
