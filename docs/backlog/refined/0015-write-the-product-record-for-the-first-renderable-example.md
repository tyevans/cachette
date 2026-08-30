---
id: 0015
title: Write the product record for the first renderable example
status: refined
created: 2026-08-30
implements: []
changes: []
creates: [PRD-0002]
serves: []
blocked-by: []
---

## Why

The project builds a first renderable example. Nobody has stated the need it
answers, so nobody can tell when it is met. A backlog item that serves no
recorded need is allowed, but a whole sprint sequence that serves none is a
plan without a target.[^1]

The need is not "a renderer". A renderer is a structure, and a structure
belongs in a decision record. The need is that a person cannot see the
simulation run, so nobody can tell a working engine from a broken one by
looking.

## Impact review

**Governed by.** No decision record governs a product record. The product
guide states the six questions the record must answer, and the registry
allocates the number.[^1] [^2]

**Changes.** None.

**Creates.** PRD-0002. The registry row is allocated with status `Idea` and
no file.

**Blockers.** None. BLK-013 and BLK-014 are answered, so the record states no
value that an open blocker governs. BLK-007 still holds every cost figure, so
the record states its cost as a derivation and cites the blocker.[^3]

**Precedent.** FND-041 records that four documents stated the project state
and none of them was checked. This record states a need, not a state.[^4]

**Serves.** Itself. This is the record every later item in the sequence cites.

## Done when

- The registry row for PRD-0002 exists before the file does.
- The file answers all six gate questions, each in its own section: the
  audience, what that person cannot do today, what good looks like as a
  checkable statement, what it does not do, what it costs at the target
  scale, and which blockers govern it.
- The record names one audience.
- The record states no data structure, no algorithm and no module
  arrangement.
- The record states no measured figure. Every cost is derived and says so.
- The product check passes.
- The record sits in `shaped/` with status `Shaped`, which an author may set.

## Outcome

Filled in on completion.

## References

[^1]: Product requirement records. `docs/product/README.md`
[^2]: Product Registry. `docs/product/REGISTRY.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-041. `docs/FINDINGS.md`
