---
id: 0029
title: Write the three storage records that entity movement needs
status: complete
created: 2026-08-30
implements: []
changes: []
creates: [ADR-0012, ADR-0014, ADR-0018]
serves: [PRD-0002]
blocked-by: []
---

## Why

Sprint 2 puts entities in storage and gives a tile a way to answer which
entities stand on it. Three registry rows hold the claims that work needs,
and none of them has a file.

- Row 0012: tiles are dense columns and units are a generational arena.
- Row 0014: entity identity is an index plus a generation.
- Row 0018: the unit-to-tile bridge, which the review retitled because units
  do not stay sorted by tile.

Item 0007 was going to write ten storage records in one pass. It was returned
to `proposed` at sprint 2 planning, because a record written before the work
that needs it states what the author expects rather than what the code does.
This item takes the three rows the sprint reaches.

## Impact review

**Governed by.** The record scope rule: each row passes the three-condition
test before its file exists, and a row that fails it is dropped with its
number retired.[^1] ADR-0066 fixes four entity shapes, so a storage record
must hold that claim rather than restate it.[^2] ADR-0004 governs row 0018,
because the bridge rebuild needs a stable key and an explicit
order.[^3] ADR-0017 governs the tile side of the bridge, because a tile is
addressed by a raw axial pair.[^4]

**Changes.** None. No record exists for these three numbers.

**Creates.** ADR-0012, ADR-0014 and ADR-0018, minus any row that fails the
test. Row 0013 says the project writes its own entity storage rather than
adopting an off-the-shelf one; that claim belongs to this group by subject,
but no code in this sprint depends on it, so it stays a row.

**Blockers.** BLK-007 governs every cost figure, so none of the three records
states one. The chunk size is a compile-time constant that the project
measures on the target platform, and ADR-0066 already says so, so a record
here states the constant parametrically and cites the blocker.[^5]

**Precedent.** FND-042 records that a registry row stated a claim while the
blocker governing it was open. Each of these three rows is checked against
the open blocker before its file is written.[^6] FND-033 records that record
length predicts churn, which is the other reason this item is three records
and not ten.[^6]

## Done when

- Each of rows 0012, 0014 and 0018 has a file, or has been dropped with its
  number retired and the reason recorded.
- Each record states one claim that a reviewer could reject on its own.
- Each record has numbered decisions, so the code that implements it cites
  one.
- Each record states the alternative it rejects and why.
- No record holds a cost figure, a version, a count, or a module
  arrangement.
- No record contradicts ADR-0066, and each says which of the four shapes it
  covers.
- The registry rows move to `Draft`.
- The record check and the citation check both pass.

## Outcome

All three rows have files and all three are accepted. None failed the
three-condition test.

The review changed all three. ADR-0012 D1 said a tile lookup is an array
subscript, which pre-empted ADR-0016's block order; it now forbids consulting
a table instead. ADR-0014 D4 rested on a false premise about slot reuse and
now states the real force. ADR-0014 gained D6, which is a defect the approver
missed: the identity packs into a value that cannot be zero, so slot zero at
generation zero was unrepresentable and the first entity the engine ever
allocates would have had no identity. ADR-0018's title claimed units stay
sorted by tile, which the record does not say and ADR-0014 forbids; the title,
the file name and the registry row all changed.

The last of those reached outside this item. ADR-0056 asserted the same
invariant in its context and consequences, and it was corrected in the same
change. A record that depends on a guarantee its dependency does not give is
worse than an unwritten record, because both look finished.

## References

[^1]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^2]: ADR-0066, entity storage holds four fixed shapes. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^3]: ADR-0004, iteration order is explicit. `docs/adrs/accepted/adr-0004-iteration-order-is-explicit.md`
[^4]: ADR-0017, the world is a rhombus, so a tile index is raw axial. `docs/adrs/accepted/adr-0017-the-world-is-a-rhombus-so-a-tile-index-is-raw-axial.md`
[^5]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^6]: Findings register. `docs/FINDINGS.md`
