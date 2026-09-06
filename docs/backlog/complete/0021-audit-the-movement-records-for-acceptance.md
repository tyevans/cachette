---
id: 0021
title: Audit the movement records for acceptance
status: complete
created: 2026-08-30
---

Both records are drafts. ADR-0056 depends on row 0018, which item 0007 writes
and item 0020 implements.

Audit each against the scope rule and against the code that now exists.
Accept, amend, or say plainly why it stays a draft.

The long-path record is settled and needs no audit. It was retired: it
specified a portal graph and a flow tile cache before any path-finding
existed and before a product record asked for a long path. Its number is in
the retired table with the reason.

Refine this at sprint 3 planning.

## Outcome

**Closed as already done. Both movement records are disposed of.** An audit read
the registry on 5 September 2026.

**ADR-0056 is accepted.** It states that movement is tile-discrete and admitted
by a sort and then an admit, and its file sits in the accepted directory.[^1]

**The second movement record is retired, with its reason recorded.** The
registry gives it a retired row, and the reasoning sits beside it: it described
a subsystem nobody had built, for a need nobody had stated.[^2]

**The dependency this item named is clear.** ADR-0018 is accepted, with a file.

## References

[^1]: ADR-0056, movement is tile-discrete and admitted by sort then admit. `docs/adrs/accepted/adr-0056-movement-is-tile-discrete-and-admitted-by-sort-then-admit.md`
[^2]: ADR Registry, the rows for 0056, 0057 and 0018. `docs/adrs/REGISTRY.md`
