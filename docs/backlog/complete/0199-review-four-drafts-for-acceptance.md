---
id: 0199
title: Review four drafts for acceptance
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

Fifteen records sit in `docs/adrs/draft/`, and the queue behind them is the
bottleneck. A draft binds nothing, so work built on one is built on sand. Four
of them were reviewed under this item: the influence solve, the tile field, the
tile upgrade and the residence.

The registry sets the conditions. A record is read against the code and not
against the intent. A reviewer may not review what it wrote. A review that
lists no attempted objection did not happen.[^1]

## What the work does

1. Read each record against the tree, decision by decision.
2. Write one review file, with every objection attempted and the verdict for
   each record.[^2]
3. Set the registry status of each record, or leave it and say what must change
   first.
4. Record what the review found beyond the records.

## Impact review

**Governed by.** The registry, for who may review and what a delegated review
must do.[^1] The reviews guide, for what a review must contain.[^2] The record
scope rule, for what belongs in a record and what does not.[^3]

**Changes.** No record changes. A reviewer that rewrites a record into
acceptability has authored it.

**Creates.** No record.

**Blockers.** None.

## Done when

- Each of the four records is read against the code, decision by decision.
- One review file holds every objection attempted and a verdict for each
  record.
- The registry status of each record is set or explicitly left, with the reason.
- The registers hold anything the review found beyond the records.
- The document checks pass.

## Outcome

**No record was accepted, and each was returned for a stated reason.**

ADR-0087 holds against the code in every decision, and the reviewer failed to
break any of them. It is returned because it cannot bind while DEC-067 is open:
accepting it would put two accepted records in conflict, and the record itself
declines to claim that the level record permits it. Nothing in it needs to
change.

ADR-0088 holds in substance. One of the three checkable properties of its
decision D1 says building a world visits no tile of the field, and the register
already records that a world is still not built without a pass over every
tile.[^4] The review holds the replacement text.

ADR-0090 holds in substance. One sentence of its decision D3 claims that every
caller which asks how many units a tile holds calls the composition, and three
callers read the ground alone. The review holds the replacement text.

ADR-0081 is returned for the reason a previous review already gave, which was
checked again and still holds. Two things were added: its second frequent
caller does not exist either, and its decision D1 has no code at all, which the
priority index did not say.

**Registers.** FND-193 records that the fold reporting the largest capacity
walks one of the two capacity tables the engine states. DEC-081 holds the
choice that follows, and item 0200 holds the work behind it.

**The priority index gained what it was missing.** Two rows now say that the
record they name runs ahead of its code, which is the index's own rule.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: Reviews guide. `docs/reviews/README.md`
[^3]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^4]: Findings register, FND-162. `docs/FINDINGS.md`
