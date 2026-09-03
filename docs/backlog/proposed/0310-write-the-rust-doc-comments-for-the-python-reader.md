---
id: 0310
title: Write the Rust doc comments for the Python reader
status: proposed
created: 2026-09-03
implements: [ADR-0107 D2]
changes: []
creates: []
serves: [PRD-0021]
blocked-by: []
---

## Why

**The published reference carries only the prose that the Rust doc comments
hold, and a record says plainly that it makes nothing write that prose.** The
record fixes where the prose lives. A member with no Rust doc comment publishes
with no prose, and nothing fails.[^1]

**The audience of a doc comment is now two audiences.** The record names them:
the Rust reader who changes the core, and the Python developer who never opens
it.[^1] [^2] A comment written for the first alone under-serves the second, and
the record states that no check can see that.

**The product record states three answers the reader must get without opening a
Rust file.** What a returned column holds. In what unit a value is expressed.
Which error a call raises, and what the error means.[^2] Every error the engine
raises is a declared type, and the reader must be able to name it.[^3]

This item reads the generated page and repairs the crate behind it.

## What the work does

Read the published reference. For each public member of the compiled module,
answer three questions: does prose exist, does it state the return and its unit,
and does it name the errors the call raises. Repair the Rust doc comment of each
member that fails one.

The work happens in the bindings crate. Nothing is written into a documentation
page and nothing is written into the type stub, because the record forbids
both.[^1]

## What it does not do

It does not document the Rust core. A person who changes the engine is a
different audience, and the product record excludes them.[^2]

It does not state a cost figure. One blocker governs every figure in this
project, and a writer who wants to make a performance claim cites it rather than
inventing a number.[^4]

It does not answer whether an upgrade changes hands when the ground does. That
question is open, and a writer who reaches the behaviour says so and cites the
blocker.[^5]

## Why this is not refined

The finish line depends on what the generated page shows, and no page exists
yet. Item 0309 publishes it.[^6] Refining this item means reading that page and
turning the gap into a list.

## References

[^1]: ADR-0107, the Python reference is generated from the compiled module, decision D2 and the consequences. `docs/adrs/draft/adr-0107-the-python-reference-is-generated-from-the-compiled-module.md`
[^2]: Product requirement record 0021, a developer can use the control plane without reading its source. `docs/product/accepted/prd-0021-a-developer-can-use-the-control-plane-without-reading-its-source.md`
[^3]: ADR-0046, every error is typed, decision D3. `docs/adrs/draft/adr-0046-every-error-is-typed.md`
[^4]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^5]: Blockers register, BLK-034. `docs/BLOCKERS.md`
[^6]: Backlog item 0309, publish the Python reference generated from the compiled module. `docs/backlog/refined/0309-publish-the-python-reference-generated-from-the-compiled-module.md`
