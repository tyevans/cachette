---
id: 0161
title: Let the control plane found a group
status: proposed
created: 2026-09-01
implements: []
changes: []
creates: []
serves: [PRD-0012]
blocked-by: []
---

## Why

A real run seats a group through the founding. The founding surveys the ground,
chooses a place, seats a settlement, spreads the group over its disc, and sets
what the site produces.[^1] It is one command for a set, which is the shape the
design principle asks of the control plane.[^2]

The control plane cannot call it. Python puts a unit in the world one unit at a
time, through the per-unit pair the event column work added.[^3] That pair is
lower than the engine's own path, and a caller that builds a population with it
loops over entities.

The decisions register holds the question of which verb the project promises,
and its recommendation is that the founding run becomes the verb a caller
reaches for.[^4]

## What the work does

Expose the founding run to Python. The call takes a group size and returns what
the engine founded: the place, the settlement identity and the identities of
the people. Every identity crosses whole, and the engine resolves one that
comes back.[^5]

## What it must not do

It must not return a slot index for any of those identities.

It must not withdraw the per-unit pair on its own. A founding never frees a slot
that a later founding reuses, so the pair is still the only way to reach the
case that the identity rule exists to catch. Withdrawing it is a separate
decision, and the register holds it.[^4]

## Done when

Filled in when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: The founding run. `crates/cachette-core/src/world.rs`
[^2]: Project orientation, the design principles. `CLAUDE.md`
[^3]: Backlog item 0153. `docs/backlog/refined/0153-let-python-read-an-event-without-repeating-its-layout.md`
[^4]: Decisions register, DEC-063. `docs/DECISIONS.md`
[^5]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves. `docs/adrs/draft/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
