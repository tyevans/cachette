---
id: 0211
title: Check the agent tool surface against the engine
status: proposed
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The agent protocol server grows one tool at a time, against a need somebody
stated.[^1] That rule is right, and it has one failure mode that the record
names and does not fix: the surface can go stale against the engine, and
nothing fails when it does.

The engine gains a reader. Nobody states a need for it, so no tool reports it.
An agent then meets a wall, decides the engine does not hold the value, and
either works around it or writes a throwaway test. The engine held the value
the whole time.

This already happened once, in the other direction. The engine held every
quantity a viewer overhaul needed. The server reported none of them. The worker
who needed them found the gap by reading the Rust source. Nothing told anybody
the gap existed.

The growth rule says do not build ahead of a need. It does not say never look.

## What the work might do

Something that lists the readers of the bindings that no tool reaches, and
reports them without failing. A report, not a gate: a reader with no tool is a
question nobody has asked yet. That is the intended state, so failing on it
would invert the rule.

The report belongs where a worker sees it before starting, not in the gate
output at the end.

Whether the comparison reads the bindings or the server is open. Both are
declared in one place each today.

## What is not yet worked out

- Which side is the source: the bindings, the type stubs, or the core.
- Whether a reader that deliberately has no tool needs a way to say so, and
  where that statement would live.
- Whether this belongs with the record checks or with the tools an agent runs.

## Done when

- A worker can see which engine readers no tool reaches, without reading the
  Rust source.
- The report does not fail a build.

## References

[^1]: ADR-0092, the agent tool surface grows one tool at a time, against a stated need, decision D1. `docs/adrs/draft/adr-0092-the-agent-tool-surface-grows-against-a-stated-need.md`
