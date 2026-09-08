---
id: 0528
title: Let a stored policy play a demonstration to the end of a game
status: complete
created: 2026-09-07
implements: [ADR-0018 D3, ADR-0018 D4]
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**A watcher can seat a stored policy and cannot watch it finish.** The
demonstration now gives a faction to a weight file, and the faction acts. The
observation reader then refuses, and the run stops with a view error that says
the world cannot describe its own units.

Three probes place the condition. A world that steps with nobody acting reads
every observation for four hundred ticks and never refuses. A world with one
seated policy takes fifty-one decisions over five hundred ticks and never
refuses. A world with three seated policies refuses at tick one hundred and
ten. No renderer ran in any of the three, so the drawing pass is not involved.
The findings register holds the readings.[^1]

The verbs the three policies ran were the settling verb, the queue verb and the
build verb. Each of those adds a unit or a site, so the derived unit structure
is the first thing to look at.

## Impact review

**Governed by.** ADR-0018 D3 states that the derived unit structure rebuilds
at the frame barrier, by a full sort on the key, and never by an incremental
update while systems run. ADR-0018 D4 states that a per-tile answer reads that
structure. A verb outside a step reaches no barrier, so the verb restores the
structure with the same full rebuild. Nothing here updates it incrementally.

**Changes.** None. The work honours ADR-0018 rather than changing it.

**Creates.** None. No future contributor could reasonably choose to leave the
world unreadable between two steps, so the rule needs no record of its own.

**Blockers.** None.

**Precedent.** FND-644 held the first report. FND-647 records the cause and
what each of the four reports really showed.

## Done when

- A verb that a caller runs between two steps leaves the world readable.
- The refusal of a faction reader names its cause, and a stale structure
  names both revisions.
- A test drives the demonstration frames with two seated policies, and the
  founding of one does not stop the other.
- The tests fail when the defect is put back.

## Outcome

**The cause was a missing rebuild, not a reader asking too early.** A verb
that a caller runs between two steps changes the soldier arena and left the
derived unit structure behind it. The step rebuilds that structure at its
barriers, so a reader between the verb and the next step met a refusal. The
action verb, the founding verb, the conversion verb, the spawn verb and the
removal verb all did this, and each now restores the structure before it
returns.

**The learner environment never reported it because it reads before it
acts, once, for one faction.** The demonstration gives each seated policy its
decision in seat order, so the read of the second policy falls between the
verb of the first and the next step. A run with one policy has no such read.

**The refusal now names its cause.** The reader returned nothing at all, and
the binding turned that into one sentence that named none of the four
refusals the structure can give. Four reports of the defect named four
different events before the traceback, and none of them was the cause.

## References

[^1]: Findings register, FND-644 and FND-647. `docs/FINDINGS.md`
