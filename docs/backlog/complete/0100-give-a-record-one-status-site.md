---
id: 0100
title: Give a record one status site
status: complete
created: 2026-08-31
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The registry allocates a record's number and holds its status, and it says so.
It is meant to be the only document that holds it.

Most record files carry a `Status:` line as well. Six do not, and those six
are the most recently written. So the project has two shapes of record, one
that states its status twice and one that states it once, and nothing fails
when the two copies disagree.

This is the first shape in the recurring defect list: one value declared in
more than one place, with nothing that fails on disagreement.[^1] The register
already holds a related instance, where a citation carried a status and the
status went stale.[^2] The acceptance of fourteen records showed the cost: the
registry rows changed in one edit, and the status line inside each file had to
be swept separately.

## What the work does

Remove the `Status:` line from every record file. Keep the registry row as the
only site. Add a check that fails when a status line comes back.

## Impact review

**Governed by.** No decision record governs this work. A record states a
constraint on the engine, and this item changes the shape of a document and
one check script. Three rules govern it instead: the recurring defect rule,
which names the shape and states that a second site needs a check that fails
on disagreement;[^1] the record scope rule, which says an accepted record does
not change except in status;[^5] and the registry, which states that it holds
the status and is the only document that does.[^4]

**Changes.** No record changes. The work deletes one line from twenty-four
accepted record files and changes no decision, no title and no reference in
any of them.

**Creates.** No record. The work states no new constraint. The registry
already states that it holds the status, so a record that repeated it would be
a description, and the scope rule refuses a description.[^5]

**Blockers.** None. No value here waits on a measurement.

**Precedent.** FND-055 records the same shape one level out: a footnote called
a record a draft after the registry accepted it, and the copies disagreed.[^2]
The check for that case already exists, and its comment names the registry as
the single site.[^3] This item finishes the same argument inside the record
file.

**Serves.** No product record. This is repository hygiene, not a user need.

## Which site wins

The registry wins. Three reasons, and none of them is preference.

1. **The registry is the allocator.** It creates the row before the file
   exists. A status of `Proposed` means no file exists yet, so the registry
   must hold a status that no file can hold. A file line can therefore never
   be the whole record of status, and a partial site is the worse site.
2. **A status is a fact about process, not about content.** It says where a
   record sits in review. The record's own text states a constraint. The
   record scope rule keeps material that changes out of a record body, and a
   status changes at least twice in a record's life.[^5]
3. **The check that already exists chose the registry.** The citation check
   fails when a footnote calls an accepted record a draft, and it states that
   the registry holds the status.[^3] A second, opposite answer inside the
   record file would leave the project holding both.

The other direction was considered. If the file line won, the registry would
have to derive its status column from the files. That fails at once, because a
`Proposed` row has no file to derive from, and because the registry is one
table a reader scans while thirty files are not.

## Done when

- No record file under `docs/adrs/` holds a status line.
- The record check fails when a status line appears in a record file.
- A broken fixture reintroduces a status line, and the probe recipe proves
  that the check rejects it.
- A whole-tree search finds no record stating a status, and the search command
  is in the commit body.
- The registry row of every record keeps the status value it holds today. This
  work removes a duplicate declaration. It re-decides nothing.
- `just records`, `just records-probe` and `just check` exit 0.

## What to be careful of

- **A record's history is in git.** Removing a line from twenty-four accepted
  records is a large diff that changes no decision. Say that in the commit
  body.
- An accepted record does not change except in status, so this edit is inside
  what the retcon window permits. Say so in the commit.[^4]
- Do not add a status line to the six that lack one in order to make them
  uniform. That is the wrong direction, and it doubles the problem.
- If a file line and its registry row disagree, that is a finding. Stop and
  report both values. Do not pick one.

## Outcome

The registry won, as the review expected. Every record file lost its status
line. No decision, title, section or reference changed in any record.

No file disagreed with its registry row before the sweep, so the work filed no
finding. The commit body holds the count and the search command.

The record check gained a status check and an optional directory argument. The
argument lets the probe recipe run the check against a broken fixture, in the
shape the product record check already uses. A new fixture holds one record
that is correct except for a status line, so the probe fails for one reason
and names it. The check was also proved against the real tree: a status line
put back into one accepted record made the check fail, and the removal made it
pass again.

No register entry opened or closed. The work created no record and no number.

## References

[^1]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^2]: Findings register, FND-055. `docs/FINDINGS.md`
[^3]: The citation check script. `scripts/check_citations.py`
[^4]: ADR Registry, the status vocabulary and the retcon window. `docs/adrs/REGISTRY.md`
[^5]: Decision Record Scope, sections 4.1 and 7. `.claude/rules/adr-scope.md`
