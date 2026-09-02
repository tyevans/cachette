---
id: 0204
title: Re-review the two corrected records
status: complete
created: 2026-09-02
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

A review returned two drafts, each for one sentence that the tree falsified.
An author corrected both. A correction needs a reader that did not write it,
and the author of a correction cannot accept it.

The author declined the reviewer's replacement text for one of the two and
gave a reason. A reason from an author is evidence, not an instruction, so the
re-review had to test it rather than accept it.

## What the work does

1. Read each corrected passage against the tree.
2. Test the author's reason against the accepted record it rests on.
3. Set the registry status of each record, or leave it and say why.
4. Record what the re-review found beyond the two passages.

## Impact review

**Governed by.** The registry, for who may review.[^1] The reviews guide, for
what a review must contain.[^2] The record scope rule, for what may sit in a
record.[^3]

**Changes.** No record changes. A reviewer that edits a record has authored it.

**Creates.** No record.

**Blockers.** None.

## Done when

- Each corrected passage is read against the code.
- The author's reason for declining the reviewer's text is tested against the
  record it cites, and the outcome is stated plainly.
- One review file holds every objection attempted and a verdict for each
  record.
- The registry status of each record is set or explicitly left, with the
  reason.
- The registers hold anything found beyond the two passages.
- The document checks pass.

## Outcome

**Both records earned an accept. Neither status moved, and the reason is
mechanical.**

The correction to ADR-0088 separates building the field from building a world,
and both halves match the build test and the finding the old bullet
contradicted.[^4]

**The author was right and the reviewer was wrong about ADR-0090.** The
reviewer's replacement text narrowed the claim to "every caller that asks how
many units may stand on a tile". The founding asks exactly that and reads the
ground alone, so the reviewer's wording would have left a false universal in
place. The author keyed the claim on enforcement instead. That key is not the
author's judgement: an accepted record already states that admission is the
only enforcer, and it names the founding's behaviour in a decision of its
own.[^5]

**Accepting a record turned out to be a whole-tree sweep.** A citation names
the path of a record, and the path holds the directory, so moving a record
breaks every citation of it. Nineteen of ADR-0090's sit in source comments,
which this work was forbidden to touch. The finding and the open row hold
it.[^6] [^7]

**A fourth caller of the capacity question was found.** The drawing pass counts
a tile as at its capacity against the ground alone. A watcher would read a
correctly filled made way as over-full. It enforces nothing, so the record
stays true, and the registers gained the caller.[^8]

**One registry repair was made.** Row 0090 did not name ADR-0074 among its
dependencies, and decision D3 now rests on it. The first review missed it.

## References

[^1]: ADR Registry, who reviews. `docs/adrs/REGISTRY.md`
[^2]: Reviews guide. `docs/reviews/README.md`
[^3]: Decision Record Scope. `.claude/rules/adr-scope.md`
[^4]: Findings register, FND-162. `docs/FINDINGS.md`
[^5]: ADR-0074, a spawn may over-fill a tile, and only admission enforces the capacity, decisions D2 and D4. `docs/adrs/accepted/adr-0074-a-spawn-may-over-fill-a-tile-and-only-admission-enforces-the-capacity.md`
[^6]: Findings register, FND-197. `docs/FINDINGS.md`
[^7]: Decisions register, DEC-083. `docs/DECISIONS.md`
[^8]: Findings register, FND-193. `docs/FINDINGS.md`
