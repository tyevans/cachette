---
id: 0097
title: Write the layout record with the descent columns
status: complete
created: 2026-08-31
implements: [ADR-0066 D1, ADR-0066 D3]
changes: []
creates: [ADR-0021]
serves: []
blocked-by: [0067]
---

## Why

The character arena holds five columns and no parent edge. The descent and
succession pass is not built. A register row says the character tier wants
array-of-structs, and a reader takes that to govern the arena.[^1] The
correction says otherwise, and the decision row states the outcome and the
reasoning: the arena keeps struct-of-arrays.[^2] [^3]

Neither a register row nor a decision row binds anybody. The registry reserves
a number for the claim that layout follows the access pattern, and no file
holds it.[^4] The work that adds the descent columns is the work that can write
that claim honestly, because that work is the first to have a pass to point at.

## What the work does

1. Add the descent columns to the character arena: the two parent edges, the
   house, and the two Euler labels.
2. Write the record on the reserved row. State the constraint as the rule that
   a layout claim names one structure and one pass, and never a tier.
3. Give the alternatives that the decision row rejected, and say why the
   character row keeps its columns.
4. Cite the two research reports and the finding. Put no figure in the record.
5. Set the registry status to `Draft`, and close the decision row.

## The record is a deliverable, not a prerequisite

**Write the record inside the work that adds the descent columns. Do not write
it first, and do not make it a gate on that work.** Item 0067 adds the columns
and builds the pass.[^9] Until that pass exists, the record would state an
intent as a fact, which the scope rule forbids.[^8] The decision row says the
same, and it names this item as the home of the reserved row.[^3]

The order inside the work is therefore: add the columns, build the pass, then
write the record against the pass that exists. The record ships in the same
change as the columns.

## What holds this back

**Nothing writes the trait record.** The array-of-structs recommendation covers
a separate 64-byte trait record for the personality influence pass.[^5] That
structure does not exist. The record must state the constraint that decides a
layout, not name a structure that nobody has written.[^6]

**The record must hold no measured figure.** The crossover between the two
layouts is a measured figure, and every figure in this project is derived or is
taken on a machine that is not the target.[^7] Put a figure in the reference
tables and cite it.

## Impact review

**Governed by.** ADR-0066 D1 gives the living character its own column set, so
a layout claim about that column set is a claim about one shape and not about a
tier.[^10] ADR-0066 D3 fixes the shapes at compile time, so the layout is a
build-time property and the record binds the build.[^10] ADR-0054 declares the
tier at creation, and it makes no layout claim; the record must not read one
into it.[^11]

**Changes.** None. No accepted record states a layout for the character arena,
so the new record supersedes nothing.

**Creates.** ADR-0021, on the reserved row. The registry already holds the row
and the claim that layout follows the access pattern, so no number is
allocated by this work.[^4] The author sets the status to `Draft`. A reviewer
sets `Accepted`.

**Blockers.** BLK-007 governs every cost figure. The record states no figure,
and it states no crossover.[^7]

**Serves.** None. This work answers no recorded need. It closes a decision row.

**Precedent.** FND-072 records the misattribution that this record ends: a
figure from the trait record was read as a claim about the character row.[^2]
The record must therefore name the structure and the pass that each claim
covers, because naming the tier is what caused the error.

**Decisions.** DEC-032 closes when the record is written.

## Done when

- The character arena holds the descent columns, and a pass reads them.
- A record sits on the reserved row and states one claim: a layout claim names
  one structure and one pass, and never a tier.
- The record gives the rejected alternatives and the reason for each.
- The record holds no count, no file table, and no measured figure.
- The record cites its evidence in footnotes and repeats no research.
- The registry row reads `Draft` and names the file.
- The record check runs green.
- The whole check command runs green.

## Outcome

**Done on 2 September 2026.**

**The columns went to the record of descent, not to the character arena.** The
item said the arena, and item 0067 had already put descent in a separate
append-only record keyed on a descent identity rather than on a slot. The
house and the two Euler labels belong with the parent edges, for the reason
0067 gave for the edges: the arena reuses a slot after a death, so a column on
the arena would lose the house of every dead ancestor and a dynasty would end
whenever its founder died. The finding records the correction.[^12]

**The two parent edges were already there.** Item 0067 recorded them. This
work added the three that were missing: the house, and the two Euler interval
labels of the father forest.

**A pass reads them.** A relabel walks the father forest once in preorder and
writes the labels. After it runs, "is this character a patrilineal ancestor of
that one" is two integer comparisons at any depth of line, "every patrilineal
descendant" is one span of the Euler order, and a cadet split is a map over
that span. A house is named by the character that founded it, so a split
allocates nothing.

**The labels are ungapped, and a birth after the pass leaves the new row
unlabelled.** The research names the gapped variant as the optimisation to
take if a measurement demands it, and says not to build it first. No
measurement exists on the target platform, so nothing demands it. An
unlabelled row answers nothing rather than answering from a stale label.

**The record sits on the reserved row and states one claim**: a layout claim
names one structure and one pass, and never a tier. It gives four rejected
alternatives with a reason for each. It holds no count, no file table and no
measured figure, and it states no crossover. The registry row reads `Draft`
and names the file. A reviewer sets `Accepted`.

**The record carries one correction that the work found.** The research
describes a cadet split as a contiguous range write. It is contiguous in the
Euler order and gathered in the columns, because the rows are stored in birth
order. That is the same error the record forbids, in miniature: "contiguous"
means nothing until it names which array.

**A restored defect found one redundant declaration.** The record held a
stored watermark for how many rows carried a label, and the two label columns
already marked an unlabelled row with a sentinel. Removing the watermark check
left the whole suite green, which proved the check read nothing. The field is
gone and the Euler order length answers instead. FND-235 records it.[^13]

**Twelve tests cover the work, and five restored defects proved they can
fail.** A birth copying the mother's house failed every test, through the
invariant check. A subtree span one position too wide failed three. The house
column removed from the state hash failed exactly one, which is the test
written for it. A new row born carrying a label failed one. A split that wrote
the whole record instead of the span failed two.

## References

[^1]: Findings register, FND-022. `docs/FINDINGS.md`
[^2]: Findings register, FND-072. `docs/FINDINGS.md`
[^3]: Decisions register, DEC-032. `docs/DECISIONS.md`
[^4]: ADR Registry, reserved row 0021. `docs/adrs/REGISTRY.md`
[^5]: Vector entity representation, section 9 and decision D155. `docs/research/reports/18-vector-entity-representation.md`
[^6]: Decision Record Scope, section 4.4. `.claude/rules/adr-scope.md`
[^7]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^8]: Decision Record Scope, section 4.6. `.claude/rules/adr-scope.md`
[^9]: Backlog item 0067. `docs/backlog/complete/0067-record-a-parent-and-walk-a-line.md`
[^10]: ADR-0066, entity storage holds four fixed shapes, decisions D1 and D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^11]: ADR-0054, an entity belongs to one of three tiers, declared at creation. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^12]: Findings register, FND-236. `docs/FINDINGS.md`
[^13]: Findings register, FND-235. `docs/FINDINGS.md`
