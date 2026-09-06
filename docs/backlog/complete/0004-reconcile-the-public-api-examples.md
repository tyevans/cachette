---
id: 0004
title: Reconcile the public description's API examples with the selector record
status: complete
created: 2026-08-30
---

The public description shows a selector call, a content definition call, and a
region query. No record has settled that surface yet. When the selector record
is written, check every example against it and correct the description.

This cannot be refined until that record exists, so it stays here.

## Outcome

**Closed as already done. The work landed under other items.** An audit read the
tree on 5 September 2026.

**The selector record exists and is accepted.** ADR-0051 states that a selector
is a lazy expression tree, and the public description cites it.[^1]

**The examples now match the compiled interface.** The description holds one
Python example, and every name in it resolves in the type stubs.[^2] The three
examples this item named are gone. An earlier commit corrected the description
and the contribution guide against the code, and this item was never moved.

## References

[^1]: ADR-0051, a selector is a lazy expression tree. `docs/adrs/accepted/adr-0051-a-selector-is-a-lazy-expression-tree.md`
[^2]: The type stubs of the control plane package. `python/cachette/_core.pyi`
