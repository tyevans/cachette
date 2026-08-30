# ADR Registry (Index)

This document is an **index**. It lists every architecture decision record in
this project, assigns their numbers, and records what depends on what.

Rule 4 of the documentation rule exempts a table that lists files and paths as
data. The tables below are that table. They are this document's reference
list.

## How to use this registry

**Assign the number here before you write the record.** Add the row first,
with status `Proposed` and no file. Then write the file.

This registry is the allocator. Three number collisions occurred during the
research phase because agents chose their own numbers. That cannot happen if
the number comes from here.

**Never reuse the number of an accepted record.** A superseded record keeps
its number forever, because other records cite it.

A number may be **reclaimed** only when the record was never accepted and
nothing cites it. This has happened once: an omnibus draft held number 0001
before the research was done. It was deleted, not superseded, and 0001 was
reassigned. The reasoning is in git history.

**Decision numbers are local to their record.** Write `ADR-0004 D3`, not `D3`.
Flat global decision numbering collided three times during research and will
do so again.

## Status vocabulary

| Status | Meaning |
|---|---|
| `Proposed` | The number is reserved. No file exists yet. |
| `Draft` | The file exists and is under review. Not binding. |
| `Accepted` | Binding. Cite it. Do not edit it except to change status. |
| `Superseded` | Replaced. The row names the replacement. Keep the file. |
| `Rejected` | Considered and declined. Keep the file; the reasoning is useful. |

An accepted record does not change. To change a decision, write a new record
that supersedes it.

## What does not belong in a record

A record is a historical document. It must not hold material that changes.

Put these in the registers and cite them:

- Per-tick cost budgets. They change with every measurement.
- The byte budget table.
- Constant values that depend on an unanswered question.

The **rules** are decisions. The **numbers** are living reference.

## The records

### Core

| No. | Title | Status | Depends on | Source |
|---|---|---|---|---|
| 0001 | Determinism as the primary constraint | Proposed | — | 03, 07, 13 |

Everything else follows from this record. It is the only decision that cannot
be retrofitted.

### Foundations

| No. | Title | Status | Depends on | Source |
|---|---|---|---|---|
| 0002 | Target platform and value types | Proposed | 0001 | 07 |
| 0003 | Storage: dense tiles and a generational arena | Proposed | 0001, 0002 | 01 |
| 0004 | The level-of-detail pyramid | Proposed | 0001, 0003 | 02 |
| 0005 | The event log | Proposed | 0001 | 03 |
| 0006 | The Python boundary | Proposed | 0001, 0003 | 05 |

### Cross-cutting models

| No. | Title | Status | Depends on | Source |
|---|---|---|---|---|
| 0007 | The kernel vocabulary | Proposed | 0001 | 06, 13 |
| 0008 | Rate, constraint and set: the composition laws | Proposed | 0001, 0004 | 13 |
| 0009 | The frame loop and the static schedule | Proposed | 0001, 0005, 0007 | 06 |
| 0010 | Selectors and verbs | Proposed | 0004, 0007, 0012 | 04 |
| 0011 | The faction model | Proposed | 0002 | 08 |
| 0012 | The entity tiers | Proposed | 0003, 0006 | 14, 15, 16 |
| 0013 | The modifier pipeline and effective stats | Proposed | 0001, 0012 | 12 |

Records 0011 and 0012 have no single source report. Both are decisions that
emerged across several reports and were never written down. Breaking the
omnibus apart is what surfaced them.

### Subsystems

| No. | Title | Status | Depends on | Source |
|---|---|---|---|---|
| 0014 | Hex coordinates and geometry | Proposed | 0003, 0004 | 02 |
| 0015 | Movement and pathing | Proposed | 0007, 0014 | 06, 10 |
| 0016 | The field operator algebra | Proposed | 0004, 0008 | 13 |
| 0017 | Fog of war | Proposed | 0004, 0011 | 08 |
| 0018 | Influence maps | Proposed | 0011, 0016 | 09 |
| 0019 | Trade and resource flow | Proposed | 0008, 0016 | 11 |
| 0020 | Production and upkeep | Proposed | 0013, 0019 | 12 |
| 0021 | Needs and consumption | Proposed | 0012, 0020 | 15 |
| 0022 | Individual agency and occupations | Proposed | 0012, 0016 | 16 |
| 0023 | Group spatial dynamics and sites | Proposed | 0003, 0015 | 17 |
| 0024 | The character graph and inheritance | Proposed | 0012 | 14 |
| 0025 | Vector entity representation | Proposed | 0012, 0013 | 18 |

## Source reports

The `Source` column gives report numbers under `docs/research/reports/`. Those
reports hold the evidence. A record states the decision and cites the report;
it does not repeat the research.

The reports were written concurrently by agents that could not see each
other's work. They conflict in places. **Read the registers below before
writing any record.**

## Registers

These change over time. Records cite them; records do not contain them.

| Path | Purpose |
|---|---|
| `docs/BLOCKERS.md` | Work that is stopped, and what would start it |
| `docs/DECISIONS.md` | Choices that are open, with a recommendation |
| `docs/FINDINGS.md` | What the project believed and had to correct |
| `docs/research/reports/MERGE-NOTES.md` | Conflicts between the reports |
| `docs/research/research-agenda.md` | Fields still to investigate |
| `docs/reference/budgets.md` | Cost and storage tables. Not yet written. |

`FINDINGS.md` is precedent. When two sources disagree, look there first — the
conflict may already be settled.

## Layout

| Path | Contents |
|---|---|
| `docs/adrs/draft/` | Records under review |
| `docs/adrs/accepted/` | Accepted records |
| `docs/research/reports/` | Research that supports the records |

A file is named `adr-NNNN-<slug>.md` and moves between directories as its
status changes. The number never changes.

## Writing order

Write the core and the foundations first, records 0001 to 0006. Every
subsystem record cites them, so their vocabulary must settle before the rest
is written.

Then the cross-cutting models, 0007 to 0013. Then the subsystems.
