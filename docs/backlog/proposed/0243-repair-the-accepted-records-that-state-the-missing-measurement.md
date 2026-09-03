---
id: 0243
title: Repair the accepted records that state the missing measurement
status: proposed
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A sweep repaired every document it was free to touch, and stopped at the
accepted decision records.** Those records say in their own words that no
measurement exists on the target platform. A benchmark ran on 3 September 2026,
so the sentence is now false in the general case.[^1]

**The freeze is why they were left.** An accepted record does not change except
in status. Repairing a citation is not an amendment, and the registry says so.
Rewording the sentence that carries the footnote marker is an amendment, and the
retcon window governs it. Every one of these records has dependents, so the
window is shut.[^2]

**The decision that unblocks this is open and a reviewer owns it.** DEC-096 asks
how the project repairs a stale consequence inside an accepted record. It
recommends striking the stale sentence in place, and it names the boundary. This
item is the same case, at a larger count, and it waits on that row.[^3]

## What the work does

1. Wait for DEC-096 to close.
2. If the answer is repair in place, strike the sentence in each record and
   leave the citation attached to the claim it supported. Say in the commit what
   changed and why the freeze did not apply, as the registry requires.[^2]
3. If the answer is supersession, write one record and say what it replaces.
4. Find the records by search, not by a list. The sweep that found them left its
   command in a commit body, and the finding holds the phrase family.[^4]
5. Repair the doc comments in the engine in the same change. They are call
   sites, and the sweep left them alone because they were out of its scope.
6. Repair the header of the provisioning script, which still says that no
   measurement exists on the target platform. That script is the one that took
   the measurement.

## What is missing before this is refined

- **DEC-096 is open.** The repair form follows from it, and no plan here
  survives the other answer.
- **Whether a source comment is covered by the freeze at all.** A doc comment is
  not a record. It probably repairs freely, and nobody has said so.
- **Who repairs the rule files and the project orientation.** Both carry the
  sentence with no blocker number near it, and both belong to the project owner.
  This item cannot touch either.
- **Whether the check in item 0242 lands first.** If it does, this item's
  outcome is a baseline that shrinks rather than a sweep, and the two should be
  sequenced.[^5]

## Done when

Stated when the item is refined.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^2]: ADR Registry, the retcon window and the citation rule. `docs/adrs/REGISTRY.md`
[^3]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^4]: Findings register, FND-258. `docs/FINDINGS.md`
[^5]: Backlog item 0242. `docs/backlog/refined/0242-fail-a-check-when-a-document-states-a-register-in-its-own-words.md`
