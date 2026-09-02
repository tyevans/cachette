---
id: 0153
title: Let Python read an event without repeating its layout
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The bindings hand Python the event log as raw bytes. The layout of an event
lives in the Rust source, which declares the field order, the field widths and
the padding.[^1] Python holds no description of it.

A reader in Python must therefore repeat the layout. That is a second
declaration site for one fact, and nothing fails when the two copies
disagree.[^2] The recurring defect rule names this exact pair as a place the
shape will recur in this project.

The consequence is visible now. The agent-facing protocol server returns the
bytes and a digest of them, because it refuses to decode them.[^3] An agent can
prove that two runs emitted the same log. It cannot see which tile changed, who
owned it, or by how much.

## What the work does

Give Python the events, from the one place that declares them. The decisions
register holds the three options and the recommendation, so this item does not
repeat them.[^4]

## What it must not do

It must not add a `struct` format string to Python. That is the defect.

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The event types. `crates/cachette-core/src/event.rs`
[^2]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: Backlog item 0152. `docs/backlog/proposed/0152-let-an-agent-drive-the-engine-through-a-protocol-server.md`
[^4]: Decisions register, DEC-060. `docs/DECISIONS.md`
