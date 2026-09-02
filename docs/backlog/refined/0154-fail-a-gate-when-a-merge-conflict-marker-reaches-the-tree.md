---
id: 0154
title: Fail a gate when a merge conflict marker reaches the tree
status: refined
created: 2026-09-01
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The decisions register carried an unresolved merge conflict marker in its
reference section, on the main branch, across several commits. A reader found
it while rebasing. No check found it, and the gate suite ran green over it
each time.

That is the point of the item. A gate that passes is evidence that the rules
the gate encodes hold, and nothing more. The register checks read the entries
and the next number. The record checks read the records. Markdown does not
care. The findings register holds this class of mistake.[^1]

A parallel run merges the registers often, so the case will recur.

The search costs nothing. One grep over the tree finds every marker in under a
second, which is why the absence of the check is worth closing rather than
living with.

## What the work does

Add a check that searches the tree for a conflict marker at the start of a line
and fails when it finds one. Run it with the other checks.[^2]

The search must cover the whole tree, not a list of directories somebody
thought of. A marker in a source file, a test fixture or a register is the same
defect.

## What it must not do

It must not skip a directory to make a fixture pass. A fixture that must hold a
marker belongs behind an exception the check names, or the fixture holds the
marker in a form the check does not match.

## Impact review

**Governed by.** No decision record governs this work. The check reads text and
writes nothing. It touches no simulation state, no arithmetic and no ordering,
so no decision of the determinism record applies.[^3]

**Changes.** No record. The item adds a target to the gate list and a script
beside the other checks.

**Creates.** No record. A check that a rule already implies needs no record. The
rule is that a merge is finished when the markers are gone, and a future
contributor could not reasonably choose otherwise, so the first condition of the
scope rule fails.[^4]

**Blockers.** None. The check needs no value that a measurement supplies.

**Precedent.** The finding that opened this item records the failure and its
cost.[^1] The recurring defect shapes name the general form: a rule that nothing
enforces is a rule that holds until somebody is busy.[^5]

**Serves.** No product record. The work answers to a gate, not to a need of a
player.

## The four markers

Git writes four line-initial markers, and the check reads all four. Three come
from the default conflict style: seven `<`, seven `=` and seven `>`. The fourth,
seven `|`, comes from the `diff3` and `zdiff3` styles, which a contributor may
set in a local configuration. A check that reads three of the four passes over
a tree that the fourth marks.

A Markdown heading underlined with `=` is the one shape that resembles a
marker. The check reads a line of exactly seven characters followed by a space
or by the end of the line, and a heading underline is a run of any other length
in practice. The tree holds no such underline today, so the shape is a risk to
name rather than a false failure to work around.

## The mode this check runs in

**It fails the gate.** A conflict marker is never intentional and never
ambiguous. The check has no judgement to make, so it cannot cry wolf, and a
false failure is the only reason to prefer a report.[^6]

## Done when

- A conflict marker anywhere in the tree fails a gate.
- The check reads all four markers, not the three of the default style.
- The check has a broken fixture that it rejects, and the probe recipe runs it,
  in the way the record checks do.[^7]
- The fixture holds a real marker, and the repository scan passes over the
  fixture directory by name rather than by a rule that a real file could also
  match.
- The check runs against the real tree and the result is reported.
- The whole check command runs green.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-136. `docs/FINDINGS.md`
[^2]: The check targets. `justfile`
[^3]: ADR-0001, one binary gives one answer at any thread count. `docs/adrs/accepted/adr-0001-one-binary-gives-one-answer-at-any-thread-count.md`
[^4]: Decision Record Scope, the test for whether a decision needs a record. `.claude/rules/adr-scope.md`
[^5]: Recurring Defect Shapes. `.claude/rules/recurring-defects.md`
[^6]: Definition of Done, pass the gates. `.claude/rules/definition-of-done.md`
[^7]: Backlog guide. `docs/backlog/README.md`
