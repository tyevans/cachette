---
id: 0234
title: Settle which end of a membership stores it, and repair the register
status: proposed
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The project holds two answers to one question, in two places, and nothing
fails when they disagree.**

The blocker on formations is resolved, and its text says formation membership is
an ownership column plus a reverse index. That stores the membership on the
member: the unit carries the identity of its group.[^1]

ADR-0065 stores it on the group: the group holds a row of entries and each entry
names a member or names nobody. The position table implements the record, not
the register.[^2]

**The record calls these one answer.** Its context quotes the register
accurately and its D1 then says the register already resolved the case this way.
A review found otherwise and returned the record.[^3]

**The difference is not cosmetic.** The record's best idea is that an empty entry
is a state and not an absence, because a group that stored only its filled
entries could not say what it is short of. A reverse index derived from an
ownership column cannot hold a vacancy: nothing owns an empty seat.

**A resolved blocker that states a superseded answer is worse than an open
one**, because nothing marks it stale and it carries the authority of a settled
question. The cost is zero today, because only the workforce case is built. It
falls on the first person who builds a formation from the register.

The decision row holds three options and recommends the record's direction.[^4]

## What is missing before this is refined

- The impact review, and the decision row must close first.
- Whether a formation wants the register's shape after all. If it does, the
  answer is two directions stated deliberately, with a rule that says which
  relation takes which, and not two directions by accident.
- How a resolved blocker records a superseded answer. Reopening it records a
  history that did not happen, and overwriting it loses what the project once
  believed.
- Whether the residence column is in scope. A unit carries the slot of the site
  it lives in, and a reverse index from the site was refused for its own
  reasons, so the project already holds both directions for relations that are
  not groups.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Blockers register, BLK-010. `docs/BLOCKERS.md`
[^2]: ADR-0065, a group is a site membership, not a region, decision D1. `docs/adrs/draft/adr-0065-a-group-is-a-site-membership-not-a-region.md`
[^3]: Review 0223, the group membership record. `docs/reviews/0223-the-group-membership-record.md`
[^4]: Decisions register, DEC-091. `docs/DECISIONS.md`
[^5]: Decisions register, DEC-036. `docs/DECISIONS.md`
