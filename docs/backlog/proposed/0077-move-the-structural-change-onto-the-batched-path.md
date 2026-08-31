---
id: 0077
title: Move the structural change onto the batched path
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: []
blocked-by: [ADR-0020]
---

## Why

The storage record states that a structural change is a move between column
sets, and that the batched tombstone and compact path applies to it.[^1] No
such path exists. The record that would define it holds a reserved registry
row and no file.[^2]

Two arenas now edit their columns inside the call. A soldier spawn, a soldier
despawn, a settlement founding and a settlement loss all take effect at once,
and not at the frame barrier. Both arenas agree with each other and neither
agrees with the record.

This item moves both onto the batched path. It cannot start until the record
that defines that path exists, so it names the record as its blocker.

The identity rule already carries the property that matters most. The
generation advances when the arena frees the slot, so a destroyed entity loses
its identity at the moment it dies, whatever path the change took.[^3] This
item is therefore about the barrier, not about identity.

## Notes

The finding register records why the gap opened and what the settlement work
did about it.[^4]

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decision D2. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: ADR Registry, row 0020. `docs/adrs/REGISTRY.md`
[^3]: ADR-0014, entity identity is an index plus a generation, decision D3. `docs/adrs/accepted/adr-0014-entity-identity-is-an-index-plus-a-generation.md`
[^4]: Findings register, FND-063. `docs/FINDINGS.md`
