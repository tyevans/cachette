---
id: 0346
title: Read the units of a faction as a set
status: complete
created: 2026-09-03
implements: [ADR-0004 D1, ADR-0044, ADR-0085 D3]
changes: []
creates: []
serves: [PRD-0030]
blocked-by: []
---

## Why

**Nothing lists the units of a faction.** The control plane reads a population
count for each faction and no identities. So a caller must keep every identity
that a spawn ever returned, in a Python list, for the life of the run.

A developer who cannot list their own people cannot use any verb on a described
set. Every write verb at this boundary already takes a set, so the gap is that
nothing produces one.

This is the read side of the selector. A research report ranks it and says the
read side must be built with the selector rather than after it, because a
selector that only feeds verbs leaves the loop exactly where it is.[^1]

## Impact review

**Governed by.** Three records govern this work.

ADR-0085 D3 states that the engine resolves an identity that Python hands back
and refuses a stale one.[^2] **This read hands back no identity.** It takes a
faction number, and the engine builds the set at the moment of the call. So
every entry names a live soldier, nothing can be stale, and the result needs no
validity mask and no sentinel. The rule that a read taking an identity must
refuse a dead one is untouched, and the singular read still does it.

ADR-0044 states that what copies and what does not is declared at the call
site.[^3] The read answers for one faction and not for the world, so the caller
narrows before the engine copies.

ADR-0004 D1 states that iteration order is explicit.[^4] The walk is in slot
order, which is the same on every run and at every thread count.

**Does the work contradict a record?** No. The prose of the singular read says
that the read stays singular while the write verbs take a set, and it gives the
reason: a set form that took identities would have to choose between failing the
whole call for one dead identity and returning a value that stands for nothing.
That reason is about a read that takes identities. It does not reach a read that
takes a faction and produces identities, and the singular read keeps its prose
and its behaviour.

**Creates.** No record. The scope rule gives three conditions and the third
fails.[^5] The reasoning is visible in the artefact: the read takes no identity,
so the mask question that would need a record never arises. The records that
already exist decide the rest.

**Changes.** No record changes.

**Blockers.** None.

**Precedent.** FND-315 is not this shape. The precedent that matters is the
research measurement: a loop over the singular read cost about four microseconds
for each call, and this repository's own Python takes that route.[^6]

**Product record.** PRD-0030.

## What fails if somebody changes it back

Two defects.

1. Return one Python object for each unit rather than two NumPy arrays. The
   element type assertion then fails, and the loop the read exists to remove
   comes back.
2. Let a dead identity into the set. The set then holds a unit the singular read
   refuses, and the two answers disagree.

The agreement test is the one that matters. It reads every unit of a fixture
through the singular read and compares it against the column, so a column that
said something else would fail.

## Done when

- One call returns the units of a faction as columns.
- The element type of each column is stated in the doc comment.
- A test proves the columns agree with the singular read for every unit of a
  fixture.
- The whole check command runs green.

## Outcome

Built. One call answers with two columns: the identity of each live soldier of
the faction as `numpy.uint64`, and the tile it stands on as `numpy.uint32`.

The read pairs with the send verb of item 0342. A caller sends a set somewhere
and then reads where the set is, and neither call loops.[^7]

The two defects above were put back. Both were caught. The review holds the
output.[^8]

## References

[^1]: Research report 20, what the Python interface should be, section 7.3. `docs/research/reports/20-the-python-interface.md`
[^2]: ADR-0085, an entity crosses to Python as one opaque identity that the engine resolves, decision D3. `docs/adrs/accepted/adr-0085-an-entity-crosses-to-python-as-one-opaque-identity.md`
[^3]: ADR-0044, what copies and what does not is declared at the call site. `docs/adrs/REGISTRY.md`
[^4]: ADR-0004, iteration order is explicit, decision D1. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^5]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^6]: Research report 20, what the Python interface should be, section 2.3. `docs/research/reports/20-the-python-interface.md`
[^7]: Backlog item 0342, let the control plane name the seed set of a strategy field. `docs/backlog/complete/0342-let-the-control-plane-name-the-seed-set-of-a-strategy-field.md`
[^8]: Review of ordered movement and the set read. `docs/reviews/0342-ordered-movement-and-the-set-read.md`
