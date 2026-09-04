---
id: 0421
title: Put the two quantity passes into the stage table
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The step opens a stage around each pass it runs, and the frame cost table reads
those stages. A pass that opens no stage is absent from every cost report the
project takes, and its cost falls silently into the gap between two neighbours.

**Two passes move a quantity and neither opens a stage.** The delivery pass
moves a carried load into the store of a unit's own site. The contract
settlement pass moves a carried load into the store of another faction's site
and then fails every contract that reached its deadline. A finding records
both.[^1]

Nothing fails today. The cost table adds up to less than a frame and no check
compares the two. A contributor who plans against the table plans against a
frame that is missing two passes.

The repair must add both at once. Adding one leaves the other, and a table that
covers one of two passes of one kind is harder to read than a table that covers
neither.

## Impact review

Not done. This item stays in `proposed/` until somebody names the records that
govern the stage list and says whether a stage may be added without a record.

Two things the review must settle.

**The stage list has a count and a test that reads it.** A test drives one
frame and compares the table against the declared entry count of each stage, so
a new stage is a change to a test as well as to the list.

**Each stage declares whether it takes a thread count.** Neither of these two
passes does. Both run on the calling thread and write one store at a time in a
stated order. A declaration that said otherwise would misdirect every plan made
from the table, which is the failure another item already holds.

## Done when

- Both passes open a stage, and the stage list names them.
- The test that compares the table against the declaration passes.
- A frame cost report names both passes.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-431. `docs/FINDINGS.md`
