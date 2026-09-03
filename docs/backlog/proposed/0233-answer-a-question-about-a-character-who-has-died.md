---
id: 0233
title: Answer a question about a character who has died
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The record of descent outlives every character in it, and no reader
delivers that.** Outliving the character is why the record exists: a parent
edge cannot live in a structure keyed on a slot, because the slot is reused and
a watcher must read a parent after that parent has died.[^1] [^2]

All four world-level readers take an entity, and an entity that names a dead
character resolves to nothing. The parents of a dead character return nothing.
Its ancestors and its descendants return an empty list. Its relation to anybody
returns zero.

**Zero already meant two things and now means three.** Two characters with no
common ancestor stand at zero, and so do two whose only common ancestor is
beyond the stated depth. A dead character is the third, and a caller cannot tell
them apart.

**The answer names things the caller cannot ask about.** The ancestor walk hands
back descent identities, and an ancestor is usually dead. No world-level reader
takes a descent identity, so a caller holds an answer and has nothing to do with
it. That is the shape the register already holds from the control plane
side.[^3]

A review found this and filed the choice rather than the work, because which
shape the readers take is a judgement.[^4] The decision row recommends adding a
reader keyed on a descent identity beside each reader keyed on an entity, and it
names the two alternatives.

## What is missing before this is refined

- The impact review, and the decision row must close first.
- Whether the relation reader should distinguish its three zeros, and how. A
  caller that cannot tell unrelated from unreachable from dead is the reason
  this item exists, and adding a reader does not by itself fix it.
- What a reader returns for a descent identity the record never issued.
- Whether the same question reaches the control plane. Nothing exposes a
  character, a parent, a walk or a relation to Python today, so the need is
  served in Rust and not in Python, and the Python answer may decide the Rust
  shape.

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: ADR-0078, descent is a bounded record, and a relation is a bounded recursion, decision D1. `docs/adrs/draft/adr-0078-descent-is-a-bounded-record-and-a-relation-is-a-bounded-recursion.md`
[^2]: PRD-0015, a unit has parents and children. `docs/product/accepted/prd-0015-a-unit-has-parents-and-children.md`
[^3]: Findings register, FND-147. `docs/FINDINGS.md`
[^4]: Decisions register, DEC-092. `docs/DECISIONS.md`
