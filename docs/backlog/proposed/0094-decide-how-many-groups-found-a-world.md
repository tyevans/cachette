---
id: 0094
title: Decide how many groups found a world
status: proposed
created: 2026-08-31
implements: []
changes: []
creates: []
serves: [PRD-0012]
blocked-by: []
---

## Why

A run begins with one group, in one place, of one faction. The world holds a
faction count, and every faction except the founding one begins with nobody
and nothing.

The product record does not decide how many groups found a world. It names one
group and one group for each faction as the two candidates, and it says
plainly that this is not its question.[^1]

The choice changes the early run more than any rule that acts on it. One
founding gives a run with one society and an empty map around it. One founding
for each faction gives a run in which the factions meet, and the tick on which
they meet follows from how far apart the engine put them.

## What the work does

1. The engine founds the number of groups the answer says.
2. Two founding places do not overlap, and the rule that keeps them apart is
   stated rather than assumed.
3. A watcher can see every founded place and compare them.
4. The demonstration shows the answer.

## What is missing before this is refined

- **The answer.** The blocker holds it, and the project owner owns it.[^2] Do
  not invent a number here.
- **The separation rule.** Two groups drawn from one bounded sample can land
  on one tile, or within one disc of each other. Whether the second founding
  refuses a place near the first, and by how much, is a decision no record
  holds.
- **Whether a second founding widens the sample.** The founding record refuses
  a sample that widens until it succeeds.[^3] A second founding that must
  avoid the first may find nothing in its sample, and refusing is a correct
  outcome under that record. Confirm that this is acceptable before writing
  code that works around it.

## Done when

- The blocker is closed with the owner's answer, and the tree is searched for
  its number so that nothing still calls it open.[^4]
- The engine founds the number of groups the answer says.
- A test asserts the separation rule at its boundary.
- `just check` runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: PRD-0012, a world starts small and grows. `docs/product/shaped/prd-0012-a-world-starts-small-and-grows.md`
[^2]: Blockers register, BLK-018. `docs/BLOCKERS.md`
[^3]: ADR-0075, the founding choice reads a bounded sample of the world, a draft record. `docs/adrs/draft/adr-0075-the-founding-choice-reads-a-bounded-sample-of-the-world.md`
[^4]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
