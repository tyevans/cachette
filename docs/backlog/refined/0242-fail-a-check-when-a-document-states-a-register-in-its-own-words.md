---
id: 0242
title: Fail a check when a document states a register in its own words
status: refined
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A register is the current statement, and a document that repeats it in prose
becomes false the moment the register moves.** Nothing fails, because a document
is prose. This project has met the shape three times, and the third time it
reached about ninety documents in one blocker.[^1] [^2]

The defence is already written down. A document names a register by citation and
never states its content in its own words.[^1] Nothing enforces it.

**The absence was tested, not assumed.** After the sweep of the last case, one
repaired sentence was put back in its stale form and all eight document checks
passed.[^3]

## What the work does

1. Add a check that reads every tracked Markdown file and fails on a phrase from
   a named family. The first family is the state of measurement: the sentences
   that say no measurement exists, that nobody has measured, and that every
   figure in the project is derived.
2. Let three kinds of file state the family. The blockers register, the
   measurement register and the findings register own the statement, so the
   check exempts them by path.
3. Carry a baseline file for the sites a sweep may not repair. An accepted
   decision record is frozen, a review is a record of a moment, and a completed
   backlog item is a record of a moment. The baseline lists them, and it may
   only shrink. This is the pattern the footnote check already uses.[^4]
4. Give the check a broken fixture, so that the gate can prove the check
   fails.[^5]
5. Wire the check into the document gate and into the fixture gate beside the
   other seven.

## Impact review

**Governed by.** No decision record governs a check script. The documentation
rule states the habit this check enforces, and the definition of done states the
sweep this check makes cheap.[^6] [^7]

**Changes.** No record. The check reports a defect that no record denies.

**Creates.** No record. A check script is a mechanism, and the reasoning for it
is visible in the script header, so the test for whether a decision needs a
record fails on all three counts.[^8]

**Blockers.** None.

**Precedent.** FND-258 gives the evidence that no existing check sees the
defect.[^3] FND-223 gives the scale of the last instance.[^1] FND-042 gives the
two earlier instances.[^2]

## The judgement this item cannot avoid

**A phrase list is a blunt instrument, and the baseline decides whether it is
usable.** A check that fails on every frozen site fails on the day it lands, so
somebody must place every existing site in one of two sets: repair it, or
baseline it. That decision is the work, and it is larger than the script.

**A wide family produces noise and a narrow family misses the next case.** Start
with the family that has an instance. Do not invent families for registers that
have never gone stale.

## Done when

- The check fails on a fixture that holds one stale sentence.
- The check passes on the tree, with a baseline that names every site a sweep
  left in place.
- The baseline file states that it may only shrink.
- The gate that proves each check can fail runs this one too.
- The commit body holds the whole-tree search that produced the baseline.

## Outcome

Filled in when the item moves to `complete/`.

## References

[^1]: Findings register, FND-223. `docs/FINDINGS.md`
[^2]: Findings register, FND-042. `docs/FINDINGS.md`
[^3]: Findings register, FND-258. `docs/FINDINGS.md`
[^4]: The footnote baseline. `scripts/footnote-baseline.txt`
[^5]: The gate that proves a check can fail. `justfile`
[^6]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^7]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
[^8]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
