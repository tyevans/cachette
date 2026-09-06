---
id: 0244
title: Fail a check when the two project orientations disagree
status: complete
created: 2026-09-03
implements: []
changes: []
creates: []
serves: []
blocked-by: []
---

## Why

**The project orientation is two tracked files and a symlink, and they have
already diverged.** One carries a sentence that a sweep repaired. The other
still says that no measurement exists on the target platform, so an agent that
reads the second reads something false. Nothing failed.[^1]

**The rule already names the defence.** One value is declared in two places,
both look authoritative, and the failure is silent. When a second site must
exist, add a check that fails when the copies disagree.[^2]

**The comparison needs no judgement.** The two orientations differ only in the
numbering of their footnotes and in the directory named inside them. The five
rule files beside them differ only in that directory. A rewrite makes the copies
comparable, and a comparison is then exact.

## What the work does

1. Add a check that reads each pair: the two orientation files, and each rule
   file against its mirror.
2. Normalise the copy before comparing. Rewrite the directory name in each path,
   and rewrite the name of the orientation file. Renumber the footnote markers
   and definitions, or compare the body with the markers removed and the
   reference sections separately.
3. Fail with the first line that differs after normalisation, so that a reader
   sees which copy is behind.
4. Give the check a broken fixture, so that the gate can prove it fails.[^3]
5. Wire the check into the document gate and into the fixture gate.

## Impact review

**Governed by.** No decision record governs a check script. The recurring defect
rule states the shape and the defence.[^2]

**Changes.** No record.

**Creates.** No record. A check script is a mechanism, and the reasoning is
visible in the script header.[^4]

**Blockers.** None.

**Precedent.** FND-259 holds the instance and the evidence.[^1] FND-050 holds
the general rule that a value derived from the tree needs something that fails
when the copies disagree.[^5]

## The judgement this item cannot avoid

**The normalisation is the whole design, and it can hide a real difference.** A
rewrite that is too broad makes two files compare equal when they say different
things. Keep the rewrite to the directory name and the orientation file name,
and do not add a rule to make a failure go away.

**Neither file may be edited by this item.** Both belong to the project owner.
The check reports the disagreement; the owner decides which copy is right.

## Done when

- The check fails on a fixture whose two copies disagree in one sentence.
- The check reports the current disagreement between the two orientations rather
  than hiding it, and the report names the file and the line.
- The gate that proves each check can fail runs this one too.
- The commit body holds the search and the difference that found the instance.

## Outcome

Built, and the design changed. The item asked for a comparison of two
hand-written files. Two of the three orientation names were already symlinks to
the third, so the divergence lived in the one name that was still a regular
file. Copying the item's design would have preserved the copy it set out to
police.

**The orientation is now one document under three names.** The third name is a
symlink as well, which is how this repository already mirrors its rules
directory and its skills directory. Neither copy was edited into agreement:
one of them stopped being a copy.

**Each name now holds exactly what the others hold, and that is the point.**
The layout table of the canonical file already names both directories and both
symlinks, so a reader of any name reads the same table. The stale copy named
one directory and neither symlink.

**The check stays, and it earns its place a different way.** It fails when
somebody replaces a symlink with a regular file that says something else. It
compares content and does not demand the link, because a checkout on a
filesystem with no symlink must still work. It reads the rules directory and
the skills directory the same way, file by file.[^6]

**The normalisation is two rewrites and no more.** It rewrites the canonical
directory name to the mirror directory name in a path, and it rewrites the name
of an orientation file. A wider rewrite would make two files compare equal
while they say different things.

**The check reported the live disagreement before the repair.** It named the
file and the line, which is what this item asked for. The finding holds what
the reading changed.[^7]

**It was proved able to fail on the tree, not only on a fixture.** The symlink
was replaced by a copy that said two million units instead of one million, the
check failed on that line, and the symlink was restored.

The item said that neither file may be edited by this work. The project owner
directed the change through the dispatcher, and the change deletes a copy
rather than editing one.

## References

[^1]: Findings register, FND-259. `docs/FINDINGS.md`
[^2]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^3]: The gate that proves a check can fail. `justfile`
[^4]: Decision Record Scope, section 1. `.claude/rules/adr-scope.md`
[^5]: Findings register, FND-050. `docs/FINDINGS.md`
[^6]: The orientation check. `scripts/check_orientation.py`
[^7]: Findings register, FND-531. `docs/FINDINGS.md`
