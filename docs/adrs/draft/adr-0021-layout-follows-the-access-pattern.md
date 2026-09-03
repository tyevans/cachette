# ADR-0021: A layout claim names one structure and one pass, and never a tier

## Context

This project stores its entities in four fixed shapes, and each shape carries
its own set of columns.[^1] A living character is one of the four. Its columns
are held as struct-of-arrays: one array for each field, and a row spread across
them.

The alternative is array-of-structs: one array of whole rows, each row
contiguous. The two layouts differ in what one memory read brings back. A pass
that reads one field of many rows wants struct-of-arrays. Every byte the cache
line carries is then a byte the pass uses. A pass that reads many fields of one
row wants array-of-structs. The reason is the same, in reverse.

Neither layout is better. The pass decides, and the number of fields the pass
touches is what decides it.

**The project has already made this mistake once, and it cost a correction.** A
register row recorded that "the character pass is a random graph gather", and
therefore that the character tier wants array-of-structs.[^2] A reader took
that to govern the character column set. It does not. The measurement behind
the row covers a separate personality trait record. Its pass reads twelve
fields of one row.[^3] The research on descent and succession recommends the
opposite layout for the character columns.[^4] [^5] The two recommendations sit
in one corpus and do not contradict each other. They describe two different
structures.

The error was not in the measurement. It was in the scope of the sentence. A
claim written over a tier reaches every structure the tier holds, and a tier
holds structures with different access patterns.

A tier is a declaration about which entities expose an object model, and it is
made when the entity is created.[^6] It says nothing about how any field is
laid out. Nothing forces the structures inside one tier to share an access
pattern, and here they do not.

Until now nothing bound this. A register row records a correction and a
finding records the misattribution, and neither instrument binds a
contributor.[^2] [^3]

**The descent and succession pass now exists**, so this record points at a
pass rather than at an intent.[^7] Its kernels are column passes. A walk of the
father forest reads one parent column. A cadet split maps over a span of an
order array. A house query filters one house column. Two operations gather at
random: the ancestor walk and the relation recursion. Each reads two or three
columns for a node.

## Decision

### D1. A layout claim names the structure it governs and the pass that decides it

A statement about how a field is laid out is not admissible unless it names
both. Some claims name a subject wider than one structure: a tier, a subsystem, a
layer. Such a claim states nothing that a reviewer can check. The wider subject
holds structures whose passes disagree.

A reviewer rejects a layout claim that names a tier. A reviewer rejects a
layout claim that names no pass.

This binds prose as much as code. The register row that caused the correction
was prose, it was read as authoritative, and nothing failed.

### D2. The column count of the deciding pass is the evidence, not the name of the entity

To choose a layout, count the fields that the pass touches for each item it
visits. A pass that touches few fields of many items takes struct-of-arrays. A
pass that touches many fields of one item takes array-of-structs.

The count is the argument. The kind of entity is not, and neither is the name
of the tier it sits in. A gather benchmark measures the crossover as a
function of the column count, and the reference tables hold what was
measured.[^8]

**The crossover figure is not in this record.** A measured figure changes when
a better measurement arrives. A decision does not.[^9] No measurement exists on
the target platform, and one blocker holds every cost figure here.[^10]

### D3. The living character columns are struct-of-arrays, and the record of descent is too

Both structures are laid out as one array for each field.

The pass that decides it is descent and succession. Every kernel of it reads
one or two columns for each row it visits. The two operations that gather at
random read two or three.[^4] [^5] That is the low end of the column count.
Struct-of-arrays wins at that end.

Two consequences follow that a row layout would forfeit. The Python control
plane takes a zero-copy view of one column, which a row layout cannot offer
without a copy.[^11] And a column that a pass does not read is never brought
into the cache at all.

**This record makes no claim about the personality trait record.** That
structure holds many values for one character and its pass reads all of
them.[^3] It is a separate structure with a separate pass, and D1 requires it
to be decided separately, by whoever writes it.

### D4. A hybrid split needs the pass before it may be proposed

A hybrid layout puts the fields that one pass reads together into a row and
leaves the rest as columns. It is admissible, and it is not admissible in
advance.

The split is exact only if the set of fields the pass reads is known. When it
is not, the split declares the same field membership in two places: the struct,
and the pass. Nothing fails when the two drift apart.[^12] Propose a hybrid
after the pass is written and measured, never before.

## Consequences

**A layout choice is now reviewable.** A reviewer asks two questions: which
structure, and which pass. A proposal that cannot answer both is refused
without argument about the layout itself.

**A contributor cannot cite a tier to move a layout.** The register row that
started this remains true about the structure it measured, and it no longer
reaches the character columns.

**The project accepts the gather cost of the ancestor walk and the relation
recursion.** Those two operations read two or three columns for each node, so
they pay more cache lines than a row layout would. The column passes are the
majority of the work and they pay less. Nothing here claims the trade is
optimal at the target scale, because nothing has been measured there.

**A subtree of the father forest is contiguous in the Euler order array, and
not in the house column.** The research describes a cadet split as a
contiguous range write.[^13] The read is contiguous: the split scans one span
of the Euler order array. The write is not: it gathers into the house column,
because the record stores its rows in birth order.

**Name the array, or the word "contiguous" says nothing.** This is the same
error this record forbids, in miniature. A claim about locality is checkable
only against one named array, in the same way a claim about layout is
checkable only against one named structure and one named pass. Storing the
rows in Euler order would make the write contiguous too, and would move every
row whenever the labels are rebuilt.

**Nothing in this record fixes where a structure lives.** It states how a
layout is chosen and what a layout claim must name. A module arrangement is
not a constraint, and a record that held one would die when the modules
moved.[^14]

## Alternatives rejected

**Array-of-structs for the character columns.** Rejected on the column count.
It charges every column pass a whole row read, to serve the two operations
that gather. Those two read two or three columns. The measurement covered
twelve.[^3] [^4] It also removes the
zero-copy column view.[^11]

**One layout for the whole tier.** Rejected because the tier holds structures
whose passes disagree. This is the claim that caused the correction.[^3]

**A hybrid row of the hot descent fields.** Rejected for now under D4. The
split cannot be exact while the field set is unsettled, and an inexact split
declares one fact at two sites.[^12]

**Union-find as the authoritative house index.** Rejected because it cannot
split, and a cadet branch leaving its parent house is a split.[^13] The house
is an explicit column instead. Union-find remains correct for the one-off
grouping pass at world generation, which never splits.

**A closure table for ancestry.** Rejected on arithmetic in the research: the
ancestor count of one character grows as a power of the generation depth, and
the table does not fit.[^15]

## References

[^1]: ADR-0066, entity storage holds four fixed shapes, decisions D1 and D3. `docs/adrs/accepted/adr-0066-entity-storage-holds-four-fixed-shapes.md`
[^2]: Findings register, FND-022. `docs/FINDINGS.md`
[^3]: Findings register, FND-072. `docs/FINDINGS.md`
[^4]: The character graph and inheritance, sections 2.1 and 3.2. `docs/research/reports/14-character-graph-and-inheritance.md`
[^5]: Decisions register, DEC-032. `docs/DECISIONS.md`
[^6]: ADR-0054, an entity belongs to one of three tiers, declared at creation, decision D1. `docs/adrs/accepted/adr-0054-an-entity-belongs-to-one-of-three-tiers-declared-at-creation.md`
[^7]: Backlog item 0097, write the layout record with the descent columns. `docs/backlog/complete/0097-write-the-layout-record-with-the-descent-columns.md`
[^8]: Budgets and costs. `docs/reference/budgets.md`
[^9]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^10]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^11]: ADR-0012, tiles are dense columns and units are a generational arena, decision D3. `docs/adrs/accepted/adr-0012-tiles-are-dense-columns-and-units-are-a-generational-arena.md`
[^12]: Recurring defect shapes, shape 1. `.claude/rules/recurring-defects.md`
[^13]: The character graph and inheritance, sections 3.3 and 3.5. `docs/research/reports/14-character-graph-and-inheritance.md`
[^14]: Decision Record Scope, section 4.4. `.claude/rules/adr-scope.md`
[^15]: The character graph and inheritance, section 3.1. `docs/research/reports/14-character-graph-and-inheritance.md`
