---
id: 0103
title: Derive a household from the dwelling slot
status: complete
created: 2026-08-31
implements: [ADR-0022 D1, ADR-0022 D2, ADR-0004 D1, ADR-0004 D4, ADR-0014 D2]
changes: []
creates: []
serves: [PRD-0014, PRD-0015]
blocked-by: []
---

## Why

A watcher cannot ask who lives together. PRD-0014 and PRD-0015 both need the
answer, and neither one owns it today. PRD-0015 states that a watcher can ask
who is in a household, and that a household follows from where people live
rather than from a second fact that somebody declares.

The unit already carries where it lives. The soldier arena holds a home
column that names a slot of the settlement arena, the founding writes it, and
the destroy path clears it.[^4] Nothing reads that column backwards.

## The answer this item takes, stated plainly

**A dwelling is stored and a household is derived.** A unit carries the slot
of the dwelling it lives in, and a household is every unit that carries one
slot.[^1]

**A household reads no descent.** Two strangers under one roof are one
household, and a parent and a child who live apart are two. A household is a
fact about a place, not a fact about a family.

**Nothing stores a household roster.** A stored roster would be a second
declaration of where a person lives, and nothing would fail when the roster
and the slot disagreed.[^2]

## Impact review

**Governed by.**

- **ADR-0022 D1 and D2.** Level 0 is the only source of truth, and every
  level above it is a pure function of level 0. The household is a pure
  function of the home column and the live column. It is not stored.
- **ADR-0004 D1 and D4.** Iteration order is explicit, and a sort uses a
  stable key. The members of a household come back in ascending arena slot
  order. That key is a property of storage, so no thread order and no
  completion order can reach it.
- **ADR-0014 D2.** Resolving an identity can fail. A read of a dead dwelling
  returns nothing rather than a wrong roster, and a read of a dead unit
  returns nothing rather than a stale dwelling.
- **ADR-0012 D3.** Units live in a generational arena. The read rebuilds each
  member identity from the slot and the generation the arena holds.
- **ADR-0067 D1.** The viewer reads the world and never writes to it. The
  household read takes a shared borrow and writes nothing.

**Governed by, and honoured without new work.**

- **ADR-0002 D1.** The read holds no floating point number. It moves
  identities only.
- **ADR-0018 D3.** The unit-to-tile bridge is derived and rebuilds at the
  barrier. The household read is not that shape: it reads the arena directly,
  so no staleness window exists and no rebuild is owed.

**Changes.** None. No accepted record changes.

**Creates.** None. The read states no constraint that ADR-0022 does not
already state, so it earns no record of its own.

**Blockers.** BLK-007 governs every cost figure, so this item states none and
the code holds none.

**No longer blocked by item 0059.** The item named 0059 as its blocker because
0059 was to supply the residence column. FND-116 records that the column, its
writer and its eviction path already exist, and FND-131 records that item 0059
planned three things the engine had already built.[^4] [^5] Item 0059 is
stopped because a review rejected ADR-0081. This item does not wait for it.

**Precedent.** FND-116 records that the site a unit draws from and the place a
unit lives are one fact under the record that fixes a settlement to a tile.
This item reads that one fact and adds no second one.[^4]

**A tension with PRD-0014, stated rather than hidden.** PRD-0014 rejects a
residency query that walks the population, because a watcher asks for
occupancy often. The occupancy number is already answered without a walk: the
cohort table holds a per-site, per-faction headcount derived from the same
home column. The roster is a different question, a watcher asks it of one
dwelling at a time, and this item answers it by a pass over the unit arena. No
measurement exists on the target platform, so nothing here can say whether
that pass is too slow.[^3] A separate item holds the reverse index, and it
should be taken when a measurement asks for it, not before.[^6]

## Done when

- A watcher names a dwelling and reads every unit that lives in it, through
  the public interface.
- A watcher names a unit and reads the dwelling it lives in.
- No structure stores a household, and no check reconciles two rosters,
  because there is only one.
- A unit that moves to another dwelling leaves one household and joins
  another, with no rule that names a household. A test asserts both sides of
  the move.
- A unit that lives nowhere is in no household, and that is a representable
  answer rather than an error.
- A dwelling with no residents reads as an empty household, not as an error.
- A dead dwelling identity and a dead unit identity each read as nothing.
- The read order over the members of one household is fixed by a stable key,
  and a test asserts that the members come back in the same order after the
  step has run at 1, 2 and 12 threads.
- The fixture holds a dwelling with one resident, a dwelling with several, an
  empty dwelling, a unit that lives nowhere, and a transfer between dwellings.
  The commit body says how that was checked.[^7]
- No cost figure appears in the code or in a comment.
- `just check` runs green.

## Outcome

**Done.** Three readers reach the public interface: a watcher names a dwelling
and gets its members, hands a buffer it owns and gets the same answer without
a new allocation, or names a unit and gets the dwelling it lives in. Nothing
stores a household. The settlement arena gained the reverse of its slot
reader, so a column that names a settlement slot can give the identity back
without a caller assembling one.

**A unit leaves a household by moving.** One column holds one slot, so the
write that puts a unit in a new dwelling is the same write that takes it out
of the old one. A test asserts both sides.

**The experiment.** Four perturbations went back one at a time. A comparison
that let a moved unit stay in the dwelling it left, a reversed read order, and
a removed guard against the value that means no home were each caught. A walk
over every slot instead of over the live units changed no answer, because the
arena clears the home of a slot it frees and its invariant states that a dead
slot names no dwelling. FND-156 records that, and the code cites it.[^8]

**One test passed for the wrong reason and was repaired.** The test that
proves a household needs no barrier asserted that the roster grew by one, and
it grew by one under the first perturbation as well. It now names the exact
membership on both sides of the write.[^9]

**Left undone, on purpose.** The control plane cannot read a household,
because the binding exposes no settlement and a binding written today would be
a capability nothing invokes. Item 0168 holds it. No reverse index was built.
Item 0167 holds it and waits on a measurement.[^6] [^10]

## References

[^1]: Decisions register, DEC-039. `docs/DECISIONS.md`
[^2]: Recurring Defect Shapes, section 1. `.claude/rules/recurring-defects.md`
[^3]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^4]: Findings register, FND-116. `docs/FINDINGS.md`
[^5]: Findings register, FND-131. `docs/FINDINGS.md`
[^6]: Backlog item 0167. `docs/backlog/proposed/0167-index-the-units-of-one-dwelling.md`
[^7]: Testing Rules, section 2a. `.claude/rules/testing.md`
[^8]: Findings register, FND-156. `docs/FINDINGS.md`
[^9]: Findings register, FND-143 and FND-148. `docs/FINDINGS.md`
[^10]: Backlog item 0168. `docs/backlog/proposed/0168-let-the-control-plane-read-a-household.md`
