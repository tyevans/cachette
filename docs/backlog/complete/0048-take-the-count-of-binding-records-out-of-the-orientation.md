---
id: 0048
title: Take the count of binding records out of the orientation file
status: complete
created: 2026-08-31
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

The orientation file says "Eight records are binding: the seven determinism
records and the entity storage record." The registry says fifteen.

The file contradicts itself two sentences later, where it says that the
registry holds the status of every record and is the only place that does. A
reader who believes the first sentence writes code against eight constraints
and misses seven.

**The count was already wrong before the work that made it worse.** At the
start of the session that raised this item the registry held thirteen accepted
records against a claim of eight. The claim decays every time a record is
accepted, which is a thing the project does on purpose and often.

This is the shape the recurring-defect rule names first: one fact in two
places, with nothing that fails when the copies disagree. The orientation file
is the most read document in the repository, so it is the worst place to keep
a second copy.

The same file already carries the lesson. Its open questions section says
plainly that a register does not decay and a summary does, that the section
once held a summary, that the summary went stale, and that a finding records
what it cost. The status section is the same mistake in the same file.

## What the work does

1. The status section states what is settled without counting it.
2. It points at the registry for which records are binding, in the way the
   open questions section points at the blockers register.
3. Nothing else in the file states a count that a change to the tree can
   falsify.

## Impact review

**Governed by.** The registry states that it holds the status of every record
and that no other place does. The scope rule forbids a count in a record and
gives the reason, which applies to any document that decays.

**Changes.** No record changes. One orientation file loses a count.

**Creates.** No record.

**Blockers.** None.

**Precedent.** The finding that the open questions section went stale is in
the register with its evidence. This is a second instance in the same file,
and the item records it as one rather than treating it as new.

## Outcome

The status section says to read the registry for which records are binding,
and to read those records before writing code. It counts nothing.

**It also says why.** A reader who arrives at a section that used to hold a
list is entitled to know that it stopped holding one on purpose, and the
sentence points at the finding.

**Recorded as a second instance rather than a new finding.** FND-039 already
held this shape, found in the open questions section of the same file. A
second number for the same shape in the same file would spread the evidence
across two rows and make neither look like a pattern. The instance is
evidence, and it sits under the finding it is evidence for.

**No check can catch the next one.** Nothing reads prose for a number. The
only defence is the rule that a document naming a register names it and stops,
and that rule is in the finding.
