# Review of ADR-0096: the corrections

This document holds the corrections that a review returned to the author of one
architecture decision record.[^1] It is a review artefact and it is not a
record. It holds counts, file positions and quoted text freely, because none of
that material binds anything.

**The verdict is Return.** Three defects block the record. Two more corrections
are required. One imprecision is not blocking, and it is written down here
because an imprecision that nobody writes down stays in the record.

The record keeps its status. A reviewer returned it, so the registry still
reads `Draft` and the review set nothing.[^2]

**Apply the corrections in the order below.** Correction 4 repoints an existing
footnote, and corrections 1 and 3 add markers beside it. The last section states
what the numbering does.

## What the review checked against the source

The author claimed that the choice pass obeys the accepted record on parallel
stages and floors anyway. The review checked that claim against the code and
found it accurate in every part.[^3]

The pass collects the live units with `self.soldiers.iter().collect()` before
the thread scope opens. That is serial. Each thread reads four values for each
unit: the tile of the unit, the cell of that tile, the summary of that cell, and
the need of the unit from a column. Every one of those reads is keyed on the
arena slot. The pass then applies its results with a serial loop that calls
`set_intent_at` for each entry.

The pass keeps all three decisions of the parallel-stage record. Each thread
owns one output slot and the inputs are shared reads. The join reads the slots
in slot order. The chunks are contiguous and come from the data and the thread
count.[^4] [^5] [^6]

The need column is `&[Fix32]`.[^7] That single fact is what breaks D4, and the
next section states why.

The review took three things on trust. The first is that the unit arena never
compacts; the review read two records that state it and did not read the arena.
The second is every figure in the target platform table. The third is that one
weight profile serves every unit alive; the review read the finding and not the
source.[^8]

## Correction 1. Blocking. D4 names a key that reduces nothing

**The defect.** D4 names the pair of the cell and the need as the key of the
computed answer. A need is a fixed-point quantity, so it takes far more values
than a world holds units. In a world that consumes, two units in one cell almost
never hold the same need. The distinct pairs then equal the population, and D4
reduces no work at all. D4 therefore cannot deliver D1.

The reference table states the same thing in its own words: the bucket width is
the mechanism and not a detail of it, and unbucketed the collapse is 1.[^9]

**The record holds no figure, and it must not gain one.** A search of the record
returns no numeral outside record numbers, decision labels and level names. The
collapse ceiling in the reference table is a measured figure bounded above by a
parameter that nobody has chosen, and the scope rule keeps such a figure out of
a record body.[^10] The correction adds the mechanism and a parameter. It adds
no number.[^11]

**Replace this paragraph:**

```markdown
The engine holds one weight profile for every unit alive, and no unit carries a
type or a profile of its own, so today the inputs to a choice are the unit's
cell and the unit's need and nothing else.[^11] A value computed for each
distinct pair is computed once for every unit that shares it.
```

**With this text:**

```markdown
The engine holds one weight profile for every unit alive, and no unit carries a
type or a profile of its own, so today the inputs to a choice are the unit's
cell and the unit's need and nothing else.[^11]

**The need enters the key as a bucket, and the width of the bucket is the
mechanism and not a detail of it.** A need is a fixed-point quantity, so it
takes far more values than a world holds units. Unbucketed, two units in one
cell almost never share a need, the distinct pairs equal the population, and
this decision buys nothing.

**The width is a parameter, and this record does not set it.** No measurement
exists of how many need values coexist in one cell in a world that consumes,
and a blocker governs every cost figure this project holds.[^3] A wide bucket
makes two units of different need act alike. A narrow one approaches one answer
for each unit. The item that implements this decision carries that choice.[^17]

A value computed for each distinct pair of a cell and a need bucket is computed
once for every unit that shares it.
```

The marker `[^17]` in the new text is the backlog item that implements the
decision. That item already holds the bucket question in its own open list, so
the citation lands correctly today.[^12] The marker `[^3]` must point at the
blockers register after correction 4 moves it. Read the last section.

## Correction 2. Blocking. The record claims a citation that no file holds

**The defect.** The consequences say that both existing field records cite this
one. Neither does.

**This is the inert-capability shape, applied to a record rather than to code,
and the review agrees with that framing.** The shape is that a project declares
a capability, documents it, and never calls it.[^13] Here a record declares a
relationship with the tree, and nothing in the tree instantiates it. The scope
rule already names the record-level form of the shape, in the section that says
a record must not state an intent as if it were a fact. That section came from a
reference record which claimed that a declared list of telemetry keys described
what a library emits, when several of the keys had no write site.[^14] The two
cases are the same case.

**One thing differs, and it favours this project.** Inert code passes its own
test, so nothing catches it. This claim is caught already: the record check
reports that no other record and no source file cites ADR-0096.

**The evidence, two ways.** A search of the draft directory for the string
`0096` returns the record's own file and nothing else. The record check reports
`note: ADR-0096 is cited by no other record and no source file.` The commands
are in the last section.

**Replace this paragraph:**

```markdown
**A subsystem record about a field is now an instance and not a precedent.**
Both existing field records cite this one, and a third subsystem that needs a
field cites this one rather than copying either of them.[^8] [^9]
```

**With this text:**

```markdown
**A subsystem record about a field is an instance of this record and not a
precedent.** A third subsystem that needs a field cites this record, rather
than copying either of the two that exist.[^8] [^9] **Neither of those two
cites this record today.** Both were written before it, and both are drafts,
so a later edit may add the citation. This record does not claim the citation
exists.
```

## Correction 3. Blocking. D4 calls a closed decision open

**The defect.** D4 ends by saying that the repair of a stale consequence is an
open question. That decision closed in this review. The register now holds the
outcome, and the registry holds the rule beside the citation rule.[^15] [^2]

This is the shape that the definition of done warns about: a record written
under an open question states a false thing the moment the question closes, and
nothing fails, because a record is prose.[^16]

**Replace this sentence:**

```markdown
That sentence is a consequence the record derived, not a decision it made, and
how the project repairs a stale consequence inside an accepted record with
dependents is an open question that this record does not settle.[^12]
```

**With this text:**

```markdown
That sentence is a consequence the record derived, not a decision it made. A
closed decision now says that the project strikes such a sentence in place
rather than superseding the record, and the registry holds the rule.[^12]
[^NEW] The strike belongs to the commit that accepts this record.
```

Define the new marker as the registry section on repairing a derived
consequence.[^2] Keep `[^12]` pointing at the decisions register, because the
row is still there and it now holds the outcome.

## Correction 4. Required. One footnote sends a claim to the wrong register

**The defect, in two parts.** The context says that a reference register holds
the measurements with their derivation, and it sends the reader to the blockers
register. The rows live in the target platform table.[^9] The rows are also
measured and not derived, and this project holds that distinction as a rule.

**Replace this sentence:**

```markdown
The measurements are on the target platform, and the reference register holds
them with their derivation.[^3]
```

**With this text:**

```markdown
The measurements are on the target platform. A reference table holds every row
with the machine and the commit that produced it.[^3] One blocker still governs
the cost figures that no benchmark reached.[^NEW]
```

Repoint `[^3]` at the target platform table. Define the new marker as the
blockers register.[^17]

**Add one citation in the same pass.** The context of the record reproduces the
evidence of a finding almost sentence for sentence, and cites it nowhere.[^18]
Put a marker after the sentence that reads "It scales poorly anyway, and the
reasons are visible in the source rather than in a profile." One fact in two
places, with nothing that fails when the copies disagree, is the first recurring
defect shape.[^19]

## Correction 5. Not blocking. The apply loop walks a different list

**The defect.** The context says the pass applies the results by walking the
collected list again. It walks the results, which are the subset whose cell
chose on this frame. Both are serial and both grow with the population, so the
sentence that follows stays true.

**Replace this clause:**

```markdown
and it applies the results by walking that list again afterwards
```

**With this text:**

```markdown
and it applies the results by walking them afterwards
```

The word "again" goes with "that list", because the pass does not walk the
collected list a second time.

**The same wording is in the finding that supports the record.**[^18] That is a
register edit and it belongs to whoever owns the findings register. This review
did not make it.

## What the footnote numbering does

Correction 4 changes where `[^3]` points and adds one marker beside it.
Corrections 1 and 3 each add one marker. The documentation rule numbers the
footnotes in the order that the markers occur in the body, so renumber the
whole list once, after every correction is applied.[^20] The footnote check
reports an out-of-order list without failing, so the check will not catch a
missed renumber.

The dependency column of the registry row is also short. It names three
records, and the record cites six.[^2] Correct the row when the text is
final.

## The commands that produced the evidence

```
grep -rn "0096" docs/adrs/draft/
python3 scripts/check_adrs.py
grep -rn "DEC-096" . --exclude-dir=.git
```

The first returns the record's own file and nothing else. The second reports 63
records, 0 failures and 14 notes, and one of the notes names ADR-0096. The third
returned three files outside the decisions register when the review began. The
review repaired two of them. The third is a backlog item, which belongs to
another owner.[^21]

## References

[^1]: ADR-0096, cost follows the lattice, not the population, and a unit is a reader. `docs/adrs/draft/adr-0096-cost-follows-the-lattice-not-the-population.md`
[^2]: ADR Registry. `docs/adrs/REGISTRY.md`
[^3]: The choice pass of the world. `crates/cachette-core/src/world.rs`
[^4]: ADR-0009, parallel stages write disjoint outputs, decision D1. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^5]: ADR-0009, parallel stages write disjoint outputs, decision D2. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^6]: ADR-0009, parallel stages write disjoint outputs, decision D3. `docs/adrs/accepted/adr-0009-parallel-stages-write-disjoint-outputs.md`
[^7]: The need column of the soldier arena. `crates/cachette-core/src/soldier.rs`
[^8]: Findings register, FND-251. `docs/FINDINGS.md`
[^9]: Target platform costs, the collapse of the choice pass. `docs/reference/graviton-costs.md`
[^10]: Decision Record Scope, section 4.1. `.claude/rules/adr-scope.md`
[^11]: Decision Record Scope, section 4.5. `.claude/rules/adr-scope.md`
[^12]: Backlog item 0238. `docs/backlog/complete/0238-decide-per-cell-and-need-rather-than-per-unit.md`
[^13]: Recurring Defect Shapes, shape 3. `.claude/rules/recurring-defects.md`
[^14]: Decision Record Scope, section 4.6. `.claude/rules/adr-scope.md`
[^15]: Decisions register, DEC-096. `docs/DECISIONS.md`
[^16]: Definition of Done, section 4. `.claude/rules/definition-of-done.md`
[^17]: Blockers register, BLK-007. `docs/BLOCKERS.md`
[^18]: Findings register, FND-252. `docs/FINDINGS.md`
[^19]: Recurring Defect Shapes, shape 1. `.claude/rules/recurring-defects.md`
[^20]: Documentation Rules, section 3. `.claude/rules/documentation.md`
[^21]: Backlog item 0236. `docs/backlog/complete/0236-repair-every-record-that-calls-blk-007-open.md`
